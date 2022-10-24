FROM rust:latest as builder

WORKDIR /app

# Context must be repo root

COPY ./services/common /app/common
COPY ./services/backend /app/backend
COPY ./Cargo.toml /app

WORKDIR /app/backend

RUN cargo build --release

FROM gcr.io/distroless/cc

EXPOSE 3000

COPY --from=builder /app/backend/target/release/backend /

CMD ["/backend"]
