const MAX_USERS: usize = 1000;
const MAX_NAME_LEN: usize = 50;
const INACTIVITY_THRESHOLD: i32 = 5;
const MAX_EMAIL_LEN: usize = 50;
const MAX_PASSWORD_LENGTH: usize = 100;
const MAX_SESSION_TOKEN_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum OwnershipType {
    RUST_OWNED,
    C_OWNED,
    SHARED_RUST_PRIMARY,
    SHARED_C_PRIMARY,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UserStruct {
    pub password: [u8; MAX_PASSWORD_LENGTH],
    pub username: [u8; MAX_NAME_LEN],
    pub user_id: i32,
    pub email: [u8; MAX_EMAIL_LEN],
    pub inactivity_count: i32,
    pub is_active: i32,
    pub session_token: [u8; MAX_SESSION_TOKEN_LEN],
    pub ownership: OwnershipType,
}

impl Default for UserStruct {
    fn default() -> Self {
        UserStruct {
            password: [0; MAX_PASSWORD_LENGTH],
            username: [0; MAX_NAME_LEN],
            user_id: 0,
            email: [0; MAX_EMAIL_LEN],
            inactivity_count: 0,
            is_active: 0,
            session_token: [0; MAX_SESSION_TOKEN_LEN],
            ownership: OwnershipType::RUST_OWNED,
        }
    }
}

#[derive(Debug)]
pub struct UserDatabase {
    pub users: [Option<Box<UserStruct>>; MAX_USERS],
    pub count: i32,
    pub capacity: i32,
}

pub fn init_database() -> Box<UserDatabase> {
    let db = UserDatabase {
        users: std::array::from_fn(|_| None),
        count: 0,
        capacity: MAX_USERS as i32,
    };
    return Box::new(db);
}

pub fn add_user(db: &mut UserDatabase, mut user: Box<UserStruct>) {
    println!("[RUST] ADD_USER()");
    (*user).user_id = (*db).count + 1;
    println!("[RUST] USER_ID: {:?}", (*user).user_id);
    let index: usize = (*db).count as usize;

    println!("[RUST] CURRENT DB COUNT: {:?}", db.count);
    println!("[RUST] MAX USERS: {}", MAX_USERS);
    (*db).users[index] = Some(user);
    println!(
        "[RUST] USER: {:?} ADDED TO DB AT INDEX: {:?}",
        (*db).users[index],
        index
    );
    (*db).count += 1;
}

fn copy_string(dest: &mut [u8], src: &str, n: usize) {
    //gets the minimum between src.len and n
    let copy_len = src.len().min(n);
    for i in 0..copy_len {
        dest[i] = src.as_bytes()[i];
    }
    if copy_len < dest.len() {
        dest[copy_len] = 0;
    }
}

pub fn create_user(username: &str, email: &str, user_id: i32, password: &str) -> Box<UserStruct> {
    let mut user = UserStruct {
        username: [0; MAX_NAME_LEN],
        email: [0; MAX_EMAIL_LEN],
        password: [0; MAX_PASSWORD_LENGTH],
        user_id: user_id,
        inactivity_count: 0,
        is_active: 1,
        session_token: [0; MAX_SESSION_TOKEN_LEN],
        ownership: OwnershipType::RUST_OWNED,
    };

    copy_string(&mut user.username, username, MAX_NAME_LEN);
    copy_string(&mut user.email, email, MAX_EMAIL_LEN);
    copy_string(&mut user.password, password, MAX_PASSWORD_LENGTH);

    println!("[RUST] USER CREATED: {:?}", user.username);

    return Box::new(user);
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

pub fn find_user_by_username<'a>(
    db: &'a UserDatabase,
    username: &'a str,
) -> Option<&'a UserStruct> {
    for user in &(*db).users {
        if let Some(_user) = user {
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

pub fn find_user_by_username_mut<'a>(
    db: &'a mut UserDatabase,
    username: &'a str,
) -> Option<&'a mut UserStruct> {
    for user in &mut (*db).users {
        if let Some(_user) = user {
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

pub fn print_user(user: &Box<UserStruct>) {
    let username = u8_to_string_no_nullt(&user.username);
    let email = u8_to_string_no_nullt(&user.email);
    let password = u8_to_string_no_nullt(&user.password);

    println!(
        "User: {:?}, ID: {:?} Email: {:?}, Inactivity: {:?}, Password: {:?}",
        username, user.user_id, email, user.inactivity_count, password
    );
}

pub fn print_database(db: &UserDatabase) {
    for user in &(*db).users {
        if let Some(_user) = user {
            print_user(_user);
        }
    }
}

pub fn update_database_daily(db: &mut UserDatabase) {
    let mut user_removed = 0;
    println!("[RUST] UPDATE_DATABASE_DAILY()");
    for i in 0..(*db).count as usize {
        if let Some(mut _user) = (*db).users[i].take() {
            println!(
                "[RUST] CURRENT USER INACTIVITY COUNT: {:?}",
                _user.inactivity_count
            );
            if _user.inactivity_count > INACTIVITY_THRESHOLD {
                println!(
                    "[RUST] USER: {:?} HAS BEEN INACTIVE, REMOVING",
                    _user.user_id
                );
                user_removed += 1;
            } else {
                _user.inactivity_count += 1;
                db.users[i] = Some(_user);
            }
        }
    }

    db.count -= user_removed;
}

pub fn user_login(db: &mut UserDatabase, username: &str) {
    let user = find_user_by_username_mut(db, username);
    if let Some(_user) = user {
        _user.inactivity_count = 0;
    }
}

fn main() {
    print!("Hello, world!");
}
