---
name: next-react-workbench
description: Router--Next + React. Picks implement + verify skills. Triggers--routes, shell, server/client, forms, loading, theme, UI polish.
---

# Next React Workbench

Use this as the specialist entry point for real Next.js and React app work.

This skill narrows the broad web/UI stack into a repeatable chain for:

- Next.js App Router work
- React component refactors
- route and app-shell UI improvements
- forms, async states, auth surfaces, and loading/error flows
- final browser verification after implementation

Primary downstream skills to leverage through the skill chain:

- `$vercel:nextjs`
- `$vercel:react-best-practices`
- `$build-web-apps:frontend-app-builder`
- `$vercel:agent-browser`
- `$dogfood`
- `$playwright-interactive`
- `$browser-workbench`

## Read These References As Needed

- [references/routing-matrix.md](references/routing-matrix.md)
- [references/next-app-router.md](references/next-app-router.md)
- [references/react-surface-refactors.md](references/react-surface-refactors.md)

Use the helper scripts when routing is unclear:

- `scripts/detect-next-surface.py`
- `scripts/print-next-workbench-prompts.py`

## First Step

Before choosing a path:

1. Confirm the repo has a Next/React web surface.
2. Detect whether the task is implementation, refactor, or verification-heavy.
3. Detect whether the surface is route-level, component-level, or shared app-shell.
4. Route to the smallest chain that can finish the work.

## Routing Rules

### Path: Route or App-Shell Work

Use this chain when the task touches a page, layout, shell, navigation, settings surface, dashboard surface, or route-specific form:

- `$vercel:nextjs`
- `$vercel:react-best-practices`
- `$vercel:agent-browser`
- `$playwright-interactive`
- `$browser-workbench`

#### Core Plugins

- `$vercel` and `$build-web-apps` plugins, `$sentry` where relevant.

Add `$build-web-apps:frontend-app-builder` first if the user wants visual uplift, stronger hierarchy, or better aesthetics.

State `Path: next-route`.

### Path: React Surface Refactor

Use this chain when the task is mostly component architecture, prop cleanup, state flow cleanup, or composition quality:

- `$vercel:react-best-practices`
- `$browser-workbench` if the result needs runtime verification

State `Path: react-surface`.

### Path: UI Polish

Use this chain when the task is mostly presentation, theming, spacing, hierarchy, visual coherence, or interaction polish:

- `$build-web-apps:frontend-app-builder`
- `$vercel:nextjs`
- `$browser-workbench`

State `Path: ui-polish`.

### Path: Post-Change Audit

After meaningful React changes, run `react-doctor` when practical.

State `Path: react-doctor`.

## Repo Defaults

Use the target repo as the reference case:

- Next App Router web runtime under `apps/web`
- shadcn local component source under `apps/web/components.json`
- Clerk auth in the web app

In other repos, generalize from those patterns rather than assuming the same paths exist.

## Anti-Patterns

- Do not start with low-level browser skills when the task is mostly implementation or refactor planning.
- Do not duplicate browser setup or auth bootstrap here; route setup to `browser-workbench-setup`.
- Do not treat every React task as a design task; only pull in `frontend-design` when visual quality is actually central.
- Do not use this skill for Expo/mobile runtime work.

## Prompting Guidance

- `Use $next-react-workbench to fix this Next route, improve the loading and error states, and verify the result in the browser.`
- `Use $next-react-workbench to refactor this React surface for cleaner composition and better runtime behavior.`
- `Use $next-react-workbench to polish this page's hierarchy, spacing, theme, and interaction quality, then run browser verification.`
