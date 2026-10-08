# Dev stage: live-reload capable Rust environment with sqlx-cli installed.
FROM rust:1.90-slim-bookworm AS dev

WORKDIR /app

RUN apt-get update \
    && apt-get install -y pkg-config libssl-dev postgresql-client \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install sqlx-cli --version 0.8.6 --locked --no-default-features --features native-tls,postgres

ENV SQLX_OFFLINE=false
ENV CARGO_TARGET_DIR=/app/target

# Cache dependencies by copying workspace manifests first.
COPY Cargo.toml ./
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/migrations/Cargo.toml ./crates/migrations/Cargo.toml
COPY crates/domain/Cargo.toml ./crates/domain/Cargo.toml

RUN mkdir -p crates/api/src crates/migrations/src crates/domain/src \
    && echo 'fn main() {}' > crates/api/src/main.rs \
    && echo 'fn main() {}' > crates/migrations/src/main.rs \
    && echo '' > crates/domain/src/lib.rs \
    && cargo build --release \
    && rm -rf crates/api/src crates/migrations/src crates/domain/src

CMD ["sh", "-c", "sqlx migrate run --source crates/migrations/migrations && cargo run --release -p power-os-api"]

# Frontend builder
FROM node:20-alpine AS frontend-builder

WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./frontend/
RUN cd frontend && npm install
COPY frontend ./frontend
RUN cd frontend && npm run build

# Production builder
FROM dev AS builder

COPY . .
RUN cargo build --release

# Production runtime
FROM debian:bookworm-slim AS production

RUN apt-get update \
    && apt-get install -y ca-certificates postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/power-os-api /usr/local/bin/power-os-api
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

EXPOSE 3000

CMD ["power-os-api"]
