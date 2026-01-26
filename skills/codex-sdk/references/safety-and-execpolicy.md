# Safety, sandboxing, approvals, and execpolicy

## Threat model (practical)

Agentic coding systems are vulnerable to:

- prompt injection (from issues, diffs, docs, screenshots)
- secret exfiltration (logs, environment, files, network)
- destructive commands (rm, curl|bash, privilege escalation)

## Default stance

- Run analysis-only by default: `read-only` sandbox, `approval-policy: never`.
- Grant write access only when needed; prefer `workspace-write` over `danger-full-access`.
- Avoid “YOLO” flags unless externally sandboxed.

## Execpolicy rules (command allow/block)

Use execpolicy to prevent risky commands even when the model requests them.

Typical policies:

- forbid `sudo`, `curl | sh`, package installs in prod, credential tools
- prompt for `git push`, `rm -rf`, changing auth/config
- allow safe read-only commands (`git status`, `rg`, `ls`)

Template starter file:

- `assets/templates/execpolicy/default.rules`

## Guardrail patterns that work

- require structured outputs for decisions (schemas + validation)
- gate handoffs on artifacts and tests passing
- keep prompts explicit about stop conditions and out-of-scope actions
