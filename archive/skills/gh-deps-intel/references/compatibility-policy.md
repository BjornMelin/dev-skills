# Compatibility Policy (Default: runtime-pinned)

## Objective

Upgrade each dependency to the latest release compatible with the repository runtime constraints.

## Rules

1. Use detected runtime hints from:
- Node: `engines.node`, `.nvmrc`, `.node-version`, `.tool-versions`, Volta fields.
- Python: `.python-version`, `project.requires-python`.

2. `@types/node` alignment:
- Keep major aligned with detected Node major.
- Example: Node 24 -> pick latest `@types/node` v24.x even if v25 exists.

3. Python packages:
- Prefer newest release satisfying known `requires_python` metadata when available.

4. If runtime constraints are absent or ambiguous:
- Use latest available and mark confidence/risk accordingly.
