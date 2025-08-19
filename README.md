# `capsule`

A "read later" web service, learning to build production-grade Rust web services

## Outcome

The goal of this project is to build a service with the following components:

- REST + OpenAPI service
- User auth
- Background fetching
- Full-text search
- Tracing
- Monitoring via Prometheus

## Stack

- Server: `axum`, `tokio`, `tower`, `sqlx`, `serde`, `utoipa`, `tracing`
- Jobs: background tasks via `tokio::spawn` + `jobs` table
- Fetch + Extract: `reqwest`, `scraper` or `kuchiki`, optional reability pass
- Search: `tantivy` in-process
- Auth: `argon2` password hashes, JWT (`jsonwebtoken`)

## Flow

1. Create item -> enqueue `fetch_and_extract(item_id)`
2. Worker fetches HTML, normalizes, stores `content.text`, updates `status=finished`
3. Indexer ingests `(title + site + tags + text)` to `tantivy`
4. Search returns ranked hits with snippets

## Non-functional

Some non-functional aspects I want to include in this project are:

- Timeouts, retries, circuit breaking (`tower`)
- Graceful shutdown with in-flight drain
- Per-user rate limits
- PII minimization, secure cookies for web, HTTPS

## Tests

Different testing techniques I want to explore in this project:

- Unit: extractors, parsers, auth
- Integration: happy paths + failure modes (`5XX`, timeouts)
- Property: URL normilization, idempotent enqueue
- Load: 1k items user, `p95` search < 100ms on local machine
- Fuzz: HTML extractor inputs

## Roadmap

I have the following roadmap for `v0.1.0`:

- Milestone 1: schema, migrations, auth, skeleton routes, CI
- Milestone 2: fetcher, extractor, jobs, content store, tracing
- Milestone 3: `tantivy` index + search API, snippets, tags, rate limit
- Milestone 4: hardening, docs, seed data, `Dockerfile`

## Stretch Goals

- Read-it-later browser extension (`POST` to API)
- RSS import, EPUB export
- Per-user encryption at rest (key per user)
