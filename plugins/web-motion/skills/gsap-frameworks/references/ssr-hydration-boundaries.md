# SSR and hydration boundaries for framework GSAP

Skill: gsap-frameworks
Checked at: 2026-06-04

## When To Load

- Read when Vue, Svelte, Astro, Nuxt, or islands code crosses server/client runtime boundaries.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.context()
- https://gsap.com/docs/v3/GSAP/gsap.to()

## Reference Notes

- Keep GSAP setup inside client-only lifecycle hooks or island hydration boundaries. Server-rendered code can prepare static state but cannot touch DOM, window, or measured layout.
- Use a component root ref for scoped selectors so framework reactivity and hydration do not animate matching nodes outside the component.
- Treat route transitions and island teardown as cleanup boundaries; revert contexts and kill timelines there.

## Focused Checks

- Confirm the code path cannot run during server render.
- Verify hydration does not flash from the animated start state.

## Failure Modes

- Top-level `window`, `document`, or DOM measurement in SSR-capable modules.
- Global selectors in reusable framework components.


## Operating Guidance

GSAP in Vue, Svelte, Nuxt, Astro, and vanilla component lifecycles with scoped selectors and cleanup.

### Decision Boundaries

- Use gsap-react for React and Next.js React code.
- Use vanilla GSAP core when no framework lifecycle is involved.
- Keep this skill explicit-only for non-React frameworks.

### Workflow Details

1. Identify framework lifecycle hooks and mount/unmount boundaries.
2. Scope selectors to the component root.
3. Register plugins once at module/app boundary.
4. Revert contexts, kill timelines, and clean listeners on unmount.

### Gotchas

- Do not let selector text escape a component root.
- Hydration/client-only boundaries matter in SSR frameworks.
- Framework transitions may already own enter/exit timing; avoid duplicate animation owners.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
