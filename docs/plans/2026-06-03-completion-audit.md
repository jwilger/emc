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
| CLI model read and mutation | CLI tests cover init, list, show, add workflow, add slice, connect workflow, update workflow, update slice, remove transition, remove slice, remove workflow, validate, verify, check, review, Gherkin suite execution, and site generation paths. The final `emc --help` audit advertises every implemented command family used by the README command list. | Strong evidence | Keep command-surface help synchronized as command families are added. |
| MCP model read and mutation | MCP stdio tests cover create, read, update, remove, validate, verify, check, review gate, review record, and transport behavior. HTTP coverage includes request validation and packaged `check_project` tool execution. `record_clean_review.reviewed_at` advertises the deterministic UTC millisecond timestamp contract in JSON Schema. | Strong evidence | Continue tightening tool schemas when new semantic formats are added. |
| Same semantic command core for CLI and MCP | `src/command.rs` provides shared effect plans, and architecture guardrails prevent MCP handlers from directly constructing command effects. | Strong evidence | Keep guardrails updated as new commands appear. |
| Functional core and imperative shell | Core operations describe effects through `src/core/effect.rs`; shell modules interpret filesystem, process, stdio, network, and environment effects. Architecture guardrails cover the known direct-I/O regressions. A final direct-I/O scan across `src/core` found no direct filesystem, process, environment, network, clock, or stdio calls. | Strong evidence | Keep direct-I/O scans and architecture guardrails current as new effect classes are added. |
| Step/trampoline variant effect pattern | Command planning returns effect plans interpreted at the shell boundary. CLI and MCP route through that effect plan layer. The final command-surface change only exposed existing Gherkin effect-plan commands in help; it did not add direct shell execution to the parser. | Strong evidence | Keep new command families routed through `EffectPlan`. |
| Semantic data types only past boundaries | `nutype = 0.7.0` is pinned. Boundary parsers cover project names, paths, model descriptions, workflow slugs, slice slugs, transition endpoints and kinds, review timestamps, reviewers, and validation definition names. Static guardrails reject primitive-bearing core API regressions and now scan every `src/core` Rust file to reject public raw `Vec<T>` inputs or slice-returning collection APIs. Validation structural builder parts and `with_*` assembly methods are crate-private DTO-parser internals. Layout, effect, workflow mutation, formal emission, digest, verify, review-record, workflow-document, validation, and browser projection APIs expose semantic collection types or crate-private parser details. | Strong evidence | Keep boundary parsers and core API guardrails current as new command surfaces are added. |
| Parse-don't-validate | CLI and MCP DTO parsing convert raw strings and JSON fields into semantic types before command execution. Duplicate-key and project-path checks happen at boundary parsing. Validation orchestration now routes raw event-model files through one shared parse path into semantic documents before rule checks, and architecture guardrails prevent a second raw event-model parse path from returning. | Strong evidence | Keep new boundary parsers and validation entrypoints on the shared raw-to-semantic path. |
| Actual business model represented in Lean4 and Quint | Workflow and slice mutations emit Lean4 and Quint modules for the modeled workflows, slices, transitions, identity fields, slice details, and digests. Verification runs Lean4/Lake and Quint against project root, workflow, and slice artifacts. `src/core/formal_graph.rs` parses generated Lean4 and Quint workflow declarations back into normalized semantic workflow graphs, and `emc check` compares those graphs against the workflow document. | Strong evidence | Keep the normalized graph reader current as generated formal declarations expand. |
| Deterministic drift checking | `emc check` rejects drift across browser data, Lean4 workflow artifacts, Quint workflow artifacts, normalized Lean4 and Quint workflow graphs, slice artifacts, root artifacts, tool config, duplicate metadata, unmodeled artifacts, stale declarations, and generated browser data. | Strong evidence | Keep graph checks, marker checks, and digest checks synchronized as formal artifacts evolve. |
| Lean4 proof surface | Generated Lean4 modules include identity, slice-detail, transition-structure, root namespace, and slice identity obligations. Check coverage rejects stale or tautological declarations in the current proof surface, and focused emission plus drift tests passed in the final audit. | Strong evidence | Strengthen obligations as the modeled formal surface grows. |
| Quint model surface | Generated Quint modules include executable workflow and slice invariants, init and step surfaces, and pinned verification through `emc verify`. Focused emission tests reject tautological transition invariants and assert executable invariant/action entrypoints. | Strong evidence | Strengthen executable behavior as the modeled formal surface grows. |
| Event-model validation rules | Gherkin fixture counts are checked in for validator, review-gate, browser, and runner/meta suites. Validator tests cover a broad set of structure, source, slice, board, timeline, outcome, review, and browser-data diagnostics. `docs/gherkin-traceability.md` maps every checked-in scenario to the Rust test target that executes its rule surface, and `tests/rule_fixtures.rs` keeps the map synchronized with feature paths, scenario titles, and expected executable targets. | Strong evidence | Keep scenario traceability current as rules are added, removed, or moved. |
| Review gate | CLI and MCP review gate enforce current clean reviews for workflow slug, digest, categories, mandatory findings, stale reviews, and clean follow-up. Review record creation is deterministic, package-smoked, and advertises its strict timestamp contract over MCP. | Strong evidence | Keep review schema metadata synchronized with semantic boundary parsers. |
| Generated browser site | `emc generate site` produces stable browser data and replaces stale output. Browser composition tests cover workflow selector data, lanes, main path, branch cards, source chains, controls, navigation targets, command/view definitions, and review overlays. Nix package smoke serves the generated site and renders it through headless Chromium, asserting project, workflow, and slice text in the rendered DOM. | Strong evidence | Keep rendered package smoke current as browser assets change. |
| Browser visual parity | Browser assets preserve the reference board, timeline, definition, source-chain, control, navigation, and review-overlay presentation while accepting project branding and avoiding unrelated labels. Composition tests preserve the key data contracts. Package smoke runs the generated site in headless Chromium, asserts rendered DOM content, captures a 1440x1000 PNG screenshot, and rejects missing or malformed visual output. | Strong evidence | Keep rendered package smoke current as browser assets change. |
| Strict Rust lints | `Cargo.toml` enumerates strict Clippy policy; `justfile` runs fmt, clippy, tests, and build with `RUSTFLAGS='-Dwarnings'`; lint-policy tests guard the setup. | Strong evidence | Keep final `just ci` as a release gate. |
| Mutation testing balance | `just mutants-diff`, `just mutants-core`, and `just mutants-full` exist outside CI. Recent Rust behavior commits ran `just mutants-diff` before commit. CI guardrails ensure mutation testing is not accidentally folded into routine CI. | Strong evidence | Continue running `just mutants-diff` before meaningful Rust behavior commits and `just mutants-core` before larger core milestones. |
| Nix package and container | Nix checks build the package, run package smoke, and build a Docker-compatible image. Package smoke exercises Lean4/Lake, Quint, review record/gate, site generation, MCP stdio, and MCP HTTP tool calls. | Strong evidence | Optionally add a container runtime smoke when a local container runtime is available; current Nix check proves image construction. |
| README and user docs | README explains EMC purpose, user workflows, Lean4 and Quint roles, CLI/MCP usage, Gherkin rule suites, review gates, generated site, guardrails, mutation testing, and Nix packaging. The final help-surface audit confirmed the README command list matches implemented CLI families, including the checked-in Gherkin runner commands now advertised by `emc --help`. | Strong evidence | Keep README command examples synchronized with `emc --help`; no README-specific tests are required. |
| No unrelated project references | Recent forbidden scans across README, docs, source, tests, Cargo manifest, justfile, and CI paths were clean. | Strong evidence | Keep the forbidden scan in every final verification pass. |

## Highest-Value Remaining Increments

None. The final closure pass has been completed.

## Current Bottom Line

EMC has strong implementation evidence for the binary, command surfaces, MCP
access, deterministic mutation paths, formal artifact emission, drift checks,
review gate, rendered browser execution, semantic public core APIs, normalized
Lean4 and Quint workflow graph reading, strict lints, mutation-testing workflow,
validation scenario traceability, and Nix packaging. The final closure gates
passed on the current worktree: `just ci`, `just mutants-diff` for the Rust help
change, `nix flake check`, the forbidden unrelated-reference scan, the public
core collection API scan, and `git diff --check`.
