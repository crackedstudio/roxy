# Multi-stage Dockerfile for Roxy Price (Linera Application)
# Build stage
FROM rust:1.86.0-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Copy source code
COPY src ./src

# Copy tests directory (needed for cargo build)
COPY tests ./tests

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Build the project
RUN cargo build --release --target wasm32-unknown-unknown

# Build native binaries for testing
RUN cargo build --release

# Runtime stage - minimal image for running the application
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy built artifacts from builder
COPY --from=builder /app/target/wasm32-unknown-unknown/release/*.wasm ./
COPY --from=builder /app/target/release/predictive_manager_contract ./
COPY --from=builder /app/target/release/predictive_manager_service ./
COPY --from=builder /app/config.json ./

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Default command (can be overridden)
CMD ["./predictive_manager_service"]

