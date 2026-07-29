# Official GreenSock plugin skill source

Skill: gsap-plugins
Checked at: 2026-06-04

## When To Load

- Use this to verify upstream plugin coverage and license.

## Source Verification Notes

- Verify each plugin import against the installed `gsap` package before generating code.
- Keep ScrollTrigger scene semantics routed to `gsap-scrolltrigger`; this file covers general plugin availability and registration.
- Respect license/package boundaries for bonus plugins and avoid private registry guidance.

## Plugin Registration Guidance

- Register plugins once at the runtime boundary before first use:
  `gsap.registerPlugin(Draggable, Observer, Flip)`.
- Keep plugin registration out of hot render paths and repeated component
  renders. In React, route lifecycle-specific cleanup to `gsap-react`.
- Verify public package availability before writing imports. Do not invent
  imports for bonus plugins that are absent from the local dependency set.
- For text/SVG/layout plugins, include accessibility and reduced-motion
  fallbacks, especially when splitting text or animating layout state.

## Command References

- Run the plugin audit:
  `node scripts/audit.mjs scan --root <repo> --format markdown`
- Emit JSON findings for review:
  `node scripts/audit.mjs scan --root <repo> --format json --output gsap-plugins-audit.json`
- Inspect package context:
  `node scripts/audit.mjs doctor --root <repo> --format json`

## Validation Notes

- Confirm installed `gsap` version and plugin availability.
- Verify lifecycle cleanup for plugin-created instances.
- Keep private registry/token guidance out of generated answers.
