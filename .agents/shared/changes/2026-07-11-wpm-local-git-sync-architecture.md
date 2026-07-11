# WPM Local Git-Sync Architecture

Date: 2026-07-11

## Decision recorded

WPM MVP will be a Rust/Dioxus desktop application with local SQLx/SQLite and no
separate backend. Immutable events on a dedicated `wpm-data` Git branch distribute
new nodes, comments, tasks, and other WPM changes to developer clients.

## Important constraints

- SQLite files are never committed or shared.
- A commit indicates new shared WPM event data is available.
- Clients import unseen events idempotently and refresh affected Dioxus state.
- Sync uses an isolated worktree so active source branches are untouched.
- Git provides eventual collaboration, not instant real-time concurrency.
- Axum, centralized PostgreSQL, and mobile synchronization are deferred.
