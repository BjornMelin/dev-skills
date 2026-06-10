# Troubleshooting

## `gh api` authentication failures

- Run `gh auth status`.
- Ensure token scopes cover repository metadata access.

## Secondary rate limit responses (403/429)

- Re-run with `--mode safe`.
- Keep fast mode concurrency low (`--max-concurrency 2` or `3`).
- Use `scripts/gh_rate_limit_diag.py` before large runs.

## Outdated command parse failures

- The skill falls back to registry metadata.
- Verify package manager command exists in PATH.
- Check command traces in JSON report (`command_traces`).

## Missing changelog/release details

- Some repos do not publish releases or changelogs.
- Skill falls back to registry/project URLs and compare summaries when possible.

## Monorepo misses packages

- Ensure workspace declarations are correct.
- Skill also performs recursive fallback scanning excluding ignored build/vendor directories.
