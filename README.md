# QR Link

Dynamic QR code backend. Users create named QR codes pointing to a redirect URL; the destination can be changed at any time, so a printed QR keeps working even after the target moves. Scans are logged with browser/device/IP metadata. PNG images are generated on create and stored in RustFS.

## Tech stack

- **Framework**: Axum 0.8 + Tower
- **DB**: PostgreSQL 15, SQLx 0.8, `time` crate
- **Object storage**: RustFS (S3-compatible) via `aws-sdk-s3`
- **Auth**: JWT validation via [`rust-generic-user-backend`](https://github.com/Sonit90/rust-generic-user-backend) (`generic-auth-auth` crate, git dep)
- **Logging**: `tracing` (pretty in dev, JSON in prod — toggled by `APP_ENV`)
- **OpenAPI**: `utoipa` annotations → Swagger UI at `/api/docs`
- **QR**: `qrcode` + `image` crates → PNG bytes

## Architecture

Workspace with 3 crates:

```
crates/
├── api/      Axum HTTP server, routes, middleware, storage client, OpenAPI
├── domain/   QR manager, scan logger, image generator
└── db/       SQLx models + repositories
```

**Database**: shared Postgres instance owned by auth service.

- `auth.*` — owned by rust-generic-user-backend (Postgres at `localhost:5434`). Tables: `users`, `roles`, `refresh_tokens`, etc.
- `qr.*`   — owned by this service (same DB, different schema)

Each service tracks its migrations in its OWN `_sqlx_migrations` table — auth's in `auth._sqlx_migrations`, qr's in `qr._sqlx_migrations`. Per-connection `search_path` override gives each service isolated migration tracking.

FK: `qr.qr_codes.user_id REFERENCES auth.users(id) ON DELETE CASCADE`. Deleting a user cascades to their QR codes (no orphans).

**Object storage**: PNG images live in RustFS bucket `qr-images`, keyed `{qr_id}.png`. Bucket is publicly readable; URLs returned in API responses are direct.

## Setup

### Prerequisites

- Docker + Docker Compose v2
- Auth service running ([rust-generic-user-backend](https://github.com/Sonit90/rust-generic-user-backend)) — provides Postgres on `localhost:5434` with `auth.*` schema.

### 1. Configure env

```sh
cp .env.example .env
```

Edit `.env` and set the values that must match the auth service:

| Variable                  | Action                                              |
|---------------------------|-----------------------------------------------------|
| `APP__AUTH__JWT_SECRET`   | Copy from auth's config — must match exactly        |
| `APP__AUTH__JWT_ISSUER`   | Match auth's issuer (default `generic-auth`)        |
| `DATABASE_URL`            | Native run: `localhost:5434` to auth's Postgres     |
| `DOCKER_DB_URL`           | Docker run: `host.docker.internal:5434`             |
| `RUSTFS_ROOT_USER`/`_PASSWORD` | RustFS admin credentials (also used as S3 keys) |

All other vars have sensible defaults — see `.env.example` for the full list.

### 2A. Run with Docker (recommended)

```sh
docker compose up --build -d
```

Brings up three containers:
- `${RUSTFS_CONTAINER_NAME}` — RustFS S3 on host port `${RUSTFS_S3_PORT}`, console on `${RUSTFS_CONSOLE_PORT}`
- `${RUSTFS_INIT_CONTAINER_NAME}` — one-shot bucket creator (exits after success)
- `${QR_API_CONTAINER_NAME}` — qr-api on host port `${QR_API_HOST_PORT}`

qr-api migrations run automatically. Logs:
```sh
docker compose logs -f qr-api
```

### 2B. Run natively (development)

Requires Postgres + RustFS running (use docker compose for the latter):

```sh
docker compose up -d rustfs rustfs-init
cargo run -p qr-api
```

Server reads vars from `.env` directly (note: native run uses `DATABASE_URL`, not `DOCKER_DB_URL`).

### Verify

- `http://localhost:3001/api/health` → `ok`
- `http://localhost:3001/api/docs` → Swagger UI
- `http://localhost:9001` → RustFS console (login = `RUSTFS_ROOT_USER` / `RUSTFS_ROOT_PASSWORD`)

## Deployment

### Production checklist

1. **Secrets**: rotate `APP__AUTH__JWT_SECRET`, `RUSTFS_ROOT_USER`, `RUSTFS_ROOT_PASSWORD`. Never commit `.env`.
2. **APP_ENV**: set to `production` → JSON tracing + strict CORS.
3. **CORS**: set `APP__CORS__ALLOWED_ORIGINS` to comma-separated frontend origins.
4. **Storage URL**: set `APP__STORAGE__PUBLIC_BASE_URL` to your CDN / reverse-proxy URL (NOT `localhost`).
5. **RustFS access**: production should NOT expose `RUSTFS_S3_PORT` publicly. Front with CDN or reverse-proxy that serves `qr-images/*` read-only.
6. **DB**: point `DOCKER_DB_URL` to managed Postgres. Drop `host.docker.internal` — use proper DNS/connection string.
7. **TLS**: terminate at reverse-proxy (nginx, traefik, caddy). qr-api speaks plain HTTP behind it.

### Build production image

```sh
docker build -t qr-api:$(git rev-parse --short HEAD) .
```

### Image tag override

In `.env`:
```
QR_API_IMAGE=registry.example.com/qr-api
QR_API_IMAGE_TAG=v0.1.0
```

### Migrations

Migrations are embedded in the binary at compile time via `sqlx::migrate!`. They run on every `qr-api` startup against the `qr` schema. No separate migration step needed.

### Volume backups

- `rustfs_data` named volume holds all uploaded PNGs. Snapshot regularly.
- Postgres is owned by the auth service — back up there.

## API endpoints (v1)

### QR Management

| Method | Path                          | Auth   | Description                              |
|--------|-------------------------------|--------|------------------------------------------|
| POST   | `/api/v1/qr`                  | Bearer | Create QR + initial target URL + name; PNG uploaded to RustFS |
| GET    | `/api/v1/qr`                  | Bearer | List authenticated user's QRs            |
| GET    | `/api/v1/qr/{id}`             | Bearer | Get QR details + scan count + image URL  |
| DELETE | `/api/v1/qr/{id}`             | Bearer | Delete QR + cascade scans                |
| PATCH  | `/api/v1/qr/{id}/enabled`     | Bearer | Toggle enabled/disabled                  |
| PATCH  | `/api/v1/qr/{id}/name`        | Bearer | Update QR name                           |
| GET    | `/api/v1/qr/{id}/scans`       | Bearer | Scan stats + recent logs                 |
| PUT    | `/api/v1/qr/{id}/target`      | Bearer | Update target URL (appends new version)  |

### Public

| Method | Path              | Auth | Description                                              |
|--------|-------------------|------|----------------------------------------------------------|
| GET    | `/api/health`     | —    | Liveness check, returns `ok`                             |
| GET    | `/r/{qr_code}`    | —    | Log scan, redirect to target (or disabled-frontend URL)  |

### OpenAPI

| Method | Path                          | Auth | Description                          |
|--------|-------------------------------|------|--------------------------------------|
| GET    | `/api/docs/openapi.json`      | —    | OpenAPI spec (generated by utoipa)   |
| GET    | `/api/docs`                   | —    | Interactive API explorer (Swagger)   |

## Configuration

| Variable                              | Required | Default                                    | Notes                                  |
|---------------------------------------|----------|--------------------------------------------|----------------------------------------|
| `APP_ENV`                             | no       | `development`                              | `development` → pretty tracing + permissive CORS |
| `RUST_LOG`                            | no       | `info,qr_api=debug,sqlx=warn`              |                                        |
| `DATABASE_URL`                        | **yes**  | —                                          | Shared Postgres URL                    |
| `SERVER_HOST`                         | no       | `127.0.0.1`                                | Use `0.0.0.0` in Docker                |
| `SERVER_PORT`                         | no       | `3001`                                     |                                        |
| `APP__AUTH__JWT_SECRET`               | **yes**  | —                                          | MUST match auth service                |
| `APP__AUTH__JWT_ISSUER`               | no       | `generic-auth`                             | Must match `iss` claim issued          |
| `APP__AUTH__JWT_ACCESS_TTL_MIN`       | no       | `30`                                       |                                        |
| `APP__AUTH__JWT_REFRESH_TTL_DAYS`     | no       | `14`                                       |                                        |
| `APP__CORS__ALLOWED_ORIGINS`          | prod     | empty                                      | Comma-separated; ignored in dev        |
| `APP__STORAGE__ENDPOINT_URL`          | **yes**  | —                                          | RustFS/S3 endpoint (e.g. `http://rustfs:9000`) |
| `APP__STORAGE__BUCKET`                | no       | `qr-images`                                |                                        |
| `APP__STORAGE__REGION`                | no       | `us-east-1`                                | S3-compatibility placeholder           |
| `APP__STORAGE__ACCESS_KEY`            | **yes**  | —                                          |                                        |
| `APP__STORAGE__SECRET_KEY`            | **yes**  | —                                          |                                        |
| `APP__STORAGE__PUBLIC_BASE_URL`       | **yes**  | —                                          | Prefix used in `image_url` responses   |
| `DISABLED_QR_FRONTEND_URL`            | no       | `http://localhost:3000/qr-disabled`        | Redirect target for disabled QRs       |
| `REDIRECT_BASE_URL`                   | no       | `http://localhost:3001/r`                  | Embedded in PNG QR images              |

## Auth flow

1. Client logs into the auth service (`rust-generic-user-backend`) — gets JWT
2. Client calls qr-api with `Authorization: Bearer <jwt>`
3. `require_auth` middleware validates token via `JwtCodec.verify()` from `generic-auth-auth`
4. `Claims` inserted into request extensions
5. `AuthUser` extractor pulls `Claims.sub` (user UUID) for ownership checks

JWT secret + issuer must match auth service's config.

## QR image lifecycle

- **On create** (`POST /api/v1/qr`): PNG generated (referring to `/r/{code}`), uploaded to RustFS at `{qr_id}.png`, public URL stored in `qr.qr_codes.image_url`.
- **On read** (`GET /api/v1/qr/{id}`): if `image_url` is null (legacy row or upload failed), it's regenerated + uploaded lazily, then saved.
- The QR's `code` (and thus scan URL) never changes — only `target_url` does. Stored PNG stays valid forever.

## Disabled QR behavior

When `qr.qr_codes.enabled = false`:
- Scan is still logged (for analytics)
- `/r/{qr_code}` redirects to `DISABLED_QR_FRONTEND_URL?qr_id=<uuid>`
- Frontend reads `qr_id` query param, shows "disabled" notification

## Adding a new endpoint

Per `CLAUDE.md`:
1. Add handler to `crates/api/src/routes/`
2. Register route in `crates/api/src/routes/mod.rs`
3. Add `#[utoipa::path(...)]` annotation
4. Add row to the correct table above (sorted by path within group)

## Development

```sh
cargo check                 # type-check
cargo build --bin qr-api    # build binary
cargo test                  # run unit tests
cargo run -p qr-api         # start server (requires running RustFS + Postgres)
docker compose up --build   # full stack in containers
```
