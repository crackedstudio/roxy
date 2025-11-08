# Multi-stage Dockerfile for Roxy Price (Linera Application)
# Based on Linera buildathon template requirements
# Build stage
FROM rust:1.86.0-slim as builder

# Install build dependencies including Linera requirements
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    curl \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory (template uses /build)
WORKDIR /build

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Copy config file (if it exists)
COPY config.json* ./

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

# Runtime stage - includes Linera CLI and dependencies
# Note: We need Rust/Cargo in runtime stage for run.bash to build
FROM rust:1.86.0-slim

# Install runtime dependencies including Linera CLI requirements
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    bash \
    procps \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    libclang-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install wasm32 target for run.bash builds
RUN rustup target add wasm32-unknown-unknown

# Install Linera CLI using cargo (as per https://linera.dev/developers/getting_started/installation.html)
# Install linera-storage-service and linera-service
# Note: Docker layer caching means this only installs once unless Dockerfile changes
RUN cargo install --locked linera-storage-service@0.15.5 && \
    cargo install --locked linera-service@0.15.5

# Create build directory (template requirement)
WORKDIR /build

# Copy built artifacts from builder
COPY --from=builder /build/target/wasm32-unknown-unknown/release/*.wasm ./
COPY --from=builder /build/target/release/predictive_manager_contract ./
COPY --from=builder /build/target/release/predictive_manager_service ./
# Copy config.json if it exists (optional)
COPY --from=builder /build/config.json* ./

# Copy run.bash script
COPY run.bash ./
RUN chmod +x run.bash

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Default command (will be overridden by compose.yaml)
CMD ["/bin/bash", "run.bash"]

