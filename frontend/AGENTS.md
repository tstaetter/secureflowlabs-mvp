# AGENTS.md — Frontend

This file provides guidance to AI coding agents working on the frontend of this repository.

## TDD (Test-Driven Development) — Mandatory

**All feature development MUST follow the TDD red-green-refactor cycle.** No exceptions.

### The TDD Cycle

1. **Red** — Write a failing test that defines the desired behavior.
2. **Green** — Write the minimum amount of code to make the test pass.
3. **Refactor** — Clean up the code while keeping tests green.

### Rules

- **Never write implementation code before its corresponding test.** The test must exist and fail first.
- **Every component, utility function, and page must have tests.** Test coverage is non-negotiable for new code.
- **Tests live alongside the code they test**, using the `.spec.ts` or `.test.ts` suffix convention.
- **Use descriptive, behavior-driven test names** (e.g., `it('returns 401 when the session is expired')`).
- **Tests must be independent and isolated.** No shared mutable state between tests. Each test sets up its own context.
- **Mock external dependencies** (API calls, browser APIs) rather than hitting real services.
- **Do not test implementation details.** Test behavior: given input X, expect output/rendering Y.

### TDD Checkpoint

Before considering a feature complete, the agent MUST verify:
- [ ] Tests were written first
- [ ] All unit tests pass: `deno task test`
- [ ] All integration tests pass: `deno task test`
- [ ] Type check passes: `deno task check`
- [ ] Lint passes: `deno task lint`
- [ ] Format check passes: `deno task lint`

## Testing with Vitest

This project uses [Vitest](https://vitest.dev/) for all testing, with two [project configurations](https://vitest.dev/guide/projects) defined in `vite.config.ts`.

### Project: `server` (Node.js)

- **Environment**: Node.js
- **Scope**: Pure logic — utility functions, stores, API client code, server-side SvelteKit code.
- **Glob**: `src/**/*.{test,spec}.{js,ts}` (excluding `.svelte` files)

```sh
# Run server-side tests only
deno run -A npm:vitest run --project server

# Run server-side tests in watch mode
deno run -A npm:vitest --project server
```

### Project: `client` (Browser)

- **Environment**: Chromium via Playwright
- **Scope**: Svelte components — render in a real browser, assert against DOM.
- **Glob**: `src/**/*.svelte.{test,spec}.{js,ts}`
- **Renderer**: `vitest-browser-svelte`

```sh
# Run browser tests only
deno run -A npm:vitest run --project client

# Run browser tests in watch mode
deno run -A npm:vitest --project client
```

### Example: Server-side test (utility)

```ts
import { describe, it, expect } from 'vitest';
import { formatCurrency } from './format';

describe('formatCurrency', () => {
    it('formats a number as USD', () => {
        expect(formatCurrency(42.5)).toBe('$42.50');
    });

    it('throws on negative values', () => {
        expect(() => formatCurrency(-1)).toThrow();
    });
});
```

### Example: Browser test (component)

```ts
import { page } from 'vitest/browser';
import { describe, it, expect } from 'vitest';
import { render } from 'vitest-browser-svelte';
import Button from './Button.svelte';

describe('Button', () => {
    it('emits a click event when pressed', async () => {
        const screen = render(Button, { label: 'Save' });

        await expect
            .element(screen.getByRole('button', { name: 'Save' }))
            .toBeInTheDocument();

        // The component test verifies DOM presence and interaction,
        // not internal state or implementation.
    });
});
```

### Commands

```sh
# Watch mode (primary TDD command)
deno task test:unit

# Run all tests once
deno task test

# Run specific test
deno run -A npm:vitest run src/lib/format.spec.ts
```

## Integration Tests — Mandatory

**Every page route and every API integration layer MUST have integration tests.** Integration tests verify that the frontend behaves correctly when composed with real backend interactions and user flows.

### Integration Test vs Unit Test

|               | Unit Test                           | Integration Test                        |
| ------------- | ----------------------------------- | --------------------------------------- |
| **Scope**     | Single function or component        | Full page load / user flow / API call   |
| **Backend**   | Mocked (never called)               | Mock service worker (MSW) or proxy      |
| **Browser**   | Component only (`vitest-browser`)   | Full page navigation (Playwright)       |
| **Location**  | `src/**/*.spec.ts` (alongside code)| `tests/` directory                      |
| **Speed**     | Fast (milliseconds)                 | Moderate (network simulation)           |
| **Purpose**   | Verify unit behavior                | Verify composed system behavior         |

### Integration Test Rules

- **Every SvelteKit page route** (`+page.svelte`) must have integration tests covering:
  - Page renders with expected content (happy path)
  - Loading states (skeleton, spinner)
  - Error states (API failure, not found, unauthorized)
  - Empty states (no data to display)
- **Every API client module** (e.g., `src/lib/api/users.ts`) must have integration tests verifying:
  - Successful responses are parsed correctly
  - Error responses are handled properly (timeout, 4xx, 5xx, network failure)
  - Request payloads are serialized correctly
- **Critical user flows** must have end-to-end integration tests:
  - Form submission → success/error feedback
  - Authentication flow (login → redirect → session)
  - Navigation between pages

### Integration Test Structure

```
tests/
├── fixtures/
│   ├── handlers.ts          # MSW request handlers (mock backend)
│   └── data.ts              # Shared test data factories
├── pages/
│   ├── home.test.ts         # Home page integration tests
│   ├── uploads.test.ts      # Uploads page integration tests
│   └── settings.test.ts     # Settings page integration tests
├── api/
│   ├── users.test.ts        # User API client tests
│   └── uploads.test.ts      # Upload API client tests
└── flows/
    ├── auth.test.ts         # Authentication flow tests
    └── upload.test.ts       # Upload creation flow tests
```

### Mocking the Backend

Use [MSW (Mock Service Worker)](https://mswjs.io/) to intercept network requests in integration tests. This gives full control over backend responses without needing a running server.

```ts
// tests/fixtures/handlers.ts
import { http, HttpResponse } from 'msw';

export const handlers = [
    http.get('/api/health', () => {
        return HttpResponse.json({ status: 'ok' });
    }),

    http.post('/api/uploads', async ({ request }) => {
        const body = await request.json();
        return HttpResponse.json({ id: 'abc123', ...body }, { status: 201 });
    }),
];
```

```ts
// tests/pages/home.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { setupServer } from 'msw/node';
import { handlers } from '../fixtures/handlers';

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('Home page', () => {
    it('shows a welcome message', async () => {
        // Navigate to page, assert content renders
    });
});
```

### Integration Test Checkpoint

Before marking a feature complete, the agent MUST verify:
- [ ] At least one integration test per new page route
- [ ] At least one integration test per API client module
- [ ] Happy path covered
- [ ] Loading state covered
- [ ] Error state covered
- [ ] Empty state covered
- [ ] Backend responses are mocked (no dependency on a running server)

## Project Structure

```
frontend/
├── src/
│   ├── app.html
│   ├── app.css              # Tailwind import
│   ├── app.d.ts
│   ├── lib/                 # Shared code
│   │   ├── api/             # API client modules
│   │   ├── components/      # Reusable Svelte components
│   │   ├── utils/           # Pure utility functions
│   │   ├── assets/          # Images, icons, fonts
│   │   └── server/          # Server-only code (excluded from browser)
│   └── routes/              # SvelteKit file-based routes
│       ├── +layout.svelte
│       ├── +page.svelte
│       ├── api/             # API route handlers (+server.ts)
│       └── (feature)/       # Feature-specific route groups
├── tests/                   # Integration tests (MANDATORY)
│   ├── fixtures/            # MSW handlers & test data
│   ├── pages/               # Page-level integration tests
│   ├── api/                 # API client integration tests
│   └── flows/               # End-to-end user flow tests
├── static/                  # Unprocessed static assets
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── eslint.config.js
├── .prettierrc
├── README.md
└── AGENTS.md
```

## Code Conventions

### Svelte 5 / SvelteKit

- **Use runes exclusively.** `$state`, `$derived`, `$effect`, `$props` — no `export let`, no `$:` reactive statements, no legacy stores.
- **Type all props** with an interface or inline type on `$props<T>()`.
- **One component per file.** Keep components focused and composable.
- **Use `.svelte.ts` files** for shared reactive state, `.ts` for pure logic.
- **Follow SvelteKit file conventions**: page data in `+page.ts`, server-only data in `+page.server.ts`, API endpoints in `+server.ts`, layout wrappers in `+layout.svelte`.

### TypeScript

- **Strict mode is enforced** (`tsconfig.json` has `"strict": true`).
- **Prefer `interface`** for object shapes, `type` for unions and utility types.
- **Never use `any`.** Use `unknown` if the type is genuinely indeterminate, and narrow it with type guards.
- **Export types alongside the functions/components that use them.**

### Styling (Tailwind CSS)

- **Use Tailwind utility classes** for all styling. Avoid inline `style=` attributes and raw CSS unless absolutely necessary.
- **Extract repeated utility patterns** into `@apply` directives in `app.css` if the same combination appears in 3+ places.
- **Responsive**: use `sm:`, `md:`, `lg:` breakpoints. Mobile-first.
- **Dark mode**: use `dark:` variants when dark mode is implemented.

### API Calls

- **Centralize API logic** in `src/lib/api/`. Never call `fetch` directly in components.
- **Use typed return values.** Every API function returns a typed `Promise<T>`.
- **Handle errors at the API layer.** Return discriminated union types (`{ success: true, data: T } | { success: false, error: string }`) rather than throwing.

### Files & Naming

- **Components**: `PascalCase.svelte` (e.g., `UploadForm.svelte`)
- **Utilities**: `kebab-case.ts` (e.g., `format-currency.ts`)
- **Tests**: `<same-name>.spec.ts` (e.g., `format-currency.spec.ts`)
- **Routes**: SvelteKit conventions (`+page.svelte`, `+layout.svelte`, `+server.ts`)

## Before Committing

Run this checklist — the commit is blocked if any step fails:

```sh
# 1. All tests pass (unit + integration)
deno task test

# 2. Type check passes
deno task check

# 3. Lint and format check passes
deno task lint
```

Before merging a feature, additionally verify:
- [ ] Every new route has integration tests (happy + loading + error + empty states)
- [ ] Every new API client has integration tests
- [ ] No skipped or flaky tests without documented reason
- [ ] Components use runes, not legacy patterns
- [ ] No `any` types introduced
- [ ] No inline `fetch` calls in components (use API layer)

## Common Pitfalls

- **Mixing legacy Svelte patterns with runes**: `$props()` replaces `export let`, `$state()` replaces `let`, `$derived()` replaces `$:`. Do not mix them.
- **Running browser tests without Playwright browsers installed**: run `deno run -A npm:playwright install chromium` if browser tests fail with "browser not found".
- **Testing implementation instead of behavior**: Avoid asserting on component internals (`component.$$.ctx`). Assert on the rendered DOM and emitted events.
- **Forgetting loading/empty/error states**: Every data-fetching page must handle all four states. Integration tests enforce this.
- **Tailwind class conflicts**: If a component accepts `class` as a prop, merge it with `clsx` or template literals — don't let the caller's classes silently overwrite the component's own styles.
