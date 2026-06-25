# Key Factories

Use key factories when an app has multiple related query families.

## Rules

- Create `all`, `lists`, `list(filters)`, `details`, and `detail(id)` style factories where useful.
- Pair key factories with `queryOptions` factories.
- Use broad roots for invalidating a whole domain and narrow keys for detail updates.
- Avoid stringly typed scattered keys.
