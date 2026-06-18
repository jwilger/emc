<!-- Copyright 2026 John Wilger -->

# Plan: pure event-log artifact emit (the literal "never parsed back")

## Goal

Make the event log the single source of truth for **all** command decisions, with
Lean/Quint artifacts as write-only outputs **regenerated from the log and never
parsed back**. Inc 1 (PR #140) already delivered the runtime correctness:

- Command **decisions** (workflow lookups, slice membership, guards) read the
  event-log `ProjectedModel` via `projected_formal_workflow_graphs()`; no decision
  parses artifact text.
- Artifacts **self-heal**: every command regenerates them from the log, so on-disk
  drift / deletion cannot persist.

What is *not* yet literal: the regeneration **mechanism** still parses artifacts.

## Why the remaining work is large (corrected understanding)

`ProjectedModel::effects` (src/core/events.rs ~3970-4060) regenerates a slice by
emitting an empty module **shell** (`emit_slice_module`, src/core/emit/lean.rs:427
/ quint.rs) and then **replaying every fact back through the live handlers**:

```
self.event_definitions.into_iter().map(Effect::AddEventDefinitionFromSlice)
self.command_definitions.into_iter().map(Effect::AddCommandDefinitionFromSlice)
... (read_models, views, scenarios, outcomes, external_payloads, …)
```

Each `Effect::Add*FromSlice` handler calls an `add_*` builder
(src/core/formal_slice_facts.rs) / `add_project_*` builder
(src/core/formal_project_facts.rs) that **reads the current artifact text, parses
the target list, and splices the new record in** (`append_record`,
`append_record_if_missing`, `merge_or_append_named_record`). So:

- The handlers' "in-place mutation" is NOT vestigial — it is the shared emit
  mechanism for both live authoring and regeneration. (Deleting it empties the
  artifacts; verified: a shared event regenerated to `sliceEventDefinitions := []`.)
- Regeneration therefore parses the artifact it is building, incrementally.

To make emit literally never parse an artifact, the artifact production must
become a **pure function of the `ProjectedModel`** that renders each whole module
in one shot.

## Design

Add complete, pure emitters that render an entire module with every definition
list fully populated, reusing the existing per-record renderers
(`lean_event_definition_record`, `quint_command_definition_record`,
`lean_scenario_record`, … — already present in formal_slice_facts.rs ~2950-3360
and the project-root equivalents in formal_project_facts.rs):

- `emit_slice_module_complete(&ProjectedSlice) -> (lean: String, quint: String)`
- `emit_workflow_module_complete(&ProjectedWorkflow) -> (lean, quint)`
- `emit_project_root_complete(&ProjectedModel) -> (lean, quint)`

Each builds the module text directly: shell + every named list rendered by
mapping the projected facts through the per-record renderers and joining — no
text parse, no incremental splice, no disk read.

**Merge-by-name (PR #130) is preserved as in-memory grouping**, not text merge:
group `event_definitions` by event name and union their attributes; group
`command_definitions` by command name and union inputs/emitted-events/errors;
group views/read-models/external-payloads by name and union fields/controls. The
grouping rules mirror `COMMAND_CHILD_LIST_FIELDS` / `VIEW_CHILD_LIST_FIELDS` /
`ChildMergeMode` already encoded in formal_slice_facts.rs.

`ProjectedModel::effects` then emits one `WriteFile` per module via the complete
emitters — no `Add*FromSlice` replay.

Live mutating handlers collapse to: validate against `ProjectedModel` → append
event (`ExportEvent`) → (central post-step regenerate, already added pattern) →
emit the exact report lines inline (catalog already produced). They no longer
read artifacts or call `add_*`.

Once nothing replays facts for emit, the incremental machinery is dead and is
removed: `add_*` / `add_project_*` builders, the text splice helpers
(`append_record`, `append_record_if_missing`, `merge_or_append_named_record`,
`merge_list_record`, `record_field_value_span`, …), the `read_*_artifact_paths_and_contents`
readers, and every `parse_lean_project_*` / `parse_quint_project_*` parser that
exists only to read artifacts back.

## Correctness strategy (the hard part)

The ~700 `.contains(...)` artifact-text assertions across tests/ require the
complete emit to be **byte-identical** to today's replay-built output.

1. **Golden equivalence harness first.** Before wiring anything in, add a test
   that, for a battery of rich fixtures (multi-attribute events, multi-input
   commands, shared/observed events, navigation + command + event + outcome
   transitions, read-models/views with multiple fields, external payloads,
   automations, translations, board elements/connections, entry-lifecycle), builds
   artifacts the **current** way (replay) and the **new** way (complete emit) and
   asserts the two are byte-identical. This pins parity before switchover and
   localizes any per-record / ordering / digest discrepancy.
2. **Switch `ProjectedModel::effects` to the complete emitters**; run the full
   suite (`nix shell nixpkgs#jdk_headless -c cargo test`). Any diff is an emitter
   bug, caught by the existing assertions + the harness.
3. **Collapse handlers + delete dead code** only after emit parity holds.

Ordering and digests are the likely sources of byte diffs: the replay appends in
fact-arrival order; the complete emitter must reproduce that order
(`ProjectedSlice` vectors are already in arrival order) and recompute the same
slice/workflow/root digests from the same canonical content.

## Progress (branch `refactor/pure-emit-from-projection`)

### `check_project` is now log-sourced (the last decision-driving parse-back is gone)

`check_project` previously parsed the on-disk Lean/Quint **project root** back into
`ModeledProjectRootInventories` (the 25 `read_synchronized_project_*` readers in
`shell.rs`, each cross-checking Lean == Quint) and fed those parsed rows into the
completeness check. That was the last command **decision** that read an artifact
back. It now sources those inventories from the event log:

- `formal_project_facts::project_root_inventories_from_slices(slices)` reproduces
  every root inventory row directly from the projected slice facts. It builds each
  `NewProject*` via the same `from_*` constructors the replay uses, converts to the
  parsed `Project*` shape field-for-field (mirroring the `lean_*_record` renderers),
  and dedups each of the 25 lists by value (== the builders' `append_record_if_missing`).
- `events::projected_project_root_inventories()` loads the log and calls it;
  `ProjectedModel::project_root_inventories()` iterates workflows → slices in
  projection order.
- `Effect::CheckCurrentProject` calls it; the 25 `read_synchronized_project_*`
  readers and all `parse_(lean|quint)_project_*` wrappers are deleted.

**Digest bug fixed:** `layout::digest_data_flows` and `digest_command_errors` were
the only two of the 25 root-inventory digest functions that did **not** sort their
rows, while the replay's digest (`formal_project_facts`) sorts all of them. With the
parsed-artifact source they happened to agree (the parsed list order matched), but a
log-sourced check exposed the order dependence as `modelDigest` drift. Both now
`sort_unstable()` like the other 23, making the digest a true order-independent
canonical-content hash. Gate: full `check_project` + `add_formal_slice_facts` suites.

### Regeneration is now a pure complete-emit (self-heal / verify-read path)

The projection (`ProjectedModel::effects` / `ProjectedSlice::effects`) no longer
replays `Add*FromSlice` to splice the generated artifacts. It renders each module
in one shot from the event log:

- `ProjectedSlice::effects` emits the slice shell then fills it with
  `populate_slice_lists` (the parity-proven slice assembler) for both flavors.
- `ProjectedModel::effects` emits the root shell (`emit_project_root_shells`) then
  fills it with `layout::populate_project_root_modules` — a new root assembler that
  string-replaces the 25 `:= []` data lists (via the existing `lean_model_*_list` /
  `quint_model_*_list` renderers), the 25 `.length = 0` count theorems, the Quint
  `modelDataFlowCount`, and the `modelDigest` (recomputed with the full inventories
  from `project_root_inventories_from_slices`). No `read_*_artifact`, no parse.

So every entrypoint that regenerates artifacts (the self-heal that runs on `check`,
`list`, `verify`, and as the pre-step of every command) is now a pure function of
the log. Verified: `check_project` (15) green, rich-model `emc check` green;
`add_formal_slice_facts` (50) is the byte-parity gate.

Remaining for full purity (Increment 3): the **live mutating commands**
(`add slice`, `add scenario`, …) still emit+splice their own artifacts
(`workflow.rs` / `slice.rs` / the `Add*FromSlice` handlers' `add_*` / `add_project_*`
calls) — output assembly that reads the artifact to append, not a decision. Collapse
them to validate-against-log + `ExportEvent` + pure regenerate, then delete the
`add_*` / `add_project_*` builders, splice helpers, and `read_*_artifact` readers.

### Pure slice list renderers (earlier work)

Pure slice list renderers are implemented and **parity-proven byte-identical**
to the incremental merge builders (commits on this branch), each guarded by a
test that folds the same facts through `merge_or_append_named_record` /
`append_record(_if_missing)` and asserts equality:

- `render_append_list` / `render_dedup_list` — cover every Append / AppendIfMissing
  reference + scalar list (outcomes, scenarios, automations, translations,
  data-flows, board elements/connections, streams, event/command/read-model/view
  reference lists).
- `render_slice_event_definitions` — events (merge: union `attributes`).
- `render_slice_read_model_definitions` — read models (merge: union `fields`).
- `render_slice_external_payload_definitions` — external payloads (merge: union
  `fields`; reproduces the final-field trailing-space quirk: a merged record
  closes `]}`, a single-field record `] }`).
- `render_slice_command_definitions` — commands (merge: accumulate `inputs`,
  union `emittedEvents`/`observedStreams`/`errors` via `union_if_missing`).
- `render_slice_view_definitions` — views (merge: accumulate `fields`/`controls`,
  union `readModels`/`sketchTokens`/`localStates`/`filters`; final-field quirk on
  `filters` when a later member contributes non-empty filters).

Helper `group_named` does first-seen-order grouping; `union_if_missing` takes the
first member verbatim then appends-if-missing — matching the builder exactly.

Full slice-list inventory (from `emit_slice_module` `:= []` placeholders) is
enumerated and every list maps to one of the above. **The hardest correctness
risk — merge byte-identity — is retired.**

**Slice assembler DONE + proven** (`populate_slice_lists` + `SliceModuleFacts` in
formal_slice_facts.rs): fills every one of the 19 slice lists of a freshly-emitted
shell directly from the projected facts (replacing each `:= []` placeholder),
wiring each list to its proven renderer (5 merges, append lists, dedup reference
lists, and the views→translations→automations concat for
`sliceReferencedCommands`). Golden harness `populate_slice_lists_matches_builder_replay`
folds the live builders over a synthetic shell in `effects()` order and asserts
byte-identical for both flavors. `RecordFlavor` is now `pub(crate)`.

### Corrected understanding of the switch (verified empirically)

- The **workflow module** (`emit_lean_workflow_module` in events.rs) is ALREADY a
  pure complete emitter — it takes `WorkflowModuleData` and renders in one shot, no
  replay. Nothing to do there.
- The **project-root module** is NOT pure. `emit_lean_project_root`
  (project.rs:145) emits the root as a SHELL with every definition list `:= []`
  (and `.length = 0` theorems) — but the `ProjectedSlice::effects` `Add*FromSlice`
  replay populates BOTH the slice module AND the root: interpreting each
  `Add*FromSlice` effect runs the slice `add_*` builder AND the `add_project_*`
  builder (formal_project_facts.rs, shell.rs:462-859), splicing the definition into
  the root's `modelCommands`/`modelEvents`/`modelViewControls`/… lists.
- **Therefore the switch is ATOMIC: slice + root together.** Wiring
  `populate_slice_lists` into `ProjectedSlice::effects` alone (and dropping the
  `Add*FromSlice` replay) populates slices correctly (proven — the slice content
  tests pass) but leaves the root's definition lists empty → 12 root-inventory
  tests in `add_formal_slice_facts` fail ("Lean project root must inventory authored
  …"). A trial wire of just the slice side confirmed exactly this and was reverted.

Remaining work, in order:
1. Build the **project-root assembler** `populate_project_root_lists` in
   formal_project_facts.rs, mirroring the 13 `add_project_*` builders — but the root
   aggregates across ALL slices/workflows and keys rows by (workflow, slice, …)
   coordinates, and the shell's `.length = N` / `native_decide` theorems must match
   the populated counts. Parity-prove each list + a golden harness folding the live
   `add_project_*` builders, exactly as done for the slice side.
2. **Atomic switch** in `ProjectedModel::effects` / `ProjectedSlice::effects`: emit
   each shell, populate slice lists via `populate_slice_lists` and root lists via
   `populate_project_root_lists`, drop the `Add*FromSlice` replay. Full
   ~700-assertion suite (incl. Lean `verify_project`) is the gate.
3. Delete the dead `add_*`/`add_project_*` builders + splice helpers + `parse_*`
   readers once nothing replays.

Slice side fully proven; the root assembler (step 1) is the large remaining piece.

## Increments (each its own PR, suite green + clippy clean)

1. **Pure emitters + golden equivalence harness** — implement
   `emit_*_complete`, prove byte-identical to replay on fixtures. Not yet wired
   into the projection. (Largest single piece; pure-additive, low risk to prod.)
2. **Switch the projection to complete emit** — `ProjectedModel::effects` uses the
   complete emitters; drop the `Add*FromSlice`-replay emit. Full suite green.
3. **Collapse mutating handlers** — validate + `ExportEvent` + inline reports;
   remove artifact reads. (Group by handler family; reports per the catalog.)
4. **Delete dead incremental/parse code** — `add_*`, `add_project_*`, splice
   helpers, `read_*_artifact_paths_and_contents`, `parse_*_project_*`.

## Risk / cost

- **High regression surface**: byte-identity vs ~700 assertions; the digest and
  ordering parity is exacting. The golden harness (Inc 1 of this plan) is the
  mitigation — it fails loudly and locally on any divergence.
- **Large**: touches the emit core (formal_slice_facts.rs, formal_project_facts.rs,
  emit/lean.rs, emit/quint.rs, events.rs projection, shell.rs handlers) and
  removes a lot of code. Multiple PRs, slow Lean CI per step.
- **Value over Inc 1**: purity / single emit path / dead-code removal. No new
  runtime correctness beyond Inc 1 (decisions already log-sourced; artifacts
  already self-heal deterministically from the log).

## Recommendation

Land Inc 1 (#140) first. Then execute this plan increment-by-increment, gated by
the golden equivalence harness, only if the purity/maintainability win justifies
the regression risk against the emit core.
