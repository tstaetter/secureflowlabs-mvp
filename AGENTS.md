# AGENTS.md

This file provides guidance to AI coding agents working on this repository.

## Project Overview

This is a full-stack web application organized as a monorepo:

- **`backend/`** — Rust web server using the [Axum](https://docs.rs/axum/latest/axum/) framework with OpenAPI v3 spec support.
- **`frontend/`** — Web client built with [SvelteKit](https://svelte.dev/docs/kit) (Svelte 5) and TypeScript.

## Tech Stack

| Layer    | Technology                              |
| -------- | --------------------------------------- |
| Backend  | Rust (edition 2024), Axum 0.8, openapiv3 2.2, serde_json |
| Frontend | Svelte 5, SvelteKit 2, TypeScript 6, Vite 8 |
| Testing  | Vitest (unit), Playwright (browser)     |
| Linting  | ESLint, Prettier                        |

## Project Structure

```
mvp/
├── backend/
│   ├── Cargo.toml          # Rust dependencies & manifest
│   └── src/
│       └── main.rs         # Entry point
├── frontend/
│   ├── src/                # SvelteKit app source
│   │   ├── app.html        # HTML shell
│   │   ├── app.css         # Global styles
│   │   ├── lib/            # Shared components & utilities
│   │   └── routes/         # File-based page routes
│   ├── static/             # Static assets
│   ├── package.json        # Node dependencies & scripts
│   ├── svelte.config.js    # SvelteKit configuration
│   ├── vite.config.ts      # Vite configuration
│   └── tsconfig.json       # TypeScript configuration
├── AGENTS.md               # This file
└── README.md               # Human-facing README
```

## Development Commands

### Backend

```sh
# Build
cd backend && cargo build

# Run (development)
cd backend && cargo run

# Check formatting & lint
cd backend && cargo fmt --check && cargo clippy

# Run tests
cd backend && cargo test
```

### Frontend

```sh
# Install dependencies
cd frontend && npm install

# Start dev server
cd frontend && npm run dev

# Build for production
cd frontend && npm run build

# Run type checking
cd frontend && npm run check

# Lint
cd frontend && npm run lint

# Format code
cd frontend && npm run format

# Run unit tests
cd frontend && npm run test:unit
```

### Running Both

```sh
# Terminal 1: backend
cd backend && cargo run

# Terminal 2: frontend
cd frontend && npm run dev
```

## Code Conventions

### General

- Use meaningful, self-documenting names for files, functions, and variables.
- Write comments sparingly — only when the "why" is not obvious from the code.
- Keep functions small and single-purpose.
- Handle errors explicitly; do not silently swallow them.

### Rust / Backend

- Follow standard Rust idioms and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `cargo fmt` and `cargo clippy` before committing.
- Prefer `Result` and `Option` over panicking.
- Structure handlers, models, and routes into separate modules as the project grows.

### Svelte / Frontend

- Use Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`) instead of legacy stores or `export let`.
- Prefer TypeScript and define interfaces/types for props and data.
- Keep components focused: one component per file, and each component should do one thing well.
- Use Prettier and ESLint before committing (`npm run format && npm run lint`).
- Follow SvelteKit conventions: page loads in `+page.ts`, server-side logic in `+page.server.ts`, API routes in `+server.ts`.

## API Design

- The backend exposes a REST API documented via OpenAPI v3 (`openapiv3` crate).
- The frontend communicates with the backend over HTTP.
- During development, configure the frontend to proxy API requests to the backend (via Vite's `server.proxy`).

## Testing Philosophy

- Write tests alongside new code — do not defer testing.
- Backend: unit tests in the same module (`#[cfg(test)] mod tests { ... }`).
- Frontend: unit tests with Vitest, component/e2e tests with Playwright.
- Aim for meaningful coverage: test behavior, not implementation details.

## Common Pitfalls

- **Cargo edition 2024**: The backend uses Rust edition 2024 — some older syntax (e.g., `unsafe` blocks, RPIT lifetime captures) may differ. Consult the edition guide if something looks off.
- **Svelte 5 Runes**: Svelte 5 uses runes (`$state`, `$derived`, `$effect`). Legacy patterns like `$:` reactive statements and `export let` should not be used.
- **Port conflicts**: Backend defaults to port `3000`. If the frontend dev server also defaults to `3000`, adjust one of them or use the proxy approach.
