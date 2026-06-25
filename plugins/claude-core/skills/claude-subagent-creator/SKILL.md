---
name: claude-subagent-creator
description: >-
  Author, validate, and install custom Claude Code subagents as .claude/agents Markdown files. Use
  when the user asks to create a subagent, custom agent, or .claude/agents role; define a reusable
  reviewer, explorer, planner, or worker persona; set an agent's model or tool allowlist; or turn a
  recurring workflow into a delegatable role. Produces Markdown agent files with YAML frontmatter
  (name, description, model, tools) plus a focused system-prompt body, scoped to the smallest tool
  surface that does the job (read-only by default). Covers naming, description wording for autonomous
  delegation, model policy (Opus for reasoning/review, inherit for scans), tool scoping, project vs
  user scope, and a smoke test. Pairs with fan-out, which delegates to these roles. Not for one-off
  ad-hoc delegation (use fan-out) or authoring Agent Skills (SKILL.md files).
license: MIT
---

# Claude subagent creator

Create well-scoped custom subagents Claude Code can auto-delegate to. A subagent is a Markdown file
with YAML frontmatter plus a system-prompt body; it runs in its own context window with its own tool
surface.

## When to use

- The user wants a reusable persona (reviewer, explorer, planner, scoped worker).
- A workflow recurs often enough to deserve a named role `fan-out` can target.
- You need to pin a model or restrict tools for a class of delegated work.

For a one-off delegation, don't author a file — just use `fan-out`'s inline spawn contract.

## Agent file anatomy

`./.claude/agents/<name>.md` (project) or `~/.claude/agents/<name>.md` (user):

```markdown
---
name: <hyphen-case, matches filename>
description: <what it does + WHEN to delegate to it — explicit triggers; this drives auto-selection>
model: opus            # or sonnet / haiku / omit to inherit the session model
tools: Read, Grep, Glob, Bash   # smallest surface that works; omit = inherit all
---
<system prompt: role, exact scope, output contract, hard boundaries>
```

## Authoring workflow

1. **Name + intent** — hyphen-case name; one clear job. Avoid catch-alls.
2. **Description for delegation** — lead with the action, then `Use when ...` with concrete triggers
   ("Use when reviewing a diff for security issues"). The main agent matches this to decide delegation;
   vague descriptions don't get auto-selected.
3. **Model policy** — `opus` for reasoning/review/debugging/planning; `sonnet`/`haiku` or inherit for
   read-heavy scans and inventories. Omit `model:` to inherit the session model.
4. **Tool scoping** — grant the minimum. Default **read-only** (`Read, Grep, Glob, Bash`); add
   `Edit, Write` only for workers that own a disjoint write surface.
5. **System prompt** — state the role, the exact scope, the return contract (status, evidence with
   file:line, commands run, findings + confidence, risks), and hard boundaries ("never edit outside
   your assigned files"; "you inherit no parent transcript — work only from this prompt").
6. **Install + smoke test** — write the file, then confirm via `/agents` (or the Task tool with
   `subagent_type: <name>`) that it loads and a trivial scoped task returns the contracted shape.

## Description-for-delegation rules

- Action verb first; then explicit "Use when" triggers and the keywords a request would contain.
- Name the boundary ("read-only", "does not commit") so the parent scopes it correctly.
- One responsibility per agent — overlapping agents compete and dilute auto-selection.

## Scope

- **Project** (`./.claude/agents/`) — committed, shared with the repo, available in that project.
- **User** (`~/.claude/agents/`) — personal, available across all your projects.
- Project agents shadow user agents of the same name.

## Failure modes

| Symptom | Fix |
|---|---|
| Agent never auto-delegated | weak description — add explicit "Use when" + trigger keywords |
| Agent over-reaches / edits too much | tighten `tools:` and the scope line in the body |
| Inconsistent results | pin `model: opus`; specify an exact return contract |
| Two agents fight over the same work | merge or sharpen responsibilities; one job each |
| Worker reverted another's edits | enforce disjoint write surfaces; read-only by default |

Use `fan-out` to delegate to the roles you create here.
