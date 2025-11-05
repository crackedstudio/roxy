# Quick Start Guide - Docker Setup



```bash
# Build dev image (faster - just installs tools)
docker build -f Dockerfile.dev -t roxy-dev .

# Run with volume mounting (instant rebuilds)
docker run -it --rm \
  -v $(pwd):/app \
  -v cargo-cache:/usr/local/cargo/registry \
  -v target-cache:/app/target \
  roxy-dev bash

# Inside container, build and test
cargo build
cargo test
```

## Using Docker Compose (Easiest)

```bash
# Start development container (uses volumes - no rebuild needed)
docker-compose up -d roxy-dev

# Access container shell
docker-compose exec roxy-dev bash

# Build inside container (uses cached volumes)
cargo build --release
```
# Access the container shell
```bash
docker-compose exec roxy-dev bash
```

# Or run commands directly
```bash
docker-compose exec roxy-dev cargo build
docker-compose exec roxy-dev cargo test
docker-compose exec roxy-dev cargo check
```

# Stop containers
```bash
docker-compose down
```

# Start again
```bash
docker-compose up -d roxy-dev
```

## Memory Issues? (Linker Killed)

If you see `ld terminated with signal 9 [Killed]`, it's an out-of-memory issue.

**Quick Fix:**
```bash
# Run tests with single job (uses less memory)
docker-compose exec roxy-dev cargo test --jobs 1

# Or build in release mode (uses less memory)
docker-compose exec roxy-dev cargo test --release --jobs 1
```

**Better Solution:** Increase Docker Desktop memory limit:
1. Docker Desktop → Settings → Resources → Advanced
2. Increase Memory to 4GB+ (recommended: 6-8GB)
3. Apply & Restart

See `DOCKER_TROUBLESHOOTING.md` for more solutions.

