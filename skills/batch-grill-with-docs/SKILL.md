---
name: batch-grill-with-docs
description: "A relentless batch interview that sharpens a plan or design a round at a time: it asks the whole dependency-settled frontier each round with recommended answers (and weighted 0.0 to 10.0 scores on close calls), then captures resolved terms and decisions as domain-modeling docs (ADRs and a glossary) as it goes. Use to stress-test a plan round by round and produce its docs. Opt-in only."
disable-model-invocation: true
---

Stress-test this plan or design a round at a time until we reach a shared understanding, and capture the results as docs as we go.

Model the plan as a design tree: every decision branches into the decisions that hang off it. Work in rounds. The frontier is every decision whose prerequisites are already settled: the questions you can ask now without guessing at answers you haven't heard yet.

Ask the whole frontier in one round: number each question and give your recommended answer. For genuinely close or high-stakes calls, add a weighted decision-framework score from 0.0 to 10.0 for each viable option (recommended option first, options mutually exclusive) and aim for a 9.0+ choice when realistically achievable; skip the scoring on clear-cut questions. Then wait for the user's answers before the next round.

Finding facts is your job, never the user's. When a frontier question needs a fact from the environment (filesystem, tools, web, GitHub, package source), dispatch a sub-agent or use your tools to find it: don't ask the user for anything you could look up yourself. Don't block on it: a running exploration is an unsettled prerequisite, so only the questions downstream of it wait for the result: ask the rest of the frontier now. The decisions are the user's: put each to them and wait.

Each round the user answers reshapes the tree: settled decisions push the frontier outward and unblock questions that depended on them. Recompute the frontier and ask the next round. A question whose answer depends on another question still open in this round belongs to a later round, not this one.

As terms and decisions settle, use the `/domain-modeling` skill to capture them as you go: record resolved terminology in the glossary and each significant resolved decision as an ADR.

The session is done when the frontier is empty: every branch of the design tree visited, nothing left silently assumed, and the docs reflect what was decided. Do not act on the plan until the user confirms you have reached a shared understanding.
