# AGENTS.md — Backend

This file provides guidance to AI coding agents working on the backend of this repository.

## TDD (Test-Driven Development) — Mandatory

**All feature development MUST follow the TDD red-green-refactor cycle.** No exceptions.

### The TDD Cycle

1. **Red** — Write a failing test that defines the desired behavior.
2. **Green** — Write the minimum amount of code to make the test pass.
3. **Refactor** — Clean up the code while keeping tests green.

### Rules

- **Never write implementation code before its corresponding test.** The test must exist and fail first.
- **Every public function, handler, and module must have tests.** Test coverage is non-negotiable for new code.
- **Tests live in the same file as the code they test**, inside a `#[cfg(test)] mod tests { ... }` block.
- **Use descriptive test names** that document expected behavior (e.g., `test_create_user_with_valid_payload_returns_201`).
- **Tests must be independent and isolated.** No shared mutable state between tests. Use fresh test fixtures or setup/teardown per test.
- **Mock external dependencies** (database, HTTP clients) using trait abstractions. Never hit a real database in unit tests.

### TDD Checkpoint

Before considering a feature complete, the agent MUST verify:
- [ ] Tests were written first
- [ ] All unit tests pass: `cargo nextest run`
- [ ] All integration tests pass: `cargo nextest run` (includes `tests/`)
- [ ] No warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt --check`

## Integration Tests — Mandatory

**Every API endpoint and database operation MUST have at least one integration test.** Integration tests are the gate between "code compiles" and "feature actually works." Unit tests alone are insufficient.

### Integration Test vs Unit Test

|                  | Unit Test                          | Integration Test                       |
| ---------------- | ---------------------------------- | -------------------------------------- |
| **Scope**        | Single function or module          | Full request → response lifecycle      |
| **Database**     | Mocked (trait abstraction)         | Real MongoDB (test instance)           |
| **Server**       | No HTTP server                     | Real Axum server bound to a port       |
| **Location**     | `src/` (inline `#[cfg(test)]`)     | `tests/` directory                     |
| **Speed**        | Fast (milliseconds)                | Slower (real I/O, network)             |
| **Purpose**      | Verify logic correctness           | Verify system behavior                 |

### Integration Test Rules

- **Every route handler** must have integration tests covering:
  - Happy path (valid request → expected response)
  - Error path (invalid input → proper error response)
  - Edge cases (empty body, missing fields, boundary values)
- **Every database operation** must have integration tests verifying:
  - CRUD operations work end-to-end
  - Error handling (duplicate keys, not found, etc.)
  - Concurrency where relevant
- **Tests must be self-contained.** Each test seeds its own data and cleans up after itself.
- **Use a dedicated test database.** Never run integration tests against production or development databases.
- **Spin up a real Axum server** for route-level integration tests using `axum::serve` on a random port.

### Integration Test Structure

```
tests/
├── common/
│   ├── mod.rs              # Shared test utilities re-exported
│   ├── db.rs               # Test database setup/teardown helpers
│   └── server.rs            # Test server fixture (spawn, port, client)
├── health.rs                # Health endpoint integration tests
├── uploads.rs               # Upload endpoint integration tests
└── db/                      # Database-only integration tests
    └── source.rs            # ApiSource CRUD tests
```

### Test Fixture Patterns

#### Test Database Helper (`tests/common/db.rs`)

- Connect to a `MONGODB_TEST_URI` env var (default: `mongodb://localhost:27017`).
- Use a unique database name per test run (e.g., `mvp_test_{uuid}`) to avoid collisions.
- Provide `setup()` → returns `(Client, Database)` and `teardown(db)` → drops the database.

#### Test Server Fixture (`tests/common/server.rs`)

- Build the full Axum `Router` with all routes and state.
- Bind to `127.0.0.1:0` (random port), return the port number.
- Provide an async `spawn()` → returns `(TestServer, reqwest::Client)`.
- Implement `Drop` on `TestServer` to shut down gracefully.

#### Test Flow (each test)

```rust
#[tokio::test]
async fn test_create_upload_returns_201() {
    // 1. Setup
    let (db_client, db) = common::db::setup().await;
    let (server, client) = common::server::spawn(db_client).await;

    // 2. Execute
    let response = client
        .post(&format!("http://{}/api/uploads", server.addr))
        .json(&payload)
        .send()
        .await
        .unwrap();

    // 3. Assert
    assert_eq!(response.status(), 201);

    // 4. Teardown
    drop(server);
    common::db::teardown(db).await;
}
```

### Integration Test Commands

```sh
# Run integration tests only
cargo nextest run -E 'test(tests/)'

# Run unit tests only
cargo nextest run -E 'not test(tests/)'

# Run everything (unit + integration)
cargo nextest run
```

### Integration Test Checkpoint

Before marking a feature complete, the agent MUST verify:
- [ ] At least one integration test per route handler
- [ ] At least one integration test per database operation
- [ ] Happy path covered
- [ ] Error path covered
- [ ] Edge cases covered
- [ ] Tests use real HTTP requests (not just calling handlers directly)
- [ ] Database is cleaned up between tests (no test pollution)

## Testing with Nextest

This project uses [cargo-nextest](https://nexte.st/) exclusively for test execution.

```sh
# Primary TDD command — fast feedback loop
cargo nextest run --profile tdd

# Run all tests (default profile)
cargo nextest run

# Run tests for a specific module
cargo nextest run --test my_module

# CI-style run (collect all failures, no fail-fast)
cargo nextest run --profile ci

# List all tests
cargo nextest list
```

### Watch Mode (TDD)

For continuous TDD feedback, use `cargo-watch`:

```sh
# Install: cargo install cargo-watch
cargo watch -x "nextest run --profile tdd"
```

### Nextest Profiles

| Profile  | Behavior                          | Use Case              |
| -------- | --------------------------------- | --------------------- |
| `tdd`    | fail-fast, failures only          | Active development    |
| `default`| fail-fast, full output            | General testing       |
| `ci`     | no fail-fast, full output         | CI/CD pipelines       |

Config is in `.config/nextest.toml`.

## Project Structure

```
backend/
├── .config/
│   └── nextest.toml           # Nextest configuration
├── src/
│   ├── main.rs                # Server entry point
│   ├── routes/                # Axum route handlers
│   ├── models/                # Data models & DTOs
│   ├── db/                    # Database access layer
│   ├── error.rs               # Error types (thiserror)
│   └── ...                    # Additional modules as needed
├── tests/                     # Integration tests (MANDATORY)
│   ├── common/                # Shared fixtures & helpers
│   │   ├── mod.rs
│   │   ├── db.rs              # DB setup/teardown
│   │   └── server.rs          # Test server fixture
│   ├── health.rs              # Health endpoint tests
│   ├── uploads.rs             # Upload endpoint tests
│   └── db/                    # Database-only integration tests
│       └── source.rs
├── Cargo.toml
├── Cargo.lock
├── README.md
└── AGENTS.md
```

## Tech Stack Reference

| Crate               | Version | Purpose                       |
| ------------------- | ------- | ----------------------------- |
| `axum`              | 0.8     | Web framework                 |
| `tokio`             | 1       | Async runtime                 |
| `mongodb`           | 3       | MongoDB driver                |
| `bson`              | 3       | BSON serialization            |
| `serde` / `serde_json` | 1    | JSON/BSON serialization       |
| `utoipa`            | 5       | OpenAPI spec generation       |
| `openapiv3`         | 2       | OpenAPI v3 types              |
| `reqwest`           | 0.13    | HTTP client                   |
| `tracing`           | 0.1     | Structured logging            |
| `thiserror`         | 2       | Error type derivation         |

## Code Conventions

### Rust-Specific

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `Result<T, AppError>` for fallible operations. Never `unwrap()` or `expect()` in production code paths.
- Error types use `thiserror::Error` derive. Define a single `AppError` enum that all modules contribute to.
- Prefer `async` functions with `tokio`. Avoid blocking the async runtime.
- Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`) — not `println!` or `eprintln!`.
- Derive `Serialize`/`Deserialize` for all API models.
- Mark API types with `utoipa::ToSchema` for automatic OpenAPI documentation.

### Module Organization

- **`routes/`** — One file per resource (e.g., `users.rs`, `projects.rs`). Each exports a `Router` via `pub fn router() -> Router`.
- **`models/`** — Request/response DTOs and domain models. Keep separate from database models.
- **`db/`** — Database access functions. Return domain models, not raw BSON documents.
- **`error.rs`** — Centralized `AppError` enum. Implement `IntoResponse` so errors become proper HTTP responses.

### API Design

- RESTful endpoints. Use proper HTTP verbs (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`).
- Return appropriate status codes (201 for creation, 204 for no-content, 404 for not-found, 409 for conflict, etc.).
- All responses are JSON. Set `Content-Type: application/json`.
- Document every endpoint with `utoipa` attributes.
- Use `axum::Json<T>` extractors for request/response bodies.

## Before Committing

Run this checklist — the commit is blocked if any step fails:

```sh
# 1. All tests pass (unit + integration)
cargo nextest run

# 2. No clippy warnings
cargo clippy -- -D warnings

# 3. Code is formatted
cargo fmt --check
```

Before merging a feature, additionally verify:
- [ ] Every new route handler has integration tests (happy path + error path + edge cases)
- [ ] Every new database operation has integration tests (CRUD + error handling)
- [ ] No skipped or ignored tests (no `#[ignore]` without justification)
- [ ] Test database is not polluted (tests clean up after themselves)

## Common Pitfalls

- **MongoDB connection pooling**: Create one `mongodb::Client` at startup and share it via Axum state. Do not create a new client per request.
- **Blocking in async context**: Never call `std::thread::sleep` or synchronous I/O in an async handler. Use `tokio::time::sleep` and async APIs.
- **Missing OpenAPI docs**: Every public handler must have `utoipa` path attributes. The spec is useless if undocumented.
- **Leaking database internals**: Do not return raw `bson::Document` from route handlers. Map to domain models first.
