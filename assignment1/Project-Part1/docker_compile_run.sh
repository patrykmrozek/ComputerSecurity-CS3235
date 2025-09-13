#!/bin/bash

# Script to compile and run C files inside Docker container
# Usage: compile_run <c_file> [args...]

if [ $# -lt 1 ]; then
    echo "Usage: compile_run <c_file> [args...]"
    echo "Example: compile_run program.c arg1 arg2"
    exit 1
fi

c_file="$1"
shift


if [ ! -f "$c_file" ]; then
    echo "Error: C file '$c_file' not found"
    exit 1
fi


filename=$(basename "$c_file" .c)

echo "Compiling $c_file..."


gcc -fno-stack-protector -fcf-protection=none -Wformat-overflow=0 -D_FORTIFY_SOURCE=0 \
    -fno-delete-null-pointer-checks -no-pie -z norelro -z execstack -O0 -g \
    -o "$filename" "$c_file"

if [ $? -ne 0 ]; then
    echo "Compilation failed."
    exit 1
fi

echo "Running $filename with arguments: $@"


GLIBC_TUNABLES=glibc.malloc.tcache_count=0 stdbuf -oL -eL ./"$filename" "$@"
exit_code=$?

if [ $exit_code -eq 139 ]; then
    echo ""
    echo "Program crashed with segmentation fault (exit code 139)"
elif [ $exit_code -ne 0 ]; then
    echo ""
    echo "Program exited with code $exit_code"
fi

exit $exit_code
