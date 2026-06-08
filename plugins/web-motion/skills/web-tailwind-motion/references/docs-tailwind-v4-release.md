# Tailwind v4 Motion-Relevant Notes

Source: https://tailwindcss.com/blog/tailwindcss-v4

Use this for v4-specific migration and implementation context.

## Package Pin Context

- Current official docs show the v4.3 docs track.
- `tailwindcss@4.3.0` is the current stable package pin in this skill's source
  ledger.
- The v4 package source exports CSS entrypoints such as `index.css`,
  `theme.css`, `utilities.css`, and package source TypeScript. Use local
  installed versions before assuming latest-doc behavior.

## CSS-First Configuration

Tailwind v4 moved configuration toward CSS:

- import Tailwind with `@import "tailwindcss";`;
- define design tokens and custom utilities in CSS;
- expose design tokens as native CSS variables;
- use modern CSS features such as cascade layers, registered custom
  properties, `color-mix()`, and logical properties.

For motion work, this means `@theme` is usually the right place for reusable
animation and easing tokens in v4 projects.

## Source Detection

Tailwind v4 has automatic content detection and ignores common generated,
dependency, binary, and gitignored paths. Use CSS directives when detection
needs adjustment:

- `@source "../path"` registers additional source files.
- `@source not "../path"` excludes noisy paths.
- `@import "tailwindcss" source(none);` disables automatic detection for that
  stylesheet.
- `@source inline("...")` safelists finite utilities that do not appear
  literally in source files.
- `@source not inline("...")` explicitly excludes finite utilities.

Do not carry old JS-config safelist habits into v4 unless the project still
uses a v3-style config boundary.

## Motion-Relevant v4 Features

- Dynamic utility values reduce the need for many arbitrary spacing and sizing
  values, but they do not make open-ended JavaScript class interpolation safe.
- `starting:` supports the CSS `@starting-style` feature for entry
  transitions in compatible browsers.
- `transition-discrete` supports discrete transition behavior where available.
- Registered custom properties can make some variable-driven visual effects
  interpolate more predictably.
- Theme variables can be reused by JavaScript animation libraries through CSS
  variables when a library boundary needs shared design tokens.
- Tailwind v4 core browser support depends on modern CSS features. Check local
  product policy before relying on newer motion-adjacent behavior such as
  discrete transitions or entry transitions.

## Review Notes

- Check the installed Tailwind version before using v4-only directives.
- Prefer CSS entrypoint changes over adding a new Tailwind config file in v4
  projects.
- Keep generated class sets finite and auditable.
- Avoid v3-style `content`/`safelist` edits in v4 projects unless the repo
  documents a compatibility boundary.
- Verify class generation after adding `@source inline()` or custom
  `--animate-*` tokens.
