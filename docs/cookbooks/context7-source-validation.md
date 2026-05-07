# Cookbook: Context7 and Source Validation

Use this when docs need to be checked against implementation or package source.

## Context7 Docs Lookup

Search for the library:

```bash
codex-research context7 search --library "Next.js" --query "middleware auth redirects" --version v16.0.0
```

Pick the best library ID by:

- exact name match;
- source reputation/trust;
- benchmark score;
- snippet coverage;
- version availability;
- relevance to the question.

Fetch context:

```bash
codex-research context7 context --library-id "/vercel/next.js" --query "middleware auth redirects"
```

When the repo pins a version, use a version-pinned ID when available:

```bash
codex-research context7 context --library-id "/vercel/next.js@v16.0.0" --query "cache components"
```

## Refresh Latest-Critical Docs

```bash
codex-research context7 refresh --library-name "/vercel/next.js"
```

After refresh, verify latest-critical claims through at least one additional
primary source, because docs may be returned while refresh work continues.

## Validate Against Source

Use `$opensrc` or GitHub hydration when docs are not enough.

Good source-validation questions:

- Is this option still accepted by the parser?
- Does the default documented value match implementation?
- Did the changelog claim land in the tagged source?
- Is a migration path backed by tests or examples?

Example flow:

```bash
codex-research github search-code 'repo:owner/repo configOption in:file' --per-page 5
codex-research github file owner/repo path/to/config.rs --ref v1.2.3
codex-research github compare owner/repo v1.2.2 v1.2.3 --per-page 100
```

Then inspect package source with `$opensrc` when the package version is the
source of truth.

## Conflict Resolution

If docs and source disagree:

1. Prefer current source for behavior.
2. Prefer official docs for intended public API when source is ambiguous.
3. Prefer release notes/changelog for migration timing.
4. Mark the claim `UNVERIFIED` if authority cannot be resolved.

Record conflicts in the final answer:

```text
Conflict: docs say X, source at ref Y does Z.
Resolution: use source behavior for runtime risk; treat docs as stale.
```

## Completion Criteria

- Context7 library ID and version are recorded.
- Source files or package versions are recorded when implementation matters.
- Stale or refreshed docs are marked.
- Final recommendation cites both docs and source when they support different
  parts of the conclusion.
