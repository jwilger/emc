<!-- Copyright 2026 John Wilger -->

# Modeled Element Mutations Ledger

This ledger tracks #159 across multiple PRs.

## Completed increments

- PR #168: added event-sourced CLI and MCP update/remove support for slice
  scenarios. Covered acceptance scenarios in behavior tests and reused the same
  scenario payload path for contract scenarios.
- PR #169: release-plz v0.1.8 release PR for PR #168 was merged.
- PR #170: added event-sourced CLI and MCP update/remove support for
  command definitions. Covered behavior through synchronized Lean4/Quint
  artifact assertions for both CLI and MCP entry points.
- PR #172: added event-sourced CLI and MCP update/remove support for
  event definitions. Covered behavior through synchronized Lean4/Quint artifact
  assertions for both CLI and MCP entry points.
- PR #174: added event-sourced CLI and MCP update/remove support for read
  model definitions. Covers behavior through synchronized Lean4/Quint artifact
  assertions for both CLI and MCP entry points.
- PR #176: added event-sourced CLI and MCP update/remove support for view
  definitions. Covers behavior through synchronized Lean4/Quint artifact
  assertions for both CLI and MCP entry points.
- PR #178: added event-sourced CLI and MCP update/remove support for view
  controls. Covers behavior through synchronized Lean4/Quint artifact assertions
  for both CLI and MCP entry points.
- PR #180: added event-sourced CLI and MCP update/remove support for
  outcome definitions. Covers behavior through synchronized Lean4/Quint artifact
  assertions for both CLI and MCP entry points.
- PR #182: added event-sourced CLI and MCP update/remove support for
  automation definitions. Covers behavior through synchronized Lean4/Quint
  artifact assertions for both CLI and MCP entry points.
- PR #184: added event-sourced CLI and MCP update/remove support for
  translation definitions. Covers behavior through synchronized Lean4/Quint
  artifact assertions for both CLI and MCP entry points.
- PR #186: added event-sourced CLI and MCP update/remove support for
  external payload definitions. Covers behavior through synchronized Lean4/Quint
  artifact assertions for both CLI and MCP entry points.
- Current branch: adds event-sourced CLI and MCP update/remove support for
  board elements. Covers behavior through synchronized Lean4/Quint
  artifact assertions for both CLI and MCP entry points.

## Current PR boundary

- This PR should broaden beyond scenarios, commands, events, read models, views,
  controls, outcomes, automations, translations, and external payload
  definitions to board elements without closing #159. Keep #159 open until all
  modeled element families listed in the issue have CLI and MCP update/remove
  coverage.

## Remaining modeled element targets

- Workflow and slice lifecycle: already have update/remove coverage for primary
  workflow and slice fields; confirm MCP/CLI parity and any gaps before closing.
- Workflow transitions: remove exists; update likely means replace by removing
  and adding only if that is semantically valid, otherwise add explicit update.
- Workflow evidence facts: workflow outcomes, command errors, owned definitions,
  transition evidence, entry lifecycle coverage/state.
- Slice-owned definitions: commands are covered by PR #170, events are covered
  by PR #172, read models are covered by PR #174, views are covered by PR #176,
  controls are covered by PR #178, outcomes are covered by PR #180, automations
  are covered by PR #182, translations are covered by PR #184, external
  payloads are covered by PR #186, and board elements are covered by the current
  branch. Remaining families are board connections and data-flow facts.
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
- `cargo test --test update_slice event_definition`
- `cargo test --test mcp_stdio event_definition`
- `cargo test --test update_slice read_model_definition`
- `cargo test --test mcp_stdio read_model_definition`
- `cargo test --test update_slice view_definition`
- `cargo test --test mcp_stdio view_definition`
- `cargo test --test update_slice control_definition`
- `cargo test --test mcp_stdio control_definition`
- `cargo test --test update_slice outcome_definition`
- `cargo test --test mcp_stdio outcome_definition`
- `cargo test --test update_slice automation_definition`
- `cargo test --test mcp_stdio automation_definition`
- `cargo test --test update_slice translation_definition`
- `cargo test --test mcp_stdio translation_definition`
- `cargo test --test update_slice external_payload_definition`
- `cargo test --test mcp_stdio external_payload_definition`
- `cargo test --test update_slice board_element`
- `cargo test --test mcp_stdio board_element`

## Next likely increment

Extend the same update/remove pattern to board connections, then continue
through data-flow facts.
