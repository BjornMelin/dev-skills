# Routing Matrix

Use this file when the right Next/React chain is not obvious.

| Task Shape | Default Path | Why |
| --- | --- | --- |
| Route/page bug | `next-best-practices` -> `browser-workbench` | Route correctness plus runtime evidence |
| App-shell/layout issue | `next-best-practices` -> `browser-workbench` | Layout and RSC boundary awareness |
| React component refactor | `vercel-composition-patterns` -> `vercel-react-best-practices` | Better API shape and runtime discipline |
| Visual uplift or theming pass | `frontend-design` -> `browser-workbench` | Design plus verification |
| Post-change audit | `react-doctor` | Fast correctness/perf sweep |

If a task needs both code-quality refactor and visible UX polish, do the refactor path first, then the UI polish path.
