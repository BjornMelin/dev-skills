# File Organization

Keep client, server, and shared contracts obvious.

## Rules

- Prefer route files for route contracts and UI entrypoints.
- Use `.functions.ts` or clearly named server modules for server function families.
- Use `.server.ts` for server-only helpers that should never enter client bundles.
- Keep shared validation schemas small and boundary-specific.
- Avoid Next.js/Remix terms such as server actions or loaders as if they were Start APIs.
