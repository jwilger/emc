# Gherkin Traceability

This file maps each checked-in event-model Gherkin scenario to the Rust test
target that executes the corresponding rule surface. `tests/rule_fixtures.rs`
keeps this table synchronized with the feature files, so adding, removing, or
renaming a scenario requires updating the executable coverage mapping.

| Feature | Scenario | Executable coverage |
| --- | --- | --- |
| `tests/features/event_model_cucumber_execution.feature` | Runner meta-check feature is registered without recursive execution | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_cucumber_execution.feature` | Validator feature runner discovers validator feature files | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_cucumber_execution.feature` | Browser feature runner discovers browser feature files | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_cucumber_execution.feature` | Review-gate feature runner discovers review-gate feature files | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_cucumber_execution.feature` | Legacy TUI acceptance runner does not own event-model feature suites | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_cucumber_execution.feature` | Retired Rust-native first-launch harness artifacts are absent | `cargo test --test cucumber_runner_config` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Composed board lanes are not repeated per slice | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Timeline steps render workflow order rather than concatenated slice internals | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Timeline highlights disconnected supporting or alternate branches distinctly | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Timeline transition labels explain why a step is reachable | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Timeline renders alternate outcome branches apart from the happy path | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Timeline renders retry branches apart from the happy path | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Timeline renders error recovery branches apart from the happy path | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Timeline overlays show unreachable or weakly justified steps during review | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Definition views show ownership and back-references | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Definition views show full source chains for displayed fields | `cargo test --test browser_composition` |
| `tests/features/event_model_browser/timeline_rendering.feature` | Definition views show control effects and navigation targets | `cargo test --test browser_composition` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Clean review records must match the current model digest | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Clean review records use category result markers | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Mandatory findings are associated with the model digest that produced them | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Empty review output is not a clean review | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Validator success alone is not a clean review | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Bare clean markers are rejected when required review categories are absent | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Required review categories must be clean | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | A workflow cannot advance while mandatory review findings remain | `cargo test --test review_gate` |
| `tests/features/event_model_review_gate/workflow_review_gate.feature` | Review findings require another review after correction | `cargo test --test review_gate` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Board lanes must use the canonical lane ids | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Board lanes must include every canonical lane | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Board lanes may not duplicate canonical lane ids | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Legacy projection lanes are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Board lane names must match the canonical lane purposes | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | View elements appear in the UX lane | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Automation elements appear in the UX lane | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | External event elements appear in the UX lane | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Read-model elements appear in the actions lane | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Command elements appear in the actions lane | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Event elements appear in the events lane | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Board elements reference known declarations | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Every board element kind references an allowed declared model element | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Automation board elements must be declared automations, not fake intermediates | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Undeclared non-event board elements cannot bridge dependencies | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | External events use explicit external event elements, not automation elements | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | External event to command is a valid translation trigger connection | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | External events cannot update read models directly | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | External event triggers must match translation external_event declarations | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Invalid board connection kind pairs are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Commands have real incoming triggers | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Command-to-event connections match command produces declarations | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Event-to-read-model connections match read model field sources | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | View-to-command connections match owned controls | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Read models feeding views have incoming event updates | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Read models cannot feed commands on the board | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow compositions declare explicit steps for browser timelines | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | A valid workflow composition includes entry, main path, branch, async branch, and workflow exit | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow slice files must exist | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow slice files must each be valid | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow steps reference referenced slice files | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Referenced non-supporting slices appear in workflow steps | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Non-entry workflow steps need incoming reachability | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | A workflow has exactly one entry step | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow step slugs are unique | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Every non-supporting step is reachable from the single entry step | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Transitions target known workflow steps or explicit workflow exits | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Branch steps still declare their trigger or incoming transition rationale | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Composed workflows reject non-canonical lanes in referenced slices | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Lifecycle branches are not modeled as required linear happy path | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow entry handles first-arrival lifecycle state before bootstrap | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Application-entry state views cover important lifecycle and session states | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Event transitions use events produced or observed by adjacent slices | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Event transitions require the source slice to emit or observe the transition event | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Event transitions require the target slice to observe or emit the transition event | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Command transitions come from controls owned by source views | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Command transitions target the slice that owns the command | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Navigation transitions come from controls owned by source views | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Navigation transitions resolve to the target workflow step's entry view | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | External trigger transitions declare trigger payload contracts | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow exits name the target workflow | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Validator rejects disconnected board islands in a main workflow step | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow files compose whole slices without redefining internals | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/board_timeline_and_workflow.feature` | Workflow steps do not select internal scenarios from a slice | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Externally relevant outcomes have unique labels | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Different outcomes cannot use the same event set | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Different outcomes cannot use the same multi-event set in a different order | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Outcomes must declare at least one event in their event set | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Outcome event sets reference events emitted or observed by the slice | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Outcome event sets cannot reference unrelated known events | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Workflow handles every externally relevant outcome | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Workflow transitions cannot branch on command-local errors as outcomes | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Scenario error references must be declared by the command | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Declared command errors require scenario coverage | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Views handle every error returned by commands they issue | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Automations handle every error returned by commands they issue | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Error handling describes recovery, not only display text | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | State-change slices include concrete Given When Then scenarios | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | State-view slices include empty and partial reachable states | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Translation slices include one scenario for each external payload variant | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | Automation slices include one scenario for each trigger event | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | A minimal composed workflow with owned slices and reachable transitions is valid | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | A minimal composed workflow with complete information chains is valid | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/outcomes_errors_and_review.feature` | A minimal composed workflow with handled errors and canonical boards is valid | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-view slices must own at least one view | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-view read models are updated by observed events before feeding views | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-view slices do not own state-changing commands | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-view controls may link to commands owned by state-change slices | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-change slices emit at least one event | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-change slices do not own source or post-command views | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-change slices do not own non-command UI/projection/automation definitions | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Legacy command read-model reads are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | State-change scenarios name stream reads for written events | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Singleton state changes declare repeat behavior | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Translation slices have an external signal or payload contract | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Translation slices do not own screens | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Board read-model to command dependencies require declared automations | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Automation slices declare a trigger | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Automation slices issue one command for a single triggered operation | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Automations handle every command error they can receive | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate commands across composed slices are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate read models across composed slices are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate views across composed slices are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate controls across composed slices are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate automations across composed slices are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate translations across composed slices are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate scenarios across composed slices are rejected unless they belong to the same owned slice | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate UI wireframes across composed slices are rejected | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Duplicate events are allowed only when their definitions are identical | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Identical duplicate events are allowed across slices | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Shared events cannot differ by stream | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/slice_architecture.feature` | Shared events cannot differ by source provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Model files must contain valid JSON | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Model must be a JSON object | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Required top-level sections are present | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Explicit board data is required | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Top-level named definitions are unique within a file | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Every core top-level named definition list rejects duplicates | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Slice files contain exactly one slice | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Slice files may not be empty | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Slice files reject legacy scenarios fields | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Slice files accept first-class scenario fields | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | First-class scenario fields require Given When Then | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Acceptance scenarios describe only user-facing behavior | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Slice scenario names are unique across first-class scenario fields | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Slice scenario names are unique within a slice | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | State-view read models require projector contract scenarios | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Slice outcome names are unique within a slice | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Events reference known streams | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Every locally emitted event is produced by a command | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Command-sourced event attributes reference declared command inputs | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | External event attributes reference declared external payloads | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | External event attributes reference declared external payload fields | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Event attributes cannot source from read models | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Generated event attributes use a non-empty generated source kind | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Read model fields source known event attributes | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Read model fields must not source directly from commands | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Derived read model fields declare derivation provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Derived read model fields require derivation scenario coverage | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Transitive relationship read models declare source fields, derivation rule, and examples | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Absence/default projection fields declare absence semantics | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Absence/default projection fields require absence scenario coverage | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/structure_and_sources.feature` | Translation command input provenance references declared external payload fields | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Views require information sketches | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Wireframe tokens map to modeled fields, controls, or actor inputs | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Every displayed field appears in the information sketch | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Every control appears in the information sketch | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Event attributes must declare source provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Read model fields must declare source provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | View fields must declare source provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | View fields must source from referenced read model fields | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | View fields must not source directly from events | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Every displayed datum has a full source chain to original provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Command inputs have reportable source chains | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Command inputs require modeled provenance when no issuing control supplies them | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Command inputs sourced from read models trace back to original provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Controls reference known commands | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Controls provide every command input | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Control inputs require source provenance | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Control inputs require descriptions | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Actor-provided inputs must be visible in the sketch | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Hidden session inputs may stay out of the sketch but still need descriptions | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Decision fields must be visible to the actor | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Views handle every returned error from commands they issue | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Navigation controls declare a navigation type | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Modeled-view navigation targets existing views | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | Local view-state navigation identifies the owning view state or filter | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | External workflow navigation names a workflow target | `cargo test --test validate_event_model` |
| `tests/features/event_model_validator/views_controls_and_information.feature` | External system navigation names the external system and handoff contract | `cargo test --test validate_event_model` |
