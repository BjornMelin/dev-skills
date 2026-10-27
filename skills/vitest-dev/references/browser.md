# Browser testing

## Select the boundary

Use Node for runtime-independent logic, an existing DOM simulator for tests whose contract it represents, and Browser Mode for genuine browser component behavior. Keep full application navigation, authentication redirects, multiple pages, deployed routes, and cross-service journeys in the appropriate end-to-end runner. Vitest's Playwright provider uses Playwright automation; it is not a replacement for all Playwright tests. Do not migrate an effective component suite merely to standardize tooling.

Use `@vitest/browser-playwright` with compatible pinned Vitest versions and installed browser binaries. Prefer the framework's `vitest-browser-react`, `vitest-browser-vue`, or `vitest-browser-svelte` renderer when it fits the existing stack. Await `render`. Check each companion's peer ranges rather than assuming its version matches core.

A complete browser-only configuration, when creating that lane is necessary:

```ts
import { playwright } from '@vitest/browser-playwright'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['src/**/*.browser.test.{ts,tsx}'],
    browser: {
      enabled: true,
      headless: true,
      provider: playwright(),
      instances: [{ browser: 'chromium' }],
    },
  },
})
```

Reuse required framework plugins from the real application's config; this generic example does not supply React/Vue/Svelte transforms. In a mixed suite, use disjoint project includes so Node and browser workers do not both collect the same file. Browser Mode is not `environment: 'browser'` and does not need jsdom. Add engines only when the product's support matrix requires them.

## Interact as a user

Import `page` and `userEvent` from `vitest/browser`. Prefer role and label locators with meaningful accessible names. Use provider-backed actions and `await expect.element(locator)` for retryable assertions; avoid stale raw elements, CSS implementation selectors, manual event dispatch, and sleeps.

In v5, locator string matching is exact by default. `toHaveTextContent` means exact text; `toMatchTextContent` is the partial/regular-expression alternative. Do not globally restore loose matching to silence migration failures.

Test relevant keyboard navigation, focus transfer/restoration, disabled/loading/error states, and recovery. Assert the intended accessible structure with an ARIA snapshot when that is the contract, but do not call it a complete accessibility audit. Test reduced-motion behavior rather than simply disabling animation everywhere. Automated assertions supplement visual judgment; they do not establish design quality by themselves.

For visual regression, control viewport, fonts, locale/timezone, data, clock, and animation completion. Select representative states and meaningful layout breakpoints. Review changed baselines deliberately. Never fix unexplained diffs by increasing tolerance or snapshotting every variant. Reference images and failure screenshots have different paths and lifecycles.

## Debug without burdening every run

`browser.trace` produces Playwright traces. `browser.traceView` is a separate DOM-snapshot replay feature, still experimental in 5.0.5, usable through Vitest UI or HTML reports. Choose the needed diagnostic; recording both globally increases work. Traces can contain sensitive DOM/form data, so protect artifacts and retention. DOM replay is not full rendering/resource/video fidelity.

Use genuine browser boundaries for mocks. Native ESM namespace exports cannot simply be overwritten with `vi.spyOn`; a necessary external module can use Vitest's browser-supported factory or `{ spy: true }` strategy. Do not assume Node filesystem/process APIs exist in browser tests, or that fake clocks advance automation internals as in later prereleases.

## Respect framework boundaries

Expo native components belong in the supported native/Jest or device-testing lane, not a fabricated browser adapter. A web build does not validate Hermes or native modules. For Next.js server rendering, use the framework-supported integration/E2E path where component execution is unsupported. With Convex, preserve real schema/generated APIs and the compatible `convex-test` harness rather than mocking a fabricated database context. With Stripe, retain real raw-body signature verification and test duplicate/out-of-order handling; separate sandbox contracts from deterministic local tests. Verify integration compatibility before upgrading Vitest underneath a harness.

Sources: [Browser Mode](https://vitest.dev/guide/browser), [component testing](https://vitest.dev/guide/browser/component-testing), [assertions](https://vitest.dev/api/browser/assertions), [trace view](https://vitest.dev/guide/browser/trace-view), [visual regression](https://vitest.dev/guide/browser/visual-regression-testing).
