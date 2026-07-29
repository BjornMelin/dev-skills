# Official GreenSock React skill source

Skill: gsap-react
Checked at: 2026-06-04

## When To Load

- Use this to verify upstream @gsap/react guidance.


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
