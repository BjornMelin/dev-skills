# AI Stack Scanner

`ai_stack_scan.v1` is an offline JSON contract for finding common AI application
stack migration and safety signals before Codex edits a TypeScript repository.
It is intentionally conservative: every finding is a review prompt, not an
automatic fix instruction.

## Commands

Run the canonical scanner from this repository against the target repo:

```bash
python3 skills/ai-sdk-core/scripts/ai_stack_scan.py --root <repo> --pretty
```

Skill-local wrappers default to the matching family:

```bash
python3 skills/ai-sdk-agents/scripts/ai_stack_scan.py --root <repo> --pretty
python3 skills/ai-sdk-ui/scripts/ai_stack_scan.py --root <repo> --pretty
python3 skills/streamdown/scripts/ai_stack_scan.py --root <repo> --pretty
python3 skills/zod-v4/scripts/ai_stack_scan.py --root <repo> --pretty
python3 skills/supabase-ts/scripts/ai_stack_scan.py --root <repo> --pretty
```

Use `--family` to narrow or repeat families. The scanner performs no network
calls, prunes generated/vendor directories before descent, skips symlinked
files, and only reads contained files under `--root`.

Scanner JSON is repo-confidential evidence. Do not paste full output into
external search, Firecrawl, Exa, issue trackers, or PR comments. Share only the
specific redacted signals needed for review. Use `--include-evidence` only for
local debugging when source snippets are safe to display.

## JSON Contract

Top-level fields:

- `schema`: always `ai_stack_scan.v1`
- `scanner_version`: scanner ruleset date
- `root`: `<scan-root>` by default; absolute scan root only when
  `--include-absolute-root` is passed
- `families`: selected families
- `docs`: current authority URLs to verify findings
- `package_manifests`: package.json dependency snapshots
- `packages`: recognized AI stack packages and version hints
- `signals`: deterministic findings with `id`, `family`, `severity`, `path`,
  optional `line`, optional `evidence`, and `docs`
- `summary`: signal counts by severity and family
- `privacy`: whether source evidence or absolute root paths were included

Severity is advisory. `error` means a signal is security-sensitive or likely
unsafe, but the next step is still to verify against live code and current docs.

## Rule Families

- `ai-sdk-core`: AI SDK package majors, legacy `maxSteps`, removed stream
  response helpers, missing `tool({ inputSchema })`, and MCP client lifecycle
  cleanup prompts.
- `ai-sdk-ui`: `UIMessage` content-to-parts migration, legacy `useChat`
  helper usage, missing `@ai-sdk/react` dependency, and current stream response
  helpers.
- `ai-sdk-agents`: `ToolLoopAgent` loop-control prompts, especially explicit
  `stopWhen` for repeated tool loops.
- `streamdown`: `react-markdown` in AI streaming chat surfaces, `Streamdown`
  without `isAnimating`, missing dependency declarations, and Tailwind
  `streamdown/dist` source/content configuration, including CSS `@source`.
- `zod-v4`: pre-v4 package specs, deprecated string-format methods,
  `required_error`/`invalid_type_error`, `{ message: ... }`, `z.nativeEnum`,
  and `error.errors`.
- `supabase-ts`: legacy auth-helper packages, server-side `getSession()` authz
  checks, public service-role env names, service-role JWT literals,
  service-role references in explicit browser surfaces, and direct `auth.uid()`
  RLS calls. This is not a replacement for a dedicated secret scanner.

## Evidence Authorities

Use these as the default verification route after a scanner signal:

- AI SDK docs: <https://ai-sdk.dev/docs>
- AI SDK `stepCountIs`: <https://ai-sdk.dev/docs/reference/ai-sdk-core/step-count-is>
- Streamdown docs/source: <https://streamdown.ai> and
  <https://github.com/vercel/streamdown>
- Zod v4 docs: <https://zod.dev/v4>
- Supabase SSR auth docs: <https://supabase.com/docs/guides/auth/server-side>
- Supabase SSR source: <https://github.com/supabase/ssr>

When a scanner warning affects a public API, migration plan, security boundary,
or package version choice, refresh the official docs or source before patching.
