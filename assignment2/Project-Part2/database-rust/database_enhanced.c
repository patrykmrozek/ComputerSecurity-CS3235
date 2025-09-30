#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

// Read Only - Do not modify
#define MAX_USERS 1000
#define MAX_NAME_LEN 50
#define INACTIVITY_THRESHOLD 5
#define MAX_EMAIL_LEN 50
#define MAX_PASSWORD_LENGTH 100
#define SESSION_MAX_IDLE_TIME 1
#define MAX_SESSIONS 100
#define MAX_SESSION_TOKEN_LEN 32

typedef enum {
    C_OWNED = 0,
    RUST_OWNED = 1,
    SHARED_RUST_PRIMARY = 2,
    SHARED_C_PRIMARY = 3
} OwnershipType;

typedef struct {
    char password[MAX_PASSWORD_LENGTH];
    char username[MAX_NAME_LEN];
    int user_id;
    char email[MAX_EMAIL_LEN];
    int inactivity_count;
    int is_active;
    char session_token[MAX_SESSION_TOKEN_LEN];
    OwnershipType ownership;
} UserStruct_t;

typedef struct {
    UserStruct_t *users[MAX_USERS];
    int count;
    int capacity;
} UserDatabase_t;

typedef struct {
    int user_id;
    char username[MAX_NAME_LEN];
    char session_token[MAX_SESSION_TOKEN_LEN];
    int session_idle_time;
    int is_active;
} SessionInfo_t;

typedef struct {
    SessionInfo_t *sessions[MAX_SESSIONS];
    int session_count;
    UserDatabase_t* db_ref;
} SessionManager_t;

// Global state for cross-language interaction
static SessionManager_t* global_session_manager = NULL;
static UserDatabase_t* global_db = NULL;
int *global_day_counter;

// Core database functions
UserDatabase_t* init_database(const int *dc) {
    UserDatabase_t* db = malloc(sizeof(UserDatabase_t));
    db->count = 0;
    db->capacity = MAX_USERS;
    global_db = db;
    global_day_counter = (int*)dc;
    return global_db;
}

void free_user(UserStruct_t* user) {
    #ifdef DEBUG_EN
    printf("[C-Code] Freeing user: %s\n", user->username);
    #endif
    printf("[C] ATTEMPTING TO FREE USER AT %p\n", (void*)user);
    if (user != NULL) {
        if (user->ownership == C_OWNED) {
            printf("[C] FREEING A C OWNED USER: %s - OWNERSHIP: %d\n", user->username, user->ownership);
            free(user);
        } else {
            printf("[C] SKIPPING USER NOW C OWNED: %s\n", user->username);
        }
    }
}

void add_user(UserDatabase_t* db, UserStruct_t* user) {
    printf("[C] ADDING USER: %s WITH OWNERSHIP: %d\n", user->username, user->ownership);
    if (db->count >= MAX_USERS) {
        #ifdef DEBUG_En
        printf("[C-Code] DB Full, cannot add user: %s\n", user->username);
        #endif
        free_user(user);
        return;
    }

    if (user != NULL) {
        #ifdef DEBUG_EN
        printf("[C-Code] Adding user: %s\n increasing count to %d\n", user->username, db->count + 1);
        #endif
        printf("[C] OWNERSHIP BEFORE: %d\n", user->ownership);
        user->user_id = db->count + 1;
        db->users[db->count++] = user;
        printf("[C] OWNERSHIP AFTER: %d\n", user->ownership);
    }
}


void cleanup_database(UserDatabase_t* db) {
    for (int i = 0; i < db->count; i++) {
        free_user(db->users[i]);
    }
    free(db);

}

void print_database(UserDatabase_t *db) {
    for(int i = 0; i < db->count; i++) {
        if (db->users[i] == NULL) {
            continue;
        }
        printf("User: %s, ID: %d, Email: %s, Inactivity: %d  Password = %s\n", db->users[i]->username, db->users[i]->user_id, db->users[i]->email, db->users[i]->inactivity_count, db->users[i]->password);
    }
}

void copy_string(char* dest, char* src, size_t n) {
    int min_len = (strlen(src) < n-1) ? strlen(src) : n-1;
    for (size_t i = 0; i < min_len; i++)
    {
        dest[i] = src[i];
    }
    dest[min_len] = '\0'; // Ensure null termination
}
int get_current_time() {
    return time(NULL);
}

UserStruct_t* create_user(char* username, char* email, int user_id, char* password) {
    UserStruct_t* user = malloc(sizeof(UserStruct_t));

    copy_string(user->username, username, MAX_NAME_LEN);
    copy_string(user->email, email, MAX_EMAIL_LEN);
    copy_string(user->password, password, MAX_PASSWORD_LENGTH);

    user->user_id = user_id;
    user->inactivity_count = 0;
    user->is_active = 1;
    user->ownership = C_OWNED;
    printf("[C] Created C-owned user at %p\n", (void*)user);

    return user;
}
void update_day_counter(int *day_counter) {
    global_day_counter = day_counter;
}

UserStruct_t* find_user_by_id(UserDatabase_t* db, int user_id) {
    for (int i = 0; i < db->count; i++) {
        if (db->users[i]->user_id == user_id) {
            return db->users[i];
        }
    }
    return NULL;
}
int init_session_manager() {
    if (global_session_manager != NULL) {
        return 0;
    }

    global_session_manager = malloc(sizeof(SessionManager_t));
    if (!global_session_manager) {
        return -1;
    }

    global_session_manager->session_count = 0;
    global_session_manager->db_ref = global_db;
    memset(global_session_manager->sessions, 0, sizeof(SessionInfo_t*) * MAX_SESSIONS);
    #ifdef DEBUG_EN
    printf("[C-Code] Session manager initialized\n");
    #endif
    return 0;
}

void generate_token(char *token, char *name, int timestamp){
    char temp[MAX_SESSION_TOKEN_LEN];
    snprintf(temp, sizeof(temp), "session_%s_%d", name, timestamp);
    copy_string(token, temp, MAX_SESSION_TOKEN_LEN);
}

char* create_user_session(UserStruct_t *user) {
    if (!global_session_manager) {
        if(init_session_manager()){
            return NULL;
        }
    }

    if (!user) {
        return NULL;
    }

    if (global_session_manager->session_count >= MAX_SESSIONS) {
        #ifdef DEBUG_EN
        printf("[C-Code] Too many active sessions\n");
        #endif
        exit(1);
    }

    char* token = malloc(MAX_SESSION_TOKEN_LEN);
    generate_token(token, user->username, get_current_time());


    SessionInfo_t* session = malloc(sizeof(SessionInfo_t));
    session->user_id = user->user_id;
    copy_string(session->username, user->username, MAX_NAME_LEN);
    copy_string(session->session_token, token, MAX_SESSION_TOKEN_LEN);
    session->is_active = 1;
    session->session_idle_time = 0;
    global_session_manager->sessions[global_session_manager->session_count] = session;
    global_session_manager->session_count++;

    #ifdef DEBUG_EN
    printf("Created session for user %d: %s\n", user_id, token);
    #endif
    return token;
}

// Memory management and optimization functions
int get_non_null_ref_count(UserDatabase_t* db) {
    int count = 0;
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] != NULL) {
            count++;
        }
    }
    return count;
}

//Hint : Interesting function
UserStruct_t** get_user_reference_for_debugging(UserDatabase_t* db) {
    int non_null = get_non_null_ref_count(db);
    UserStruct_t** user = malloc(non_null * sizeof(UserStruct_t*));

    if (user == NULL) {
        return NULL;
    }

    #ifdef DEBUG_EN
    printf("[C-Code] Scanning database for non-null users... among %d users\n", db->count);
    #endif

    int index = 0;
    for(int i = 0; i < db->count; i++) {
        UserStruct_t* useri = db->users[i];
        if(useri!=NULL){
            #ifdef DEBUG_EN
            printf("[C-Code] Adding user reference for %s\n", useri->username);
            #endif
            user[index++] = useri;
        }
    }
    return user;
}



void clone_user(UserStruct_t* src, UserStruct_t* dest) {
    copy_string(dest->username, src->username, MAX_NAME_LEN);
    copy_string(dest->email, src->email, MAX_EMAIL_LEN);
    copy_string(dest->password, src->password, MAX_PASSWORD_LENGTH);
    dest->inactivity_count = src->inactivity_count;
    copy_string(dest->session_token, src->session_token, MAX_SESSION_TOKEN_LEN);
    dest->is_active = src->is_active;
}

//Hint : Interesting function
void memory_pressure_cleanup(UserDatabase_t* db) {
    #ifdef DEBUG_EN
    printf("[C-Code] System under memory pressure - performing selective cleanup\n");
    #endif
    // shift users together and compact the array
    int write_index = 0;
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] != NULL) {
            if (write_index != i) {
                db->users[write_index] = db->users[i];
                db->users[i] = NULL;
            }
            write_index++;
        }
    }

    db->count = write_index;
    #ifdef DEBUG_EN
    printf("Memory pressure cleanup completed\n");
    #endif
}


SessionInfo_t* find_session_by_token(SessionManager_t* sm, char* token) {
    for (int i = 0; i < sm->session_count; i++) {
        if (sm->sessions[i]->is_active && strcmp(sm->sessions[i]->session_token, token) == 0) {
            return sm->sessions[i];
        }
    }
    return NULL;
}

int validate_user_session(char* token) {
    if (!global_session_manager || !token) {
        return 0;
    }
    SessionInfo_t *session = find_session_by_token(global_session_manager, token);
    if (!session) {
        return 0;
    }
    if (session->session_idle_time > SESSION_MAX_IDLE_TIME) {

        session->is_active = 0;
        //free(session);
        return 1;
    }
    else session->session_idle_time += 1;
    return 0;
}

void merge_duplicate_handles(UserDatabase_t *db){
    for(int i = (db->count-1); i >= 0; i--){
        if (!db->users[i]) {
            continue;
        }
        for(int j = 0; j < i; j++){
            if (!db->users[j]) {
                continue;
            }
            if(strcmp(db->users[i]->username, db->users[j]->username) == 0 && strcmp(db->users[i]->email, db->users[j]->email) == 0 && strcmp(db->users[i]->password, db->users[j]->password) == 0){
                #ifdef DEBUG_EN
                printf("[C-Code] Merging duplicate user handles for %s\n", db->users[i]->username);
                #endif
                UserStruct_t* to_free = db->users[j];
                db->users[j] = NULL;
                free_user(to_free);
                break;
            }
        }
    }
    //memory_pressure_cleanup(db);
}

void update_database_daily(UserDatabase_t* db) {
    printf("[C] UPDATING DATABASE!\n");
    printf("[C] CURRENT DB COUNT: %d\n", db->count);

    for (int i = 0; i < db->count; i++) {
        printf("[C] ITERATING THROUGH USERS: %d\n", i);

        if (db->count >= MAX_USERS) {
            printf("[C] USER LIMIT REAHED!\n");
            return;
        }

        if (db->users[i] == NULL) {
            printf("[C] USER IS NULL - SKIPPING!\n");
            continue;
        }

        if (!db->users[i]->is_active && db->users[i]->inactivity_count > INACTIVITY_THRESHOLD) {
            //#ifdef DEBUG_EN
            printf("[C-Code] Removing user[%d] %s due to inactivity for %d days\n", db->users[i]->user_id, db->users[i]->username, db->users[i]->inactivity_count);
            //#endif
            printf("[C] FREEING USER BECAUSE INACTIVE!\n");
            free_user(db->users[i]);
            db->users[i] = NULL;

            continue;
        }

            int valid_session = (validate_user_session(db->users[i]->session_token));
            if (!valid_session) {
                db->users[i]->is_active = 0;
            } else {
                db->users[i]->is_active = 1;
            }


            //#ifdef DEBUG_EN
            printf("[C-Code] %d is max allowed threshold Incrementing inactivity for user[%d] %s to %d days\n", INACTIVITY_THRESHOLD, db->users[i]->user_id, db->users[i]->username, db->users[i]->inactivity_count + 1);
            //#endif
            printf("[C] INCREMENTING INACTIVITY!\n");
            printf("[C] CURRENT USER: %s - INACTIVITY COUNT: %d\n", db->users[i]->username, db->users[i]->inactivity_count);
            db->users[i]->inactivity_count++;
            printf("[C] INACTIVITY COUNT INCREASED TO %d FOR USER %d\n", db->users[i]->inactivity_count, db->users[i]->user_id);
    }

    printf("[C] CURRENT DAY: %d\n", *global_day_counter);

    if(*global_day_counter % 4 == 0){
        printf("[C] CALLING MERGE DUPLICATE HANDLES()\n");
        merge_duplicate_handles(db);
    }

    if (*global_day_counter % 8 == 0){
        printf("CALLING MEMORY PRESSURE CLEANUP()\n");
        memory_pressure_cleanup(db);
    }
}

UserStruct_t* find_user_by_username(UserDatabase_t* db, char* user_name) {
    printf("[C] FINDING USER BY USERNAME: %s", user_name);
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] == NULL) {
            printf("USER IS NULL!\n");
            continue;
        }
        if (strcmp(db->users[i]->username, user_name) == 0) {
            return db->users[i];
        }
    }
    return NULL;
}


char* user_login(UserDatabase_t* db, char* user_name) {
    UserStruct_t* user = find_user_by_username(db, user_name);

    #ifdef DEBUG_EN
    printf("[C-Code] User[%d] %s logged in after %d days\n", user->user_id, user->username, user->inactivity_count);
    #endif
    user->inactivity_count = 0;
    char *token = create_user_session(user);
    copy_string(user->session_token, token, MAX_SESSION_TOKEN_LEN);
    user->is_active = 1;
    return token;
}

char* get_password(UserDatabase_t* db, char* username) {
    UserStruct_t* user = find_user_by_username(db, username);
    #ifdef DEBUG_EN
        printf("Password  Request for User[%d] %s is %s\n", user->user_id, user->username, user->password);
    #endif
    return user->password;
}




UserStruct_t* find_user_by_session_token(UserDatabase_t* db, char* session_token) {
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] != NULL && strcmp(db->users[i]->session_token, session_token) == 0) {
            return db->users[i];
        }
    }
    return NULL;
}


void deactivate_users(UserDatabase_t* rust_db) {
    for (int i = 0; i < global_session_manager->session_count; i++) {

        if (global_session_manager->sessions[i]->session_idle_time > SESSION_MAX_IDLE_TIME) {
            global_session_manager->sessions[i]->is_active = 0;
        }

        UserStruct_t *user = find_user_by_session_token(global_db, global_session_manager->sessions[i]->session_token);
        user->is_active = 0;

        user = find_user_by_session_token(rust_db, global_session_manager->sessions[i]->session_token);
        user->is_active = 0;

        free(global_session_manager->sessions[i]);
    }
}
