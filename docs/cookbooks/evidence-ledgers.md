# Cookbook: Evidence Ledgers and Reports

Use evidence ledgers when research needs to survive beyond the chat turn or when
claims need auditability.

## Create a Ledger

```bash
codex-research ledger init
```

Default path:

```text
.codex/research/ledger.jsonl
```

Use a custom path:

```bash
codex-research ledger init --path /tmp/research-ledger.jsonl
```

## Add Sources

```bash
codex-research ledger add-source \
  --provider context7 \
  --url https://context7.com/docs/api-guide \
  --title "Context7 API Guide" \
  --route context7
```

Providers can be any useful label:

- `codex-web`
- `context7`
- `github`
- `direct`
- `agent-browser`
- `firecrawl`
- `exa`
- `opensrc`

The command prints a source ID.

Provider commands such as `context7`, `github`, `fetch get --store`, and
`fetch firecrawl` also return cached source IDs. Add those directly:

```bash
codex-research ledger add-source --from-cache <source-id>
```

## Add Claims

```bash
codex-research ledger add-claim \
  --text "Direct Context7 API should be preferred over removed research-mode behavior." \
  --confidence 0.95 \
  --source <source-id> \
  --note "Supported by current API docs and local CLI changelog research."
```

Use confidence as a sober engineering score:

| Confidence | Meaning |
| --- | --- |
| `0.95-1.0` | directly supported by current primary source and no conflict |
| `0.80-0.94` | strong evidence with minor freshness or scope caveat |
| `0.60-0.79` | plausible but incomplete or partly secondary |
| `<0.60` | weak; likely `UNVERIFIED` in final prose |

## Inspect

```bash
codex-research ledger inspect
codex-research --json ledger inspect
codex-research cache sources --provider github --limit 20
codex-research cache source <source-id>
```

## Render a Report

```bash
codex-research report --out .codex/research/report.md
```

The report lists claims first, then sources. The JSONL ledger remains the
canonical audit artifact.

## Best Practices

- Add sources before claims.
- Use one claim per material assertion.
- Keep claim text precise and falsifiable.
- Do not mix evidence and recommendation in one claim.
- Use `note` for caveats, conflicts, or scope limitations.
- Prefer source IDs over pasted excerpts in final answers.
- Keep private source details out of ledgers that may be committed.

## Do Not Commit By Default

`.codex/research/` is a working artifact location. Do not commit run-specific
ledgers or reports unless the user explicitly wants tracked research artifacts.
