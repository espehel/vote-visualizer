set dotenv-load

# Keep in sync with the sqlx version in services/rust/Cargo.lock
sqlx_version := "0.9.0"

default:
    @just --list

install:
    pnpm install
    cargo fetch --manifest-path services/rust/Cargo.toml --locked
    cargo install sqlx-cli --version {{ sqlx_version }} --locked --no-default-features --features rustls,postgres

db-up:
    docker compose up -d --wait postgres

# Apply pending migrations from ./migrations
migrate: db-up
    sqlx migrate run

# Create a new migration: just migrate-add <name>
migrate-add name:
    sqlx migrate add {{ name }}

# Show applied and pending migrations
migrate-info: db-up
    sqlx migrate info

dev: db-up
    pnpm exec turbo run dev

build:
    pnpm exec turbo run build

check:
    pnpm exec turbo run check lint

test:
    pnpm exec turbo run test
