use std::env;
use std::ptr::{copy, null};

#[derive(Debug)]
enum PayloadType {
    OUT_OF_BOUNDS_PAYLOAD,
    DOUBLE_FREE_PAYLOAD,
    USE_AFTER_FREE_PAYLOAD,
}

//====================================Dont change these definitions==========================
const MAX_USERS: usize = 100;
const MAX_NAME_LEN: usize = 50;
const MAX_EMAIL_LEN: usize = 50;
const MAX_PASSWORD_LEN: usize = 100;
const INACTIVITY_THRESHOLD: i32 = 5;
const MAX_SESSION_TOKEN_LEN: usize = 32;

#[derive(Debug, Clone)] //Clone - allows copying of UserStruct
pub struct UserStruct {
    pub password: [u8; MAX_PASSWORD_LEN],
    pub username: [u8; MAX_NAME_LEN],
    pub user_id: i32,
    pub email: [u8; MAX_EMAIL_LEN],
    pub inactivity_count: i32,
    pub is_active: i32,
    pub session_token: [u8; MAX_SESSION_TOKEN_LEN],
}

#[derive(Debug)]
pub struct UserDatabase {
    //Option<Box<UserStruct>> - allows storing optional user data on the heap
    //Option: may or may not exist - Some(t): contains, None: empty
    //Box: smart pointer that manages memory on the heap - owned, single owner, auto cleanup
    pub users: [Option<Box<UserStruct>>; MAX_USERS], //arrays = [Type; Size]
    pub count: i32,
    pub capacity: i32,
}
//====================================End of Definitions=======================================

//Complete the body of the program here.

fn init_database() -> UserDatabase {
    let db = UserDatabase {
        users: std::array::from_fn(|_| None), //creates MAX_USERS number of None values and stores in users array
        //can use _ as index as rust infers it needs to create an array of length MAX_USERS
        count: 0,
        capacity: MAX_USERS as i32,
    };
    return db;
}

//&mut: can modify the value it points to
//mut: can modify the parameter
fn add_user(db: &mut UserDatabase, mut user: Box<UserStruct>) {
    (*user).user_id = (*db).count;
    //indexing arrays in rust always requires usize indeces
    let index: usize = (*db).count as usize;
    (*db).users[index] = Some(user);
    (*db).count += 1;
}

//dont need to manually free
//fn free_user(user: &mut UserStruct) {}

fn print_database(db: &UserDatabase) {
    for user in &(*db).users {
        if let Some(_user) = user {
            print_user(_user);
        }
    }
}

fn copy_string(dest: &mut [u8], src: &str, n: usize) {
    //gets the minimum between src.len and n, so we dont go over the range
    let copy_len = src.len().min(n);
    for i in 0..copy_len {
        dest[i] = src.as_bytes()[i];
    }
    if copy_len < dest.len() {
        dest[copy_len] = 0;
    }
}

fn create_user(username: &str, email: &str, user_id: i32, password: &str) -> Box<UserStruct> {
    let mut user = UserStruct {
        username: [0; MAX_NAME_LEN],
        email: [0; MAX_EMAIL_LEN],
        password: [0; MAX_PASSWORD_LEN],
        user_id,
        inactivity_count: 0,
        is_active: 1,
        session_token: [0; MAX_SESSION_TOKEN_LEN],
    };

    copy_string(&mut user.username, username, MAX_NAME_LEN - 1);
    copy_string(&mut user.email, email, MAX_EMAIL_LEN - 1);
    copy_string(&mut user.password, password, MAX_PASSWORD_LEN - 1);

    //makes a pointer on the stack to user which is now on the heap
    return Box::new(user); //Box is like a malloc pointer, with auto free, auto dereference...
}

fn find_user_by_id(db: &UserDatabase, user_id: i32) -> Option<Box<UserStruct>> {
    //db = &UserDatabase
    //*db = UserDatabase
    //(*db).users = [actual user array]
    //&(*db).users = &[reference to user array]
    for user in &(*db).users {
        //Some(...) = &Box<UserStruct>
        if let Some(_user) = user {
            //(*..).user_id = actual i32 user_id (deref)
            if (*_user).user_id == user_id {
                //Some((Box<UserStruct>).clone())
                //creates new Box<UserStruct> with copy of UserStruct
                return Some((*_user).clone());
            }
        }
    }
    return None;
}

/*
fn cleanup_db(db: &mut UserDatabase) {
    drop(db);
}
*/

fn update_database_daily(db: &mut UserDatabase) {
    for i in 0..(*db).count as usize {
        //copy user at index i to _user if it it not None (borrow)
        //ref: creates a reference - primarily used in pattern matching, &mut used everuwhere else pretty much
        if let Some(ref mut _user) = (*db).users[i] {
            if _user.inactivity_count > INACTIVITY_THRESHOLD {
                (*db).users[i] = None;
            } else {
                (*_user).inactivity_count += 1;
            }
        }
    }
}

fn update_username(db: &mut UserDatabase, username: &str, new_username: &str) {
    let user: Option<&mut Box<UserStruct>> = find_user_by_username_mut(db, username);
    if let Some(_user) = user {
        //println!("OLD USERNAME: {:?}", _user.username);
        copy_string(&mut _user.username, new_username, MAX_NAME_LEN - 1);
        //println!("NEW USERNAME: {:?}", _user.username);
    }
}

fn user_login(db: &mut UserDatabase, username: &str) {
    let user: Option<&mut Box<UserStruct>> = find_user_by_username_mut(db, username);
    if let Some(_user) = user {
        _user.inactivity_count = 0;
    }
}

fn get_nullt_index_from_u8(bytes: &[u8]) -> usize {
    for i in 0..bytes.len() {
        if bytes[i] == 0 {
            return i;
        }
    }
    return bytes.len();
}

//from u8 to string, excluding null terminator
fn u8_to_string_no_nullt(bytes: &[u8]) -> String {
    let null_idx: usize = get_nullt_index_from_u8(bytes);
    return String::from_utf8_lossy(&bytes[0..null_idx]).to_string();
}

//'a - lifetime specifier: specifies which parameter eth reference of the return is to
fn get_password(db: &UserDatabase, username: &str) -> Option<String> {
    let user = find_user_by_username(db, username)?;
    return Some(u8_to_string_no_nullt(&user.password));
}

fn update_password(user: &mut Option<Box<UserStruct>>, password: &str) {
    if let Some(ref mut _user) = user {
        copy_string(&mut _user.password, password, password.len());
    }
}

fn print_user(user: &Box<UserStruct>) {
    let username = u8_to_string_no_nullt(&user.username);
    let email = u8_to_string_no_nullt(&user.email);
    let password = u8_to_string_no_nullt(&user.password);
    println!(
        "User {:?}: {:?} Email: {:?}, Inactivity: {:?}, Password: {:?}\n",
        user.user_id, username, email, user.inactivity_count, password
    );
}

//returns an immutable reference to the UserStruct Option
fn find_user_by_username<'a>(db: &'a UserDatabase, username: &str) -> Option<&'a Box<UserStruct>> {
    for user in &(*db).users {
        if let Some(_user) = user {
            //println!("USERNAME: {:?} VS TARGET: {:?}", username.as_bytes(), _user.username);
            let null_idx = get_nullt_index_from_u8(&_user.username);
            if username
                .as_bytes()
                .eq_ignore_ascii_case(&_user.username[0..null_idx])
            {
                return Some(_user);
            }
        }
    }
    return None;
}

//returns a mutable reference to a UserStruct option
fn find_user_by_username_mut<'a>(
    db: &'a mut UserDatabase,
    username: &str,
) -> Option<&'a mut Box<UserStruct>> {
    for user in &mut (*db).users {
        if let Some(_user) = user {
            //println!("USERNAME: {:?} VS TARGET: {:?}", username.as_bytes(), _user.username);
            let null_idx = get_nullt_index_from_u8(&_user.username);
            if username
                .as_bytes()
                .eq_ignore_ascii_case(&_user.username[0..null_idx])
            {
                return Some(_user);
            }
        }
    }
    return None;
}

/*
fn find_user_by_username(db: &UserDatabase, username: &str) -> Option<Box<UserStruct>> {
    for user in &(*db).users {
        if let Some(ref _user) = user {
            //println!("USERNAME: {:?} VS TARGET: {:?}", username.as_bytes(), _user.username);
            if username
                .as_bytes()
                .eq_ignore_ascii_case(&_user.username[0..username.len()])
            {
                return Some((*_user).clone());
            }
        }
    }
    return None;
}
*/

/*
for i in 0..(*db).count as usize {
    if let Some(ref mut _user) = &(*db).users[i] {
        if (*_user).username.eq_ignore_ascii_case(username.as_bytes()) {
            return Some((_user).clone());
        }
    }
}
*/

fn main() {
    let args: Vec<String> = env::args().collect(); //arguments from command line

    if args.len() > 2 {
        println!("Usage: {:?} <payload_type>", args[0]);
        println!("OUT_OF_BOUNDS_PAYLOAD");
        println!("DOUBLE_FREE_PAYLOAD");
        println!("USE_AFTER_FREE_PAYLOAD");
        return;
    }

    let payload_num = match args[1].parse::<i32>() {
        //returns Result<T, error>
        Ok(num) => num, //if it succeeds, store it in num
        Err(_) => {
            println!("Invalid payload type: {}", args[1]);
            return;
        }
    };

    let payload_type = match payload_num {
        0 => PayloadType::OUT_OF_BOUNDS_PAYLOAD,
        1 => PayloadType::USE_AFTER_FREE_PAYLOAD,
        2 => PayloadType::DOUBLE_FREE_PAYLOAD,
        _ => {
            println!("Invalid payload num");
            return;
        }
    };

    let payload_name = match payload_type {
        PayloadType::OUT_OF_BOUNDS_PAYLOAD => "OUT_OF_BOUNDS_PAYLOAD",
        PayloadType::USE_AFTER_FREE_PAYLOAD => "USE_AFTER_FREE_PAYLOAD",
        PayloadType::DOUBLE_FREE_PAYLOAD => "DOUBLE_FREE_PAYLOAD",
    };

    println!("Running test payload {:?}", payload_name);

    let mut db = init_database();
    let mallory = create_user("Mallory", "mallory@nus.edu.sg", 0, "malloryisnotevil");
    add_user(&mut db, mallory);
    let alice = create_user("Alice", "alice@nus.edu.sg", 1, "aliceinthewonderland");
    add_user(&mut db, alice);
    let bob = create_user("Bob", "bob@nus.edu.sg", 2, "bobthebuilder");
    add_user(&mut db, bob);
    let eve = create_user("Eve", "eve@nus.edu.sg", 3, "eve4ever");
    add_user(&mut db, eve);

    match payload_type {
        PayloadType::OUT_OF_BOUNDS_PAYLOAD => {
            let long_username = "C".repeat(99); //string with 99 Cs
            update_username(&mut db, "Mallory", &long_username);
        }
        PayloadType::USE_AFTER_FREE_PAYLOAD => {
            println!("1");
        }
        PayloadType::DOUBLE_FREE_PAYLOAD => {
            println!("2");
        }
    };

    println!("==============================Final Database State:===========================================\n");
    print_database(&db);
    println!("==============================================================================================\n");

    println!("If this program didn't crash, were you lucky ? Check the logs\n");

    /*
    println!("Hello, world!"); // Placeholder main function

    //(a)so this creates a mutable varaiable - meaning you can modify the variable itself
    let mut user_db: UserDatabase = init_database();
    let user1: Box<UserStruct> = create_user("John", "John@gmail.com", 0, "John123");
    let user2: Box<UserStruct> = create_user("Jim", "Jim@u.nus.edu", 1, "Jim000");

    //(b)this expects a &mut - mutable reference
    //(c)mut user_db lets you reassign user_db itself
    //(d)&mut user_db lets you modify the contents of user_db
    add_user(&mut user_db, user1);
    //print_database(&user_db);

    //add_user(&mut user_db, user2); //(a)value of Jim user moved here
    add_user(&mut user_db, user2.clone()); //cloned as to not take ownership
                                           //print_database(&user_db);

    let some_user = find_user_by_id(&user_db, 0);
    println!("{:?}", some_user); //returns the user

    let none_user = find_user_by_id(&user_db, -1);
    println!("{:?}", none_user); //returns None

    update_database_daily(&mut user_db);

    let some_user = find_user_by_id(&user_db, 0);
    println!("AFTER DB UPDATE: {:?}", some_user);

    let found_user = find_user_by_username(&user_db, "Jim");
    println!("FOUND USER: {:?}", found_user);

    update_username(&mut user_db, "Jim", "Jimmy");

    println!("JIMMY BEFORE LOGIN: {:?}", user2); //(b)value of Jim user borrowed here after move - used .clone()
    user_login(&mut user_db, "Jimmy");
    println!("JIMMY AFTER LOGIN: {:?}", user2);

    let jimmys_password: Option<String> = get_password(&user_db, "Jimmy");
    println!("JIMMYS PASSWORD: {:?}", jimmys_password);

    update_password(&mut user_db, "Jimmy", "Jimmy999");
    let jimmys_new_password: Option<String> = get_password(&user_db, "Jimmy");
    println!("JIMMYS NEW PASSWORD: {:?}", jimmys_new_password);

    print_user(user2);
    */
}
