# Pre-Mortem Framework

Systematically find the weaknesses in a plan before reality does.

## The core idea

Most plans fail for predictable reasons. Not bad luck, bad assumptions: overestimated demand or adoption, underestimated complexity, dependencies nobody questioned, timing that made sense on paper but not in the real world.

The pre-mortem technique: **imagine it is 12 months from now and this plan failed. Now work backwards. Why?**

## When to run one

- Before committing significant time, money, or reputation to a plan
- Before a hard-to-reverse decision (a public API, a schema migration, a framework choice, an architecture, a hire)
- When you are only hearing positive feedback
- When the plan needs multiple external dependencies to align
- When there is pressure to move fast and "figure it out later"
- When you feel excited about the plan (excitement is a signal to scrutinize harder)

## The framework

### Step 1: Extract core assumptions
Surface everything the plan needs to be true. For each part of the plan, ask: what has to be true for this to work? What are we assuming about the users, the environment, our own execution, and everything we depend on?

Common categories (pick the ones that fit the plan):
- **Technical**: the approach scales, the API/library behaves as documented, performance holds, the data model fits, no undiscovered edge cases
- **Execution**: team capacity and velocity, no blocking hires or unknowns, estimates are real not theoretical
- **User/customer**: they have the problem, know they have it, and will adopt or pay for the solution
- **Dependency**: a partner/service/upstream delivers on time, an API or contract will not change, a platform or regulation will not shift
- **Competitive/market**: size and growth, incumbents' response, the moat holds
- **Financial**: cost, timing, and unit economics

### Step 2: Rate each assumption
Rate every assumption on two axes.

**Confidence it is true:** High (verified with data/evidence); Medium (directionally right, unvalidated); Low (plausible, untested); Unknown (we don't know).

**Impact if wrong:** Critical (plan fails entirely); High (major delay/cost/rework); Medium (significant rework); Low (manageable adjustment).

### Step 3: Map vulnerabilities
`Vulnerability = Low/Unknown confidence x Critical/High impact`. These are the bets you are making. The question is whether you are making them consciously.

### Step 4: Find the dependency chain
Many plans fail not because a single assumption is wrong, but because several have to be right at once. Map it: does B depend on A being true first? If the first thing goes wrong, how many downstream things break? What is on the critical path with zero slack?

### Step 5: Test reversibility
For each critical vulnerability: if this turns out wrong at month 3, what do you do? Can you pivot or cut scope? Is money already spent or a commitment already made? The less reversible, the more rigorously you validate before committing.

## Output: Challenge Report

```
CORE ASSUMPTIONS (extracted)
1. [Assumption] (confidence: [H/M/L/?]; impact if wrong: [Critical/High/Medium/Low])
2. ...

VULNERABILITY MAP
Critical risks (act before proceeding):
- [#N] [Assumption]: why it might be wrong, and what breaks if it is
High risks (validate before scaling):
- ...

DEPENDENCY CHAIN
[A] -> depends on -> [B] -> which enables -> [C]
Weakest link: [X]. If this breaks, [Y] and [Z] also fail

REVERSIBILITY ASSESSMENT
- Reversible bets: [list]
- Irreversible commitments: [list, treat with extreme care]

KILL SWITCHES
What would have to be true at [30/60/90 days] to continue vs. kill/pivot?
- Continue if: ...
- Kill/pivot if: ...

HARDENING ACTIONS
1. [Specific validation to do before proceeding]
2. [Alternative approach to consider]
3. [Contingency to build into the plan]
```

The template is the definition of done: the pre-mortem is complete only when every section is filled. Before you finalize, self-check the report against the template: an empty KILL SWITCHES or HARDENING ACTIONS section means the analysis isn't finished, not that there is nothing to say. Never stop at the vulnerability map.

## Patterns by plan type

### Technical design / architecture
- Where does this break under 10x the data, traffic, or concurrency you expect?
- What happens if the anchor component takes 3x longer than estimated?
- Which decision here is a one-way door (public API, schema, framework, data format)?
- What are we assuming the dependency (library, service, platform) will keep doing that it has not promised?
- Who owns the decision when two requirements conflict?

### Product roadmap
- Are we building what users will adopt, or what they said they wanted?
- Does the velocity estimate reflect real team capacity, not theoretical?
- What happens if the anchor feature slips or its usage is a fraction of forecast?

### Go-to-market / org / fundraising
- What is the actual conversion/adoption rate, not the hoped-for one?
- What is the fallback if the key partner, hire, or investor falls through?
- What breaks if the timeline is 2x, or you land at the low end of the target?

## The hardest questions
These are the ones people skip:
- "What's the bear case, not the base case?"
- "If a team we don't trust ran this exact plan, would it work?"
- "What are we not saying out loud because it's uncomfortable?"
- "Who has incentives to make this plan sound better than it is?"
- "What would an enemy of this plan attack first?"
