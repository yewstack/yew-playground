FROM rust:latest as builder

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown

RUN cargo install --locked trunk

COPY . .

RUN trunk build --release
