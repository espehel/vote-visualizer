# Vote Visualizer

Visualizes how representatives and political parties vote in the Norwegian Storting, using [Stortinget's open data API](https://data.stortinget.no/).

- **`services/rust`**: Rust service (Axum, SQLx) that imports votes from Stortinget, validates them, stores them in PostgreSQL, and serves them through a read API.
- **`apps/web`**: Next.js frontend that reads from that API.
- **PostgreSQL** holds the imported data. Imports are idempotent and each vote is saved in a single transaction, so a failed import never leaves partial data.

Currently one vote can be imported at a time through a local dev endpoint. Automated tests, request pacing, and the frontend integration are still to come. See [docs/architecture-design.md](docs/architecture-design.md) for the design and implementation status.

## Getting started

Prerequisites: [mise](https://mise.jdx.dev/) (installs Node, pnpm, and Just from `mise.toml`), [rustup](https://rustup.rs/) (picks up the toolchain from `rust-toolchain.toml`), and Docker.

```sh
mise install
just install    # JS and Cargo dependencies, plus the pinned sqlx-cli
just migrate    # start PostgreSQL and apply migrations
just dev        # run the Rust API and the web app
```

| What | Where |
| --- | --- |
| Rust API | http://127.0.0.1:8080 |
| Web app | http://localhost:3001 |
| PostgreSQL | `localhost:5433`, connection string in `.env` |

Import the example vote and read it back:

```sh
curl -H 'Content-Type: application/json' \
  -d '{"case_id":83657,"vote_id":16897}' \
  http://127.0.0.1:8080/dev/imports/vote

curl http://127.0.0.1:8080/votes/16897
```

## Developer commands

Run `just` to list all recipes.

| Command | What it does |
| --- | --- |
| `just install` | Install JS dependencies, fetch Cargo dependencies, install `sqlx-cli` |
| `just dev` | Start PostgreSQL, then run all apps and services in dev mode |
| `just build` | Build everything |
| `just check` | Biome, TypeScript type-check, `cargo check`, and Clippy |
| `just test` | Run all tests |
| `just db-up` | Start PostgreSQL only |
| `just migrate` | Apply pending migrations |
| `just migrate-add <name>` | Create a new migration in `migrations/` |
| `just migrate-info` | Show applied and pending migrations |
| `pnpm format` | Format and fix JS/TS with Biome |
| `cargo fmt` | Format Rust (run in `services/rust`) |

If `cargo` on your `PATH` isn't rustup's (for example a Homebrew Rust install), the pinned toolchain in `rust-toolchain.toml` is ignored.
