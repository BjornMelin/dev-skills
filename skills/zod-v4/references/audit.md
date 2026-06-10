# Audit Scripts

## Contents

- [Zod-Specific Audit](#zod-specific-audit)
- [AI Stack Scanner](#ai-stack-scanner)
- [Severity Model](#severity-model)

## Zod-Specific Audit

Use `zod-audit.ts` for rule-aware Zod v4 migration and footgun checks:

```bash
bun "$skill_dir/scripts/zod-audit.ts" --root . --format text
```

Useful flags:

- `--format text|json|md`
- `--fail-on error|warn|info`
- `--include-exts ts,tsx,js,jsx,mts,cts`
- `--exclude-dirs node_modules,.next,dist`
- `--list-rules` to list every documented rule file
- `--list-checks` to list rule IDs implemented by the scanner
- `--explain <ruleId>` to print the linked rule doc

The script is report-only and has no external dependencies.

## AI Stack Scanner

Use `ai_stack_scan.py` for the broader offline AI-stack scanner contract:

```bash
python3 "$skill_dir/scripts/ai_stack_scan.py" --root . --family zod-v4 --pretty
```

It emits `ai_stack_scan.v1`, performs no network calls, skips symlinks, and
flags broad Zod migration signals such as pre-v4 dependency specs, deprecated
string-format methods, legacy error parameters, `z.nativeEnum`, and
`error.errors`.

## Severity Model

- `error`: likely ignored, invalid, or unsafe behavior that should normally be
  fixed before shipping.
- `warn`: deprecated, migration-risk, or behavior-changing pattern that needs
  review.
- `info`: valid Zod 4.4.3 code that this skill discourages for consistency,
  explicitness, or maintainability.

Verify findings against current code before patching. Do not treat scanner
output as an automatic rewrite recipe.
