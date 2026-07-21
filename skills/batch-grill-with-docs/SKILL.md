---
name: batch-grill-with-docs
description: "Batch-grill a plan or design round by round: whole settled frontier per round, recommended answers + close-call scores, ADRs + a glossary via domain-modeling, /pre-mortem handoff on irreversible calls."
disable-model-invocation: true
---

Stress-test this plan or design a round at a time until we reach a shared understanding, and capture the results as docs as we go.

Model the plan as a design tree: every decision branches into the decisions that hang off it. Work in rounds. The frontier is every decision whose prerequisites are already settled: the questions you can ask now without guessing at answers you haven't heard yet.

Ask the whole frontier in one round: number each question and give your recommended answer. For genuinely close or high-stakes calls, add a weighted decision-framework score from 0.0 to 10.0 for each viable option (recommended option first, options mutually exclusive) and aim for a 9.0+ choice when realistically achievable; skip the scoring on clear-cut questions. Then wait for the user's answers before the next round.

Finding facts is your job, never the user's; don't ask the user for anything you could look up yourself. For a quick fact, use your tools inline: explore the codebase, `context7` for library or API docs, `gh` cli/api for GitHub, `opensrc` for package source, and web search for everything else. For a frontier question that needs real investigation, dispatch the `/research` skill (a background agent that returns cited findings; it can draw on `deep-researcher` for deep cited research). Don't block on it: a running exploration is an unsettled prerequisite, so only the questions downstream of it wait for the result: ask the rest of the frontier now. The decisions are the user's: put each to them and wait.

Each round the user answers reshapes the tree: settled decisions push the frontier outward and unblock questions that depended on them. Recompute the frontier and ask the next round. A question whose answer depends on another question still open in this round belongs to a later round, not this one.

As terms and decisions settle, use the `/domain-modeling` skill to capture them as you go: record resolved terminology in the glossary and each significant resolved decision as an ADR. If a settled decision is high-stakes or hard to reverse, pre-mortem that branch before moving on by offering a `/pre-mortem` on it.

The session is done when the frontier is empty: every branch of the design tree visited, nothing left silently assumed, every dispatched lookup rendezvoused before you synthesize or act, and the docs reflect what was decided. Before you hand back, offer to run a full `/pre-mortem` on the settled plan. Do not act on the plan until the user confirms you have reached a shared understanding.
