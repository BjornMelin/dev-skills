# DMC Core Patterns And Components

## Core Patterns

### Theming

Configure theme via MantineProvider:

```python
theme = {
    "primaryColor": "blue",
    "fontFamily": "Inter, sans-serif",
    "defaultRadius": "md",
    "colors": {
        "brand": ["#f0f9ff", "#e0f2fe", "#bae6fd", "#7dd3fc", "#38bdf8",
                  "#0ea5e9", "#0284c7", "#0369a1", "#075985", "#0c4a6e"]
    },
    "components": {
        "Button": {"defaultProps": {"size": "md", "radius": "md"}},
        "TextInput": {"defaultProps": {"size": "sm"}},
    }
}

app.layout = dmc.MantineProvider(
    theme=theme,
    forceColorScheme="light",  # or "dark", or None for auto
    children=[...]
)
```

**Theme Toggle Pattern** (clientside callback):

```python
from dash import clientside_callback, ClientsideFunction

app.layout = dmc.MantineProvider(
    id="mantine-provider",
    children=[
        dcc.Store(id="theme-store", storage_type="local", data="light"),
        dmc.Switch(id="theme-switch", label="Dark mode", checked=False),
        # ... rest of layout
    ]
)

clientside_callback(
    """(checked) => checked ? "dark" : "light" """,
    Output("mantine-provider", "forceColorScheme"),
    Input("theme-switch", "checked"),
)
```

→ Full theming guide: [references/theming-patterns.md](references/theming-patterns.md)

### Styling

**Style Props** - Universal props on all DMC components:

| Prop | CSS Property | Values |
|------|--------------|--------|
| `m`, `mt`, `mb`, `ml`, `mr`, `mx`, `my` | margin | `xs`, `sm`, `md`, `lg`, `xl` or number (px) |
| `p`, `pt`, `pb`, `pl`, `pr`, `px`, `py` | padding | same as margin |
| `c` | color | `"blue"`, `"red.6"`, `"dimmed"`, `"var(--mantine-color-text)"` |
| `bg` | background | same as color |
| `w`, `h` | width, height | `"100%"`, `"50vw"`, number (px) |
| `maw`, `mah`, `miw`, `mih` | max/min width/height | same as w, h |
| `fw` | font-weight | `400`, `500`, `700` |
| `fz` | font-size | `xs`, `sm`, `md`, `lg`, `xl` or number |
| `ta` | text-align | `"left"`, `"center"`, `"right"` |
| `td` | text-decoration | `"underline"`, `"line-through"` |

**Responsive Values** - Dict with breakpoints:

```python
dmc.Button("Click", w={"base": "100%", "sm": "auto", "lg": 200})
dmc.Stack(gap={"base": "xs", "md": "lg"})
```

**Styles API** - Target nested elements:

```python
dmc.Select(
    data=["A", "B", "C"],
    classNames={"input": "my-input", "dropdown": "my-dropdown"},
    styles={"label": {"fontWeight": 700}, "input": {"borderColor": "blue"}},
)
```

→ Full styling guide: [references/styling-guide.md](references/styling-guide.md)

### Callbacks

**Basic Pattern**:

```python
from dash import callback, Input, Output, State

@callback(
    Output("output", "children"),
    Input("button", "n_clicks"),
    State("input", "value"),
    prevent_initial_call=True,
)
def update(n_clicks, value):
    return f"Clicked {n_clicks} times with value: {value}"
```

**Pattern-Matching** (dynamic components):

```python
from dash import ALL, MATCH, callback_context as ctx

# ALL: Respond to any button with type "dynamic-btn"
@callback(
    Output("output", "children"),
    Input({"type": "dynamic-btn", "index": ALL}, "n_clicks"),
)
def handle_all(n_clicks_list):
    triggered = ctx.triggered_id  # {"type": "dynamic-btn", "index": X}
    return f"Button {triggered['index']} clicked"

# MATCH: Update the output matching the triggered input
@callback(
    Output({"type": "item-output", "index": MATCH}, "children"),
    Input({"type": "item-btn", "index": MATCH}, "n_clicks"),
    prevent_initial_call=True,
)
def handle_match(n):
    return f"Clicked {n} times"
```

**Clientside Callback** (browser-side JavaScript):

```python
from dash import clientside_callback

clientside_callback(
    """(n) => n ? `Clicked ${n} times` : "Not clicked" """,
    Output("output", "children"),
    Input("button", "n_clicks"),
)
```

**DMC-Specific Props**:
- `debounce=300` - Delay callback trigger (ms) for TextInput, Textarea
- `persistence=True` - Persist value across page reloads
- `persistence_type="local"` - Storage type: memory, local, session

→ Full callbacks reference: [references/callbacks-advanced.md](references/callbacks-advanced.md)

---

## Multi-Page Apps

Use Dash Pages with DMC AppShell:

```python
# app.py
import dash
from dash import Dash
import dash_mantine_components as dmc

app = Dash(__name__, use_pages=True, pages_folder="pages")

app.layout = dmc.MantineProvider([
    dmc.AppShell(
        [
            dmc.AppShellHeader(dmc.Group([
                dmc.Title("My App", order=3),
                dmc.Switch(id="theme-switch"),
            ], h="100%", px="md")),
            dmc.AppShellNavbar([
                dmc.NavLink(label=page["name"], href=page["path"], active=page["path"] == "/")
                for page in dash.page_registry.values()
            ], p="md"),
            dmc.AppShellMain(dash.page_container),
        ],
        header={"height": 60},
        navbar={"width": 250, "breakpoint": "sm", "collapsed": {"mobile": True}},
        padding="md",
    )
])

if __name__ == "__main__":
    app.run(debug=True)
```

```python
# pages/home.py
import dash
import dash_mantine_components as dmc

dash.register_page(__name__, path="/", name="Home")

layout = dmc.Container([
    dmc.Title("Welcome", order=2),
    dmc.Text("Home page content"),
], py="xl")
```

```python
# pages/analytics.py
import dash
import dash_mantine_components as dmc

dash.register_page(__name__, path="/analytics", name="Analytics")

layout = dmc.Container([
    dmc.Title("Analytics", order=2),
    # Charts, tables, etc.
], py="xl")
```

**Variable Paths**:

```python
# pages/user.py
dash.register_page(__name__, path_template="/user/<user_id>")

def layout(user_id=None):
    return dmc.Container([
        dmc.Title(f"User: {user_id}", order=2),
    ])
```

→ Full multi-page guide: [references/multi-page-apps.md](references/multi-page-apps.md)

---

## Component Categories

Quick links to reference documentation:

| Category | Components | Reference |
|----------|------------|-----------|
| **All Components** | 90+ components with props/events | [components-quick-ref.md](references/components-quick-ref.md) |
| **Theming** | MantineProvider, theme object, colors | [theming-patterns.md](references/theming-patterns.md) |
| **Styling** | Style props, Styles API, CSS variables | [styling-guide.md](references/styling-guide.md) |
| **Callbacks** | Pattern-matching, clientside, background | [callbacks-advanced.md](references/callbacks-advanced.md) |
| **Multi-Page** | Dash Pages, routing, AppShell | [multi-page-apps.md](references/multi-page-apps.md) |
| **Charts** | Data formats, series config | [charts-data-formats.md](references/charts-data-formats.md) |
| **Date Pickers** | DatePicker, DatesProvider, localization | [date-pickers-guide.md](references/date-pickers-guide.md) |
| **Dash Core** | dcc.Store, caching, performance | [dash-fundamentals.md](references/dash-fundamentals.md) |
| **Migration** | v1.x to v2.x breaking changes | [migration-v2.md](references/migration-v2.md) |

### Asset Templates

Copy and adapt these templates:

| Template | Description |
|----------|-------------|
| [app_single_page.py](assets/app_single_page.py) | Complete single-page DMC app with theme toggle |
| [app_multi_page.py](assets/app_multi_page.py) | Multi-page app with Dash Pages and AppShell |
| [callbacks_patterns.py](assets/callbacks_patterns.py) | All callback pattern examples |
| [theme_presets.py](assets/theme_presets.py) | Pre-built theme configurations |

### Utility Scripts

| Script | Usage |
|--------|-------|
| [fetch_docs.py](scripts/fetch_docs.py) | `python fetch_docs.py "Select"` - Fetch/search official llms.txt |
| [scaffold_app.py](scripts/scaffold_app.py) | `python scaffold_app.py myapp --type multi --shell` |
| [generate_theme.py](scripts/generate_theme.py) | `python generate_theme.py --primary "#0ea5e9"` |
| [component_search.py](scripts/component_search.py) | `python component_search.py "select"` |

---
