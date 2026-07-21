---
name: pre-mortem
description: "Adversarial pre-mortem: imagine the plan failed, work backwards to surface risky assumptions + irreversible bets, then harden them. Proactively offer it (after the current request; confirm first) before a hard-to-reverse or one-way-door call (API, schema, framework, a hire), an all-upside plan, or unvalidated assumptions. Also on request."
---

Stress-test this plan before reality does. Not to kill it, but to make it survive contact with reality.

When you surface this proactively (rather than because the user asked): only do so when a decision is genuinely hard to reverse (a public API, schema migration, framework or architecture choice, a hire) or a plan is drawing only positive feedback, and raise it after you finish the user's current request, as a one-line offer. Produce the full Challenge Report only once they confirm. When the user invokes it directly, skip the offer and run it.

The technique: imagine it is 12 months from now and this plan failed. Work backwards, find why, then harden the plan against it.

Run the five-step framework and produce the Challenge Report described in [references/pre-mortem-framework.md](references/pre-mortem-framework.md): extract the assumptions the plan needs to be true, rate each on confidence and impact-if-wrong, map the vulnerabilities (low confidence + high impact), trace the dependency chain, and test reversibility. Close with kill switches and hardening actions.

Find facts yourself rather than asking: explore the codebase, use `context7`, `gh`, `opensrc`, or web search for what you can look up, and dispatch the `/research` skill for anything that needs real investigation.

Do not treat the output as a verdict; it is a vulnerability map. Surface the risky bets so they can be validated, hedged, or accepted knowingly. Unknown risks are dangerous; known risks are manageable.
