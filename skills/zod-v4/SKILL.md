---
name: zod-v4
description: Zod v4 TypeScript schema design, Zod 3 to 4 migration, validation boundaries, errors, codecs, JSON Schema/OpenAPI, metadata, Zod Mini/Core, RHF/tRPC/Hono/Next integrations, and rule-aware audits for Zod 4.4.3.
---

# Zod v4

Rule-first Zod v4.4.3 guidance for TypeScript. Start with the highest-risk
matching rule, then load the linked reference only when deeper API detail is
needed.

## Quick Start

```bash
bun add zod@^4.4.3
```

```ts
import { z } from "zod";

const User = z.object({
  name: z.string().min(1, { error: "Required" }),
  email: z.email({ error: "Invalid email" }),
});

type User = z.infer<typeof User>;
```

## Route By Task

| Task | Start Here |
| --- | --- |
| v3 migration or deprecated API cleanup | `rules/_index.md`, then `references/migration-v3-to-v4.md` |
| User input, env, HTTP, DB, queue, or JSON boundaries | `parse-*`, `schema-*`, and `references/codecs-v4.md` |
| Current schema APIs, string formats, records, template literals, XOR, defaults | `references/schema-surface-v4.md` |
| UI/API/CLI error messages | `error-*`, `migrate-error-*`, and `references/errors-v4.md` |
| Object composition, strictness, or refined schemas | `object-*` rules |
| JSON Schema/OpenAPI export | `jsonschema-*`, `meta-*`, and `references/json-schema-v4.md` |
| Bidirectional wire/internal transforms | `codec-*` and `references/codecs-v4.md` |
| React Hook Form, tRPC, Hono, or Next.js actions | `references/ecosystem-*.md` |
| Zod Mini, package exports, or library-author work | `references/package-surfaces-v4.md` |
| Repo audit before a migration | `references/audit.md` and `scripts/zod-audit.ts` |

## Priority Rules

1. **Validate at boundaries**: use `safeParse` for untrusted input, `parseAsync`
   for async refinements/transforms, and never trust `JSON.parse` output.
2. **Prefer current v4 APIs**: top-level string formats, `z.enum`, `z.object`
   plus `z.strictObject` or `z.looseObject`, unified `{ error }`, top-level
   error formatters, and `z.toJSONSchema`.
3. **Avoid false enforcement**: default imports, namespace root imports, and
   one-arg `z.record(valueSchema)` are valid in Zod 4.4.3; treat them as local
   style or migration-advisory findings, not package-invalid code.
4. **Use codecs when direction matters**: prefer `z.codec`, `z.decode`,
   `z.encode`, and `z.invertCodec` for reversible wire/internal mappings.
5. **Use `zod/v4/core` only for library tooling**: app code normally imports
   from `zod`; bundle-sensitive app code can use `zod/mini`; schema tooling and
   library adapters should read `references/package-surfaces-v4.md`.

## Rule Categories

| Priority | Category | Prefix |
| --- | --- | --- |
| 1 | Migration + deprecations | `migrate-` |
| 2 | Parsing + boundaries | `parse-` |
| 3 | Error handling | `error-` |
| 4 | Objects + composition | `object-` |
| 5 | Schema definitions | `schema-` |
| 6 | Metadata + registries | `meta-` |
| 7 | JSON Schema/OpenAPI | `jsonschema-` |
| 8 | Codecs | `codec-` |
| 9 | Package surfaces | `package-` |

Start with `rules/_index.md`. Each rule file has the failure mode, compact
bad/good examples, and linked references when nuance matters.

## Automation

Resolve `skill_dir` as the directory containing this skill before running
bundled scripts.

### Zod Audit

Run the Zod-specific report-only scanner:

```bash
bun "$skill_dir/scripts/zod-audit.ts" --root . --format text
```

Useful commands:

```bash
bun "$skill_dir/scripts/zod-audit.ts" --list-rules
bun "$skill_dir/scripts/zod-audit.ts" --list-checks
bun "$skill_dir/scripts/zod-audit.ts" --explain migrate-top-level-string-formats
```

Use `--fail-on warn|error|info` only after reviewing whether advisory rules are
appropriate for the target repo.

### AI Stack Scanner

Use the shared dependency-free scanner for broad AI-stack migration signals:

```bash
python3 "$skill_dir/scripts/ai_stack_scan.py" --root . --family zod-v4 --pretty
```

Treat scanner output as private local evidence. Verify each signal against the
current code and Zod docs/source before editing.

### Rule Maintenance

```bash
bun "$skill_dir/scripts/build-rules-index.ts"
bun "$skill_dir/scripts/check-skill-integrity.ts"
```

### Schema Runner

Load a schema from a local module and exercise parse, decode/encode, or JSON
Schema behavior:

```bash
bun "$skill_dir/scripts/zod-run.ts" --module src/schemas/user.ts --export User --mode safeParse --input '{"name":"A","email":"a@b.com"}'
bun "$skill_dir/scripts/zod-run.ts" --module src/schemas/date.ts --export IsoDate --mode encode --input '"2026-05-13T00:00:00.000Z"'
bun "$skill_dir/scripts/zod-run.ts" --module src/schemas/user.ts --export User --mode toJSONSchema --io input --target openapi-3.0
```

`zod-run.ts` imports the target module, so avoid modules with heavy side
effects.

## References

- Reference router: `references/index.md`
- Migration checklist: `references/migration-v3-to-v4.md`
- Schema surface highlights: `references/schema-surface-v4.md`
- Errors and formatting: `references/errors-v4.md`, `references/error-formatting-v4.md`
- Metadata and JSON Schema: `references/metadata-registries-v4.md`, `references/json-schema-v4.md`
- Codecs: `references/codecs-v4.md`
- Package surfaces, Mini, Core, and library authors: `references/package-surfaces-v4.md`
- Ecosystem: `references/ecosystem-react-hook-form.md`, `references/ecosystem-trpc.md`, `references/ecosystem-hono.md`, `references/ecosystem-nextjs-server-actions.md`
- Audit guide: `references/audit.md`
- Rule template: `assets/templates/rule-template.md`
