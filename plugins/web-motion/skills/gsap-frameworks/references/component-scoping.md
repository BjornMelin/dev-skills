# Scoped selectors and cleanup contracts

Skill: gsap-frameworks
Checked at: 2026-06-04

## When To Load

- Use this when selectors, refs, or framework component boundaries are involved.


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
