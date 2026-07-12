<!-- Copyright 2026 John Wilger -->

# EMC Modeling Process

This document describes the working process for creating and changing EMC event
models. It is intentionally about modeling judgment, not command syntax. The
formal metamodel rules remain in `docs/event-model/formal-modeling-rules.md`;
this document explains how to approach a model so those rules describe useful
business behavior instead of well-formed but misplaced facts.

An EMC model is complete only when the event-sourced history projects to
synchronized Lean4 and Quint artifacts, `emc check` passes, `emc verify`
passes, and review-gate evidence is current for the modeled workflow. Generated
Lean4 and Quint files are projections, not authoring surfaces. Make changes
through EMC operations, then regenerate and verify.

## Modeling Standard

Model the business system from outside-observable behavior down to mechanically
checked provenance.

The ideal model says:

- what external actors and systems are trying to accomplish;
- which workflows describe those journeys;
- which slices own the behavior;
- which commands make decisions;
- which events record facts that happened;
- which read models and views expose state to actors;
- which automations, translations, and external payloads connect boundaries;
- which scenarios prove the important behavior and contracts;
- where every meaningful datum originates, how it changes, and where it is
  used;
- which formal facts prove the model is complete enough to trust.

Do not model internal implementation desires as user behavior. Acceptance
scenarios are for externally driveable behavior from a user or external actor
perspective. Internal invariants, provenance requirements, command decisions,
projection behavior, derived state, and error handling belong in contract
scenarios, command/event/read-model facts, data-flow facts, information
completeness facts, and formal workflow evidence.

Prefer explicit modeling over implication. If a command uses a value, model the
input and source. If a read model displays or derives a value, model the field,
source, derivation, absence behavior, and provenance. If a transition can
happen, model the trigger and legal source/target. If an external system sends
or receives data, model the boundary contract.

## Phase-By-Phase Modeling Order

The phases are the preferred order for a new workflow. Real modeling is
iterative: discoveries in a later phase often send you back to refine earlier
elements. When that happens, update the existing element rather than deleting
and recreating a larger container.

### Phase 1: Establish the Domain Boundary

Start by naming the product area, business capability, or bounded context. Write
down what the model is for and what is outside it. Identify the humans,
organizations, systems, scheduled jobs, and external event sources that can
initiate behavior.

At this phase, do not design storage tables, service classes, or UI widgets.
Capture the language of the business process: actor goals, decisions, state
changes, externally visible outcomes, and external contracts.

Good output from this phase:

- a project name;
- a small glossary of domain terms;
- a list of external actors and systems;
- one or more candidate workflows;
- known non-goals and out-of-bound systems.

### Phase 2: Find Workflow Journeys

A workflow is an externally meaningful journey through the domain. It should
have an entry condition, a business goal, possible branches, terminal outcomes,
and enough continuity that a reviewer can understand why the steps belong
together.

Create workflows before detailed slices. A workflow gives every later slice,
transition, scenario, and read model a business context. Avoid creating a
workflow for every technical operation. If two flows share most steps and
branch only by outcome, they usually belong in one workflow. If they have
different actors, entry points, lifecycle states, and goals, split them.

For each workflow, identify:

- entry actor or trigger;
- happy path;
- alternate business outcomes;
- failures that matter to users;
- external systems involved;
- lifecycle states that affect entry, such as uninitialized or unauthenticated
  states;
- the review gate that will certify the workflow when complete.

### Phase 3: Slice the Behavior

A slice is the smallest useful modeled behavior boundary. A good slice owns one
coherent responsibility and can be reasoned about independently. Slices are not
layers. They should not be split into "UI slice", "service slice", and
"database slice" for the same behavior.

Choose the slice kind by responsibility:

- `state_view`: owns views, read models, controls, and the state needed to show
  or drive user-visible decisions.
- `state_change`: owns commands, emitted events, outcomes, command errors, and
  state-changing business decisions.
- `translation`: owns external event or payload translation across system
  boundaries.
- `automation`: owns a triggered reaction that issues commands without a human
  directly pressing a control.

Only events are shared across slices. Commands, views, controls, read models,
automations, translations, scenarios, and UI belong to one slice. If two slices
need the same fact, share an event and let each slice model its own projection
or command input from that event.

### Phase 4: Sketch the Workflow Skeleton

Add the initial workflow steps and transitions before filling every detail.
Connect enough of the graph to show the expected path and obvious branches.
This makes missing slices and illegal ownership relationships visible early.

Use transition kinds intentionally:

- `navigation`: a user moves from one view to another through a control.
- `command`: a control or actor action issues a command owned by a target
  state-change slice.
- `event`: an emitted or observed event lets another slice react or continue.
- `external_trigger`: an outside system starts or advances behavior through a
  payload contract.
- `outcome`: a business result from one slice selects the next path.

Do not use workflow transitions to cover command-local errors. Command-local
errors belong with command error definitions, contract scenarios, and view
recovery behavior. Use workflow outcomes for externally meaningful business
branches.

### Phase 5: Capture Acceptance Scenarios

Acceptance scenarios describe user-facing or external-actor behavior. They
should be driveable from the application interface, a public API, an external
message, or another observable boundary. They are not a place for internal
payload invariants, database implementation details, or provenance bookkeeping.

Use acceptance scenarios to clarify:

- actor intent;
- starting context;
- external action or trigger;
- visible result;
- business outcome;
- user-facing branch or error.

If a scenario cannot be phrased from an external actor's point of view, it is
probably not an acceptance scenario. Move it to a contract scenario, data-flow
fact, command input provenance, read-model derivation, or formal rule.

### Phase 6: Model State Changes

For each state-changing behavior, define the command, inputs, emitted events,
outcomes, and command-local errors.

Commands make business decisions. They accept inputs from actor invocation,
session state, generated values, external payloads, or event-stream-derived
state. Every input needs provenance. Inputs that affect decisions should be
visible to the actor unless the source is intentionally hidden, such as session
state, in which case it still needs a description.

Events record facts that happened. They are not command requests and they are
not read-model snapshots. Event attributes should source from command inputs,
external payload fields, generated values, session values, or modeled
derivations. They must not source from read models.

Outcomes describe externally relevant business results. Command errors describe
local failures and recovery paths. Do not model a command-local error as a
workflow outcome unless it changes the business journey in a way an external
actor cares about.

### Phase 7: Model Reads, Views, Controls, And Recovery

State-view slices own what actors see and how they initiate commands.

Create read models from event facts or explicitly modeled absence. Then create
views that expose read-model fields and controls. A view field should not source
directly from an event; the read model is the projection boundary. A command
input should not source from a read model; commands that need prior state should
observe event-stream-derived state with provenance back to events.

Controls connect user intent to commands. Every control-provided command input
must have a source, description, sketch token, and visibility/decision marking.
Views that issue commands must handle every returned command error with a
recovery behavior such as retry, staying on the screen, navigation, or an
explicit recovery action.

### Phase 8: Model External Boundaries

Use translation slices for external messages and payload contracts. Use
automation slices for reactions triggered by modeled events or external
conditions. Keep external event ingestion separate from interpreted domain
understanding when the distinction matters.

Raw evidence events should remain separate from interpreted understanding
events. If downstream behavior depends on how raw evidence was interpreted,
model that interpretation as a command, event, read-model derivation, or
contract. Do not hide the distinction inside an acceptance scenario.

Every external boundary should declare payload contracts, field-level
provenance, and the command or translation that consumes the payload. External
events may trigger translation commands. They should not update read models
directly.

### Phase 9: Prove Information Completeness

Once the behavior is sketched, audit every meaningful datum. A complete model
knows where each datum came from, how it changed, where it is stored, where it
is displayed, where it is sent, and what bit-level representation semantics
apply.

For each datum, ask:

- What is the original source?
- Is the source an actor, session, generated value, external payload, event
  stream, absence, or derivation?
- Which command input, event attribute, read-model field, view field, or
  external payload field carries it?
- Does it need transformation, projection, derivation, default, absence, or
  bit-level encoding semantics?
- Which scenario covers the important behavior?
- Which formal rule proves it is complete?

Information completeness is where many misplaced acceptance scenarios should
land. "The system cites raw evidence event ids" is not user behavior by itself;
it is a provenance or traceability requirement that should be represented in
the data and contract model.

### Phase 10: Review, Check, Verify, And Record Readiness

Run `emc sync` to regenerate artifacts from the event history, then `emc check`
to prove the generated artifacts match it. Run
`emc verify` to prove Lean4 and Quint accept the current model. Then perform a
structured review against the workflow's review gate and record a clean review
when the model is ready.

Verification is not a replacement for modeling judgment. A model can verify and
still encode the wrong concept in the wrong place. Use review to catch category
errors such as internal invariants modeled as acceptance scenarios, command
decisions hidden in read models, or external payloads treated as domain events
without translation.

### Phase 11: Iterate By Updating The Right Element

When review identifies a modeling error, change the smallest semantic element
that owns the error. Do not remove and recreate a workflow or slice just to edit
a scenario, command input, event attribute, read-model field, or transition.

Use update operations when the element identity remains the same and the model
fact was incomplete or inaccurate. Use remove operations when the concept is no
longer part of the model. Use replacement only when the identity itself is
wrong and keeping history under the old identity would mislead future readers.

After every meaningful edit or removal, run `emc sync` when regeneration is
needed, then check, verify, and refresh
review evidence as needed.

## Element Modeling Guidance

This section describes how to create, update, and remove each kind of modeled
element. The command surface may lag this process for some element families;
the process still defines the desired semantic behavior.

### Projects

Create a project when you are starting a distinct event-modeling authority. The
project name should be the product, bounded context, or capability family, not
the implementation repository name unless those are the same.

Update a project when the domain name changes or when root-level verification
configuration must be regenerated from the event history. Do not hand-edit
generated root artifacts.

Remove a project by removing the whole project directory or repository context,
not by deleting individual generated files. Preserve the event log if the model
history must remain auditable.

### Workflows

Create a workflow for an externally meaningful journey. A workflow should have
one coherent goal, an entry point, reachable steps, and recognizable business
outcomes.

When creating a workflow, define:

- slug and name in business language;
- description of actor goal and scope;
- likely entry lifecycle states;
- expected slices and path shape;
- review gate expectations.

Update a workflow when the journey's name, description, entry framing, or
composition changes. Keep the same workflow identity when the business journey
is the same. Rename only when the old name misleads.

Remove a workflow when the journey is no longer part of the product model.
Before removal, check for incoming workflow-exit transitions, shared events, and
review records that would become misleading.

### Slices

Create a slice when a cohesive responsibility emerges inside a workflow. The
slice kind should match ownership, not current implementation convenience.

When creating a slice, define:

- owning workflow;
- slug and name;
- kind;
- description of responsibility;
- expected commands, views, translations, automations, or scenarios;
- likely shared events.

Update a slice when its name, description, kind, or responsibility changes.
Changing kind is safe only when the owned elements still satisfy the target
kind's rules. If a state-view slice has state-changing commands, the slice is
wrong; move or remodel the commands rather than merely changing the type.

Remove a slice only after deciding what happens to owned definitions,
scenarios, transitions, and formal artifacts. Transitions involving the slice
must be removed or redirected. Shared events may remain if another slice still
owns compatible meaning for them.

### Workflow Transitions

Create a transition when the workflow can legally move from one step to another
or exit to another workflow. Every transition needs a kind, trigger, source,
target, and evidence that the source/target relationship is legal.

Use:

- navigation transitions for view-to-view movement through controls;
- command transitions for control-issued commands;
- event transitions for emitted or observed domain facts;
- external-trigger transitions for outside payloads;
- outcome transitions for business branch results.

Update a transition when the trigger, kind, endpoint, or evidence was modeled
wrong. If changing the transition would alter the business path, update related
scenarios and workflow evidence in the same modeling pass.

Remove a transition when the path is impossible, obsolete, or better modeled as
a different kind of relationship. Removing a transition may make a step
unreachable; check the workflow graph immediately afterward.

### Acceptance Scenarios

Create an acceptance scenario for externally observable behavior. It should
have a clear Given, When, and Then from the perspective of a user, external
actor, external system, or public boundary.

Good acceptance scenarios describe:

- a user accomplishing a goal;
- an external system sending a meaningful event;
- a visible state or result;
- an externally relevant error or branch;
- a workflow-level outcome.

Do not use acceptance scenarios for:

- internal event payload invariants;
- database shape;
- provenance bookkeeping by itself;
- command implementation details;
- read-model derivation mechanics;
- proof obligations that are not externally driveable.

Update an acceptance scenario when its wording, kind, streams, or expected
result is inaccurate but the external behavior remains part of the slice.

Remove an acceptance scenario when it describes internal behavior, duplicate
coverage, obsolete behavior, or a requirement that belongs in a contract or
formal data-flow fact.

### Contract Scenarios

Create a contract scenario when a modeled element has business logic or
projection behavior that must be specified internally. Contract scenarios cover
commands, events, read models, views, automations, translations, and
derivations.

Use contract scenarios for:

- command decisions and command-local errors;
- projector/read-model behavior;
- derived fields;
- absence/default behavior;
- automation reaction contracts;
- translation and external payload contracts;
- internal provenance and traceability requirements.

Update a contract scenario when the covered definition, contract kind, read or
written streams, error references, or Given/When/Then changes.

Remove a contract scenario only after confirming the covered definition no
longer needs that coverage or the coverage has moved to a more precise
scenario.

### Commands

Create a command when the model needs a state-changing decision. A command
should be owned by a state-change slice and should emit or observe events that
explain its effect.

When creating a command, define:

- command name;
- owning slice;
- inputs and source chains;
- observed streams when prior state is needed;
- emitted events;
- singleton or repeat behavior when applicable;
- command-local errors;
- contract scenarios for decisions and errors.

Update a command when its name, inputs, observed streams, emitted events,
repeat behavior, or errors change. If the command stops being state-changing,
it probably should not remain a command.

Remove a command only after removing or redirecting controls, automations,
translations, scenarios, emitted events, errors, and workflow transitions that
reference it.

### Command Inputs

Create a command input for every value the command needs to make a decision or
create an event. Inputs must have a source kind, description, provenance chain,
and any source-specific fields.

Use source kinds intentionally:

- actor input for values provided by a human;
- invocation argument for direct API or command invocation;
- session for contextual runtime state;
- generated for system-created values;
- external payload for boundary data;
- event-stream state for prior domain facts.

Update a command input when its source, description, provenance chain, or
source-specific mapping changes. Keep decision-making inputs visible to actors
unless the source is intentionally hidden and described.

Remove a command input only after removing event attributes, controls,
contract scenarios, and command logic that depend on it.

### Events And Streams

Create an event when something has happened and the business will rely on that
fact later. Create or reference a stream to define where the event belongs.

Events should be named as facts, not commands. Prefer `TicketOpened` over
`OpenTicket`. Attributes should be business facts with provenance, not copies
of read-model state.

Update an event when its name, stream, source participation, or attributes were
wrong. Shared events may be updated only when every sharing slice agrees on the
same definition, stream, and provenance.

Remove an event only after removing command emissions, read-model projections,
workflow transitions, outcomes, board connections, and scenarios that depend on
it.

### Event Attributes

Create an event attribute for every meaningful fact the event records. The
attribute source should be a command input, external payload field, generated
value, session value, or modeled derivation.

Update an attribute when its source, source field, generated source kind,
provenance, or bit-level semantics changes.

Remove an attribute only after removing read-model fields, view fields,
external outputs, data-flow facts, and contract scenarios that rely on it.

### Read Models

Create a read model when the system needs projected state for views, decisions,
or external reporting. A read model belongs to a state-view slice unless the
modeling rules explicitly support a different owner.

Read models should source from event facts or modeled absence. They should not
invent facts and should not be used as the source of command truth.

Update a read model when its name, source event, projected fields,
relationships, derivations, absence/default behavior, or contract scenarios
change.

Remove a read model only after removing views, fields, controls, contract
scenarios, and data-flow facts that depend on it.

### Read-Model Fields

Create a read-model field for a datum actors or downstream processes need from
projected state. Record source event and attribute, derivation rule,
transitive relationship, absence/default behavior, provenance, and scenario
coverage as applicable.

Update a field when its source, derivation, absence behavior, transitive rule,
scenario coverage, or provenance changes.

Remove a field only after removing view fields, controls, sketch tokens, and
external outputs that display or use it.

### Views

Create a view when an actor needs to inspect state or initiate behavior. Views
belong to state-view slices and should be backed by read models.

When creating a view, define:

- view name;
- read model;
- visible fields;
- sketch tokens or wireframe tokens;
- controls;
- local states, filters, or navigation targets;
- error recovery behavior for issued commands.

Update a view when visible information, controls, navigation behavior, local
states, filters, or recovery behavior changes.

Remove a view only after removing workflow navigation targets, controls,
sketch references, and scenarios that depend on it.

### View Fields

Create a view field when a view displays or uses a read-model field. Every view
field should trace through the read model to an event attribute and original
provenance.

Update a view field when the displayed source, sketch token, bit encoding, or
provenance changes.

Remove a view field when the actor no longer sees or uses that datum. Check
that removing it does not hide a decision field required for command input.

### Controls

Create a control when an actor can take an action from a view. Controls may
issue commands and may navigate. Every control should appear in the sketch and
should provide every command input required by the command it issues.

Update a control when command target, input provisions, handled errors,
recovery behavior, navigation type, or sketch token changes.

Remove a control only after removing command transitions, navigation
transitions, scenarios, and command-input provisions that depend on it.

### Outcomes

Create an outcome when a slice can produce an externally relevant business
result. Outcomes should be backed by non-empty event sets and should not be
duplicates with different labels.

Update an outcome when the label, event set, or external relevance changes.

Remove an outcome when the business no longer distinguishes that result.
Remove or redirect workflow outcome transitions that reference it.

### Command Errors And Recovery

Create a command error for a local failure the command can return. The error
needs scenario coverage and every issuing view must describe recovery behavior.

Use command errors for local validation and recovery. Use outcomes for business
branches that matter beyond the command boundary.

Update a command error when its name, scenario coverage, recovery behavior, or
handling views change.

Remove a command error when the command can no longer return it. Remove error
references from contract scenarios and controls at the same time.

### Board Elements And Board Connections

Create board elements and connections to document causal shape. Boards should
show views, automations, external events, commands, read models, and events in
canonical lanes.

Use board connections to show real causal semantics:

- view/control to command;
- command to event;
- event to read model;
- event or external trigger to automation or translation;
- read model to view.

Update board facts when the causal path changes. Remove board facts when they
no longer match modeled declarations. A board should not preserve a
disconnected island just because it was useful during brainstorming.

### Automations

Create an automation slice or automation definition when the system reacts to a
trigger without direct human action. An automation should represent one
coherent reaction and issue one command per triggered operation.

Update an automation when trigger, command, handled errors, or reaction
description changes.

Remove an automation when the reaction is no longer valid. Remove board
connections, command references, and scenarios that depended on it.

### Translations

Create a translation slice or translation definition when data crosses an
external boundary and must be converted into domain meaning. Translation is
where external payloads become commands, events, or modeled facts.

Update a translation when the external event, payload contract, command, or
mapping semantics change.

Remove a translation when the boundary is removed or replaced. Check external
triggers, payload contracts, automations, commands, and scenarios before
removal.

### External Payloads

Create an external payload for data received from or sent to an outside system.
Payload fields need provenance and bit-level representation semantics when
they carry meaningful data.

Update a payload when the external contract changes. Keep versioning and
compatibility explicit when an outside system may still send the old shape.

Remove a payload only after removing translations, commands, event attributes,
external-trigger transitions, and data-flow facts that depend on it.

### Data-Flow Facts

Create data-flow facts to prove that meaningful data has source,
transformation, target, and bit-level encoding semantics. Use them especially
when a datum crosses boundaries, derives from other data, changes
representation, or is easy to lose.

Update a data-flow fact when the source, transformation, target, or encoding
semantics changes.

Remove a data-flow fact only when the datum no longer exists or the flow is now
represented more precisely elsewhere.

### Workflow-Owned Definitions And Evidence

Create workflow-owned definition facts when workflow composition needs to state
which slice owns which definition and how that definition participates in the
workflow. Create workflow transition evidence when transition legality depends
on modeled source and target evidence.

Update these facts when ownership, participation, source evidence, or target
evidence changes.

Remove them when the workflow no longer composes that definition or transition.
Then re-check reachability, ownership uniqueness, and transition legality.

### Entry Lifecycle Coverage

Create entry lifecycle coverage for workflows that can be entered in different
application states, such as fresh, initialized, unauthenticated,
authenticated, partially configured, or fully configured.

Update coverage when supported entry states or evidence changes. Remove
coverage only when the workflow no longer has that entry condition.

### Reviews And Readiness

Create clean review records only after a workflow has been checked against the
current formal model digest. Verification readiness is declared by `emc verify`
after Lean4 and Quint verification succeeds for an unchanged event frontier.

Update review evidence by recording a new review for the current digest rather
than editing old review records. Remove or ignore stale review evidence when
the model frontier changes.

## Change Triage

When a model feels wrong, classify the problem before changing it:

- Wrong external behavior: update acceptance scenarios, workflow path, slices,
  views, controls, outcomes, or transitions.
- Wrong internal decision: update command inputs, command contract scenarios,
  errors, emitted events, or observed streams.
- Wrong projection: update read models, read-model fields, derivation
  scenarios, absence behavior, or view fields.
- Wrong provenance: update command input sources, event attribute sources,
  external payload fields, data-flow facts, or information completeness facts.
- Wrong external boundary: update translations, external payloads,
  external-trigger transitions, or automations.
- Wrong proof shape: update formal facts, workflow evidence, data-flow facts,
  or the model structure that caused the failed check.

Prefer the smallest semantic edit that makes the model true. A smaller edit
keeps event history understandable and avoids churn in generated artifacts.

## Completion Checklist

Before treating a modeling increment as complete, verify all of these:

- The modeled behavior is stated from the right perspective.
- Acceptance scenarios describe external behavior only.
- Contract scenarios cover internal decisions, projections, derivations,
  translations, automations, and errors.
- Every command input has source and provenance.
- Every event attribute has source and provenance.
- Every read-model and view field traces back to original provenance.
- Every meaningful datum has data-flow and bit-level representation semantics
  when needed.
- Every workflow step is reachable and every transition is legal.
- Every command error has scenario coverage and recovery handling.
- Every external boundary has a payload contract and translation path.
- `emc check` passes.
- `emc verify` passes.
- Review-gate evidence is current for the workflow digest.
