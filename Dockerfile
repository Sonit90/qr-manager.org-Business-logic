# syntax=docker/dockerfile:1.7

# ---------- chef ----------
FROM rust:1.95-bookworm AS chef
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-chef --locked
WORKDIR /build

# ---------- planner ----------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------- builder ----------
FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations

RUN cargo build --release --locked --bin qr-api && \
    cp target/release/qr-api /usr/local/bin/qr-api

# ---------- runtime ----------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl tini && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system app && useradd --system --gid app --create-home app

WORKDIR /qr-app
COPY --from=builder /usr/local/bin/qr-api /usr/local/bin/qr-api

USER app
EXPOSE 3001
ENV APP_ENV=production \
    RUST_LOG=info,qr_api=info,sqlx=warn

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -fsS http://localhost:3001/api/health || exit 1

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/qr-api"]
