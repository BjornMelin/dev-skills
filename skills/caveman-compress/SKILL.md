---
name: caveman-compress
description: >
  Caveman-compress repo docs + prose for fewer tokens. Keep substance, code, URLs, structure.
  Triggers: docs, memory file, repo note, dense markdown.
---

# Caveman Compress

## Purpose

Compress docs and natural language files such as `AGENTS.md`, todos, preferences, and repo notes into caveman-speak to reduce input tokens. Work directly in the active Codex session. Do not create backup files by default.

## Trigger

`/caveman:compress <filepath>` or user asks to compress memory file.

## Discovery

- No files named → inspect current repo first.
- `git status --porcelain`, `git diff --name-only` for changes.
- `rg`, `find` for nearby docs; bounded semantic match for prose neighbors.
- Prefer repo-owned docs + notes; outside-repo paths when user names them.
- Ambiguous / low-confidence → `request_user_input` + scored recs.

## Process

1. If the user names specific files, compress those directly.
2. If the target is a repo sweep, compress matching docs from the candidate set discovered above.
3. Map changed files to nearby docs with path, stem, README/AGENTS/docs conventions, and bounded semantic search.
4. Default to source docs and repo notes only. Skip generated artifacts and outputs unless the user explicitly names them.
5. Compress prose directly in the active agent session.
6. Preserve code blocks, inline code, URLs, links, file paths, commands, headings, tables, and exact technical terms.
7. Auto-apply only high-confidence matches.
8. If a file is not compressible, leave it unchanged and explain why.

## Compression Rules

### Remove

- Articles: a, an, the
- Filler: just, really, basically, actually, simply, essentially, generally
- Pleasantries: "sure", "certainly", "of course", "happy to", "I'd recommend"
- Hedging: "it might be worth", "you could consider", "it would be good to"
- Redundant phrasing: "in order to" → "to", "make sure to" → "ensure", "the reason is because" → "because"
- Connective fluff: "however", "furthermore", "additionally", "in addition"

### Preserve EXACTLY (never modify)

- Code blocks (fenced ``` and indented)
- Inline code (`backtick content`)
- URLs + links (full URLs, markdown links)
- File paths (`/src/components/...`, `./config.yaml`)
- Commands (`npm install`, `git commit`, `docker build`)
- Technical terms (library names, API names, protocols, algorithms)
- Proper nouns (project names, people, companies)
- Dates, version numbers, numeric values
- Environment variables (`$HOME`, `NODE_ENV`)

### Preserve Structure

- All markdown headings (keep exact heading text, compress body below)
- Bullet point hierarchy (keep nesting level)
- Numbered lists (keep numbering)
- Tables (compress cell text, keep structure)
- Frontmatter/YAML headers in markdown files

### Compress

- Use short synonyms: "big" not "extensive", "fix" not "implement a solution for", "use" not "utilize"
- Fragments OK: "Run tests before commit" not "You should always run tests before committing"
- Drop "you should", "make sure to", "remember to" - just state the action
- Merge redundant bullets that say the same thing differently
- Keep one example where multiple examples show the same pattern

CRITICAL RULE:
Anything inside ``` ... ``` must be copied EXACTLY.
Do not:

- remove comments
- remove spacing
- reorder lines
- shorten commands
- simplify anything

Inline code (`...`) must be preserved EXACTLY.
Do not modify anything inside backticks.

If file contains code blocks:

- Code blocks = read-only regions
- Only compress text outside them
- Do not merge sections around code

## Pattern

Original:

> You should always make sure to run the test suite before pushing any changes to the main branch. This is important because it helps catch bugs early and prevents broken builds from being deployed to production.

Compressed:

> Run tests before push to main. Catch bugs early, prevent broken prod deploys.

Original:

> The application uses a microservices architecture with the following components. The API gateway handles all incoming requests and routes them to the appropriate service. The authentication service is responsible for managing user sessions and JWT tokens.

Compressed:

> Microservices architecture. API gateway route all requests to services. Auth service manage user sessions + JWT tokens.

## Boundaries

- ONLY compress natural language files (.md, .txt, extensionless)
- Common doc variants in scope: `.md`, `.mdx`, `.markdown`, `.rst`, `.txt`, extensionless notes
- NEVER modify: .py, .js, .ts, .json, .yaml, .yml, .toml, .env, .lock, .css, .html, .xml, .sql, .sh
- If file has mixed content (prose + code), compress ONLY the prose sections
- If unsure whether something is code or prose, leave it unchanged
- Prefer current repo paths and explicit user targets over broad scans when they conflict
- Default confidence threshold for automatic edits: 0.8
- For lower-confidence matches, ask the user before editing
