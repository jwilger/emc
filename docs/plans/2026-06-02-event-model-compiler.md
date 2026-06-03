# Event Model Compiler Full-System Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build EMC as a Rust executable that lets users create, modify, validate, browse, and serve business event models through CLI and MCP without requiring Lean4 or Quint knowledge.

**Architecture:** EMC uses a functional-core/imperative-shell design. All I/O is represented as step/trampoline variant effects interpreted only at the outer shell. The core owns semantic model transformations, emits synchronized Lean4 and Quint canonical representations, derives browser data, and validates drift between all artifacts.

**Tech Stack:** Rust 2024, Clap, rmcp, serde, nutype 0.7.0, Lean4/Lake, Quint, Nix flakes, Vite/React browser assets, and generated browser data.

---

## Non-Negotiable Engineering Rules

- Warnings are errors everywhere: `RUSTFLAGS='-Dwarnings'` for build, test, fmt checks, clippy, and CI.
- Maintain a strict Rust lint posture: enumerate `clippy::all` lints, keep high-signal lints at `forbid`, and allow only documented exact-lint `deny` carve-outs when third-party macros require them.
- Use functional-core/imperative-shell architecture. The model core must be deterministic and side-effect free.
- All I/O must go through a step/trampoline variant effect pattern. Production code may describe effects but may not perform file, process, network, stdio, clock, or environment I/O inside core modules.
- Use only semantic data types inside the system. Primitives and structural DTOs are allowed only at I/O boundaries.
- Practice parse-don't-validate: boundary data is immediately parsed into semantic types and all downstream code accepts only those semantic types.
- Use `nutype` for semantic data types unless a richer hand-written type is required for algebraic behavior. The current crate version is `0.7.0`.
- Do not add exceptions to these rules without a new checked-in decision record.

## System Shape

- `emc` is the single user-facing executable.
- CLI subcommands: `init`, `list`, `show`, `add`, `update`, `connect`, `validate`, `verify`, `check`, `generate site`, and `mcp`.
- MCP transports:
  - `emc mcp stdio` for local editor/agent clients.
  - `emc mcp http` for container/network use with Origin validation, localhost-safe defaults, and authentication when exposed beyond local host.
- Nix builds a single distributable unit containing `emc`, Lean/Lake, Quint, and generated browser assets. The release path includes a local app closure and a Docker-compatible image.

## Canonical Model

- Lean4 and Quint are dual canonical model representations for the actual business event model, not generic event-modeling meta-rules.
- Every EMC mutation rewrites both Lean4 and Quint deterministically from the same semantic operation.
- EMC reads both representations into a normalized semantic graph, computes stable digests, and fails `emc check` if Lean4, Quint, or generated browser data drift.
- Lean4 owns static proof obligations for model structure and invariants.
- Quint owns executable state/transition behavior and temporal checks.
- External Lean4 and Quint tools are acceptable, but they must be pinned and hidden behind the `emc` executable and Nix packaging.

## Event Model Rule Coverage

- Check in the current event-model rule surface as acceptance coverage:
  - 159 validator scenarios.
  - 9 review-gate scenarios.
  - 11 browser scenarios.
  - 6 runner/meta scenarios.
- Preserve browser behavior for layout, styling, workflow selector behavior, timeline composition, branch cards, source chains, control effects, navigation targets, and review overlays.
- Make visible project title/branding configurable; do not hard-code unrelated product labels in EMC projects.
- Generate the same browser data shape: `data/index.json`, `data/workflows/*.eventmodel.json`, and `data/slices/*.eventmodel.json`.

## Implementation Sequence

## Progress Notes

- 2026-06-02: `emc check` now rejects workflow drift between browser JSON and generated Lean4/Quint artifacts for workflow identity fields, composed slice lists, and navigation transition lists. It also rejects workflows that reference missing browser slice artifacts. This is deterministic artifact synchronization coverage, not yet the full normalized semantic graph drift check described above.
- 2026-06-02: Review-gate fixture parity is checked in. `emc review gate --workflow <slug>` and MCP `review_gate` enforce current structured clean reviews for the workflow slug, model digest, required categories, mandatory findings, and clean follow-up review after corrected findings.
- 2026-06-02: Event-model Gherkin fixtures are checked in with scenario-count guardrails for validator, review-gate, browser, and runner/meta suites.
- 2026-06-02: `emc validate` and the MCP `validate_event_model` tool accept a single `.eventmodel.json` file target as well as a directory target.
- 2026-06-02: `emc check` rejects browser workflow data files that are present under `model/browser/data/workflows` but missing from `model/browser/data/index.json`.
- 2026-06-02: Workflow browser JSON emitted by `emc add workflow` includes the full event-model top-level shape, so `emc validate` reaches workflow composition diagnostics instead of failing on missing required sections.
- 2026-06-02: Slice files emitted by `emc add slice` use the slice slug as the file stem, matching workflow step slugs so composed workflow validation reaches referenced slice diagnostics.
- 2026-06-02: `emc check` rejects browser slice data files that are present under `model/browser/data/slices` but not referenced by any workflow composition.
- 2026-06-02: `emc check` rejects generated Lean4 and Quint workflow artifacts that are not owned by the initialized project module or a workflow listed in browser data.
- 2026-06-02: Generated Lean4 and Quint workflow artifacts now include semantic slice detail records for composed business slices: slug, name, type, and description. `emc check` rejects slice-detail drift between browser workflow data and the canonical artifacts.
- 2026-06-02: `emc update workflow` now preserves existing workflow composition, slice file references, transitions, and canonical Lean4/Quint slice details while rewriting the workflow description.
- 2026-06-02: Local mutation testing guardrails are available through explicit `just mutants-diff`, `just mutants-core`, and `just mutants-full` recipes. They are intentionally outside `just ci` so mutation testing can run as a regular manual engineering gate without slowing every local/hosted CI pass.
- 2026-06-02: `emc connect workflow` now supports command, event, navigation, external-trigger, and workflow-exit transitions through the same CLI/MCP semantic mutation path, and generated Lean4/Quint transition labels preserve those transition kinds.
- 2026-06-02: Generated Lean4 and Quint workflow artifacts now include a slice-detail completeness obligation tying the workflow slice list to generated slice metadata. Lean proves the length equality by reduction, and Quint exposes the same boolean as a named verification value.
- 2026-06-02: `emc verify` now asks Quint to verify the emitted `workflowIdentityStable` and `workflowSliceDetailsComplete` invariant values explicitly through both CLI and MCP verification paths.
- 2026-06-02: `emc check` now rejects Lean4 and Quint invariant drift for generated workflow slice-detail completeness obligations, including stale or duplicate verified Quint invariant declarations.
- 2026-06-02: `emc check` now treats formal artifact digest markers as canonical declarations, rejecting generated Lean4 or Quint artifacts that retain the expected digest alongside stale duplicate digest metadata.
- 2026-06-02: `emc check` now rejects Lean4 namespace drift and Quint module declaration drift for generated workflow artifacts, so formal artifact ownership must match the modeled workflow module as well as the filename.
- 2026-06-02: `emc check` now rejects Lean4 theorem drift and Quint invariant drift for generated workflow identity obligations, preventing trivially true or stale formal identity assertions from passing artifact synchronization.
- 2026-06-02: `emc check` now rejects stale Lean4 closing module declarations, so both the opening namespace and closing `end` ownership must match the modeled workflow module.
- 2026-06-02: Browser index parsing now rejects duplicate workflow paths before they become semantic workflow layouts, so `emc check` cannot accept duplicated workflow entries as a synchronized model.
- 2026-06-02: Browser index parsing now rejects duplicate workflow names before semantic conversion, preventing separate browser workflow paths from colliding on the same generated Lean4 and Quint module identity.
- 2026-06-02: Browser index parsing now rejects duplicate semantic workflow slugs after path parsing, preventing distinct raw workflow paths from normalizing to the same modeled workflow identity.
- 2026-06-02: `emc check` now rejects referenced browser slice filenames that do not round-trip through the semantic slice slug parser to the canonical generated filename, preventing path-level slice identity drift from bypassing formal artifact checks.
- 2026-06-02: `emc validate` now applies the same semantic slice filename canonicalization at the raw workflow boundary before corpus validation, so noncanonical referenced slice paths cannot be interpreted as a different composed slice identity.
- 2026-06-02: Workflow-targeted shell reads now require the semantic workflow slug to be present in the browser index before reading workflow JSON, so `show`, update, connect, add-slice, and MCP callers cannot operate on stale unindexed workflow files.
- 2026-06-02: `emc verify` now maps failed Lean4 or Quint process exits to actionable EMC diagnostics that name the verification surface and tell users to run `emc check` before retrying verification.
- 2026-06-02: MCP `tools/call` now returns JSON-RPC error responses for tool execution failures, including failed formal verification, instead of terminating the stdio or HTTP transport with a shell error.
- 2026-06-02: Project path arguments now reject absolute paths and parent-directory traversal at the semantic `ProjectPath` boundary, keeping validation and site generation scoped to deterministic project-local paths across CLI and MCP entrypoints.
- 2026-06-02: `nix flake check` now builds the Docker-compatible EMC container image as a named check, so the packaged runtime closure is exercised alongside the package and command smoke gate.
- 2026-06-02: MCP HTTP now returns a `400 Bad Request` response for malformed JSON-RPC bodies instead of terminating the HTTP transport with a shell error.
- 2026-06-02: Generated formal artifact digest markers now include workflow name, slug, and description, so `emc check` rejects stale Lean4 or Quint digest metadata after semantic workflow identity fields change.
- 2026-06-02: Generated formal artifact digest markers now include workflow slice details and transition labels as well as identity fields. `emc check` derives the expected digest from browser workflow JSON before comparing Lean4 and Quint artifacts, moving the digest guardrail closer to a normalized workflow graph check.
- 2026-06-03: Generated Lean4 and Quint workflow artifacts now represent business workflow transitions as structured source, target, kind, and trigger records instead of opaque transition strings. `emc check` derives the same structured records from browser workflow JSON when checking formal artifact drift.
- 2026-06-03: Structured workflow transitions are now part of the formal verification surface: Lean4 emits a named transition-structure theorem, Quint emits a named `workflowTransitionsStructured` invariant, `emc check` rejects stale copies of both declarations, and `emc verify` asks Quint to verify the transition invariant with the existing workflow invariants.
- 2026-06-03: Workflow update mutations now parse browser workflow JSON into a semantic `WorkflowDocument` type before deriving workflow identity, slice details, transitions, and rewritten file contents. A static architecture guardrail prevents `src/core/workflow.rs` from directly manipulating raw `serde_json::Value`.
- 2026-06-03: Slice mutations now use the same semantic `WorkflowDocument` API to append slice-file references and workflow steps before deriving synchronized browser, Lean4, and Quint artifacts. Static architecture guardrails now prevent both `src/core/workflow.rs` and `src/core/slice.rs` from directly manipulating raw JSON values.
- 2026-06-03: Workflow connection mutations now use semantic `WorkflowDocument` transition additions to append command, event, navigation, external-trigger, and workflow-exit transitions before deriving synchronized artifacts. Static architecture guardrails now prevent workflow, slice, and connection mutation modules from directly manipulating raw JSON values.
- 2026-06-03: Project initialization now writes the same Quint workflow invariant list that `emc verify` asks Quint to check: identity stability, slice-detail completeness, and structured transitions. Init coverage asserts the exact root `quint.json` content so the project skeleton cannot silently lag behind the verification surface.
- 2026-06-03: Formal artifact digest derivation now parses browser workflow JSON through the semantic `WorkflowDocument` type instead of maintaining a separate raw-JSON interpretation path. A static architecture guardrail prevents `src/core/digest.rs` from directly manipulating raw JSON values.
- 2026-06-03: Check-time Lean and Quint transition markers now derive transition labels from the same semantic `WorkflowDocument` parser used by mutations and formal digests. A shell guardrail prevents reintroducing duplicate transition-label helper semantics in the interpreter.
- 2026-06-03: Check-time Lean and Quint slice-list and slice-detail markers now derive from semantic `WorkflowSliceDetail` values parsed by `WorkflowDocument`. A shell guardrail prevents raw workflow step field access for slice identity, name, type, or description in marker derivation.
- 2026-06-03: Shell validation, review-digest, and check-time slice-file traversal now use semantic `WorkflowSliceFileReference` values from `WorkflowDocument::slice_files()`. A shell guardrail prevents raw `slice_files` field parsing from reappearing outside the semantic document parser.
- 2026-06-03: Validation-time referenced slice-file checks now use semantic `WorkflowSliceFileReference` values from `WorkflowDocument::optional_slice_files()`, preserving existing validator diagnostics for non-object workflow documents while preventing raw `slice_files` parsing in validation code.
- 2026-06-03: Shell browser-index workflow path checks now reuse the boundary parser and `ModeledWorkflowLayout::browser_data_path()` instead of duplicating raw `workflows[].path` traversal. A shell guardrail prevents direct browser-index workflow path parsing from reappearing in the interpreter.
- 2026-06-03: Review-gate shell checks now parse review JSON through `ReviewRecordDocument` and compare semantic workflow slugs, artifact digests, statuses, category names, and mandatory finding digests. A shell guardrail prevents direct review-record field parsing from returning to the interpreter.
- 2026-06-03: Generic shell JSON-object checks now use `JsonObjectDocument` instead of parsing `serde_json::Value` in the interpreter. A shell guardrail prevents direct raw JSON value parsing from reappearing in shell checks.
- 2026-06-03: Browser workflow main-path composition now derives entry/main step names from `WorkflowDocument::main_path_step_names()` instead of duplicating raw workflow-step traversal in `src/core/browser.rs`. A browser guardrail prevents the raw main-path helper from returning.
- 2026-06-03: Browser branch-card composition now derives branch names and labels from `WorkflowDocument::branch_details()`, including alternate-outcome labels, instead of duplicating raw workflow-step traversal in `src/core/browser.rs`. A browser guardrail prevents the raw branch helper from returning.
- 2026-06-03: Browser transition-card composition now derives transition names, sources, targets, kinds, and labels from `WorkflowDocument::transition_details()` instead of duplicating raw workflow transition traversal in `src/core/browser.rs`. A browser guardrail prevents the raw transition-card helpers from returning.
- 2026-06-03: Browser review overlays now derive step names, statuses, and missing rules from `WorkflowDocument::review_overlay_details()` instead of parsing raw workflow review diagnostics in `src/core/browser.rs`. A browser guardrail prevents direct review-diagnostics parsing from returning.
- 2026-06-03: Browser lane composition now derives board lane IDs from `BrowserDataDocument::board_lane_ids()` instead of parsing raw `board.lanes` in `src/core/browser.rs`. A browser guardrail prevents direct lane traversal from returning to browser composition.
- 2026-06-03: Browser event-element composition now derives board event element names from `BrowserDataDocument::event_element_names()` instead of parsing raw board slice elements in `src/core/browser.rs`. A browser guardrail prevents direct event-element traversal from returning to browser composition.
- 2026-06-03: Browser error-recovery composition now derives command error names and source screens from `BrowserDataDocument::error_recovery_details()` instead of parsing raw control error handling in `src/core/browser.rs`. A browser guardrail prevents direct error-handling traversal from returning to browser composition.
- 2026-06-03: Browser command-definition composition now derives command names, owning slices, source controls, and section labels from `BrowserDataCorpus::command_definition_details()` instead of resolving raw command/view/slice fields in `src/core/browser.rs`. A browser guardrail prevents direct command-definition traversal from returning to browser composition.
- 2026-06-03: Browser view-definition composition now derives field source chains and control effects from `BrowserDataCorpus::view_definition_details()` instead of traversing raw view/read-model/event fields in `src/core/browser.rs`. A browser guardrail prevents direct view-definition traversal from returning to browser composition.
- 2026-06-03: Generated Lean4 and Quint transition-structure obligations now assert that each workflow transition carries non-empty source, target, kind, and trigger fields instead of accepting tautological transition-list self-comparisons. `emc check` rejects stale transition-structure declarations that still use the old tautologies.
- 2026-06-03: Generated Lean4 workflow transitions now use named `WorkflowTransition` records instead of positional tuples, and the Quint transition-structure invariant now uses a pure list `select(...).length()` expression instead of a braced action block. Check-time transition markers were updated to match the formal emitters, with a guardrail against reintroducing Lean transition tuple marker helpers.
- 2026-06-03: `emc add slice` now emits per-slice Lean4 and Quint modules under `model/lean/slices` and `model/quint/slices` so business slices are represented formally as well as in workflow composition. `emc check` rejects workflows whose referenced business slices are missing corresponding Lean4 or Quint slice artifacts.
- 2026-06-03: `emc check` now verifies canonical declarations inside per-slice Lean4 and Quint artifacts for slice module identity, name, slug, kind, and description, so stale formal slice modules cannot pass by merely existing at the expected path.
- 2026-06-03: `emc verify` now runs Lean4 and Quint verification for generated business slice modules as well as workflow modules. Quint slice modules expose a generated `sliceIdentityStable` invariant plus deterministic `init` and `step` actions so the packaged pinned Quint runtime verifies actual slice artifacts.
- 2026-06-03: Generated Lean4 and Quint business slice modules now carry deterministic slice digest markers derived from semantic slice name, slug, kind, and description. `emc check` rejects stale or duplicate slice digest metadata in formal slice artifacts.
- 2026-06-03: `emc check` now rejects stale Lean4 `sliceIdentityIsStable` theorem declarations and Quint `sliceIdentityStable` invariant declarations in generated business slice modules, so slice formal obligations must match the semantic slice identity instead of only carrying matching fields and digests.
- 2026-06-03: `emc check` now rejects unmodeled Lean4 and Quint business slice modules under `model/lean/slices` and `model/quint/slices`, deriving the allowed formal slice artifact set from semantic workflow slice details instead of accepting extra orphan formal modules.
- 2026-06-03: `emc add slice` now rejects slice names that would collide on the same generated Lean4/Quint module name, preventing a mutation from overwriting an existing business slice formal artifact and leaving `emc check` to discover the invalid state afterward.
- 2026-06-03: `emc add workflow` now rejects workflow names that would collide on the same generated Lean4/Quint module name, preventing one business workflow mutation from overwriting another workflow's formal artifacts.
- 2026-06-03: `emc add slice` now rejects duplicate semantic slice slugs before planning writes, preventing duplicate workflow steps or overwritten browser/formal slice artifacts from being created by mutation commands.
- 2026-06-03: `emc add workflow` now rejects duplicate semantic workflow slugs before planning writes, preventing mutation commands from replacing an existing workflow's browser and formal artifacts.
- 2026-06-03: `emc connect workflow` now rejects duplicate semantic workflow transition labels before planning writes, so repeated mutation requests cannot append duplicate browser transitions or rewrite Lean4/Quint workflow artifacts with duplicate transition records.
- 2026-06-03: `emc connect workflow` now rejects transitions to unknown in-workflow slice targets before planning writes, preventing mutation commands from generating browser, Lean4, and Quint artifacts that validation would later reject as unreachable or unresolved.
- 2026-06-03: `emc add slice` now rejects workflow document name drift against the indexed workflow identity before planning writes, preventing a corrupted browser workflow document from steering new slice mutations into mismatched Lean4/Quint workflow modules.
- 2026-06-03: `emc connect workflow` now rejects workflow document name drift against the indexed workflow identity before planning writes, preventing a corrupted browser workflow document from steering transition mutations into mismatched Lean4/Quint workflow modules.
- 2026-06-03: `emc add slice` now rejects workflow document description drift against the indexed workflow description before planning writes, preventing a corrupted browser workflow document from changing formal workflow digests through an unrelated slice mutation.
- 2026-06-03: `emc connect workflow` now rejects workflow document description drift against the indexed workflow description before planning writes, preventing a corrupted browser workflow document from changing formal workflow digests through an unrelated transition mutation.
- 2026-06-03: MCP HTTP origin validation now compares the request Origin against the request Host authority when present, so authenticated clients can reach a server bound to `0.0.0.0` through their actual host while cross-origin requests still fail deterministically.
- 2026-06-03: The Nix package smoke gate now exercises a packaged MCP HTTP `tools/call` request for `check_project`, proving the packaged HTTP transport can invoke EMC tools instead of only proving that the transport initializes.
- 2026-06-03: MCP stdio now streams requests line-by-line and flushes each JSON-RPC response immediately, so editor and agent clients do not need to close stdin before receiving tool responses.
- 2026-06-03: Formal workflow transition emission, digesting, duplicate detection, and check-time marker derivation now use semantic `WorkflowTransitionRecord` values instead of reparsing canonical transition-label strings back into source, target, kind, and trigger fields.
- 2026-06-03: Generated browser sites now pass project-specific branding into the bundled browser runtime and the browser asset no longer carries unrelated product labels in its visible header.
- 2026-06-03: `emc verify` now checks initialized project root formal artifacts before generated workflow and slice artifacts: Lean4 root modules run through `lake env lean`, and Quint root modules run through `quint typecheck` because root modules do not declare workflow invariants.
- 2026-06-03: `emc check` now rejects initialized project root formal artifact drift: root Lean4 namespace/end declarations, root Quint module declarations, and root Quint verification config must match the initialized project module and invariant surface.
- 2026-06-03: `emc check` now rejects Lean project verification config drift: generated `lakefile.lean` and `lean-toolchain` contents must match the deterministic Lake package and pinned Lean4 toolchain emitted by initialization.
- 2026-06-03: `emc check` now rejects project manifest drift: generated `emc.toml` project name, Lean module, and Quint module declarations must remain canonical for the initialized project root.
- 2026-06-03: `emc init` now creates empty formal slice artifact directories under `model/lean/slices` and `model/quint/slices`, and `emc check` requires their keep files as part of the deterministic project skeleton.
- 2026-06-03: `emc check` now rejects missing Quint module closing declarations for generated project root, workflow, and business slice artifacts, so malformed formal modules fail deterministic synchronization checks before `emc verify`.
- 2026-06-03: `emc validate` now rejects duplicate JSON object keys before event-model parsing, sharing the deterministic JSON key guardrail used by `emc check` so boundary parsing cannot silently accept last-key-wins model data.
- 2026-06-03: `emc generate site` now replaces generated browser data instead of overlaying it, so regenerating a site removes stale workflow or slice data files from previous generations.

### Task 1: Guardrails and Project Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/core/mod.rs`
- Create: `src/core/effect.rs`
- Create: `src/core/project.rs`
- Create: `tests/init_project.rs`
- Create: `tests/lint_policy.rs`
- Create: `justfile`
- Modify: `flake.nix`

- [ ] Write failing tests for `emc init` producing a deterministic project effect plan without touching the filesystem in core code.
- [ ] Add Rust manifest with `nutype = "0.7.0"`, strict lints, and CLI/test dependencies.
- [ ] Add just recipes using `RUSTFLAGS='-Dwarnings'`.
- [ ] Implement only enough step/trampoline infrastructure and shell interpretation for `emc init`.
- [ ] Verify `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `just ci`.

### Task 2: Semantic Type Layer

**Files:**
- Create: `src/core/types.rs`
- Create: `src/io/dto.rs`
- Create: `tests/semantic_types.rs`

- [ ] Define `nutype` semantic types for project names, model names, workflow slugs, slice slugs, definition names, file paths, digests, Lean module names, and Quint module names.
- [ ] Keep raw primitives only in DTO modules and CLI argument structs.
- [ ] Add parse-don't-validate tests proving invalid boundary data cannot enter core APIs.
- [ ] Add lint tests that reject primitive-bearing public core APIs except explicitly marked boundary modules.

### Task 3: EMC Project Layout

**Files:**
- Create: `src/core/layout.rs`
- Create: `tests/project_layout.rs`

- [ ] Define deterministic project paths for Lean4, Quint, browser data, generated site, review records, and compatibility fixtures.
- [ ] Make `emc init` create the full empty project layout through interpreted effects.
- [ ] Ensure repeated init is deterministic and reports existing files without corrupting content.

### Task 4: Workflow Mutation Coverage

**Files:**
- Create/modify: `src/core/workflow.rs`
- Create/modify: `src/core/slice.rs`
- Create/modify: `src/core/connection.rs`
- Create/modify: `tests/add_workflow.rs`
- Create/modify: `tests/add_slice.rs`
- Create/modify: `tests/connect_workflow.rs`

- [ ] Create workflows through EMC commands and MCP tools.
- [ ] Create slices through EMC commands and MCP tools.
- [ ] Connect workflow steps through EMC commands and MCP tools.
- [ ] Emit Lean4, Quint, and browser JSON from the same semantic mutation.
- [ ] Assert stable digests and no primitive DTO leakage beyond command boundaries.

### Task 5: Validation Rule Port

**Files:**
- Create: `src/core/validation/`
- Create: `tests/features/event_model_validator/`
- Create: `tests/validation_rules.rs`

- [ ] Check in validator Gherkin as EMC test fixtures.
- [ ] Implement validation as pure functions over semantic model types.
- [ ] Preserve current diagnostics where tests depend on user-facing messages.
- [ ] Fail validation on Lean/Quint/browser drift.

### Task 6: Lean4 and Quint Emission

**Files:**
- Create: `src/core/emit/lean.rs`
- Create: `src/core/emit/quint.rs`
- Create: `tests/emit_lean.rs`
- Create: `tests/emit_quint.rs`

- [ ] Generate deterministic Lean4 modules for the actual business model.
- [ ] Generate deterministic Quint modules for the same model.
- [ ] Add golden tests for workflows, slices, transitions, invariants, and proof/model-check entrypoints.
- [ ] Ensure generated files are stable across repeated runs.

### Task 7: Verification Shell

**Files:**
- Create: `src/shell/verify.rs`
- Create: `tests/verify_shell.rs`

- [ ] Interpret process effects for Lean/Lake and Quint.
- [ ] Hide tool details behind `emc verify`.
- [ ] Surface actionable diagnostics without requiring the user to read Lean4 or Quint output first.
- [ ] Use pinned Nix tools in CI and release packaging.

### Task 8: Browser Generation

**Files:**
- Create: `browser/`
- Create: `src/core/browser_data.rs`
- Create: `tests/features/event_model_browser/`
- Create: `tests/browser_generation.rs`

- [ ] Preserve the browser UI behavior needed for event-model browsing.
- [ ] Make title and project branding configurable.
- [ ] Generate stable `data/index.json`, workflow JSON, and slice JSON.
- [ ] Preserve composed workflow loading, canonical lanes, timeline ordering, branch rendering, source chains, control effects, navigation targets, and review overlays.

### Task 9: Review Gate

**Files:**
- Create: `src/core/review_gate.rs`
- Create: `tests/features/event_model_review_gate/`
- Create: `tests/review_gate.rs`

- [ ] Implement review-gate semantics from the checked-in rule fixtures.
- [ ] Store review records by workflow slug and model digest.
- [ ] Block advancement on stale clean reviews, missing categories, non-clean categories, and mandatory findings.
- [ ] Require a clean follow-up review after model changes that address findings.

### Task 10: MCP Server

**Files:**
- Create: `src/mcp/`
- Create: `tests/mcp_stdio.rs`
- Create: `tests/mcp_http.rs`

- [ ] Expose read, validate, verify, generate, and mutation tools over MCP.
- [ ] Route MCP operations through the same semantic command core as CLI operations.
- [ ] Support stdio and Streamable HTTP transports.
- [ ] Enforce HTTP Origin checks and authentication policy for non-local exposure.

### Task 11: Packaging and CI

**Files:**
- Modify: `flake.nix`
- Create: `.github/workflows/ci.yml`
- Create: `tests/package_smoke.rs`

- [ ] Build `emc` with Nix.
- [ ] Build a Docker-compatible image containing the full runtime closure.
- [ ] Run `emc check`, `emc generate site`, `emc mcp stdio`, and `emc mcp http` smoke tests from the package.
- [ ] Keep CI using the same strict warnings-as-errors and lint policy as local development.

## Completion Evidence

- `just ci` passes.
- `emc check` proves Lean4, Quint, and browser data are synchronized.
- `emc verify` runs Lean4 and Quint verification through pinned tools.
- All validator, review-gate, browser, and meta scenarios pass.
- Nix app and container image build and pass smoke tests.
- Public core APIs expose semantic types, not primitives or structural DTOs.
- Static guardrails prove I/O only appears in shell/interpreter modules.
