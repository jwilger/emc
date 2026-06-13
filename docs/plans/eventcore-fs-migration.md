<!-- Copyright 2026 John Wilger -->

# eventcore 0.9 + eventcore-fs migration ledger

Goal: replace the custom filesystem event log (`model/events/v1/*.json` DAG +
bespoke conflict detection) and the external SQLite operational store with a
single committed, git-mergeable `eventcore_fs::FileEventStore`. Deliver as one
PR on `feat/eventcore-fs-migration`, shepherd through CI/review to merge.

## Architecture decision

- `FileEventStore` rooted in-repo at `model/events` becomes the single store.
  Only `model/events/events/` (immutable JSONL transactions) is committed; the
  store writes its own `.gitignore`/`.gitattributes` excluding `tmp/`,
  `index/`, `.eventcore/`, `.lock`.
- `EmcEvent` (already `Serialize`/`Deserialize`, carries full semantic
  payloads) is the sole stored event type.
- Write path: `EventDraft` -> eventcore `Command` -> `execute(&store, …)` which
  appends `EmcEvent`. No custom JSON file is written anymore.
- Read/projection path: `FileEventStore::read_events::<EmcEvent>()` in ingestion
  order -> convert `EmcEvent` -> `ExportedEventBody` -> existing
  `ProjectedModel::from_events` (refactored to take bodies). Projection logic is
  otherwise unchanged.
- Conflicts: EMC's semantic conflict (two `WorkflowUpdated`/`SliceUpdated`
  sharing parents with divergent payloads) == eventcore-fs `Fork` (two
  transactions extending a stream from the same base version).
  `emc list conflicts` -> `FileEventStore::status()/detect_forks()`.
  `emc resolve conflict` -> `FileEventStore::reconcile(resolver)` choosing the
  selected branch.
- Projection fingerprint: derived from the ordered stored events (content), no
  longer from custom exported event ids.

## What gets deleted

- `eventcore-sqlite` dependency; `eventcore` `sqlite` feature; `fs4` (store
  owns its locking) if the project-runtime lock is replaced by the store lock.
- `event_runtime.rs`: SQLite store/migrate/sync/prune/prerequisite-repair.
- `events.rs`: `ExportedEvent`, `ExportedEventMeta`, `ExportedEventId`,
  `ExportedCommandId`, `ExportedEventHeader`, every `to_json_value` /
  `from_json_value` / `payload_json` / `tagged_json_value` codec, the DAG
  helpers (`exported_events_in_topological_order`, `parents`,
  `command_ordinal_for_stream`), and the bespoke conflict detection
  (`event_conflicts`, `ConflictKey`, `ConflictPayload`, …).
- `EVENT_EXPORT_DIRECTORY` (`model/events/v1`) and `export_event_file_contents`.

## What is kept / adapted

- `ExportedEventBody` enum + `EventDraft` builders (in-memory lingua franca).
- eventcore command layer (`event_commands.rs`) — runs against `FileEventStore`.
- `ProjectedModel` and all Lean/Quint emission.

## Increment plan (TDD, keep build green per step where possible)

1. Cargo.toml: eventcore 0.9 (drop `sqlite` feat), add `eventcore-fs`, drop
   `eventcore-sqlite`. Adjust clippy `multiple_crate_versions`/`cargo` lint note
   if the exception is no longer required.
2. event_runtime.rs: introduce `FileEventStore` rooted at `model/events`;
   execute commands against it; delete SQLite/sync/repair. Update
   `event_runtime_external_tests.rs`.
3. shell.rs `ExportEvent` handler: execute command only (store writes files);
   remove custom JSON write; repoint store-existence checks to `model/events`.
4. events.rs read path: `EmcEvent` -> `ExportedEventBody` conversion; projection
   reads from store; fingerprint from stored events.
5. events.rs conflicts: forks via store; `resolve` via reconcile.
6. Delete dead `ExportedEvent` JSON/DAG/conflict code.
7. Rewrite tests: `event_log_export.rs` (behavioral, not JSON-file-text),
   `internal_semantic_tests.rs`, MCP/CLI tests touching events.
8. `just ci` green locally; open PR; shepherd.

## Fidelity finding (important)

`EmcEvent` is the stored event type, but one variant is **lossy** vs the rich
`ExportedEventBody` the projection historically consumed:
`EmcEvent::WorkflowConnected` omits `source_control` / `target_view`, which
`workflow_transition_record_from_connection` uses to emit navigation
transitions. Audit of all other variants: complete (slice facts wrap the full
`ExportedEventBody` inside `SliceFactEvent`). Resolution: **enrich**
`EmcEvent::WorkflowConnected` (+ `ConnectWorkflowCommand`) with those two
fields, then make the projection consume `EmcEvent` directly (flat fields), so
no fragile reconstruction of `NewWorkflow`/`WorkflowConnection` is needed.

Read-path design (revised): `ProjectedModel::from_events(Vec<EmcEvent>)`. For
`EmcEvent::SliceFactAdded { fact }`, expose `SliceFactEvent::to_event_body()`
and sub-match the slice-fact `ExportedEventBody`. `ExportedEventBody` then lives
only on the write path (`EventDraft` builders + the runtime draft→command map)
and inside `SliceFactEvent`.

## Status

- [done] Increment 1 — dependencies (eventcore 0.9, eventcore-fs, eventcore-types; dropped eventcore-sqlite).
- [done] event_runtime.rs rewritten onto FileEventStore (store root `model/events`, runtime lock under `locks/`, read_all_emc_events, list_forks, reconcile_with).
- [done] shell.rs ExportEvent handler no longer writes custom JSON; ensure_event_store wired.
- [in progress] Increment 4 — enrich EmcEvent::WorkflowConnected; rewrite projection to consume EmcEvent; rewire events.rs read path to the store.
- [done] Increment 4 — `EmcEvent::WorkflowConnected` enriched (source_control/target_view); `ProjectedModel::from_events` now consumes `Vec<EmcEvent>` (helpers `require`/`workflow_mut`/`slice_mut`/`apply_slice_fact_body`); events.rs read path reads from store; fingerprint hashes sorted per-event content digests.
- [done] Increment 5 (initial) — conflicts via `list_forks` + `reconcile_choose_branch` (conflict_id = forked stream id, chosen_event_id = branch transaction id).
- [done] Increment 6 — deleted dead `ExportedEvent`/`ExportedEventMeta`/`ExportedEventId`/`ExportedCommandId`/`ExportedEventHeader`/`ExportedEventFrontier`, the DAG + bespoke conflict detection, `export_event_file_contents`, dead helpers. `ExportedEventBody` + per-payload codecs KEPT (used by `SliceFactEvent` + projection slice-fact arms).
- **`cargo build` (lib+bin) is GREEN with zero warnings.**
- [done] Increment 7 — tests rewritten. `event_runtime_external_tests.rs` replaced with FileEventStore behavioral tests + a two-replica fork/reconcile test; `internal_semantic_tests.rs` dropped the obsolete ExportedEvent-JSON/EventStreamId-parsing tests and added the enriched WorkflowConnected/WorkflowTransitionEvidence fields; `event_log_export.rs` dropped format-coupled/SQLite/conflict tests and kept the behavioral `check_rebuilds_*` suite; `init_project.rs` artifact-only recovery now checks the committed `model/events/events/` store.
- Found & fixed a second lossy variant during testing: `EmcEvent::WorkflowTransitionEvidenceAdded` also dropped `source_control`/`target_view`; enriched it like WorkflowConnected.
- **Local gates green:** copyright-headers, `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo build`, and the full `cargo test` suite all pass. Next: push branch, open PR, shepherd CI/review. Broken: `internal_semantic_tests.rs` (uses `EventStreamId::try_new`, builds `EmcEvent::WorkflowConnected` missing new fields, exercises deleted JSON codecs), `event_runtime_external_tests.rs` (entirely SQLite-based — replace with FileEventStore behavior), `event_log_export.rs` (4099 lines: `*_exports_domain_event` assert custom JSON files → now assert store/projection behavior; `check_rebuilds_*` fixtures write `model/events/v1` JSON → rebuild via CLI or store appends; SQLite-cache tests delete; conflict tests rebuild forks via two-replica + git-union), cfg(test) mods in events.rs (from_json_str). Also MCP/CLI tests that may seed `model/events/v1`.

## Resume notes (post-compaction)

- Branch `feat/eventcore-fs-migration`. Source compiles; only tests remain.
- Store root: `model/events`; committed subdir is `model/events/events/` (JSONL). `.gitignore` written by the store covers tmp/index/.eventcore/locks/.lock.
- To build a project's events in a test, drive `emc` CLI mutating commands (they append to the store) OR open `eventcore_fs::FileEventStore` and `execute` commands. There is no more `model/events/v1/*.json`.
- `EMC_EVENT_STORE_PATH` / `XDG_STATE_HOME` env handling was REMOVED (store is always in-repo at `model/events`). Tests relying on those must drop them.
- Conflict UX changed: `emc list conflicts` reports forks (`conflict <stream> base <n> branches <tx-ids>`); `emc resolve conflict <stream-id> <branch-tx-id>` reconciles.
</content>
</invoke>
