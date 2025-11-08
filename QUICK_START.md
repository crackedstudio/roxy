# Quick Start Guide - Docker Setup

## Production Build (Linera Buildathon Template)

This setup follows the [Linera Buildathon Template](https://github.com/linera-io/buildathon-template) requirements.

### Build and Run

```bash
# Build the Docker image
docker compose build roxy

# Run the application (builds and starts Linera localnet)
docker compose up --force-recreate

# Or run in detached mode
docker compose up --force-recreate -d
```

### Access Points

Once running, the application is available at:
- **Frontend**: http://localhost:5173 (if applicable)
- **Linera Faucet**: http://localhost:8080
- **Validator Proxy**: http://localhost:9001
- **Validator**: http://localhost:13001

### Monitor Logs

```bash
# View logs
docker compose logs -f roxy

# View last 50 lines
docker compose logs --tail 50 roxy
```

### Check Status

```bash
# Check container status
docker compose ps

# Check if container is healthy
docker compose ps roxy
```

### Stop Containers

```bash
# Stop containers
docker compose down

# Stop and remove volumes
docker compose down -v
```

## Development Setup

### Using Docker Compose (Easiest)

```bash
# Start development container (uses volumes - no rebuild needed)
docker compose up -d roxy-dev

# Access container shell
docker compose exec roxy-dev bash

# Build inside container (uses cached volumes)
cargo build --release
```

### Access the container shell
```bash
docker compose exec roxy-dev bash
```

### Or run commands directly
```bash
docker compose exec roxy-dev cargo build
docker compose exec roxy-dev cargo test
docker compose exec roxy-dev cargo check
```

### Stop containers
```bash
docker compose down
```

### Start again
```bash
docker compose up -d roxy-dev
```

## Memory Issues? (Linker Killed)

If you see `ld terminated with signal 9 [Killed]`, it's an out-of-memory issue.

**Quick Fix:**
```bash
# Run tests with single job (uses less memory)
docker compose exec roxy-dev cargo test --jobs 1

# Or build in release mode (uses less memory)
docker compose exec roxy-dev cargo test --release --jobs 1
```

**Better Solution:** Increase Docker Desktop memory limit:
1. Docker Desktop → Settings → Resources → Advanced
2. Increase Memory to 4GB+ (recommended: 6-8GB)
3. Apply & Restart

See `DOCKER_TROUBLESHOOTING.md` for more solutions.

