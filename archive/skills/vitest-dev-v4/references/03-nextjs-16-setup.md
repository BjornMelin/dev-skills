# Next.js 16 + Vitest setup (summary)

Canonical reference: https://nextjs.org/docs/app/guides/testing/vitest

## Recommended dev dependencies (TypeScript)

From the Next.js guide:

- `vitest`
- `@vitejs/plugin-react`
- `jsdom`
- `@testing-library/react`
- `@testing-library/dom`
- `vite-tsconfig-paths`

## Minimal `vitest.config.mts`

The Next.js guide shows:

```ts
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  plugins: [tsconfigPaths(), react()],
  test: {
    environment: 'jsdom',
  },
})
```

## Limitations

The Next.js guide notes that Vitest currently does not support **async Server Components**; it recommends E2E tests for async components.

## Practical additions (recommended)

1. `setupTests.ts` to install DOM matchers:

```ts
import '@testing-library/jest-dom/vitest'
```

2. Add `test.setupFiles` in your Vitest config so matchers are always available.

3. Mock Next-specific modules as needed:
- `next/navigation`
- `next/image`
- `next/router` (pages router apps)

A pattern for `next/navigation`:

```ts
vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), prefetch: vi.fn() }),
  usePathname: () => '/',
  useSearchParams: () => new URLSearchParams(),
}))
```

## Recommended test types in a Next app

- Pure unit tests: utilities, parsing, validation, domain rules (Node env).
- Component tests: client components (jsdom).
- E2E tests: async server components, routing, data fetching, authentication flows.
