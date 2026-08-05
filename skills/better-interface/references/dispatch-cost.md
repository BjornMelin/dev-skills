# What dispatching a lane actually costs

The measurements behind Principle 4's dispatch rules. They come from two runs of one
detect lane over one 484-line component, so treat them as evidence that the tiers differ
in kind — not as a ratio to plan capacity against. Re-measure on your own surface.

**Dispatching a lane is not cheap. Measure before you assume.** A single detect lane over one
484-line component — reading the component, its four UI primitives, and the installed Radix
source — cost roughly **150k tokens and 14 minutes** at maximum reasoning effort. Six of those
is most of a million tokens. What a lane buys is *depth and a clean context*, not savings:
that run traced into `node_modules` to check what the dialog primitive actually renders, which
an inline pass sharing context with five other rule sets will not do.

So dispatch is a deliberate spend, and the mode ladder is a cost ladder:

- `quick` and `core` are cheap because they **dispatch little or nothing**, not because
  detection is inherently cheap. `quick` runs inline. `core` judges at most two domains and
  must mark the rest `Detected only` in the coverage table — never `Clear`, because an
  unjudged candidate is not the same as no candidate.
- `full` judges every domain that produced candidates and is expensive by design. Reach for it
  on a surface that matters, not as a default sweep.

**Effort buys traversal depth, not polish.** One detect lane, one 484-line component, run twice
on the same non-Claude runtime at two reasoning tiers:

| Reasoning effort | Wall clock | Tokens | Candidates |
| --- | --- | --- | --- |
| `high` | ~4 min | ~75k | 2 of 6 |
| `max` | ~14 min | ~153k | 6 of 6 |

The four the fast tier missed were not tail noise. It never entered `node_modules`, so it
missed both findings that required reading what the dialog primitive actually renders —
including a high-confidence one, that focus never returns to the trigger because the ref the
library restores through is unset. It also missed a 16×16 hit area inside a file it had
already read.

Treat that as one data point on one file, not a law: it establishes that the tiers differ in
how far they traverse, not a fixed ratio you can plan capacity against. Re-measure on your own
surface before relying on the numbers.

**Where the tier is actually selectable.** Only on a lane you dispatch to an external runtime
with an explicit effort flag. The Claude role agents in `subagents/claude/` are pinned to
`effort: high` in their definitions, deliberately: `model-routing` holds that
verification-shaped work never gets a higher tier, and two diverse high lanes beat one deeper
lane for error decorrelation. The Agent tool has no effort parameter, so there is nothing to
override at the call site either.

So the modes differ by **how many lanes run and how deep the sweep goes**, not by reasoning
tier. `full` judges every domain with candidates; `core` judges at most two. When a detect lane
runs on an external runtime, give `full` the deeper tier and `quick`/`core` the faster one, and
record which tier actually ran in **Scope and Coverage**. What you must not do is run a
shallower sweep and report it as full coverage.

Never send interface copy, visual design, motion, or naming decisions to a non-Claude model for judgment: those are taste calls. Detection of mechanical copy defects — terminology drift, inconsistent capitalization, non-verb-first labels, a placeholder restating its label — is a lint and may run anywhere.

If the host cannot run lanes, do the same passes inline in the review order above and say so in **Scope and Coverage**. All six rule sets stay resident on that path, so prefer `quick` or a narrowed scope there.
