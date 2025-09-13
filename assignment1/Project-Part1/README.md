# Project Part 1 : Translating Unsafe C-Code to Safe Rust


This directory contains the file `database.c` which has multiple memory safety vulnerablities. There are some payloads provided that can exploit these vulnerabilities. Your task is to translate this C code to safe Rust code in [`database-rust/src/database_fix_full.rs`](database-rust/src/database_fix_full.rs) such that the payloads no longer work. The translation should be function by function and you should not change the structs defined in the skeleton of `database_fix_full.rs`. You can add helper functions if needed, but all the C-code functions should have a corresponding Rust function.

The only dependencies you have to setup is installing Docker on your machine. You can find instructions for installing Docker [here](https://docs.docker.com/engine/install/).


To help contain the environment, a Dockerfile is provided that sets up a container to run the C code and the Rust code.

> **Note: Do Not add any extra dependencies to the Rust project, do not use `unsafe` blocks in Rust, and lastly do not use FFI.**



## Instructions on building and running the code

1. To use the main docker-script which builds the docker image and runs the C or Rust code, use the `run_docker` script.


```bash
~/Desktop/Workspace/Project-Part1
⟩ ./run_docker
Usage: ./run_docker [--rebuild, --cleanup] <C-file> -- <args>
       ./run_docker [--rebuild, --cleanup] --rust <project-dir> --bin <binary-name> -- <args>
Options:
  --rebuild    Force rebuild of Docker image
  --rust       Run Rust project instead of C file
  --bin        Specify binary name for Rust project
  --cleanup    Clean up Docker resources after execution
```


2. To run the C code with a payload, you can do:

```bash
./run_docker database.c -- <0/1/2>
```

where `0`, `1` and `2` are the payloads corresponding to Buffer Overflow, Double Free and Use After Free respectively.


For example, to run the Buffer Overflow payload:

```bash
⟩ ./run_docker database.c -- 0
C mode: File 'database.c'
Using existing Docker image 'code-runner'.
Running C file database.c in Docker container...
Compiling database.c...
Running database with arguments: 0
Running Test Payload OUT_OF_BOUNDS_PAYLOAD
User: Mallory, ID: 0, Email: mallory@nus.edu.sg, Inactivity: 0  Password = malloryisnotevil
User: Alice, ID: 1, Email: alice@nus.edu.sg, Inactivity: 0  Password = aliceinthewonderland
User: Bob, ID: 2, Email: bob@nus.edu.sg, Inactivity: 0  Password = bobthebuilder
User: Eve, ID: 3, Email: eve@nus.edu.sg, Inactivity: 0  Password = eve4ever
==============================Final Database State:===========================================
User: CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC, ID: 0, Email: mallory@nus.edu.sg, Inactivity: 0  Password = malloryisnotevil
User: Alice, ID: 1, Email: alice@nus.edu.sg, Inactivity: 0  Password = CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
User: Bob, ID: 2, Email: bob@nus.edu.sg, Inactivity: 0  Password = bobthebuilder
User: Eve, ID: 3, Email: eve@nus.edu.sg, Inactivity: 0  Password = eve4ever
==============================================================================================
If this program didn't crash, were you lucky ? Check the logs
```

3. The Rust code is added to the [`database-rust/src/database_fix_full.rs`](database-rust/src/database_fix_full.rs) file. To run the Rust code, you can do:

```bash
./run_docker --rust database-rust --bin database_fix_full -- <Program-Args>
```

remember it will be after you have translated the C code to Rust code, The program args are the same as the C code.

For example the same translation of the Buffer Overflow payload would be:

```bash
./run_docker --rust database-rust --bin database_fix_full -- 0
Rust mode: Project 'database-rust', Binary 'database_fix_full'
Using existing Docker image 'code-runner'.
Running Rust project database-rust (binary: database_fix_full) in Docker container...
Building Rust project in ....
   Compiling database-rust v0.1.0 (/workspace/database-rust)
....

Running Test Payload OutOfBoundsPayload
User: Mallory, ID: 1, Email: mallory@nus.edu.sg, Inactivity: 0  Password = malloryisnotevil
User: Alice, ID: 2, Email: alice@nus.edu.sg, Inactivity: 0  Password = aliceinthewonderland
User: Bob, ID: 3, Email: bob@nus.edu.sg, Inactivity: 0  Password = bobthebuilder
User: Eve, ID: 4, Email: eve@nus.edu.sg, Inactivity: 0  Password = eve4ever
==============================Final Database State:===========================================
User: CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC, ID: 1, Email: mallory@nus.edu.sg, Inactivity: 0  Password = malloryisnotevil
User: Alice, ID: 2, Email: alice@nus.edu.sg, Inactivity: 0  Password = aliceinthewonderland
User: Bob, ID: 3, Email: bob@nus.edu.sg, Inactivity: 0  Password = bobthebuilder
User: Eve, ID: 4, Email: eve@nus.edu.sg, Inactivity: 0  Password = eve4ever
==============================================================================================
If this program didn't crash, were you lucky ? Check the logs
```


## Rules (Summary)

- Do **not** add any extra Rust dependencies.
- Do **not** use `unsafe` or FFI.
- Translate **function-by-function**: every C function must have a corresponding Rust function (extra helper functions allowed).
- The expected output is provided in `sample_output.md`.
- Do **not** change the struct definitions already provided in the Rust skeleton.