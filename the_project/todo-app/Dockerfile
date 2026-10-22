FROM rust:1.99-slim-bookworm AS builder
WORKDIR /app
# Copy manifests first so dependency layers are cached between builds
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/todo-app /usr/local/bin/todo-app
EXPOSE 3000
ENV PORT=3000
CMD ["todo-app"]
