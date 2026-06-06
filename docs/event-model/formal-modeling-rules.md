<!-- Copyright 2026 John Wilger -->

# Formal Event Modeling Rules

This document records the current event-modeling rule inventory for the EMC
Lean4 and Quint direction.

The source of truth for an event model is the Lean4 model and the Quint model.

Status markers:

- ✅ Mechanically enforced all the way through generated Lean4/Quint artifacts
  and verification.
- 🟡 Partially enforced, represented, or checked for some artifact level but not
  yet complete.
- ❌ Not currently mechanically enforced.

## Validity Standard

A model is complete and valid only when all of these are true:

- 🟡 The complete event model is represented in Lean4 and Quint.
- 🟡 The event-model semantic rules in this document are encoded as Lean
  definitions, Lean theorems, Quint model structure, and Quint invariants.
- ✅ Every data flow is represented with source, transformation or projection,
  target, and bit-level encoding semantics.
- ✅ Every displayed datum traces to read model field, event attribute, and
  original provenance.
- ✅ Every command input traces to actor input, session value, generated value,
  external payload, or event-stream-derived state.
- ✅ Every state change is justified by emitted or observed events.
- ✅ Every workflow branch, outcome, command error, navigation target, external
  boundary, and recovery path is modeled.
- ✅ Lean verification passes for every Lean module with `lake env lean`.
- ✅ Quint typechecking and invariant verification pass for every Quint module.
- ✅ Non-formal implementation code does not perform duplicate semantic
  validation for correctness.

## Model Structure

- ✅ Model structure is explicit: name, version, streams, events, commands, read
  models, slices, workflows, and composition structure are represented.
- ✅ Named definitions are unique where ownership requires uniqueness.
- ✅ Each slice model unit represents exactly one coherent slice.
- ✅ Slices have first-class `acceptance_scenarios` and `contract_scenarios`.
- ✅ Every scenario has Given, When, and Then.
- ✅ Acceptance scenarios describe user-facing behavior only.
- ✅ Contract scenarios cover projector, command, automation, translation, and
  derivation contracts.
- ✅ Scenario names are unique within a slice and across first-class scenario
  sets.
- ✅ State-view read models require projector contract scenarios.

## Events, Streams, And Provenance

- ✅ Events reference known streams.
- ✅ Every locally emitted event is produced by a command unless explicitly
  modeled as observed or shared.
- ✅ Command-sourced event attributes reference declared command inputs.
- ✅ External-sourced event attributes reference declared external payloads and
  payload fields.
- ✅ Event attributes cannot source from read models.
- ✅ Generated event attributes name a non-empty generated source kind.
- ✅ Event attributes declare source provenance.
- ✅ Read model fields declare source provenance.
- ✅ View fields declare source provenance.
- ✅ Read model fields source from known event attributes.
- ✅ Read model fields do not source directly from commands.
- ✅ Derived read model fields declare source fields and derivation rules.
- ✅ Derived read model fields have derivation scenario coverage.
- ✅ Transitive read models declare source relationship fields, transitive
  rules, and examples.
- ✅ Absence/default fields declare the event absence they derive from.
- ✅ Absence/default fields have absence-state scenario coverage.

## Commands And Inputs

- ✅ Commands get inputs from invocation arguments and event streams.
- ✅ Command inputs have reportable source chains.
- ✅ Control-provided command inputs declare source and description.
- ✅ Actor-provided inputs are visible in the information sketch.
- ✅ Hidden session inputs may be hidden but still require descriptions.
- ✅ Decision fields are visible to the actor.
- ✅ Command inputs with no issuing control still need modeled provenance.

## Read Models And Views

- ✅ State-view slices own views, read models, and projection paths.
- ✅ State-view slices own at least one view.
- ✅ State-view slices do not own state-changing commands.
- ✅ State-view controls may link to commands owned by state-change slices.
- ✅ Views require wireframes or information sketches.
- ✅ Wireframe tokens map to modeled fields, controls, or actor inputs.
- ✅ Every displayed field appears in the sketch.
- ✅ Every control appears in the sketch.
- ✅ View fields source only from read models used by the view.
- ✅ View fields do not source directly from events.
- ✅ Every displayed datum has a complete source chain to original provenance.

## Controls, Errors, And Recovery

- ✅ Controls reference known commands.
- ✅ Controls provide every command input.
- ✅ Control inputs require source provenance.
- ✅ Control inputs require descriptions.
- ✅ Actor-provided control inputs are visible in the sketch.
- ✅ Hidden session inputs may stay out of the sketch but still need
  descriptions.
- ✅ Decision fields must be visible to the actor.
- ✅ Views handle every returned error from commands they issue.
- ✅ Command-local errors are declared.
- ✅ Command-local errors have scenario coverage.
- ✅ Scenario error references must be declared by the command.
- ✅ Error handling describes recovery behavior, not only display text.
- ✅ Recovery behavior may be retry, stay-on-screen behavior, navigation, or an
  explicit recovery action.

## Slice Architecture

- 🟡 A slice is the smallest useful modeled behavior boundary.
- ✅ Only events may be shared across slices.
- ✅ Commands, views, controls, read models, automations, translations,
  scenarios, and UI are owned by one slice.
- ✅ State-change slices own commands, emitted event facts, outcomes, and
  errors.
- ✅ State-change slices emit at least one event.
- ✅ State-change slices do not own views, read models, automations,
  translations, controls, or wireframes.
- ✅ State-change scenarios name the streams they read and write.
- ✅ Singleton state changes declare already-exists or idempotent behavior.
- ✅ Translation slices declare external events or payload contracts.
- ✅ Translation slices do not own screens.
- ✅ Automation slices declare triggers.
- ✅ Automation slices represent one coherent reaction.
- ✅ Automations issue one command per triggered operation.
- ✅ Automations handle every command error they can receive.
- ✅ Duplicate non-event definitions across composed slices are invalid.
- ✅ Duplicate events are allowed only when their definitions are identical.
- ✅ Shared events cannot differ by stream.
- ✅ Shared events cannot differ by source provenance.

## Outcomes And Branching

- ✅ Outcome labels are unique within a slice.
- ✅ Outcomes are backed by non-empty event sets.
- ✅ Distinct outcomes cannot use the same event set, regardless of event order.
- ✅ Outcome events must be emitted or observed by the slice.
- ✅ Workflow compositions handle every externally relevant outcome.
- ✅ Workflow transitions cannot treat command-local errors as business
  outcomes.

## Board And Causal Shape

- ✅ Board lanes are canonical: `ux`, `actions`, and `events`.
- ✅ Canonical lane names match their purpose.
- ✅ Views, automations, and external events appear in `ux`.
- ✅ Commands and read models appear in `actions`.
- ✅ Events appear in `events`.
- ✅ Board elements reference real declarations.
- ✅ Automation board elements are declared automations.
- ✅ External events are modeled as external events, not automations.
- ✅ External events may trigger translation commands.
- ✅ External events cannot update read models directly.
- ✅ Board connections match causal semantics.
- ✅ Commands have real incoming triggers.
- ✅ Command-to-event edges match command `produces` declarations.
- ✅ Event-to-read-model edges match projection sources.
- ✅ View-to-command edges match owned controls.
- ✅ Read models feeding views have incoming event updates.
- ✅ Main-path boards cannot have disconnected unclassified islands.

## Navigation

- ✅ Navigation controls declare a navigation type.
- ✅ Valid navigation types include modeled view, local view state, external
  system, and external workflow.
- ✅ Modeled-view navigation targets existing composed views.
- ✅ Local-view-state navigation targets declared local state or filters.
- ✅ External-workflow navigation names the target workflow.
- ✅ External-system navigation names the external system and handoff or return
  contract.

## Workflow Composition

- ✅ Workflow compositions declare explicit steps.
- ✅ Workflow steps reference composed formal slice modules.
- ✅ Referenced non-supporting slices appear in workflow steps.
- ✅ Workflow files compose whole slices without redefining internals.
- ✅ A workflow has exactly one entry step.
- ✅ Workflow step slugs are unique.
- ✅ Every non-supporting step is reachable from entry.
- ✅ Non-entry main steps need incoming reachability.
- ✅ Branch and alternate steps declare a trigger or incoming-transition
  rationale.
- ✅ Async lifecycle steps are not modeled as required linear happy path.
- ✅ Workflow entry handles first-arrival lifecycle state before bootstrap.
- ✅ Application-entry state views cover fresh/uninitialized, initialized
  unauthenticated, initialized authenticated, partially configured, and fully
  configured states.
- ✅ Transitions target known workflow steps or explicit workflow exits.
- ✅ Event transitions are shared by source and target slices.
- ✅ Event transitions require the source slice to emit or observe the
  transition event.
- ✅ Event transitions require the target slice to observe or emit the
  transition event.
- ✅ Command transitions come from controls owned by source views.
- ✅ Command transitions target the slice that owns the command.
- ✅ Navigation transitions come from controls owned by source views.
- ✅ Navigation transitions resolve to the target workflow step's entry view.
- ✅ External-trigger transitions declare trigger payload contracts.
- ✅ Workflow exits name the target workflow and why the exit is reached.

## Information Completeness

- ✅ Every meaningful datum that flows through the system is modeled.
- ✅ Every datum has an original source.
- ✅ Every datum has an explicit target.
- ✅ Every datum has transformation, projection, derivation, default, or absence
  semantics when applicable.
- ✅ Every datum has bit-level representation semantics sufficient to verify
  that no information is lost or invented.
- ✅ Every user-visible datum traces through the model to original provenance.
- ✅ Every stored event fact traces to command input, external payload,
  generated value, session value, or modeled derivation.
- ✅ Every read model field traces to event facts or explicitly modeled absence.
- ✅ Every command input traces to an invocation source and, where sourced from
  prior state, back to event provenance.
- ✅ Every derivation has source fields, a derivation rule, and scenario
  coverage.
- ✅ Every external boundary has a payload contract and field-level provenance.

## Mechanical Proof Obligations

Lean4 should prove at least:

- ✅ Model identity and digest stability.
- ✅ Required structure is present.
- ✅ Ownership uniqueness rules.
- ✅ Slice architecture rules.
- ✅ Event, stream, command, read model, view, and scenario well-formedness.
- ✅ Source-chain completeness.
- ✅ Bit-level data-flow completeness.
- ✅ Outcome and error coverage.
- ✅ Workflow reachability and transition resolution.

Quint should verify at least:

- ✅ Workflow identity stability.
- ✅ Workflow slice details are complete.
- ✅ Workflow transitions are structured.
- ✅ Slice identity stability.
- ✅ State-change slices emit at least one event.
- ✅ Bit-level data flows are structured.
- ✅ Workflow reachability invariants.
- ✅ Transition legality invariants.
- ✅ Outcome handling invariants.
- ✅ Command error handling invariants.
- ✅ External boundary and payload-contract invariants.

## Non-Goals

- ✅ Do not duplicate event-model semantic validation in Rust or JavaScript.
