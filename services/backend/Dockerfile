FROM rust:latest as builder

WORKDIR /app

# Context must be repo root

COPY ./services .
COPY Cargo.* .

RUN cargo build --package backend --release

FROM gcr.io/distroless/cc

EXPOSE 3000

COPY --from=builder /app/target/release/backend /

CMD ["/backend"]
