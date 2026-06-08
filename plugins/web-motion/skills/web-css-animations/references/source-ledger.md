# web-css-animations Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, generated evals, examples, and audit rules. It
intentionally does not reference local scrape paths or machine cache locations.

## Primary Sources

- MDN CSS Animations, Transitions, and prefers-reduced-motion documentation.
- CSSWG drafts for new CSS motion features when MDN notes support limits.
- Agent Skills specification and best practices.

## Source-Backed Notes

- `references/docs-css-modern-motion-notes.md`
- `references/docs-browser-motion-performance.md`

## Tailored Reference Files

- `references/css-motion-field-guide.md` - CSS transition/keyframe field guide
- `references/browser-support-and-accessibility.md` - Support, @supports, and reduced-motion notes
- `references/property-performance-matrix.md` - CSS property performance matrix
- `references/scroll-driven-and-view-timelines.md` - Scroll-driven and view-timeline CSS motion
- `references/discrete-entry-exit-transitions.md` - Discrete entry and exit transitions
- `references/docs-mdn-css-animations.md` - MDN CSS animations routing notes
- `references/docs-mdn-css-transitions.md` - MDN CSS transitions routing notes
- `references/docs-mdn-prefers-reduced-motion.md` - MDN reduced-motion routing notes

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
