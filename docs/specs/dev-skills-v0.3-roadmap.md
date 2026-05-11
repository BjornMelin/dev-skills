# dev-skills v0.3/v1 Roadmap

Status: planned.

Tracking: #37 through #57.

This roadmap is the canonical source for the next release sequence after the
initial `codex-dev`, `codex-dev-tui`, and `codex-research` operating-layer
wave. Implement each child issue as one branch and one pull request into
`main`. After each merge, sync local `main` before starting the next issue.

## Release Boundary

The v0.3/v1 roadmap turns the repo into a local agent development platform:

- `codex-dev` records strict task, evidence, validation, subagent, and PR state.
- `codex-dev-core` becomes the shared contract and read-model crate for local
  tools.
- `codex-dev-tui` becomes the read-only terminal workbench over local task
  capsules.
- `codex-research` keeps provider routing, source hydration, evidence bundles,
  cache, ledgers, reports, budgets, and evals.
- Skill, subagent, validation, release, and install workflows become explicit
  enough for agents to run without guessing.

The first execution wave remains local CLI/TUI first. Tauri desktop and Axum
local web service surfaces are design-only until contract, security, release,
and PR-agent prerequisites are stable.

## Approved Decisions

| Decision | Selected option | Score | Deferred alternatives |
| --- | --- | ---: | --- |
| Issue graph breadth | 20 child issues plus parent epic | 9.3 | Compressing to 16 issues would make later PRs larger; expanding further would delay execution. |
| Product surface | Local CLI/TUI first | 9.7 | Desktop and web surfaces add security, auth, updater, and daemon lifecycle before contracts are stable. |
| Contract posture | Strict hard-cut | 9.5 | Guided migration keeps temporary compatibility; loose convenience weakens the control plane. |
| PR automation | Full PR agent, apply-gated writes | 9.1 | Autonomous default increases blast radius; read-only conflicts with the roadmap goal. |
| Rust architecture | Extract `codex-dev-core` first | 9.2 | Two core crates or a full SDK layer add complexity before contracts prove themselves. |
| Release posture | Audited local release first | 9.2 | Publishing crates now is irreversible; skipping gates weakens trust in globally installed tooling. |
| Future surfaces | Planning issues only | 9.0 | Building Tauri/Axum now would dilute local CLI/TUI and PR-agent work; omitting them loses roadmap context. |

## Execution Rules

- Work in issue order unless a later issue is explicitly proven independent and
  the parent epic is updated.
- Use the branch name listed in this roadmap unless the issue body is updated
  with a better SemVer-safe Conventional Commit branch.
- Keep every branch independently reviewable and linked to its child issue and
  #37.
- Before creating each PR, run specialized subspawn review agents against the
  branch diff and fix only valid findings.
- PR descriptions must include summary, linked issue, validation evidence, docs
  impact, provider/deploy notes, screenshots where relevant, and residual risk.
- Babysit every PR until CI passes and hosted review state is clean. Verify
  review comments against current code before fixing or answering them.
- If a CodeRabbit or human review comment is stale, incorrect, or less optimal
  than the implementation plan, answer with evidence instead of blindly applying
  the suggestion.
- Merge only when checks pass, review threads are clean, and the PR is ready.
  Then return to `main`, pull/fetch, close or update the child issue, and start
  the next issue.

## Issue Ledger

| Issue | Branch | Owner | Depends on | Validation lane | Non-goals |
| --- | --- | --- | --- | --- | --- |
| #38 `docs(roadmap): define the v0.3 operating graph` | `docs/dev-skills-v0.3-roadmap` | Docs/specs | #37 | Docs links, whitespace | No CLI, TUI, PR-agent, or release implementation. |
| #39 `fix(codex-dev): stabilize capsule and PR evidence contracts` | `fix/codex-dev-contract-stabilization` | `codex-dev`, `codex-dev-tui` contracts | #38 | `codex-dev`, `codex-dev-tui`, docs links | No new PR-agent behavior or core crate extraction. |
| #40 `feat(codex-dev-core): extract shared contract/read-model crate` | `feat/codex-dev-core-contracts` | `codex-dev-core` | #39 | `codex-dev-core`, `codex-dev`, `codex-dev-tui`, workspace check | No GitHub/provider APIs or broad SDK layer. |
| #41 `security(codex-dev): define PR-agent token and hosted-write safety model` | `security/codex-dev-pr-agent-safety-model` | Security docs, PR-agent design | #38, #40 direction | Docs links, whitespace | No hosted writes, tokens, or new secret files. |
| #42 `feat(codex-dev): add structured evidence append commands` | `feat/codex-dev-evidence-appenders` | `codex-dev`, `codex-dev-core` | #39, #40 | `codex-dev-core`, `codex-dev`, docs links | No provider calls, raw dumps, or TUI panels. |
| #43 `feat(codex-dev): record subspawn plans and outcomes` | `feat/codex-dev-subagent-evidence` | `codex-dev`, `subspawn` bridge | #42 | `codex-dev`, subspawn validators, eval lab | No direct subagent spawning from `codex-dev`. |
| #44 `feat(codex-dev): add validation policy profiles` | `feat/codex-dev-policy-profiles` | `codex-dev` policy | #39, #40 | `codex-dev` policy manifests, docs links | No automatic full-gate execution by default. |
| #45 `chore(validation): make validation matrix manifest-backed` | `chore/validation-matrix-manifest` | Validation docs/tooling | #44 | manifest tests, docs links, Python compile checks | No replacement for repo-native validators. |
| #46 `feat(codex-dev): normalize GitHub and review-pack PR evidence` | `feat/codex-dev-pr-evidence-normalizers` | PR evidence contracts | #39, #40, #41 | `codex-dev` fixture tests, docs links | No comments, thread resolution, reruns, or merges. |
| #47 `feat(codex-dev): add live PR-agent state engine` | `feat/codex-dev-pr-agent-state` | PR-agent state | #41, #46 | `codex-dev` state fixtures, optional sandbox smoke | No hosted writes or daemon. |
| #48 `feat(codex-dev): add apply-gated hosted PR actions` | `feat/codex-dev-pr-agent-apply-actions` | PR-agent writes | #41, #47 | offline action-plan tests, optional sandbox smoke | No default writes and no auto-merge. |
| #49 `feat(codex-dev): add CI, review, and merge readiness loop` | `feat/codex-dev-pr-agent-readiness-loop` | PR-agent closeout | #47, #48 | PR-state fixtures, optional sandbox smoke | No daemon and no merge with unresolved valid feedback. |
| #50 `feat(codex-research): add evidence bundle closeout` | `feat/codex-research-evidence-bundles` | `codex-research` | #40 | `codex-research` tests, doctor, strict eval | No provider credential changes or private raw dumps. |
| #51 `refactor(codex-research): split provider, run, cache, ledger, and eval modules` | `refactor/codex-research-modules` | `codex-research` modules | #50 if closeout clarifies ownership | `codex-research` tests, doctor, evals | No new providers or CLI behavior changes. |
| #52 `feat(codex-dev-tui): add multi-capsule dashboard navigation` | `feat/codex-dev-tui-dashboard` | `codex-dev-tui` | #39, #40 | `codex-dev-tui`, manual terminal smoke, docs links | No hosted writes, web app, or validation execution. |
| #53 `feat(codex-dev-tui): add evidence, subagent, and PR-agent panels` | `feat/codex-dev-tui-evidence-panels` | `codex-dev-tui` panels | #42, #43, #46, #52 | TUI fixtures/smoke, docs links | No live GitHub/provider calls from TUI. |
| #54 `chore(release): add audited local release and supply-chain baseline` | `chore/local-release-supply-chain-baseline` | Release and supply chain | #40, stable CLI shape | Cargo metadata/tree/audit/deny where available, docs links | No crates.io publish or credentials. |
| #55 `feat(cli): add completions, manpages, and global install/update workflow` | `feat/cli-completions-manpages-install-update` | CLI UX/install docs | #54 | workspace tests, install/help smoke, docs links | No public binary release or shell rc mutation. |
| #56 `docs(skills): clarify subagent template and skill packaging authority` | `docs/subagent-template-skill-packaging-authority` | Skills/subagent docs | #43 or independent docs-only path | skill validators, subagent validators, docs links | No global installs or broad skill rewrites. |
| #57 `docs(future): design gated Tauri desktop and Axum local web surfaces` | `docs/future-tauri-axum-surfaces` | Future surface design | #40, #41, #52, #54 | Docs links, whitespace | No Tauri, Axum, frontend assets, or daemon implementation. |

## First-Wave Dependency Graph

The first wave is intentionally contract-first:

```text
#38 roadmap
  -> #39 strict contracts
  -> #40 codex-dev-core
  -> #41 PR-agent safety model
  -> #42 evidence appenders
  -> #43 subspawn evidence
  -> #44 policy profiles
  -> #45 validation manifest
  -> #46 PR evidence normalizers
  -> #47 PR-agent state
  -> #48 apply-gated hosted actions
  -> #49 readiness loop
```

`codex-research` work (#50 and #51), TUI work (#52 and #53), release/install
work (#54 and #55), skill packaging docs (#56), and future-surface design (#57)
can begin only when their listed prerequisites are merged and local `main` is
current.

## External Reference Families

Use current official docs or primary source pages when implementing each lane:

- OpenAI Codex AGENTS.md:
  <https://developers.openai.com/codex/guides/agents-md>
- OpenAI Codex skills: <https://developers.openai.com/codex/skills>
- OpenAI Codex subagents: <https://developers.openai.com/codex/subagents>
- OpenAI Codex internet access and safety:
  <https://developers.openai.com/codex/cloud/internet-access>
- Clap: <https://docs.rs/clap/latest/clap/>
- clap derive: <https://docs.rs/clap/latest/clap/_derive/>
- clap_complete: <https://docs.rs/clap_complete/latest/clap_complete/>
- clap_mangen: <https://docs.rs/clap_mangen/latest/clap_mangen/>
- Ratatui: <https://docs.rs/ratatui/latest/ratatui/>
- Ratatui event handling: <https://ratatui.rs/concepts/event-handling/>
- Ratatui backends: <https://ratatui.rs/concepts/backends/>
- GitHub REST pull requests: <https://docs.github.com/en/rest/pulls/pulls>
- GitHub REST reviews: <https://docs.github.com/en/rest/pulls/reviews>
- GitHub REST review comments:
  <https://docs.github.com/en/rest/pulls/comments>
- GitHub REST checks: <https://docs.github.com/en/rest/checks/runs>
- GitHub REST statuses: <https://docs.github.com/en/rest/commits/statuses>
- GitHub Actions workflow runs:
  <https://docs.github.com/en/rest/actions/workflow-runs>
- GitHub rate limits:
  <https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api>
- GitHub GraphQL mutations:
  <https://docs.github.com/en/graphql/reference/mutations>
- Cargo install:
  <https://doc.rust-lang.org/cargo/commands/cargo-install.html>
- Cargo package:
  <https://doc.rust-lang.org/cargo/commands/cargo-package.html>
- Cargo publish:
  <https://doc.rust-lang.org/cargo/commands/cargo-publish.html>
- Cargo manifest: <https://doc.rust-lang.org/cargo/reference/manifest.html>
- RustSec: <https://rustsec.org/>
- cargo-audit:
  <https://github.com/rustsec/rustsec/tree/main/cargo-audit>
- cargo-deny: <https://embarkstudios.github.io/cargo-deny/>
- cargo-vet: <https://mozilla.github.io/cargo-vet/>
- Axum: <https://docs.rs/axum/latest/axum/>
- tower-http: <https://docs.rs/tower-http/latest/tower_http/>
- Tauri v2 security: <https://v2.tauri.app/security/>
- Tauri v2 calling Rust: <https://v2.tauri.app/develop/calling-rust/>
- Tauri plugin permissions:
  <https://v2.tauri.app/learn/security/using-plugin-permissions/>
- Tauri updater: <https://v2.tauri.app/plugin/updater/>

## Final Release Closeout

The roadmap is complete only when:

- #38 through #57 are closed or explicitly updated with a documented deferral.
- Every implementation PR is merged into `main`.
- Local `main` is synced with `origin/main`.
- Final repo gates pass using the validation matrix that exists after #45.
- Global/local CLI install posture is verified after #55.
- #37 contains final evidence links and no residual required follow-up.
