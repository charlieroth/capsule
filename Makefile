.PHONY: dev fmt lint test audit deny check help db-up db-down db-health db-wait db-migrate db-reset db-logs install-tools prepare

DB_URL ?= postgres://capsule:capsule_password@localhost:5432/capsule_dev
export DATABASE_URL := $(DB_URL)

# Default target
all: check

# Development target - runs the project in development mode
dev:
	cargo run

# Format code using rustfmt
fmt:
	cargo fmt --all

# Lint code using clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo test --all-features

# Security audit using cargo-audit
audit:
	cargo audit

# Check for banned/denied dependencies using cargo-deny
deny:
	cargo deny check

# Comprehensive check - format, lint, test, and audit
check: fmt lint test audit deny
	@echo "All checks passed!"

# Install required tools (run once)
install-tools:
	cargo install cargo-audit
	cargo install cargo-deny
	cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Clean build artifacts
clean:
	cargo clean

# Build the project
build:
	cargo build

# Build optimized release version
release:
	cargo build --release

# --- Database lifecycle ---

# Start PostgreSQL service via docker compose
db-up:
	docker compose up -d postgres

# Stop PostgreSQL service
db-down:
	docker compose stop postgres

# Check database health (exit 0 if healthy)
db-health:
	./scripts/db-health.sh check

# Wait for database to become healthy (honors DB_WAIT_TIMEOUT)
db-wait:
	./scripts/db-health.sh wait

# Run sqlx migrations against running container
db-migrate: db-wait
	sqlx migrate run

# Nuke volume and recreate DB, then migrate
db-reset:
	docker compose down -v
	docker compose up -d postgres
	$(MAKE) db-wait
	sqlx migrate run

# Tail postgres logs
db-logs:
	docker compose logs -f postgres

# Keep sqlx offline metadata up to date (optional but recommended)
prepare:
	cargo sqlx prepare --workspace -- --all-features

# --- Database Helpers ---
pgcli:
	pgcli $(DB_URL)

# Show help
help:
	@echo "Available targets:"
	@echo "  dev           - Run the project in development mode"
	@echo "  fmt           - Format code using rustfmt"
	@echo "  lint          - Lint code using clippy"
	@echo "  test          - Run tests"
	@echo "  audit         - Security audit using cargo-audit"
	@echo "  deny          - Check for banned dependencies using cargo-deny"
	@echo "  check         - Run all checks (fmt, lint, test, audit, deny)"
	@echo "  install-tools - Install required tools (cargo-audit, cargo-deny, sqlx-cli)"
	@echo "  clean         - Clean build artifacts"
	@echo "  build         - Build the project"
	@echo "  release       - Build optimized release version"
	@echo "  db-up         - Start PostgreSQL container (docker compose)"
	@echo "  db-down       - Stop PostgreSQL container"
	@echo "  db-health     - Check PostgreSQL health (pg_isready/psql)"
	@echo "  db-wait       - Wait for PostgreSQL to become healthy"
	@echo "  db-migrate    - Run database migrations with sqlx"
	@echo "  db-reset      - Drop volumes, recreate DB, run migrations"
	@echo "  db-logs       - Tail PostgreSQL logs"
	@echo "  prepare       - Update sqlx offline metadata (cargo sqlx prepare)"
	@echo "  help          - Show this help message"
