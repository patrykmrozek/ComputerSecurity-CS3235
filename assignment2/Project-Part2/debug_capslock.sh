#!/bin/bash


capslock_dir="$(realpath ../capslock)"

if [ ! -d "$capslock_dir" ]; then
    echo "Capslock directory not found. Setting up capslock..."
    ./setup-capslock.sh "$capslock_dir"
else
    echo "Capslock directory already exists. Skipping setup."
fi



debug=false
shifts=("$@")
# parse args with flags
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -dir) project_dir="$2"; shift ;;
        -bin) binary_name="$2"; shift ;;
        -gdb) debug=true; ;;

    esac
    shift
done

project_dir=$(realpath "$project_dir")
binary_path="$project_dir/target/riscv64gc-unknown-linux-gnu/debug/$binary_name"

if [ -d "$project_dir" ] && [ -n "$binary_name" ]; then
    cd "$capslock_dir" && ./docker-build "$project_dir"
    if [ "$debug" = true ]; then
        ./docker-gdb "$binary_path"
    else
        ./docker-run "$binary_path"
    fi
else
    echo "Project directory or binary name not provided or invalid."
    echo "Usage: ./debug_capslock -dir <project_directory> -bin <binary_name> [-gdb]"
    echo "Note: binary_name should be just the executable name (e.g., 'mixed_code_database')"
    echo "      The full path will be constructed as: <project_dir>/target/riscv64gc-unknown-linux-gnu/debug/<binary_name>"
    exit 1
fi

