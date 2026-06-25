# Navigation

Use typed Router primitives instead of raw anchors or history mutation.

## Rules

- Prefer `Link` for user navigation.
- Use `useNavigate({ from })` in route-aware components.
- Preserve or reset search params intentionally.
- Use active link props for accessible active states.
- Use route masks only when modal or overlay URLs need a distinct display URL.
