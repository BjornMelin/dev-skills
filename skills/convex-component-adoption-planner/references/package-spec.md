# Package specification

This reference defines the output package for Convex component adoption work in Signr.

## Output folder

Default:

```text
.agents/plans/YYYY-MM/MM-DD/convex-components/<component-slug>/
```

Required files:

- `PLAN.md`
- `CODEX_FULL_PROMPT.md`

## `PLAN.md` minimum contract

Include these sections:

1. `Objective`
   - component under review
   - target feature or workstream
   - explicit adoption or rejection decision
2. `Locked decisions`
   - final choice table
   - weighted scores
   - unresolved items, if any
3. `Current Signr ownership map`
   - current mounted components
   - app-owned feature boundary
   - duplicate-ownership risks
4. `Live source set`
   - Convex components index URL
   - component markdown URL
   - any official docs used
   - any `opensrc` source paths used
5. `Repo context to read in a fresh session`
   - exact backend files
   - exact specs
   - exact modernization-package prompt files, if the feature is already planned
6. `Context regathering commands`
   - `curl` commands
   - `opensrc` commands
   - `ctx7` commands if useful
   - `rg` commands for repo context
7. `Preferred architecture`
   - owner of raw state
   - owner of derived state
   - component mounting shape
   - auth, org scoping, and boundary rules
8. `Implementation sequence`
   - exact files expected to change
   - migration plan if replacing an existing owner
   - prompt-package updates required in the same workstream
9. `Validation`
   - repo validation commands to run later
   - focused smoke checks
10. `Acceptance criteria`
11. `Risks and rollback posture`

## `CODEX_FULL_PROMPT.md` minimum contract

Write this for a fresh Codex session with no assumed context beyond the repo.

Include:

1. Mission
2. Locked decisions that must not drift
3. Exact files to read first
4. Exact external URLs to fetch
5. Exact shell commands to regather context
6. Required implementation order
7. Prompt-package alignment instructions
8. Validation commands to run at the end
9. Final response contract

## Prompt-package alignment rule

If the feature is already represented in:

```text
.agents/signr-implementation-prompts-modernization-package/docs/implementation-prompts/
```

then the plan must explicitly decide whether those docs need to change and list the exact files.

At minimum, inspect:

- the feature plan markdown
- the matching codex trigger prompt
- the modernization-package `README.md` when discoverability changes
- `prompt-status-ledger.json` when prompt lifecycle or state changes

## Tooling posture

Use tools in this order:

1. `curl` for the live component index and markdown page
2. `opensrc` when package internals or source contracts matter
3. `ctx7` for extra docs lookup when helpful
4. `bunx convex` only when local Convex CLI context adds value
5. `gh` only when GitHub objects are directly relevant

Do not let tool usage replace ownership analysis.

## Question contract

When user clarification is needed:

- use `request_user_input`
- ask one question at a time
- provide 2-3 options
- include `0.0-10.0` weighted scores in each option description
- keep the recommended option first

## Rejection packages

If the answer is "do not adopt":

- still create `PLAN.md`
- explain the current durable owner
- explain why the component would create duplicate ownership or unacceptable maintenance load
- state what would need to change for a future re-evaluation
- make `CODEX_FULL_PROMPT.md` a future re-evaluation prompt, not an implementation prompt
