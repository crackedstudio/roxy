# Makefile for Roxy Price Project
.PHONY: help build test clean docker-build docker-up docker-down docker-dev docker-test

# Default target
help:
	@echo "Roxy Price - Available commands:"
	@echo ""
	@echo "Local Development:"
	@echo "  make build          - Build the project (release mode)"
	@echo "  make build-wasm     - Build WASM binaries"
	@echo "  make test           - Run all tests"
	@echo "  make clippy         - Run clippy linter"
	@echo "  make fmt            - Format code"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build   - Build Docker images"
	@echo "  make docker-up      - Start Docker containers"
	@echo "  make docker-down    - Stop Docker containers"
	@echo "  make docker-dev     - Start development container"
	@echo "  make docker-test    - Run tests in Docker"
	@echo "  make docker-shell   - Access development container shell"

# Local development
build:
	cargo build --release

build-wasm:
	cargo build --release --target wasm32-unknown-unknown

test:
	cargo test --verbose

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clean:
	cargo clean

# Docker commands
docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-dev:
	docker-compose up -d roxy-dev
	@echo "Development container started. Use 'make docker-shell' to access it."

docker-shell:
	docker-compose exec roxy-dev bash

docker-test:
	docker-compose exec roxy-dev cargo test --verbose

docker-build-prod:
	docker build -t roxy:latest .

docker-run-prod:
	docker run -p 8080:8080 roxy:latest

# Fuzzing
fuzz-proptest:
	cargo test --test fuzz_tests

# CI/CD helpers
ci-build: fmt-check clippy build build-wasm test fuzz-proptest

