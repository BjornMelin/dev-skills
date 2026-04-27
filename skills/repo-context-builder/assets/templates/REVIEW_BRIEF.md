# REVIEW_BRIEF

<!--
Fill this document from the current user request plus repository evidence.
If the user did not specify a task, infer the highest-value next task and state that it was inferred.
Replace every placeholder.
-->

## 1. Task identity

- **Task / feature / problem:** 
- **Desired outcome:** 
- **Status:** new | in progress | refresh | inferred from repo
- **Priority / urgency:** 
- **Repo / package / area:** 
- **Related issue / PR / branch:** 
- **Prepared on:** 
- **Prepared by:** 

## 2. User request or inferred brief

> Paste or paraphrase the request concisely. If inferred, say so explicitly.

## 3. Scope definition

### In scope

- 
- 

### Out of scope

- 
- 

### Assumptions

- 
- 

### Constraints

- 
- 

### Non-goals

- 
- 

## 4. Relevant repository context

### Key components and systems involved

- 
- 

### Relevant files and directories

| Path | Why it matters |
| --- | --- |
|  |  |

### Existing patterns to follow

Describe the implementation patterns, abstractions, conventions, or workflows already present in the repo that should be followed for this task.

## 5. Current-state findings

| Finding | Evidence | Impact |
| --- | --- | --- |
|  |  |  |

## 6. Gaps, risks, and unknowns

| Item | Severity | Why it matters | How to resolve |
| --- | --- | --- | --- |
|  |  |  |  |

## 7. Recommended plan

### 7.1 Implementation or review sequence

1. 
2. 
3. 

### 7.2 Design decisions

| Decision area | Recommendation | Alternatives considered | Rationale |
| --- | --- | --- | --- |
|  |  |  |  |

### 7.3 Files likely to change

| Path | Expected change |
| --- | --- |
|  |  |

## 8. Verification plan

### Automated checks

~~~bash
# exact commands from repo, if found
~~~

### Manual / exploratory checks

- 
- 

### Rollback or safety checks

- 
- 

## 9. Deliverables expected from the next working session

- code changes:
- tests:
- docs:
- infra / config:
- PR / handoff notes:

## 10. Ready-to-paste working prompt for the next session

~~~text
Use the attached REPO_CONTEXT.md and REVIEW_BRIEF.md as the source of truth for this repository and task. Re-read the relevant files referenced in both documents before changing code. Then execute the plan in REVIEW_BRIEF.md, keep changes scoped, follow existing repo conventions, update docs/tests as needed, and verify the work with the exact commands listed in the brief before finishing.
~~~

## 11. Handoff notes

Capture anything a future agent or engineer should know before starting.

- 
- 
