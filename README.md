<!-- Copyright 2026 John Wilger -->

# EMC

EMC is the Event Model Compiler. It helps local agents and developers author
business event models as mechanically checked Lean4 and Quint artifacts.

Lean4 and Quint are the authoritative model forms. Browser bundles, generated
HTML sites, JSON projections, and duplicate Rust or JavaScript event-model
validators are not correctness sources.

## What EMC is for

EMC is for building event models that can be checked as formal artifacts:

- create a deterministic project layout;
- add workflows, slices, and transitions through CLI or MCP tools;
- keep Lean4 and Quint artifacts synchronized;
- run project drift checks;
- run Lean4/Lake and Quint verification;
- record review-gate evidence tied to the formal artifact digest.

The intended MCP workflow is that an LLM authors and updates the model directly
through EMC operations that emit Lean4 and Quint. A model is acceptable only when
`emc check` and `emc verify` pass.

## Current Status

EMC has a Rust CLI, MCP stdio and HTTP entrypoints, review-gate checks, Lean4
and Quint artifact emission, exported event replay for project/workflow/slice,
slice-fact, review, conflict-resolution, and workflow-readiness events, strict
lint guardrails, and package smoke tests.
The formal metamodel encodes the current rule inventory in
[docs/event-model/formal-modeling-rules.md](https://git.johnwilger.com/Slipstream/emc/src/branch/main/docs/event-model/formal-modeling-rules.md).

## Why Lean4 and Quint

Rust is useful for construction discipline in EMC itself: parsing CLI and MCP
inputs, preserving authored facts while rewriting artifacts, preventing invalid
tooling states such as empty slugs, and running the formal verification tools.
Those checks prove properties of the authoring engine.

Lean4 and Quint represent the event model itself. The model carries the
definitions, proof obligations, state-machine structure, and invariants for
workflow reachability, transition legality, scenario completeness, provenance,
source chains, and bit-level data-flow completeness. Lean4 proves static model
properties; Quint typechecks and verifies behavioral invariants.

Lean4 is a programming language plus a proof checker. You write definitions,
statements, and proofs, and Lean checks whether each proof really proves the
statement.

```lean
theorem two_plus_two : 2 + 2 = 4 := by
  rfl
```

Lean is not testing that `2 + 2 = 4`; it is mechanically checking that the
statement follows from the rules of logic and the definitions involved. A
normal test says, "I tried some examples and they worked." A Lean theorem says,
"given these definitions, this claim is impossible to violate."

In EMC, Lean artifacts define the event model facts and the rules those facts
must satisfy, then include machine-checkable theorems proving that the current
model satisfies those rules.

```lean
def commandEmitsKnownEvents : Bool := ...

theorem commandEmitsKnownEventsIsStable :
  commandEmitsKnownEvents = true := rfl
```

If the model changes and the rule no longer holds, Lean verification stops
accepting the artifact.

Quint is a modeling language for systems that change over time. It describes
the possible states of a system, the actions that can move it from one state to
another, and invariants that must always hold.

```quint
module Counter {
  var count: int

  action Init = count' = 0

  action Increment = count' = count + 1

  val countNeverNegative = count >= 0
}
```

In Quint, `count` is system state, `Init` describes how the system starts,
`Increment` describes an allowed state transition, `count'` is the next value
of `count`, and `countNeverNegative` is an invariant to check. Quint asks
whether an allowed model behavior can ever violate that invariant.

In EMC, Quint artifacts are useful for workflow and state behavior: which
slices exist, which transitions are allowed, whether workflow steps are
reachable, whether command transitions target the right owner, whether external
triggers declare payload contracts, and whether forbidden dependencies such as
read models feeding commands can occur.

Lean4 is strongest at exact logical structure and static proof obligations.
Quint is strongest at workflows, state machines, transitions, reachability, and
behavioral invariants. Together, they provide different mechanical checks over
the same event model.

Keeping those obligations in Lean4 and Quint prevents Rust from becoming a
duplicate semantic validator. Rust may reject malformed edits and artifact
drift, but event-model correctness is accepted only from the formal artifacts
mechanically verifying.

## Quick start

From this repository:

```sh
nix develop
cargo build
```

Create a new EMC project:

```sh
emc init --name "Repair Desk"
```

Create a workflow:

```sh
emc add workflow --slug open-ticket --name "Open ticket" --description "Actor opens a repair ticket."
```

Add and connect slices:

```sh
emc add slice \
  --workflow open-ticket \
  --slug capture-ticket \
  --name "Capture ticket" \
  --type state_view \
  --description "Actor enters repair ticket details."

emc add slice \
  --workflow open-ticket \
  --slug review-ticket \
  --name "Review ticket" \
  --type state_view \
  --description "Agent reviews the captured ticket."

emc connect workflow \
  --workflow open-ticket \
  --from capture-ticket \
  --to review-ticket \
  --via navigation \
  --name review-ticket-screen
```

Check synchronized formal artifacts:

```sh
emc check
```

Run Lean4/Lake and Quint verification:

```sh
emc verify
```

`emc verify` expects `lake` and `quint` on `PATH`. The Nix package and
development shell provide those tools.

## Project layout

`emc init` creates:

```text
emc.toml
model/
  events/
    .lock
    projection.fingerprint
    v1/
      <event-id>.json
  lean/
    lakefile.lean
    lean-toolchain
    <ProjectModule>.lean
    slices/
  quint/
    <ProjectModule>.qnt
    slices/
reviews/
```

EMC-managed model artifacts live under `model/lean` and `model/quint`.
Mutating project, workflow, slice, slice-fact, conflict-resolution, and
clean-review operations export domain events under `model/events/v1`.
`emc check` can rebuild `emc.toml`, generated Lean4 and Quint artifacts, and
review records from those exported events. The projection fingerprint records
the event set used for the latest rebuild.

`emc verify` runs Lean4/Lake and Quint against the current projected event
frontier. After successful verification, EMC appends workflow-scoped
`WorkflowReadinessDeclared` events recording the verified projection
fingerprint, formal model content digest, verifier identity, timestamp, and
optional review event reference. If the exported event frontier changes during
verification, readiness is not appended and `emc verify` must be retried.

EMC also initializes an operational SQLite event-store cache outside the
repository at `$XDG_STATE_HOME/emc/projects/<sha256-realpath>/events.sqlite3`.
Set `EMC_EVENT_STORE_PATH` to override that location. The exported JSON events
are the repository-tracked interchange format; the SQLite path is an
operational cache location.

## CLI commands

Current user-facing commands include:

```sh
emc init --name <project-name>
emc list workflows
emc list slices
emc list transitions
emc list conflicts
emc show workflow <workflow-slug>
emc show workflow --slug <workflow-slug>
emc show slice <slice-slug>
emc show slice --slug <slice-slug>
emc add workflow --slug <slug> --name <name> --description <description>
emc update workflow --slug <slug> --description <description>
emc update workflow --slug <slug> --name <name>
emc remove workflow --slug <workflow-slug>
emc add slice --workflow <workflow-slug> --slug <slug> --name <name> --type <kind> --description <description>
emc update slice --slug <slice-slug> --description <description>
emc update slice --slug <slice-slug> --type <kind>
emc update slice --slug <slice-slug> --name <name>
emc remove slice --slug <slice-slug>
emc connect workflow --workflow <workflow-slug> --from <slice-slug> --to <slice-slug> --via <kind> --name <trigger-name>
emc connect workflow --workflow <workflow-slug> --from <slice-slug> --to-workflow <workflow-slug> --via outcome --name <outcome-name> --reason <rationale>
emc remove transition --workflow <workflow-slug> --from <slice-slug> --to <slice-slug> --via <kind> --name <trigger-name>
emc remove transition --workflow <workflow-slug> --from <slice-slug> --to-workflow <workflow-slug> --via outcome --name <outcome-name>
emc resolve conflict --id <conflict-id> --choose-event <event-id>
emc review gate --workflow <workflow-slug>
emc review record --workflow <workflow-slug> --reviewer <reviewer-id> --reviewed-at <timestamp>
emc check
emc verify
emc gherkin list --suite <suite>
emc gherkin run --suite <suite>
emc gherkin run --all
emc mcp stdio
emc mcp http --host <host> --port <port>
emc mcp http --host <host> --port <port> --auth-token <token>
```

The parser is intentionally strict. If a command fails with a usage or parse
error, check the argument order shown above.

## MCP access

Start the local MCP server over stdio:

```sh
emc mcp stdio
```

For local HTTP MCP access:

```sh
emc mcp http --host 127.0.0.1 --port 7331
```

Non-local HTTP binds require a bearer token:

```sh
emc mcp http --host 0.0.0.0 --port 7331 --auth-token "$EMC_MCP_TOKEN"
```

Current MCP tools are:

```text
init_project
list_workflows
list_slices
list_transitions
show_workflow
show_slice
check_project
verify_project
review_gate
record_clean_review
add_workflow
add_slice
update_workflow
update_workflow_name
update_slice
update_slice_kind
update_slice_name
remove_slice
remove_workflow
connect_workflow
remove_transition
list_conflicts
resolve_conflict
```

`show_workflow` and `show_slice` return the Lean4 and Quint artifacts for the
requested model element.

## Checking and Verification

- `emc check`: confirms project files and generated Lean4/Quint artifacts are
  present, canonical, and synchronized.
- `emc verify`: runs the generated Lean4/Lake and Quint verification entry
  points.

Rust command preconditions reject edits such as duplicate workflow slugs,
unknown transition targets, and stale artifact drift. They are editing
guardrails, not an independent semantic validator for event-model correctness.

## Formal Modeling Rules

The acceptance checklist for the formal metamodel and MCP authoring workflow is
documented in
[docs/event-model/formal-modeling-rules.md](https://git.johnwilger.com/Slipstream/emc/src/branch/main/docs/event-model/formal-modeling-rules.md).

Information completeness means every datum that flows through the modeled system
is represented down to source, transformation or projection, target, and
bit-level encoding semantics.

## Development

Run the strict local gate:

```sh
just ci
```

`just ci` runs formatting, clippy, tests, and build with Rust warnings treated
as errors.

Useful direct checks:

```sh
cargo fmt
cargo test
cargo clippy --all-targets --all-features -- -D warnings
lake build
quint test model/quint/*.qnt
```

Mutation testing is available as an explicit local engineering gate:

```sh
just mutants-diff
just mutants-core
just mutants-full
```

The Nix gate is:

```sh
nix flake check
```

## Release Automation

EMC releases are managed by release-plz in Forgejo Actions. The release
workflow runs after pushes to `main`:

- regular feature and fix PRs merged to `main` create or update a
  `release-plz-*` release PR with the next crate version and changelog;
- the release PR goes through the normal PR CI and review gates;
- merging the release PR to `main` publishes unpublished crates, creates the git
  tag, and creates the Forgejo release.

The Forgejo repository must provide these secrets:

- `RELEASE_PLZ_TOKEN`: Forgejo application token with repository read/write
  access and issue read/write access so release-plz can create tags, releases,
  labels, and release PRs.
- `CARGO_REGISTRY_TOKEN`: crates.io token with `publish-new` and
  `publish-update` scopes.

`release-plz.toml` sets `release_always = false`, so publishing is tied to the
merged release PR. The Forgejo server must support the commit-to-PR API used by
release-plz to recognize release PR merge commits.
