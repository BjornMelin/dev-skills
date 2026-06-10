---
name: dmc-py
description: Dash + DMC v2—MantineProvider, styles, Styles API, callbacks (pattern, clientside, bg), Pages, charts, modals, full DMC. Dashboards, forms, viz.
---

# Dash Mantine Components (DMC) v2.x

Build modern Dash applications with 100+ Mantine UI components.

## Quick Start

Minimal DMC app requiring MantineProvider wrapper:

```python
from dash import Dash, callback, Input, Output
import dash_mantine_components as dmc

app = Dash(__name__)

app.layout = dmc.MantineProvider([
    dmc.Container([
        dmc.Title("My DMC App", order=1),
        dmc.TextInput(label="Name", id="name-input", placeholder="Enter name"),
        dmc.Button("Submit", id="submit-btn", mt="md"),
        dmc.Text(id="output", mt="md"),
    ], size="sm", py="xl")
])

@callback(Output("output", "children"), Input("submit-btn", "n_clicks"), Input("name-input", "value"))
def update_output(n_clicks, name):
    if not n_clicks:
        return ""
    return f"Hello, {name or 'World'}!"

if __name__ == "__main__":
    app.run(debug=True)
```

**Critical**: All DMC components MUST be inside `dmc.MantineProvider`.

> **Version Note:** This skill targets DMC 2.x (Mantine 8.x). Run `pip show dash-mantine-components` to check your installed version. For the latest features and API changes, use `fetch_docs.py` to query the official documentation at <https://www.dash-mantine-components.com/assets/llms.txt>

---

## Workflow Decision Tree

Select components by use case:

### Form Inputs

| Need | Component | Key Props |
| --- | --- | --- |
| Text input | `TextInput` | `label`, `placeholder`, `value`, `debounce` |
| Dropdown | `Select` | `data`, `value`, `searchable`, `clearable` |
| Multi-select | `MultiSelect` | `data`, `value`, `searchable` |
| Checkbox | `Checkbox` | `label`, `checked` |
| Toggle | `Switch` | `label`, `checked`, `onLabel`, `offLabel` |
| Number | `NumberInput` | `value`, `min`, `max`, `step` |
| Date | `DatePickerInput` | `value`, `type`, `minDate`, `maxDate` |
| Rich text | `Textarea` | `label`, `value`, `autosize`, `minRows` |
| File upload | `FileInput` | `value`, `accept`, `multiple` |

### Layout

| Need | Component | Key Props |
| --- | --- | --- |
| Content wrapper | `Container` | `size`, `px`, `py` |
| Vertical stack | `Stack` | `gap`, `align`, `justify` |
| Horizontal row | `Group` | `gap`, `justify`, `wrap` |
| CSS Grid | `Grid`, `GridCol` | `columns`, `gutter`, `span` |
| Full app shell | `AppShell` | `header`, `navbar`, `aside`, `footer` |
| Card container | `Card` | `shadow`, `padding`, `radius`, `withBorder` |
| Flex layout | `Flex` | `direction`, `wrap`, `gap`, `align` |

### Navigation

| Need | Component | Key Props |
| --- | --- | --- |
| Nav item | `NavLink` | `label`, `href`, `active`, `leftSection` |
| Tabs | `Tabs`, `TabsList`, `TabsPanel` | `value`, `orientation` |
| Breadcrumb | `Breadcrumbs` | `separator` |
| Stepper | `Stepper`, `StepperStep` | `active`, `onStepClick` |
| Pagination | `Pagination` | `value`, `total`, `siblings` |
| Table of contents | `TableOfContents` | `links`, `variant`, `active` |

### Feedback & Overlays

| Need | Component | Key Props |
| --- | --- | --- |
| Modal dialog | `Modal` | `opened`, `onClose`, `title`, `centered` |
| Side panel | `Drawer` | `opened`, `onClose`, `position`, `size` |
| Toast | `Notification` | `title`, `message`, `color`, `icon` |
| Alert banner | `Alert` | `title`, `color`, `variant`, `icon` |
| Loading | `Loader`, `LoadingOverlay` | `size`, `type`, `visible` |
| Progress | `Progress`, `RingProgress` | `value`, `size`, `sections` |
| Tooltip | `Tooltip` | `label`, `position`, `withArrow` |
| Copy button | `CopyButton` | `value`, `timeout` |

### Data Display

| Need | Component | Key Props |
| --- | --- | --- |
| Data table | `Table` | `data`, `striped`, `highlightOnHover` |
| Accordion | `Accordion`, `AccordionItem` | `value`, `multiple`, `variant` |
| Timeline | `Timeline`, `TimelineItem` | `active`, `bulletSize` |
| Badge | `Badge` | `color`, `variant`, `size` |

### Charts

| Need | Component | Key Props |
| --- | --- | --- |
| Line | `LineChart` | `data`, `dataKey`, `series` |
| Bar | `BarChart` | `data`, `dataKey`, `series`, `orientation` |
| Area | `AreaChart` | `data`, `dataKey`, `series` |
| Pie/Donut | `DonutChart`, `PieChart` | `data`, `chartLabel` |
| Scatter | `ScatterChart` | `data`, `dataKey`, `series` |

### What's New in Recent Versions

**v2.5.x:**

- `TableOfContents` - Auto-generated table of contents from headings
- `selectFirstOptionOnDropdownOpen` prop for Select/MultiSelect/Autocomplete
- `openOnFocus` prop for Combobox components
- AppShell `mode="static"` for nested shells
- `window.MantineCore` / `window.MantineHooks` for custom component building

**v2.4.x:**

- `CopyButton` / `CustomCopyButton` - Clipboard operations
- `getEditor(id)` - Access RichTextEditor TipTap instance in clientside callbacks
- Function props for chart axis/grid customization

**v2.3.x:**

- `MiniCalendar` - Compact calendar component
- `ScrollAreaAutoheight` - Auto-sizing scroll area
- `DirectionProvider` - RTL text direction support

→ Full component reference: [references/components-quick-ref.md](references/components-quick-ref.md)

---

## Detailed References

Load these on demand:

- [references/core-patterns-and-components.md](references/core-patterns-and-components.md) for theming, styling, callbacks, Pages, component categories, asset templates, and utility scripts.
- [references/best-practices.md](references/best-practices.md) for the consolidated DMC best-practices rule index.
- [references/best-practices-expanded.md](references/best-practices-expanded.md) for the expanded compiled rule reference.
- [references/common-tasks-and-troubleshooting.md](references/common-tasks-and-troubleshooting.md) for forms, modals, loading states, charts, common errors, debug tips, and DMC v2 gotchas.
- [references/components-quick-ref.md](references/components-quick-ref.md) for component selector and API reminders.
- [references/migration-v2.md](references/migration-v2.md) when upgrading older DMC code.

## Operating Rules

- Wrap all DMC components in `dmc.MantineProvider`.
- Check the installed DMC version with `pip show dash-mantine-components` before using version-sensitive APIs.
- Prefer official docs via `scripts/fetch_docs.py` when API details matter.
- Use DMC v2/Mantine 8 patterns; do not preserve old v0/v1 compatibility unless the repo requires it.
- For reviews and refactors, apply the best-practices rules in `rules/` by priority before broad redesign.
