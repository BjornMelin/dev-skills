# Fan-out reference: role stubs & Workflow skeleton

Load this only when you need a concrete custom role or a multi-stage Workflow. For ad-hoc
delegation, the inline spawn contract in `SKILL.md` is enough — no file needed.

## Custom `.claude/agents/*.md` role stubs

Authoring these is `claude-subagent-creator`'s job; these are illustrative shapes only.

```markdown
---
name: repo-explorer
description: Read-only fan-out search across a subsystem; returns located code, not reviews.
tools: Read, Grep, Glob, Bash
model: sonnet
---
You are a read-only explorer. Search the assigned scope, return file:line evidence and a short map.
Never edit files. Do not exceed the assigned scope.
```

```markdown
---
name: reviewer
description: Adversarial read-only reviewer; returns confirmed findings with severity + evidence.
tools: Read, Grep, Glob, Bash
model: opus
---
You are a skeptical reviewer. Default to refuting a finding unless the evidence is concrete.
Return: finding, severity, file:line, why-real, confidence. No edits.
```

```markdown
---
name: implementation-worker
description: Scoped edit-worker that owns a disjoint file set; fixes then verifies.
tools: Read, Edit, Write, Bash
model: opus
---
Edit ONLY the files named in your task. You are not alone in this repo — never touch or revert
files outside your set. After editing, run the named verification and report the result.
```

## Workflow skeleton (dependent / DAG batches)

When stages depend on each other, prefer the dynamic Workflow tool over hand-rolled batches:

```
phase('Find')
const found = await parallel(AREAS.map(a => () =>
  agent(`Search ${a} for X. Return file:line evidence.`, { schema: FINDINGS })))
phase('Verify')
const verified = await pipeline(found.flat().filter(Boolean),
  f => agent(`Adversarially verify: ${f.title}`, { schema: VERDICT }))
return verified.filter(v => v?.isReal)
```

Rules: `pipeline` by default (no barrier between stages); `parallel` only when a stage truly needs
all prior results at once; always `.filter(Boolean)` returns (a dropped agent resolves to `null`).
