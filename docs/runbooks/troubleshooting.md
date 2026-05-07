# Troubleshooting Runbook

## codex-research Is Missing

Symptom:

```text
zsh: command not found: codex-research
```

Fix:

```bash
cargo install --path crates/codex-research --force
```

Or run through the skill wrapper:

```bash
skills/deep-researcher/scripts/codex-research doctor
```

## Context7 Calls Fail

Check:

```bash
codex-research --json doctor
```

Common causes:

- `CONTEXT7_API_KEY` missing;
- library ID not found;
- library not finalized;
- rate limit;
- latest-critical docs still refreshing.

Recovery:

```bash
codex-research context7 search --library "<library>" --query "<question>"
codex-research context7 refresh --library-name "/org/project"
```

Then verify with official docs, GitHub source, or package source.

## GitHub Search Fails or Looks Incomplete

Check auth:

```bash
gh auth status
codex-research --json doctor
```

Recovery:

- narrow the query;
- reduce `--per-page`;
- hydrate files through `github file`;
- switch to GitHub app/plugin for private/session data;
- clone or sparse checkout if code search is incomplete.

## Firecrawl Rate Limited

Symptom:

```text
Firecrawl rate limited; Retry-After=<value>
```

Recovery:

- wait for `Retry-After`;
- reduce breadth;
- use direct fetch or `agent-browser` first;
- use `--fresh` only when latest-critical;
- avoid Firecrawl for GitHub and Context7-coverable docs.

## fetch probe Routes Poorly

If `fetch probe` says `direct` but extracted text is weak:

1. Increase `--max-bytes`.
2. Inspect the body with native Codex web or direct fetch.
3. Use `agent-browser` for app-shell pages.
4. Use Firecrawl for public pages if browser extraction is insufficient.

Record the route limitation in the ledger or final answer.

## Subagent Creator Validation Fails

Run:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate <path>
```

Common causes:

- filename does not match `name`;
- role name is not snake_case;
- role shadows `default`, `worker`, or `explorer`;
- invalid `sandbox_mode`;
- invalid `model_reasoning_effort`;
- missing redaction or no-nested-subagent instruction.

Fix templates before installing.

## Subspawn Wait Was Violated

Symptom: parent continued reading files, browsing, editing, or answering while
subagents were running.

Recovery:

1. Stop local work.
2. Wait for all spawned agents.
3. Account for every agent.
4. Synthesize results before proceeding.
5. In final answer, state any duplicated work or missed evidence risk.

## Skill Validation Fails

Run the exact failing validator:

```bash
python3 tools/skill/quick_validate.py skills/<skill-name>
```

Common causes:

- missing `SKILL.md`;
- frontmatter parse failure;
- unsupported frontmatter keys;
- skill directory name does not match `name`.

## Rust Build Fails After Dependency Change

Run:

```bash
cargo update -p <crate>
cargo check -p codex-research
```

If a feature is invalid, inspect the crate's current feature names and update
`crates/codex-research/Cargo.toml`.

Keep `Cargo.lock` committed for reproducible CLI builds.

