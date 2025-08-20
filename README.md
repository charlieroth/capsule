# capsule

A pragmatic "read later" service built in Rust to explore production-grade web service patterns:

- Authentication
- Background jobs
- Full‑text search
- Observability
- Robust operations

## Features (Planned / In Progress)

- REST API with generated OpenAPI docs (axum + utoipa)
- User authentication (argon2 password hashes + JWT)
- Background fetch & extract pipeline (reqwest + scraper)
- Full‑text search (tantivy)
- Structured logging & tracing (tracing) + metrics (future Prometheus endpoint)
- Database persistence (PostgreSQL via sqlx; async, compile‑time checked queries when `make prepare` is run)
- Schema documentation / ERD via SchemaSpy (`make erd` -> `./erd/index.html`)

## Architecture Overview

High-level flow:

1. Client creates an item (URL + metadata)
2. A background task fetches & normalizes HTML, stores text content
3. Indexer updates tantivy with (title + site + tags + text)
4. Search endpoint returns ranked results with snippets

## Repository Layout

```
src/
  bin/
    api.rs        # HTTP server entrypoint
    migrate.rs    # One-shot migration runner (used in Docker / local)
  lib.rs          # (future) shared library code
migrations/       # sqlx migrations (*.up.sql / *.down.sql)
Makefile          # Developer workflow commands
Dockerfile        # Multi-stage container build (api + migrate)
docker-compose.yml# Postgres + migrate + api + schemaspy services
scripts/db-health.sh # Wait/health checks for Postgres
erd/              # Generated SchemaSpy output (HTML + diagrams)
docs/PROJECT.md   # Vision, roadmap, non-functional goals
```

## Quick Start (Local Dev)

Pre-requisites:

- Rust (see `rust-toolchain.toml`)
- Docker (for Postgres + ERD generation)
- Run `make install-tools`

Steps:

```bash
# 1. Start Postgres
make db-up

# 2. Run migrations
make db-migrate

# 3. Launch the API (defaults to 0.0.0.0:8080 via Config)
make dev

# 4. Hit the root endpoint
curl -s localhost:8080/
```

Expected response: `Hello from capsule!`

Tear down:

```bash
make db-down
```

Full reset (drops volume):

```bash
make db-reset
```

## Configuration

`Config::from_env()` (see `config/mod.rs`) loads environment variables. Key variable:

- `DATABASE_URL` (required for API & migrations) e.g. `postgres://capsule:capsule_password@localhost:5432/capsule_dev`

Additional configuration knobs (future): bind address, logging level, JWT secrets, rate limits.

## Database & Migrations

Migrations live in `migrations/` and are executed by either:

- `make db-migrate` (sqlx-cli) OR
- The `capsule-migrate` binary (used in `docker-compose.yml` as the `migrate` service)

Generate / update sqlx offline metadata (speeds up compile-time query checking):

```bash
make prepare
```

Check database health:

```bash
make db-health    # exit 0 if healthy
make db-wait      # block until healthy (used in CI / scripts)
```

Open a psql-like shell (requires `pgcli` installed):

```bash
make pgcli
```

## Schema / ERD Docs

Generate ERD & HTML docs (writes into `./erd`):

```bash
make erd
open erd/index.html  # macOS
```

## Docker / Compose

Build & run everything (Postgres + migrations + API):

```bash
docker compose up --build api
```

Services:

- `postgres` (port 5432)
- `migrate` (runs once; executes migrations then exits)
- `api` (exposes port 8080)
- `schemaspy` (on-demand ERD generation: `make erd`)

Environment is baked with `DATABASE_URL` pointing at the compose network host.

## Makefile Cheat Sheet

| Target       | Purpose                          |
| ------------ | -------------------------------- |
| `dev`        | Run API locally (debug)          |
| `fmt`        | Format sources                   |
| `lint`       | Clippy (deny warnings)           |
| `test`       | Run tests                        |
| `audit`      | Security audit (cargo-audit)     |
| `deny`       | Dependency policy (cargo-deny)   |
| `check`      | fmt + lint + test + audit + deny |
| `db-up`      | Start Postgres via Docker        |
| `db-down`    | Stop Postgres                    |
| `db-migrate` | Apply migrations                 |
| `db-reset`   | Drop volume & reinit DB          |
| `db-health`  | Health probe (fast)              |
| `db-wait`    | Wait until healthy               |
| `db-logs`    | Tail Postgres logs               |
| `prepare`    | sqlx offline metadata            |
| `erd`        | Generate ERD docs                |

Install tooling once:

```bash
make install-tools
```

## Testing Strategy (Planned)

- Unit tests for parsing, auth, extraction
- Integration tests exercising HTTP routes & DB side-effects
- Property tests (URL normalization; idempotent job enqueue)
- Fuzzing extractor inputs

Run tests:

```bash
make test
```
