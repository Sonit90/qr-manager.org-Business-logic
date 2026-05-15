# syntax=docker/dockerfile:1.6

# ---- Build stage ----
FROM rust:1.82-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations

RUN cargo build --release --bin qr-api

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1001 --gid 0 --create-home --shell /sbin/nologin qrapi

COPY --from=builder /app/target/release/qr-api /usr/local/bin/qr-api

USER 1001

EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -fsS http://localhost:3001/api/health || exit 1

CMD ["qr-api"]
