#!/bin/bash
set -e

echo "=== Roxy Price - Linera Application ==="
echo "Building and running backend..."

# Disable rustup auto-update to avoid DNS issues
export RUSTUP_IO_THREADS=1
export RUSTUP_INIT_SKIP_PATH_CHECK=yes

# Step 1: Check if WASM binaries already exist (from Docker build)
if [ -f "target/wasm32-unknown-unknown/release/predictive_manager_contract.wasm" ]; then
    echo "Step 1: WASM binaries already exist, skipping build..."
else
    echo "Step 1: Building WASM binaries..."
    # Try offline first, fallback to online if needed
    cargo build --release --target wasm32-unknown-unknown --offline 2>/dev/null || \
    cargo build --release --target wasm32-unknown-unknown
fi

# Step 2: Check if native binaries already exist (from Docker build)
if [ -f "target/release/predictive_manager_service" ]; then
    echo "Step 2: Native binaries already exist, skipping build..."
else
    echo "Step 2: Building native binaries..."
    # Try offline first, fallback to online if needed
    cargo build --release --offline 2>/dev/null || \
    cargo build --release
fi

# Step 3: Check if Linera CLI is available
if ! command -v linera &> /dev/null; then
    echo "Warning: Linera CLI not found. Please install it to run the application."
    echo "Visit: https://linera.io/docs/getting-started"
    echo "For now, keeping container running for manual setup..."
    exec tail -f /dev/null
fi

# Step 3: Start Linera localnet
echo "Step 3: Starting Linera localnet..."
echo "Service will be available on:"
echo "  - Faucet: http://localhost:8080"
echo "  - Validator proxy: http://localhost:9001"
echo "  - Validator: http://localhost:13001"
echo ""

# Start Linera localnet with the application
# Note: linera net up uses --faucet-port for the faucet (default: 8080)
# The validator and proxy ports are automatically assigned
linera net up --with-faucet --faucet-port 8080 || {
    echo "Failed to start Linera localnet. Check Linera CLI installation."
    echo "Keeping container running for manual debugging..."
    exec tail -f /dev/null
}

# Step 4: Publish the application (if needed)
# Uncomment and adjust based on your Linera version:
# echo "Step 4: Publishing application..."
# linera publish-and-create predictive_manager_contract.wasm predictive_manager_service.wasm

# Step 5: Application is running
echo "Step 5: Application is running!"
echo "Access the application at:"
echo "  - Frontend: http://localhost:5173 (if applicable)"
echo "  - Faucet: http://localhost:8080"
echo "  - Validator proxy: http://localhost:9001"
echo "  - Validator: http://localhost:13001"
echo ""
echo "Press Ctrl+C to stop the service"

# Keep container running
exec tail -f /dev/null

