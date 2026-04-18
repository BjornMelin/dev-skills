---
name: "commit"
description: "Stage + commit changes in semantic + reviewablegroups. Use when user wants to commit changes, organize commits, or clean up a branch before pushing."
---

1. `git status --short` first; scope to files from current task or explicit user request.
2. Leave unrelated dirty files alone unless user explicitly asks for a whole-tree commit.
3. Mixed tree: inspect diffs before staging; keep staged set one semantic change.
4. Stage by purpose, not `git add .`, unless whole task one coherent change.
5. Default one scoped conventional commit when task cohesive. Split only for clearly separate semantic groups. Optimize for reviewability.
6. Multiple commits: prefer order `feat`, `fix`, `test`, `docs`, `refactor`, `chore`.
7. Short scoped conventional messages; match what changed.
8. Each commit minimal, reviewable, one change type.
9. Between groups: `git status -sb`; confirm next commit has only intended files.
10. Ask before staging unrelated user changes, unfamiliar generated churn, or ambiguous dirty tree that makes ownership ambiguous.
