# Rust Rules

- Prioritize correctness and clarity.
- Add comments only when they explain a non-obvious reason.
- Prefer existing files unless introducing a real logical component.
- Propagate errors with `?` or handle them explicitly. Do not silently discard
  fallible results with `let _ =`.
- Avoid `unwrap()` and other avoidable panic paths.
- Avoid unchecked indexing when bounds are not guaranteed.
- Ensure async failures reach the UI as meaningful feedback.
- Do not create new `mod.rs` files; use descriptive module files.
- For a new crate, prefer a descriptive `[lib] path` rather than a default
  `lib.rs`.
- Use full words for identifiers.
- Scope cloned values inside async blocks to keep borrows and lifetimes clear.
- Do not add unsafe code; the workspace forbids it.
- Avoid creative or unrelated additions.
