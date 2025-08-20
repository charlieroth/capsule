# build
FROM rust:1.85-bookworm AS build
WORKDIR /app
# cache deps
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --bin migrate
RUN cargo build --release --bin api

# runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/migrate /app/capsule-migrate
COPY --from=build /app/target/release/api /app/capsule-api