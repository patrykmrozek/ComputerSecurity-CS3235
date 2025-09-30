# Project Rules: Mixed Codebase

## Overview
This project involves fixing memory safety issues in a mixed Rust/C codebase where both languages share pointers to user data structures. The goal is to enforce memory safety while maintaining the existing architecture and pointer-sharing design.

## Core Rules

- **DO NOT** remove or replace the use of pointers between Rust and C code.  
- **DO NOT** create duplicate in-memory copies of user entries in both databases.  
- **MAINTAIN** a single pointer that coexists in both databases (multiple aliases).  
- **IMPLEMENT** an ownership model: whichever database created the pointer is responsible for deallocating it (especially during join operations).  
- **DO NOT** remove existing free operations; instead, modify them to respect ownership rules (most changes will be in C code).  
- **ENSURE** that read-only marked functions remain unchanged. If modification is unavoidable, add a comment explaining why it was strictly necessary.  
- **DO NOT** move session management responsibilities from C into Rust. Continue to communicate via FFI.  
- **DO NOT** introduce manual memory management in Rust to mimic C behavior.  

At the end of the simulation, both databases must produce identical final states after 30 days.



## Implementation Hints for Join with Ownership Enforcement

1. Consider adding metadata fields like:
```c
typedef enum {
    RUST_OWNED,
    C_OWNED,
    SHARED_RUST_PRIMARY,
    SHARED_C_PRIMARY
} OwnershipType;
```

2. Implement return structures for operations (if required):

```c
typedef struct {
    int success;
    int requires_deallocation;
    int ownership_transfer;
    void* updated_pointer;
} DatabaseOperationResult;
```

Before any pointer operation:
1. Validate pointer is not NULL
2. Check ownership metadata
3. Verify pointer hasn't been invalidated
4. Proceed with operation or delegate to owning code base

## Call Graph (For Reference)


```
main()
├── EnhancedStudentDatabase::new()
│   ├── database_fix_full::init_database()        // Rust DB init
│   └── initialize_enhanced_database()            // C DB init
├── Read the day data (login and signup) from the generated file
│
└── For each day_data:
    ├── Process Signups:
    │   ├── enqueue_user()                        // Queue signup requests
    │   └── sync_database()                       
    │       └── add_user_with_sync()              
    │           ├── c_extensions.sync_user_to_c_backend()  // C allocation path
    │           └── add_user(&mut rust_db)        // Rust allocation path
    │
    ├── Process Logins:
    │   └── login_user()                          
    │       ├── find_user_by_name(&rust_db)       // Check Rust DB first
    │       ├── check in user local references    // (cached references of c)
    │       ├── c_extensions.get_user_in_c_backend() // Check C DB if not found
    │       ├── c_extensions.create_session()     // Create session via C
    │       └── activate_user()                  
    │
    └── increase_day()                           
        ├── sync_database()                       // add enqueued users
        ├── validate_active_user_session()        // expired logins
        ├── update_database_daily(&mut rust_db)   // Rust DB (from part 1)
        ├── join_databases() [EVERY 5 DAYS]      
        │   ├── sync_user_from_rust_db()          // Rust->C pointer sharing
        │   └── get_all_user_references()         // C->Rust pointer sharing
        └── c_extensions.increment_day()          // C DB daily update
            └── update_database_daily(c_db)       
                ├── merge_duplicate_handles()    //Optimization Functions 
                └── memory_pressure_cleanup()    //Optimization Functions
```

