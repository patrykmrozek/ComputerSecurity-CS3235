use crate::database_fix_full::OwnershipType;
use crate::database_fix_full::{UserDatabase, UserStruct};
use crate::{MAX_PASSWORD_LENGTH, SESSION_TOKEN_MAX_LEN};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::usize::MAX;

const MAX_USERS: usize = 1000;
const MAX_NAME_LEN: usize = 50;
const INACTIVITY_THRESHOLD: i32 = 5;
const MAX_EMAIL_LEN: usize = 50;
const MAX_SESSION_TOKEN_LEN: usize = 32;

// C struct representations
#[repr(C)]
pub struct UserStructT {
    pub password: [c_char; MAX_PASSWORD_LENGTH],
    pub username: [c_char; MAX_NAME_LEN],
    pub user_id: c_int,
    pub email: [c_char; MAX_EMAIL_LEN],
    pub inactivity_count: c_int,
    pub is_active: c_int,
    pub session_token: [c_char; MAX_SESSION_TOKEN_LEN],
    pub ownership: OwnershipType,
}

#[repr(C)]
pub struct UserDatabaseT {
    pub users: [*mut UserStructT; MAX_USERS],
    pub count: c_int,
    pub capacity: c_int,
}
/*
pub struct DatabaseOperationResult {
    success: i8,
    requires_dealloc: i8,
    ownership_transfer: i8,
    updated_ptr: *mut UserStructT,
}
*/

extern "C" {
    fn init_database(dc: *const i32) -> *mut UserDatabaseT;
    fn create_user(
        username: *const c_char,
        email: *const c_char,
        user_id: c_int,
        password: *const c_char,
    ) -> *mut UserStructT;
    fn add_user(db: *mut UserDatabaseT, user: *mut UserStructT);
    // fn find_user_by_id(db: *mut UserDatabaseT, user_id: c_int) -> *mut UserStructT;

    // Session management
    pub fn create_user_session(user: *const UserStructT) -> *mut c_char;
    fn validate_user_session(token: *const c_char) -> c_int;

    // Memory management and optimization
    fn get_user_reference_for_debugging(db: *mut UserDatabaseT) -> *mut *mut UserStructT;

    // Additional C functions present in database_enhanced.c
    fn print_database(db: *mut UserDatabaseT);
    fn update_database_daily(db: *mut UserDatabaseT);
    fn user_login(db: *mut UserDatabaseT, user_name: *const c_char) -> *const c_char;
    fn get_password(db: *mut UserDatabaseT, user_name: *const c_char) -> *const c_char;
    fn get_non_null_ref_count(db: *mut UserDatabaseT) -> c_int;
    fn find_user_by_username(db: *mut UserDatabaseT, user_name: *const c_char) -> *mut UserStructT;
    fn deactivate_users(db: *mut UserDatabaseT);
    fn init_session_manager();
    fn update_day_counter(dc: *const i32);
}

/*
//function to transform into a C array while null terminating to avoid buffer overflow
fn to_c_array<const N: usize>(src: &[u8; N]) -> [c_char; N] {
    let mut arr = [0 as c_char; N];
    let mut len = 0;

    while len < N - 1 {
        let temp = src[len];
        if temp == 0 {
            break;
        }
        arr[len] = temp as c_char;
        len += 1;
    }

    arr[len] = 0;
    return arr;
}

fn to_rust_array<const N: usize>(src: &[c_char; N]) -> [u8; N] {
    let mut arr = [0u8; N];
    let mut len = 0;

    while len < N - 1 && src[len] != 0 {
        arr[len] = src[len] as u8;
        len += 1;
    }

    return arr;
}
*/

pub struct UserReference {
    pub username: String,
    pub ptr: *mut UserStructT,
}

impl UserReference {
    pub fn new(username: String, ptr: *mut UserStructT) -> Self {
        UserReference { username, ptr }
    }
}

pub struct DatabaseExtensions {
    db: *mut UserDatabaseT,
}

impl DatabaseExtensions {
    pub fn new(dc: *const i32) -> Self {
        let db = unsafe { init_database(dc) };
        unsafe {
            init_session_manager();
        }
        DatabaseExtensions { db }
    }
    pub fn get_user_password(&self, user: *mut UserStructT) -> String {
        unsafe {
            let password_ptr = get_password(self.db, (*user).username.as_ptr());
            CStr::from_ptr(password_ptr).to_string_lossy().to_string()
        }
    }
    pub fn get_user_in_c_backend(&self, username: &str) -> *mut UserStructT {
        println!("[RUST] GET_USER_IN_C_BACKEND() - username: {:?}", username);
        let c_username = match CString::new(username) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        println!(
            "[RUST] AQCUIRED C_USERNAME: {:?} vs USERNAME: {:?}",
            c_username, username
        );
        unsafe {
            println!("[RUST] GETTING POINTER TO USER!");
            println!(
                "[RUST] PASSING IN C_USERNAME AS A POINTER: {:?}",
                c_username.as_ptr()
            );
            let user_ptr = find_user_by_username(self.db, c_username.as_ptr());
            println!("[RUST] USER_PTR WAS ASSIGNED!");
            if user_ptr.is_null() {
                println!("[RUST] USER POINTER IS NULL!!");
                std::ptr::null_mut()
            } else {
                println!("[RUST] GOT USER POINTER: {:?}", user_ptr);
                user_ptr
            }
        }
    }
    pub fn get_last_user_id(&self) -> i32 {
        unsafe { (*self.db).count - 1 }
    }
    pub fn sync_user_to_c_backend(
        &self,
        username: &str,
        email: &str,
        user_id: i32,
        password: &str,
    ) -> Result<(), String> {
        let c_username = CString::new(username).map_err(|_| "Invalid username")?;
        let c_email = CString::new(email).map_err(|_| "Invalid email")?;
        let c_password = CString::new(password).map_err(|_| "Invalid password")?;

        println!("CREATING USER TO SYNC WITH C BACKEND: {:?}", c_username);
        unsafe {
            let user = create_user(
                c_username.as_ptr(),
                c_email.as_ptr(),
                user_id,
                c_password.as_ptr(),
            );
            (*user).ownership = OwnershipType::C_OWNED;
            println!("[RUST] SET OWNERSHIP OF C USER: {:?}", (*user).ownership);
            if user.is_null() {
                return Err("Failed to create user".to_string());
            }
            println!("[RUST] ADDING USER TO DB");
            add_user(self.db, user);
        }
        Ok(())
    }
    pub fn sync_user_from_rust_db(&self, user: *mut UserStructT) {
        unsafe {
            println!(
                "[RUST] SYNC_USER_FROM_RUST_DB() USER: {:?} \nOWNERSHIP: {:?}",
                user,
                (*user).ownership
            );

            if (*user).ownership == OwnershipType::SHARED_C_PRIMARY
                || (*user).ownership == OwnershipType::SHARED_RUST_PRIMARY
            {
                println!("[RUST] USER ALREADY SHARED - SKIP");
                return;
            }

            (*user).ownership = OwnershipType::SHARED_C_PRIMARY;
            println!("[RUST] AFTER SYNC OWNERSHIP: {:?}", (*user).ownership);
            add_user(self.db, user);
            println!("[RUST] USER ADDED TO C DATABASE");
        }
    }

    pub fn cast_user_struct(user: &UserStruct) -> *const UserStructT {
        user as *const UserStruct as *const UserStructT
    }

    pub fn create_session(&self, user: &UserStruct) -> Result<String, String> {
        /*
        let c_user = UserStructT {
            password: to_c_array::<MAX_PASSWORD_LENGTH>(&user.password),
            username: to_c_array::<MAX_NAME_LEN>(&user.username),
            email: to_c_array::<MAX_EMAIL_LEN>(&user.email),
            session_token: to_c_array::<MAX_SESSION_TOKEN_LEN>(&user.session_token),
            user_id: user.user_id,
            inactivity_count: user.inactivity_count,
            is_active: user.is_active,
            ownership: OwnershipType::C_OWNED,
        };
        */

        unsafe {
            let userp = DatabaseExtensions::cast_user_struct(user);
            let token_ptr = create_user_session(userp);
            if token_ptr.is_null() {
                return Err("Failed to create session".to_string());
            }

            let token = CStr::from_ptr(token_ptr).to_string_lossy().to_string();
            Ok(token)
        }
    }

    pub fn validate_session(&self, token: &str) -> Result<i32, String> {
        let c_token = CString::new(token).map_err(|_| "Invalid token")?;

        unsafe {
            let user_id = validate_user_session(c_token.as_ptr());
            if user_id == 0 {
                Err("Invalid session".to_string())
            } else {
                Ok(user_id)
            }
        }
    }

    pub fn login_user(&self, user_name: &str) -> Result<String, String> {
        unsafe {
            let c_user_name = CString::new(user_name).map_err(|_| "Invalid username")?;
            let token_ptr = user_login(self.db, c_user_name.as_ptr());
            if token_ptr.is_null() {
                return Err("Failed to create session".to_string());
            }
            Ok(CStr::from_ptr(token_ptr).to_string_lossy().to_string())
        }
    }

    pub fn get_all_user_references(&self) -> Vec<Box<UserStruct>> {
        println!("[RUST] GETTING ALL USER REFS");
        let refs = unsafe { get_user_reference_for_debugging(self.db) };
        let ref_count = unsafe { get_non_null_ref_count(self.db) };
        println!("[RUST] REF COUNT: {:?}", ref_count);
        let mut user_refs = Vec::new();
        let refs_slice = unsafe { std::slice::from_raw_parts(refs, ref_count as usize) };
        for &user_ptr in refs_slice {
            if !user_ptr.is_null() {
                unsafe {
                    let c_user = &*user_ptr;
                    if c_user.ownership == OwnershipType::SHARED_C_PRIMARY {
                        let mut rust_user = Box::from_raw(user_ptr as *mut UserStruct);
                        rust_user.ownership = OwnershipType::SHARED_RUST_PRIMARY;
                        user_refs.push(rust_user);
                    }
                };
            }
        }
        return user_refs;
    }
    pub fn increment_day(&self, rust_db: &UserDatabase) {
        unsafe {
            println!("[RUST] DB WRAPPER INCREMENT!\n");
            update_database_daily(self.db);
            //self.deactivate_idle_users(rust_db);
        }
    }
    pub fn deactivate_idle_users(&self, db: &UserDatabase) {
        unsafe {
            let db_ptr = db as *const UserDatabase as *mut UserDatabaseT;
            deactivate_users(db_ptr);
        }
    }
    pub fn create_session_for_c_ptr(&self, user: *const UserStructT) -> Result<String, String> {
        unsafe {
            let token_ptr = create_user_session(user);
            if token_ptr.is_null() {
                return Err("Failed to create session".to_string());
            }
            let token = CStr::from_ptr(token_ptr).to_string_lossy().to_string();
            Ok(token)
        }
    }
    pub fn print_database_full(&self) {
        println!("[C] PRINTING DATABASE!");
        unsafe {
            print_database(self.db);
        }
    }
}

pub fn initialize_enhanced_database(dc: &i32) -> DatabaseExtensions {
    let dc_ptr = dc as *const i32;
    DatabaseExtensions::new(dc_ptr)
}
