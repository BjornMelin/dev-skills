---
name: pre-mortem
description: "Adversarial pre-mortem that stress-tests a plan or design before reality does: imagine it failed 12 months out and work backwards to surface the assumptions, vulnerabilities, dependency chains, and irreversible bets, then harden them. Use before committing to a high-stakes or hard-to-reverse plan, when you are only hearing positive feedback, or when a decision hinges on assumptions that have not been validated. Opt-in only."
disable-model-invocation: true
---

Stress-test this plan before reality does. Not to kill it, but to make it survive contact with reality.

The technique: imagine it is 12 months from now and this plan failed. Work backwards, find why, then harden the plan against it.

Run the five-step framework and produce the Challenge Report described in [references/pre-mortem-framework.md](references/pre-mortem-framework.md): extract the assumptions the plan needs to be true, rate each on confidence and impact-if-wrong, map the vulnerabilities (low confidence + high impact), trace the dependency chain, and test reversibility. Close with kill switches and hardening actions.

Find facts yourself rather than asking: explore the codebase, use `context7`, `gh`, `opensrc`, or web search for what you can look up, and dispatch the `/research` skill for anything that needs real investigation.

Do not treat the output as a verdict; it is a vulnerability map. Surface the risky bets so they can be validated, hedged, or accepted knowingly. Unknown risks are dangerous; known risks are manageable.
