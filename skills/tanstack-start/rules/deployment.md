# Deployment

Use current Start hosting docs and installed package behavior instead of older adapter-centric snippets.

## Rules

- Use `@tanstack/react-start/plugin/vite`, not the legacy Start plugin package path.
- Put `tanstackStart()` before the React plugin in Vite config.
- Pin Start/Router versions for production work and plan upgrades deliberately.
- Treat Nitro/runtime output as deployment-specific; verify provider docs before adding provider flags.
- Keep env var examples generic unless a repo-specific deployment target is established.
