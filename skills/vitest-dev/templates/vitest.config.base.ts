import { defineConfig } from 'vitest/config'

/**
 * Base config for TS/Node projects.
 *
 * Notes:
 * - Prefer `threads` pool for pure TS/JS logic for speed.
 * - Switch to `forks` if you depend on process APIs or native addons.
 * - Consider `isolate: false` ONLY for Node-only units with strict discipline.
 */
export default defineConfig({
  test: {
    environment: 'node',

    // Performance knobs
    pool: process.env.VITEST_POOL as any || (process.env.CI ? 'forks' : 'threads'),
    isolate: false,
    fileParallelism: true,

    // In CI, you might want to reduce workers to avoid OOM:
    // maxWorkers: process.env.CI ? '50%' : undefined,

    // Limits only tests marked with test.concurrent
    maxConcurrency: 5,

    // Cache (enabled by default). Pin the directory if you want stable CI caching.
    cache: {
      dir: 'node_modules/.vite/vitest',
    },

    // Keep setup files lightweight - they run before each test file.
    // setupFiles: ['./test/setup-node.ts'],

    // Optional: treat console noise as signal in CI
    // silent: process.env.CI ? true : false,
  },
})
