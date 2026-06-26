# Route Generation

Prefer file-based routing with the router plugin.

## Rules

- Keep route files under `src/routes` unless the repo config says otherwise.
- Treat `routeTree.gen.ts` as generated output; do not hand-edit it.
- Use `tanstackRouter({ target: 'react' })` from `@tanstack/router-plugin/vite`; the legacy PascalCase Vite helper is deprecated.
- Put the router plugin before the framework plugin in Vite config.
- Run the repo route-generation command after adding or renaming routes.
- Discover the repo route-generation command from existing scripts, Vite config, or local TanStack CLI availability before inventing a command.
