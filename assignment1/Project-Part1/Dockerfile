FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    software-properties-common \
    curl \
    && add-apt-repository ppa:ubuntu-toolchain-r/test \
    && apt-get update && apt-get install -y \
    gcc-11 g++-11\
    gdb \
    build-essential \
    locales \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN locale-gen en_US.UTF-8
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8
ENV GLIBC_TUNABLES=glibc.malloc.tcache_count=0

RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-11 100 \
 && update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-11 100

# Create scripts to compile and run C files and Rust projects
COPY docker_compile_run.sh /usr/local/bin/compile_run
COPY docker_rust_run.sh /usr/local/bin/rust_run
RUN chmod +x /usr/local/bin/compile_run /usr/local/bin/rust_run

WORKDIR /workspace

# Default command runs bash, but can be overridden
CMD [ "bash" ]
