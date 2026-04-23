# Tool Routing

Use this matrix to avoid rebuilding workflows that already exist elsewhere.

## Primary lanes

- PR discovery, review bundles, and thread-aware remediation
  - Prefer GitHub plugin reads plus `review-pack`.
  - Use `$gh-pr-review-fix` as the active review-remediation router.
- GitHub Actions failure diagnosis
  - Use `$github:gh-fix-ci`.
  - Inspect the exact failed run and logs before editing.
- Expo and EAS
  - Prefer `$expo:expo-cicd-workflows` for workflow shape and validation.
  - Prefer `$expo:upgrading-expo` for SDK, dependency, prebuild, or version
    drift issues.
  - Use Expo MCP when available for current Expo platform guidance.
  - Use EAS CLI through repo-native commands for live workflow and build state.
- Vercel, Turborepo, and build graph issues
  - Prefer Vercel plugin capabilities plus `$vercel:turborepo`.
  - Use Vercel CLI for deployment, logs, and project state.
  - Use Turbo CLI for graph, affected, dry-run, and cache inspection.
- Bun and package-manager/runtime posture
  - Use `$bun-dev`.
- Repo-wide docs alignment
  - Prefer `$repo-docs-align` if installed.
  - Else prefer `$docs-align`.
  - Else perform a direct authority-doc sweep anchored on the changed concerns.
- Post-push monitoring and late-arriving feedback
  - Use `$babysit-pr`.
- Commit shaping
  - Use `$commit`.

## Source priority

For time-sensitive or platform-specific conclusions, prefer this order:

1. live repo and hosted state
2. official docs and CLI help
3. official release notes and changelogs
4. official GitHub repositories and API references
5. local skill references

If the current repo or hosted state contradicts an older review comment, trust
the live state.

## Signr command posture

- Use repo scripts first.
- Use `bunx vercel` from `apps/web`.
- Use `cd apps/mobile && bun run eas -- <command>` for EAS work.
- Use `bunx turbo` only when direct Turbo inspection adds value beyond the repo
  scripts.
- Never switch to `npm`, `npx`, `pnpm`, or `yarn` for repo tasks.
