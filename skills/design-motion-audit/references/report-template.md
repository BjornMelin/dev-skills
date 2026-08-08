# Motion Audit Report Template

Use this shape for a design-motion audit result. Keep it scannable; put the
prioritized punch list first, and route every fix to the skill that owns the stack.

```markdown
# Motion Audit — <target> (<date>)

## Verdict
<pass | pass-with-risks | block> — <one sentence>.

## Stack & scope
- Stacks detected: <R3F / three.js / Reanimated / GSAP / CSS / Motion>
- Files/routes/scenes reviewed: <n> (<how scoped>)
- Other static/runtime tools: <r3f art-direction audit / web-three-r3f audit / playwright-cli /
  MotionScore, or "static only; runtime unverified">

## Optional analyzers
| Tool | Exact version run (from `doctor`) | Categories run |
| --- | --- | --- |
| `motion-token-audit` | [exact installed version or not run] | [all, scan CSV, or not run] |
| `expo-motion-audit` | [exact installed version or not run] | [all, scan CSV, or not run] |
| `gsap-audit` | [exact installed version or not run] | [all, scan CSV, or not run] |

## Prioritized punch list
| # | Severity | Area | File:line | Finding | Fix | Route to |
|---|---|---|---|---|---|---|
| 1 | high | perf | src/Hero.tsx:42 | setState in useFrame | move to a ref | web-three-r3f |
| 2 | med | tokens | src/theme.css:12 | hardcoded 200ms == token `short` (drift) | use `var(--motion-duration-short)` | this skill (tokens) |
| … |

## Dimension coverage
- **Design tokens** — <tokenization %/stack; drift vs orphan count from motion-token-audit>
- **R3F / three.js** — <lifecycle (web-three-r3f) + art-direction (this repo's audit)>
- **Reanimated / Expo** — <expo-motion-audit findings; New-Arch/worklets>
- **Interaction physicality** — <velocity release, interruptibility>
- **Performance** — <frame budget; runtime FPS/MotionScore grade or UNVERIFIED>
- **Reduced motion** — <coverage; each camera/parallax/loop/bounce has a branch?>
- **Accessibility & readability** — <text legibility during motion>
- **Missing hallmark opportunities** — <where a signature motion would add value>

## Runtime proof
<what was driven, artifacts (screenshot/video paths), FPS/grade — or the exact
runtime claim left unverified and why. See references/runtime-verification.md.>

## Residual risks
- <risk> — <mitigation / follow-up>
```

Score against `references/quality-gates.md`. Treat every static-tool finding as a
lead — verify against the real code before reporting it. Hand each punch-list item
to the owning skill (`expo-motion`, `web-three-r3f` for R3F setup/correctness,
`r3f-scene-polish` for look-dev, or `gsap`) for the fix.
Token and system-level items are handled in this skill.
