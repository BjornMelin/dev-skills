# Deep Researcher Runbook

## Fast Path

1. Run `codex-research doctor`.
2. Run `codex-research plan "<query>" --profile standard`.
3. Run `codex-research run init "<query>" --profile standard --topic general --out .codex/research/run.json`.
4. Use Codex-native web tools for official/current checks and debit them with
   `codex-research run debit --provider codex-web`.
5. Use provider commands only when the route plan calls for them, passing
   `--run .codex/research/run.json`.
6. Add source and claim records to `.codex/research/ledger.jsonl`, using
   `ledger add-source --from-cache <source-id>` when provider commands return
   source IDs.
7. Render `codex-research report`.

## Deep Path

Use when the query is ambiguous, high-stakes, recent, or source-heavy.

1. Create a deep profile plan.
2. Split independent lanes:
   - official docs/current web;
   - Context7 package docs;
   - GitHub repository/source/issues/releases;
   - package implementation source;
   - rendered/crawled web;
   - citation audit.
3. Spawn focused subagents with `$subspawn`, then immediately wait for all.
4. Hydrate all search hits before synthesis.
5. Ask one follow-up only if a policy/security/freshness decision remains
   genuinely blocked.

## GitHub Search Pattern

Use targeted shards, not broad searches:

```bash
codex-research github search-repos 'topic or package name pushed:>2025-01-01'
codex-research github search-code 'repo:owner/repo symbol in:file'
codex-research github search-issues 'repo:owner/repo error text is:issue'
codex-research github releases owner/repo --per-page 5
codex-research github release owner/repo --latest
codex-research github tags owner/repo
codex-research github compare owner/repo v1.2.2 v1.2.3 --per-page 100
codex-research github issue owner/repo 123 --comments
codex-research github pr owner/repo 456 --files --comments --reviews
codex-research github file owner/repo CHANGELOG.md --ref main
```

If code search is incomplete, hydrate likely files through contents API or clone
the repo and use local `rg`.

## Context7 Pattern

```bash
codex-research context7 search --library "React" --query "server actions form status"
codex-research context7 context --library-id "/vercel/next.js@v16.0.0" --query "cache components"
codex-research context7 refresh --library-name "/vercel/next.js"
```

Use version-pinned IDs when the target repo pins versions. If Context7 returns
existing docs while refreshing, mark latest-critical claims as pending a second
verification source.

## Rendered Page Pattern

```bash
codex-research fetch probe "https://example.com/docs/page"
```

- `direct`: fetch and cite text.
- `agent-browser`: use local browser extraction first.
- `firecrawl`: use Firecrawl when public and classified policy allows it.
- `github`: hydrate through GitHub APIs instead of scraping HTML.

Firecrawl refuses private or ambiguous external-provider input by default. Use
`--privacy public` only after verifying public status, and
`--allow-private-external` only with explicit user permission.

## Ledger Pattern

```bash
codex-research ledger init
codex-research ledger add-source --from-cache <source-id>
codex-research ledger add-source --provider github --url https://github.com/owner/repo/releases --title "Repo releases" --route github
codex-research ledger add-claim --text "Claim text" --confidence 0.87 --source abc123
codex-research report --ledger .codex/research/ledger.jsonl --out .codex/research/report.md
```

Keep Markdown reports concise. The ledger carries detail for replay and audit.
