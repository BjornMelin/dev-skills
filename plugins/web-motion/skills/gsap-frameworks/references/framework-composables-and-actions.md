# Framework composables, actions, and directive wrappers

Skill: gsap-frameworks
Checked at: 2026-06-04

## When To Load

- Read before wrapping GSAP in a Vue composable, Svelte action, Astro component script, or Nuxt plugin.

## Source Anchors

- https://gsap.com/docs/v3/Plugins/
- https://agentskills.io/specification

## Reference Notes

- A wrapper should own one lifecycle boundary and return cleanup. Avoid helper APIs that hide cleanup or selector scope from the caller.
- Register plugins at an app/module boundary when possible, not inside every render or reactive effect.
- Reactive dependency changes should recreate only the animation that depends on that data, not the whole page motion system.

## Focused Checks

- Inspect the wrapper API for explicit target, scope, dependency, and cleanup inputs.
- Run a mount/unmount/remount check in the framework runtime when the wrapper is shared.

## Failure Modes

- Composable APIs that accept raw selector strings without a scope.
- Repeated plugin registration inside reactive render/update paths.


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
