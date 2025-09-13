#!/bin/bash

# Script to build and run Rust projects inside Docker container
# Usage: rust_run <project_dir> <binary_name> [args...]

if [ $# -lt 2 ]; then
    echo "Usage: rust_run <project_dir> <binary_name> [args...]"
    echo "Example: rust_run my-project my-binary arg1 arg2"
    exit 1
fi

project_dir="$1"
binary_name="$2"
shift 2 


if [ ! -d "$project_dir" ]; then
    echo "Error: Project directory '$project_dir' not found"
    exit 1
fi


if [ ! -f "$project_dir/Cargo.toml" ]; then
    echo "Error: No Cargo.toml found in '$project_dir'. Not a valid Rust project."
    exit 1
fi

echo "Building Rust project in $project_dir..."


cd "$project_dir"


cargo build --bin "$binary_name"

if [ $? -ne 0 ]; then
    echo "Cargo build failed."
    exit 1
fi

echo "Running $binary_name with arguments: $@"


stdbuf -oL -eL cargo run --bin "$binary_name" -- "$@"
exit_code=$?

if [ $exit_code -eq 139 ]; then
    echo ""
    echo "Program crashed with segmentation fault (exit code 139)"
elif [ $exit_code -ne 0 ]; then
    echo ""
    echo "Program exited with code $exit_code"
fi

exit $exit_code
