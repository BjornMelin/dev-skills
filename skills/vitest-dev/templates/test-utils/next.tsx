import { vi } from 'vitest'

/**
 * Common Next.js module mocks.
 * Use these only when needed; prefer testing pure logic without Next globals.
 */

export function mockNextNavigation() {
  vi.mock('next/navigation', () => ({
    useRouter: () => ({
      push: vi.fn(),
      replace: vi.fn(),
      prefetch: vi.fn(),
      back: vi.fn(),
      forward: vi.fn(),
      refresh: vi.fn(),
    }),
    usePathname: () => '/',
    useSearchParams: () => new URLSearchParams(),
  }))
}

/**
 * Next/Image often requires a mock in unit tests because it uses optimizations
 * not available in jsdom.
 */
export function mockNextImage() {
  vi.mock('next/image', () => ({
    default: (props: any) => {
      // eslint-disable-next-line jsx-a11y/alt-text
      return <img {...props} />
    },
  }))
}
