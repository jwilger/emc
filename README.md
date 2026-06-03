# EMC

EMC is the Event Model Compiler. It helps teams create, change, check, and browse
business event models without asking every user to become a formal-methods
expert.

An event model describes how work moves through a business system: what starts a
workflow, what screens people use, what commands are issued, what events happen,
what state is remembered, and how the next step becomes possible. EMC keeps that
model synchronized across three views:

- a human-browsable site for product, design, engineering, and operations work;
- Lean4 artifacts for mechanical structure and invariant checks;
- Quint artifacts for executable transition and behavior checks.

Most users interact with EMC through the `emc` command or through MCP tools in
an editor or agent. Lean4 and Quint are guardrails behind the scenes.

## What EMC is for

EMC is for teams that want their business workflow model to be more than a
diagram or a pile of JSON files. It is meant to make the model actionable:

- create a project with a deterministic layout;
- add workflows, slices, and transitions through commands;
- validate event-modeling rules that are easy to miss in manual review;
- generate a browsable site for people who need to understand the model;
- run formal verification without exposing users to Lean4 or Quint syntax;
- expose the same operations through MCP for local agents and tools.

The model is the source of truth for the business software being described. EMC
does not use Lean4 and Quint only to describe generic event-modeling theory. It
generates Lean4 and Quint files for the actual workflows, slices, transitions,
and invariants in the project.

## Why event models need a compiler

Event models tend to drift. A workflow diagram says one thing, a browser view
says another thing, generated files go stale, and review notes stop matching the
current model. That drift makes the model less useful exactly when the team
starts depending on it.

EMC treats the model like compiled software:

1. Users make changes through deterministic commands or MCP tools.
2. EMC parses raw boundary input into semantic data types immediately.
3. The core computes the next model state without performing I/O.
4. The shell writes synchronized browser, Lean4, and Quint artifacts.
5. `emc check` rejects missing files and artifact drift.
6. `emc verify` runs the external proof and model-checking tools.

That gives the team repeatable guardrails instead of relying on everyone to
remember every modeling rule during review.

## You do not need to know Lean4 or Quint

Lean4 and Quint are important to EMC, but they are not the user interface.

Lean4 is used for static proof obligations: structure, invariants, and facts
that should always hold for the event model. Quint is used for executable
behavior: transitions, state changes, and temporal checks.

As an EMC user, you normally do not edit those files by hand. You use commands
such as `emc add workflow`, `emc connect workflow`, `emc check`, and
`emc verify`. EMC generates the formal artifacts and runs the tools. If a check
fails, the first message should tell you what project artifact or model rule
needs attention before you inspect Lean4 or Quint output directly.

## Quick start

From this repository, the easiest development build is:

```sh
nix develop
cargo build
```

Inside the development shell, run commands through Cargo or use the binary from
`target/debug`:

```sh
cargo run -- init --name "Repair Desk"
./target/debug/emc --help
```

You can also run the packaged executable through Nix:

```sh
nix run . -- --help
```

Create a new EMC project:

```sh
emc init --name "Repair Desk"
```

Create a workflow:

```sh
emc add workflow --slug open-ticket --name "Open ticket" --description "Actor opens a repair ticket."
```

Add the first business slice:

```sh
emc add slice \
  --workflow open-ticket \
  --slug capture-ticket \
  --name "Capture ticket" \
  --type state_view \
  --description "Actor enters repair ticket details."
```

Add another slice and connect the workflow:

```sh
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

Check that required project artifacts exist and synchronized generated files have
not drifted:

```sh
emc check
```

Run Lean4/Lake and Quint verification through EMC:

```sh
emc verify
```

Generate the browsable site:

```sh
emc generate site --output site
```

Start the local MCP server over stdio:

```sh
emc mcp stdio
```

For local HTTP MCP access:

```sh
emc mcp http --host 127.0.0.1 --port 7331
```

When binding HTTP beyond the local machine, provide an auth token:

```sh
emc mcp http --host 0.0.0.0 --port 7331 --auth-token "$EMC_MCP_TOKEN"
```

## A normal modeling loop

Most day-to-day work follows this loop:

1. Add or change the model through `emc` commands or MCP tools.
2. Run `emc check` to confirm browser, Lean4, and Quint artifacts are present
   and synchronized.
3. Run `emc validate <target>` when changing event-model JSON rule coverage or
   when checking a specific workflow or slice file.
4. Run `emc review gate --workflow <slug>` before treating a workflow as
   reviewed and ready to advance.
5. Run `emc verify` to execute the generated Lean4 and Quint verification entry
   points.
6. Run `emc generate site --output site` when people need to browse the current
   model.

If `emc check` fails, fix synchronization drift before running `emc verify`.
Verification assumes the generated formal artifacts match the browser model.

## Mental model

An EMC project has a project name, a set of workflows, and generated artifacts.

A workflow is a user-visible business journey, such as opening a repair ticket,
granting organization access, or processing an order. It answers: "What path can
the business process take?"

A slice is one meaningful part of that workflow. EMC currently models these
slice kinds:

- `state_change`: records a business decision or fact by emitting events.
- `state_view`: shows remembered state to an actor.
- `automation`: reacts to a trigger without a person driving the step.
- `translation`: turns an external signal into modeled business information.

A transition connects one slice to another. EMC supports command, event,
navigation, external-trigger, and workflow-exit transitions. The transition is
part of the model because the reason a workflow can move forward matters as much
as the target step.

An invariant is a rule that should remain true about the generated model. EMC
generates Lean4 and Quint artifacts so those invariants can be checked
mechanically for the actual business workflows and slices in the project.

EMC writes three synchronized representations:

- browser JSON for the generated site;
- Lean4 modules under `model/lean`;
- Quint modules under `model/quint`.

The browser representation is for humans. The Lean4 and Quint representations
are for mechanical guardrails. All three should describe the same business
event model.

## Project layout

`emc init` creates the project layout that later commands use:

```text
emc.toml
model/
  browser/
    data/
      index.json
      workflows/
      slices/
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

In normal use, edit the model through EMC commands or MCP tools. Treat generated
browser, Lean4, and Quint files as synchronized artifacts. Manual edits may be
rejected by `emc check` when they drift from the rest of the model.

## CLI commands

Current user-facing commands include:

```sh
emc init --name <project-name>
emc list workflows
emc list slices
emc list transitions
emc show workflow <workflow-slug>
emc show slice <slice-slug>
emc add workflow --slug <slug> --name <name> --description <description>
emc update workflow --slug <slug> --description <description>
emc update workflow --slug <slug> --name <name>
emc update slice --slug <slice-slug> --description <description>
emc update slice --slug <slice-slug> --type <kind>
emc add slice --workflow <workflow-slug> --slug <slug> --name <name> --type <kind> --description <description>
emc connect workflow --workflow <workflow-slug> --from <slice-slug> --to <slice-slug> --via <kind> --name <trigger-name>
emc connect workflow --workflow <workflow-slug> --from <slice-slug> --to-workflow <workflow-slug> --via outcome --name <outcome-name> --reason <rationale>
emc validate <eventmodel-json-file>
emc review gate --workflow <workflow-slug>
emc check
emc verify
emc generate site --output <directory>
emc gherkin list --suite <suite>
emc gherkin run --suite <suite>
emc gherkin run --all
emc mcp stdio
emc mcp http
```

The command parser is intentionally strict. If a command fails with a usage or
parse error, check the argument order shown above.

## What `check`, `validate`, and `verify` mean

These commands answer different questions:

- `emc check`: Are the generated browser, Lean4, and Quint artifacts present,
  canonical, and synchronized?
- `emc validate <target>`: Does an event-model JSON file or directory satisfy
  the event-modeling rule suite?
- `emc verify`: Do the generated Lean4 and Quint verification entrypoints pass
  through the pinned external tools?

Run `emc check` first when something looks wrong. It catches local drift and
missing generated files before the formal tools need to run.

## MCP access

EMC exposes read, validation, verification, generation, review-gate, and
mutation operations over MCP. This lets local tools and agents work with the
same deterministic command core as the CLI.

Use stdio for local editor or agent integrations:

```sh
emc mcp stdio
```

Use HTTP for container or networked setups:

```sh
emc mcp http --host 127.0.0.1 --port 7331
```

Localhost HTTP requests are protected with Origin checks. Non-local binds require
an explicit bearer token so an exposed MCP server is not accidentally left open.

## Browser output

`emc generate site --output <directory>` copies the browser application and the
current model data into a browsable site. The generated browser data shape is
stable and intended for the EMC browser:

```text
data/index.json
data/workflows/*.eventmodel.json
data/slices/*.eventmodel.json
```

The browser is meant for people who need to inspect workflows, timeline order,
branch cards, source chains, control effects, navigation targets, and review
overlays without reading generated formal artifacts.

## Validation and review gates

`emc validate <file>` checks the event-model Gherkin rule suite. These rules
cover structure, source provenance, slice ownership, board
connections, workflow reachability, transitions, views, controls, information
flow, outcomes, errors, and review-related behavior.

`emc review gate --workflow <workflow-slug>` checks that a workflow has a current
clean review record for the model digest. It blocks stale reviews, missing
categories, non-clean categories, unresolved mandatory findings, and workflows
that changed after findings were corrected but before a clean follow-up review.

## Engineering guardrails

EMC is built with strict mechanical guardrails:

- Rust warnings are treated as errors.
- Clippy lint policy is checked in and enforced.
- The core follows functional core / imperative shell architecture.
- Core logic describes I/O as step/trampoline variant effects.
- Shell modules interpret file, process, network, stdio, and environment I/O.
- Boundary data is parsed immediately into semantic data types.
- Semantic data types are implemented with `nutype` where practical.
- Primitive and structural DTOs belong at I/O boundaries, not in public core APIs.
- Architecture tests reject direct core I/O and primitive-bearing public core APIs.

These rules are part of the product. They keep EMC deterministic enough to be
useful as a compiler for business models rather than just a script that edits
files.

## Development

Run the strict local gate:

```sh
just ci
```

`just ci` runs formatting, clippy, tests, and build with Rust warnings treated
as errors. It is the fast default gate.

Mutation testing is available as an explicit local engineering gate. It is not
part of `just ci`, because it is slower and should be used at the right cadence:

```sh
just mutants-diff
just mutants-core
just mutants-full
```

Use `just mutants-diff` before committing meaningful Rust changes. It runs
mutation testing only against the current Rust source diff. Use
`just mutants-core` after changes to workflow, slice, or connection semantics.
Use `just mutants-full` before larger milestones or releases.

The Nix flake builds the `emc` package, wraps it with Lean4/Lake and Quint on
`PATH`, and provides a Docker-compatible image for container use.

The Nix gate is:

```sh
nix flake check
```

That builds the packaged binary, runs package smoke checks, and builds the
Docker-compatible image.

## Current status

EMC is under active development. It already has a Rust CLI, MCP stdio and HTTP
entrypoints, validation fixtures, review-gate checks,
browser generation, Lean4 and Quint artifact emission, strict lint guardrails,
and package smoke tests.

The long-term target is a single executable that lets users create and maintain
business event models while `emc check` and `emc verify` mechanically enforce
that browser, Lean4, and Quint representations stay synchronized.
