import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'

/**
 * Multi-project example:
 * - Fast Node unit tests (threads, isolate off)
 * - UI tests in jsdom (isolate on)
 *
 * Reference:
 * - https://vitest.dev/guide/projects
 */
export default defineConfig({
  test: {
    projects: [
      {
        name: 'unit-node',
        test: {
          environment: 'node',
          pool: process.env.CI ? 'forks' : 'threads',
          isolate: false,
          include: ['src/**/*.test.ts'],
        },
      },
      {
        name: 'ui-jsdom',
        plugins: [tsconfigPaths(), react()],
        test: {
          environment: 'jsdom',
          isolate: true,
          include: ['src/**/*.test.tsx'],
          setupFiles: ['./test/setup-tests.ts'],
        },
      },
    ],
  },
})
