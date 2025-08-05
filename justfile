# Justfile for typst-oxide
# A language server for Personal Knowledge Management (PKM) systems using Typst

# Default recipe - show available commands
default:
    @just --list

# Development commands

# Install development dependencies
setup:
    cargo install cargo-watch
    cargo install cargo-audit
    pre-commit install

# Run the language server in development mode
dev:
    cargo run

# Watch for changes and rebuild
watch:
    cargo watch -x check -x test -x run

# Build commands

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Testing commands

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run tests and show coverage
test-coverage:
    cargo test --all-features

# Code quality commands

# Format code
fmt:
    cargo fmt --all

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Check code without building
check:
    cargo check --all-targets --all-features

# Run all quality checks (format, lint, test)
ci: fmt-check lint test
    @echo "All CI checks passed!"

# Security and maintenance

# Audit dependencies for security vulnerabilities
audit:
    cargo audit

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Documentation

# Generate and open documentation
docs:
    cargo doc --open

# Generate documentation without opening
docs-build:
    cargo doc --no-deps

# Release commands

# Prepare for release (run all checks)
pre-release: ci audit
    @echo "Ready for release!"

# Release a new version (requires version argument)
release version:
    ./scripts/release.sh {{version}}

# Git and pre-commit

# Run pre-commit hooks on all files
pre-commit:
    pre-commit run --all-files

# Install pre-commit hooks
pre-commit-install:
    pre-commit install

# Utility commands

# Show project information
info:
    @echo "Project: typst-oxide"
    @echo "Description: A language server for Personal Knowledge Management (PKM) systems using Typst"
    @echo "Version: $(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"

# Install the binary locally
install:
    cargo install --path .

# Uninstall the binary
uninstall:
    cargo uninstall typst-oxide

# Run benchmarks (if any)
bench:
    cargo bench

# Example usage for development
example:
    @echo "Example usage:"
    @echo "  just dev          # Run in development mode"
    @echo "  just watch        # Watch for changes"
    @echo "  just test         # Run tests"
    @echo "  just ci           # Run all CI checks"
    @echo "  just release 0.1.1 # Release version 0.1.1"