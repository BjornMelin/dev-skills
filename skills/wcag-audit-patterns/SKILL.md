---
name: wcag-audit-patterns
description: "Conduct WCAG 2.2 accessibility audits with automated testing, manual verification, and remediation guidance. Use when auditing websites for accessibility, fixing WCAG violations, or implementing accessible design patterns."
license: MIT
metadata:
  version: "1.0.0"
---

# WCAG Audit Patterns

Comprehensive guide to auditing web content against WCAG 2.2 guidelines with actionable remediation strategies.

## When to Use This Skill

- Conducting accessibility audits
- Fixing WCAG violations
- Implementing accessible components
- Preparing for accessibility lawsuits
- Meeting ADA/Section 508 requirements
- Achieving VPAT compliance

## Core Concepts

### 1. WCAG Conformance Levels

| Level   | Description            | Required For      |
| ------- | ---------------------- | ----------------- |
| **A**   | Minimum accessibility  | Legal baseline    |
| **AA**  | Standard conformance   | Most regulations  |
| **AAA** | Enhanced accessibility | Specialized needs |

### 2. POUR Principles

```
Perceivable:  Can users perceive the content?
Operable:     Can users operate the interface?
Understandable: Can users understand the content?
Robust:       Does it work with assistive tech?
```

### 3. Common Violations by Impact

```
Critical (Blockers):
├── Missing alt text for functional images
├── No keyboard access to interactive elements
├── Missing form labels
└── Auto-playing media without controls

Serious:
├── Insufficient color contrast
├── Missing skip links
├── Inaccessible custom widgets
└── Missing page titles

Moderate:
├── Missing language attribute
├── Unclear link text
├── Missing landmarks
└── Improper heading hierarchy
```

## Audit Checklist

Covers every WCAG 2.2 Level A and AA success criterion. The four AAA criteria
(2.4.12, 2.4.13, 3.3.9, and the rest of the AAA set) are deliberately out of scope —
audit them against the specification directly when a product targets AAA.

4.1.1 Parsing is obsolete in WCAG 2.2 and is retained below only as a good-practice note.

Where `better-accessibility` is named, that skill owns the prescriptive remedy; this
checklist owns the conformance question. When it is unavailable, apply the checklist item
against the specification and record that the remedy detail went uncovered.

### Perceivable (Principle 1)

````markdown
## 1.1 Text Alternatives

### 1.1.1 Non-text Content (Level A)

- [ ] All images have alt text
- [ ] Decorative images have alt=""
- [ ] Complex images have long descriptions
- [ ] Icons with meaning have accessible names
- [ ] CAPTCHAs have alternatives

Check:

```html
<!-- Good -->
<img src="chart.png" alt="Sales increased 25% from Q1 to Q2" />
<img src="decorative-line.png" alt="" />

<!-- Bad -->
<img src="chart.png" />
<img src="decorative-line.png" alt="decorative line" />
```
````

## 1.2 Time-based Media

### 1.2.1 Audio-only and Video-only (Level A)

- [ ] Audio has text transcript
- [ ] Video has audio description or transcript

### 1.2.2 Captions (Level A)

- [ ] All video has synchronized captions
- [ ] Captions are accurate and complete
- [ ] Speaker identification included

### 1.2.3 Audio Description (Level A)

- [ ] Video has audio description for visual content

### 1.2.4 Captions (Live) (Level AA)

- [ ] Live audio content has real-time captions

### 1.2.5 Audio Description (Prerecorded) (Level AA)

- [ ] Prerecorded video has a full audio description track

## 1.3 Adaptable

### 1.3.1 Info and Relationships (Level A)

- [ ] Headings use proper tags (h1-h6)
- [ ] Lists use ul/ol/dl
- [ ] Tables have headers
- [ ] Form inputs have labels
- [ ] ARIA landmarks present

Check:

```html
<!-- Heading hierarchy -->
<h1>Page Title</h1>
<h2>Section</h2>
<h3>Subsection</h3>
<h2>Another Section</h2>

<!-- Table headers -->
<table>
  <thead>
    <tr>
      <th scope="col">Name</th>
      <th scope="col">Price</th>
    </tr>
  </thead>
</table>
```

### 1.3.2 Meaningful Sequence (Level A)

- [ ] Reading order is logical
- [ ] CSS positioning doesn't break order
- [ ] Focus order matches visual order

### 1.3.3 Sensory Characteristics (Level A)

- [ ] Instructions don't rely on shape/color alone
- [ ] "Click the red button" → "Click Submit (red button)"

### 1.3.4 Orientation (Level AA)

- [ ] Content works in both portrait and landscape
- [ ] Orientation locked only where essential

### 1.3.5 Identify Input Purpose (Level AA)

- [ ] Inputs collecting user data carry the right `autocomplete` token

```html
<input name="email" type="email" autocomplete="email" />
<input name="tel" type="tel" autocomplete="tel" />
```

## 1.4 Distinguishable

### 1.4.1 Use of Color (Level A)

- [ ] Color is not only means of conveying info
- [ ] Links distinguishable without color
- [ ] Error states not color-only

### 1.4.2 Audio Control (Level A)

- [ ] Audio playing over 3 seconds can be paused or stopped
- [ ] Volume control independent of system volume

### 1.4.3 Contrast (Minimum) (Level AA)

- [ ] Text: 4.5:1 contrast ratio
- [ ] Large text (18pt+): 3:1 ratio
- [ ] UI components: 3:1 ratio

Tools: WebAIM Contrast Checker, axe DevTools

### 1.4.4 Resize Text (Level AA)

- [ ] Text resizes to 200% without loss
- [ ] No horizontal scrolling at 320px
- [ ] Content reflows properly

### 1.4.5 Images of Text (Level AA)

- [ ] Real text used instead of images of text
- [ ] Exceptions are logos and essential presentation only

### 1.4.10 Reflow (Level AA)

- [ ] Content reflows at 400% zoom
- [ ] No two-dimensional scrolling
- [ ] All content accessible at 320px width

### 1.4.11 Non-text Contrast (Level AA)

- [ ] UI components have 3:1 contrast
- [ ] Focus indicators visible
- [ ] Graphical objects distinguishable

### 1.4.12 Text Spacing (Level AA)

- [ ] No content loss with increased spacing
- [ ] Line height 1.5x font size
- [ ] Paragraph spacing 2x font size
- [ ] Letter spacing 0.12x font size
- [ ] Word spacing 0.16x font size

### 1.4.13 Content on Hover or Focus (Level AA)

- [ ] Hover/focus content is dismissible without moving the pointer
- [ ] Pointer can move onto the revealed content without it disappearing
- [ ] Content stays visible until dismissed, invalid, or no longer relevant

Tooltips and hover menus are the usual failures. `better-accessibility` owns the rule.

````

### Operable (Principle 2)

```markdown
## 2.1 Keyboard Accessible

### 2.1.1 Keyboard (Level A)
- [ ] All functionality keyboard accessible
- [ ] No keyboard traps
- [ ] Tab order is logical
- [ ] Custom widgets are keyboard operable

Check:
```javascript
// Custom button must be keyboard accessible
<div role="button" tabindex="0"
     onkeydown="if(event.key === 'Enter' || event.key === ' ') activate()">
````

### 2.1.2 No Keyboard Trap (Level A)

- [ ] Focus can move away from all components
- [ ] Modal dialogs trap focus correctly
- [ ] Focus returns after modal closes

### 2.1.4 Character Key Shortcuts (Level A)

- [ ] Single-character shortcuts can be turned off, remapped, or are focus-scoped

A bare `/` or `s` hotkey fires while a speech-input user is dictating.

## 2.2 Enough Time

### 2.2.1 Timing Adjustable (Level A)

- [ ] Session timeouts can be extended
- [ ] User warned before timeout
- [ ] Option to disable auto-refresh

### 2.2.2 Pause, Stop, Hide (Level A)

- [ ] Moving content can be paused
- [ ] Auto-updating content can be paused
- [ ] Animations respect prefers-reduced-motion

```css
/* Reduce motion, do not remove feedback. A blanket kill also removes the state
   change the animation was communicating, leaving the user with no signal at all. */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    /* Collapse movement to a crossfade rather than deleting the transition. */
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

Replace slides, scales, and parallax with an opacity crossfade; keep loading and progress
indicators, which convey state rather than decoration. `better-accessibility` owns this rule.

## 2.3 Seizures and Physical Reactions

### 2.3.1 Three Flashes (Level A)

- [ ] No content flashes more than 3 times/second
- [ ] Flashing area is small (<25% viewport)

## 2.4 Navigable

### 2.4.1 Bypass Blocks (Level A)

- [ ] Skip to main content link present
- [ ] Landmark regions defined
- [ ] Proper heading structure

```html
<a href="#main" class="skip-link">Skip to main content</a>
<main id="main">...</main>
```

### 2.4.2 Page Titled (Level A)

- [ ] Unique, descriptive page titles
- [ ] Title reflects page content

### 2.4.3 Focus Order (Level A)

- [ ] Focus order matches visual order
- [ ] tabindex used correctly

### 2.4.4 Link Purpose (In Context) (Level A)

- [ ] Links make sense out of context
- [ ] No "click here" or "read more" alone

```html
<!-- Bad -->
<a href="report.pdf">Click here</a>

<!-- Good -->
<a href="report.pdf">Download Q4 Sales Report (PDF)</a>
```

### 2.4.5 Multiple Ways (Level AA)

- [ ] More than one route to each page: nav, search, sitemap, or index
- [ ] Exception is a page that is a step in a process

### 2.4.6 Headings and Labels (Level AA)

- [ ] Headings describe content
- [ ] Labels describe purpose

### 2.4.7 Focus Visible (Level AA)

- [ ] Focus indicator visible on all elements
- [ ] Custom focus styles meet contrast

```css
/* :focus-visible, not :focus -- bare :focus also rings on mouse click. */
:focus-visible {
  outline: 3px solid #005fcc;
  outline-offset: 2px;
}
```

### 2.4.11 Focus Not Obscured (Minimum) (Level AA) - WCAG 2.2

- [ ] Focused element not *entirely* hidden by author-created content
- [ ] Sticky headers, fixed footers, and cookie banners don't obscure focus

Partial obscuring passes at AA; full obscuring fails. `better-accessibility` owns the
`scroll-padding-block` remedy.

## 2.5 Input Modalities

### 2.5.1 Pointer Gestures (Level A)

- [ ] Multipoint and path-based gestures have a single-pointer alternative
- [ ] Pinch-zoom, swipe, and drag-along-a-path are not the only path

### 2.5.2 Pointer Cancellation (Level A)

- [ ] Action fires on up-event, not down-event
- [ ] Action can be aborted by moving away before release

### 2.5.3 Label in Name (Level A)

- [ ] Accessible name contains the visible label text, in the same order
- [ ] `aria-label` does not contradict the visible text a speech user will say

```html
<!-- Bad: speech user says "Search", nothing matches -->
<button aria-label="Submit query">Search</button>

<!-- Good -->
<button aria-label="Search products">Search</button>
```

### 2.5.4 Motion Actuation (Level A)

- [ ] Shake, tilt, and other device-motion triggers have a UI equivalent
- [ ] Motion response can be disabled

### 2.5.7 Dragging Movements (Level AA) - WCAG 2.2

- [ ] Every drag operation has a single-pointer path that is not dragging
- [ ] Reorderable lists, sliders, kanban boards, and resize handles checked
- [ ] Exceptions claimed only where dragging is essential or user-agent owned

`better-accessibility` owns the remedy patterns.

### 2.5.8 Target Size (Minimum) (Level AA) - WCAG 2.2

- [ ] Targets are at least 24x24 CSS pixels, or meet a defined exception
- [ ] Exception claimed by name: Spacing, Equivalent, Inline, User Agent Control, Essential

Undersized targets are not automatic failures — check the five exceptions before
reporting one. `better-accessibility` owns the sizing rule and the spacing math.

````

### Understandable (Principle 3)

```markdown
## 3.1 Readable

### 3.1.1 Language of Page (Level A)
- [ ] HTML lang attribute set
- [ ] Language correct for content

```html
<html lang="en">
````

### 3.1.2 Language of Parts (Level AA)

- [ ] Language changes marked

```html
<p>The French word <span lang="fr">bonjour</span> means hello.</p>
```

## 3.2 Predictable

### 3.2.1 On Focus (Level A)

- [ ] No context change on focus alone
- [ ] No unexpected popups on focus

### 3.2.2 On Input (Level A)

- [ ] No automatic form submission
- [ ] User warned before context change

### 3.2.3 Consistent Navigation (Level AA)

- [ ] Navigation consistent across pages
- [ ] Repeated components same order

### 3.2.4 Consistent Identification (Level AA)

- [ ] Same functionality = same label
- [ ] Icons used consistently

### 3.2.6 Consistent Help (Level A) - WCAG 2.2

- [ ] Help mechanisms appear in the same relative order on every page that has them
- [ ] Contact details, chat, and help links do not move between pages

Applies only to help that already exists; the criterion does not require adding help.

## 3.3 Input Assistance

### 3.3.1 Error Identification (Level A)

- [ ] Errors clearly identified
- [ ] Error message describes problem
- [ ] Error linked to field

```html
<input aria-describedby="email-error" aria-invalid="true" />
<span id="email-error" role="alert">Please enter valid email</span>
```

### 3.3.2 Labels or Instructions (Level A)

- [ ] All inputs have visible labels
- [ ] Required fields indicated
- [ ] Format hints provided

### 3.3.3 Error Suggestion (Level AA)

- [ ] Errors include correction suggestion
- [ ] Suggestions are specific

### 3.3.4 Error Prevention (Level AA)

- [ ] Legal/financial forms reversible
- [ ] Data checked before submission
- [ ] User can review before submit

### 3.3.7 Redundant Entry (Level A) - WCAG 2.2

- [ ] Information already entered in the same process is auto-populated or selectable
- [ ] Exceptions are re-entry for security, and entries no longer valid

Multi-step checkouts repeating an address are the common failure.

### 3.3.8 Accessible Authentication (Minimum) (Level AA) - WCAG 2.2

- [ ] No cognitive function test (puzzle, memory, transcription) is required to log in
- [ ] Or an alternative path exists, or a mechanism assists
- [ ] Password fields allow paste and password managers

Blocking paste on a password field is the most common failure of this criterion.
Object recognition and personal-content identification remain permitted.

````

### Robust (Principle 4)

```markdown
## 4.1 Compatible

### 4.1.1 Parsing (Level A) - Obsolete in WCAG 2.2
- [ ] Valid HTML (good practice)
- [ ] No duplicate IDs
- [ ] Complete start/end tags

### 4.1.2 Name, Role, Value (Level A)
- [ ] Custom widgets have accessible names
- [ ] ARIA roles correct
- [ ] State changes announced

```html
<!-- Accessible custom checkbox -->
<div role="checkbox"
     aria-checked="false"
     tabindex="0"
     aria-labelledby="label">
</div>
<span id="label">Accept terms</span>
````

### 4.1.3 Status Messages (Level AA)

- [ ] Status updates announced
- [ ] Live regions used correctly

```html
<div role="status" aria-live="polite">3 items added to cart</div>

<div role="alert" aria-live="assertive">Error: Form submission failed</div>
```

````

## Automated Testing

```javascript
// axe-core integration
const axe = require('axe-core');

async function runAccessibilityAudit(page) {
  await page.addScriptTag({ path: require.resolve('axe-core') });

  const results = await page.evaluate(async () => {
    return await axe.run(document, {
      runOnly: {
        type: 'tag',
        values: ['wcag2a', 'wcag21a', 'wcag2aa', 'wcag21aa', 'wcag22aa']
      }
    });
  });

  return {
    violations: results.violations,
    passes: results.passes,
    incomplete: results.incomplete
  };
}

// Playwright test example
test('should have no accessibility violations', async ({ page }) => {
  await page.goto('/');
  const results = await runAccessibilityAudit(page);

  // Failing on `incomplete` as well means axe's "could not determine" nodes
  // never disappear from the audit: they must be manually reviewed or
  // explicitly explained, not silently passed.
  expect({ violations: results.violations, incomplete: results.incomplete }).toEqual({
    violations: [],
    incomplete: []
  });
});
````

```bash
# CLI tools
npx @axe-core/cli https://example.com
npx pa11y https://example.com
lighthouse https://example.com --only-categories=accessibility
```

## Remediation Patterns

### Fix: Missing Form Labels

```html
<!-- Before -->
<input type="email" placeholder="Email" />

<!-- After: Option 1 - Visible label -->
<label for="email">Email address</label>
<input id="email" type="email" />

<!-- After: Option 2 - aria-label -->
<input type="email" aria-label="Email address" />

<!-- After: Option 3 - aria-labelledby -->
<span id="email-label">Email</span>
<input type="email" aria-labelledby="email-label" />
```

### Fix: Insufficient Color Contrast

```css
/* Before: 2.5:1 contrast */
.text {
  color: #767676;
}

/* After: 4.5:1 contrast */
.text {
  color: #595959;
}

/* Or add background */
.text {
  color: #767676;
  background: #000;
}
```

### Fix: Keyboard Navigation

```javascript
// Make custom element keyboard accessible
class AccessibleDropdown extends HTMLElement {
  connectedCallback() {
    this.setAttribute("tabindex", "0");
    this.setAttribute("role", "combobox");
    this.setAttribute("aria-expanded", "false");

    this.addEventListener("keydown", (e) => {
      switch (e.key) {
        case "Enter":
        case " ":
          this.toggle();
          e.preventDefault();
          break;
        case "Escape":
          this.close();
          break;
        case "ArrowDown":
          this.focusNext();
          e.preventDefault();
          break;
        case "ArrowUp":
          this.focusPrevious();
          e.preventDefault();
          break;
      }
    });
  }
}
```

## Best Practices

### Do's

- **Start early** - Accessibility from design phase
- **Test with real users** - Disabled users provide best feedback
- **Automate what you can** - 30-50% issues detectable
- **Use semantic HTML** - Reduces ARIA needs
- **Document patterns** - Build accessible component library

### Don'ts

- **Don't rely only on automated testing** - Manual testing required
- **Don't use ARIA as first solution** - Native HTML first
- **Don't hide focus outlines** - Keyboard users need them
- **Don't disable zoom** - Users need to resize
- **Don't use color alone** - Multiple indicators needed

## Resources

- [WCAG 2.2 Guidelines](https://www.w3.org/TR/WCAG22/)
- [WebAIM](https://webaim.org/)
- [A11y Project Checklist](https://www.a11yproject.com/checklist/)
- [axe DevTools](https://www.deque.com/axe/)
