---
description: Operating mode for high-risk or review-heavy repos where findings, upgrade waves, and touched-files mapping should be finalized before edits begin.
---

# Plan-First Variant

Use this reference when the repo is high-risk, the blast radius is large, or
the user expects an explicit plan before edits.

## Operating Assumptions

- Do not begin broad edits immediately.
- Finish the inventory, research, risk analysis, and execution map first.
- The plan must be decision-complete, not aspirational.

## Required Output Before Editing

1. Findings:
   - vulnerabilities
   - outdated dependencies
   - deprecated APIs
   - abandoned packages
   - custom code likely removable after upgrades
2. Upgrade matrix:
   - package
   - current version
   - target version
   - latest safe version
   - breaking-change notes
   - affected packages/apps
3. Touched-files map:
   - exact directories and files likely to change
   - framework lanes involved
   - verification commands required
4. Risk map:
   - public contract boundaries
   - persisted data boundaries
   - likely migration hotspots
   - any forced incremental major-version steps

## Execution Bias

- Once the plan is approved or clearly implied by the workflow, execute
  end-to-end without reopening basic questions already resolved by the plan.
- Keep edits grouped by upgrade wave so the final diff remains explainable.
- Maintain hard-cut and reducing-entropy bias during implementation, not just in
  the plan.

## Verification Bias

- State expected verification before edits, then run it after edits.
- If the plan assumed a framework migration or codemod step, verify that it
  actually landed everywhere.
- Report residual risks separately from completed work.
