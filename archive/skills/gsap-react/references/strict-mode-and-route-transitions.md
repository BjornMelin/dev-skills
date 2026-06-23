# React Strict Mode, dependency, and route transition checks

Skill: gsap-react
Checked at: 2026-06-04

## When To Load

- Read when GSAP setup reruns, double-renders in development, or participates in Next.js route transitions.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.context()
- https://gsap.com/docs/v3/GSAP/gsap.matchMedia()

## Reference Notes

- React development Strict Mode can expose side effects that were hidden by single-mount assumptions. Treat duplicate setup as a signal to scope and cleanup correctly.
- Dependencies should rebuild only the animation affected by changed data. Recreating the whole scene on every render is usually a bug.
- Next.js App Router components that touch GSAP need client boundaries and route-level cleanup proof.

## Focused Checks

- Run mount/remount and route navigation checks.
- Inspect dependency arrays and ref identity for unnecessary animation reconstruction.

## Failure Modes

- Running GSAP in server components.
- Using React state updates inside high-frequency GSAP callbacks.


## Operating Guidance

React and Next.js GSAP integration using @gsap/react, useGSAP(), refs, scoped contexts, cleanup, SSR/client boundaries, and dependency-safe animation setup.

### Decision Boundaries

- Use gsap-core for framework-free tween semantics.
- Use gsap-scrolltrigger when scroll scene semantics dominate.
- Use web-motion-react if the implementation should use Motion instead of GSAP.

### Workflow Details

1. Confirm React/client boundary and installed @gsap/react.
2. Register useGSAP where the project pattern expects plugin registration.
3. Scope selectors to refs or context.
4. Revert contexts and kill manually owned animations on cleanup.

### Gotchas

- Do not run GSAP setup in a server component.
- Avoid unscoped string selectors in component code.
- Dependency changes should either rebuild inside useGSAP safely or use contextSafe callbacks.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
