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
- import an existing EMC event model while preserving browser compatibility;
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

Create a new EMC project:

```sh
emc init --name "Repair Desk"
```

Import an existing EMC event model:

```sh
emc import emc --source ../emc/docs/event-model
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

## Mental model

An EMC project has a project name, a set of workflows, and generated artifacts.

A workflow is a user-visible business journey, such as granting organization
access or repairing a device. A slice is one meaningful part of that workflow:
a state change, a state view, an automation, or a translation from an external
signal. A transition connects one slice to another through a command, event,
navigation action, or external trigger.

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
  quint/
    quint.json
    <ProjectModule>.qnt
reviews/
```

In normal use, edit the model through EMC commands or MCP tools. Treat generated
browser, Lean4, and Quint files as synchronized artifacts. Manual edits may be
rejected by `emc check` when they drift from the rest of the model.

## CLI commands

Current user-facing commands include:

```sh
emc init --name <project-name>
emc import emc --source <emc-event-model-directory>
emc list workflows
emc show workflow <workflow-slug>
emc add workflow --slug <slug> --name <name> --description <description>
emc update workflow --slug <slug> --description <description>
emc add slice --workflow <workflow-slug> --slug <slug> --name <name> --type <kind> --description <description>
emc connect workflow --workflow <workflow-slug> --from <slice-slug> --to <slice-slug> --via <kind> --name <trigger-name>
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
compatible with EMC-style event-model browsing:

```text
data/index.json
data/workflows/*.eventmodel.json
data/slices/*.eventmodel.json
```

The browser is meant for people who need to inspect workflows, timeline order,
branch cards, source chains, control effects, navigation targets, and review
overlays without reading generated formal artifacts.

## Validation and review gates

`emc validate <file>` checks event-model JSON rules ported from EMC's Gherkin
suite. These rules cover structure, source provenance, slice ownership, board
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

The Nix flake builds the `emc` package, wraps it with Lean4/Lake and Quint on
`PATH`, and provides a Docker-compatible image for container use.

## Current status

EMC is under active development. It already has a Rust CLI, MCP stdio and HTTP
entrypoints, EMC import compatibility, validation fixtures, review-gate checks,
browser generation, Lean4 and Quint artifact emission, strict lint guardrails,
and package smoke tests.

The long-term target is a single executable that lets users create and maintain
business event models while `emc check` and `emc verify` mechanically enforce
that browser, Lean4, and Quint representations stay synchronized.
