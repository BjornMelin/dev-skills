# Setup and Project Profile

Use this reference when running or changing `setup`, `profile --refresh`, or
doctor freshness checks.

## Purpose

`setup` creates durable, reviewable project intelligence under
`.agents/kimi-ui-agent/`. It combines deterministic local scanning with optional
harness-assisted refinement by the active agent.

Committed files:

- `config.json`: machine-readable policy for protected paths, branch prefix,
  adapter defaults, artifact locations, and redaction extensions.
- `project-profile.md`: concise stack and product notes.
- `frontend-map.md`: route, app, and component ownership map.
- `design-system.md`: tokens, component conventions, density, accessibility,
  and visual QA expectations.
- `verification.md`: lint, typecheck, test, build, browser, screenshot, and
  Storybook checks.
- `protected-paths.md`: human-readable view of `config.json` protected paths.
- `profile.lock.json`: deterministic scan evidence for staleness checks.

Volatile files live under `.agents/kimi-ui-agent/runs/` and are ignored by that
directory's nested `.gitignore`.

## Harness Refinement

After `setup --apply`, the active harness may refine the Markdown context files
by reading current source. Keep the edits concise. Do not copy secrets, private
tokens, raw logs, or large code excerpts into profile files.

The CLI owns JSON schemas and safe writes. The harness owns interpretation.

## Refresh

Use:

```bash
KIMI_UI_AGENT_CLI="skills/kimi-ui-agent/scripts/kimi-ui-agent.ts"
bun "$KIMI_UI_AGENT_CLI" --json profile --refresh --dry-run
bun "$KIMI_UI_AGENT_CLI" --json profile --refresh --apply
```

Prefer refresh after frontend framework, routing, styling, component library,
or validation-script changes.

Refresh preserves existing editable Markdown context files. Use the dry-run
write summary to see which files would be created or skipped; update curated
profile facts by editing the Markdown files directly.

Directory scans skip local agent tooling roots such as `.agents/`,
`.kimi-code/`, `.claude/`, and `.codex/` so generated adapters and skills are
not recorded as product UI directories.
