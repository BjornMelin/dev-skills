import { afterEach, vi } from 'vitest'

// Node-only suites: keep global state clean by default.
afterEach(() => {
  vi.restoreAllMocks()
})
