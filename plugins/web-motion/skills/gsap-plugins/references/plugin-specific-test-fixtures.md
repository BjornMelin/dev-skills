# Plugin-specific fixtures and verification

Skill: gsap-plugins
Checked at: 2026-06-04

## When To Load

- Read when validating Flip, Draggable, Observer, MotionPath, ScrollTo, SVG, or text plugin behavior.

## Source Anchors

- https://gsap.com/docs/v3/Plugins/
- https://gsap.com/docs/v3/GSAP/gsap.to()

## Reference Notes

- Build tiny fixtures around the plugin contract: before/after DOM for Flip, pointer input for Draggable/Observer, path coordinates for MotionPath, and scroll container identity for ScrollTo.
- Plugin behavior often depends on measured layout or device input, so typecheck-only validation is not enough.
- Prefer stable selectors/refs and deterministic fixture sizes for repeatable audits.

## Focused Checks

- Verify resize, interruption, cleanup, and reduced-motion alternatives.
- Run pointer/keyboard accessibility checks for interactive plugins.

## Failure Modes

- Testing Flip without a real before/after state change.
- Shipping drag/observer interactions without keyboard or pointer fallback review.


## Operating Guidance

GSAP plugin registration, public package imports, Flip, Draggable, Observer, MotionPath, ScrollTo, SVG/text/ease plugins, and plugin boundary review.

### Decision Boundaries

- Use gsap-scrolltrigger for ScrollTrigger-specific scene control.
- Respect GSAP package license and plugin availability.
- Do not invent imports for plugins absent from the public package or local dependency set.

### Workflow Details

1. Check installed gsap version and plugin availability.
2. Register plugins exactly once in the runtime boundary.
3. Keep plugin setup out of hot render paths.
4. Verify lifecycle cleanup and license/package constraints.

### Gotchas

- Plugin imports differ by plugin and package availability; verify before generating code.
- SplitText-style text effects need accessibility and reduced-motion fallbacks.
- Flip needs measured before/after state ownership; do not mix with separate layout animators.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
