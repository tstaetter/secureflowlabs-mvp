# Frontend

Web client for the MVP application.

## Tech Stack

| Component    | Technology                                         |
| ------------ | -------------------------------------------------- |
| Framework    | [SvelteKit](https://svelte.dev/docs/kit) 2 (Svelte 5) |
| Language     | [TypeScript](https://www.typescriptlang.org/) 6    |
| Styling      | [Tailwind CSS](https://tailwindcss.com/) v4        |
| Build Tool   | [Vite](https://vite.dev/) 8                        |
| Runtime      | [Deno](https://deno.com/)                          |
| Unit Testing | [Vitest](https://vitest.dev/) 4                    |
| Browser Test | [Playwright](https://playwright.dev/) (Chromium)   |
| Linting      | [ESLint](https://eslint.org/) + [Prettier](https://prettier.io/) |

## Prerequisites

- [Deno](https://deno.com/) 2 or later

## Getting Started

```sh
cd frontend

# Install dependencies
deno install

# Start dev server
deno task dev
```

The dev server starts at `http://localhost:5173` and proxies API requests to the backend at `http://localhost:3000`.

## Project Structure

```
frontend/
├── src/
│   ├── app.html             # HTML shell
│   ├── app.css              # Global styles (Tailwind import)
│   ├── app.d.ts             # TypeScript declarations
│   ├── lib/                 # Shared components & utilities
│   │   ├── assets/          # Static assets (images, icons)
│   │   └── server/          # Server-only code
│   └── routes/              # File-based routes
│       ├── +layout.svelte   # Root layout
│       └── +page.svelte     # Home page
├── static/                  # Static files served as-is
├── tests/                   # Integration tests
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── README.md
└── AGENTS.md
```

## Testing

Tests are organized into two [Vitest projects](https://vitest.dev/guide/projects) configured in `vite.config.ts`:

| Project  | Environment | Scope                                              |
| -------- | ----------- | -------------------------------------------------- |
| `client` | Browser (Chromium via Playwright) | Svelte components: `*.svelte.{test,spec}.{js,ts}` |
| `server` | Node.js     | Logic, utilities: `*.{test,spec}.{js,ts}`          |

```sh
# Run all tests (both projects)
deno task test

# Run tests in watch mode (TDD)
deno task test:unit

# Run a specific test file
deno run -A npm:vitest run src/lib/greet.spec.ts
```

Component tests use [`vitest-browser-svelte`](https://www.npmjs.com/package/vitest-browser-svelte) to render Svelte components in the browser and assert against the rendered DOM.

## Useful Commands

| Command              | Description                     |
| -------------------- | ------------------------------- |
| `deno task dev`        | Start development server        |
| `deno task build`      | Build for production            |
| `deno task preview`    | Preview production build        |
| `deno task check`      | Type-check the project          |
| `deno task lint`       | Lint (Prettier + ESLint)        |
| `deno task format`     | Format code with Prettier       |
| `deno task test:unit`  | Run tests in watch mode         |
| `deno task test`       | Run all tests once              |

## API Proxy

During development, the Vite dev server proxies `/api` requests to the backend server. Configure the target in `vite.config.ts` under `server.proxy`.
