# Security and Privacy

Public `kimi-ui-agent` must not inherit private project assumptions.

## Sanitization

- No private repo names, product prompts, absolute workstation paths, internal
  workflow names, private command names, or confidential policy text in public
  assets.
- Generated profile files are project-local and reviewable; they are not copied
  from any private source.
- `.skill` bundles must be inspected before publishing.

## Path Safety

- Run IDs and adapter IDs must be safe single path segments.
- Resolve every path before write/read/remove and prove it stays inside the
  expected root.
- Reject symlinked roots and parent traversal.
- Use `.agents/kimi-ui-agent/runs/.gitignore` for volatile artifacts instead of
  asking users to edit the root `.gitignore`.

## Environment Safety

Launches and MCP calls should pass only allowlisted environment variables.
Never echo or store full tokens. `doctor` reports token availability by source
category only when token checks are added.

## Hooks

Kimi and Codex hooks can block obvious mistakes, but they are not a sandbox.
They may fail open or be disabled. Keep plan-first worktrees and harness
permissions as the main safety controls.
