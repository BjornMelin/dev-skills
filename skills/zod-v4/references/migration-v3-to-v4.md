# Migration Checklist (Zod v3 -> v4)

## Contents

- [Imports](#imports)
- [String Format APIs](#string-format-apis)
- [Enums](#enums)
- [Objects](#objects)
- [Records](#records)
- [Errors](#errors)
- [Promises](#promises)
- [Package Surfaces](#package-surfaces)

This is a short, decision-oriented checklist. For exact fixes, jump to the linked rule files.

## Imports

- Prefer named root imports for app code: `import { z } from "zod"`.
- Default and namespace root imports are valid in Zod 4.4.3; treat them as
  local style/migration cleanup, not broken package behavior.
- Use `zod/mini` for bundle-sensitive app code only when the tradeoff is
  justified.
- Use `zod/v4/core` for schema tooling and libraries that accept or introspect
  user-provided schemas.
- Keep `zod/v3` imports only when a dependency is pinned to Zod 3.

## String Format APIs

- Replace `z.string().email()/url()/uuid()/...` with top-level formats: `rules/migrate-top-level-string-formats.md`

## Enums

- Replace `z.nativeEnum(...)` with `z.enum(...)`: `rules/migrate-nativeenum-to-enum.md`

## Objects

- Replace `.strict()/.passthrough()/.strip()` with `z.strictObject(...)` / `z.looseObject(...)` / default behavior:
- `rules/migrate-object-strict-passthrough-strip.md`
- Replace `.merge(...)` with `.extend(other.shape)` or object spread:
- `rules/migrate-object-merge-to-extend-shape.md`

## Records

- Prefer explicit key schemas:
- `rules/migrate-record-value-only-signature.md`

Zod 4.4.3 still supports one-arg `z.record(valueSchema)` as compatibility
behavior, but explicit keys make migrations and enum-key behavior clearer.

## Errors

- Replace `err.format()` / `err.flatten()` / `z.formatError(...)` with:
- `z.treeifyError(err)` / `z.flattenError(err)`:
- `rules/migrate-error-format-flatten.md`

## Promises

- Avoid `z.promise(...)` (deprecated); `await` then parse:
- `rules/migrate-z-promise-deprecated.md`

## Package Surfaces

- Root `zod` is Zod 4 in the 4.x line.
- `zod/mini` is the current Mini import.
- `zod/v4/core` is the right surface for library authors and schema tooling.
- See `references/package-surfaces-v4.md`.
