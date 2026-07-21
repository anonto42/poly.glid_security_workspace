# UI-First Roadmap

PolyGlid is desktop-first. The Dioxus application is the primary product
surface; the CLI is frozen as a development and regression harness until the
desktop MVP is complete.

The architectural source of truth is
[Client Architecture](../architecture/CLIENT_ARCHITECTURE.md). The exact MVP
gate is [UI-First MVP](MVP.md).

## Current Baseline

| Capability | Current state |
| --- | --- |
| Rust workspace, WIT contract, Wasmtime runtime, and Recon component | Present |
| Native Dioxus workbench | Connected |
| UI-safe DTOs, typed errors, `ClientGateway`, and `LocalClient` | Connected and tested |
| SQLite workspace/project catalog and shell preferences | Connected |
| Project create, rename, remove, and delete | Connected |
| Plugin inspect, install, enable/disable, and uninstall | Connected |
| Official Recon signature | Blocked: v0.10.0 omits the `.component.sig` required by Balanced policy |
| Permission review | Explicit allow-once review; durable decisions remain |
| Desktop execution | Asynchronous `JobId`, events, history, and cancellation connected |
| Reports | Persisted detail plus JSON, Markdown, and SARIF export connected |
| Targets | Validated and persisted |
| Work Tracks, Automation, AI Agents, source preview, and terminal | Removed from desktop production source |
| Native release archives | Automated; installers and dependency declaration remain |

“Connected” is not the same as MVP-complete. The current desktop proves the
main local slices, while the unchecked work below hardens their ownership,
permission scope, recovery behavior, testing, and distribution.

## Phase 0: Adopt the Client Direction

Status: complete.

- [x] Make Dioxus Desktop the primary client.
- [x] Record the CLI as a frozen test harness.
- [x] Define local and future remote gateway boundaries.
- [x] Document real, partial, and removed UI states.
- [x] Mark CLI-first/Tauri plans as superseded.

Exit condition: contributors can identify the canonical client architecture and
do not add new product behavior directly to the CLI.

## Phase 1: Client Contracts and Local Host

Status: foundation delivered; controller separation remains.

- [x] Add UI-safe client DTOs, stable job IDs, typed events, and errors.
- [x] Introduce cloneable `ClientGateway` and in-process `LocalClient`.
- [x] Open one local service graph for configuration, SQLite, registry,
      execution manager, and runtime.
- [x] Load workspaces, projects, plugins, targets, executions, reports, and shell
      preferences through one bootstrap snapshot.
- [x] Remove the old `DesktopBackend` module and database/runtime types from UI
      models.
- [x] Add focused local-client and core tests.
- [ ] Move remaining `LocalClient` calls out of Dioxus wiring components and
      into feature controllers.
- [ ] Extract stable contracts to `crates/client-api` before adding a second
      client.

Exit condition: views communicate only through feature controllers and stores,
and no presentation component knows the concrete local adapter.

## Phase 2: Explicit Permission Decisions

Status: allow-once slice delivered; durable scoped policy remains.

- [x] Keep plugin installation separate from execution authorization.
- [x] Show each live manifest capability request and its resource scope.
- [x] Start every per-run review with nothing approved.
- [x] Disable execution until every requested capability kind is selected.
- [x] Re-inspect the installed executable and reject missing or unexpected
      capability kinds immediately before submission.
- [x] Audit required and granted allow-once decisions.
- [ ] Represent decisions with stable approval IDs rather than copied
      capability-kind arrays.
- [ ] Enforce the exact manifest resource scope and bind it to plugin identity,
      target, checksum, and version.
- [ ] Add expiration, revocation, allow-session, and allow-workspace decisions.
- [ ] Add a decision-management view and forged/stale/mismatched approval tests.

Exit condition: enabling or installing a plugin cannot implicitly authorize a
host capability, and every linked capability has a valid scoped decision.

## Phase 3: Feature Stores and Clear Components

Status: first split and product cleanup delivered; controller tests remain.

- [x] Replace the former flat state collection with `ShellStore`,
      `CatalogStore`, `PluginStore`, and `RunStore`.
- [x] Use one overlay enum instead of unrelated modal flags.
- [x] Restrict navigation to Projects, New scan, Executions, Reports, Plugins,
      and Settings.
- [x] Remove seeded plugins/targets and preview areas from production source.
- [x] Add consistent focus, disabled, busy, loading, empty, error, permission,
      execution, and report styles.
- [ ] Split Scanner, Executions, Reports, and Settings stores as behavior grows.
- [ ] Make each feature controller the only writer for its store.
- [ ] Add controller transition and component-state tests.
- [ ] Complete focus restoration, reduced-motion, and screen-reader checks.

Exit condition: each production component is understandable and testable
without reading global state, database code, or runtime code.

## Phase 4: Asynchronous Executions and Persisted Reports

Status: end-to-end foundation delivered; richer progress and metadata remain.

- [x] Return `JobId` immediately from execution submission.
- [x] Subscribe to typed state, completion, failure, and log events.
- [x] Show execution history and allow active-job cancellation.
- [x] Persist reports and expose Executions and Reports views.
- [x] Render real summaries, severities, findings, and execution metrics.
- [x] Export JSON, Markdown, and SARIF through one client operation.
- [ ] Add richer typed progress stages and confirmed `cancelling` presentation.
- [ ] Harden subscription reconnect/lag recovery and late-event correlation.
- [ ] Link report history explicitly to selected project and approval identity.
- [ ] Add report filters and expose HTML export when its product presentation is
      ready.

Exit condition: an operator can start, observe, cancel, reopen, and export a run
without blocking the interface or using the CLI, including after restart and
event reconnect.

## Phase 5: Honest Desktop MVP

Status: product surface is honest; acceptance work remains.

- [x] Use Projects, New scan, Executions, Reports, Plugins, and Settings as the
      only product navigation.
- [x] Remove Work Tracks, Automation, AI Agents, source preview, terminal, and
      fabricated result metrics from active routing.
- [x] Add startup, catalog, target, plugin, permission, execution, report, and
      export error presentation.
- [ ] Make project selection the explicit target/execution/report context.
- [ ] Persist and validate all product settings.
- [ ] Complete keyboard, focus, minimum-window, reduced-motion, and screen-reader
      acceptance checks.
- [ ] Add an end-to-end packaged-desktop test for the full MVP journey.
- [ ] Complete every item in the [MVP checklist](MVP.md#completion-checklist).

Exit condition: every visible control is connected to real behavior and the MVP
checklist passes on supported clean machines.

## Phase 6: Desktop Distribution

Status: release archives exist; user-grade packaging remains.

- [x] Build native desktop executables for Linux, Windows, macOS Intel, and
      macOS Apple Silicon in release automation.
- [x] Publish checksums and verified download links.
- [ ] Define supported operating-system versions and architectures.
- [ ] Produce normal platform installers/packages with declared dependencies.
- [ ] Create an offline long-lived Ed25519 release-signing identity and protect
      its private seed only in the release environment/CI secret store.
- [ ] Pin the official public key or fingerprint for explicit trust bootstrap.
- [ ] Make CI sign and verify `recon-probe.component.wasm`, then package the
      adjacent `recon-probe.component.sig` in every desktop artifact.
- [ ] Publish a new signed release; do not weaken Balanced policy or silently
      fall back to Development policy for the unsigned v0.10.0 component.
- [ ] Test packages and the primary journey on clean supported machines.
- [ ] Sign platform artifacts and define upgrade/rollback behavior.

Exit condition: a new user can install a supported package and complete the MVP
journey without manually discovering native dependencies or weakening security
policy.

## Phase 7: Additional Plugin Value

Status: future, after desktop security and reports stabilize.

- add bounded reconnaissance and defensive audit components;
- require stable structured reports for every component;
- complete plugin signature and publisher trust policy;
- design registry/index distribution without bypassing local review;
- retain replaceable, independently testable WASM components.

Exit condition: additional plugins use the same permission, execution, audit,
and report journey without feature-specific UI exceptions.

## Phase 8: Secured Remote Clients

Status: future.

- version the client/server protocol;
- add authentication, authorization, transport encryption, rate limits, and
  actor-aware audit records;
- implement a server-side application-host adapter;
- implement `RemoteGateway` for a web report/workspace client;
- add mobile status, report review, and approval only after remote approval
  security is designed and tested.

Exit condition: remote clients preserve the same policy and audit guarantees as
the local desktop client.

## Deferred CLI Work

The CLI remains buildable for automated runtime checks, component inspection,
and recovery diagnostics. During the desktop MVP phases:

- do not design new end-user workflows around terminal output;
- do not make a CLI command an acceptance condition when the desktop cannot
  complete the same product outcome;
- keep focused CLI regression paths where they provide cheap engine/runtime
  verification;
- return to CLI UX and standalone packaging only after the desktop MVP gate is
  complete.
