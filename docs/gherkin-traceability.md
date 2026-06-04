# Gherkin Traceability

This file maps each checked-in EMC Gherkin scenario to the Rust test target
that executes the corresponding rule surface. `tests/rule_fixtures.rs` keeps
this table synchronized with the feature files, so adding, removing, or
renaming a scenario requires updating the executable coverage mapping.

Lean4 and Quint event-model rules are tracked in
`docs/event-model/formal-modeling-rules.md`, not as legacy browser or JSON
validator Gherkin.

| Feature | Scenario | Executable coverage |
| --- | --- | --- |
| `tests/features/event_model_cucumber_execution.feature` | Runner meta-check feature is registered without recursive execution | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_cucumber_execution.feature` | Review-gate feature runner discovers review-gate feature files | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_cucumber_execution.feature` | Legacy TUI acceptance runner does not own event-model feature suites | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Clean review records must match the current model digest | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Clean review records use category result markers | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Mandatory findings are associated with the model digest that produced them | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Empty review output is not a clean review | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Validator success alone is not a clean review | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Bare clean markers are rejected when required review categories are absent | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Required review categories must be clean | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | A workflow cannot advance while mandatory review findings remain | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Review findings require another review after correction | `cargo test --test review_gate` |
