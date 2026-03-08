# Troubleshooting

## No libraries found

- Try a broader `libraryName` (product name instead of package name, or vice versa).
- Ensure the `query` includes the real task (not just a keyword).
- If still no results: state `UNVERIFIED` and ask for the official repo/docs URL or the exact package name.

## Too many matches / ambiguous results

Common ambiguity patterns:

- Framework docs vs SDK vs integration packages
- Multiple major versions (v1/v2) documented separately

Resolution:

- Prefer the official docs site entry with higher reputation/coverage.
- If the choice changes the answer materially, ask one clarifying question.

## Docs returned but irrelevant

- Narrow the query to specific symbols (function/class/method names).
- Add constraints: runtime (node/browser), framework layer (server/client), and version.
- If still irrelevant within budget: report `UNVERIFIED` and ask for a narrower target.

## Version mismatch

- If user provided a version, use `/org/project/version` IDs when available.
- If repo context implies a version, call it out explicitly and treat it as a constraint.

