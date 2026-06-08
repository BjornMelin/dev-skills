# web-tailwind-motion Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- Tailwind CSS v4 official documentation and npm package metadata.
- MDN prefers-reduced-motion documentation for motion media behavior.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-tailwind-animation.md`
- `references/docs-tailwind-theme.md`
- `references/docs-tailwind-transition-property.md`
- `references/docs-tailwind-v4-release.md`
- `references/tailwind-motion-field-guide.md`

## Tailored Reference Files

- `references/tailwind-v4-motion-utilities.md` - Tailwind transition and animation utilities
- `references/token-and-theme-motion.md` - Tailwind v4 @theme motion tokens
- `references/class-safety-audit.md` - Static class and runtime string safety
- `references/responsive-motion-variants.md` - Responsive and motion preference variants
- `references/semantic-token-naming.md` - Semantic motion token naming

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
