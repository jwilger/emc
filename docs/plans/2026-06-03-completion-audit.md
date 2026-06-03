# EMC Completion Audit - 2026-06-03

**Goal:** Identify what remains before EMC can be treated as meeting the full
system goal: a single Rust executable for CLI and MCP event-model work, backed
by deterministic Lean4 and Quint artifacts for the actual business model, with
strict semantic typing, functional-core/imperative-shell I/O, generated browser
output, and mechanical guardrails.

**Status Key:**

- **Strong evidence:** implemented and covered by focused tests plus recent
  integration or packaging gates.
- **Partial evidence:** implemented in meaningful depth, but at least one goal
  requirement still needs a stronger mechanical check or narrower audit.
- **Open gap:** not yet proven enough for the goal.

## Requirement Matrix

| Requirement | Current Evidence | Status | Remaining Work |
| --- | --- | --- | --- |
| Single binary executable | `Cargo.toml`, `src/main.rs`, and Nix package smoke exercise the packaged `emc` binary across init, mutation, check, verify, review, site generation, MCP stdio, and MCP HTTP. | Strong evidence | Keep package smoke current as commands are added. |
| CLI model read and mutation | CLI tests cover init, list, show, add workflow, add slice, connect workflow, update workflow, update slice, remove transition, remove slice, remove workflow, validate, verify, check, review, and site generation paths. | Strong evidence | Final command-surface audit against `emc --help` before closing the goal. |
| MCP model read and mutation | MCP stdio tests cover create, read, update, remove, validate, verify, check, review gate, review record, and transport behavior. HTTP coverage includes request validation and packaged `check_project` tool execution. `record_clean_review.reviewed_at` advertises the deterministic UTC millisecond timestamp contract in JSON Schema. | Strong evidence | Continue tightening tool schemas when new semantic formats are added. |
| Same semantic command core for CLI and MCP | `src/command.rs` provides shared effect plans, and architecture guardrails prevent MCP handlers from directly constructing command effects. | Strong evidence | Keep guardrails updated as new commands appear. |
| Functional core and imperative shell | Core operations describe effects through `src/core/effect.rs`; shell modules interpret filesystem, process, stdio, network, and environment effects. Architecture guardrails cover the known direct-I/O regressions. | Partial evidence | Run a final direct-I/O audit across `src/core` and add static guardrails for any uncovered I/O classes found. |
| Step/trampoline variant effect pattern | Command planning returns effect plans interpreted at the shell boundary. CLI and MCP route through that effect plan layer. | Strong evidence | Confirm final command additions keep using `EffectPlan` instead of direct shell calls. |
| Semantic data types only past boundaries | `nutype = 0.7.0` is pinned. Boundary parsers cover project names, paths, model descriptions, workflow slugs, slice slugs, transition endpoints and kinds, review timestamps, reviewers, and validation definition names. Static guardrails reject several primitive-bearing core API regressions. Validation structural builder parts and `with_*` assembly methods are now crate-private DTO-parser internals rather than public core API. | Partial evidence | Finish the public core API audit for remaining collection-bearing signatures and add semantic collection types or explicit guardrails where needed. |
| Parse-don't-validate | CLI and MCP DTO parsing convert raw strings and JSON fields into semantic types before command execution. Duplicate-key and project-path checks happen at boundary parsing. Validation orchestration now routes raw event-model files through one shared parse path into semantic documents before rule checks, and architecture guardrails prevent a second raw event-model parse path from returning. | Strong evidence | Keep new boundary parsers and validation entrypoints on the shared raw-to-semantic path. |
| Actual business model represented in Lean4 and Quint | Workflow and slice mutations emit Lean4 and Quint modules for the modeled workflows, slices, transitions, identity fields, slice details, and digests. Verification runs Lean4/Lake and Quint against project root, workflow, and slice artifacts. | Partial evidence | Decide whether the goal requires EMC to parse Lean4 and Quint artifacts back into a normalized semantic graph, not only compare deterministic declarations, markers, digests, and invariant names. If yes, add that graph reader and drift tests. |
| Deterministic drift checking | `emc check` rejects drift across browser data, Lean4 workflow artifacts, Quint workflow artifacts, slice artifacts, root artifacts, tool config, duplicate metadata, unmodeled artifacts, stale declarations, and generated browser data. | Strong evidence | Strengthen only if normalized graph reading is adopted. |
| Lean4 proof surface | Generated Lean4 modules include identity, slice-detail, transition-structure, root namespace, and slice identity obligations. Check coverage rejects stale or tautological declarations in the current proof surface. | Strong evidence | Final proof-surface audit for meaningful obligations versus marker-only declarations before closing the goal. |
| Quint model surface | Generated Quint modules include executable workflow and slice invariants, init and step surfaces, and pinned verification through `emc verify`. | Strong evidence | Final model-surface audit for each emitted invariant and transition surface. |
| Event-model validation rules | Gherkin fixture counts are checked in for validator, review-gate, browser, and runner/meta suites. Validator tests cover a broad set of structure, source, slice, board, timeline, outcome, review, and browser-data diagnostics. `docs/gherkin-traceability.md` maps every checked-in scenario to the Rust test target that executes its rule surface, and `tests/rule_fixtures.rs` keeps the map synchronized with feature paths, scenario titles, and expected executable targets. | Strong evidence | Keep scenario traceability current as rules are added, removed, or moved. |
| Review gate | CLI and MCP review gate enforce current clean reviews for workflow slug, digest, categories, mandatory findings, stale reviews, and clean follow-up. Review record creation is deterministic, package-smoked, and advertises its strict timestamp contract over MCP. | Strong evidence | Keep review schema metadata synchronized with semantic boundary parsers. |
| Generated browser site | `emc generate site` produces stable browser data and replaces stale output. Browser composition tests cover workflow selector data, lanes, main path, branch cards, source chains, controls, navigation targets, command/view definitions, and review overlays. Nix package smoke serves the generated site and renders it through headless Chromium, asserting project, workflow, and slice text in the rendered DOM. | Strong evidence | Keep rendered package smoke current as browser assets change. |
| Browser visual parity | Browser assets now accept project branding and avoid unrelated labels. Composition tests preserve the key data contracts, and package smoke proves the browser runtime renders modeled content. | Partial evidence | Add higher-fidelity interaction or screenshot/pixel smoke if exact visual parity remains a closure requirement beyond rendered DOM behavior. |
| Strict Rust lints | `Cargo.toml` enumerates strict Clippy policy; `justfile` runs fmt, clippy, tests, and build with `RUSTFLAGS='-Dwarnings'`; lint-policy tests guard the setup. | Strong evidence | Keep final `just ci` as a release gate. |
| Mutation testing balance | `just mutants-diff`, `just mutants-core`, and `just mutants-full` exist outside CI. Recent Rust behavior commits ran `just mutants-diff` before commit. CI guardrails ensure mutation testing is not accidentally folded into routine CI. | Strong evidence | Continue running `just mutants-diff` before meaningful Rust behavior commits and `just mutants-core` before larger core milestones. |
| Nix package and container | Nix checks build the package, run package smoke, and build a Docker-compatible image. Package smoke exercises Lean4/Lake, Quint, review record/gate, site generation, MCP stdio, and MCP HTTP tool calls. | Strong evidence | Optionally add a container runtime smoke when a local container runtime is available; current Nix check proves image construction. |
| README and user docs | README explains EMC purpose, user workflows, Lean4 and Quint roles, CLI/MCP usage, review gates, generated site, guardrails, mutation testing, and Nix packaging. | Partial evidence | Final docs audit against the actual help surface and current command names; no README-specific tests are required. |
| No unrelated project references | Recent forbidden scans across README, docs, source, tests, Cargo manifest, justfile, and CI paths were clean. | Strong evidence | Keep the forbidden scan in every final verification pass. |

## Highest-Value Remaining Increments

1. **Semantic-boundary audit:** finish scanning public core APIs for remaining
   collection-bearing signatures and add semantic collection types or explicit
   guardrails where needed.
2. **Formal graph decision:** either implement Lean4/Quint artifact readers that
   normalize formal artifacts back into semantic graph data, or explicitly
   narrow the goal to deterministic generated declarations, digests, and tool
   verification. The original goal wording leans toward the stronger graph
   reader, so treat this as an open decision until resolved.
3. **Final closure pass:** rerun `just ci`, local mutation testing appropriate to
   the touched Rust surface, `nix flake check`, package smoke, forbidden scan,
   and a line-by-line goal audit before declaring the full goal complete.

## Current Bottom Line

EMC has strong implementation evidence for the binary, command surfaces, MCP
access, deterministic mutation paths, formal artifact emission, drift checks,
review gate, rendered browser execution, strict lints, mutation-testing
workflow, validation scenario traceability, and Nix packaging. The goal should
not be closed until semantic-boundary coverage and the Lean4/Quint normalized
graph decision are resolved.
