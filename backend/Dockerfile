FROM rust:1.58.1 as builder

WORKDIR /backend

COPY . .

RUN curl https://raw.githubusercontent.com/hamza1311/waichu/master/.github/workflows/test-build.yml > /dev/null

RUN cargo build --release

EXPOSE 3000
# the app image is built from ../app directory. it contains the build files and the cargo project where we run
# the trunk build command
FROM app as runner

WORKDIR /backend

COPY --from=builder /backend/target/release/backend .

ENV APP_DIR="/playground"
ENV TRUNK_BIN="/usr/bin/trunk"

CMD ["./backend"]
