# Context7 Research Playbook

## Selection rubric (resolve results)

Prefer libraries in this order:

1. **Canonical docs site** for the library/framework (usually highest snippet coverage)
2. **Official repo** for the library (often has versions listed)
3. Other sources (only if the above are missing)

Tie-breakers (in order):

- Closest name match to the intended library
- Source reputation (High > Medium > Low)
- Higher snippet coverage
- Higher benchmark score
- Versions that match the requested version (if applicable)

## Spend the 3-call budget

Write the three queries before calling `query-docs`.

### Query 1: Overview + where-to-look

Goal: orient quickly and pick the right conceptual entry point.

Template:
- “Explain the concept, the recommended pattern, and where it is documented. Include the smallest working example.”

### Query 2: API reference for the user’s objective

Goal: exact signatures, required options, and usage examples.

Template:
- “Show exact signatures and usage examples for {task}. Include TypeScript examples if relevant.”

### Query 3: Edge cases + errors + version differences

Goal: avoid footguns and mismatches.

Template:
- “List common pitfalls, errors, and version differences for {task}. Include recommended best practices.”

## Query-writing rules

- Use the **user’s full objective**; avoid single-keyword prompts.
- Ask for **one** thing per query (concept OR API details OR pitfalls).
- Name **specific symbols** (functions/classes/hooks) when possible.
- If results are too broad, narrow with: module name, runtime (node/browser), framework layer (server/client), and version.

## Output checklist

Include:

- The chosen `libraryId` (and version suffix if used)
- The key doc-backed recommendation(s)
- At least one code snippet when the task is code-related
- “Assumptions / Gaps” and `UNVERIFIED` for anything not supported by docs

