#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_USERS 100
#define MAX_NAME_LEN 50
#define INACTIVITY_THRESHOLD 10
#define MAX_EMAIL_LEN 50
#define MAX_PASSWORD_LENGTH 100

typedef struct {
    char password[MAX_PASSWORD_LENGTH];
    int user_id;
    char email[MAX_EMAIL_LEN];
    int inactivity_count;
    char username[MAX_NAME_LEN];
} UserStruct_t;

typedef struct {
    UserStruct_t *users[MAX_USERS];
    int count;
    int capacity;
} UserDatabase_t;

typedef enum {
    OUT_OF_BOUNDS_PAYLOAD,
    DOUBLE_FREE_PAYLOAD,
    USE_AFTER_FREE_PAYLOAD
} payload_t;

UserStruct_t* find_user_by_username(UserDatabase_t* db, char* user_name);
UserDatabase_t* init_database() {
    UserDatabase_t* db = malloc(sizeof(UserDatabase_t));
    db->count = 0;
    db->capacity = MAX_USERS;
    return db;
}

void add_user(UserDatabase_t* db, UserStruct_t* user) {
        user->user_id = db->count;
        db->users[db->count++] = user;
}

void free_user(UserStruct_t* user) {
    free(user);
}

void print_database(UserDatabase_t *db) {
    for(int i = 0; i < db->count; i++) {
        printf("User: %s, ID: %d, Email: %s, Inactivity: %d  Password = %s\n", db->users[i]->username, db->users[i]->user_id, db->users[i]->email, db->users[i]->inactivity_count, db->users[i]->password);
    }
}

void copy_string(char* dest, char* src, size_t n) {
    for (size_t i = 0; i < n; i++)
    {
        dest[i] = src[i];
    }
    dest[n] = '\0'; // Ensure null termination
}

UserStruct_t* create_user(char* username, char* email, int user_id, char* password) {
    UserStruct_t* user = malloc(sizeof(UserStruct_t));

    copy_string(user->username, username, strlen(username));
    copy_string(user->email, email, strlen(email));
    copy_string(user->password, password, strlen(password));

    user->user_id = user_id;
    user->inactivity_count = 0;
    return user;
}

UserStruct_t* find_user_by_id(UserDatabase_t* db, int user_id) {
    for (int i = 0; i <= db->count; i++) {
        if (db->users[i]->user_id == user_id) {
            return db->users[i];
        }
    }
    return NULL;
}

void cleanup_database(UserDatabase_t* db) {
    for (int i = 0; i < db->count; i++) {
        free_user(db->users[i]);
    }
    free(db);
}


void update_database_daily(UserDatabase_t* db) {
    for (int i = 0; i < db->count; i++) {
        if (db->users[i]->inactivity_count > INACTIVITY_THRESHOLD) {
            free_user(db->users[i]);
        } else {
            db->users[i]->inactivity_count++;
        }
    }
}

void update_username(UserDatabase_t* db, char* username, char* new_username) {
    UserStruct_t* user = find_user_by_username(db, username);
    copy_string(user->username, new_username, strlen(new_username));
}
void user_login(UserDatabase_t* db, char* username) {
    UserStruct_t* user = find_user_by_username(db, username);
    user->inactivity_count = 0;
}


char* get_password(UserDatabase_t* db, char* username) {
    UserStruct_t* user = find_user_by_username(db, username);
    return user->password;
}

void update_password(UserStruct_t *user, char *password){
    copy_string(user->password, password, strlen(password));
}

void print_user(UserStruct_t *user) {
    printf("User[%d] %s: Email: %s, Inactivity: %d, Password: %s\n",
           user->user_id, user->username, user->email, user->inactivity_count, user->password);
}

UserStruct_t* find_user_by_username(UserDatabase_t* db, char* user_name) {
    for (int i = 0; i < db->count; i++) {
        if (strcmp(db->users[i]->username, user_name) == 0) {
            return db->users[i];
        }
    }
    return NULL;
}

int main(int argc, char* argv[]) {


    if (argc < 2) {
        printf("Usage: %s <payload_type>\n 0. OUT_OF_BOUNDS_PAYLOAD\n 1. DOUBLE_FREE_PAYLOAD\n 2. USE_AFTER_FREE_PAYLOAD\n", argv[0]);
        return 1;
    }


    payload_t iter = atoi(argv[1]);

    printf("Running Test Payload %s\n", iter == OUT_OF_BOUNDS_PAYLOAD ? "OUT_OF_BOUNDS_PAYLOAD" :
           iter == USE_AFTER_FREE_PAYLOAD ? "USE_AFTER_FREE_PAYLOAD" :
           iter == DOUBLE_FREE_PAYLOAD ? "DOUBLE_FREE_PAYLOAD" : "UNKNOWN");

    UserDatabase_t* db = init_database();
    UserStruct_t  *mallory, *alice,*bob,*eve;
    mallory = create_user("Mallory", "mallory@nus.edu.sg",0,"malloryisnotevil");
    add_user(db, mallory);
    alice = create_user("Alice", "alice@nus.edu.sg", 1, "aliceinthewonderland");
    add_user(db, alice);
    bob = create_user("Bob", "bob@nus.edu.sg", 2, "bobthebuilder");
    add_user(db, bob);
    eve = create_user("Eve", "eve@nus.edu.sg", 3, "eve4ever");
    add_user(db, eve);

    switch (iter)
    {
        case OUT_OF_BOUNDS_PAYLOAD:
            char long_username[100];
            memset(long_username, 'C', 99);
            long_username[99] = '\0';
            update_username(db, "Mallory", long_username);
            break;

        case DOUBLE_FREE_PAYLOAD:
            for(int i = 0; i <= INACTIVITY_THRESHOLD+2; i++) {
                printf("----------------------Day %d: User login and database update-----------------------\n", i + 1);
                user_login(db, "Alice");
                user_login(db, "Bob");
                update_database_daily(db);
                printf("\n");
            }

        break;

        case USE_AFTER_FREE_PAYLOAD:
                printf("--------------Starting a sprint of the database update and user login-------------\n");
                for(int i = 0; i <= INACTIVITY_THRESHOLD+1; i++) {
                    user_login(db, "Alice");
                    user_login(db, "Bob");
                    update_database_daily(db);
                }

                printf("--------------Database update and user login sprint finished----------------------\n");
                UserStruct_t *charlie, *bruce, *joker;
                charlie = create_user("Charlie", "charlie@nus.edu.sg", 3, "charlieandthechocolatefactory");
                add_user(db, charlie);
                bruce = create_user("Bruce", "bruce@nus.edu.sg", 4, "iambatman");
                add_user(db, bruce);
                joker = create_user("Joker", "joker@nus.edu.sg", 5, "whysoserious");
                add_user(db, joker);
                printf("Mallory wants to login and update her password \n");
                update_password(mallory, "Malloryiswatchingyou");
                printf("Eve wants to get her password => \n");
                char* password = get_password(db, "Eve");
                printf("Eve's password is: %s\n", password);
                break;

        default:
            printf("Unknown payload type: Self destruction\n");
            cleanup_database(db);
            break;
    }
    printf("==============================Final Database State:===========================================\n");
    print_database(db);
    printf("==============================================================================================\n");

    printf("If this program didn't crash, were you lucky ? Check the logs\n");
    return 0;
}
