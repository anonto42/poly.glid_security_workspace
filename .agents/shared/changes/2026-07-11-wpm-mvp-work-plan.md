# WPM MVP Work Plan

Date: 2026-07-11

## What changed

Added a compact execution plan for the WPM MVP and a deferred feature roadmap.
The repeated attachment matches the previously reviewed proposal exactly.

## Current direction

The initial assumption was Rust/Axum, SQLite local-first, an existing
Tauri/React UI, single-user, and a read-only workspace helper.

This UI assumption was superseded on 2026-07-14. The canonical application is
now the single Rust/Dioxus desktop project at `slices/polyglid-desktop`.
