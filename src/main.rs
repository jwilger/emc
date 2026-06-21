// Copyright 2026 John Wilger

#![cfg_attr(
    test,
    allow(
        dead_code,
        reason = "items used only by the binary are unused when compiling the test harness"
    )
)]

use std::env;
use std::process::ExitCode;

use clap::{Arg, Command as ClapCommand};
use emc::modeling_process_guide;
mod command;
mod core;
mod io;
mod mcp;
mod shell;

use crate::core::connection::{
    ConnectionKind, WorkflowConnection, WorkflowTransitionRemoval, WorkflowTransitionUpdate,
};
use crate::core::effect::{ArtifactDigest, ChosenEventId, EventConflictId};
use crate::core::formal_slice_facts::{
    CommandErrorDefinitions, CommandErrorNames, CommandInputProvenanceChain, CommandInputSource,
    CommandObservedStreams, EmittedEventNames, NewAutomationDefinition, NewBitLevelDataFlow,
    NewBoardConnection, NewBoardElement, NewCommandDefinition, NewCommandErrorDefinition,
    NewCommandInput, NewControlDefinition, NewControlInputProvision, NewEventAttribute,
    NewEventDefinition, NewExternalPayloadDefinition, NewNavigationTarget, NewOutcomeDefinition,
    NewReadModelDefinition, NewReadModelField, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition, NewViewField, OutcomeEventNames, ReadModelDerivationSourceFields,
    ReadModelFieldSource, ReadModelRelationshipFields, ScenarioKind, ScenarioStreamNames,
    ViewControls, ViewFilters, ViewLocalStates,
};
use crate::core::gherkin::GherkinSuite;
use crate::core::modeling_enums::MODELING_ENUMS;
use crate::core::project::ProjectName;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    AutomationName, BoardElementName, CommandInputSourceKind, CommandName, ControlName,
    ControlRecoveryBehavior, EventAttributeSourceField, EventAttributeSourceName, EventName,
    ModelDescription, ModelName, OutcomeLabelName, PayloadContractName, ReadModelFieldSourceKind,
    ReadModelName, ReviewTimestamp, ReviewerId, ScenarioName, SketchToken, SliceSlug, StreamName,
    TranslationName, ViewName, WorkflowCommandErrorRecord, WorkflowEntryLifecycleStateRecord,
    WorkflowEventParticipation, WorkflowOutcomeRecord, WorkflowOwnedDefinitionKind,
    WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord, WorkflowSlug,
    WorkflowTransitionEndpoint, WorkflowTransitionEvidenceNavigationEndpoints,
    WorkflowTransitionEvidenceRecord, WorkflowTransitionKind, WorkflowViewRole,
};
use crate::core::workflow::NewWorkflow;
use crate::io::dto::{
    parse_automation_name, parse_automation_reaction_description, parse_automation_trigger_name,
    parse_bit_encoding_semantics, parse_board_connection_endpoint,
    parse_board_connection_endpoint_kind, parse_board_element_declared_name,
    parse_board_element_kind, parse_board_element_name, parse_board_lane_id,
    parse_command_error_name, parse_command_error_names, parse_command_error_recovery_kind,
    parse_command_input_source_description, parse_command_input_source_kind, parse_command_name,
    parse_connection_kind, parse_contract_kind_name, parse_control_name,
    parse_control_recovery_behavior, parse_covered_definition_name, parse_data_flow_source,
    parse_data_flow_source_kind, parse_data_flow_target, parse_datum_name, parse_datum_names,
    parse_event_attribute_name, parse_event_attribute_source_field,
    parse_event_attribute_source_kind, parse_event_attribute_source_name, parse_event_name,
    parse_event_names, parse_generated_event_attribute_source_kind, parse_gherkin_suite,
    parse_model_description, parse_model_name, parse_navigation_target_name,
    parse_navigation_target_names, parse_navigation_target_type, parse_outcome_label_name,
    parse_payload_contract_name, parse_project_name, parse_provenance_description,
    parse_read_model_derivation_rule, parse_read_model_field_source_kind, parse_read_model_name,
    parse_read_model_transitive_rule, parse_review_timestamp, parse_reviewer_id,
    parse_scenario_kind, parse_scenario_name, parse_scenario_step_text,
    parse_singleton_repeat_behavior, parse_sketch_token, parse_slice_kind, parse_slice_slug,
    parse_source_chain_hops, parse_stream_name, parse_stream_names, parse_transformation_semantics,
    parse_transition_trigger_name, parse_translation_external_event_name, parse_translation_name,
    parse_view_field_name, parse_view_field_source_kind, parse_view_name,
    parse_workflow_entry_lifecycle_evidence_text, parse_workflow_entry_lifecycle_state_name,
    parse_workflow_event_participation, parse_workflow_owned_definition_kind,
    parse_workflow_owned_definition_name, parse_workflow_slug, parse_workflow_transition_endpoint,
    parse_workflow_transition_kind, parse_workflow_transition_source_evidence_text,
    parse_workflow_transition_target_evidence_text, parse_workflow_view_role,
};
use crate::mcp::{serve_http, serve_stdio};
use crate::shell::{ShellError, interpret};

struct Cli {
    command: Command,
}

enum Command {
    AddAutomationDefinition {
        automation: NewAutomationDefinition,
    },
    UpdateAutomationDefinition {
        automation: NewAutomationDefinition,
    },
    RemoveAutomationDefinition {
        slice_slug: SliceSlug,
        automation_name: AutomationName,
    },
    AddBitLevelDataFlow {
        data_flow: NewBitLevelDataFlow,
    },
    UpdateBitLevelDataFlow {
        previous: NewBitLevelDataFlow,
        replacement: NewBitLevelDataFlow,
    },
    RemoveBitLevelDataFlow {
        data_flow: NewBitLevelDataFlow,
    },
    AddBoardConnection {
        connection: NewBoardConnection,
    },
    UpdateBoardConnection {
        previous: NewBoardConnection,
        replacement: NewBoardConnection,
    },
    RemoveBoardConnection {
        connection: NewBoardConnection,
    },
    AddBoardElement {
        element: NewBoardElement,
    },
    UpdateBoardElement {
        element: NewBoardElement,
    },
    RemoveBoardElement {
        slice_slug: SliceSlug,
        element_name: BoardElementName,
    },
    AddCommandDefinition {
        command: NewCommandDefinition,
    },
    UpdateCommandDefinition {
        command: NewCommandDefinition,
    },
    RemoveCommandDefinition {
        slice_slug: SliceSlug,
        command_name: CommandName,
    },
    AddEventDefinition {
        event: NewEventDefinition,
    },
    UpdateEventDefinition {
        event: NewEventDefinition,
    },
    RemoveEventDefinition {
        slice_slug: SliceSlug,
        event_name: EventName,
    },
    AddExternalPayloadDefinition {
        external_payload: NewExternalPayloadDefinition,
    },
    UpdateExternalPayloadDefinition {
        external_payload: NewExternalPayloadDefinition,
    },
    RemoveExternalPayloadDefinition {
        slice_slug: SliceSlug,
        payload_name: EventAttributeSourceName,
        payload_field: EventAttributeSourceField,
    },
    AddOutcomeDefinition {
        outcome: NewOutcomeDefinition,
    },
    UpdateOutcomeDefinition {
        outcome: NewOutcomeDefinition,
    },
    RemoveOutcomeDefinition {
        slice_slug: SliceSlug,
        outcome_label: OutcomeLabelName,
    },
    AddReadModelDefinition {
        read_model: NewReadModelDefinition,
    },
    UpdateReadModelDefinition {
        read_model: NewReadModelDefinition,
    },
    RemoveReadModelDefinition {
        slice_slug: SliceSlug,
        read_model_name: ReadModelName,
    },
    AddViewDefinition {
        view: NewViewDefinition,
    },
    UpdateViewDefinition {
        view: NewViewDefinition,
    },
    RemoveViewDefinition {
        slice_slug: SliceSlug,
        view_name: ViewName,
    },
    UpdateControlDefinition {
        slice_slug: SliceSlug,
        view_name: ViewName,
        control: NewControlDefinition,
    },
    RemoveControlDefinition {
        slice_slug: SliceSlug,
        view_name: ViewName,
        control_name: ControlName,
    },
    AddSlice {
        slice: NewSlice,
    },
    AddSliceScenario {
        scenario: NewSliceScenario,
    },
    UpdateSliceScenario {
        scenario: NewSliceScenario,
    },
    RemoveSliceScenario {
        slice_slug: SliceSlug,
        scenario_name: ScenarioName,
    },
    AddTranslationDefinition {
        translation: NewTranslationDefinition,
    },
    UpdateTranslationDefinition {
        translation: NewTranslationDefinition,
    },
    RemoveTranslationDefinition {
        slice_slug: SliceSlug,
        translation_name: TranslationName,
    },
    AddWorkflow {
        workflow: NewWorkflow,
    },
    AddWorkflowCommandError {
        workflow_slug: WorkflowSlug,
        error: WorkflowCommandErrorRecord,
    },
    UpdateWorkflowCommandError {
        workflow_slug: WorkflowSlug,
        previous: WorkflowCommandErrorRecord,
        replacement: WorkflowCommandErrorRecord,
    },
    RemoveWorkflowCommandError {
        workflow_slug: WorkflowSlug,
        error: WorkflowCommandErrorRecord,
    },
    AddWorkflowOwnedDefinition {
        workflow_slug: WorkflowSlug,
        definition: WorkflowOwnedDefinitionRecord,
    },
    UpdateWorkflowOwnedDefinition {
        workflow_slug: WorkflowSlug,
        previous: WorkflowOwnedDefinitionRecord,
        replacement: WorkflowOwnedDefinitionRecord,
    },
    RemoveWorkflowOwnedDefinition {
        workflow_slug: WorkflowSlug,
        definition: WorkflowOwnedDefinitionRecord,
    },
    AddWorkflowTransitionEvidence {
        workflow_slug: WorkflowSlug,
        evidence: WorkflowTransitionEvidenceRecord,
    },
    AddWorkflowEntryLifecycleState {
        workflow_slug: WorkflowSlug,
        coverage: WorkflowEntryLifecycleStateRecord,
    },
    AddWorkflowOutcome {
        workflow_slug: WorkflowSlug,
        outcome: WorkflowOutcomeRecord,
    },
    UpdateWorkflowOutcome {
        workflow_slug: WorkflowSlug,
        previous: WorkflowOutcomeRecord,
        replacement: WorkflowOutcomeRecord,
    },
    RemoveWorkflowOutcome {
        workflow_slug: WorkflowSlug,
        outcome: WorkflowOutcomeRecord,
    },
    Check,
    ConnectWorkflow {
        connection: WorkflowConnection,
    },
    GherkinList {
        suite: GherkinSuite,
    },
    GherkinRun {
        suite: GherkinSuite,
    },
    GherkinRunAll,
    Help,
    HelpEnums,
    HelpModeling,
    Init {
        name: ProjectName,
    },
    ListSlices,
    ListConflicts,
    ListTransitions,
    ListWorkflows,
    RequireWorkflowEntryLifecycleCoverage {
        workflow_slug: WorkflowSlug,
    },
    McpStdio,
    McpHttp {
        host: String,
        port: u16,
        once: bool,
        auth_token: Option<String>,
    },
    RemoveTransition {
        removal: WorkflowTransitionRemoval,
    },
    UpdateTransition {
        update: WorkflowTransitionUpdate,
    },
    RemoveSlice {
        slug: SliceSlug,
    },
    ReviewGate {
        slug: WorkflowSlug,
    },
    RecordCleanReview {
        slug: WorkflowSlug,
        reviewer: ReviewerId,
        reviewed_at: ReviewTimestamp,
    },
    RemoveWorkflow {
        slug: WorkflowSlug,
    },
    ResolveConflict {
        conflict_id: EventConflictId,
        chosen_event_id: ChosenEventId,
    },
    ShowSlice {
        slug: SliceSlug,
    },
    ShowWorkflow {
        slug: WorkflowSlug,
    },
    UpdateSliceDescription {
        slug: SliceSlug,
        description: ModelDescription,
    },
    UpdateSliceKind {
        slug: SliceSlug,
        kind: SliceKind,
    },
    UpdateSliceName {
        slug: SliceSlug,
        name: ModelName,
    },
    UpdateWorkflowDescription {
        slug: WorkflowSlug,
        description: ModelDescription,
    },
    UpdateWorkflowName {
        slug: WorkflowSlug,
        name: ModelName,
    },
    Verify,
}

fn main() -> ExitCode {
    match parse_cli(&env::args().skip(1).collect::<Vec<_>>()).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), ShellError> {
    match cli.command {
        Command::AddAutomationDefinition { automation } => {
            interpret(&command::add_automation_definition(automation))
        }
        Command::UpdateAutomationDefinition { automation } => {
            interpret(&command::update_automation_definition(automation))
        }
        Command::AddBitLevelDataFlow { data_flow } => {
            interpret(&command::add_bit_level_data_flow(data_flow))
        }
        Command::UpdateBitLevelDataFlow {
            previous,
            replacement,
        } => interpret(&command::update_bit_level_data_flow(previous, replacement)),
        Command::AddBoardConnection { connection } => {
            interpret(&command::add_board_connection(connection))
        }
        Command::UpdateBoardConnection {
            previous,
            replacement,
        } => interpret(&command::update_board_connection(previous, replacement)),
        Command::AddBoardElement { element } => interpret(&command::add_board_element(element)),
        Command::UpdateBoardElement { element } => {
            interpret(&command::update_board_element(element))
        }
        Command::AddCommandDefinition {
            command: definition,
        } => interpret(&command::add_command_definition(definition)),
        Command::UpdateCommandDefinition {
            command: definition,
        } => interpret(&command::update_command_definition(definition)),
        Command::AddEventDefinition { event } => interpret(&command::add_event_definition(event)),
        Command::UpdateEventDefinition { event } => {
            interpret(&command::update_event_definition(event))
        }
        Command::AddExternalPayloadDefinition { external_payload } => {
            interpret(&command::add_external_payload_definition(external_payload))
        }
        Command::UpdateExternalPayloadDefinition { external_payload } => interpret(
            &command::update_external_payload_definition(external_payload),
        ),
        Command::AddOutcomeDefinition { outcome } => {
            interpret(&command::add_outcome_definition(outcome))
        }
        Command::UpdateOutcomeDefinition { outcome } => {
            interpret(&command::update_outcome_definition(outcome))
        }
        Command::AddReadModelDefinition { read_model } => {
            interpret(&command::add_read_model_definition(read_model))
        }
        Command::UpdateReadModelDefinition { read_model } => {
            interpret(&command::update_read_model_definition(read_model))
        }
        Command::AddViewDefinition { view } => interpret(&command::add_view_definition(view)),
        Command::UpdateViewDefinition { view } => interpret(&command::update_view_definition(view)),
        Command::UpdateControlDefinition {
            slice_slug,
            view_name,
            control,
        } => interpret(&command::update_control_definition(
            slice_slug, view_name, control,
        )),
        Command::AddSlice { slice } => interpret(&command::add_slice(slice)),
        Command::AddSliceScenario { scenario } => interpret(&command::add_slice_scenario(scenario)),
        Command::UpdateSliceScenario { scenario } => {
            interpret(&command::update_slice_scenario(scenario))
        }
        Command::AddTranslationDefinition { translation } => {
            interpret(&command::add_translation_definition(translation))
        }
        Command::UpdateTranslationDefinition { translation } => {
            interpret(&command::update_translation_definition(translation))
        }
        other => run_workflow_commands(other),
    }
}

fn run_workflow_commands(command: Command) -> Result<(), ShellError> {
    match command {
        Command::AddWorkflow { workflow } => interpret(&command::add_workflow(workflow)),
        Command::AddWorkflowCommandError {
            workflow_slug,
            error,
        } => interpret(&command::add_workflow_command_error(workflow_slug, error)),
        Command::UpdateWorkflowCommandError {
            workflow_slug,
            previous,
            replacement,
        } => interpret(&command::update_workflow_command_error(
            workflow_slug,
            previous,
            replacement,
        )),
        Command::RemoveWorkflowCommandError {
            workflow_slug,
            error,
        } => interpret(&command::remove_workflow_command_error(
            workflow_slug,
            error,
        )),
        Command::AddWorkflowOwnedDefinition {
            workflow_slug,
            definition,
        } => interpret(&command::add_workflow_owned_definition(
            workflow_slug,
            definition,
        )),
        Command::UpdateWorkflowOwnedDefinition {
            workflow_slug,
            previous,
            replacement,
        } => interpret(&command::update_workflow_owned_definition(
            workflow_slug,
            previous,
            replacement,
        )),
        Command::RemoveWorkflowOwnedDefinition {
            workflow_slug,
            definition,
        } => interpret(&command::remove_workflow_owned_definition(
            workflow_slug,
            definition,
        )),
        Command::AddWorkflowTransitionEvidence {
            workflow_slug,
            evidence,
        } => interpret(&command::add_workflow_transition_evidence(
            workflow_slug,
            evidence,
        )),
        Command::AddWorkflowEntryLifecycleState {
            workflow_slug,
            coverage,
        } => interpret(&command::add_workflow_entry_lifecycle_state(
            workflow_slug,
            coverage,
        )),
        Command::AddWorkflowOutcome {
            workflow_slug,
            outcome,
        } => interpret(&command::add_workflow_outcome(workflow_slug, outcome)),
        Command::UpdateWorkflowOutcome {
            workflow_slug,
            previous,
            replacement,
        } => interpret(&command::update_workflow_outcome(
            workflow_slug,
            previous,
            replacement,
        )),
        Command::RemoveWorkflowOutcome {
            workflow_slug,
            outcome,
        } => interpret(&command::remove_workflow_outcome(workflow_slug, outcome)),
        other => run_query_commands(other),
    }
}

fn run_query_commands(command: Command) -> Result<(), ShellError> {
    match command {
        Command::Check => interpret(&command::check_project()),
        Command::ConnectWorkflow { connection } => {
            interpret(&command::connect_workflow(connection))
        }
        Command::GherkinList { suite } => interpret(&command::gherkin_list(&suite)),
        Command::GherkinRunAll => interpret(&command::gherkin_run_all()),
        Command::GherkinRun { suite } => interpret(&command::gherkin_run(&suite)),
        Command::Help => print_help(),
        Command::HelpEnums => {
            print_enum_help();
            Ok(())
        }
        Command::HelpModeling => {
            print_modeling_help();
            Ok(())
        }
        Command::Init { name } => interpret(&command::init(&name)),
        Command::ListConflicts => interpret(&command::list_conflicts()),
        Command::ListSlices => interpret(&command::list_slices()),
        Command::ListTransitions => interpret(&command::list_transitions()),
        Command::ListWorkflows => interpret(&command::list_workflows()),
        Command::RequireWorkflowEntryLifecycleCoverage { workflow_slug } => interpret(
            &command::require_workflow_entry_lifecycle_coverage(workflow_slug),
        ),
        Command::McpHttp {
            host,
            port,
            once,
            auth_token,
        } => serve_http(&host, port, once, auth_token.as_deref()),
        Command::McpStdio => serve_stdio(),
        other => run_mutation_commands(other),
    }
}

fn run_mutation_commands(command: Command) -> Result<(), ShellError> {
    match command {
        Command::ReviewGate { slug } => interpret(&command::review_gate_for_workflow(slug)),
        Command::RecordCleanReview {
            slug,
            reviewer,
            reviewed_at,
        } => interpret(&command::record_clean_review(slug, reviewer, reviewed_at)),
        other => run_definition_removal_commands(other),
    }
}

fn run_definition_removal_commands(command: Command) -> Result<(), ShellError> {
    match command {
        Command::RemoveCommandDefinition {
            slice_slug,
            command_name,
        } => interpret(&command::remove_command_definition(
            slice_slug,
            command_name,
        )),
        Command::RemoveBoardElement {
            slice_slug,
            element_name,
        } => interpret(&command::remove_board_element(slice_slug, element_name)),
        Command::RemoveBoardConnection { connection } => {
            interpret(&command::remove_board_connection(connection))
        }
        Command::RemoveBitLevelDataFlow { data_flow } => {
            interpret(&command::remove_bit_level_data_flow(data_flow))
        }
        Command::RemoveEventDefinition {
            slice_slug,
            event_name,
        } => interpret(&command::remove_event_definition(slice_slug, event_name)),
        Command::RemoveAutomationDefinition {
            slice_slug,
            automation_name,
        } => interpret(&command::remove_automation_definition(
            slice_slug,
            automation_name,
        )),
        Command::RemoveTranslationDefinition {
            slice_slug,
            translation_name,
        } => interpret(&command::remove_translation_definition(
            slice_slug,
            translation_name,
        )),
        Command::RemoveExternalPayloadDefinition {
            slice_slug,
            payload_name,
            payload_field,
        } => interpret(&command::remove_external_payload_definition(
            slice_slug,
            payload_name,
            payload_field,
        )),
        Command::RemoveOutcomeDefinition {
            slice_slug,
            outcome_label,
        } => interpret(&command::remove_outcome_definition(
            slice_slug,
            outcome_label,
        )),
        Command::RemoveReadModelDefinition {
            slice_slug,
            read_model_name,
        } => interpret(&command::remove_read_model_definition(
            slice_slug,
            read_model_name,
        )),
        Command::RemoveViewDefinition {
            slice_slug,
            view_name,
        } => interpret(&command::remove_view_definition(slice_slug, view_name)),
        Command::RemoveControlDefinition {
            slice_slug,
            view_name,
            control_name,
        } => interpret(&command::remove_control_definition(
            slice_slug,
            view_name,
            control_name,
        )),
        Command::RemoveSlice { slug } => interpret(&command::remove_slice(slug)),
        Command::RemoveSliceScenario {
            slice_slug,
            scenario_name,
        } => interpret(&command::remove_slice_scenario(slice_slug, scenario_name)),
        other => run_remaining_mutation_commands(other),
    }
}

fn run_remaining_mutation_commands(command: Command) -> Result<(), ShellError> {
    match command {
        Command::RemoveTransition { removal } => interpret(&command::remove_transition(removal)),
        Command::UpdateTransition { update } => interpret(&command::update_transition(update)),
        Command::RemoveWorkflow { slug } => interpret(&command::remove_workflow(slug)),
        Command::ResolveConflict {
            conflict_id,
            chosen_event_id,
        } => interpret(&command::resolve_conflict(conflict_id, chosen_event_id)),
        Command::ShowSlice { slug } => interpret(&command::show_slice(slug)),
        Command::ShowWorkflow { slug } => interpret(&command::show_workflow(slug)),
        Command::UpdateSliceDescription { slug, description } => {
            interpret(&command::update_slice_description(slug, description))
        }
        Command::UpdateSliceKind { slug, kind } => {
            interpret(&command::update_slice_kind(slug, kind))
        }
        Command::UpdateSliceName { slug, name } => {
            interpret(&command::update_slice_name(slug, name))
        }
        Command::UpdateWorkflowDescription { slug, description } => {
            interpret(&command::update_workflow_description(slug, description))
        }
        Command::UpdateWorkflowName { slug, name } => {
            interpret(&command::update_workflow_name(slug, name))
        }
        Command::Verify => interpret(&command::verify()),
        _ => Err(ShellError::message("unsupported command")),
    }
}

fn parse_artifact_digest(label: &str, value: String) -> Result<ArtifactDigest, ShellError> {
    ArtifactDigest::try_new(value)
        .map_err(|error| ShellError::message(format!("invalid {label}: {error}")))
}

fn parse_event_conflict_id(value: &str) -> Result<EventConflictId, ShellError> {
    parse_artifact_digest("event conflict id", value.to_owned()).map(EventConflictId::new)
}

fn parse_chosen_event_id(value: &str) -> Result<ChosenEventId, ShellError> {
    parse_artifact_digest("chosen event id", value.to_owned()).map(ChosenEventId::new)
}

fn parse_slice_and_view_name(slice: &str, name: &str) -> Result<(SliceSlug, ViewName), ShellError> {
    let slice_slug =
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
    let view_name =
        parse_view_name(name).map_err(|error| ShellError::message(error.to_string()))?;
    Ok((slice_slug, view_name))
}

fn parse_board_connection_cli(
    slice: &str,
    source: &str,
    source_kind: &str,
    target: &str,
    target_kind: &str,
) -> Result<NewBoardConnection, ShellError> {
    Ok(NewBoardConnection::new(
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?,
        parse_board_connection_endpoint(source)
            .map_err(|error| ShellError::message(error.to_string()))?,
        parse_board_connection_endpoint_kind(source_kind)
            .map_err(|error| ShellError::message(error.to_string()))?,
        parse_board_connection_endpoint(target)
            .map_err(|error| ShellError::message(error.to_string()))?,
        parse_board_connection_endpoint_kind(target_kind)
            .map_err(|error| ShellError::message(error.to_string()))?,
    ))
}

fn build_view_read_model_field(
    read_model: &str,
    field: &str,
    source_field: &str,
    sketch_token: &str,
    field_provenance: &str,
    bit_encoding: &str,
) -> Result<NewViewField, ShellError> {
    let read_model_name = parse_read_model_name(read_model)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let field_name =
        parse_view_field_name(field).map_err(|error| ShellError::message(error.to_string()))?;
    let source_field_name = parse_view_field_name(source_field)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let sketch_token =
        parse_sketch_token(sketch_token).map_err(|error| ShellError::message(error.to_string()))?;
    let provenance_description = parse_provenance_description(field_provenance)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let bit_encoding = parse_bit_encoding_semantics(bit_encoding)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_kind = parse_view_field_source_kind("read_model")
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(NewViewField::new(
        field_name,
        source_kind,
        read_model_name,
        source_field_name,
        sketch_token,
        provenance_description,
        bit_encoding,
    ))
}

fn parse_control_name_and_command(
    control: &str,
    control_command: &str,
) -> Result<(ControlName, CommandName), ShellError> {
    let control_name =
        parse_control_name(control).map_err(|error| ShellError::message(error.to_string()))?;
    let control_command = parse_command_name(control_command)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok((control_name, control_command))
}

fn build_control_input_provision(
    control_input: &str,
    control_input_source: &str,
    control_input_description: &str,
    control_input_sketch_token: &str,
    control_input_visible: &str,
    control_input_decision: &str,
) -> Result<NewControlInputProvision, ShellError> {
    let control_input =
        parse_datum_name(control_input).map_err(|error| ShellError::message(error.to_string()))?;
    let control_input_source = parse_command_input_source_kind(control_input_source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let control_input_description =
        parse_command_input_source_description(control_input_description)
            .map_err(|error| ShellError::message(error.to_string()))?;
    let control_input_sketch_token = parse_sketch_token(control_input_sketch_token)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let control_input_visible = parse_bool_flag(control_input_visible)?;
    let control_input_decision = parse_bool_flag(control_input_decision)?;
    Ok(NewControlInputProvision::new(
        control_input,
        control_input_source,
        control_input_description,
        control_input_sketch_token,
        control_input_visible,
        control_input_decision,
    ))
}

fn parse_control_errors_recovery_sketch(
    handled_errors: &str,
    recovery_behavior: &str,
    control_sketch_token: &str,
) -> Result<(CommandErrorNames, ControlRecoveryBehavior, SketchToken), ShellError> {
    let handled_errors = parse_command_error_names(handled_errors)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let recovery_behavior = parse_control_recovery_behavior(recovery_behavior)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let control_sketch_token = parse_sketch_token(control_sketch_token)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok((
        CommandErrorNames::from_names(handled_errors),
        recovery_behavior,
        control_sketch_token,
    ))
}

fn build_navigation_target(
    navigation_type: &str,
    navigation_target: &str,
) -> Result<NewNavigationTarget, ShellError> {
    let navigation_type = parse_navigation_target_type(navigation_type)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let navigation_target = parse_navigation_target_name(navigation_target)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(NewNavigationTarget::new(navigation_type, navigation_target))
}

fn build_navigation_external_system(
    navigation_type: &str,
    navigation_target: &str,
    external_system: &str,
    handoff_contract: &str,
) -> Result<NewNavigationTarget, ShellError> {
    let navigation = build_navigation_target(navigation_type, navigation_target)?;
    let external_system = parse_navigation_target_name(external_system)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let handoff_contract = parse_payload_contract_name(handoff_contract)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(navigation.with_external_system(external_system, handoff_contract))
}

fn build_navigation_external_workflow(
    navigation_type: &str,
    navigation_target: &str,
    external_workflow: &str,
) -> Result<NewNavigationTarget, ShellError> {
    let navigation = build_navigation_target(navigation_type, navigation_target)?;
    let external_workflow = parse_navigation_target_name(external_workflow)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(navigation.with_external_workflow(external_workflow))
}

#[derive(Clone, Copy)]
struct ControlDefinitionInput<'a> {
    name: &'a str,
    command: &'a str,
    input: &'a str,
    input_source: &'a str,
    input_description: &'a str,
    input_sketch_token: &'a str,
    input_visible: &'a str,
    input_decision: &'a str,
    handled_errors: &'a str,
    recovery_behavior: &'a str,
    sketch_token: &'a str,
    navigation_type: &'a str,
    navigation_target: &'a str,
}

fn build_control_definition(
    input: ControlDefinitionInput<'_>,
) -> Result<NewControlDefinition, ShellError> {
    let (control_name, control_command) =
        parse_control_name_and_command(input.name, input.command)?;
    let provision = build_control_input_provision(
        input.input,
        input.input_source,
        input.input_description,
        input.input_sketch_token,
        input.input_visible,
        input.input_decision,
    )?;
    let (handled_errors, recovery_behavior, control_sketch_token) =
        parse_control_errors_recovery_sketch(
            input.handled_errors,
            input.recovery_behavior,
            input.sketch_token,
        )?;
    let navigation = build_navigation_target(input.navigation_type, input.navigation_target)?;
    Ok(NewControlDefinition::new(
        control_name,
        control_command,
        provision,
        handled_errors,
        recovery_behavior,
        control_sketch_token,
        navigation,
    ))
}

fn build_add_view_cli(
    slice_slug: SliceSlug,
    view_name: ViewName,
    view_field: NewViewField,
    controls: [NewControlDefinition; 1],
    local_states: Option<&str>,
    filters: Option<&str>,
) -> Result<Cli, ShellError> {
    let mut view = NewViewDefinition::new(slice_slug, view_name, view_field);
    if let Some(local_states) = local_states {
        let local_states = parse_navigation_target_names(local_states)
            .map_err(|error| ShellError::message(error.to_string()))?;
        view = view.with_local_states(ViewLocalStates::from_targets(local_states));
    }
    if let Some(filters) = filters {
        let filters = parse_navigation_target_names(filters)
            .map_err(|error| ShellError::message(error.to_string()))?;
        view = view.with_filters(ViewFilters::from_targets(filters));
    }
    let view = view.with_controls(ViewControls::from_controls(controls));
    Ok(Cli {
        command: Command::AddViewDefinition { view },
    })
}

fn view_definition_cli(command: &str, view: NewViewDefinition) -> Cli {
    let command = if command == "update" {
        Command::UpdateViewDefinition { view }
    } else {
        Command::AddViewDefinition { view }
    };
    Cli { command }
}

fn remove_view_definition_cli(slice: &str, name: &str) -> Result<Cli, ShellError> {
    let (slice_slug, view_name) = parse_slice_and_view_name(slice, name)?;
    Ok(Cli {
        command: Command::RemoveViewDefinition {
            slice_slug,
            view_name,
        },
    })
}

fn remove_command_definition_cli(slice: &str, name: &str) -> Result<Cli, ShellError> {
    let slice_slug =
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
    let command_name =
        parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
    Ok(Cli {
        command: Command::RemoveCommandDefinition {
            slice_slug,
            command_name,
        },
    })
}

fn remove_event_definition_cli(slice: &str, name: &str) -> Result<Cli, ShellError> {
    let slice_slug =
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
    let event_name =
        parse_event_name(name).map_err(|error| ShellError::message(error.to_string()))?;
    Ok(Cli {
        command: Command::RemoveEventDefinition {
            slice_slug,
            event_name,
        },
    })
}

fn read_model_definition_cli(command: &str, read_model: NewReadModelDefinition) -> Cli {
    let command = if command == "update" {
        Command::UpdateReadModelDefinition { read_model }
    } else {
        Command::AddReadModelDefinition { read_model }
    };
    Cli { command }
}

fn remove_read_model_definition_cli(slice: &str, name: &str) -> Result<Cli, ShellError> {
    let slice_slug =
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
    let read_model_name =
        parse_read_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
    Ok(Cli {
        command: Command::RemoveReadModelDefinition {
            slice_slug,
            read_model_name,
        },
    })
}

fn remove_control_definition_cli(slice: &str, view: &str, name: &str) -> Result<Cli, ShellError> {
    let (slice_slug, view_name) = parse_slice_and_view_name(slice, view)?;
    let control_name =
        parse_control_name(name).map_err(|error| ShellError::message(error.to_string()))?;
    Ok(Cli {
        command: Command::RemoveControlDefinition {
            slice_slug,
            view_name,
            control_name,
        },
    })
}

fn remove_external_payload_definition_cli(
    slice: &str,
    name: &str,
    field: &str,
) -> Result<Cli, ShellError> {
    let slice_slug =
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
    let payload_name = parse_event_attribute_source_name(name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let payload_field = parse_event_attribute_source_field(field)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(Cli {
        command: Command::RemoveExternalPayloadDefinition {
            slice_slug,
            payload_name,
            payload_field,
        },
    })
}

fn remove_translation_definition_cli(slice: &str, name: &str) -> Result<Cli, ShellError> {
    let slice_slug =
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
    let translation_name =
        parse_translation_name(name).map_err(|error| ShellError::message(error.to_string()))?;
    Ok(Cli {
        command: Command::RemoveTranslationDefinition {
            slice_slug,
            translation_name,
        },
    })
}

fn update_control_definition_cli(
    slice: &str,
    view: &str,
    input: ControlDefinitionInput<'_>,
) -> Result<Cli, ShellError> {
    let (slice_slug, view_name) = parse_slice_and_view_name(slice, view)?;
    let control = build_control_definition(input)?;
    Ok(Cli {
        command: Command::UpdateControlDefinition {
            slice_slug,
            view_name,
            control,
        },
    })
}

fn parse_cli(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [] => Ok(Cli {
            command: Command::Help,
        }),
        [flag] if flag == "--help" || flag == "-h" => Ok(Cli {
            command: Command::Help,
        }),
        [command, subject] if command == "help" && subject == "enums" => Ok(Cli {
            command: Command::HelpEnums,
        }),
        [command, subject] if command == "help" && subject == "modeling" => Ok(Cli {
            command: Command::HelpModeling,
        }),
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "scenario"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let scenario_name = parse_scenario_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RemoveSliceScenario {
                    slice_slug,
                    scenario_name,
                },
            })
        }
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "command"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            remove_command_definition_cli(slice, name)
        }
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "automation"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let automation_name = parse_automation_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RemoveAutomationDefinition {
                    slice_slug,
                    automation_name,
                },
            })
        }
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "translation"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            remove_translation_definition_cli(slice, name)
        }
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            field_flag,
            field,
        ] if command == "remove"
            && subject == "external-payload"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && field_flag == "--field" =>
        {
            remove_external_payload_definition_cli(slice, name, field)
        }
        [command, subject, slice_flag, slice, label_flag, label]
            if command == "remove"
                && subject == "outcome"
                && slice_flag == "--slice"
                && label_flag == "--label" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let outcome_label = parse_outcome_label_name(label)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RemoveOutcomeDefinition {
                    slice_slug,
                    outcome_label,
                },
            })
        }
        _ => parse_cli_control_or_data_flow(arguments),
    }
}

fn parse_cli_control_or_data_flow(arguments: &[String]) -> Result<Cli, ShellError> {
    if let Some(cli) = parse_cli_control(arguments)? {
        return Ok(cli);
    }
    parse_cli_data_flow(arguments)
}

fn parse_cli_control(arguments: &[String]) -> Result<Option<Cli>, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            view_flag,
            view,
            name_flag,
            name,
        ] if command == "remove"
            && subject == "control"
            && slice_flag == "--slice"
            && view_flag == "--view"
            && name_flag == "--name" =>
        {
            remove_control_definition_cli(slice, view, name).map(Some)
        }
        [
            command,
            subject,
            slice_flag,
            slice,
            view_flag,
            view,
            name_flag,
            name,
            control_command_flag,
            control_command,
            control_input_flag,
            control_input,
            control_input_source_flag,
            control_input_source,
            control_input_description_flag,
            control_input_description,
            control_input_sketch_token_flag,
            control_input_sketch_token,
            control_input_visible_flag,
            control_input_visible,
            control_input_decision_flag,
            control_input_decision,
            handled_errors_flag,
            handled_errors,
            recovery_behavior_flag,
            recovery_behavior,
            control_sketch_token_flag,
            control_sketch_token,
            navigation_type_flag,
            navigation_type,
            navigation_target_flag,
            navigation_target,
        ] if command == "update"
            && subject == "control"
            && slice_flag == "--slice"
            && view_flag == "--view"
            && name_flag == "--name"
            && control_command_flag == "--command"
            && control_input_flag == "--input"
            && control_input_source_flag == "--input-source"
            && control_input_description_flag == "--input-description"
            && control_input_sketch_token_flag == "--input-sketch-token"
            && control_input_visible_flag == "--input-visible"
            && control_input_decision_flag == "--input-decision"
            && handled_errors_flag == "--handled-errors"
            && recovery_behavior_flag == "--recovery-behavior"
            && control_sketch_token_flag == "--sketch-token"
            && navigation_type_flag == "--navigation-type"
            && navigation_target_flag == "--navigation-target" =>
        {
            update_control_definition_cli(
                slice,
                view,
                ControlDefinitionInput {
                    name,
                    command: control_command,
                    input: control_input,
                    input_source: control_input_source,
                    input_description: control_input_description,
                    input_sketch_token: control_input_sketch_token,
                    input_visible: control_input_visible,
                    input_decision: control_input_decision,
                    handled_errors,
                    recovery_behavior,
                    sketch_token: control_sketch_token,
                    navigation_type,
                    navigation_target,
                },
            )
            .map(Some)
        }
        _ => Ok(None),
    }
}

fn parse_cli_data_flow(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            datum_flag,
            datum,
            source_flag,
            source,
            source_kind_flag,
            source_kind,
            transformation_flag,
            transformation,
            target_flag,
            target,
            bit_encoding_flag,
            bit_encoding,
        ] if (command == "add" || command == "remove")
            && subject == "data-flow"
            && slice_flag == "--slice"
            && datum_flag == "--datum"
            && source_flag == "--source"
            && source_kind_flag == "--source-kind"
            && transformation_flag == "--transformation"
            && target_flag == "--target"
            && bit_encoding_flag == "--bit-encoding" =>
        {
            let data_flow = parse_data_flow_cli(
                slice,
                datum,
                source,
                source_kind,
                transformation,
                target,
                bit_encoding,
            )?;
            let command = if command == "add" {
                Command::AddBitLevelDataFlow { data_flow }
            } else {
                Command::RemoveBitLevelDataFlow { data_flow }
            };
            Ok(Cli { command })
        }
        _ => parse_cli_update_data_flow_or_remove_event(arguments),
    }
}

fn parse_cli_update_data_flow_or_remove_event(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            datum_flag,
            datum,
            source_flag,
            source,
            source_kind_flag,
            source_kind,
            transformation_flag,
            transformation,
            target_flag,
            target,
            bit_encoding_flag,
            bit_encoding,
            new_datum_flag,
            new_datum,
            new_source_flag,
            new_source,
            new_source_kind_flag,
            new_source_kind,
            new_transformation_flag,
            new_transformation,
            new_target_flag,
            new_target,
            new_bit_encoding_flag,
            new_bit_encoding,
        ] if command == "update"
            && subject == "data-flow"
            && slice_flag == "--slice"
            && datum_flag == "--datum"
            && source_flag == "--source"
            && source_kind_flag == "--source-kind"
            && transformation_flag == "--transformation"
            && target_flag == "--target"
            && bit_encoding_flag == "--bit-encoding"
            && new_datum_flag == "--new-datum"
            && new_source_flag == "--new-source"
            && new_source_kind_flag == "--new-source-kind"
            && new_transformation_flag == "--new-transformation"
            && new_target_flag == "--new-target"
            && new_bit_encoding_flag == "--new-bit-encoding" =>
        {
            Ok(Cli {
                command: Command::UpdateBitLevelDataFlow {
                    previous: parse_data_flow_cli(
                        slice,
                        datum,
                        source,
                        source_kind,
                        transformation,
                        target,
                        bit_encoding,
                    )?,
                    replacement: parse_data_flow_cli(
                        slice,
                        new_datum,
                        new_source,
                        new_source_kind,
                        new_transformation,
                        new_target,
                        new_bit_encoding,
                    )?,
                },
            })
        }
        _ => parse_cli_remove_event_or_2(arguments),
    }
}

fn parse_data_flow_cli(
    slice: &str,
    datum: &str,
    source: &str,
    source_kind: &str,
    transformation: &str,
    target: &str,
    bit_encoding: &str,
) -> Result<NewBitLevelDataFlow, ShellError> {
    let slice_slug =
        parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
    let datum = parse_datum_name(datum).map_err(|error| ShellError::message(error.to_string()))?;
    let source =
        parse_data_flow_source(source).map_err(|error| ShellError::message(error.to_string()))?;
    let source_kind = parse_data_flow_source_kind(source_kind)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let transformation = parse_transformation_semantics(transformation)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let target =
        parse_data_flow_target(target).map_err(|error| ShellError::message(error.to_string()))?;
    let bit_encoding = parse_bit_encoding_semantics(bit_encoding)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(NewBitLevelDataFlow::new(
        slice_slug,
        datum,
        source_kind,
        source,
        transformation,
        target,
        bit_encoding,
    ))
}

fn parse_cli_remove_event_or_2(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "event"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            remove_event_definition_cli(slice, name)
        }
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "read-model"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            remove_read_model_definition_cli(slice, name)
        }
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "view"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            remove_view_definition_cli(slice, name)
        }
        _ => parse_cli_2(arguments),
    }
}

fn parse_cli_2(arguments: &[String]) -> Result<Cli, ShellError> {
    if let Some(cli) = parse_workflow_owned_definition_mutation_cli(arguments)? {
        return Ok(cli);
    }

    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            definition_kind_flag,
            definition_kind,
            definition_name_flag,
            definition_name,
            definition_stream_flag,
            definition_stream,
            source_provenance_flag,
            source_provenance,
            event_participation_flag,
            event_participation,
        ] if command == "add"
            && subject == "workflow-owned-definition"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && definition_kind_flag == "--definition-kind"
            && definition_name_flag == "--definition-name"
            && definition_stream_flag == "--definition-stream"
            && source_provenance_flag == "--source-provenance"
            && event_participation_flag == "--event-participation" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_kind = parse_workflow_owned_definition_kind(definition_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_name = parse_workflow_owned_definition_name(definition_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_stream = parse_stream_name(definition_stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_provenance = parse_model_description(source_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let event_participation = parse_workflow_event_participation(event_participation)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowOwnedDefinition {
                    workflow_slug,
                    definition:
                        WorkflowOwnedDefinitionRecord::new_with_event_identity_and_participation(
                            source_slice,
                            definition_kind,
                            definition_name,
                            definition_stream,
                            source_provenance,
                            event_participation,
                        ),
                },
            })
        }
        _ => parse_cli_3(arguments),
    }
}

fn parse_workflow_owned_definition_mutation_cli(
    arguments: &[String],
) -> Result<Option<Cli>, ShellError> {
    let [command, subject, ..] = arguments else {
        return Ok(None);
    };
    if subject != "workflow-owned-definition" {
        return Ok(None);
    }
    match command.as_str() {
        "update" => {
            let workflow_slug = parse_required_workflow_slug_flag(arguments)?;
            Ok(Some(Cli {
                command: Command::UpdateWorkflowOwnedDefinition {
                    workflow_slug,
                    previous: parse_workflow_owned_definition_flags(arguments, "")?,
                    replacement: parse_workflow_owned_definition_flags(arguments, "new-")?,
                },
            }))
        }
        "remove" => {
            let workflow_slug = parse_required_workflow_slug_flag(arguments)?;
            Ok(Some(Cli {
                command: Command::RemoveWorkflowOwnedDefinition {
                    workflow_slug,
                    definition: parse_workflow_owned_definition_flags(arguments, "")?,
                },
            }))
        }
        _ => Ok(None),
    }
}

fn parse_required_workflow_slug_flag(arguments: &[String]) -> Result<WorkflowSlug, ShellError> {
    required_cli_flag(arguments, "--workflow").and_then(|workflow| {
        parse_workflow_slug(workflow).map_err(|error| ShellError::message(error.to_string()))
    })
}

fn parse_workflow_owned_definition_flags(
    arguments: &[String],
    prefix: &str,
) -> Result<WorkflowOwnedDefinitionRecord, ShellError> {
    let source_slice = required_cli_flag(arguments, &format!("--{prefix}source-slice")).and_then(
        |source_slice| {
            WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        },
    )?;
    let definition_kind = required_cli_flag(arguments, &format!("--{prefix}definition-kind"))
        .and_then(|definition_kind| {
            parse_workflow_owned_definition_kind(definition_kind)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let definition_name = required_cli_flag(arguments, &format!("--{prefix}definition-name"))
        .and_then(|definition_name| {
            parse_workflow_owned_definition_name(definition_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let definition_stream = optional_cli_flag(arguments, &format!("--{prefix}definition-stream"))
        .map(parse_stream_name)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_provenance = optional_cli_flag(arguments, &format!("--{prefix}source-provenance"))
        .map(parse_model_description)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let event_participation =
        optional_cli_flag(arguments, &format!("--{prefix}event-participation"))
            .map(parse_workflow_event_participation)
            .transpose()
            .map_err(|error| ShellError::message(error.to_string()))?;
    let view_role = optional_cli_flag(arguments, &format!("--{prefix}view-role"))
        .map(parse_workflow_view_role)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;

    build_workflow_owned_definition_cli(
        source_slice,
        definition_kind,
        definition_name,
        definition_stream,
        source_provenance,
        event_participation,
        view_role,
    )
}

fn required_cli_flag<'args>(
    arguments: &'args [String],
    flag: &str,
) -> Result<&'args str, ShellError> {
    optional_cli_flag(arguments, flag)
        .ok_or_else(|| ShellError::message(format!("{flag} is required")))
}

fn optional_cli_flag<'args>(arguments: &'args [String], flag: &str) -> Option<&'args str> {
    arguments
        .windows(2)
        .find(|window| window.first().is_some_and(|candidate| candidate == flag))
        .and_then(|window| window.get(1))
        .map(String::as_str)
}

fn build_workflow_owned_definition_cli(
    source_slice: WorkflowTransitionEndpoint,
    definition_kind: WorkflowOwnedDefinitionKind,
    definition_name: WorkflowOwnedDefinitionName,
    definition_stream: Option<StreamName>,
    source_provenance: Option<ModelDescription>,
    event_participation: Option<WorkflowEventParticipation>,
    view_role: Option<WorkflowViewRole>,
) -> Result<WorkflowOwnedDefinitionRecord, ShellError> {
    match (
        definition_stream,
        source_provenance,
        event_participation,
        view_role,
    ) {
        (None, None, None, None) => Ok(WorkflowOwnedDefinitionRecord::new(
            source_slice,
            definition_kind,
            definition_name,
        )),
        (None, None, None, Some(view_role)) => WorkflowOwnedDefinitionRecord::new_with_view_role(
            source_slice,
            definition_kind,
            definition_name,
            view_role,
        )
        .ok_or_else(|| ShellError::message("view_role requires definition_kind view")),
        (Some(definition_stream), Some(source_provenance), None, None) => {
            Ok(WorkflowOwnedDefinitionRecord::new_with_event_identity(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
            ))
        }
        (Some(definition_stream), Some(source_provenance), Some(event_participation), None) => Ok(
            WorkflowOwnedDefinitionRecord::new_with_event_identity_and_participation(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
                event_participation,
            ),
        ),
        (_, _, Some(_), _) => Err(ShellError::message(
            "event_participation requires definition_stream and source_provenance",
        )),
        (_, _, _, Some(_)) => Err(ShellError::message(
            "view_role cannot be combined with event identity fields",
        )),
        _ => Err(ShellError::message(
            "definition_stream and source_provenance must be provided together",
        )),
    }
}

fn parse_cli_3(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            definition_kind_flag,
            definition_kind,
            definition_name_flag,
            definition_name,
            view_role_flag,
            view_role,
        ] if command == "add"
            && subject == "workflow-owned-definition"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && definition_kind_flag == "--definition-kind"
            && definition_name_flag == "--definition-name"
            && view_role_flag == "--view-role" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_kind = parse_workflow_owned_definition_kind(definition_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_name = parse_workflow_owned_definition_name(definition_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let view_role = parse_workflow_view_role(view_role)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowOwnedDefinition {
                    workflow_slug,
                    definition: WorkflowOwnedDefinitionRecord::new_with_view_role(
                        source_slice,
                        definition_kind,
                        definition_name,
                        view_role,
                    )
                    .ok_or_else(|| {
                        ShellError::message("view_role requires definition_kind view")
                    })?,
                },
            })
        }
        _ => parse_cli_4(arguments),
    }
}

fn parse_cli_4(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            definition_kind_flag,
            definition_kind,
            definition_name_flag,
            definition_name,
            definition_stream_flag,
            definition_stream,
            source_provenance_flag,
            source_provenance,
        ] if command == "add"
            && subject == "workflow-owned-definition"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && definition_kind_flag == "--definition-kind"
            && definition_name_flag == "--definition-name"
            && definition_stream_flag == "--definition-stream"
            && source_provenance_flag == "--source-provenance" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_kind = parse_workflow_owned_definition_kind(definition_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_name = parse_workflow_owned_definition_name(definition_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_stream = parse_stream_name(definition_stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_provenance = parse_model_description(source_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowOwnedDefinition {
                    workflow_slug,
                    definition: WorkflowOwnedDefinitionRecord::new_with_event_identity(
                        source_slice,
                        definition_kind,
                        definition_name,
                        definition_stream,
                        source_provenance,
                    ),
                },
            })
        }
        _ => parse_cli_5(arguments),
    }
}

fn parse_cli_5(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            source_name_flag,
            source_name,
            source_field_flag,
            source_field,
        ] if (command == "add" || command == "update")
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && source_name_flag == "--source-name"
            && source_field_flag == "--source-field" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let source_name = parse_event_attribute_source_name(source_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_field = parse_event_attribute_source_field(source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            match input_source {
                                CommandInputSourceKind::Generated => {
                                    CommandInputSource::generated(source_name, source_field)
                                }
                                other => {
                                    return Err(command_input_source_kind_mismatch(
                                        other,
                                        CommandInputSourceKind::Generated,
                                    ));
                                }
                            },
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    ),
                },
            })
        }
        _ => parse_cli_6(arguments),
    }
}

fn parse_cli_6(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            source_argument_flag,
            source_argument,
            source_field_flag,
            source_field,
        ] if command == "add"
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && source_argument_flag == "--source-argument"
            && source_field_flag == "--source-field" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let source_argument = parse_event_attribute_source_name(source_argument)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_field = parse_event_attribute_source_field(source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            match input_source {
                                CommandInputSourceKind::InvocationArgument => {
                                    CommandInputSource::invocation_argument(
                                        source_argument,
                                        source_field,
                                    )
                                }
                                other => {
                                    return Err(command_input_source_kind_mismatch(
                                        other,
                                        CommandInputSourceKind::InvocationArgument,
                                    ));
                                }
                            },
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    ),
                },
            })
        }
        _ => parse_cli_7(arguments),
    }
}

fn parse_cli_7(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            source_session_flag,
            source_session,
            source_field_flag,
            source_field,
        ] if command == "add"
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && source_session_flag == "--source-session"
            && source_field_flag == "--source-field" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let source_session = parse_event_attribute_source_name(source_session)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_field = parse_event_attribute_source_field(source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            match input_source {
                                CommandInputSourceKind::Session => {
                                    CommandInputSource::session(source_session, source_field)
                                }
                                other => {
                                    return Err(command_input_source_kind_mismatch(
                                        other,
                                        CommandInputSourceKind::Session,
                                    ));
                                }
                            },
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    ),
                },
            })
        }
        _ => parse_cli_8(arguments),
    }
}

fn parse_cli_8(arguments: &[String]) -> Result<Cli, ShellError> {
    let args: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let [
        "add",
        "view",
        "--slice",
        slice,
        "--name",
        name,
        "--read-model",
        read_model,
        "--field",
        field,
        "--source-field",
        source_field,
        "--sketch-token",
        sketch_token,
        "--field-provenance",
        field_provenance,
        "--bit-encoding",
        bit_encoding,
        "--control",
        control,
        "--control-command",
        control_command,
        "--control-input",
        control_input,
        "--control-input-source",
        control_input_source,
        "--control-input-description",
        control_input_description,
        "--control-input-sketch-token",
        control_input_sketch_token,
        "--control-input-visible",
        control_input_visible,
        "--control-input-decision",
        control_input_decision,
        "--handled-errors",
        handled_errors,
        "--recovery-behavior",
        recovery_behavior,
        "--control-sketch-token",
        control_sketch_token,
        "--navigation-type",
        navigation_type,
        "--navigation-target",
        navigation_target,
        "--external-system",
        external_system,
        "--handoff-contract",
        handoff_contract,
    ] = args.as_slice()
    {
        let (slice_slug, view_name) = parse_slice_and_view_name(slice, name)?;
        let view_field = build_view_read_model_field(
            read_model,
            field,
            source_field,
            sketch_token,
            field_provenance,
            bit_encoding,
        )?;
        let (control_name, control_command) =
            parse_control_name_and_command(control, control_command)?;
        let provision = build_control_input_provision(
            control_input,
            control_input_source,
            control_input_description,
            control_input_sketch_token,
            control_input_visible,
            control_input_decision,
        )?;
        let (handled_errors, recovery_behavior, control_sketch_token) =
            parse_control_errors_recovery_sketch(
                handled_errors,
                recovery_behavior,
                control_sketch_token,
            )?;
        let navigation = build_navigation_external_system(
            navigation_type,
            navigation_target,
            external_system,
            handoff_contract,
        )?;
        let control = NewControlDefinition::new(
            control_name,
            control_command,
            provision,
            handled_errors,
            recovery_behavior,
            control_sketch_token,
            navigation,
        );
        build_add_view_cli(slice_slug, view_name, view_field, [control], None, None)
    } else {
        parse_cli_9(arguments)
    }
}

fn parse_cli_9(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            observes_flag,
            observes,
            source_event_flag,
            source_event,
            source_attribute_flag,
            source_attribute,
        ] if command == "add"
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && observes_flag == "--observes"
            && source_event_flag == "--source-event"
            && source_attribute_flag == "--source-attribute" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let observed_streams = parse_stream_names(observes)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_event = parse_event_name(source_event)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_attribute = parse_event_attribute_name(source_attribute)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            match input_source {
                                CommandInputSourceKind::EventStreamState => {
                                    CommandInputSource::event_stream_state(
                                        source_event,
                                        source_attribute,
                                    )
                                }
                                other => {
                                    return Err(command_input_source_kind_mismatch(
                                        other,
                                        CommandInputSourceKind::EventStreamState,
                                    ));
                                }
                            },
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    )
                    .with_observed_streams(CommandObservedStreams::from_streams(observed_streams)),
                },
            })
        }
        _ => parse_cli_10(arguments),
    }
}

fn parse_cli_10(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            source_payload_flag,
            source_payload,
            source_field_flag,
            source_field,
        ] if command == "add"
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && source_payload_flag == "--source-payload"
            && source_field_flag == "--source-field" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let source_payload = parse_event_attribute_source_name(source_payload)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_field = parse_event_attribute_source_field(source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            match input_source {
                                CommandInputSourceKind::ExternalPayload => {
                                    CommandInputSource::external_payload(
                                        source_payload,
                                        source_field,
                                    )
                                }
                                other => {
                                    return Err(command_input_source_kind_mismatch(
                                        other,
                                        CommandInputSourceKind::ExternalPayload,
                                    ));
                                }
                            },
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    ),
                },
            })
        }
        _ => parse_cli_11(arguments),
    }
}

fn parse_cli_11(arguments: &[String]) -> Result<Cli, ShellError> {
    let args: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let [
        "add",
        "view",
        "--slice",
        slice,
        "--name",
        name,
        "--read-model",
        read_model,
        "--field",
        field,
        "--source-field",
        source_field,
        "--sketch-token",
        sketch_token,
        "--field-provenance",
        field_provenance,
        "--bit-encoding",
        bit_encoding,
        "--control",
        control,
        "--control-command",
        control_command,
        "--control-input",
        control_input,
        "--control-input-source",
        control_input_source,
        "--control-input-description",
        control_input_description,
        "--control-input-sketch-token",
        control_input_sketch_token,
        "--control-input-visible",
        control_input_visible,
        "--control-input-decision",
        control_input_decision,
        "--handled-errors",
        handled_errors,
        "--recovery-behavior",
        recovery_behavior,
        "--control-sketch-token",
        control_sketch_token,
        "--navigation-type",
        navigation_type,
        "--navigation-target",
        navigation_target,
        "--external-workflow",
        external_workflow,
    ] = args.as_slice()
    {
        let (slice_slug, view_name) = parse_slice_and_view_name(slice, name)?;
        let view_field = build_view_read_model_field(
            read_model,
            field,
            source_field,
            sketch_token,
            field_provenance,
            bit_encoding,
        )?;
        let (control_name, control_command) =
            parse_control_name_and_command(control, control_command)?;
        let provision = build_control_input_provision(
            control_input,
            control_input_source,
            control_input_description,
            control_input_sketch_token,
            control_input_visible,
            control_input_decision,
        )?;
        let (handled_errors, recovery_behavior, control_sketch_token) =
            parse_control_errors_recovery_sketch(
                handled_errors,
                recovery_behavior,
                control_sketch_token,
            )?;
        let navigation = build_navigation_external_workflow(
            navigation_type,
            navigation_target,
            external_workflow,
        )?;
        let control = NewControlDefinition::new(
            control_name,
            control_command,
            provision,
            handled_errors,
            recovery_behavior,
            control_sketch_token,
            navigation,
        );
        build_add_view_cli(slice_slug, view_name, view_field, [control], None, None)
    } else {
        parse_cli_12(arguments)
    }
}

fn parse_cli_12(arguments: &[String]) -> Result<Cli, ShellError> {
    let args: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let [
        "add",
        "view",
        "--slice",
        slice,
        "--name",
        name,
        "--read-model",
        read_model,
        "--field",
        field,
        "--source-field",
        source_field,
        "--sketch-token",
        sketch_token,
        "--field-provenance",
        field_provenance,
        "--bit-encoding",
        bit_encoding,
        "--control",
        control,
        "--control-command",
        control_command,
        "--control-input",
        control_input,
        "--control-input-source",
        control_input_source,
        "--control-input-description",
        control_input_description,
        "--control-input-sketch-token",
        control_input_sketch_token,
        "--control-input-visible",
        control_input_visible,
        "--control-input-decision",
        control_input_decision,
        "--handled-errors",
        handled_errors,
        "--recovery-behavior",
        recovery_behavior,
        "--control-sketch-token",
        control_sketch_token,
        "--navigation-type",
        navigation_type,
        "--navigation-target",
        navigation_target,
        "--local-states",
        local_states,
        "--filters",
        filters,
    ] = args.as_slice()
    {
        let (slice_slug, view_name) = parse_slice_and_view_name(slice, name)?;
        let view_field = build_view_read_model_field(
            read_model,
            field,
            source_field,
            sketch_token,
            field_provenance,
            bit_encoding,
        )?;
        let (control_name, control_command) =
            parse_control_name_and_command(control, control_command)?;
        let provision = build_control_input_provision(
            control_input,
            control_input_source,
            control_input_description,
            control_input_sketch_token,
            control_input_visible,
            control_input_decision,
        )?;
        let (handled_errors, recovery_behavior, control_sketch_token) =
            parse_control_errors_recovery_sketch(
                handled_errors,
                recovery_behavior,
                control_sketch_token,
            )?;
        let navigation = build_navigation_target(navigation_type, navigation_target)?;
        let control = NewControlDefinition::new(
            control_name,
            control_command,
            provision,
            handled_errors,
            recovery_behavior,
            control_sketch_token,
            navigation,
        );
        build_add_view_cli(
            slice_slug,
            view_name,
            view_field,
            [control],
            Some(local_states),
            Some(filters),
        )
    } else {
        parse_cli_13(arguments)
    }
}

fn parse_cli_13(arguments: &[String]) -> Result<Cli, ShellError> {
    let args: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let [
        "add",
        "view",
        "--slice",
        slice,
        "--name",
        name,
        "--read-model",
        read_model,
        "--field",
        field,
        "--source-field",
        source_field,
        "--sketch-token",
        sketch_token,
        "--field-provenance",
        field_provenance,
        "--bit-encoding",
        bit_encoding,
        "--control",
        control,
        "--control-command",
        control_command,
        "--control-input",
        control_input,
        "--control-input-source",
        control_input_source,
        "--control-input-description",
        control_input_description,
        "--control-input-sketch-token",
        control_input_sketch_token,
        "--control-input-visible",
        control_input_visible,
        "--control-input-decision",
        control_input_decision,
        "--handled-errors",
        handled_errors,
        "--recovery-behavior",
        recovery_behavior,
        "--control-sketch-token",
        control_sketch_token,
        "--navigation-type",
        navigation_type,
        "--navigation-target",
        navigation_target,
    ] = args.as_slice()
    {
        let (slice_slug, view_name) = parse_slice_and_view_name(slice, name)?;
        let view_field = build_view_read_model_field(
            read_model,
            field,
            source_field,
            sketch_token,
            field_provenance,
            bit_encoding,
        )?;
        let (control_name, control_command) =
            parse_control_name_and_command(control, control_command)?;
        let provision = build_control_input_provision(
            control_input,
            control_input_source,
            control_input_description,
            control_input_sketch_token,
            control_input_visible,
            control_input_decision,
        )?;
        let (handled_errors, recovery_behavior, control_sketch_token) =
            parse_control_errors_recovery_sketch(
                handled_errors,
                recovery_behavior,
                control_sketch_token,
            )?;
        let navigation = build_navigation_target(navigation_type, navigation_target)?;
        let control = NewControlDefinition::new(
            control_name,
            control_command,
            provision,
            handled_errors,
            recovery_behavior,
            control_sketch_token,
            navigation,
        );
        build_add_view_cli(slice_slug, view_name, view_field, [control], None, None)
    } else {
        parse_cli_14(arguments)
    }
}

fn parse_cli_14(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            field_flag,
            field,
            field_source_flag,
            field_source,
            source_event_flag,
            source_event,
            source_attribute_flag,
            source_attribute,
            field_provenance_flag,
            field_provenance,
            transitive_flag,
            transitive,
            relationship_fields_flag,
            relationship_fields,
            transitive_rule_flag,
            transitive_rule,
            example_scenario_flag,
            example_scenario,
        ] if (command == "add" || command == "update")
            && subject == "read-model"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && field_flag == "--field"
            && field_source_flag == "--field-source"
            && source_event_flag == "--source-event"
            && source_attribute_flag == "--source-attribute"
            && field_provenance_flag == "--field-provenance"
            && transitive_flag == "--transitive"
            && relationship_fields_flag == "--relationship-fields"
            && transitive_rule_flag == "--transitive-rule"
            && example_scenario_flag == "--example-scenario" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let read_model_name = parse_read_model_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let field_name =
                parse_datum_name(field).map_err(|error| ShellError::message(error.to_string()))?;
            let field_source_kind = parse_read_model_field_source_kind(field_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_event = parse_event_name(source_event)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_attribute = parse_event_attribute_name(source_attribute)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let transitive = parse_bool_flag(transitive)?;
            let relationship_fields = parse_datum_names(relationship_fields)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let transitive_rule = parse_read_model_transitive_rule(transitive_rule)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let example_scenario = parse_scenario_name(example_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let read_model = NewReadModelDefinition::new(
                slice_slug,
                read_model_name,
                NewReadModelField::new(
                    field_name,
                    match field_source_kind {
                        ReadModelFieldSourceKind::EventAttribute => {
                            ReadModelFieldSource::event_attribute(source_event, source_attribute)
                        }
                        other => {
                            return Err(read_model_field_source_kind_mismatch(
                                other,
                                ReadModelFieldSourceKind::EventAttribute,
                            ));
                        }
                    },
                    provenance_description,
                ),
            );
            let read_model = if transitive {
                read_model.with_transitive_semantics(
                    ReadModelRelationshipFields::from_fields(relationship_fields),
                    transitive_rule,
                    example_scenario,
                )
            } else {
                read_model
            };
            Ok(read_model_definition_cli(command, read_model))
        }
        _ => parse_cli_15(arguments),
    }
}

fn parse_cli_15(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            field_flag,
            field,
            field_source_flag,
            field_source,
            derivation_rule_flag,
            derivation_rule,
            source_fields_flag,
            source_fields,
            derivation_scenario_flag,
            derivation_scenario,
            field_provenance_flag,
            field_provenance,
        ] if (command == "add" || command == "update")
            && subject == "read-model"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && field_flag == "--field"
            && field_source_flag == "--field-source"
            && derivation_rule_flag == "--derivation-rule"
            && source_fields_flag == "--source-fields"
            && derivation_scenario_flag == "--derivation-scenario"
            && field_provenance_flag == "--field-provenance" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let read_model_name = parse_read_model_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let field_name =
                parse_datum_name(field).map_err(|error| ShellError::message(error.to_string()))?;
            let field_source_kind = parse_read_model_field_source_kind(field_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let derivation_rule = parse_read_model_derivation_rule(derivation_rule)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_fields = parse_datum_names(source_fields)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let derivation_scenario = parse_scenario_name(derivation_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(read_model_definition_cli(
                command,
                NewReadModelDefinition::new(
                    slice_slug,
                    read_model_name,
                    NewReadModelField::new(
                        field_name,
                        match field_source_kind {
                            ReadModelFieldSourceKind::Derivation => {
                                ReadModelFieldSource::derivation(
                                    derivation_rule,
                                    ReadModelDerivationSourceFields::from_fields(source_fields),
                                    derivation_scenario,
                                )
                            }
                            other => {
                                return Err(read_model_field_source_kind_mismatch(
                                    other,
                                    ReadModelFieldSourceKind::Derivation,
                                ));
                            }
                        },
                        provenance_description,
                    ),
                ),
            ))
        }
        _ => parse_cli_16(arguments),
    }
}

fn parse_cli_16(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            stream_flag,
            stream,
            attribute_flag,
            attribute,
            attribute_source_flag,
            attribute_source,
            attribute_source_name_flag,
            attribute_source_name,
            attribute_source_field_flag,
            attribute_source_field,
            generated_source_kind_flag,
            generated_source_kind,
            attribute_provenance_flag,
            attribute_provenance,
            shared_flag,
            shared,
        ] if (command == "add" || command == "update")
            && subject == "event"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && stream_flag == "--stream"
            && attribute_flag == "--attribute"
            && attribute_source_flag == "--attribute-source"
            && attribute_source_name_flag == "--attribute-source-name"
            && attribute_source_field_flag == "--attribute-source-field"
            && generated_source_kind_flag == "--generated-source-kind"
            && attribute_provenance_flag == "--attribute-provenance"
            && shared_flag == "--shared" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let event_name =
                parse_event_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let stream_name = parse_stream_name(stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute = parse_event_attribute(
                attribute,
                attribute_source,
                attribute_source_name,
                attribute_source_field,
                Some(generated_source_kind),
                attribute_provenance,
            )?;
            let event = if parse_shared_flag(shared)? {
                NewEventDefinition::new_shared(slice_slug, event_name, stream_name, attribute)
            } else {
                NewEventDefinition::new(slice_slug, event_name, stream_name, attribute)
            };
            Ok(Cli {
                command: if command == "update" {
                    Command::UpdateEventDefinition { event }
                } else {
                    Command::AddEventDefinition { event }
                },
            })
        }
        _ => parse_cli_17(arguments),
    }
}

fn parse_cli_17(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            stream_flag,
            stream,
            attribute_flag,
            attribute,
            attribute_source_flag,
            attribute_source,
            attribute_source_name_flag,
            attribute_source_name,
            attribute_source_field_flag,
            attribute_source_field,
            generated_source_kind_flag,
            generated_source_kind,
            attribute_provenance_flag,
            attribute_provenance,
            observed_flag,
            observed,
        ] if (command == "add" || command == "update")
            && subject == "event"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && stream_flag == "--stream"
            && attribute_flag == "--attribute"
            && attribute_source_flag == "--attribute-source"
            && attribute_source_name_flag == "--attribute-source-name"
            && attribute_source_field_flag == "--attribute-source-field"
            && generated_source_kind_flag == "--generated-source-kind"
            && attribute_provenance_flag == "--attribute-provenance"
            && observed_flag == "--observed" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let event_name =
                parse_event_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let stream_name = parse_stream_name(stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute = parse_event_attribute(
                attribute,
                attribute_source,
                attribute_source_name,
                attribute_source_field,
                Some(generated_source_kind),
                attribute_provenance,
            )?;
            let event = if parse_observed_flag(observed)? {
                NewEventDefinition::new_observed(slice_slug, event_name, stream_name, attribute)
            } else {
                NewEventDefinition::new(slice_slug, event_name, stream_name, attribute)
            };
            Ok(Cli {
                command: if command == "update" {
                    Command::UpdateEventDefinition { event }
                } else {
                    Command::AddEventDefinition { event }
                },
            })
        }
        _ => parse_cli_18(arguments),
    }
}

fn parse_cli_18(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            stream_flag,
            stream,
            attribute_flag,
            attribute,
            attribute_source_flag,
            attribute_source,
            attribute_source_name_flag,
            attribute_source_name,
            attribute_source_field_flag,
            attribute_source_field,
            generated_source_kind_flag,
            generated_source_kind,
            attribute_provenance_flag,
            attribute_provenance,
        ] if (command == "add" || command == "update")
            && subject == "event"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && stream_flag == "--stream"
            && attribute_flag == "--attribute"
            && attribute_source_flag == "--attribute-source"
            && attribute_source_name_flag == "--attribute-source-name"
            && attribute_source_field_flag == "--attribute-source-field"
            && generated_source_kind_flag == "--generated-source-kind"
            && attribute_provenance_flag == "--attribute-provenance" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let event_name =
                parse_event_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let stream_name = parse_stream_name(stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute = parse_event_attribute(
                attribute,
                attribute_source,
                attribute_source_name,
                attribute_source_field,
                Some(generated_source_kind),
                attribute_provenance,
            )?;
            let event = NewEventDefinition::new(slice_slug, event_name, stream_name, attribute);
            Ok(Cli {
                command: if command == "update" {
                    Command::UpdateEventDefinition { event }
                } else {
                    Command::AddEventDefinition { event }
                },
            })
        }
        _ => parse_cli_19(arguments),
    }
}

fn parse_cli_19(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            field_flag,
            field,
            field_source_flag,
            field_source,
            absence_event_flag,
            absence_event,
            absence_scenario_flag,
            absence_scenario,
            field_provenance_flag,
            field_provenance,
        ] if (command == "add" || command == "update")
            && subject == "read-model"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && field_flag == "--field"
            && field_source_flag == "--field-source"
            && absence_event_flag == "--absence-event"
            && absence_scenario_flag == "--absence-scenario"
            && field_provenance_flag == "--field-provenance" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let read_model_name = parse_read_model_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let field_name =
                parse_datum_name(field).map_err(|error| ShellError::message(error.to_string()))?;
            let field_source_kind = parse_read_model_field_source_kind(field_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let absence_event = parse_event_name(absence_event)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let absence_scenario = parse_scenario_name(absence_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(read_model_definition_cli(
                command,
                NewReadModelDefinition::new(
                    slice_slug,
                    read_model_name,
                    NewReadModelField::new(
                        field_name,
                        match field_source_kind {
                            ReadModelFieldSourceKind::AbsenceDefault => {
                                ReadModelFieldSource::absence_default(
                                    absence_event,
                                    absence_scenario,
                                )
                            }
                            other => {
                                return Err(read_model_field_source_kind_mismatch(
                                    other,
                                    ReadModelFieldSourceKind::AbsenceDefault,
                                ));
                            }
                        },
                        provenance_description,
                    ),
                ),
            ))
        }
        _ => parse_cli_20(arguments),
    }
}

fn parse_cli_20(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            kind_flag,
            kind,
            lane_flag,
            lane,
            declared_name_flag,
            declared_name,
            main_path_flag,
            main_path,
        ] if (command == "add" || command == "update")
            && subject == "board-element"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && kind_flag == "--kind"
            && lane_flag == "--lane"
            && declared_name_flag == "--declared-name"
            && main_path_flag == "--main-path" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let element_name = parse_board_element_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let element_kind = parse_board_element_kind(kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let lane_id = parse_board_lane_id(lane)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let declared_name = parse_board_element_declared_name(declared_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let main_path = parse_bool_flag(main_path)?;
            let element = NewBoardElement::new(
                slice_slug,
                element_name,
                element_kind,
                lane_id,
                declared_name,
                main_path,
            );
            let command = if command == "add" {
                Command::AddBoardElement { element }
            } else {
                Command::UpdateBoardElement { element }
            };
            Ok(Cli { command })
        }
        [command, subject, slice_flag, slice, name_flag, name]
            if command == "remove"
                && subject == "board-element"
                && slice_flag == "--slice"
                && name_flag == "--name" =>
        {
            Ok(Cli {
                command: Command::RemoveBoardElement {
                    slice_slug: parse_slice_slug(slice)
                        .map_err(|error| ShellError::message(error.to_string()))?,
                    element_name: parse_board_element_name(name)
                        .map_err(|error| ShellError::message(error.to_string()))?,
                },
            })
        }
        _ => parse_cli_21(arguments),
    }
}

fn parse_board_connection_command(arguments: &[String]) -> Result<Option<Cli>, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            source_flag,
            source,
            source_kind_flag,
            source_kind,
            target_flag,
            target,
            target_kind_flag,
            target_kind,
        ] if (command == "add" || command == "remove")
            && subject == "board-connection"
            && slice_flag == "--slice"
            && source_flag == "--source"
            && source_kind_flag == "--source-kind"
            && target_flag == "--target"
            && target_kind_flag == "--target-kind" =>
        {
            let connection =
                parse_board_connection_cli(slice, source, source_kind, target, target_kind)?;
            let command = if command == "add" {
                Command::AddBoardConnection { connection }
            } else {
                Command::RemoveBoardConnection { connection }
            };
            Ok(Some(Cli { command }))
        }
        [
            command,
            subject,
            slice_flag,
            slice,
            source_flag,
            source,
            source_kind_flag,
            source_kind,
            target_flag,
            target,
            target_kind_flag,
            target_kind,
            new_source_flag,
            new_source,
            new_source_kind_flag,
            new_source_kind,
            new_target_flag,
            new_target,
            new_target_kind_flag,
            new_target_kind,
        ] if command == "update"
            && subject == "board-connection"
            && slice_flag == "--slice"
            && source_flag == "--source"
            && source_kind_flag == "--source-kind"
            && target_flag == "--target"
            && target_kind_flag == "--target-kind"
            && new_source_flag == "--new-source"
            && new_source_kind_flag == "--new-source-kind"
            && new_target_flag == "--new-target"
            && new_target_kind_flag == "--new-target-kind" =>
        {
            Ok(Some(Cli {
                command: Command::UpdateBoardConnection {
                    previous: parse_board_connection_cli(
                        slice,
                        source,
                        source_kind,
                        target,
                        target_kind,
                    )?,
                    replacement: parse_board_connection_cli(
                        slice,
                        new_source,
                        new_source_kind,
                        new_target,
                        new_target_kind,
                    )?,
                },
            }))
        }
        _ => Ok(None),
    }
}

fn parse_cli_21(arguments: &[String]) -> Result<Cli, ShellError> {
    if let Some(cli) = parse_board_connection_command(arguments)? {
        return Ok(cli);
    }

    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            field_flag,
            field,
            field_provenance_flag,
            field_provenance,
            bit_encoding_flag,
            bit_encoding,
        ] if (command == "add" || command == "update")
            && subject == "external-payload"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && field_flag == "--field"
            && field_provenance_flag == "--field-provenance"
            && bit_encoding_flag == "--bit-encoding" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let payload_name = parse_event_attribute_source_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let payload_field = parse_event_attribute_source_field(field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let field_provenance = parse_provenance_description(field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let bit_encoding = parse_bit_encoding_semantics(bit_encoding)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let external_payload = NewExternalPayloadDefinition::new(
                slice_slug,
                payload_name,
                payload_field,
                field_provenance,
                bit_encoding,
            );
            let command = if command == "add" {
                Command::AddExternalPayloadDefinition { external_payload }
            } else {
                Command::UpdateExternalPayloadDefinition { external_payload }
            };
            Ok(Cli { command })
        }
        _ => parse_cli_22(arguments),
    }
}

fn parse_cli_22(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            label_flag,
            label,
            events_flag,
            events,
            externally_relevant_flag,
            externally_relevant,
        ] if (command == "add" || command == "update")
            && subject == "outcome"
            && slice_flag == "--slice"
            && label_flag == "--label"
            && events_flag == "--events"
            && externally_relevant_flag == "--externally-relevant" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let label = parse_outcome_label_name(label)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let events = parse_event_names(events)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let externally_relevant = parse_bool_flag(externally_relevant)?;
            let outcome = NewOutcomeDefinition::new(
                slice_slug,
                label,
                OutcomeEventNames::from_events(events),
                externally_relevant,
            );
            let command = if command == "add" {
                Command::AddOutcomeDefinition { outcome }
            } else {
                Command::UpdateOutcomeDefinition { outcome }
            };
            Ok(Cli { command })
        }
        _ => parse_cli_23(arguments),
    }
}

fn parse_cli_23(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            read_model_flag,
            read_model,
            field_flag,
            field,
            source_field_flag,
            source_field,
            sketch_token_flag,
            sketch_token,
            field_provenance_flag,
            field_provenance,
            bit_encoding_flag,
            bit_encoding,
        ] if (command == "add" || command == "update")
            && subject == "view"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && read_model_flag == "--read-model"
            && field_flag == "--field"
            && source_field_flag == "--source-field"
            && sketch_token_flag == "--sketch-token"
            && field_provenance_flag == "--field-provenance"
            && bit_encoding_flag == "--bit-encoding" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let view_name =
                parse_view_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let read_model_name = parse_read_model_name(read_model)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let field_name = parse_view_field_name(field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_field = parse_view_field_name(source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let sketch_token = parse_sketch_token(sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let bit_encoding = parse_bit_encoding_semantics(bit_encoding)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(view_definition_cli(
                command,
                NewViewDefinition::new(
                    slice_slug,
                    view_name,
                    NewViewField::new(
                        field_name,
                        parse_view_field_source_kind("read_model")
                            .map_err(|error| ShellError::message(error.to_string()))?,
                        read_model_name,
                        source_field,
                        sketch_token,
                        provenance_description,
                        bit_encoding,
                    ),
                ),
            ))
        }
        _ => parse_cli_24(arguments),
    }
}

fn parse_cli_24(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            field_flag,
            field,
            field_source_flag,
            field_source,
            source_event_flag,
            source_event,
            source_attribute_flag,
            source_attribute,
            field_provenance_flag,
            field_provenance,
        ] if (command == "add" || command == "update")
            && subject == "read-model"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && field_flag == "--field"
            && field_source_flag == "--field-source"
            && source_event_flag == "--source-event"
            && source_attribute_flag == "--source-attribute"
            && field_provenance_flag == "--field-provenance" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let read_model_name = parse_read_model_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let field_name =
                parse_datum_name(field).map_err(|error| ShellError::message(error.to_string()))?;
            let field_source_kind = parse_read_model_field_source_kind(field_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_event = parse_event_name(source_event)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_attribute = parse_event_attribute_name(source_attribute)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(read_model_definition_cli(
                command,
                NewReadModelDefinition::new(
                    slice_slug,
                    read_model_name,
                    NewReadModelField::new(
                        field_name,
                        match field_source_kind {
                            ReadModelFieldSourceKind::EventAttribute => {
                                ReadModelFieldSource::event_attribute(
                                    source_event,
                                    source_attribute,
                                )
                            }
                            other => {
                                return Err(read_model_field_source_kind_mismatch(
                                    other,
                                    ReadModelFieldSourceKind::EventAttribute,
                                ));
                            }
                        },
                        provenance_description,
                    ),
                ),
            ))
        }
        _ => parse_cli_25(arguments),
    }
}

fn parse_cli_25(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            stream_flag,
            stream,
            attribute_flag,
            attribute,
            attribute_source_flag,
            attribute_source,
            attribute_source_name_flag,
            attribute_source_name,
            attribute_source_field_flag,
            attribute_source_field,
            attribute_provenance_flag,
            attribute_provenance,
            shared_flag,
            shared,
        ] if command == "add"
            && subject == "event"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && stream_flag == "--stream"
            && attribute_flag == "--attribute"
            && attribute_source_flag == "--attribute-source"
            && attribute_source_name_flag == "--attribute-source-name"
            && attribute_source_field_flag == "--attribute-source-field"
            && attribute_provenance_flag == "--attribute-provenance"
            && shared_flag == "--shared" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let event_name =
                parse_event_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let stream_name = parse_stream_name(stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_name = parse_event_attribute_name(attribute)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_kind = parse_event_attribute_source_kind(attribute_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_name = parse_event_attribute_source_name(attribute_source_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_field = parse_event_attribute_source_field(attribute_source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(attribute_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute = NewEventAttribute::new(
                attribute_name,
                attribute_source_kind,
                attribute_source_name,
                attribute_source_field,
                provenance_description,
            );
            let event = if parse_shared_flag(shared)? {
                NewEventDefinition::new_shared(slice_slug, event_name, stream_name, attribute)
            } else {
                NewEventDefinition::new(slice_slug, event_name, stream_name, attribute)
            };
            Ok(Cli {
                command: Command::AddEventDefinition { event },
            })
        }
        _ => parse_cli_26(arguments),
    }
}

fn parse_cli_26(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            stream_flag,
            stream,
            attribute_flag,
            attribute,
            attribute_source_flag,
            attribute_source,
            attribute_source_name_flag,
            attribute_source_name,
            attribute_source_field_flag,
            attribute_source_field,
            attribute_provenance_flag,
            attribute_provenance,
            observed_flag,
            observed,
        ] if command == "add"
            && subject == "event"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && stream_flag == "--stream"
            && attribute_flag == "--attribute"
            && attribute_source_flag == "--attribute-source"
            && attribute_source_name_flag == "--attribute-source-name"
            && attribute_source_field_flag == "--attribute-source-field"
            && attribute_provenance_flag == "--attribute-provenance"
            && observed_flag == "--observed" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let event_name =
                parse_event_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let stream_name = parse_stream_name(stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_name = parse_event_attribute_name(attribute)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_kind = parse_event_attribute_source_kind(attribute_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_name = parse_event_attribute_source_name(attribute_source_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_field = parse_event_attribute_source_field(attribute_source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(attribute_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute = NewEventAttribute::new(
                attribute_name,
                attribute_source_kind,
                attribute_source_name,
                attribute_source_field,
                provenance_description,
            );
            let event = if parse_observed_flag(observed)? {
                NewEventDefinition::new_observed(slice_slug, event_name, stream_name, attribute)
            } else {
                NewEventDefinition::new(slice_slug, event_name, stream_name, attribute)
            };
            Ok(Cli {
                command: Command::AddEventDefinition { event },
            })
        }
        _ => parse_cli_27(arguments),
    }
}

fn parse_cli_27(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            stream_flag,
            stream,
            attribute_flag,
            attribute,
            attribute_source_flag,
            attribute_source,
            attribute_source_name_flag,
            attribute_source_name,
            attribute_source_field_flag,
            attribute_source_field,
            attribute_provenance_flag,
            attribute_provenance,
        ] if command == "add"
            && subject == "event"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && stream_flag == "--stream"
            && attribute_flag == "--attribute"
            && attribute_source_flag == "--attribute-source"
            && attribute_source_name_flag == "--attribute-source-name"
            && attribute_source_field_flag == "--attribute-source-field"
            && attribute_provenance_flag == "--attribute-provenance" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let event_name =
                parse_event_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let stream_name = parse_stream_name(stream)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_name = parse_event_attribute_name(attribute)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_kind = parse_event_attribute_source_kind(attribute_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_name = parse_event_attribute_source_name(attribute_source_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let attribute_source_field = parse_event_attribute_source_field(attribute_source_field)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(attribute_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddEventDefinition {
                    event: NewEventDefinition::new(
                        slice_slug,
                        event_name,
                        stream_name,
                        NewEventAttribute::new(
                            attribute_name,
                            attribute_source_kind,
                            attribute_source_name,
                            attribute_source_field,
                            provenance_description,
                        ),
                    ),
                },
            })
        }
        _ => parse_cli_update_scenario_name_first(arguments),
    }
}

fn parse_cli_update_scenario_name_first(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            kind_flag,
            scenario_kind,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
        ] if command == "update"
            && subject == "scenario"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && kind_flag == "--kind"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let scenario =
                parse_basic_scenario(slice_slug, scenario_kind, name, given, when, then)?;
            Ok(Cli {
                command: Command::UpdateSliceScenario { scenario },
            })
        }
        _ => parse_cli_update_scenario_kind_first(arguments),
    }
}

fn parse_cli_update_scenario_kind_first(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            kind_flag,
            scenario_kind,
            name_flag,
            name,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
        ] if command == "update"
            && subject == "scenario"
            && slice_flag == "--slice"
            && kind_flag == "--kind"
            && name_flag == "--name"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let scenario =
                parse_basic_scenario(slice_slug, scenario_kind, name, given, when, then)?;
            Ok(Cli {
                command: Command::UpdateSliceScenario { scenario },
            })
        }
        _ => parse_cli_28(arguments),
    }
}

fn parse_cli_28(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            kind_flag,
            scenario_kind,
            name_flag,
            name,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
        ] if command == "add"
            && subject == "scenario"
            && slice_flag == "--slice"
            && kind_flag == "--kind"
            && name_flag == "--name"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let scenario =
                parse_basic_scenario(slice_slug, scenario_kind, name, given, when, then)?;
            Ok(Cli {
                command: Command::AddSliceScenario { scenario },
            })
        }
        [
            command,
            subject,
            slice_flag,
            slice,
            kind_flag,
            scenario_kind,
            name_flag,
            name,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
            read_streams_flag,
            read_streams,
            written_streams_flag,
            written_streams,
        ] if command == "add"
            && subject == "scenario"
            && slice_flag == "--slice"
            && kind_flag == "--kind"
            && name_flag == "--name"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then"
            && read_streams_flag == "--read-streams"
            && written_streams_flag == "--written-streams" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let (read_streams, written_streams) =
                parse_scenario_streams(read_streams, written_streams)?;
            let scenario =
                parse_basic_scenario(slice_slug, scenario_kind, name, given, when, then)?
                    .with_streams(read_streams, written_streams);
            Ok(Cli {
                command: Command::AddSliceScenario { scenario },
            })
        }
        _ => parse_cli_29(arguments),
    }
}

fn parse_cli_29(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            kind_flag,
            scenario_kind,
            name_flag,
            name,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
            contract_kind_flag,
            contract_kind,
            covered_definition_flag,
            covered_definition,
        ] if command == "add"
            && subject == "scenario"
            && slice_flag == "--slice"
            && kind_flag == "--kind"
            && name_flag == "--name"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then"
            && contract_kind_flag == "--contract-kind"
            && covered_definition_flag == "--covered-definition" =>
        {
            let scenario_kind = parse_scenario_kind(scenario_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            if scenario_kind != ScenarioKind::contract() {
                return Err(ShellError::message(
                    "--contract-kind and --covered-definition require --kind contract",
                ));
            }
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let name = parse_scenario_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let given = parse_scenario_step_text(given)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let when = parse_scenario_step_text(when)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let then = parse_scenario_step_text(then)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let contract_kind = parse_contract_kind_name(contract_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let covered_definition = parse_covered_definition_name(covered_definition)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddSliceScenario {
                    scenario: NewSliceScenario::new_contract(
                        slice_slug,
                        name,
                        given,
                        when,
                        then,
                        contract_kind,
                        covered_definition,
                    ),
                },
            })
        }
        _ => parse_cli_30(arguments),
    }
}

fn parse_cli_30(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            kind_flag,
            scenario_kind,
            name_flag,
            name,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
            contract_kind_flag,
            contract_kind,
            covered_definition_flag,
            covered_definition,
            error_references_flag,
            error_references,
        ] if command == "add"
            && subject == "scenario"
            && slice_flag == "--slice"
            && kind_flag == "--kind"
            && name_flag == "--name"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then"
            && contract_kind_flag == "--contract-kind"
            && covered_definition_flag == "--covered-definition"
            && error_references_flag == "--error-references" =>
        {
            let scenario_kind = parse_scenario_kind(scenario_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            if scenario_kind != ScenarioKind::contract() {
                return Err(ShellError::message(
                    "--contract-kind and --covered-definition require --kind contract",
                ));
            }
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let name = parse_scenario_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let given = parse_scenario_step_text(given)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let when = parse_scenario_step_text(when)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let then = parse_scenario_step_text(then)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let contract_kind = parse_contract_kind_name(contract_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let covered_definition = parse_covered_definition_name(covered_definition)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let error_references = parse_command_error_names(error_references)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddSliceScenario {
                    scenario: NewSliceScenario::new_contract(
                        slice_slug,
                        name,
                        given,
                        when,
                        then,
                        contract_kind,
                        covered_definition,
                    )
                    .with_error_references(CommandErrorNames::from_names(error_references)),
                },
            })
        }
        _ => parse_cli_31(arguments),
    }
}

fn parse_cli_31(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            trigger_flag,
            trigger,
            command_flag,
            issued_command,
            handled_errors_flag,
            handled_errors,
            reaction_flag,
            reaction,
        ] if (command == "add" || command == "update")
            && subject == "automation"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && trigger_flag == "--trigger"
            && command_flag == "--command"
            && handled_errors_flag == "--handled-errors"
            && reaction_flag == "--reaction" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let automation_name = parse_automation_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger_name = parse_automation_trigger_name(trigger)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command_name = parse_command_name(issued_command)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let handled_errors = parse_command_error_names(handled_errors)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let reaction_description = parse_automation_reaction_description(reaction)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let automation = NewAutomationDefinition::new(
                slice_slug,
                automation_name,
                trigger_name,
                command_name,
                CommandErrorNames::from_names(handled_errors),
                reaction_description,
            );
            let command = if command == "add" {
                Command::AddAutomationDefinition { automation }
            } else {
                Command::UpdateAutomationDefinition { automation }
            };
            Ok(Cli { command })
        }
        _ => parse_cli_32(arguments),
    }
}

fn parse_cli_32(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            external_event_flag,
            external_event,
            payload_contract_flag,
            payload_contract,
            command_flag,
            target_command,
        ] if (command == "add" || command == "update")
            && subject == "translation"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && external_event_flag == "--external-event"
            && payload_contract_flag == "--payload-contract"
            && command_flag == "--command" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let translation_name = parse_translation_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let external_event_name = parse_translation_external_event_name(external_event)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let payload_contract_name = parse_payload_contract_name(payload_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command_name = parse_command_name(target_command)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let translation = NewTranslationDefinition::new(
                slice_slug,
                translation_name,
                external_event_name,
                payload_contract_name,
                command_name,
            );
            let command = if command == "add" {
                Command::AddTranslationDefinition { translation }
            } else {
                Command::UpdateTranslationDefinition { translation }
            };
            Ok(Cli { command })
        }
        _ => parse_cli_33(arguments),
    }
}

fn parse_cli_33(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            kind_flag,
            scenario_kind,
            name_flag,
            name,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
            contract_kind_flag,
            contract_kind,
            covered_definition_flag,
            covered_definition,
            read_streams_flag,
            read_streams,
            written_streams_flag,
            written_streams,
        ] if command == "add"
            && subject == "scenario"
            && slice_flag == "--slice"
            && kind_flag == "--kind"
            && name_flag == "--name"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then"
            && contract_kind_flag == "--contract-kind"
            && covered_definition_flag == "--covered-definition"
            && read_streams_flag == "--read-streams"
            && written_streams_flag == "--written-streams" =>
        {
            let scenario_kind = parse_scenario_kind(scenario_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            if scenario_kind != ScenarioKind::contract() {
                return Err(ShellError::message(
                    "--contract-kind and --covered-definition require --kind contract",
                ));
            }
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let name = parse_scenario_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let given = parse_scenario_step_text(given)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let when = parse_scenario_step_text(when)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let then = parse_scenario_step_text(then)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let contract_kind = parse_contract_kind_name(contract_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let covered_definition = parse_covered_definition_name(covered_definition)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let (read_streams, written_streams) =
                parse_scenario_streams(read_streams, written_streams)?;
            Ok(Cli {
                command: Command::AddSliceScenario {
                    scenario: NewSliceScenario::new_contract(
                        slice_slug,
                        name,
                        given,
                        when,
                        then,
                        contract_kind,
                        covered_definition,
                    )
                    .with_streams(read_streams, written_streams),
                },
            })
        }
        _ => parse_cli_34(arguments),
    }
}

fn parse_cli_34(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            kind_flag,
            scenario_kind,
            name_flag,
            name,
            given_flag,
            given,
            when_flag,
            when,
            then_flag,
            then,
            contract_kind_flag,
            contract_kind,
            covered_definition_flag,
            covered_definition,
            read_streams_flag,
            read_streams,
            written_streams_flag,
            written_streams,
            error_references_flag,
            error_references,
        ] if command == "add"
            && subject == "scenario"
            && slice_flag == "--slice"
            && kind_flag == "--kind"
            && name_flag == "--name"
            && given_flag == "--given"
            && when_flag == "--when"
            && then_flag == "--then"
            && contract_kind_flag == "--contract-kind"
            && covered_definition_flag == "--covered-definition"
            && read_streams_flag == "--read-streams"
            && written_streams_flag == "--written-streams"
            && error_references_flag == "--error-references" =>
        {
            let scenario_kind = parse_scenario_kind(scenario_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            if scenario_kind != ScenarioKind::contract() {
                return Err(ShellError::message(
                    "--contract-kind and --covered-definition require --kind contract",
                ));
            }
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let name = parse_scenario_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let given = parse_scenario_step_text(given)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let when = parse_scenario_step_text(when)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let then = parse_scenario_step_text(then)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let contract_kind = parse_contract_kind_name(contract_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let covered_definition = parse_covered_definition_name(covered_definition)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let (read_streams, written_streams) =
                parse_scenario_streams(read_streams, written_streams)?;
            let error_references = parse_command_error_names(error_references)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddSliceScenario {
                    scenario: NewSliceScenario::new_contract(
                        slice_slug,
                        name,
                        given,
                        when,
                        then,
                        contract_kind,
                        covered_definition,
                    )
                    .with_streams(read_streams, written_streams)
                    .with_error_references(CommandErrorNames::from_names(error_references)),
                },
            })
        }
        _ => parse_cli_35(arguments),
    }
}

fn parse_cli_35(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            error_flag,
            error,
            error_scenario_flag,
            error_scenario,
            error_recovery_flag,
            error_recovery,
        ] if command == "add"
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && error_flag == "--error"
            && error_scenario_flag == "--error-scenario"
            && error_recovery_flag == "--error-recovery" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let command_error = parse_command_error_name(error)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command_error_scenario = parse_scenario_name(error_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command_error_recovery = parse_command_error_recovery_kind(error_recovery)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            require_actor_command_input_source(input_source)?,
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    )
                    .with_errors(CommandErrorDefinitions::from_errors([
                        NewCommandErrorDefinition::new(
                            command_error,
                            command_error_scenario,
                            command_error_recovery,
                        ),
                    ])),
                },
            })
        }
        _ => parse_cli_36(arguments),
    }
}

fn parse_cli_36(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            singleton_flag,
            singleton,
            repeat_behavior_flag,
            repeat_behavior,
        ] if command == "add"
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && singleton_flag == "--singleton"
            && repeat_behavior_flag == "--repeat-behavior" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let singleton = parse_bool_flag(singleton)?;
            let repeat_behavior = parse_singleton_repeat_behavior(repeat_behavior)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command = NewCommandDefinition::new(
                slice_slug,
                command_name,
                NewCommandInput::new(
                    input_name,
                    require_actor_command_input_source(input_source)?,
                    input_description,
                    CommandInputProvenanceChain::from_hops(provenance_chain),
                ),
                EmittedEventNames::from_events(emitted_events),
            );
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: if singleton {
                        command.with_singleton_repeat_behavior(repeat_behavior)
                    } else {
                        command
                    },
                },
            })
        }
        _ => parse_cli_37(arguments),
    }
}

fn parse_cli_37(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
            observes_flag,
            observes,
        ] if command == "add"
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits"
            && observes_flag == "--observes" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let observed_streams = parse_stream_names(observes)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            require_actor_command_input_source(input_source)?,
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    )
                    .with_observed_streams(CommandObservedStreams::from_streams(observed_streams)),
                },
            })
        }
        _ => parse_cli_38(arguments),
    }
}

fn parse_cli_38(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slice_flag,
            slice,
            name_flag,
            name,
            input_flag,
            input,
            input_source_flag,
            input_source,
            input_description_flag,
            input_description,
            input_provenance_flag,
            input_provenance,
            emits_flag,
            emits,
        ] if (command == "add" || command == "update")
            && subject == "command"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && input_flag == "--input"
            && input_source_flag == "--input-source"
            && input_description_flag == "--input-description"
            && input_provenance_flag == "--input-provenance"
            && emits_flag == "--emits" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let command_name =
                parse_command_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let input_name =
                parse_datum_name(input).map_err(|error| ShellError::message(error.to_string()))?;
            let input_source = parse_command_input_source_kind(input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let input_description = parse_command_input_source_description(input_description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_chain = parse_source_chain_hops(input_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let emitted_events =
                parse_event_names(emits).map_err(|error| ShellError::message(error.to_string()))?;
            let command_definition = NewCommandDefinition::new(
                slice_slug,
                command_name,
                NewCommandInput::new(
                    input_name,
                    require_actor_command_input_source(input_source)?,
                    input_description,
                    CommandInputProvenanceChain::from_hops(provenance_chain),
                ),
                EmittedEventNames::from_events(emitted_events),
            );
            Ok(Cli {
                command: if command == "update" {
                    Command::UpdateCommandDefinition {
                        command: command_definition,
                    }
                } else {
                    Command::AddCommandDefinition {
                        command: command_definition,
                    }
                },
            })
        }
        _ => parse_cli_39(arguments),
    }
}

fn parse_cli_39(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            slug_flag,
            slug,
            name_flag,
            name,
            type_flag,
            slice_type,
            description_flag,
            description,
        ] if command == "add"
            && subject == "slice"
            && workflow_flag == "--workflow"
            && slug_flag == "--slug"
            && name_flag == "--name"
            && type_flag == "--type"
            && description_flag == "--description" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_kind = parse_slice_kind(slice_type)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let slice_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddSlice {
                    slice: NewSlice::new(
                        workflow_slug,
                        slice_slug,
                        slice_name,
                        slice_description,
                        slice_kind,
                    ),
                },
            })
        }
        _ => parse_cli_40(arguments),
    }
}

fn parse_cli_40(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            payload_contract_flag,
            payload_contract,
        ] if command == "connect"
            && subject == "workflow"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name"
            && payload_contract_flag == "--payload-contract" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug =
                parse_slice_slug(target).map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let payload_contract = parse_payload_contract_name(payload_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::ConnectWorkflow {
                    connection: WorkflowConnection::new_with_payload_contract(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                        payload_contract,
                    ),
                },
            })
        }
        [
            command,
            subject,
            slug_flag,
            slug,
            name_flag,
            name,
            description_flag,
            description,
        ] if command == "add"
            && subject == "workflow"
            && slug_flag == "--slug"
            && name_flag == "--name"
            && description_flag == "--description" =>
        {
            let workflow_slug = parse_workflow_slug(slug)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflow {
                    workflow: NewWorkflow::new(workflow_name, workflow_description, workflow_slug),
                },
            })
        }
        _ => parse_cli_41(arguments),
    }
}

fn parse_cli_41(arguments: &[String]) -> Result<Cli, ShellError> {
    if let Some(cli) = parse_workflow_outcome_cli(arguments)? {
        return Ok(cli);
    }
    if let Some(cli) = parse_workflow_command_error_cli(arguments)? {
        return Ok(cli);
    }

    parse_cli_42(arguments)
}

fn parse_workflow_command_error_cli(arguments: &[String]) -> Result<Option<Cli>, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            command_flag,
            command_name,
            error_flag,
            error_name,
            new_source_slice_flag,
            new_source_slice,
            new_command_flag,
            new_command,
            new_error_flag,
            new_error,
        ] if command == "update"
            && subject == "workflow-command-error"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && command_flag == "--command"
            && error_flag == "--error"
            && new_source_slice_flag == "--new-source-slice"
            && new_command_flag == "--new-command"
            && new_error_flag == "--new-error" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let previous =
                parse_workflow_command_error_record(source_slice, command_name, error_name)?;
            let replacement =
                parse_workflow_command_error_record(new_source_slice, new_command, new_error)?;
            Ok(Some(Cli {
                command: Command::UpdateWorkflowCommandError {
                    workflow_slug,
                    previous,
                    replacement,
                },
            }))
        }
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            command_flag,
            command_name,
            error_flag,
            error_name,
        ] if (command == "add" || command == "remove")
            && subject == "workflow-command-error"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && command_flag == "--command"
            && error_flag == "--error" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let error =
                parse_workflow_command_error_record(source_slice, command_name, error_name)?;
            Ok(Some(Cli {
                command: if command == "add" {
                    Command::AddWorkflowCommandError {
                        workflow_slug,
                        error,
                    }
                } else {
                    Command::RemoveWorkflowCommandError {
                        workflow_slug,
                        error,
                    }
                },
            }))
        }
        _ => Ok(None),
    }
}

fn parse_workflow_command_error_record(
    source_slice: &str,
    command_name: &str,
    error_name: &str,
) -> Result<WorkflowCommandErrorRecord, ShellError> {
    let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
        .map_err(|error| ShellError::message(error.to_string()))?;
    let command_name =
        parse_command_name(command_name).map_err(|error| ShellError::message(error.to_string()))?;
    let error_name = parse_command_error_name(error_name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(WorkflowCommandErrorRecord::new(
        source_slice,
        command_name,
        error_name,
    ))
}

fn parse_workflow_outcome_cli(arguments: &[String]) -> Result<Option<Cli>, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            label_flag,
            label,
            externally_relevant_flag,
            externally_relevant,
            new_source_slice_flag,
            new_source_slice,
            new_label_flag,
            new_label,
            new_externally_relevant_flag,
            new_externally_relevant,
        ] if command == "update"
            && subject == "workflow-outcome"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && label_flag == "--label"
            && externally_relevant_flag == "--externally-relevant"
            && new_source_slice_flag == "--new-source-slice"
            && new_label_flag == "--new-label"
            && new_externally_relevant_flag == "--new-externally-relevant" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let previous = parse_workflow_outcome_record(source_slice, label, externally_relevant)?;
            let replacement = parse_workflow_outcome_record(
                new_source_slice,
                new_label,
                new_externally_relevant,
            )?;
            Ok(Some(Cli {
                command: Command::UpdateWorkflowOutcome {
                    workflow_slug,
                    previous,
                    replacement,
                },
            }))
        }
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            label_flag,
            label,
            externally_relevant_flag,
            externally_relevant,
        ] if (command == "add" || command == "remove")
            && subject == "workflow-outcome"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && label_flag == "--label"
            && externally_relevant_flag == "--externally-relevant" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let outcome = parse_workflow_outcome_record(source_slice, label, externally_relevant)?;
            Ok(Some(Cli {
                command: if command == "add" {
                    Command::AddWorkflowOutcome {
                        workflow_slug,
                        outcome,
                    }
                } else {
                    Command::RemoveWorkflowOutcome {
                        workflow_slug,
                        outcome,
                    }
                },
            }))
        }
        _ => Ok(None),
    }
}

fn parse_workflow_outcome_record(
    source_slice: &str,
    label: &str,
    externally_relevant: &str,
) -> Result<WorkflowOutcomeRecord, ShellError> {
    let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
        .map_err(|error| ShellError::message(error.to_string()))?;
    let label =
        parse_outcome_label_name(label).map_err(|error| ShellError::message(error.to_string()))?;
    let externally_relevant = parse_bool_flag(externally_relevant)?;
    Ok(WorkflowOutcomeRecord::new(
        source_slice,
        label,
        externally_relevant,
    ))
}

fn parse_cli_42(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_slice_flag,
            source_slice,
            definition_kind_flag,
            definition_kind,
            definition_name_flag,
            definition_name,
        ] if command == "add"
            && subject == "workflow-owned-definition"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && definition_kind_flag == "--definition-kind"
            && definition_name_flag == "--definition-name" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_kind = parse_workflow_owned_definition_kind(definition_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let definition_name = parse_workflow_owned_definition_name(definition_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowOwnedDefinition {
                    workflow_slug,
                    definition: WorkflowOwnedDefinitionRecord::new(
                        source_slice,
                        definition_kind,
                        definition_name,
                    ),
                },
            })
        }
        _ => parse_cli_43(arguments),
    }
}

fn parse_cli_43(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_flag,
            source,
            target_flag,
            target,
            kind_flag,
            kind,
            trigger_flag,
            trigger,
            source_evidence_flag,
            source_evidence,
            target_evidence_flag,
            target_evidence,
        ] if command == "add"
            && subject == "workflow-transition-evidence"
            && workflow_flag == "--workflow"
            && source_flag == "--from"
            && target_flag == "--to"
            && kind_flag == "--via"
            && trigger_flag == "--name"
            && source_evidence_flag == "--source-evidence"
            && target_evidence_flag == "--target-evidence" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source = parse_workflow_transition_endpoint(source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target = parse_workflow_transition_endpoint(target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let kind = parse_workflow_transition_kind(kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            if kind == WorkflowTransitionKind::Navigation {
                return Err(ShellError::message(
                    "navigation transition evidence requires --source-control and --target-view",
                ));
            }
            let trigger = parse_transition_trigger_name(trigger)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_evidence = parse_workflow_transition_source_evidence_text(source_evidence)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target_evidence = parse_workflow_transition_target_evidence_text(target_evidence)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowTransitionEvidence {
                    workflow_slug,
                    evidence: WorkflowTransitionEvidenceRecord::new(
                        source,
                        target,
                        kind,
                        trigger,
                        source_evidence,
                        target_evidence,
                    ),
                },
            })
        }
        _ => parse_cli_44(arguments),
    }
}

fn parse_cli_44(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            source_flag,
            source,
            target_flag,
            target,
            kind_flag,
            kind,
            trigger_flag,
            trigger,
            source_control_flag,
            source_control,
            target_view_flag,
            target_view,
            source_evidence_flag,
            source_evidence,
            target_evidence_flag,
            target_evidence,
        ] if command == "add"
            && subject == "workflow-transition-evidence"
            && workflow_flag == "--workflow"
            && source_flag == "--from"
            && target_flag == "--to"
            && kind_flag == "--via"
            && trigger_flag == "--name"
            && source_control_flag == "--source-control"
            && target_view_flag == "--target-view"
            && source_evidence_flag == "--source-evidence"
            && target_evidence_flag == "--target-evidence" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source = parse_workflow_transition_endpoint(source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target = parse_workflow_transition_endpoint(target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let kind = parse_workflow_transition_kind(kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(trigger)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_control = parse_transition_trigger_name(source_control)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target_view = parse_workflow_owned_definition_name(target_view)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_evidence = parse_workflow_transition_source_evidence_text(source_evidence)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target_evidence = parse_workflow_transition_target_evidence_text(target_evidence)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowTransitionEvidence {
                    workflow_slug,
                    evidence: WorkflowTransitionEvidenceRecord::new_with_navigation_endpoints(
                        source,
                        target,
                        kind,
                        trigger,
                        WorkflowTransitionEvidenceNavigationEndpoints::new(
                            source_control,
                            target_view,
                        ),
                        source_evidence,
                        target_evidence,
                    ),
                },
            })
        }
        _ => parse_cli_45(arguments),
    }
}

fn parse_cli_45(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            state_flag,
            state,
            step_flag,
            step,
            evidence_flag,
            evidence,
        ] if command == "add"
            && subject == "workflow-entry-lifecycle-state"
            && workflow_flag == "--workflow"
            && state_flag == "--state"
            && step_flag == "--step"
            && evidence_flag == "--evidence" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let state = parse_workflow_entry_lifecycle_state_name(state)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let step = parse_workflow_transition_endpoint(step)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let evidence = parse_workflow_entry_lifecycle_evidence_text(evidence)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowEntryLifecycleState {
                    workflow_slug,
                    coverage: WorkflowEntryLifecycleStateRecord::new(state, step, evidence),
                },
            })
        }
        [command] if command == "check" => Ok(Cli {
            command: Command::Check,
        }),
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
        ] if command == "connect"
            && subject == "workflow"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug =
                parse_slice_slug(target).map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            if connection_kind == ConnectionKind::Navigation {
                return Err(ShellError::message(
                    "navigation workflow transitions require --source-control and --target-view",
                ));
            }
            Ok(Cli {
                command: Command::ConnectWorkflow {
                    connection: WorkflowConnection::new(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                    ),
                },
            })
        }
        _ => parse_cli_46(arguments),
    }
}

fn parse_cli_46(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            source_control_flag,
            source_control,
            target_view_flag,
            target_view,
        ] if command == "connect"
            && subject == "workflow"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name"
            && source_control_flag == "--source-control"
            && target_view_flag == "--target-view" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug =
                parse_slice_slug(target).map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_control = parse_transition_trigger_name(source_control)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target_view = parse_workflow_owned_definition_name(target_view)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::ConnectWorkflow {
                    connection: WorkflowConnection::new_with_navigation_endpoints(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                        source_control,
                        target_view,
                    ),
                },
            })
        }
        _ => parse_cli_47(arguments),
    }
}

fn parse_cli_47(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_workflow_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            reason_flag,
            reason,
        ] if command == "connect"
            && subject == "workflow"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_workflow_flag == "--to-workflow"
            && via_flag == "--via"
            && name_flag == "--name"
            && reason_flag == "--reason" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug = parse_workflow_slug(target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let exit_reason = parse_model_description(reason)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::ConnectWorkflow {
                    connection: WorkflowConnection::new_workflow_exit(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                        exit_reason,
                    ),
                },
            })
        }
        _ => parse_cli_48(arguments),
    }
}

#[derive(Clone, Copy)]
struct TransitionIdentityCli<'a> {
    workflow: &'a str,
    source: &'a str,
    target: &'a str,
    via: &'a str,
    name: &'a str,
}

impl<'a> TransitionIdentityCli<'a> {
    fn new(
        workflow: &'a str,
        source: &'a str,
        target: &'a str,
        via: &'a str,
        name: &'a str,
    ) -> Self {
        Self {
            workflow,
            source,
            target,
            via,
            name,
        }
    }
}

#[derive(Clone, Copy)]
struct TransitionReplacementCli<'a> {
    source: &'a str,
    target: &'a str,
    via: &'a str,
    name: &'a str,
}

impl<'a> TransitionReplacementCli<'a> {
    fn new(source: &'a str, target: &'a str, via: &'a str, name: &'a str) -> Self {
        Self {
            source,
            target,
            via,
            name,
        }
    }
}

fn parse_slice_transition_update(
    previous: TransitionIdentityCli<'_>,
    replacement: TransitionReplacementCli<'_>,
    payload_contract: Option<PayloadContractName>,
) -> Result<WorkflowTransitionUpdate, ShellError> {
    let workflow_slug = parse_workflow_slug(previous.workflow)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_slug = parse_slice_slug(previous.source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let target_slug = parse_slice_slug(previous.target)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let connection_kind = parse_connection_kind(previous.via)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let trigger = parse_transition_trigger_name(previous.name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_source_slug = parse_slice_slug(replacement.source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_target_slug = parse_slice_slug(replacement.target)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_connection_kind = parse_connection_kind(replacement.via)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_trigger = parse_transition_trigger_name(replacement.name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let previous = WorkflowTransitionRemoval::new(
        workflow_slug.clone(),
        source_slug,
        target_slug,
        connection_kind,
        trigger,
    );
    let replacement = match payload_contract {
        Some(payload_contract) => WorkflowConnection::new_with_payload_contract(
            workflow_slug,
            new_source_slug,
            new_target_slug,
            new_connection_kind,
            new_trigger,
            payload_contract,
        ),
        None => WorkflowConnection::new(
            workflow_slug,
            new_source_slug,
            new_target_slug,
            new_connection_kind,
            new_trigger,
        ),
    };
    Ok(WorkflowTransitionUpdate::new(previous, replacement))
}

fn parse_navigation_transition_update(
    previous: TransitionIdentityCli<'_>,
    replacement: TransitionReplacementCli<'_>,
    source_control: &str,
    target_view: &str,
) -> Result<WorkflowTransitionUpdate, ShellError> {
    let workflow_slug = parse_workflow_slug(previous.workflow)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_slug = parse_slice_slug(previous.source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let target_slug = parse_slice_slug(previous.target)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let connection_kind = parse_connection_kind(previous.via)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let trigger = parse_transition_trigger_name(previous.name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_source_slug = parse_slice_slug(replacement.source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_target_slug = parse_slice_slug(replacement.target)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_connection_kind = parse_connection_kind(replacement.via)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_trigger = parse_transition_trigger_name(replacement.name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_control = parse_transition_trigger_name(source_control)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let target_view = parse_workflow_owned_definition_name(target_view)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let previous = WorkflowTransitionRemoval::new(
        workflow_slug.clone(),
        source_slug,
        target_slug,
        connection_kind,
        trigger,
    );
    let replacement = WorkflowConnection::new_with_navigation_endpoints(
        workflow_slug,
        new_source_slug,
        new_target_slug,
        new_connection_kind,
        new_trigger,
        source_control,
        target_view,
    );
    Ok(WorkflowTransitionUpdate::new(previous, replacement))
}

fn parse_workflow_exit_transition_update(
    previous: TransitionIdentityCli<'_>,
    replacement: TransitionReplacementCli<'_>,
    reason: &str,
) -> Result<WorkflowTransitionUpdate, ShellError> {
    let workflow_slug = parse_workflow_slug(previous.workflow)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_slug = parse_slice_slug(previous.source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let target_slug = parse_workflow_slug(previous.target)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let connection_kind = parse_connection_kind(previous.via)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let trigger = parse_transition_trigger_name(previous.name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_source_slug = parse_slice_slug(replacement.source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_target_slug = parse_workflow_slug(replacement.target)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_connection_kind = parse_connection_kind(replacement.via)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let new_trigger = parse_transition_trigger_name(replacement.name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let reason =
        parse_model_description(reason).map_err(|error| ShellError::message(error.to_string()))?;
    let previous = WorkflowTransitionRemoval::new_workflow_exit(
        workflow_slug.clone(),
        source_slug,
        target_slug,
        connection_kind,
        trigger,
    );
    let replacement = WorkflowConnection::new_workflow_exit(
        workflow_slug,
        new_source_slug,
        new_target_slug,
        new_connection_kind,
        new_trigger,
        reason,
    );
    Ok(WorkflowTransitionUpdate::new(previous, replacement))
}

fn parse_cli_48(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            new_from_flag,
            new_source,
            new_to_flag,
            new_target,
            new_via_flag,
            new_via,
            new_name_flag,
            new_name,
        ] if command == "update"
            && subject == "transition"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name"
            && new_from_flag == "--new-from"
            && new_to_flag == "--new-to"
            && new_via_flag == "--new-via"
            && new_name_flag == "--new-name" =>
        {
            let update = parse_slice_transition_update(
                TransitionIdentityCli::new(workflow, source, target, via, name),
                TransitionReplacementCli::new(new_source, new_target, new_via, new_name),
                None,
            )?;
            Ok(Cli {
                command: Command::UpdateTransition { update },
            })
        }
        _ => parse_cli_transition_update_with_payload(arguments),
    }
}

fn parse_cli_transition_update_with_payload(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            new_from_flag,
            new_source,
            new_to_flag,
            new_target,
            new_via_flag,
            new_via,
            new_name_flag,
            new_name,
            payload_contract_flag,
            payload_contract,
        ] if command == "update"
            && subject == "transition"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name"
            && new_from_flag == "--new-from"
            && new_to_flag == "--new-to"
            && new_via_flag == "--new-via"
            && new_name_flag == "--new-name"
            && payload_contract_flag == "--new-payload-contract" =>
        {
            let payload_contract = parse_payload_contract_name(payload_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let update = parse_slice_transition_update(
                TransitionIdentityCli::new(workflow, source, target, via, name),
                TransitionReplacementCli::new(new_source, new_target, new_via, new_name),
                Some(payload_contract),
            )?;
            Ok(Cli {
                command: Command::UpdateTransition { update },
            })
        }
        _ => parse_cli_transition_update_navigation(arguments),
    }
}

fn parse_cli_transition_update_navigation(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            new_from_flag,
            new_source,
            new_to_flag,
            new_target,
            new_via_flag,
            new_via,
            new_name_flag,
            new_name,
            source_control_flag,
            source_control,
            target_view_flag,
            target_view,
        ] if command == "update"
            && subject == "transition"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name"
            && new_from_flag == "--new-from"
            && new_to_flag == "--new-to"
            && new_via_flag == "--new-via"
            && new_name_flag == "--new-name"
            && source_control_flag == "--new-source-control"
            && target_view_flag == "--new-target-view" =>
        {
            let update = parse_navigation_transition_update(
                TransitionIdentityCli::new(workflow, source, target, via, name),
                TransitionReplacementCli::new(new_source, new_target, new_via, new_name),
                source_control,
                target_view,
            )?;
            Ok(Cli {
                command: Command::UpdateTransition { update },
            })
        }
        _ => parse_cli_transition_update_workflow_exit(arguments),
    }
}

fn parse_cli_transition_update_workflow_exit(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_workflow_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            new_from_flag,
            new_source,
            new_to_workflow_flag,
            new_target,
            new_via_flag,
            new_via,
            new_name_flag,
            new_name,
            reason_flag,
            reason,
        ] if command == "update"
            && subject == "transition"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_workflow_flag == "--to-workflow"
            && via_flag == "--via"
            && name_flag == "--name"
            && new_from_flag == "--new-from"
            && new_to_workflow_flag == "--new-to-workflow"
            && new_via_flag == "--new-via"
            && new_name_flag == "--new-name"
            && reason_flag == "--new-reason" =>
        {
            let update = parse_workflow_exit_transition_update(
                TransitionIdentityCli::new(workflow, source, target, via, name),
                TransitionReplacementCli::new(new_source, new_target, new_via, new_name),
                reason,
            )?;
            Ok(Cli {
                command: Command::UpdateTransition { update },
            })
        }
        _ => parse_cli_transition_removal(arguments),
    }
}

fn parse_cli_transition_removal(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
        ] if command == "remove"
            && subject == "transition"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug =
                parse_slice_slug(target).map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RemoveTransition {
                    removal: WorkflowTransitionRemoval::new(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                    ),
                },
            })
        }
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_workflow_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
        ] if command == "remove"
            && subject == "transition"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_workflow_flag == "--to-workflow"
            && via_flag == "--via"
            && name_flag == "--name" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug = parse_workflow_slug(target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RemoveTransition {
                    removal: WorkflowTransitionRemoval::new_workflow_exit(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                    ),
                },
            })
        }
        _ => parse_cli_49(arguments),
    }
}

fn parse_cli_49(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [command, subject, suite_flag, suite]
            if command == "gherkin" && subject == "list" && suite_flag == "--suite" =>
        {
            parse_gherkin_suite(suite)
                .map(|suite| Cli {
                    command: Command::GherkinList { suite },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, suite_flag, suite]
            if command == "gherkin" && subject == "run" && suite_flag == "--suite" =>
        {
            parse_gherkin_suite(suite)
                .map(|suite| Cli {
                    command: Command::GherkinRun { suite },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, all_flag]
            if command == "gherkin" && subject == "run" && all_flag == "--all" =>
        {
            Ok(Cli {
                command: Command::GherkinRunAll,
            })
        }
        [command, name_flag, name] if command == "init" && name_flag == "--name" => Ok(Cli {
            command: Command::Init {
                name: parse_project_name(name)
                    .map_err(|error| ShellError::message(error.to_string()))?,
            },
        }),
        [command, subject, workflow_flag, workflow]
            if command == "mark"
                && subject == "workflow-entry-lifecycle-required"
                && workflow_flag == "--workflow" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RequireWorkflowEntryLifecycleCoverage { workflow_slug },
            })
        }
        [command, subject] if command == "list" && subject == "workflows" => Ok(Cli {
            command: Command::ListWorkflows,
        }),
        [command, subject] if command == "list" && subject == "conflicts" => Ok(Cli {
            command: Command::ListConflicts,
        }),
        [command, subject] if command == "list" && subject == "slices" => Ok(Cli {
            command: Command::ListSlices,
        }),
        [command, subject] if command == "list" && subject == "transitions" => Ok(Cli {
            command: Command::ListTransitions,
        }),
        [command, subject, slug_flag, slug]
            if command == "remove" && subject == "slice" && slug_flag == "--slug" =>
        {
            parse_slice_slug(slug)
                .map(|slug| Cli {
                    command: Command::RemoveSlice { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [
            command,
            subject,
            id_flag,
            conflict_id,
            choose_event_flag,
            chosen_event_id,
        ] if command == "resolve"
            && subject == "conflict"
            && id_flag == "--id"
            && choose_event_flag == "--choose-event" =>
        {
            Ok(Cli {
                command: Command::ResolveConflict {
                    conflict_id: parse_event_conflict_id(conflict_id)?,
                    chosen_event_id: parse_chosen_event_id(chosen_event_id)?,
                },
            })
        }
        _ => parse_cli_50(arguments),
    }
}

fn parse_cli_50(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [command, subject, slug_flag, slug]
            if command == "remove" && subject == "workflow" && slug_flag == "--slug" =>
        {
            parse_workflow_slug(slug)
                .map(|slug| Cli {
                    command: Command::RemoveWorkflow { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, transport] if command == "mcp" && transport == "stdio" => Ok(Cli {
            command: Command::McpStdio,
        }),
        [command, transport] if command == "mcp" && transport == "http" => Ok(Cli {
            command: Command::McpHttp {
                host: "127.0.0.1".to_owned(),
                port: 7331,
                once: false,
                auth_token: None,
            },
        }),
        [
            command,
            transport,
            host_flag,
            host,
            port_flag,
            port,
            once_flag,
        ] if command == "mcp"
            && transport == "http"
            && host_flag == "--host"
            && port_flag == "--port"
            && once_flag == "--once" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: true,
                    auth_token: None,
                },
            })
        }
        [
            command,
            transport,
            host_flag,
            host,
            port_flag,
            port,
            auth_flag,
            auth_token,
            once_flag,
        ] if command == "mcp"
            && transport == "http"
            && host_flag == "--host"
            && port_flag == "--port"
            && auth_flag == "--auth-token"
            && once_flag == "--once" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: true,
                    auth_token: Some(auth_token.clone()),
                },
            })
        }
        [command, transport, host_flag, host, port_flag, port]
            if command == "mcp"
                && transport == "http"
                && host_flag == "--host"
                && port_flag == "--port" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: false,
                    auth_token: None,
                },
            })
        }
        _ => parse_cli_51(arguments),
    }
}

fn parse_cli_51(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            transport,
            host_flag,
            host,
            port_flag,
            port,
            auth_flag,
            auth_token,
        ] if command == "mcp"
            && transport == "http"
            && host_flag == "--host"
            && port_flag == "--port"
            && auth_flag == "--auth-token" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: false,
                    auth_token: Some(auth_token.clone()),
                },
            })
        }
        [command, subject, workflow_flag, workflow]
            if command == "review" && subject == "gate" && workflow_flag == "--workflow" =>
        {
            parse_workflow_slug(workflow)
                .map(|slug| Cli {
                    command: Command::ReviewGate { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [
            command,
            subject,
            workflow_flag,
            workflow,
            reviewer_flag,
            reviewer,
            reviewed_at_flag,
            reviewed_at,
        ] if command == "review"
            && subject == "record"
            && workflow_flag == "--workflow"
            && reviewer_flag == "--reviewer"
            && reviewed_at_flag == "--reviewed-at" =>
        {
            let slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let reviewer = parse_reviewer_id(reviewer)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let reviewed_at = parse_review_timestamp(reviewed_at)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RecordCleanReview {
                    slug,
                    reviewer,
                    reviewed_at,
                },
            })
        }
        [command, subject, slug] if command == "show" && subject == "workflow" => {
            parse_workflow_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowWorkflow { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, slug_flag, slug]
            if command == "show" && subject == "workflow" && slug_flag == "--slug" =>
        {
            parse_workflow_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowWorkflow { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, slug] if command == "show" && subject == "slice" => {
            parse_slice_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowSlice { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        _ => parse_cli_52(arguments),
    }
}

fn parse_cli_52(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [command, subject, slug_flag, slug]
            if command == "show" && subject == "slice" && slug_flag == "--slug" =>
        {
            parse_slice_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowSlice { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [
            command,
            subject,
            slug_flag,
            slug,
            description_flag,
            description,
        ] if command == "update"
            && subject == "slice"
            && slug_flag == "--slug"
            && description_flag == "--description" =>
        {
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateSliceDescription {
                    slug: slice_slug,
                    description: slice_description,
                },
            })
        }
        [command, subject, slug_flag, slug, type_flag, slice_type]
            if command == "update"
                && subject == "slice"
                && slug_flag == "--slug"
                && type_flag == "--type" =>
        {
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_kind = parse_slice_kind(slice_type)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateSliceKind {
                    slug: slice_slug,
                    kind: slice_kind,
                },
            })
        }
        [command, subject, slug_flag, slug, name_flag, name]
            if command == "update"
                && subject == "slice"
                && slug_flag == "--slug"
                && name_flag == "--name" =>
        {
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateSliceName {
                    slug: slice_slug,
                    name: slice_name,
                },
            })
        }
        _ => parse_cli_53(arguments),
    }
}

fn parse_cli_53(arguments: &[String]) -> Result<Cli, ShellError> {
    match arguments {
        [
            command,
            subject,
            slug_flag,
            slug,
            description_flag,
            description,
        ] if command == "update"
            && subject == "workflow"
            && slug_flag == "--slug"
            && description_flag == "--description" =>
        {
            let workflow_slug = parse_workflow_slug(slug)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateWorkflowDescription {
                    slug: workflow_slug,
                    description: workflow_description,
                },
            })
        }
        [command, subject, slug_flag, slug, name_flag, name]
            if command == "update"
                && subject == "workflow"
                && slug_flag == "--slug"
                && name_flag == "--name" =>
        {
            let workflow_slug = parse_workflow_slug(slug)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateWorkflowName {
                    slug: workflow_slug,
                    name: workflow_name,
                },
            })
        }
        [command] if command == "verify" => Ok(Cli {
            command: Command::Verify,
        }),
        [command, ..] if command == "init" => {
            Err(ShellError::message("usage: emc init --name <project-name>"))
        }
        _ => Err(ShellError::message(
            "usage: emc <command> [arguments]; run emc --help",
        )),
    }
}

fn parse_observed_flag(raw: &str) -> Result<bool, ShellError> {
    parse_bool_flag(raw)
        .map_err(|_error| ShellError::message("invalid observed flag: expected true or false"))
}

fn command_input_source_kind_mismatch(
    actual: CommandInputSourceKind,
    expected: CommandInputSourceKind,
) -> ShellError {
    ShellError::message(format!(
        "add command source reference requires --input-source {expected}, got {actual}"
    ))
}

fn require_actor_command_input_source(
    source_kind: CommandInputSourceKind,
) -> Result<CommandInputSource, ShellError> {
    match source_kind {
        CommandInputSourceKind::Actor => Ok(CommandInputSource::actor()),
        other => Err(command_input_source_kind_mismatch(
            other,
            CommandInputSourceKind::Actor,
        )),
    }
}

fn read_model_field_source_kind_mismatch(
    actual: ReadModelFieldSourceKind,
    expected: ReadModelFieldSourceKind,
) -> ShellError {
    ShellError::message(format!(
        "add read-model source reference requires --field-source {expected}, got {actual}"
    ))
}

fn parse_event_attribute(
    attribute: &str,
    attribute_source: &str,
    attribute_source_name: &str,
    attribute_source_field: &str,
    generated_source_kind: Option<&str>,
    attribute_provenance: &str,
) -> Result<NewEventAttribute, ShellError> {
    let attribute_name = parse_event_attribute_name(attribute)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let attribute_source_kind = parse_event_attribute_source_kind(attribute_source)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let attribute_source_name = parse_event_attribute_source_name(attribute_source_name)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let attribute_source_field = parse_event_attribute_source_field(attribute_source_field)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let provenance_description = parse_provenance_description(attribute_provenance)
        .map_err(|error| ShellError::message(error.to_string()))?;

    match generated_source_kind {
        Some(generated_source_kind) => {
            let generated_source_kind =
                parse_generated_event_attribute_source_kind(generated_source_kind)
                    .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(NewEventAttribute::new_with_generated_source_kind(
                attribute_name,
                attribute_source_kind,
                attribute_source_name,
                attribute_source_field,
                generated_source_kind,
                provenance_description,
            ))
        }
        None => Ok(NewEventAttribute::new(
            attribute_name,
            attribute_source_kind,
            attribute_source_name,
            attribute_source_field,
            provenance_description,
        )),
    }
}

fn parse_shared_flag(raw: &str) -> Result<bool, ShellError> {
    parse_bool_flag(raw)
        .map_err(|_error| ShellError::message("invalid shared flag: expected true or false"))
}

fn parse_bool_flag(raw: &str) -> Result<bool, ShellError> {
    match raw {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ShellError::message("expected true or false")),
    }
}

fn print_help() -> Result<(), ShellError> {
    help_command()
        .print_help()
        .map_err(|error| ShellError::message(error.to_string()))?;
    println!();
    Ok(())
}

fn print_enum_help() {
    for modeled_enum in MODELING_ENUMS {
        println!(
            "{}: {}",
            modeled_enum.name(),
            modeled_enum.values().join(", ")
        );
    }
}

fn print_modeling_help() {
    println!("{}", modeling_process_guide());
}

fn parse_basic_scenario(
    slice_slug: SliceSlug,
    scenario_kind: &str,
    name: &str,
    given: &str,
    when: &str,
    then: &str,
) -> Result<NewSliceScenario, ShellError> {
    let name = parse_scenario_name(name).map_err(|error| ShellError::message(error.to_string()))?;
    let given =
        parse_scenario_step_text(given).map_err(|error| ShellError::message(error.to_string()))?;
    let when =
        parse_scenario_step_text(when).map_err(|error| ShellError::message(error.to_string()))?;
    let then =
        parse_scenario_step_text(then).map_err(|error| ShellError::message(error.to_string()))?;
    match parse_scenario_kind(scenario_kind)
        .map_err(|error| ShellError::message(error.to_string()))?
    {
        ScenarioKind::Acceptance => Ok(NewSliceScenario::new(
            slice_slug,
            ScenarioKind::acceptance(),
            name,
            given,
            when,
            then,
        )),
        ScenarioKind::Contract => Err(ShellError::message(
            "contract scenarios require --contract-kind and --covered-definition",
        )),
    }
}

fn parse_scenario_streams(
    read_streams: &str,
    written_streams: &str,
) -> Result<(ScenarioStreamNames, ScenarioStreamNames), ShellError> {
    let read_streams =
        parse_stream_names(read_streams).map_err(|error| ShellError::message(error.to_string()))?;
    let written_streams = parse_stream_names(written_streams)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok((
        ScenarioStreamNames::from_streams(read_streams),
        ScenarioStreamNames::from_streams(written_streams),
    ))
}

fn help_command() -> ClapCommand {
    let command = ClapCommand::new("emc")
        .about("Event Model Compiler")
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .subcommand(help_init_subcommand())
        .subcommand(help_help_subcommand())
        .subcommand(help_list_subcommand())
        .subcommand(help_show_subcommand())
        .subcommand(help_add_subcommand())
        .subcommand(help_update_subcommand())
        .subcommand(help_remove_subcommand())
        .subcommand(help_connect_subcommand())
        .subcommand(ClapCommand::new("verify").about("Run Lean4 and Quint verification"))
        .subcommand(ClapCommand::new("check").about("Check project artifact synchronization"))
        .subcommand(help_gherkin_subcommand())
        .subcommand(help_review_subcommand())
        .subcommand(help_mcp_subcommand());
    command.after_help(help_after_text())
}

fn help_init_subcommand() -> ClapCommand {
    ClapCommand::new("init")
        .about("Create a deterministic EMC project")
        .arg(Arg::new("name").long("name").value_name("PROJECT_NAME"))
}

fn help_help_subcommand() -> ClapCommand {
    ClapCommand::new("help")
        .about("Show modeling reference material")
        .subcommand(ClapCommand::new("enums").about("List accepted modeled enum values"))
        .subcommand(ClapCommand::new("modeling").about("Show the EMC modeling process guide"))
}

fn help_list_subcommand() -> ClapCommand {
    ClapCommand::new("list")
        .about("Read model indexes")
        .subcommand(ClapCommand::new("workflows").about("List modeled workflows in the project"))
        .subcommand(ClapCommand::new("slices").about("List modeled slices in the project"))
        .subcommand(
            ClapCommand::new("transitions")
                .about("List modeled workflow transitions in the project"),
        )
}

fn help_show_subcommand() -> ClapCommand {
    ClapCommand::new("show")
        .about("Read modeled artifacts")
        .subcommand(ClapCommand::new("workflow").about("Show a workflow by slug"))
        .subcommand(ClapCommand::new("slice").about("Show a slice by slug"))
}

fn help_add_subcommand() -> ClapCommand {
    ClapCommand::new("add")
        .about("Create modeled business artifacts")
        .subcommand(
            ClapCommand::new("workflow").about("Add a workflow and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("workflow-outcome")
                .about("Add a workflow composition outcome fact to formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("workflow-command-error")
                .about("Add a workflow composition command-error fact to formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("workflow-owned-definition")
                .about("Add a workflow composition ownership fact to formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("workflow-transition-evidence")
                .about("Add workflow transition legality evidence to formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("slice").about("Add a slice and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("scenario")
                .about("Add an acceptance or contract scenario to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("command").about("Add a command definition to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("event").about("Add an event definition to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("outcome")
                .about("Add an outcome definition to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("read-model")
                .about("Add a read-model projection to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("view").about("Add a view field projection to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("automation")
                .about("Add an automation reaction to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("translation")
                .about("Add a translation boundary to formal slice artifacts"),
        )
        .subcommand(
            ClapCommand::new("data-flow")
                .about("Add a bit-level data-flow fact to formal slice artifacts"),
        )
}

fn help_update_subcommand() -> ClapCommand {
    ClapCommand::new("update")
        .about("Modify modeled business artifacts")
        .subcommand(
            ClapCommand::new("workflow")
                .about("Update a workflow and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("slice").about("Update a slice and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("scenario")
                .about("Update an acceptance scenario and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("command")
                .about("Update a command definition and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("event")
                .about("Update an event definition and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("translation")
                .about("Update a translation definition and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("external-payload")
                .about("Update an external payload definition and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("data-flow")
                .about("Update a bit-level data-flow fact in synchronized formal artifacts"),
        )
}

fn help_remove_subcommand() -> ClapCommand {
    ClapCommand::new("remove")
        .about("Delete modeled business artifacts")
        .subcommand(
            ClapCommand::new("workflow")
                .about("Remove a workflow and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("slice").about("Remove a slice and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("scenario")
                .about("Remove a scenario and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("command")
                .about("Remove a command definition and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("event")
                .about("Remove an event definition and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("translation")
                .about("Remove a translation definition and synchronized formal artifacts"),
        )
        .subcommand(
            ClapCommand::new("external-payload")
                .about("Remove an external payload definition and synchronized formal artifacts"),
        )
}

fn help_connect_subcommand() -> ClapCommand {
    ClapCommand::new("connect")
        .about("Connect modeled workflow steps")
        .subcommand(
            ClapCommand::new("workflow")
                .about("Add a workflow transition and synchronized formal artifacts"),
        )
}

fn help_gherkin_subcommand() -> ClapCommand {
    ClapCommand::new("gherkin")
        .about("List or run checked-in event-model rule suites")
        .subcommand(ClapCommand::new("list").about("List configured feature files"))
        .subcommand(ClapCommand::new("run").about("Run configured rule-suite coverage"))
}

fn help_review_subcommand() -> ClapCommand {
    ClapCommand::new("review")
        .about("Evaluate review gates")
        .subcommand(ClapCommand::new("gate").about("Check a workflow review gate"))
        .subcommand(ClapCommand::new("record").about("Record a clean workflow review"))
}

fn help_mcp_subcommand() -> ClapCommand {
    ClapCommand::new("mcp")
        .about("Serve EMC tools over MCP")
        .subcommand(ClapCommand::new("stdio").about("Serve MCP over stdio"))
        .subcommand(ClapCommand::new("http").about("Serve MCP over HTTP"))
}

fn help_after_text() -> &'static str {
    "Common commands:
  emc init --name <project-name>
  emc help enums
  emc help modeling
  emc add workflow --slug <slug> --name <name> --description <text>
  emc update workflow --slug <workflow> --name <name>
  emc remove workflow --slug <workflow>
  emc add slice --workflow <workflow> --slug <slug> --name <name> --type <kind> --description <text>
  emc add scenario --slice <slice> --kind acceptance --name <name> --given <text> --when <text> --then <text>
  emc add scenario --slice <slice> --kind acceptance --name <name> --given <text> --when <text> --then <text> --read-streams <stream[,stream]> --written-streams <stream[,stream]>
  emc add scenario --slice <slice> --kind contract --name <name> --given <text> --when <text> --then <text> --contract-kind <kind> --covered-definition <name>
  emc add scenario --slice <slice> --kind contract --name <name> --given <text> --when <text> --then <text> --contract-kind <kind> --covered-definition <name> --read-streams <stream[,stream]> --written-streams <stream[,stream]>
  emc add scenario --slice <slice> --kind contract --name <name> --given <text> --when <text> --then <text> --contract-kind command --covered-definition <command> --error-references <error[,error]>
  emc add scenario --slice <slice> --kind contract --name <name> --given <text> --when <text> --then <text> --contract-kind command --covered-definition <command> --read-streams <stream[,stream]> --written-streams <stream[,stream]> --error-references <error[,error]>
  emc add workflow-outcome --workflow <workflow> --source-slice <slice> --label <label> --externally-relevant <true|false>
  emc add workflow-command-error --workflow <workflow> --source-slice <slice> --command <command> --error <error>
  emc add workflow-owned-definition --workflow <workflow> --source-slice <slice> --definition-kind <kind> --definition-name <name> [--definition-stream <stream> --source-provenance <text> [--event-participation <role>]]
  emc add workflow-owned-definition --workflow <workflow> --source-slice <slice> --definition-kind view --definition-name <name> --view-role <role>
  emc add workflow-transition-evidence --workflow <workflow> --from <step> --to <step> --via <kind> --name <trigger> --source-evidence <text> --target-evidence <text>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --observes <stream[,stream]>
  emc add command --slice <slice> --name <name> --input <datum> --input-source event_stream_state --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --observes <stream[,stream]> --source-event <event> --source-attribute <attribute>
  emc add command --slice <slice> --name <name> --input <datum> --input-source external_payload --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --source-payload <payload> --source-field <field>
  emc add command --slice <slice> --name <name> --input <datum> --input-source generated --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --source-name <source> --source-field <field>
  emc add command --slice <slice> --name <name> --input <datum> --input-source session --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --source-session <session> --source-field <field>
  emc add command --slice <slice> --name <name> --input <datum> --input-source invocation_argument --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --source-argument <argument> --source-field <field>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --singleton <true|false> --repeat-behavior <already_exists_error|idempotent>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --error <name> --error-scenario <scenario> --error-recovery <kind>
  emc update command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]>
  emc add external-payload --slice <slice> --name <name> --field <field> --field-provenance <text> --bit-encoding <semantics>
  emc add event --slice <slice> --name <event> --stream <stream> --attribute <name> --attribute-source <kind> --attribute-source-name <name> --attribute-source-field <field> [--generated-source-kind <kind>] --attribute-provenance <text> [--observed true]
  emc add event --slice <slice> --name <event> --stream <stream> --attribute <name> --attribute-source <kind> --attribute-source-name <name> --attribute-source-field <field> [--generated-source-kind <kind>] --attribute-provenance <text> --shared <true|false>
  emc update event --slice <slice> --name <event> --stream <stream> --attribute <name> --attribute-source <kind> --attribute-source-name <name> --attribute-source-field <field> [--generated-source-kind <kind>] --attribute-provenance <text> [--observed true]
  emc add outcome --slice <slice> --label <label> --events <event[,event]> --externally-relevant <true|false>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --source-event <event> --source-attribute <attribute> --field-provenance <text>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --source-event <event> --source-attribute <attribute> --field-provenance <text> --transitive <true|false> --relationship-fields <field[,field]> --transitive-rule <rule> --example-scenario <scenario>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --derivation-rule <rule> --derivation-scenario <scenario> --field-provenance <text>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --absence-event <event> --absence-scenario <scenario> --field-provenance <text>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics> --control <control> --control-command <command> --control-input <datum> --control-input-source <kind> --control-input-description <text> --control-input-sketch-token <token> --control-input-visible <true|false> --control-input-decision <true|false> --handled-errors <error[,error]> --recovery-behavior <kind> --control-sketch-token <token> --navigation-type <type> --navigation-target <target>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics> --control <control> --control-command <command> --control-input <datum> --control-input-source <kind> --control-input-description <text> --control-input-sketch-token <token> --control-input-visible <true|false> --control-input-decision <true|false> --handled-errors <error[,error]> --recovery-behavior <kind> --control-sketch-token <token> --navigation-type local_view_state --navigation-target <target> --local-states <state[,state]> --filters <filter[,filter]>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics> --control <control> --control-command <command> --control-input <datum> --control-input-source <kind> --control-input-description <text> --control-input-sketch-token <token> --control-input-visible <true|false> --control-input-decision <true|false> --handled-errors <error[,error]> --recovery-behavior <kind> --control-sketch-token <token> --navigation-type external_workflow --navigation-target <target> --external-workflow <workflow>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics> --control <control> --control-command <command> --control-input <datum> --control-input-source <kind> --control-input-description <text> --control-input-sketch-token <token> --control-input-visible <true|false> --control-input-decision <true|false> --handled-errors <error[,error]> --recovery-behavior <kind> --control-sketch-token <token> --navigation-type external_system --navigation-target <target> --external-system <name> --handoff-contract <contract>
  emc add automation --slice <slice> --name <name> --trigger <event-or-signal> --command <command> --handled-errors <error[,error]> --reaction <semantics>
  emc add translation --slice <slice> --name <name> --external-event <event-or-signal> --payload-contract <payload> --command <command>
  emc add data-flow --slice <slice> --datum <name> --source <source> --source-kind <original|modeled_target> --transformation <semantics> --target <target> --bit-encoding <semantics>
  emc update data-flow --slice <slice> --datum <name> --source <source> --source-kind <original|modeled_target> --transformation <semantics> --target <target> --bit-encoding <semantics> --new-datum <name> --new-source <source> --new-source-kind <original|modeled_target> --new-transformation <semantics> --new-target <target> --new-bit-encoding <semantics>
  emc update slice --slug <slice> --description <text>
  emc update slice --slug <slice> --type <kind>
  emc update slice --slug <slice> --name <name>
  emc update scenario --slice <slice> --kind acceptance --name <name> --given <text> --when <text> --then <text>
  emc remove slice --slug <slice>
  emc remove scenario --slice <slice> --name <name>
  emc remove command --slice <slice> --name <name>
  emc remove event --slice <slice> --name <name>
  emc remove data-flow --slice <slice> --datum <name> --source <source> --source-kind <original|modeled_target> --transformation <semantics> --target <target> --bit-encoding <semantics>
  emc connect workflow --workflow <workflow> --from <slice> --to <slice> --via <kind> --name <trigger> [--payload-contract <contract>]
  emc remove transition --workflow <workflow> --from <slice> --to <slice> --via <kind> --name <trigger>
  emc remove transition --workflow <workflow> --from <slice> --to-workflow <workflow> --via outcome --name <trigger>
  emc list workflows
  emc list slices
  emc list transitions
  emc list conflicts
  emc show workflow <workflow>
  emc show workflow --slug <workflow>
  emc show slice <slice>
  emc show slice --slug <slice>
  emc resolve conflict --id <conflict-id> --choose-event <event-id>
  emc verify
  emc check
  emc gherkin list --suite <suite>
  emc gherkin run --suite <suite>
  emc gherkin run --all
  emc review gate --workflow <workflow>
  emc review record --workflow <workflow> --reviewer <reviewer> --reviewed-at <timestamp>
  emc mcp stdio
  emc mcp http --host 127.0.0.1 --port 7331
  emc mcp http --host 0.0.0.0 --port 7331 --auth-token <token>"
}

fn parse_port(port: &str) -> Result<u16, ShellError> {
    port.parse::<u16>()
        .map_err(|error| ShellError::message(format!("invalid MCP HTTP port: {error}")))
}
