# Qryvanta Justfile
# https://just.systems/

# Default recipe - show available commands
default:
    @just --list

# =============================================================================
# Development
# =============================================================================

# Run all dev servers (API, web, docs) in parallel
dev:
    pnpm dev

# Run API dev server only
dev-api:
    pnpm dev:api

# Run web dev server only
dev-web:
    pnpm dev:web

# Run docs dev server only
dev-docs:
    pnpm dev:docs

# =============================================================================
# Build
# =============================================================================

# Build all packages
build:
    pnpm build

# Build Rust workspace only
rust-build:
    cargo build --workspace

# Build API binary
api-build:
    cargo build -p qryvanta-api

# =============================================================================
# Check & Lint
# =============================================================================

# Run all checks (TypeScript + Rust)
check:
    pnpm check

# Run all linters
lint:
    pnpm lint

# TypeScript check only
ts-check:
    pnpm --filter @qryvanta/web check
    pnpm --filter @qryvanta/docs check

# Rust check only
rust-check:
    cargo xcheck

# Rust lint only
rust-lint:
    cargo xclippy

# =============================================================================
# Test
# =============================================================================

# Run all tests
test:
    pnpm test

# Rust tests only
rust-test:
    cargo xtest

# =============================================================================
# Format
# =============================================================================

# Format all code
fmt:
    pnpm exec prettier --write "**/*.{ts,tsx,js,jsx,json,md,yml,yaml}"
    cargo fmt --all

# Format TypeScript/JavaScript only
ts-fmt:
    pnpm exec prettier --write "**/*.{ts,tsx,js,jsx,json,md,yml,yaml}"

# Format Rust only
rust-fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    pnpm exec prettier --check "**/*.{ts,tsx,js,jsx,json,md,yml,yaml}"
    cargo fmt --all -- --check

# =============================================================================
# Infrastructure
# =============================================================================

# Start infrastructure services (postgres, kanidm)
infra-up:
    docker-compose up -d

# Stop infrastructure services
infra-down:
    docker-compose down

# Stop and remove infrastructure volumes (WARNING: deletes data)
infra-clean:
    docker-compose down -v

# View infrastructure logs
infra-logs:
    docker-compose logs -f

# Wait for postgres to be ready
infra-wait:
    @echo "Waiting for postgres..."
    @until docker-compose exec -T postgres pg_isready -U qryvanta -d qryvanta; do sleep 1; done
    @echo "Postgres is ready!"

# =============================================================================
# Database
# =============================================================================

# Run database migrations
db-migrate: infra-wait
    cargo run -p qryvanta-api -- migrate

# Reset database (drop and recreate)
db-reset: infra-wait
    docker-compose exec -T postgres psql -U qryvanta -d qryvanta -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
    @echo "Database reset. Run 'just db-migrate' to re-apply migrations."

# Open postgres shell
db-shell:
    docker-compose exec postgres psql -U qryvanta -d qryvanta

# =============================================================================
# Setup & Maintenance
# =============================================================================

# Initial project setup
setup:
    @echo "Installing pnpm dependencies..."
    pnpm install
    @echo ""
    @echo "Checking for .env file..."
    @if [ ! -f .env ]; then cp .env.example .env && echo "Created .env from .env.example"; else echo ".env already exists"; fi
    @echo ""
    @echo "Setup complete! Run 'just infra-up' to start services, then 'just dev' to start developing."

# Update dependencies
update:
    pnpm update
    cargo update

# Clean build artifacts
clean:
    rm -rf apps/web/.next
    rm -rf apps/docs/.next
    rm -rf node_modules
    cargo clean

# Deep clean (includes lockfiles - use with caution)
clean-all: clean
    rm -f pnpm-lock.yaml
    rm -f Cargo.lock

# =============================================================================
# Utility
# =============================================================================

# Generate SQLx offline query data (run this after schema changes)
sqlx-prepare:
    SQLX_OFFLINE=false cargo sqlx prepare --workspace

# Generate TypeScript API contracts from Rust DTOs
contracts-generate:
    pnpm contracts:generate

# Verify generated TypeScript API contracts are up to date
contracts-check:
    pnpm contracts:check

# Run security audit
audit:
    pnpm audit
    cargo audit

# Pre-commit checks (run before committing)
pre-commit: fmt-check lint check test
    @echo "All pre-commit checks passed!"
