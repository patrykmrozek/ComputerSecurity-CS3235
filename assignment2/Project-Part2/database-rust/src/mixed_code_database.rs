/* ------mixed_code_database.rs------
 *
 * changed UserInfoT to own the strings instead of borrowing w lifetimes
 * this allows for the removal of Box::leak()
 * changed enqueue_user() to take in owned string
 * added OwnershipType to track who owns each user
 * changed constants in database_fix_full.rs to match same in C
 * made sure the day counter references the actual dc and that it doesnt get dropped early by defining it above struct field assignments
 * re-enabled join_databases() - there is a double free error now, most likely due to shared pointers between Rust and C
 * added ownership field to UserStruct
 *
 *
 * ------database_enhanced.c------
 *
 * added null checks to free()
 * removed cleanup_database() from print_database()
 * ensured bounds checking in copy_string() with min_len
 * in create_user() used constants instead of strlen() in copy_string()
 * find_by_userid() had an off by 1 with the for loop having <= db->count
 * removed unnecessary dupliacte sizeof() in get_user_reference_for_debugging()
 * set pointers to NULL after free in memory_pressure_cleanup() and merge_duplicate_handles()
 * made sure generate_token() uses MAX_SESSION_TOKEN_LEN in copy_string()
 * changed all of the sizes in clone_user()s copy_string() calls to be constants
 * made pointer NULL after freeing in update_database_daily
 * in memory_pressure_cleanup() db->users[i] is NULL, so must first malloc() before cloning user there
 * fixed an unitialized pointer user** in get_reference_for_debugging()
 * rewrote memory_pressure_cleanup() - just using two indices one iterating through each block of memory,
 * one keeping track of where there is empty space where memory can be compacted and written to
 * added bounds check to add_user()
 * corrected indices in merge_duplicate_handles() from j < i-1 to j < i
 * added memory_pressure_cleanup() after merging duplicates in merge_duplicate_handles()
 * changed memset() in init_session_manager() from sizeof(UserStruct_t) to sizeof(UserStruct_t*)
 * init_database() now recognizes Rusts pointer to the global_day counter as to avoid later uninitialized pointer deref
 * since the pointer is on Rusts heap, when Rust updates, C reads the updated value - they both access same location
 * (currently after getting both dbs to print successfully, they are not identical - some issued with C/Rust db functions)
 * (when trying with -gen 30, no longer works - some seg fault)
 * seems like its usually crashing around the 25th user added - when current db->count is 26
 * started keeping track of the amount of users removed and decrementing the db->count accordingly
 * added debug statement for current day - seems the program is crashing on day 8 (when memory_pressure_cleanup() gets called)
 * after more debugging the error was in merge_duplicate_handles() - specifically, i needed to add a null user check for each db->users[i]
 * but this led to another seg fault, now occuring  after [INFO] Processing Logins
 * seems like this error is now back on the rust side
 * after further debugging, it seems that the root of the problem is rust calling C's find_user_by_username() function
 * find_user_by_username() needed a null check for db->users[i] - solved this seg fault
 * next seg fault is occurring at the --C Backend Database State-- stage
 * this error was resolved by adding another check for db->users[i] being null in print_database() in C
 * added the OwnershipType field to the C UserStruct - planning on checking when freeing to make sure it is C_OWNED
 * added checks to free_user to make sure the user is a SHARED_C_PRIMARY or C_OWNED user
 * added additional debug prints to free_user()
 *
 *
 * ------database_wrapper.rs------
 *
 * stopped using the unsafe cast from UserStruct to UserStructT and
 * instead manually copied over values from each field of the UserStruct to the UserStructT
 * did the same thing with get_all_user_references()
 * the original code had a pointer to a C struct, and it was cast to a rust struct pointer
 * it was directly taking Cs memory and giving it to Rust to manage
 * so instead we borrow C struct temporarily, create a comletely new struct in Rust's memory
 * copy and convert each field from C to Rust, Box the new struct - all of this lets C keep ownership
 * ^^^ I realized later that these changes violated the rules stated in rules.md so the implementations were commented out^^^
 *
 * added ownership tracking in the sync_users_from_rust() function
 * checked c_users ownership status in get_all_user_references() - if SHARED_C_PRIMARY, convert to BOX as it originated from Rust
 * if C_OWNED just continue
 * added check in sync_user_from_rust_db() to see if user already shared to avoid adding duplicates
 *
 *
 * ------database_fix_full.rs------
 *
 * added an extra check for null user in update_database_daily()
 * the following line in add_user() seems to be the problem: (*db).users[index]
 *
*/

mod database_fix_full;
mod database_wrapper;
mod generated_data;

struct UserEntry {
    email: Option<String>,
    username: String,
    password: String,
    id: Option<i32>,
}

struct DayData {
    day: i32,
    logins: Option<Vec<UserEntry>>,
    signups: Option<Vec<UserEntry>>,
}

const MAX_USERS: usize = 1000;
const SESSION_TOKEN_MAX_LEN: usize = 32;
const MAX_PASSWORD_LENGTH: usize = 100;

use database_fix_full::{
    add_user, create_user, find_user_by_username, find_user_by_username_mut, update_database_daily,
    UserDatabase, UserStruct,
};
use database_wrapper::{
    initialize_enhanced_database, DatabaseExtensions, UserReference, UserStructT,
};

/*
pub struct UserInfoT<'a> {
    // Add lifetime parameter
    email: &'a str,
    username: &'a str,
    password: &'a str,
}
*/

pub struct UserInfoT {
    email: String,
    username: String,
    password: String,
}

pub struct EnhancedStudentDatabase {
    rust_db: Box<UserDatabase>,
    c_extensions: DatabaseExtensions,
    user_references: Vec<UserReference>,
    session_tokens: Vec<String>,
    pending_requests: Vec<UserInfoT>,
    _day_counter: Box<i32>,
    c_allocated_users: Vec<i32>,
}

pub fn str_cmp(a: &[u8], b: &str) -> bool {
    let a_str = std::str::from_utf8(a).unwrap_or("");
    a_str.trim_end_matches(char::from(0)) == b
}
pub fn bytes_to_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
pub fn string_to_bytes(s: String) -> [u8; SESSION_TOKEN_MAX_LEN] {
    let mut byte_array = [0u8; SESSION_TOKEN_MAX_LEN];
    let bytes = s.as_bytes();
    let len = bytes.len().min(SESSION_TOKEN_MAX_LEN - 1);
    byte_array[..len].copy_from_slice(&bytes[..len]);
    byte_array[len] = 0;
    byte_array
}

impl EnhancedStudentDatabase {
    /// Initialize a new enhanced database instance
    pub fn new() -> Self {
        println!("Initializing Enhanced Student Database System...");
        let dc = Box::new(0);
        let c_extensions = initialize_enhanced_database(&*dc);

        let std_b = EnhancedStudentDatabase {
            rust_db: database_fix_full::init_database(),
            user_references: Vec::new(),
            session_tokens: Vec::new(),
            pending_requests: Vec::new(),
            _day_counter: dc,
            c_extensions,
            c_allocated_users: Vec::new(),
        };
        std_b
    }
    pub fn enqueue_user(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(), String> {
        let user_info = UserInfoT {
            email,
            username,
            password,
        };
        self.pending_requests.push(user_info);
        Ok(())
    }

    // Read Only : Dont Change
    pub fn sync_database(&mut self) {
        //Signup all pending users
        let drained_users: Vec<_> = self.pending_requests.drain(..).collect();
        for (_i, user) in drained_users.iter().enumerate() {
            let pending_count = drained_users.len() - _i;
            println!("[RUST] ADDING USER WITH SYNC: {:?}", user.username);
            let _ =
                self.add_user_with_sync(&user.username, &user.email, &user.password, pending_count);
        }
    }
    // Read Only : Dont Change
    pub fn activate_user(&mut self, user_name: &str) {
        // println!("Activating user with ID: {}", user_id);
        database_fix_full::user_login(&mut self.rust_db, user_name);
    }
    // Read Only : Dont Change
    pub fn add_user_with_sync(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
        pending_count: usize,
    ) -> Result<(), String> {
        // Intelligent load balancing - use C allocator when under pressure
        println!("[RUST] ADD_USERS_WITH_SYNC");
        if pending_count > 5 || self.rust_db.count >= MAX_USERS as i32 {
            println!(
                "[System] High load detected, using optimized C allocator for user {}",
                username
            );
            println!("[RUST] SYNCING USER TO C BACKEND");
            self.c_extensions
                .sync_user_to_c_backend(username, email, 0, password)?;
            let id = self.c_extensions.get_last_user_id();
            self.c_allocated_users.push(id);
            return Ok(());
        }

        println!("[RUST] CREATING USER: {:?}", username);
        let user = create_user(username, email, 0, password);
        println!("[RUST] ADDING USER: {:?}", username);
        add_user(&mut self.rust_db, user);

        println!(
            "[System] Added user {} using dual allocation strategy",
            username
        );
        Ok(())
    }

    pub fn find_user_by_name<'a>(
        &self,
        db: &'a UserDatabase,
        username: &str,
    ) -> Option<&'a UserStruct> {
        for i in 0..db.count as usize {
            if let Some(ref user) = db.users[i] {
                if str_cmp(&user.username, username) {
                    return Some(user);
                }
            }
        }
        None
    }
    fn update_user_session_token(&mut self, user_name: &str, token: String) {
        if let Some(user) = find_user_by_username_mut(&mut self.rust_db, user_name) {
            user.session_token = string_to_bytes(token.clone());
            if !self.session_tokens.contains(&token) {
                self.session_tokens.push(token);
            }
        }
    }
    /// Read Only: Dont Modify Authenticate user and create session
    pub fn login_user(&mut self, user_name: &str, password: &str) -> Result<String, String> {
        println!("[RUST] LOGIN USER CALLED FOR USER: {:?}", user_name);
        if self.find_user_by_name(&self.rust_db, user_name).is_none() {
            println!("[RUST] USER NOT FOUND IN RUST DB - EXISTS IN C DB\n");
            // User found in C backend cache
            for user_ref in self.user_references.iter_mut() {
                println!("[RUST] ACCESSING USER REFERENCE: {:?}", user_ref.username);
                if str_cmp((*user_ref).username.as_bytes(), user_name) {
                    println!(
                        "USERNAMES MATCH: {:?} vs {:?}",
                        (*user_ref).username.as_bytes(),
                        user_name
                    );
                    if self.c_extensions.get_user_password(user_ref.ptr) != password {
                        return Err("Incorrect password".to_string());
                    }
                    println!("[RUST] RESETTING USER REFERENCE LOGIN FIELDS!");
                    unsafe {
                        (*user_ref.ptr).inactivity_count = 0;
                        (*user_ref.ptr).is_active = 1;
                    }
                    let session_token = self.c_extensions.create_session_for_c_ptr(user_ref.ptr)?;
                    return Ok(session_token);
                }
            }

            println!("[RUST] FETCHING USER FROM C BACKEND");
            let user = self.c_extensions.get_user_in_c_backend(user_name);
            println!("[RUST] USER FROM C BACKEND: {:?}", user);
            if user == std::ptr::null_mut() {
                return Err("User not found in any backend".to_string());
            }

            self.user_references
                .push(UserReference::new(String::from(user_name), user));

            let user_password = self.c_extensions.get_user_password(user);

            if user_password == password {
                return self.c_extensions.login_user(user_name);
            } else {
                return Err("Incorrect password".to_string());
            }
        } else {
            let user = find_user_by_username(&self.rust_db, user_name).unwrap();
            if str_cmp(&user.password, password) {
                // println!("User[{}] {} logged in successfully", user.user_id, user_name);
                let session_token = self.c_extensions.create_session(user)?;
                self.update_user_session_token(user_name, session_token.clone());
                self.activate_user(user_name);
                return Ok(session_token);
            } else {
                return Err("Incorrect password".to_string());
            }
        }
    }

    // Read Only : Dont Change
    pub fn join_databases(&mut self) {
        //Creating shared handles for all users in Rust DB
        print!(
            "[Info] Creating shared handles for {} rust users\n",
            (*self.rust_db).count
        );
        // Sync all users from Rust DB to C backend
        // iterates over rust_db.users for rust_db.count iterations
        for user_opt in self.rust_db.users.iter().take(self.rust_db.count as usize) {
            if user_opt.is_none() {
                continue;
            }
            if let Some(user) = user_opt {
                let user_ptr = {
                    let ptr = std::ptr::addr_of!(**user);
                    ptr as *mut UserStructT
                };
                println!("[RUST:JOIN_DATABASES] SYNCING USERS FROM RUST DB");
                self.c_extensions.sync_user_from_rust_db(user_ptr);
            }
        }

        // Now perform the complementary sync from C backend to Rust DB
        println!("[Info] Syncing all user references from C backend...");

        // Get pointer references for C users and extend local references
        println!("[RUST:JOIN_DATABASES]] GETTING ALL USER REFERENCES FROM C");
        let all_c_userstructs = self.c_extensions.get_all_user_references();
        // add all users in this vector to rust db
        println!("[RUST:JOIN_DATABASES]] ADDING ALL USERS FROM C BACKEND");
        for user in all_c_userstructs {
            println!(
                "[RUST:JOIN_DATABASES]] ADDING: {:?} FRON C BACKEND TO RUST DB",
                user.username
            );
            add_user(&mut self.rust_db, user);
        }
    }

    pub fn validate_active_user_session(&self) {
        // Take all users in this database and validate their sessions in C backend
        for user in self.rust_db.users.iter().take(self.rust_db.count as usize) {
            if let Some(u) = user {
                if u.is_active == 1 {
                    let _ = self
                        .c_extensions
                        .validate_session(bytes_to_string(&u.session_token).as_str());
                }
            }
        }
    }
    //Read Only : Dont Change
    pub fn increase_day(&mut self) {
        //Resolve all signup requests
        println!("[RUST] SYNCING DB!\n");
        self.sync_database();
        // Increment the day counter
        *(self._day_counter) += 1;
        // Validate active user sessions
        println!("[RUST] VALIDATING USER SESSION!\n");
        self.validate_active_user_session();
        // Update rust database (uses the function you translated for Part 1)
        println!("[RUST] UPDATING DB!\n");
        update_database_daily(&mut self.rust_db);
        // Every 5 days, join the two databases

        if *(self._day_counter) % 5 == 0 {
            println!("[RUST] JOINING DATABASES!");
            self.join_databases();
        }

        // Perform daily updates on C backend
        println!("[RUST] CALLING C SIDE INCREMENTATION");
        self.c_extensions.increment_day(&self.rust_db);
    }

    pub fn print_both_databases(&self) {
        println!("---------------------------------C Backend Database State --------------------------------");
        self.c_extensions.print_database_full();
        println!("------------------------------------Rust Database State ----------------------------------");
        database_fix_full::print_database(&self.rust_db);
    }
}

fn main() {
    println!("=======Mixed Code Student Database System========");

    let mut db = EnhancedStudentDatabase::new();

    // Initialize with static data
    let days_data = generated_data::get_days_data();
    // Process each day's activities
    for day_data in days_data.iter() {
        let mut local_session_tokens: Vec<String> = Vec::new();

        println!(
            "============================[Info] Processing day {}===========================",
            day_data.day
        );

        if let Some(signups) = &day_data.signups {
            println!("=========[Info] Processing Signups============");
            for signup in signups {
                let username = signup.username.clone();
                let email = signup
                    .email
                    .clone()
                    .unwrap_or_else(|| "no-email@default.com".to_string());
                let password = signup.password.clone();

                /*
                match db.enqueue_user(
                    Box::leak(username.into_boxed_str()),
                    Box::leak(email.into_boxed_str()),
                    Box::leak(password.into_boxed_str()),
                )
                */

                match db.enqueue_user(username, email, password) {
                    Ok(_) => println!("[Signup] Queued user: {}", signup.username),
                    Err(e) => println!(
                        "[Signup Error] Failed to queue user {}: {}",
                        signup.username, e
                    ),
                }
            }
        }

        if let Some(logins) = &day_data.logins {
            println!("=========[Info] Processing Logins============");
            for login in logins {
                let password = login
                    .password
                    .clone()
                    .chars()
                    .take(MAX_PASSWORD_LENGTH - 1)
                    .collect::<String>();

                // Attempt user login
                println!("[RUST] ATTEMPTING USER LOGIN!\n");
                match db.login_user(&login.username, &password) {
                    Ok(session_token) => {
                        println!("[Login] User {} logged in successfully", login.username);
                        local_session_tokens.push(session_token);
                    }
                    Err(e) => {
                        println!(
                            "[Login Error] Failed to login user {}: {}",
                            login.username, e
                        );
                    }
                }
            }
        }
        println!("========[Info] Performing end-of-day updates========");
        db.increase_day();

        println!(
            "=====[Info Day {}] Total Site traffic on Rust DB = {}======",
            day_data.day,
            local_session_tokens.len()
        );
    }

    println!("\n====================Congratulations! End of Simulation====================\n");

    db.print_both_databases();

    println!("\n==========================Did you really fix it ?======================================\n");
}
