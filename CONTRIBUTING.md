# Contributing to Capsule

Thank you for your interest in contributing to Capsule! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites
- Rust (latest stable version)
- Docker and Docker Compose
- PostgreSQL client tools (optional)

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/charlieroth/capsule.git
   cd capsule
   ```

2. Set up environment variables:
   ```bash
   cp .envrc.template .envrc
   # Edit .envrc with your configuration
   source .envrc  # or use direnv
   ```

3. Start the database:
   ```bash
   make db-up
   make db-migrate
   ```

4. Build and run:
   ```bash
   make dev
   ```

## Development Workflow

### Before Making Changes

1. Ensure all tests pass: `make test`
2. Check code formatting: `make fmt`
3. Run linter: `make lint`
4. Full check: `make check`

### Making Changes

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make your changes following the code style guidelines below
3. Add tests for new functionality
4. Update documentation as needed
5. Run the full check: `make check`

### Submitting Changes

1. Push your branch: `git push origin feature/your-feature`
2. Create a Pull Request
3. Fill out the PR template completely
4. Ensure CI checks pass
5. Address any review feedback

## Code Style Guidelines

### Rust Code
- Use `rustfmt` for formatting: `make fmt`
- Follow clippy recommendations: `make lint`
- Use meaningful variable and function names
- Add docstrings for public APIs
- Use strong typing with custom domain types

### Testing
- Write tests for new functionality
- Use `#[cfg(test)] mod tests` within source files
- Follow naming convention: `test_<function>_<scenario>`
- Mock external dependencies appropriately

### Database
- Use SQLx with compile-time checked queries
- Run `make prepare` after schema changes
- Create migrations for all schema changes
- Document migration purpose in filename

### Imports Organization
```rust
// Standard library
use std::collections::HashMap;

// External crates
use axum::Router;
use sqlx::PgPool;

// Local modules
use crate::entities::User;
```

## Security

- Never commit secrets or API keys
- Use proper JWT handling for authentication
- Follow secure coding practices
- Report security issues via email (see SECURITY.md)

## Questions?

If you have questions about contributing, please:
1. Check existing issues and discussions
2. Open a new issue with the "question" label
3. Reach out to maintainers

Thank you for contributing!
