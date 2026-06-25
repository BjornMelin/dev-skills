# Monitor

Use `firecrawl monitor` for recurring scrape/crawl checks and change tracking.
Prefer monitors when the user wants alerts, ongoing checks, or future diffs.

## Quick Start

```bash
firecrawl monitor create --name "Blog" --schedule "every 30 minutes" \
  --page https://example.com/blog \
  --goal "Alert only for new product or release announcements" \
  --email alerts@example.com

firecrawl monitor list --limit 20
firecrawl monitor get <monitorId>
firecrawl monitor run <monitorId>
firecrawl monitor checks <monitorId> --limit 10
firecrawl monitor check <monitorId> <checkId> --page-status changed
firecrawl monitor update <monitorId> --state paused
firecrawl monitor delete <monitorId>
```

Use JSON for advanced change tracking:

```bash
firecrawl monitor create monitor.json
cat monitor.json | firecrawl monitor create
firecrawl monitor update <monitorId> monitor.json
```

## Released 1.19.x Subcommands

`create`, `list`, `get`, `update`, `delete`, `run`, `checks`, `check`.

## Gotchas

- Use `--page` for one URL, `--scrape-urls` for multiple page URLs, and
  `--crawl-url` for a recurring crawl target.
- Always write a short `--goal` for AI change judging unless the user wants raw
  diffs only.
- Use `--state` for active/paused updates.
- Use `--page-status` for filtering check page results.
- Monitoring may be unavailable for zero-data-retention teams.
- Monitor-triggered scrapes can use `scrapeOptions.maxAge` in JSON payloads, but
  recurring monitors are for future change tracking, not local cache reuse.

## JSON Change Tracking

Use JSON payloads when the user cares about structured fields such as price,
headline, availability, or list items. Include a `changeTracking` format with
`modes: ["json"]` and a schema under the target scrape options. Use mixed mode
with `["json", "git-diff"]` when markdown side-by-side diffs are also useful.

## Output Shape

Monitor list/check commands return JSON. Save and inspect them before assuming
IDs or page result paths:

```bash
firecrawl monitor list --limit 20 --pretty -o .firecrawl/monitors.json
jq 'keys' .firecrawl/monitors.json
jq -r '.. | objects | .id? // empty' .firecrawl/monitors.json

firecrawl monitor checks <monitorId> --limit 10 --pretty -o .firecrawl/monitor-checks.json
jq -r '.. | objects | .id? // empty' .firecrawl/monitor-checks.json

firecrawl monitor check <monitorId> <checkId> --page-status changed --pretty -o .firecrawl/monitor-check.json
jq 'keys' .firecrawl/monitor-check.json
jq -r '.. | objects | .url? // .sourceURL? // empty' .firecrawl/monitor-check.json
```

For change-tracking monitors, prefer JSON fields for precise comparisons and
use git-diff or markdown only for human-readable review.
