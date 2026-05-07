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
- library redirected;
- rate limit;
- retryable 5xx or timeout;
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
- hydrate issues through `github issue --comments`;
- hydrate PRs through `github pr --files --comments --reviews`;
- use `github compare`, `github tags`, or `github release` for version
  archaeology;
- switch to GitHub app/plugin for private/session data;
- clone or sparse checkout if code search is incomplete.

If the command reports GitHub rate-limit headers, wait for reset or switch to a
better-authenticated route. Do not retry in a tight loop.

## Provider Budget Exhausted

Symptom:

```text
budget exhausted for github; remaining=0 requested=1
```

Recovery:

- inspect `codex-research run status --run .codex/research/run.json`;
- narrow the query and reuse cached source IDs where possible;
- close the run and start a deeper profile only if the extra calls are
  justified;
- pass `--no-budget` only for a deliberate one-off exception.

Native Codex web calls are not visible to the CLI, so record them with:

```bash
codex-research run debit --provider codex-web --run .codex/research/run.json --count 1 --note "native web search"
```

## External Provider Privacy Refusal

Symptom:

```text
firecrawl refused private/authenticated input
```

Recovery:

- use GitHub, Context7, direct fetch, local files, or browser extraction first;
- pass `--privacy public` only when you have verified the URL is public;
- pass `--allow-private-external` only when the user explicitly allows sending
  that material to the external provider.

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
- research templates missing required evidence return headings.

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
