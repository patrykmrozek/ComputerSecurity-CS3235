# Mixed Code Database with C and Rust

This repository contains a mixed codebase with both C and Rust components. The project combines a Rust implementation of a database with an enhanced C database program. The relevant files are:

1. [`database_fix_full`](database-rust/src/database_fix_full) – Your Rust database implementation from Part 1 of the project.
2. [`database_wrapper.rs`](database-rust/src/database_wrapper.rs) – A Rust wrapper that exposes the C database through FFI.
3. [`mixed_code_database.rs`](database-rust/src/mixed_code_database.rs) – The driver code that integrates the Rust and C databases into a single program.
4. [`database_enhanced.c`](database-rust/database_enhanced.c) – The enhanced C database implementation.


## Problem Description

The program currently suffers from several **memory safety issues** in the C code. Many of these issues are similar to the ones addressed in Part 1 of the project. Even after fixing them, the program still fails due to the **`join_databases` operation**.

The `join_databases` function synchronizes state between the Rust and C databases by creating pointers from Rust references and sharing them with C code, and vice versa. As execution progresses, both Rust and C attempt to operate on these shared pointers without coordination. This leads to **undefined behavior**, including crashes, dangling pointers, and potential double frees.


## Project Goal

Your task is to fix the **join operation** so that memory safety is preserved, even in the mixed-language setting. The solution must enforce the **ownership rules of Rust** across both databases.

The recommended approach and rules are detailed in the [rules.md](rules.md) file.

But in summary you should ensure a single owner of each created pointer while still allowing Rust and C to share pointers, without duplicating user entries.


## Instructions on debugging/running the code

Scripts are provided to help build, debug and run the code.

### Method 1: Using the main docker script

To use the main docker-script which builds the docker image and runs the C or Rust code, use the `run_docker` script.

```bash
⟩ ./run_docker
Usage: ./run_docker [--rebuild, --cleanup, --debug, -gen <days>] <C-file> -- <args>
       ./run_docker [--rebuild, --cleanup, --debug, -gen <days>] --rust <project-dir> --bin <binary-name> -- <args>
Options:
  --rebuild    Force rebuild of Docker image
  --rust       Run Rust project instead of C file
  --bin        Specify binary name for Rust project
  --cleanup    Clean up Docker resources after execution
  --debug      Enable debug mode with sanitizers
  -gen <days>  Generate test data for specified number of days

Examples:
  ./run_docker -gen 30 --rust database-rust --bin mixed_code_database
  ./run_docker --debug --rust database-rust --bin mixed_code_database
```

> *Note* : To generate the input data for your program, you can either use the Python script directly or use the integrated `-gen` flag:

```bash
./run_docker -gen <number_of_days> --rust database-rust --bin mixed_code_database
```


### Method 2: Using CapsLock directly (Optional)

You can use the `debug_capslock.sh` script to build and run projects with CapsLock for debugging.

```bash
./debug_capslock.sh -dir <project_directory> -bin <binary_name>

./debug_capslock.sh -dir <project_directory> -bin <binary_name> -gdb

# Examples:
./debug_capslock.sh -dir database-rust/ -bin mixed_code_database
./debug_capslock.sh -dir sample/ -bin sample -gdb
```

**CapsLock Script Options:**

- `-dir`: Specify the project directory to build
- `-bin`: Specify the binary name (just the executable name, e.g., 'mixed_code_database')
- `-gdb`: Enable GDB debugging mode

**Note:** The script automatically constructs the full binary path as:
`<project_dir>/target/riscv64gc-unknown-linux-gnu/debug/<binary_name>`

The script will automatically:

- Check if CapsLock is set up, and set it up if needed
- Build the project with CapsLock's custom compiler
- Run the binary or start a GDB debugging session
- Once in gdb session you have to enter `c` or `continue` twice to start execution/resume
