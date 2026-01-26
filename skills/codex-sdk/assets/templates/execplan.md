# <Short, action-oriented description>

This ExecPlan is a living document. The sections **Progress**, **Surprises & Discoveries**, **Decision Log**, and **Outcomes & Retrospective** must be kept up to date as work proceeds.

This ExecPlan must be maintained in accordance with `.agent/PLANS.md`.

## Purpose / Big Picture

Explain, in a few sentences, what someone gains after this change and how they can see it working.

## Progress

Use checkboxes and timestamps. Every stopping point must be recorded here, even if it requires splitting a task (“done” vs “remaining”).

- [ ] (YYYY-MM-DD HH:MMZ) …

## Surprises & Discoveries

Document unexpected behavior, bugs, performance tradeoffs, or “this assumption was wrong” discoveries. Include short evidence (test output is ideal).

## Decision Log

- Decision: …
  Rationale: …
  Date/Author: …

## Outcomes & Retrospective

Summarize what was achieved, what remains, and the main lessons learned.

## Context and Orientation

Assume the reader knows nothing about this repo. Define any non-obvious term you use and point to the concrete file(s)/command(s) where it appears.

Include (as relevant):

- key repo paths and what they are for
- exact commands to run (include working directory assumptions)
- constraints (permissions, sandbox, network)
- IDs needed to resume work (e.g. `threadId`, `SESSION_ID`, JSONL file paths, SQLite DB path)

## Plan of Work

Describe the sequence of edits and additions in prose. Name the exact files and functions/modules involved.

## Concrete Steps

List the exact commands to run and what outputs should look like.

## Validation and Acceptance

Describe how to prove the behavior works (tests, CLI invocations, or other observable checks). Phrase acceptance as behavior with specific inputs/outputs.

## Idempotence and Recovery

Describe how steps can be safely re-run and how to recover from partial failures.

## Artifacts and Notes

Include short, focused transcripts and evidence that prove progress and correctness.

## Interfaces and Dependencies

Be prescriptive: name the libraries, modules, and interfaces that must exist when the plan is complete.
