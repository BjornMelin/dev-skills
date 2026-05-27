# Advanced Modes

Use the narrowest mode that answers the current UI question.

## `advise`

Default mode for targeted implementation advice. Use after Codex has already
identified the relevant surface and wants Kimi to propose concrete code or
design-system changes.

## `audit`

Use before a redesign or polish pass. Ask for ranked issues, evidence, quick
wins, and verification checks. This mode is best when Codex needs a backlog of
UI problems before choosing what to patch.

## `redesign`

Use when the screen needs a cohesive professional direction, not isolated
tweaks. Provide product context, audience, density, brand constraints, and the
files that own the layout.

## `component`

Use for a single component or component family. Point Kimi at implementation,
style, test, and story files when available. Ask for states, variants,
composition, accessibility, and visual regression coverage.

## `screenshot-review`

Use after Codex captures screenshots with Playwright or browser tooling.
Pass each image with `--image` or `--screenshot`. Kimi should inspect images and
map visual findings back to code-level fixes.

## `compare`

Use after Codex applies changes. Pass before images with `--before-image` and
after images with `--after-image`. Ask Kimi to find regressions, residual polish
gaps, and pass/fail acceptance criteria.

## Design Briefs

For high-stakes UI work, pass `--design-brief-file` with the template in
`templates/design-brief.md`. Keep briefs short and concrete: audience, workflow,
quality bar, constraints, and what must not change.
