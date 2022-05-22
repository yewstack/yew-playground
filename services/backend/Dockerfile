FROM rust:latest as builder

WORKDIR /

# Context must be services dir

COPY ./common /
COPY ./backend /

WORKDIR /backend

RUN cargo build --release

FROM gcr.io/distroless/cc

EXPOSE 3000

COPY --from=builder /services/backend/target/release/backend /

CMD ["/backend"]
