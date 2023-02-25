FROM rust:latest

WORKDIR /app

RUN apt update; apt upgrade -y 
RUN apt install -y g++-mingw-w64-x86-64 gcc-mingw-w64-x86-64 g++-mingw-w64-i686 gcc-mingw-w64-i686

RUN rustup toolchain install nightly-x86_64-unknown-linux-gnu
RUN rustup toolchain install nightly-x86_64-pc-windows-gnu
RUN rustup toolchain install nightly-i686-pc-windows-gnu
RUN rustup default nightly

RUN cargo install cargo-make

CMD ["cargo", "make", "--makefile", "Makefile.toml"]