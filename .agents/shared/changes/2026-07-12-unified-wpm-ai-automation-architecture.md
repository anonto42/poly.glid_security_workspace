# Unified WPM, AI, and Automation Architecture

Date: 2026-07-12

## Decision

WPM, AI, and automation will be developed as capabilities of one Rust platform.
Dioxus WPM is the control plane, a central execution engine owns task lifecycle
and policy, and AI/automation/Git/workspace functions are replaceable adapters.

## Why

This removes competing orchestration paths, centralizes permissions and evidence,
keeps domain logic UI/provider independent, and permits later Axum/PostgreSQL
collaboration without rewriting the MVP.

## Follow-up

Implementation begins with shared contracts, domain types, SQLite audit/outbox,
and executor state-machine tests before migrating current AI or Make functionality.
