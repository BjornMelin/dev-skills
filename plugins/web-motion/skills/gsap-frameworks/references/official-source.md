# Official GreenSock framework skill source and license gate

Skill: gsap-frameworks
Checked at: 2026-06-04

## When To Load

- Use this when verifying copied upstream framework guidance.

## Source Verification Notes

- Treat upstream GreenSock framework guidance as the source for lifecycle examples.
- Confirm framework-specific mount and unmount hooks before adapting examples.
- Keep React and Next.js React work routed to `gsap-react`; this file owns non-React framework examples.

## Framework Lifecycle Guidance

- Vue/Nuxt: create animations after mount, scope selectors to a root element,
  and call `ctx?.revert()` in `onUnmounted`. For Nuxt, keep browser-only GSAP
  work behind client-side lifecycle boundaries.
- Svelte: return synchronous cleanup from `onMount`; if dynamic imports are
  needed, guard async work with a cancellation flag and catch import failures.
- Astro/islands: initialize only in client-hydrated code and clean up when the
  island is removed or re-rendered.
- Vanilla components: keep timelines and listeners owned by the component root;
  expose one teardown path that reverts contexts, kills timelines, and removes
  listeners.

## Command References

- Run the framework audit:
  `node scripts/audit.mjs scan --root <repo> --format markdown`
- Inspect setup:
  `node scripts/audit.mjs doctor --root <repo> --format json`

## Validation Notes

- Confirm selectors cannot escape the component root.
- Verify SSR/hydration boundaries before importing browser-only plugins.
- Check route transitions, hot reload, and component teardown for stale
  animations or duplicated plugin registration.
