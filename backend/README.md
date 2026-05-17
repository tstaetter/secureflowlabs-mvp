# Backend

REST API server for the MVP application.

## Tech Stack

| Component   | Technology                                      |
| ----------- | ----------------------------------------------- |
| Runtime     | [Tokio](https://tokio.rs/) (async)              |
| Web Server  | [Axum](https://docs.rs/axum/latest/axum/) 0.8   |
| Database    | [MongoDB](https://www.mongodb.com/) via `mongodb` 3 |
| API Docs    | [utoipa](https://docs.rs/utoipa/) 5 + OpenAPI v3 |
| HTTP Client | [reqwest](https://docs.rs/reqwest/) 0.13        |
| Observability | [tracing](https://docs.rs/tracing/)          |
| Testing     | [nextest](https://nexte.st/)                    |

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable, edition 2024)
- [MongoDB](https://www.mongodb.com/docs/manual/installation/) 7+

## Getting Started

```sh
cd backend

# Build
cargo build

# Run (expects MongoDB at mongodb://localhost:27017)
cargo run

# Run tests with nextest
cargo nextest run

# Watch mode for TDD
cargo watch -x nextest
```

The server starts at `http://localhost:3000`.

## Testing with Nextest

This project uses [cargo-nextest](https://nexte.st/) as its test runner — it's faster than `cargo test` and has first-class support for CI and TDD workflows.

```sh
# Run all tests
cargo nextest run

# Run a specific test
cargo nextest run --test my_test

# Run with TDD profile (fail-fast, minimal output)
cargo nextest run --profile tdd

# Run with CI profile (collect all failures)
cargo nextest run --profile ci

# List all tests without running
cargo nextest list
```

Three nextest profiles are configured in `.config/nextest.toml`:
- **default** — fail-fast, full status output
- **tdd** — fail-fast, only show failures (ideal for red-green-refactor)
- **ci** — no fail-fast, full output (collect all results)

## Project Structure

```
backend/
├── .config/
│   └── nextest.toml       # Nextest configuration
├── src/
│   └── main.rs            # Entry point
├── Cargo.toml             # Dependencies & manifest
├── AGENTS.md              # AI agent guidance
└── README.md
```

## API Documentation

API docs are auto-generated via `utoipa` and exposed at `/api-docs/openapi.json` when the server is running.

## Environment Variables

| Variable        | Default                      | Description        |
| --------------- | ---------------------------- | ------------------ |
| `MONGODB_URI`   | `mongodb://localhost:27017`  | MongoDB connection |
| `SERVER_PORT`   | `3000`                      | HTTP listen port   |
| `RUST_LOG`      | `info`                      | Tracing log level  |
