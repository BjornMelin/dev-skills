# Errors and Customization (Zod v4)

This reference captures the v4 error model. The atomic decisions live in `rules/`; this file keeps only the deeper patterns and gotchas.

## What You Get Back

- `safeParse(...)` returns `{ success: true, data } | { success: false, error }`.
- `error.issues` is an array of issue objects with `code`, `path`, and `message` (plus code-specific fields).

## Global Customization (`z.config`)

```ts
import { z } from "zod";

z.config({
  customError: (iss) => {
    if (iss.code === "invalid_type") return `Expected ${iss.expected}`;
    return undefined; // yield
  },
});
```

## Internationalization (Locales)

```ts
import { z } from "zod";
import { en } from "zod/locales";

z.config(en());
```

Lazy-load locales when needed:

```ts
import { z } from "zod";

async function loadLocale(locale: string) {
  const { default: localeFn } = await import(`zod/v4/locales/${locale}.js`);
  z.config(localeFn());
}
```

## Reporting Input in Issues (`reportInput`)

Zod does not include input values on issues by default to reduce accidental sensitive-data logging.
Opt in per-parse only when reviewed:

```ts
Schema.parse(value, { reportInput: true });
```

## Where To Go Next

- Formatting/printing errors: `references/error-formatting-v4.md`
- Rules:
- `rules/error-use-unified-error-param.md`
- `rules/migrate-error-customization-to-unified-error.md`
- `rules/error-return-undefined-to-yield.md`
- `rules/error-error-precedence.md`
- `rules/error-use-z-config-customerror.md`
- `rules/error-use-locales-i18n.md`
- `rules/error-do-not-enable-reportinput-by-default.md`
