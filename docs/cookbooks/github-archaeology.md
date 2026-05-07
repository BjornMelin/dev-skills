# Cookbook: GitHub Archaeology

Use this to research behavior through repositories, source code, issues, PRs,
releases, tags, and changelogs.

## Principle

GitHub search results are leads. Hydrated GitHub API responses or local source
files are evidence.

## Start With the Best Available Surface

In a Codex session:

- prefer GitHub app/plugin for private repos, PRs, reviews, workflow logs, and
  authenticated metadata;
- use `codex-research github` for replayable public or token-backed REST calls;
- use `gh` directly only when the CLI command is clearer or the endpoint is not
  wrapped yet.

## Repository Discovery

```bash
codex-research github search-repos "<project or package> in:name" --per-page 5
```

Use repository metadata only to pick candidates. Do not cite it as behavioral
proof unless the claim is about repository metadata.

## Code Search

Use narrow query shards:

```bash
codex-research github search-code 'repo:owner/repo function_name in:file' --per-page 5
codex-research github search-code 'repo:owner/repo path:src "error text"' --per-page 5
```

Watch for:

- `incomplete_results`;
- default-branch-only coverage;
- indexed-file size limits;
- rate limits.

Hydrate the file:

```bash
codex-research github file owner/repo path/to/file.rs --ref main
```

## Issues and PRs

```bash
codex-research github search-issues 'repo:owner/repo "panic message" is:issue' --per-page 5
codex-research github search-issues 'repo:owner/repo "breaking change" is:pr' --per-page 5
```

Use issue/PR search for:

- bug reports;
- maintainer decisions;
- migration notes;
- release regressions;
- deprecation context.

Do not over-weight stale closed issues when release notes or current source
contradict them.

## Releases and Changelogs

```bash
codex-research github releases owner/repo --per-page 10
codex-research github file owner/repo CHANGELOG.md --ref main
```

If tags or compare ranges matter, use the GitHub app or `gh api` until the CLI
adds first-class compare/tag subcommands.

## When to Clone

Clone or sparse checkout when:

- code search is incomplete;
- you need cross-file control/data flow;
- generated source is not exposed through search;
- exact version refs matter and API hydration is cumbersome.

Then use local tools:

```bash
rg "symbol_or_error" path/
git log -- path/to/file
git show <tag>:path/to/file
```

## Evidence Standard

For each GitHub-derived claim, record:

- repository;
- ref/tag/branch;
- file path or issue/PR/release URL;
- fetched time;
- whether search was incomplete;
- confidence.

