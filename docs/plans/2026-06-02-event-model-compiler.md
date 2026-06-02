# Event Model Compiler Full-System Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build EMC as a Rust executable that lets users create, modify, validate, browse, and serve business event models through CLI and MCP without requiring Lean4 or Quint knowledge.

**Architecture:** EMC uses a functional-core/imperative-shell design. All I/O is represented as step/trampoline variant effects interpreted only at the outer shell. The core owns semantic model transformations, emits synchronized Lean4 and Quint canonical representations, derives browser data, and validates drift between all artifacts.

**Tech Stack:** Rust 2024, Clap, rmcp, serde, nutype 0.7.0, Lean4/Lake, Quint, Nix flakes, Vite/React browser copied from EMC for visual parity.

---

## Non-Negotiable Engineering Rules

- Warnings are errors everywhere: `RUSTFLAGS='-Dwarnings'` for build, test, fmt checks, clippy, and CI.
- Mirror EMC's strict Rust lint posture: enumerate `clippy::all` lints, keep high-signal lints at `forbid`, and allow only documented exact-lint `deny` carve-outs when third-party macros require them.
- Use functional-core/imperative-shell architecture. The model core must be deterministic and side-effect free.
- All I/O must go through a step/trampoline variant effect pattern. Production code may describe effects but may not perform file, process, network, stdio, clock, or environment I/O inside core modules.
- Use only semantic data types inside the system. Primitives and structural DTOs are allowed only at I/O boundaries.
- Practice parse-don't-validate: boundary data is immediately parsed into semantic types and all downstream code accepts only those semantic types.
- Use `nutype` for semantic data types unless a richer hand-written type is required for algebraic behavior. The current crate version is `0.7.0`.
- Do not add exceptions to these rules without a new checked-in decision record.

## System Shape

- `emc` is the single user-facing executable.
- CLI subcommands: `init`, `import emc`, `list`, `show`, `add`, `update`, `connect`, `validate`, `verify`, `check`, `generate site`, and `mcp`.
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

## EMC Compatibility

- EMC is an example and compatibility fixture, not EMC's default built-in model.
- Import current EMC event-model JSON into EMC-generated Lean4 and Quint fixtures.
- Port the current EMC rule surface as acceptance coverage:
  - 156 validator scenarios.
  - 9 review-gate scenarios.
  - 11 browser scenarios.
  - 6 runner/meta scenarios.
- Preserve browser visual parity with EMC: layout, styling, workflow selector behavior, timeline composition, branch cards, source chains, control effects, navigation targets, and review overlays.
- Make visible project title/branding configurable; do not hard-code EMC labels in EMC projects.
- Generate the same browser data shape: `data/index.json`, `data/workflows/*.eventmodel.json`, and `data/slices/*.eventmodel.json`.

## Implementation Sequence

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

### Task 4: EMC Import Compatibility

**Files:**
- Create: `src/emc_import/mod.rs`
- Create: `tests/fixtures/emc/`
- Create: `tests/emc_import.rs`

- [ ] Copy the current EMC event-model workflow and slice fixtures into EMC tests.
- [ ] Parse EMC JSON only at the import boundary.
- [ ] Convert imported DTOs into semantic EMC model types immediately.
- [ ] Emit Lean4, Quint, and browser JSON from the imported semantic graph.
- [ ] Assert stable digests and no primitive DTO leakage beyond the import boundary.

### Task 5: Validation Rule Port

**Files:**
- Create: `src/core/validation/`
- Create: `tests/features/event_model_validator/`
- Create: `tests/validation_rules.rs`

- [ ] Port EMC validator Gherkin into EMC test fixtures.
- [ ] Implement validation as pure functions over semantic model types.
- [ ] Preserve current EMC diagnostics where tests depend on user-facing messages.
- [ ] Fail validation on Lean/Quint/browser drift.

### Task 6: Lean4 and Quint Emission

**Files:**
- Create: `src/core/emit/lean.rs`
- Create: `src/core/emit/quint.rs`
- Create: `tests/emit_lean.rs`
- Create: `tests/emit_quint.rs`

- [ ] Generate deterministic Lean4 modules for the actual business model.
- [ ] Generate deterministic Quint modules for the same model.
- [ ] Add golden tests for imports, workflows, slices, transitions, invariants, and proof/model-check entrypoints.
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

- [ ] Port EMC browser UI for visual parity.
- [ ] Make title and project branding configurable.
- [ ] Generate EMC-compatible `data/index.json`, workflow JSON, and slice JSON.
- [ ] Preserve composed workflow loading, canonical lanes, timeline ordering, branch rendering, source chains, control effects, navigation targets, and review overlays.

### Task 9: Review Gate

**Files:**
- Create: `src/core/review_gate.rs`
- Create: `tests/features/event_model_review_gate/`
- Create: `tests/review_gate.rs`

- [ ] Port EMC review-gate semantics.
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
- All EMC-derived validator, review-gate, browser, and meta scenarios pass.
- Nix app and container image build and pass smoke tests.
- Public core APIs expose semantic types, not primitives or structural DTOs.
- Static guardrails prove I/O only appears in shell/interpreter modules.
