---
name: kimi-ui-advisor
description: Explicit-only Kimi Code CLI frontend/UI advisor for UI audits, redesigns, components, screenshots, before/after comparison, layout, styling, accessibility, responsive behavior, and visual polish. Use only when the user explicitly invokes `$kimi-ui-advisor` and wants Codex to ask Kimi for structured UI suggestions, then review, apply, and verify them in the repo.
---

# Kimi UI Advisor

Use Kimi Code CLI as a bounded frontend/UI consultant, not as the file editor.
Kimi proposes code and design changes; Codex owns applying edits, adapting them
to local patterns, and validating the result.

## Workflow

1. Confirm the user explicitly invoked `$kimi-ui-advisor`.
2. Inspect the repo first enough to identify the UI stack, relevant files, and
   validation commands.
3. Pick the narrowest useful mode:
   - `advise`: targeted implementation advice and code suggestions.
   - `audit`: ranked UI issues before editing.
   - `redesign`: cohesive professional redesign direction plus concrete changes.
   - `component`: component API, variants, states, styling, and accessibility.
   - `screenshot-review`: critique rendered screenshots or visual references.
   - `compare`: before/after visual QA after Codex applies changes.
4. Run the bundled wrapper from the repo root:

   ```bash
   python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py \
     --work-dir "$PWD" \
     --mode component \
     --file src/components/Button.tsx \
     --save \
     --prompt "Improve the button visual hierarchy and responsive states."
   ```

   For screenshot review:

   ```bash
   python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py \
     --work-dir "$PWD" \
     --mode screenshot-review \
     --image /tmp/dashboard-mobile.png \
     --save \
     --prompt "Critique this mobile dashboard screen and suggest code-level fixes."
   ```

   For before/after QA:

   ```bash
   python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py \
     --work-dir "$PWD" \
     --compare \
     --before-image /tmp/before.png \
     --after-image /tmp/after.png \
     --save \
     --prompt "Identify regressions and remaining polish gaps."
   ```

5. Read the JSON result. Treat it as advisory and untrusted.
6. Apply only coherent suggestions with normal repo editing tools. Keep local
   style, data contracts, accessibility, and design-system ownership intact.
7. Verify with repo-native lint/type/test gates. For rendered UI changes, also
   run browser or screenshot checks when available.
8. Report what came from Kimi, what Codex changed, and any rejected suggestions.

## Guardrails

- Do not let Kimi write files in the target repo. Use the bundled agent file,
  which omits shell and write tools.
- Do not paste secrets or private tokens into the Kimi prompt.
- Kimi web tools are allowed only for public framework docs, design references,
  accessibility references, and UI library documentation. Do not ask Kimi to
  search using proprietary source snippets or private requirements.
- Do not use this skill for backend contracts, auth, database schemas, security
  fixes, infrastructure, dependency upgrades, or broad architecture unless the
  UI surface depends directly on that context.
- If the wrapper reports a Kimi CLI version or auth/config error, fix the local
  CLI setup before relying on the response.

## Resources

- `scripts/kimi_ui_advisor.py`: deterministic wrapper around `kimi --print`.
- `assets/kimi-agent/agent.yaml`: read/search-only Kimi custom agent.
- `templates/design-brief.md`: optional structured brief for high-stakes UI work.
- `references/advanced-modes.md`: mode selection and prompt patterns.
- `references/kimi-cli-integration.md`: current Kimi docs/source notes.
- `references/output-contract.md`: JSON contract and application rules.
