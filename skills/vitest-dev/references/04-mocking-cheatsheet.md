# Mocking cheat sheet (Vitest)

## Spies (preferred when possible)

```ts
import { vi } from 'vitest'

const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
// ...
spy.mockRestore()
```

## Globals

```ts
vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({ ok: true }))))
// ...
vi.unstubAllGlobals()
```

## Environment variables

```ts
vi.stubEnv('API_URL', 'https://example.test')
// ...
vi.unstubAllEnvs()
```

## Module mocking (use sparingly)

Key principles:
- mock at the boundary
- keep mocks small and focused
- prefer returning real implementations with only the seam stubbed

```ts
vi.mock('../db', () => ({
  getUser: vi.fn(),
}))
```

## Timers

Docs example: https://vitest.dev/guide/mocking/timers

```ts
beforeEach(() => {
  vi.useFakeTimers()
})
afterEach(() => {
  vi.restoreAllMocks()
})

it('runs after 2 hours', () => {
  schedule(fn)
  vi.runAllTimers()
  expect(fn).toHaveBeenCalled()
})
```

Advanced config: `fakeTimers.toFake` includes which globals to fake. If you include `nextTick`, it's not supported with `--pool=forks` (child_process can hang), but supported with `--pool=threads`.
See: https://vitest.dev/config/faketimers
