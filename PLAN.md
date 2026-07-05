<!-- Copyright 2026 John Wilger -->

# Event-Sourced EMC Redesign Plan

Redesign EMC so every mutating CLI/MCP operation appends domain events through eventcore 0.8.0 and projects Lean4/Quint artifacts from those events. This addresses issue #41 (https://github.com/jwilger/emc/issues/41) and removes the delimiter-fragile artifact-state path exposed by issue #42 (https://github.com/jwilger/emc/issues/42).

Use eventcore/eventcore-sqlite 0.8.0 from crates.io, with normal commands using `#[derive(Command)]`, `#[stream]`, `StreamResolver`, and `require!`.

## Key Changes

- Add an event log export under `model/events/v1/<event-id>.json`; the SQLite event store is only an operational cache.
- Store SQLite outside the repo by default at `$XDG_STATE_HOME/emc/projects/<sha256-realpath>/events.sqlite3`; allow override via config/env.
- On every runtime entrypoint, import/export event files, sync SQLite, then project current events into `emc.toml`, `model/lean`, `model/quint`, and reviews.
- Replace artifact parsing as command state. Lean4/Quint remain the verification artifacts, but command decisions read event-projected state.
- Allow incomplete authoring events to exist without failing the event model; workflow readiness is declared separately by a `WorkflowReadinessDeclared` event after formal verification succeeds for an exact event frontier.
- Do not persist a durable `VerificationStarted` event in v1. In-progress verification is protected by the operational project lock plus a final projection fingerprint comparison before appending readiness.
- Change generated digests to stable canonical-content hashes, not delimiter-joined strings.
- Enforce semantic data types everywhere except I/O boundaries. Raw primitives
  and structural containers are allowed only while reading external input or
  writing external output; all command, effect, runtime, projector, and verifier
  logic must operate on semantic domain types.

## Interfaces

- Existing CLI and MCP mutation/query names remain compatible.
- Add:
  - `emc list conflicts`
  - `emc resolve conflict --id <conflict-id> --choose-event <event-id>`
  - MCP `list_conflicts`
  - MCP `resolve_conflict`
- Exported event JSON includes `schema_version`, `event_id`, `command_id`, `command_ordinal`, `stream_id`, `parents`, `type`, and typed payload data.
- Add workflow-scoped `WorkflowReadinessDeclared` events on `workflow::<workflow-slug>` when a workflow is complete enough for downstream consumption.
- `WorkflowReadinessDeclared` payload includes `workflow`, `projection_fingerprint`, `model_content_digest`, `verified_at`, `verified_by`, and optional `review_event_id`.
- Stream IDs:
  - `project`
  - `workflow::<workflow-slug>`
  - `slice::<slice-slug>`
  - `review::<workflow-slug>`

## Merge Semantics

- Independent event files merge automatically because each event is its own tracked JSON file.
- Import topologically sorts events by parents, using `event_id` only as a deterministic tie-breaker.
- Same semantic key plus identical payload is idempotent.
- Same semantic key plus different concurrent payload becomes an unresolved semantic conflict.
- Mutations fail while unresolved conflicts exist, except resolve conflict, which appends a `ConflictResolved` event.
- Workflow readiness is stale if the readiness event's `projection_fingerprint` no longer matches the current exported-event frontier.
- No legacy artifact-only migration: if an existing project has generated artifacts but no event export, EMC reports a pre-release upgrade error.

## Implementation Plan

1. Add dependencies: `eventcore = { version = "0.8", features = ["sqlite"] }`, `eventcore-sqlite = "0.8"`, `tokio`, `uuid`, `sha2`, `hex`, and `fs4`.
2. Add event runtime modules for event schema, stream IDs, event export/import, SQLite path resolution, file locking, and projection fingerprinting.
3. Add eventcore command structs for all current mutating operations; use derive macros for fixed-stream commands and manual `CommandStreams` only for import/conflict-resolution batches.
4. Build an in-memory model projector from `EmcEvent` history and refactor emitters so Lean/Quint/project/review files are generated from projected state, not patched source text.
5. Update CLI/MCP execution so both routes call the same event-sourced runtime wrapper.
6. Add semantic conflict detection and resolution events.
7. Add workflow readiness declaration:
   - Compute the current exported-event `projection_fingerprint`.
   - Project exactly that event frontier into Lean4/Quint artifacts.
   - Run Lean4 and Quint verification.
   - Compute `model_content_digest` from canonical generated model content.
   - Recompute the exported-event `projection_fingerprint` immediately before append.
   - Append `WorkflowReadinessDeclared` only if the fingerprint is unchanged.
8. Run a thorough architectural review before final PR:
   - Audit all event-sourced runtime, command, effect, projector, verifier, and parser surfaces for semantic data type use.
   - Raw `String`, `Vec<String>`, `Option<String>`, primitive booleans/numbers with domain meaning, tuples used as records, and `serde_json::Value` are allowed only at I/O boundaries.
   - I/O boundaries include CLI/MCP input parsing, JSON event import/export, SQLite adapter serialization, process argument construction, command output, and file contents.
   - Convert external values into semantic domain types immediately at the boundary, keep semantic types through all internal logic, and serialize primitives only when crossing back out through an I/O boundary.
   - Eventcore command structs and effect variants must carry semantic types internally; exported event DTOs may serialize primitives only at the event boundary.
   - Add or update focused behavior/static guardrail coverage for any discovered primitive-leak regressions.
9. Update README and `docs/event-model/formal-modeling-rules.md` status markers for generator/verification changes.

## Test Plan

- Concurrent CLI/MCP `add_slice` calls against one project all complete without corrupt artifacts; `emc check` passes.
- Slice descriptions containing commas, pipes, semicolons, and colons remain correct in event export and generated Lean/Quint.
- Deleting the XDG SQLite DB then running `emc check` rebuilds from `model/events/v1`.
- Two merged branches adding different slices import cleanly and project both.
- Two merged branches changing the same field differently produce `list conflicts`; resolving appends a resolution event and restores `emc check`.
- Workflow readiness declaration fails if the exported-event frontier changes between projection/verification and append.
- Workflow readiness declaration appends `WorkflowReadinessDeclared` after a fresh projection and successful Lean4/Quint verification.
- A previous `WorkflowReadinessDeclared` is reported stale after a later workflow-relevant event append changes the current event frontier.
- Existing behavior tests continue to assert generated artifacts and command output, not source-file text.
