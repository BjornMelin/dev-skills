# context7-research: Design Spec

## Goal

Provide a repeatable, high-signal workflow for deep library/framework/API documentation research using **only** Context7 MCP tools.

## Constraints

- Allowed tools: `mcp__context7__resolve-library-id`, `mcp__context7__query-docs`.
- Platform call budget: **≤3** `query-docs` calls per user request.
- Prefer primary documentation via Context7 sources; label uncertainty as `UNVERIFIED`.

## Architecture

### Workflow (canonical)

1. Extract `libraryName`, user objective, and version constraints (if any).
2. Resolve library ID (unless user provided `/org/project[/version]`).
3. Select best match (name match → reputation → coverage → benchmark → version fit).
4. Spend a 3-query budget (overview → API details → edge cases).
5. Synthesize answer + include `libraryId` used + assumptions/gaps.

### Version selection heuristic

- If user specifies a version, prefer a `libraryId` with that `/version` suffix (or pick a library result that lists that version).
- Otherwise default to “latest” documentation.
- If repo context implies a version (e.g., `package.json`), mention it explicitly and treat as a constraint.

### Failure modes to handle explicitly

- No libraries found → refine `libraryName` and objective; try a more general product name; otherwise return `UNVERIFIED`.
- Multiple similar libraries (framework vs client SDK vs docs site) → pick the most authoritative docs site; ask a minimal clarification only if selection materially changes the answer.
- Docs returned but irrelevant → tighten query to a specific function/class/module; if still irrelevant, state `UNVERIFIED`.

## Files

- `SKILL.md`: minimal workflow + hard rules.
- `references/playbook.md`: selection rubric + query templates + “how to spend the 3 calls”.
- `references/troubleshooting.md`: failure handling and recovery.
- `assets/report-template.md`: reusable research report skeleton.
- `scripts/new_report.py`: fills the report template placeholders into a new file.

## Decision framework (must be ≥9.0/10)

Weights: Solution leverage 35% · Application value 30% · Maintenance 25% · Adaptability 10%

### Decision: Context7 MCP-only (no Exa/web in skill)

- Leverage 9.8 · Value 9.4 · Maint 9.6 · Adapt 9.0 → **9.55**
- Rationale: maximizes authoritative docs while keeping the skill small and consistent; avoids multi-tool complexity.

### Decision: Fixed 3-query budget strategy

- Leverage 9.5 · Value 9.2 · Maint 9.4 · Adapt 9.0 → **9.33**
- Rationale: matches platform constraints; mitigated by query templates and “narrow scope” fallback.

### Decision: Include optional report template + generator script

- Leverage 9.2 · Value 9.1 · Maint 9.2 · Adapt 9.0 → **9.15**
- Rationale: improves repeatability and reduces cognitive load; script is local-only, tiny, and low maintenance.

