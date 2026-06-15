# Output Contract

The wrapper asks Kimi to return exactly one JSON object:

```json
{
  "mode": "advise|audit|redesign|component|screenshot-review|compare",
  "summary": "one or two sentences",
  "approach": "implementation approach",
  "ranked_issues": [
    {
      "severity": "high|medium|low",
      "area": "layout|hierarchy|component|styling|responsive|accessibility|motion|copy",
      "evidence": "specific observation from files or images",
      "recommendation": "what Codex should do"
    }
  ],
  "design_direction": {
    "principles": ["short principles for the UI direction"],
    "tokens": ["spacing, color, type, radius, shadow, or density guidance"],
    "interaction_model": "how interaction states and feedback should behave"
  },
  "files": [
    {"path": "relative/path", "reason": "why this file matters"}
  ],
  "image_findings": [
    {"path": "relative/or/absolute/image", "finding": "visual finding", "recommendation": "actionable fix"}
  ],
  "patch_suggestions": [
    {
      "path": "relative/path",
      "intent": "what to change",
      "content": "code, diff, or precise replacement guidance"
    }
  ],
  "component_notes": ["component structure, states, props, composition"],
  "styling_notes": ["layout, spacing, typography, responsive, theme notes"],
  "accessibility_notes": ["keyboard, semantics, focus, contrast, labels"],
  "responsive_notes": ["breakpoints, wrapping, density, touch target notes"],
  "motion_notes": ["microinteraction, transition, reduced-motion notes"],
  "verification": ["commands or visual checks Codex should run"],
  "acceptance_criteria": ["observable criteria for the improved UI"],
  "risks": ["assumptions, uncertainty, or rejected alternatives"],
  "rejected_suggestions": ["ideas intentionally not recommended and why"]
}
```

Codex should:

- prefer `patch_suggestions` for concrete code;
- adapt code to local naming, imports, design tokens, and component contracts;
- use `ranked_issues` to decide implementation order instead of blindly
  applying every suggestion;
- use `image_findings` only when Kimi actually inspected screenshots or visual
  references;
- ignore suggestions that require backend/data/security changes outside the UI
  scope unless separately validated;
- run repo-native verification and rendered UI checks after applying changes.

If parsing fails, the wrapper still emits `raw_response`. Use it only as human
advice and do not paste it blindly into source.
