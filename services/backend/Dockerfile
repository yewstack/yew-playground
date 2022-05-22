FROM rust:latest as builder

WORKDIR /backend

COPY . .

RUN cargo build --release

EXPOSE 3000

# the app image is built from ../app directory. it contains the build files and the cargo project where we run
# the trunk build command
# replace this with thae name of your app container when building locally
FROM us-docker.pkg.dev/yew-rs/yew-playground/app:latest as runner

WORKDIR /backend

COPY --from=builder /services/backend/target/release/backend .

ENV APP_DIR="/app"
ENV TRUNK_BIN="/usr/bin/trunk"

CMD ["./backend"]
