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
- PR #188: added event-sourced CLI and MCP update/remove support for
  board elements. Covers behavior through synchronized Lean4/Quint
  artifact assertions for both CLI and MCP entry points.
- PR #190: added event-sourced CLI and MCP update/remove support for
  board connections. Covers behavior through synchronized Lean4/Quint artifact
  assertions for both CLI and MCP entry points.
- PR #192: added event-sourced CLI and MCP update/remove support for
  data-flow facts. Covers behavior through synchronized Lean4/Quint artifact
  assertions, MCP entry points, and exported-event replay.
- PR #194: added event-sourced CLI and MCP update/remove support for
  workflow outcome facts. Covers behavior through synchronized Lean4/Quint
  artifact assertions, MCP entry points, and exported-event replay.
- PR #196: added event-sourced CLI and MCP update/remove support for
  workflow command-error facts. Covers behavior through synchronized
  Lean4/Quint artifact assertions, MCP entry points, and exported-event replay.
- PR #198: added event-sourced CLI and MCP update support for workflow
  transitions. Covers behavior through synchronized Lean4/Quint artifact
  assertions, MCP entry points, and exported-event replay.
- PR #200: added event-sourced CLI and MCP update/remove support for
  workflow-owned definition facts. Covers behavior through synchronized
  Lean4/Quint artifact assertions, MCP entry points, and exported-event replay.
- PR #202: added event-sourced CLI and MCP update/remove support for
  workflow transition evidence facts. Covers behavior through synchronized
  Lean4/Quint artifact assertions, MCP entry points, and exported-event replay.
- PR #204: added event-sourced CLI and MCP remove support for workflow
  entry lifecycle coverage requirements and update/remove support for workflow
  entry lifecycle state facts. Covers behavior through synchronized Lean4/Quint
  artifact assertions, MCP entry points, and exported-event replay.
- PR #206: fixed CLI contract scenario update parsing and confirmed contract
  scenario update/remove parity through CLI, MCP, and exported-event replay
  behavior tests.

## Current PR boundary

- No active PR boundary. The #159 modeled element mutation work is complete.

## Remaining modeled element targets

- Workflow and slice lifecycle: primary workflow and slice fields have CLI/MCP
  update/remove coverage.
- Workflow transitions: remove is covered by earlier workflow PRs, and update
  is covered by PR #198.
- Workflow evidence facts: entry lifecycle coverage/state are covered by PR
  #204. Workflow outcomes are covered by PR #194, command errors are covered by
  PR #196, owned definitions are covered by PR #200, and transition evidence is
  covered by PR #202.
- Slice-owned definitions: commands are covered by PR #170, events are covered
  by PR #172, read models are covered by PR #174, views are covered by PR #176,
  controls are covered by PR #178, outcomes are covered by PR #180, automations
  are covered by PR #182, translations are covered by PR #184, external
  payloads are covered by PR #186, board elements are covered by PR #188, board
  connections are covered by PR #190, and data-flow facts are covered by PR
  #192.
- Scenarios: acceptance scenario update/remove is covered by PR #168, and
  contract scenario update/remove completion evidence is covered by PR #206.

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
- `cargo test --test update_slice board_connection`
- `cargo test --test mcp_stdio board_connection`
- `cargo test --test update_slice data_flow`
- `cargo test --test mcp_stdio data_flow`
- `cargo test --test event_log_export bit_level_data_flow`
- `cargo test --test connect_workflow workflow_outcome`
- `cargo test --test mcp_connect_workflow workflow_outcome`
- `cargo test --test event_log_export workflow_outcome`
- `cargo test --test connect_workflow workflow_command_error`
- `cargo test --test mcp_connect_workflow workflow_command_error`
- `cargo test --test event_log_export workflow_command_error`
- `cargo test --test connect_workflow update_transition`
- `cargo test --test mcp_connect_workflow updates_workflow_transition`
- `cargo test --test event_log_export workflow_transition_updates`
- `cargo test --test connect_workflow workflow_owned_definition`
- `cargo test --test mcp_connect_workflow workflow_owned_definition`
- `cargo test --test event_log_export workflow_owned_definition`
- `cargo test --test connect_workflow workflow_transition_evidence`
- `cargo test --test mcp_connect_workflow workflow_transition_evidence`
- `cargo test --test event_log_export workflow_transition_evidence`
- `cargo test --test connect_workflow workflow_entry_lifecycle`
- `cargo test --test mcp_connect_workflow workflow_entry_lifecycle`
- `cargo test --test event_log_export workflow_entry_lifecycle`
- `cargo test --test update_slice contract_scenario`
- `cargo test --test mcp_stdio contract_slice_scenario`
- `cargo test --test event_log_export contract_scenario`

## Closure Status

Close #159 after verifying Forgejo still shows no other open `1.0.0`
milestone implementation issue.
