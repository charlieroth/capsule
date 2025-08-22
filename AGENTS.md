# AGENTS.md - Development Guide for Capsule

## Build/Test/Lint Commands
- **Build**: `make build` (dev), `make release` (optimized)
- **Test**: `make test` (all tests), `cargo test <test_name>` (single test)
- **Lint**: `make lint` (clippy with warnings as errors)
- **Format**: `make fmt` (rustfmt)
- **Full Check**: `make check` (fmt + lint + test + audit + deny)
- **Run locally**: `make dev` (starts API on 127.0.0.1:8080)

## Database Commands
- **Setup**: `make db-up` → `make db-migrate`
- **Reset**: `make db-reset` (drops volume, recreates DB)
- **Offline metadata**: `make prepare` (for sqlx compile-time checks)

## Architecture
**Tech Stack**: Rust + Axum + SQLx + PostgreSQL + Tantivy (search) + JWT auth
**Binaries**: `src/bin/api.rs` (HTTP server), `src/bin/migrate.rs` (migration runner)
**Core modules**: `config/` (env config), `entities/` (domain models), `passwords.rs` (argon2 hashing)
**Database**: PostgreSQL with migrations in `migrations/`, ERD via `make erd`

## Code Style & Conventions
- **Tests**: Use `#[cfg(test)] mod tests` within source files, follow `test_<function>_<scenario>` naming
- **Imports**: Group std, external crates, then local modules
- **Error Handling**: Use `anyhow` for application errors, `thiserror` for domain errors
- **Types**: Leverage strong typing with custom domain types in `entities/`
- **Database**: Use SQLx with compile-time checked queries, run `make prepare` after schema changes
- **Security**: Never commit secrets, use proper JWT handling, argon2 for passwords
