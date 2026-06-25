# Code Splitting

Prefer automatic code splitting when the bundler plugin supports it.

## Rules

- Enable `autoCodeSplitting: true` in `tanstackRouter` for supported bundlers.
- Keep critical route config in the main route file.
- Use `getRouteApi` inside extracted or lazy components.
- Keep `.lazy.tsx` as a manual fallback when auto splitting is unavailable.
- Do not split loaders away from code that needs server/client boundary clarity without checking generated output.
