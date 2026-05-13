---
description: Decision matrix for choosing whether each dependency should be upgraded, replaced, or removed during repo modernization.
---

# Dependency Decision Matrix

Use this reference during dependency triage. The goal is not to update
everything blindly. The goal is to choose the right action for each package:

- upgrade
- replace
- remove
- defer with explicit reason

## Core Rule

Do not assume every outdated dependency should be upgraded in place.

For each dependency, decide whether the repo is better off if it is:
1. upgraded
2. replaced with another dependency
3. removed entirely

## Preferred Outcomes

From strongest to weakest:

1. remove the dependency and delete related code
2. replace it with an already-present or better-maintained dependency
3. upgrade in place to a current supported version
4. defer only with explicit evidence and a concrete blocker

## Evaluation Dimensions

Evaluate each dependency along these dimensions:

### 1. Security

- vulnerable now
- recently patched
- recurring advisory history
- risky transitive tree

Default:
- vulnerable packages should not remain unless blocked by a concrete external
  dependency

### 2. Maintenance Health

- latest release recency
- active upstream maintainers
- release cadence
- open issue/PR signals
- official migration/support posture

Default:
- stagnant or abandoned packages should be replacement/removal candidates

### 3. API Quality and Modern Fit

- does the package still match the repo’s framework/runtime direction?
- does a newer major version offer a clearly better canonical API?
- does the repo currently carry glue code because the package is weak or old?

Default:
- prefer packages and versions that reduce repo-local glue code

### 4. Native Replacement Opportunity

- can the framework/runtime now do this natively?
- can an upgraded existing dependency replace this extra package?
- is custom repo code now obsolete because the dependency gained native support?

Default:
- if native support exists and reduces code, remove the extra dependency

### 5. Blast Radius

- how many packages/apps import it?
- is it internal-only or public-facing?
- is it tied to persisted data or public contract behavior?

Default:
- high blast radius means research and staging matter more, not that the package
  should be preserved automatically

### 6. Codebase Simplification Potential

- how much code can be deleted if this package is upgraded/replaced/removed?
- does it collapse wrappers, adapters, or duplicated implementations?
- does it reduce conceptual surface area?

Default:
- favor the option that materially reduces total code and maintenance burden

## Action Rules

### Choose Upgrade In Place When

- the package is maintained
- the latest supported version is viable
- migration cost is reasonable
- the package still fits the repo well
- upgrading lets you delete custom glue or deprecated usage

### Choose Replace When

- the package is vulnerable and upstream is slow or stagnant
- another dependency or framework-native feature is now clearly better
- the repo has accumulated wrappers/workarounds around the package
- the replacement reduces long-term code and cognitive load

### Choose Remove When

- the dependency’s behavior is no longer needed
- framework/runtime features supersede it
- another existing dependency already provides the needed capability
- custom code added around it can be deleted without loss of required behavior

### Choose Defer Only When

- a real compatibility boundary exists
- an upstream blocker is concrete and current
- the migration would break required external behavior without a safe path today

If deferred:
- name the blocker
- name the affected scope
- name the exact package/version issue
- state the safe holding position

## Hard-Cut Implications

When a package is upgraded, replaced, or removed:
- delete compatibility branches for the old shape
- remove adapters and translation layers
- update all consumers to the canonical current path
- remove tests that only preserve old internal behavior

Do not preserve a package-specific compatibility layer merely because the repo
used to depend on the old API.

## Minimal Triage Template

For each important package, produce:

- package name
- current version
- latest version
- latest safe version
- action: upgrade / replace / remove / defer
- why
- affected packages/files
- expected deletions
- migration risk

## Scoring Shortcut

If a package decision is ambiguous, bias using this order:

1. security risk
2. code deletion opportunity
3. maintenance health
4. framework-native replacement fit
5. migration cost

This keeps the modernization aligned with hard-cut and reducing-entropy goals.

## Anti-Patterns

Reject these:

- upgrading a package while leaving old and new APIs side by side
- preserving wrappers that newer dependencies make unnecessary
- keeping overlapping libraries “just in case”
- deferring a vulnerable or obsolete package without concrete evidence
- upgrading a package but not refactoring the code to use the improved native
  API

## Final Rule

A dependency decision is only complete when both are true:

1. the package graph is improved
2. the codebase shape is improved

If only the lockfile changed, the job is not done.
