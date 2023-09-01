FROM clux/muslrust:stable as builder

WORKDIR /app

# Context must be repo root

RUN cargo new frontend
COPY ./services ./services
COPY Cargo.* ./

RUN cargo build --package backend --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/cc

EXPOSE 3000

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/backend /

CMD ["/backend"]
