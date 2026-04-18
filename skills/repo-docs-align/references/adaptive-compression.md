# Adaptive Compression

Use this file to decide whether caveman-style compression improves a touched document.

## Default policy

Prefer selective compression, not blanket compression.

### Good compression targets

Compress when the document is mainly:
- internal operational guidance
- agent-facing instructions
- repo-maintenance notes
- execution checklists
- workflow manifests
- compact standards or runbooks where scanning matters more than prose polish

### Usually do not compress

Avoid caveman-style compression when the document is mainly:
- product strategy
- marketing or positioning
- public-facing docs
- polished teaching material
- reports where narrative and rhetorical quality matter
- nuanced architecture or design rationale that would lose important precision

### Mixed cases

For mixed docs:
- compress operational/checklist sections
- preserve richer prose in narrative, rationale, or externally shared sections

## Compression guardrails

When compressing, preserve exactly:
- commands
- code blocks
- file paths
- URLs and links
- headings
- tables
- version numbers and dates
- exact technical terms

Do not compress so far that:
- navigation gets harder
- intent becomes ambiguous
- rationale disappears where future readers need it
- the document stops matching surrounding repo style

## Interaction with `$caveman-compress`

If `$caveman-compress` is available and the target surface clearly fits this policy, route into it or apply the same rules directly.

If compression would materially harm readability or doc purpose, skip it and say why.
