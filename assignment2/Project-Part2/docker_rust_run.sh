#!/bin/bash

# Script to build and run Rust projects inside Docker container
# Usage: rust_run <project_dir> <binary_name> <debug_mode> [args...]

if [ $# -lt 3 ]; then
    echo "Usage: rust_run <project_dir> <binary_name> <debug_mode> [args...]"
    echo "Example: rust_run my-project my-binary false arg1 arg2"
    exit 1
fi

project_dir="$1"
binary_name="$2"
debug_mode="$3"
shift 3 


if [ ! -d "$project_dir" ]; then
    echo "Error: Project directory '$project_dir' not found"
    exit 1
fi


if [ ! -f "$project_dir/Cargo.toml" ]; then
    echo "Error: No Cargo.toml found in '$project_dir'. Not a valid Rust project."
    exit 1
fi

echo "Building Rust project in $project_dir..."

# Change to project directory
cd "$project_dir"

# Set build flags based on debug mode
if [ "$debug_mode" = "true" ]; then
    echo "Debug mode enabled: Building with sanitizers"
    export RUSTFLAGS="-Zsanitizer=address -Zsanitizer=leak -Copt-level=1"
    export ASAN_OPTIONS="detect_leaks=1:abort_on_error=1"
    cargo +nightly build --bin "$binary_name"
else
    echo "Normal mode: Building without sanitizers"
    cargo build --bin "$binary_name"
fi

if [ $? -ne 0 ]; then
    echo "Cargo build failed."
    exit 1
fi

echo "Running $binary_name with arguments: $@"

# Run with appropriate settings based on debug mode
if [ "$debug_mode" = "true" ]; then
    echo "Running with sanitizers enabled"
    export ASAN_OPTIONS="detect_leaks=1:abort_on_error=1"
    cargo +nightly run --bin "$binary_name" -- "$@"
else
    echo "Running in normal mode"
    cargo run --bin "$binary_name" -- "$@"
fi
exit_code=$?

if [ $exit_code -eq 139 ]; then
    echo ""
    echo "Program crashed with segmentation fault (exit code 139)"
elif [ $exit_code -ne 0 ]; then
    echo ""
    echo "Program exited with code $exit_code"
fi

exit $exit_code
