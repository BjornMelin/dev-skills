# Performance: caching + fragments + rerun control

## Caching (default performance tool)

### `st.cache_data` (data results)

Use for:
- data loads (CSV/DB queries)
- expensive transforms that produce serializable outputs

Tips:
- Avoid mutating cached return values; treat outputs as immutable.
- Use `ttl=` for time-based invalidation; use `max_entries=` to cap memory.
- If inputs are unhashable or nondeterministic, normalize inputs before caching.

### `st.cache_resource` (shared resources)

Use for:
- DB clients / connection pools
- ML model instances
- expensive singleton objects

Tips:
- Cached objects are shared across sessions; ensure thread-safety.
- Never store per-user secrets or session state inside cached resources.

## Fragments (partial reruns)

Use `@st.fragment` to rerun only a section of the script on widget interaction.

Patterns:
- For multi-container fragments, allocate containers with `st.empty()` to avoid element accumulation.
- If a fragment needs to force a full rerun, call `st.rerun()` inside it.

## Deferred downloads (`st.download_button`)

- Prefer `data=callable` for large/expensive artifacts: compute only on click.
- When a callable is passed, it is executed on click and runs on a separate thread from the resulting script rerun (per docs).
- If a download click causes unwanted full reruns, isolate the widget inside a fragment (recommended in Streamlit docs).

## Practical checklist

1) Cache expensive IO/transforms.
2) Use forms to batch inputs.
3) Use fragments for high-frequency UI sections.
4) Stabilize widget keys; avoid rebuilding dynamic widget trees on every rerun.
