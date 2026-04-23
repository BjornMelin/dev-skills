---
name: grill-me
description: Interview the user relentlessly about a plan or design until reaching shared understanding, resolving each branch of the decision tree. Use when user wants to stress-test a plan, get grilled on their design, or mentions "grill me".
---

Interview me relentlessly about every aspect of this plan until we reach a shared understanding. Walk down each branch of the design tree, resolving dependencies between decisions one-by-one and applying your deep reasoning and domain expertise. For each question, provide your recommended answer and include custom weighted decision framework scores from 0.0 to 10.0 for all viable options, and target 9.0+ decisions when realistically achievable.

Ask questions one at a time using `functions.request_user_input`. Put the recommended option first, and keep options mutually exclusive. Batch multiple non-dependent questions into a single `functions.request_user_input` whenever safe.

If a question can be answered by exploring the codebase, explore the codebase instead.

If a question can be answered by researching the web, GitHub, or package source code, use web search tools, context7, gh cli/api, or opensrc skills.
