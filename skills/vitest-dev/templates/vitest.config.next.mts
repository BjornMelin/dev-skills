import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'

/**
 * Next.js + Vitest baseline aligned with Next's documentation.
 *
 * Reference:
 * - https://nextjs.org/docs/app/guides/testing/vitest
 */
export default defineConfig({
  plugins: [tsconfigPaths(), react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./test/setup-tests.ts'],

    // Frontend suites benefit from isolation.
    isolate: true,

    // You can experiment with pool selection, but keep compatibility in mind.
    pool: process.env.CI ? 'forks' : 'threads',
  },
})
