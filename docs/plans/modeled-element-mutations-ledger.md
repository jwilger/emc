<!-- Copyright 2026 John Wilger -->

# Modeled Element Mutations Ledger

This ledger tracks #159 across multiple PRs.

## Completed increments

- PR #168: added event-sourced CLI and MCP update/remove support for slice
  scenarios. Covered acceptance scenarios in behavior tests and reused the same
  scenario payload path for contract scenarios.
- PR #169: release-plz v0.1.8 release PR for PR #168 was merged.
- Current branch: adds event-sourced CLI and MCP update/remove support for
  command definitions. Covered behavior through synchronized Lean4/Quint
  artifact assertions for both CLI and MCP entry points.

## Current PR boundary

- This PR should broaden beyond scenarios to command definitions without closing
  #159. Keep #159 open until all modeled element families listed in the issue
  have CLI and MCP update/remove coverage.

## Remaining modeled element targets

- Workflow and slice lifecycle: already have update/remove coverage for primary
  workflow and slice fields; confirm MCP/CLI parity and any gaps before closing.
- Workflow transitions: remove exists; update likely means replace by removing
  and adding only if that is semantically valid, otherwise add explicit update.
- Workflow evidence facts: workflow outcomes, command errors, owned definitions,
  transition evidence, entry lifecycle coverage/state.
- Slice-owned definitions: commands are covered by the current branch; remaining
  families are events, read models, views, controls, outcomes, automations,
  translations, external payloads, board elements, board connections, and
  data-flow facts.
- Scenario follow-up: add tests for contract scenario update/remove if needed by
  final completion evidence.

## Focused verification already run

- `just copyright-headers`
- `just fmt`
- `just clippy`
- `cargo test --test update_slice scenario_`
- `cargo test --test mcp_stdio slice_scenario`
- `cargo test --test update_slice command_definition`
- `cargo test --test mcp_stdio command_definition`

## Next likely increment

Extend the command-definition pattern to the remaining slice-owned definition
families, starting with event definitions or read model definitions because they
feed several downstream modeling facts.
