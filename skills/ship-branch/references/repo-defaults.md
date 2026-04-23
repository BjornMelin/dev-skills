# Repo Defaults

Use these defaults only when the user explicitly asks to scaffold missing repo files in the current repo.

## Contents

- `.github/pull_request_template.md`
- `AGENTS.md` PR title and description guidance
- `release-please-config.json`
- `.release-please-manifest.json`

## `AGENTS.md` section

```md
## Pull Request Title and Description Standards

When creating a new pull request, always produce a complete, reviewer-ready PR title and PR body.

### PR title
Use Conventional Commits format:

`<type>(<optional-scope>): <concise imperative summary>`

Examples:
- `feat(auth): add SSO callback validation`
- `fix(api): handle null tenant IDs in webhook parser`
- `refactor(ui): simplify settings page state`
- `feat(payments)!: remove legacy invoice endpoint`

Allowed common types:
- `feat`
- `fix`
- `refactor`
- `perf`
- `docs`
- `test`
- `build`
- `ci`
- `chore`
- `revert`

### Default versioning policy
Unless explicitly instructed otherwise for a specific repository, assume the repo is on a pre-1.0 development track and uses Release Please for automated release PRs after merges to `main`.

Default pre-1.0 release mapping:
- `fix:` -> patch bump
- `feat:` -> patch bump
- `!` or `BREAKING CHANGE:` -> minor bump

Important:
- Still mark breaking changes honestly with `!` or `BREAKING CHANGE:`
- Do not relabel a breaking change as non-breaking to suppress the bump
- Do not use `release-as` unless explicitly instructed
- Only use normal SemVer major bumps when the repo is already `1.0.0+` or the user explicitly overrides the default

### PR body requirements
Every PR body must follow the repository pull request template exactly and must include:

1. **Summary**
   - 2 to 5 concrete bullets
   - state what changed and where

2. **Why**
   - explain the bug, requirement, incident, product need, or technical motivation

3. **Scope**
   - state what is included
   - state what is intentionally not included

4. **Release Please version impact**
   - select the expected bump after merge to `main`
   - state public API impact explicitly
   - if breaking, document migration steps clearly

5. **Related issues / context**
   - use GitHub keywords when appropriate, such as `Closes #123`
   - include related design docs, specs, incidents, or follow-up PRs where relevant

6. **Implementation notes**
   - summarize important design or implementation decisions reviewers need to know

7. **Review guide**
   - provide recommended review order
   - identify risky files or areas
   - request specific review focus when useful

8. **Validation**
   - list automated checks actually run
   - list manual verification steps actually performed
   - include screenshots, logs, benchmarks, or sample payloads where relevant

9. **Risk**
   - assign Low, Medium, or High risk
   - list concrete failure modes and concerns

10. **Rollout / rollback**
   - document flags, deployment constraints, monitoring, and rollback path where relevant

### Quality rules
- Optimize for reviewer clarity, not author narration.
- Do not paste commit history into the PR body.
- Do not use vague claims like `fully tested` without listing actual validation.
- Do not claim `no breaking changes` unless API, schema, and behavior were checked.
- Do not omit migration notes for breaking changes.
- Keep PRs focused and single-purpose.
- Prefer precise bullets over long prose.
- If a section is not relevant, write `N/A` rather than filler.

### Merge and release safety
- Assume Release Please is the source of truth for release PR generation after merges to `main`
- Ensure PR titles are semantic and safe to become squash merge commit messages
- Prefer repository settings that default squash commit messages to the PR title
- Keep versions below `1.0.0` while the repo remains in active development unless explicitly instructed otherwise

### Special cases
For UI changes:
- include screenshots or recordings
- describe user-visible impact

For API or schema changes:
- include before/after examples
- document migration and compatibility impact

For performance changes:
- include benchmark method and results

For infra, security, or operational changes:
- document deployment impact, monitoring, and rollback clearly

### Output requirement
When asked to open a PR, always generate:
1. the final PR title
2. the complete PR body, filled in with repo-specific details
3. no placeholder sections unless required information is genuinely unavailable
```

## `.github/pull_request_template.md`

```md
## Summary
- 
- 
- 

## Why
- 

## Scope
### Included
- 

### Not included
- 

## Release Please version impact
Versioning track:
- [x] Pre-1.0 development track
- [ ] Stable 1.0+ SemVer track
- [ ] Explicit repo override applies

Expected Release Please bump after merge to `main`:
- [ ] None
- [ ] Patch
- [ ] Minor
- [ ] Major

Default interpretation for pre-1.0 repos unless explicitly overridden:
- `fix:` -> Patch
- `feat:` -> Patch
- `!` or `BREAKING CHANGE:` -> Minor

Public API impact:
- [ ] No public API change
- [ ] Public API added
- [ ] Public API changed, backward compatible
- [ ] Public API changed, breaking

Breaking changes:
- [ ] None
- [ ] Yes

If breaking, describe the change, affected users/systems, and required migration steps:
- 

## Related issues / context
- Closes #
- Related to #
- Depends on #

## Implementation notes
- 
- 

## Review guide
Recommended review order:
1. 
2. 
3. 

Areas needing extra attention:
- 

Specific feedback requested:
- [ ] Correctness
- [ ] Architecture
- [ ] API design
- [ ] Performance
- [ ] Security
- [ ] Backward compatibility / migration
- [ ] UX / accessibility / copy

## Validation
### Automated
- [ ] Tests added or updated
- [ ] Type checks pass
- [ ] Lint passes
- [ ] Build passes

### Manual
1. 
2. 
3. 

## Risk
Risk level:
- [ ] Low
- [ ] Medium
- [ ] High

Main risks:
- 
- 

## Rollout / rollback
- Feature flag:
- Deployment notes:
- Monitoring to watch:
- Rollback plan:

## Artifacts
- Screenshots:
- Logs:
- Benchmarks:
- Sample payloads:

## Checklist
- [ ] PR is focused and single-purpose
- [ ] PR title follows Conventional Commits
- [ ] Expected Release Please bump selected
- [ ] Public API impact documented
- [ ] Breaking changes and migration steps documented if applicable
- [ ] Linked issue(s) included where applicable
- [ ] Validation evidence included
- [ ] Risks and rollback documented
- [ ] Review guidance included
```

## `release-please-config.json`

```json
{
  "packages": {
    ".": {
      "release-type": "simple",
      "versioning": "default",
      "bump-minor-pre-major": true,
      "bump-patch-for-minor-pre-major": true
    }
  }
}
```

## `.release-please-manifest.json`

```json
{
  ".": "0.1.0"
}
```
