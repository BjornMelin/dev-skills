# Selective SSR

Start supports route-level SSR control. Use it intentionally, not as a blanket fix for hydration issues.

## Rules

- Default to SSR for content, auth shell, SEO, and first-load UX.
- Use `ssr: false` for browser-only experiences such as WebGL canvases when a static fallback is acceptable.
- Use data-only or selective SSR where server-rendered data is useful but component rendering should wait for the client.
- Document static fallback behavior for reduced-motion or no-WebGL routes.
- Keep server data authorization unchanged regardless of route SSR mode.
