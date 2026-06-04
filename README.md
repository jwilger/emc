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
  lean/
    lakefile.lean
    lean-toolchain
    <ProjectModule>.lean
    slices/
  quint/
    quint.json
    <ProjectModule>.qnt
    slices/
reviews/
```

EMC-managed model artifacts live under `model/lean` and `model/quint`.

## CLI commands

Current user-facing commands include:

```sh
emc init --name <project-name>
emc list workflows
emc list slices
emc list transitions
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
documented in:

```text
docs/event-model/formal-modeling-rules.md
```

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

## Current status

EMC has a Rust CLI, MCP stdio and HTTP entrypoints, review-gate checks, Lean4
and Quint artifact emission, strict lint guardrails, and package smoke tests.
The formal metamodel is still being expanded toward the rule inventory in
`docs/event-model/formal-modeling-rules.md`.
