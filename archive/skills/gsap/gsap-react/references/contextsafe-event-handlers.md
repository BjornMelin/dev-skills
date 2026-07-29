# contextSafe event handlers and callbacks

Skill: gsap-react
Checked at: 2026-06-04

## When To Load

- Read when GSAP code runs from React event handlers, timeouts, observers, or async callbacks after initial setup.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.context()
- https://gsap.com/docs/v3/GSAP/gsap.to()

## Reference Notes

- Use context-safe callback patterns so animations created after the initial hook setup are still associated with the component context and cleanup boundary.
- Keep event-driven tweens scoped to refs or a GSAP context. Avoid document-level selectors from React handlers.
- Store handles for animations that can outlive the synchronous event callback.

## Focused Checks

- Check event listeners, observers, timers, promises, and route callbacks for cleanup.
- Verify the component can unmount during an in-flight animation without leaked tweens.

## Failure Modes

- Creating tweens in click handlers with unscoped selectors.
- Assuming hook cleanup covers async-created animations automatically.


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
