#!/bin/bash

project_dir=$(realpath "$(pwd)")
# Takes abs path to capslock dir as arg
capslock_dir=$1
git clone https://github.com/jasonyu1996/capslock.git "$capslock_dir"
cd "$capslock_dir"
git checkout libc-interpose
./build-docker
./build-tools
cd "$project_dir"