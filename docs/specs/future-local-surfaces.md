# Future Local App Surfaces

Status: deferred design gate.

Tracking: #37 and #57.

This record evaluates two possible future surfaces for the dev-skills local
agent platform:

- a Tauri v2 desktop workbench;
- an Axum/Tower local web service for browser or agent clients.

The current decision is to build neither surface in the v0.3/v1 release wave.
The local CLI, shared `codex-dev-core` contracts, read-only Ratatui TUI,
apply-gated PR-agent model, and audited local release baseline remain the
priority.

## Decision

Defer Tauri and Axum implementation until the local control plane is stable
enough that either surface can reuse existing contracts without inventing a
parallel application model.

The only implementation decision that currently clears the repo's 9.0 major
choice target is the deferral itself. Both future implementation options have
credible value, but each adds a new trust boundary, lifecycle model, and
support surface before the current CLI/TUI work has produced enough operating
evidence.

## Decision Framework

Scores use the repo-wide major-choice weights from AGENTS.md.

| Option | Solution leverage 35% | Application value 30% | Maintenance load 25% | Adaptability 10% | Weighted score | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Defer both; keep CLI/TUI as the shipped local surface | 9.2 | 9.4 | 9.8 | 9.0 | 9.4 | Selected |
| Add an Axum/Tower local service after gates pass | 8.6 | 8.4 | 7.1 | 8.7 | 8.2 | Future candidate |
| Add a Tauri v2 desktop app after gates pass | 8.1 | 8.2 | 6.4 | 7.7 | 7.7 | Future candidate |
| Build both now | 5.0 | 5.8 | 2.8 | 4.4 | 4.6 | Rejected |

The future candidates must be rescored from current code and current upstream
docs before any implementation issue is opened. A future Tauri or Axum issue
should proceed only if its implementation plan reaches 9.0 or higher without
weakening the CLI/TUI contracts.

## Product Fit

Tauri could become useful when the repo needs a persistent desktop workbench
with native menus, system notifications, signed updates, local file affordances,
or a richer offline operator interface than a terminal can provide. It is a
poor near-term fit because it introduces frontend assets, desktop packaging,
capability policy, updater signing, and webview IPC review before the local
contract layer has stabilized.

Axum could become useful when the repo needs a local HTTP API over
`codex-dev-core` read models for browser dashboards, controlled local agent
clients, or richer inspection workflows that do not fit a TUI. It is a poor
near-term fit because it introduces daemon lifecycle, port binding, CORS, local
auth, request logging, and shutdown semantics before the read/write policy model
has enough evidence from CLI and PR-agent use.

Neither future surface should own development logic. Both must remain thin
consumers of `codex-dev-core` and existing CLI commands.

## Prerequisites

Do not open an implementation issue for either surface until all of these are
true:

- `codex-dev-core` has stable task, evidence, validation, subagent, PR, and
  PR-agent read models with schema tests and docs.
- `codex-dev` is the canonical write boundary for local task capsules,
  validation evidence, and apply-gated PR-agent actions.
- The PR-agent safety model has production evidence from merged PRs, including
  dry-run defaults, explicit `--apply`, target verification, stale-thread
  handling, idempotency, and token redaction.
- `codex-dev-tui` has produced enough dashboard and detail-view learnings to
  know which information density, filters, and panels are worth carrying into a
  richer UI.
- The canonical local release and validation owners are stable. Future surface
  issues must use the release or `full_local` policy profile and the
  [Local Release and Supply Chain](../runbooks/local-release-supply-chain.md)
  runbook unless they document an explicit, reviewed deviation.
- Local token posture is explicit: no raw GitHub, OpenAI, or provider tokens in
  UI state, browser storage, logs, screenshots, capsules, or update metadata.
- The threat model for local untrusted content is accepted. Branch names, PR
  comments, CI logs, review comments, provider summaries, and command output are
  attacker-controlled text until validated and rendered safely.

## Tauri Gate

Open a Tauri issue only when the prerequisites pass and a native desktop
workbench has a concrete job the terminal TUI cannot do well.

Required design constraints:

- Use Tauri only as a desktop shell over existing `codex-dev-core` read models
  and explicit `codex-dev` command flows.
- Treat the webview/Rust IPC boundary as hostile by default. The frontend sends
  typed requests; Rust commands validate every field and return redacted typed
  results.
- Keep the command inventory small. Every `#[tauri::command]` must be listed in
  a design table, registered through a single handler set, and mapped to the
  contract or CLI behavior it exposes.
- Capabilities must be deny-by-default and window-scoped. Plugin permissions
  must be minimal; broad filesystem, shell, process, localhost, clipboard, and
  updater access require separate justification.
- Capability files must be explicitly allowlisted from configuration. No future
  Tauri branch may rely on implicit discovery of every file under
  `src-tauri/capabilities`, and no `windows: ["*"]` capability is allowed
  without a written threat-model exception and a CI or docs check for broad or
  unreferenced capability files.
- Do not put provider tokens or hosted-write credentials in frontend state,
  webview storage, screenshots, logs, or crash reports.
- Use a strict content security policy and avoid loading remote UI code.
- Do not block the UI thread with filesystem, GitHub, validation, or package
  operations. Long-running work remains command-backed and cancellable.
- Updater support requires signed artifacts, protected private signing keys,
  HTTPS release metadata, and an operator runbook before it can ship.
- Any sidecar or shell access requires a separate security review.

Useful official references:

- Tauri security and trust boundaries: <https://v2.tauri.app/security/>
- Tauri Rust command IPC: <https://v2.tauri.app/develop/calling-rust/>
- Tauri plugin permissions and capabilities:
  <https://v2.tauri.app/learn/security/using-plugin-permissions/>
- Tauri updater signing: <https://v2.tauri.app/plugin/updater/>

## Axum Gate

Open an Axum issue only when the prerequisites pass and local HTTP unlocks a
real workflow that the CLI/TUI cannot satisfy.

Required design constraints:

- Bind to `127.0.0.1` by default, use an explicit random or configured port,
  and never expose a LAN listener without a separate security issue.
- Use per-session local auth for every non-health endpoint. A bearer token or
  equivalent local secret must be generated at launch, redacted in logs, and
  rotated on restart. Auth material must travel through an `Authorization`
  header or a non-URL one-time bootstrap flow, never query parameters,
  fragments, or ambient cookies.
- Enforce strict CORS, Origin, and Host validation. Do not use wildcard CORS
  for authenticated endpoints.
- Start read-only. Mutation endpoints must map to existing apply-gated
  `codex-dev` commands and preserve the PR-agent safety model.
- Use typed Axum extractors and `State` over clone-cheap shared state. Handlers
  stay thin and return typed errors.
- Add Tower/Tower HTTP layers deliberately: request IDs, sensitive-header
  redaction, tracing, request body limits, timeouts, CORS, and panic handling
  where appropriate.
- Support graceful shutdown. No background daemon autostart, hidden process
  manager, or lingering port after the parent command exits.
- Keep logs and responses redacted. CI logs, PR comments, provider summaries,
  command output, and local filesystem paths are not safe to echo raw.

Useful official references:

- Axum Router, handlers, extractors, responses, middleware, and state:
  <https://docs.rs/axum/latest/axum/>
- Tower HTTP middleware modules and feature flags:
  <https://docs.rs/tower-http/latest/tower_http/>

## Threat Model

Protected assets:

- provider and GitHub tokens;
- local filesystem paths and command output;
- task capsule evidence and PR review state;
- updater signing keys if a desktop app exists later;
- hosted PR mutation authority.

Attacker-controlled inputs:

- repository names, branch names, issue and PR titles, review comments, CI logs,
  provider summaries, local command output, Markdown evidence, and file paths
  from untrusted repositories.

Required invariants:

- No future surface gets direct hosted-write authority by default.
- No future surface bypasses `codex-dev-core` validation or `codex-dev`
  apply-gated command boundaries.
- No future surface logs raw secrets or renders untrusted text as executable
  markup.
- No future surface becomes the source of truth for task capsule contracts.

## Non-Goals

- No desktop app in the current roadmap.
- No HTTP daemon in the current roadmap.
- No browser UI implementation in the current roadmap.
- No Tauri, Axum, frontend, or daemon dependencies in this branch.
- No change to the local CLI/TUI priority.
- No early choice of frontend framework, desktop bundling strategy, or web app
  architecture.

## Future Issue Template

Before opening a future Tauri or Axum implementation issue, include:

- the concrete workflow the terminal TUI cannot satisfy;
- current weighted decision scores;
- exact contract files and `codex-dev-core` read models to consume;
- command/write boundaries and mutation safeguards;
- security review checklist;
- canonical release or `full_local` validation profile plus any explicit
  deviations from the local release runbook;
- release/install/update impact, including updater or daemon lifecycle changes;
- residual risks and rollback plan.
