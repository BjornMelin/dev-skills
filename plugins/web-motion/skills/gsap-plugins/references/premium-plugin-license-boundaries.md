# Plugin availability and license boundaries

Skill: gsap-plugins
Checked at: 2026-06-04

## When To Load

- Read before generating imports or examples for SplitText, MorphSVG, DrawSVG, Club plugins, or any plugin absent from local dependencies.

## Source Anchors

- https://gsap.com/docs/v3/Plugins/
- https://gsap.com/standard-license/

## Reference Notes

- Verify whether the plugin ships in the public `gsap` package, local project dependency, or a private/Club distribution before writing code.
- Keep the official GreenSock skill license separate from GSAP runtime/package licensing and plugin availability.
- If a premium plugin is unavailable, route to a public plugin or a different implementation only after explaining the tradeoff.

## Focused Checks

- Inspect `package.json`, lockfile, and existing import paths for plugin availability.
- Confirm registration happens once and is tree-shake/bundle safe for the project runtime.

## Failure Modes

- Invented import paths for plugins not present in the installed package.
- Copying premium-plugin examples into a repo without confirming license/access.


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
