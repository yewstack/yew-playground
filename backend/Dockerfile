FROM rust:latest AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

# Build the backend binary (workspace context)
WORKDIR /build
RUN cargo new frontend && cargo new devtool
COPY ./backend ./backend
COPY Cargo.* ./
RUN cargo build --package backend --release \
    && strip /build/target/release/backend

# Pre-build stable app dependencies for fast cold starts
WORKDIR /app
COPY ./app .
RUN cargo build --target wasm32-unknown-unknown --release
RUN trunk build
RUN rm -rf /app/dist

# Pre-build next app dependencies
WORKDIR /app-next
COPY ./app-next .
RUN cargo build --target wasm32-unknown-unknown --release
RUN trunk build
RUN rm -rf /app-next/dist

# --- Runtime stage: slim base, only what's needed ---
FROM rust:slim

RUN rustup target add wasm32-unknown-unknown \
    && rm -rf /usr/local/rustup/toolchains/*/share/doc \
    && rm -rf /usr/local/cargo/registry

COPY --from=builder /usr/local/cargo/bin/trunk /usr/local/cargo/bin/trunk
COPY --from=builder /usr/local/cargo/git /usr/local/cargo/git
COPY --from=builder /root/.cache /root/.cache
COPY --from=builder /build/target/release/backend /service/playground
COPY --from=builder /app /app
COPY --from=builder /app-next /app-next

ENV APP_DIR_STABLE="/app"
ENV APP_DIR_NEXT="/app-next"
ENV TRUNK_BIN="/usr/local/cargo/bin/trunk"

WORKDIR /service
EXPOSE 3000

CMD ["./playground"]
