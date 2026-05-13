# GitHub API Endpoints and Policy

## Endpoints Used

- Rate limit: `GET /rate_limit`
- Releases: `GET /repos/{owner}/{repo}/releases`
- Tags: `GET /repos/{owner}/{repo}/tags`
- Compare: `GET /repos/{owner}/{repo}/compare/{base}...{head}`
- Changelog file lookup: `GET /repos/{owner}/{repo}/contents/{path}`
- GraphQL fallback: `POST /graphql` via `gh api graphql` for release nodes when REST is empty/unavailable

## Headers

- `Accept: application/vnd.github+json`
- `X-GitHub-Api-Version: 2022-11-28`

## Pagination

Use `per_page=100&page=N` loops for list endpoints.

## Rate-Limit Handling

- Primary budget (PAT): typically 5,000 requests/hour.
- Secondary limits still apply regardless of PAT.
- Default behavior:
  - Serial queue in safe mode.
  - Retry with exponential backoff on 403/429/rate-limit signals.
- Fast mode:
  - Bounded concurrency.
  - Auto-fallback to safe mode for failed/rate-limited dependencies.
