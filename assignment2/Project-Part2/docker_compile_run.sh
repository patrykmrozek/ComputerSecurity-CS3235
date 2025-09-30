#!/bin/bash

# Script to compile and run C files inside Docker container
# Usage: compile_run <c_file> <debug_mode> [args...]

if [ $# -lt 2 ]; then
    echo "Usage: compile_run <c_file> <debug_mode> [args...]"
    echo "Example: compile_run program.c false arg1 arg2"
    exit 1
fi

c_file="$1"
debug_mode="$2"
shift 2


if [ ! -f "$c_file" ]; then
    echo "Error: C file '$c_file' not found"
    exit 1
fi


filename=$(basename "$c_file" .c)

echo "Compiling $c_file..."

# Set compilation flags based on debug mode
if [ "$debug_mode" = "true" ]; then
    echo "Debug mode enabled: Adding sanitizers"
    gcc_flags="-fsanitize=address -fsanitize=undefined -fsanitize-address-use-after-scope -fno-omit-frame-pointer -g -O1"
else
    echo "Normal mode: Disabling security features"
    gcc_flags="-fno-stack-protector -fcf-protection=none -Wformat-overflow=0 -D_FORTIFY_SOURCE=0 -fno-delete-null-pointer-checks -no-pie -z norelro -z execstack -O0 -g"
fi

gcc $gcc_flags -o "$filename" "$c_file"

if [ $? -ne 0 ]; then
    echo "Compilation failed."
    exit 1
fi

echo "Running $filename with arguments: $@"


if [ "$debug_mode" = "true" ]; then
    echo "Running with sanitizers enabled"
    ASAN_OPTIONS="detect_leaks=1:abort_on_error=1:check_initialization_order=1" \
    UBSAN_OPTIONS="print_stacktrace=1:abort_on_error=1" \
    ./"$filename" "$@"
else
    echo "Running with security mitigations disabled"
    GLIBC_TUNABLES=glibc.malloc.tcache_count=0 ./"$filename" "$@"
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
