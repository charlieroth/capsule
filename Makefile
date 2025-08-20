.PHONY: dev fmt lint test audit deny check help db-up db-down

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

# Clean build artifacts
clean:
	cargo clean

# Build the project
build:
	cargo build

# Build optimized release version
release:
	cargo build --release

# Start PostgreSQL service via docker compose
db-up:
	docker compose up -d postgres

# Stop PostgreSQL service
db-down:
	docker compose stop postgres

# Show help
help:
	@echo "Available targets:"
	@echo "  dev          - Run the project in development mode"
	@echo "  fmt          - Format code using rustfmt"
	@echo "  lint         - Lint code using clippy"
	@echo "  test         - Run tests"
	@echo "  audit        - Security audit using cargo-audit"
	@echo "  deny         - Check for banned dependencies using cargo-deny"
	@echo "  check        - Run all checks (fmt, lint, test, audit, deny)"
	@echo "  install-tools- Install required tools (cargo-audit, cargo-deny)"
	@echo "  clean        - Clean build artifacts"
	@echo "  build        - Build the project"
	@echo "  release      - Build optimized release version"
	@echo "  db-up        - Start PostgreSQL container (docker compose)"
	@echo "  db-down      - Stop PostgreSQL container"
	@echo "  help         - Show this help message"
