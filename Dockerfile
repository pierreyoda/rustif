FROM rust:1.60-slim

WORKDIR /usr/src/rustifzm/

COPY Cargo.toml \
    Cargo.lock \
    rustifzm/Cargo.toml \
    ./
COPY rustifzm ./
RUN cargo build

COPY . .
