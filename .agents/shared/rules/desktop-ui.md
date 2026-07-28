# Desktop UI Rules

- Use the existing Dioxus 0.7 component and signal patterns.
- Keep `AppState` as the shared shell/catalog state until a distinct feature
  requires a separate logical store.
- Call controllers from event handlers; do not bypass the client boundary.
- Run blocking gateway calls with `spawn_blocking`.
- Handle both task-join errors and operation errors.
- Clear stale notices/errors when a new mutation starts.
- Disable conflicting actions while a mutation is active.
- Preserve keyboard and accessibility behavior, including labels, status roles,
  alert roles, busy states, focus, and confirmation text.
- Add styles to the existing phase stylesheet when the change belongs to the
  current Projects feature.
