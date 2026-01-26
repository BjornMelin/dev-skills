import '@testing-library/jest-dom/vitest'
import { afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'

// Keep tests hermetic by default.
afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
})
