# Research Lanes

Default lanes:

- `$repo-modernize-upgrade-audit`
- `$opensrc`
- `$opensrc-inspect`
- `$technical-writing`
- `$hard-cut`

Conditional lanes:

- `$bun-dev`
  - only when Bun is part of the target repo's package-manager or runtime
    posture
- `$github`
  - when release metadata, issues, PRs, or repo state materially improve the
    pack
- framework/plugin lanes
  - only when the target repo detects that framework or platform
  - for Next.js packs, prefer `$vercel` and `$vercel:nextjs` alongside official
    `nextjs.org` docs
  - for Expo/EAS packs, prefer `$expo` and `$expo:upgrading-expo` alongside
    official `docs.expo.dev` guidance
  - for Convex packs, prefer the repo-local Convex skill plus official
    `docs.convex.dev` guidance
  - for Turborepo packs, prefer `$turborepo` and `$vercel:turborepo`
    alongside official `turborepo.com` docs
- browser verification lanes
  - only when the package family changes visible UI behavior

Do not use raster/image-generation lanes for dependency upgrade packs unless
the dependency family explicitly includes bitmap asset generation.
