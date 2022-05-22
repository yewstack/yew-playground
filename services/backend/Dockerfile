FROM rust:latest as builder

WORKDIR /backend

COPY . .

RUN cargo build --release



FROM gcr.io/distroless/cc

EXPOSE 3000

COPY --from=builder /services/backend/target/release/backend /

CMD ["/backend"]
