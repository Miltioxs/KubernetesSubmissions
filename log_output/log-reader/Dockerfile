FROM rust:1.99-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/log-reader /usr/local/bin/log-reader
CMD ["log-reader"]
