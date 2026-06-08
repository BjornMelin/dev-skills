# web-motion-react Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- Motion for React official documentation.
- React and framework SSR/client-boundary docs where applicable.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-motion-animate-presence.md`
- `references/docs-motion-layout-animations.md`
- `references/docs-motion-react.md`
- `references/docs-motion-use-reduced-motion.md`
- `references/docs-motion-use-scroll.md`
- `references/motion-react-field-guide.md`

## Tailored Reference Files

- `references/motion-react-presence-layout.md` - Presence and layout workflow
- `references/scroll-gestures-reduced-motion.md` - Scroll, gesture, and reduced-motion hooks
- `references/react-ssr-client-boundaries.md` - React and SSR/client boundaries
- `references/variants-and-motion-values.md` - Variants and MotionValue state ownership
- `references/next-router-presence-boundaries.md` - Next.js and router presence boundaries

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
