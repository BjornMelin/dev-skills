# Lifecycle Protocol

Use this reference when starting, continuing, finalizing, or debugging runs.

## Commands

```bash
KIMI_UI_AGENT_CLI="skills/kimi-ui-agent/scripts/kimi-ui-agent.ts"
bun "$KIMI_UI_AGENT_CLI" --json start --task "Improve dashboard loading states" --dry-run
bun "$KIMI_UI_AGENT_CLI" --json start --task "Improve dashboard loading states" --apply
bun "$KIMI_UI_AGENT_CLI" --json status --run-id <run-id>
bun "$KIMI_UI_AGENT_CLI" --json reply --run-id <run-id> --message "Approved with notes" --apply
bun "$KIMI_UI_AGENT_CLI" --json continue --run-id <run-id> --apply
bun "$KIMI_UI_AGENT_CLI" --json finalize --run-id <run-id> --apply
bun "$KIMI_UI_AGENT_CLI" --json abort --run-id <run-id> --reason "scope changed" --apply
```

`start` is dry-run by default. Applied starts create an isolated git worktree
under XDG state, snapshot existing setup context from `.agents/kimi-ui-agent/`
into that worktree, and write human-readable artifacts under the worktree:

```text
.agents/kimi-ui-agent/runs/<run-id>/
  INPUT.md
  KIMI_PROMPT.md
  QUESTIONS.md
  PLAN.md
  APPROVAL.md
  RESULT.md
  CHANGED_FILES.txt
  ANSWERS.md
  ABORTED.md
  run.json
  events.jsonl
```

Controller state is mirrored under XDG state so the parent harness can locate
runs without scanning every worktree.

Setup context snapshots include `config.json`, profile Markdown files,
`protected-paths.md`, and `profile.lock.json` when they exist in the source
checkout. They do not copy prior run artifacts or adapter snippets.

## Review Rules

- Review `PLAN.md` before implementation proceeds.
- Use `QUESTIONS.md` for blocking decisions.
- Use `APPROVAL.md` and `ANSWERS.md` for parent-agent responses.
- Treat `RESULT.md` and `CHANGED_FILES.txt` as claims. Verify with `git diff`,
  repo-native checks, and visual/browser checks before finalizing.
