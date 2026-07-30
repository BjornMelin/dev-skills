# React Surface Refactors

Use this file when the task is primarily component architecture and state flow.

Focus areas:

- boolean prop cleanup
- compound component opportunities
- context vs prop threading
- derived state and effect cleanup
- transient values in refs
- event and async handling patterns

Default policy:

- use `vercel-composition-patterns` first for API shape
- use `vercel-react-best-practices` second for runtime/perf guidance
- verify in browser if the refactor changes user-visible behavior
