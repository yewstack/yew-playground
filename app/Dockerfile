FROM rust:latest as builder

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown && \
    TRUNK_VERSION=$(curl -s https://api.github.com/repos/thedodd/trunk/releases/latest | grep -oP '(?<="tag_name": ")[^"]*') && \
    wget -qO- https://github.com/thedodd/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- && \
    mv trunk /usr/bin

COPY . .

RUN ls *

RUN trunk build --release --dist /playground
