# DMC Common Tasks And Troubleshooting

## Common Tasks

### Form with Validation

```python
@callback(
    Output("submit-btn", "disabled"),
    Output("error-text", "children"),
    Input("email-input", "value"),
    Input("password-input", "value"),
)
def validate_form(email, password):
    errors = []
    if not email or "@" not in email:
        errors.append("Valid email required")
    if not password or len(password) < 8:
        errors.append("Password must be 8+ characters")
    return bool(errors), ", ".join(errors)
```

### Modal Open/Close

```python
app.layout = dmc.MantineProvider([
    dmc.Button("Open Modal", id="open-modal-btn"),
    dmc.Modal(
        id="my-modal",
        title="Confirm Action",
        children=[
            dmc.Text("Are you sure?"),
            dmc.Group([
                dmc.Button("Cancel", id="cancel-btn", variant="outline"),
                dmc.Button("Confirm", id="confirm-btn", color="red"),
            ], justify="flex-end", mt="md"),
        ],
    ),
])

@callback(
    Output("my-modal", "opened"),
    Input("open-modal-btn", "n_clicks"),
    Input("cancel-btn", "n_clicks"),
    Input("confirm-btn", "n_clicks"),
    prevent_initial_call=True,
)
def toggle_modal(open_clicks, cancel, confirm):
    from dash import ctx
    if ctx.triggered_id == "open-modal-btn":
        return True
    return False
```

### Loading State

```python
from dash import dcc

app.layout = dmc.MantineProvider([
    dmc.Button("Load Data", id="load-btn"),
    dcc.Loading(
        id="loading",
        type="circle",
        children=dmc.Container(id="data-container"),
    ),
])

@callback(Output("data-container", "children"), Input("load-btn", "n_clicks"))
def load_data(n):
    import time
    time.sleep(2)  # Simulate slow operation
    return dmc.Text("Data loaded!")
```

### Chart with Data

```python
data = [
    {"month": "Jan", "sales": 100, "profit": 20},
    {"month": "Feb", "sales": 150, "profit": 35},
    {"month": "Mar", "sales": 120, "profit": 25},
]

dmc.BarChart(
    data=data,
    dataKey="month",
    series=[
        {"name": "sales", "color": "blue.6"},
        {"name": "profit", "color": "green.6"},
    ],
    h=300,
    withLegend=True,
    withTooltip=True,
)
```

---

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `MantineProvider is required` | Component outside provider | Wrap entire layout in `dmc.MantineProvider([...])` |
| `Invalid theme color` | Color not in theme | Use built-in colors (`blue`, `red`) or add to `theme["colors"]` |
| `Callback output not found` | Component not in layout | Ensure component with ID exists in layout |
| `Circular callback detected` | Output also used as Input | Use `State` instead of `Input` for non-triggering values |
| `Pattern-matching ID mismatch` | Dict keys don't match | Ensure `type` and `index` keys match exactly |
| `Duplicate callback outputs` | Same output in multiple callbacks | Add `allow_duplicate=True` to additional callbacks |

### Debug Tips

1. **Check browser console** for JavaScript errors
2. **Use `debug=True`** in `app.run()` for detailed Python errors
3. **Print `ctx.triggered_id`** to see which input fired
4. **Validate JSON-serializable** callback returns (no Python objects)
5. **Test with `prevent_initial_call=True`** to avoid startup errors

### DMC v2.x Gotchas

- `DateTimePicker`: Use `timePickerProps` not `timeInputProps`
- `Carousel`: Embla options need `{"containScroll": "trimSnaps"}` wrapper
- Default `reuseTargetNode=True` may cause Portal issues - set to `False` if overlays misbehave
- Use `MantineProvider` not `MantineProviderV2` (deprecated)

→ Full migration guide: [references/migration-v2.md](references/migration-v2.md)

