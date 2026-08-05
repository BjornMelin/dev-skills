# Screen Readers

Visually hidden content, live regions, toasts, alt text, and SVG.

## Visually hidden content

The canonical `.sr-only` pattern hides content visually while keeping it in the accessibility tree:

```css
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
}
```

Use `1px` boxes, not `0`; some screen readers skip zero-sized elements. `white-space: nowrap` prevents words from being read as one run-together string. Never use `display: none` or `visibility: hidden` for this; both remove the content from assistive tech entirely.

`clip-path: inset(50%)` alone is the current recipe — Tailwind's own `sr-only` utility ships exactly this. The legacy `clip: rect(0 0 0 0)` is a deprecated property kept only for user agents without `clip-path`, which in practice means IE; add it back only when an explicit legacy-support requirement says so.

This static recipe is for content that is hidden and stays hidden. Anything focusable — a skip link, a control revealed on focus — must become visible when focused, so guard it: `.visually-hidden:not(:focus):not(:active):not(:focus-within)`. A permanently clipped focusable element is a keyboard trap in all but name.

Tailwind ships this as `sr-only`. For skip links, add a focus variant that un-hides the element (`focus:not-sr-only`, or override the positioning on `:focus`).

Use it for context that sighted users get visually: `<span class="sr-only">Opens in new tab</span>`, table caption text, or the label of an icon-only control when `aria-label` isn't an option.

## Choosing how to announce a change

Work down this list and stop at the first match:

1. **Focus moves there anyway** (opened modal, first invalid field on submit): nothing extra needed; the focus move is the announcement.
2. **Tied to a specific control** (field error, character count): `aria-describedby` on the control; it's announced with the field.
3. **Non-urgent, not tied to a control** (toast, "Saved", result count, loading state): polite live region / `role="status"`.
4. **Urgent and not tied to a control** (form-level failure, session expiring): `role="alert"`.

## Live regions

Live regions announce content that changes without a page load: toasts, validation, search-result counts, loading states.

| Mechanism | Politeness | Use for |
| --- | --- | --- |
| `role="status"` (= `aria-live="polite"` + `aria-atomic="true"`) | Waits for a pause | Toasts, "Saved", result counts, loading updates |
| `role="alert"` (= `aria-live="assertive"` + `aria-atomic="true"`) | Interrupts immediately | Errors and urgent problems only |

Rules for reliable announcements:

- For repeated polite updates, keep a stable empty region in the DOM before changing its text. Inserting a new polite region together with its content is inconsistently announced.
- Dynamically inserted `role="alert"` content is commonly announced, but behavior varies; use it only for urgent errors not tied to a control and test the target browser/screen-reader combinations.
- Default to polite. Overusing `assertive` is the most common live-region mistake; it interrupts whatever the user was reading.
- Keep messages short and self-contained; `aria-atomic="true"` re-reads the whole region on change.
- Don't move focus to a toast; announce it and leave focus where the user is working. Give toasts a generous timeout or a dismiss button, and never put the only path to an action inside an auto-dismissing toast.

```tsx
// Region rendered from the start, message injected later
<div role="status" className="sr-only">
  {statusMessage}
</div>
```

For loading states: set `aria-busy="true"` on the updating region, announce "Loading…" politely, then announce the outcome ("Loaded, 12 results").

## aria-hidden

`aria-hidden="true"` removes an element and its whole subtree from assistive tech. Use it for decorative icons and content duplicated for visual effect. Never put it on (or above) a focusable element; that creates stops you can Tab to that don't exist for a screen reader. If you hide something interactive, also remove it from the tab order.

## Alt text

Choose by purpose, not by what the image looks like:

| Purpose | Alt | Example |
| --- | --- | --- |
| Decorative, or redundant with adjacent text | `alt=""` (empty, but present) | Logo next to the company name in text |
| Informative | Describe the meaning it adds | `alt="Ticket QR code"` |
| Functional (image is the link/button) | Describe the action or destination | Search icon → `alt="Search"`, not `alt="magnifying glass"` |
| Image of text | The exact text (better: use real text) | `alt="50% off everything"` |
| Complex (chart, diagram) | Short summary in `alt`, full data as a table or text nearby | `alt="Revenue by quarter, described below"` |

A missing `alt` attribute is worse than an empty one: screen readers fall back to reading the file name.

## SVG

- Decorative SVG: `aria-hidden="true"`, and make sure neither the `<svg>` nor any descendant is focusable — `aria-hidden` removes the subtree from the accessibility tree but does not stop Tab from reaching a focusable child, which produces a focus stop that announces nothing. `focusable="false"` existed for IE and EdgeHTML tabbing; omit it for modern targets and keep it only where legacy support is an explicit requirement.
- Meaningful inline SVG: `role="img"` plus `aria-label="…"` (or a `<title>` as the first child referenced by `aria-labelledby`).
- Simple cases: `<img src="icon.svg" alt="…">` is the most reliable delivery.

```tsx
// Decorative icon inside a labeled button
<button aria-label="Close">
  <svg aria-hidden="true" focusable="false">…</svg>
</button>

// Standalone meaningful icon
<svg role="img" aria-label="Verified account">…</svg>
```

## Video and audio

Prerecorded video needs captions; provide transcripts for audio. Never autoplay with sound, and always render controls.
