FROM rust:latest as builder

WORKDIR /

# Context must be services dir

COPY ./common /common
COPY ./backend /backend

WORKDIR /backend

RUN cargo build --release

FROM gcr.io/distroless/cc

EXPOSE 3000

COPY --from=builder /backend/target/release/backend /

CMD ["/backend"]
