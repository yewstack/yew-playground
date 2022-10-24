FROM rust:latest as builder

WORKDIR /app

# Context must be root dir

COPY ./services /app
COPY Cargo.* /app

RUN cargo build --package app-compiler --release


# the app image is built from ../app directory. it contains the build files and the cargo project where we run
# the trunk build command
# replace this with thae name of your app container when building locally
FROM us-docker.pkg.dev/yew-rs/yew-playground/app:latest as runner

WORKDIR /app-compiler

COPY --from=builder /app/target/release/app-compiler .

ENV APP_DIR="/app"
ENV TRUNK_BIN="/usr/bin/trunk"

EXPOSE 4000

CMD ["./app-compiler"]
