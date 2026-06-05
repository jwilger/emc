use std::env;
use std::process::ExitCode;

use clap::{Arg, Command as ClapCommand};
use emc::command;
use emc::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use emc::core::formal_slice_facts::{
    CommandErrorDefinitions, CommandErrorNames, CommandInputProvenanceChain,
    CommandObservedStreams, EmittedEventNames, NewAutomationDefinition, NewBitLevelDataFlow,
    NewBoardConnection, NewBoardElement, NewCommandDefinition, NewCommandErrorDefinition,
    NewCommandInput, NewControlDefinition, NewControlInputProvision, NewEventAttribute,
    NewEventDefinition, NewExternalPayloadDefinition, NewNavigationTarget, NewOutcomeDefinition,
    NewReadModelDefinition, NewReadModelField, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition, NewViewField, OutcomeEventNames, ReadModelRelationshipFields, ScenarioKind,
    ScenarioStreamNames, ViewControls, ViewFilters, ViewLocalStates,
};
use emc::core::gherkin::GherkinSuite;
use emc::core::project::ProjectName;
use emc::core::slice::{NewSlice, SliceKind};
use emc::core::types::{
    ModelDescription, ModelName, ReviewTimestamp, ReviewerId, SliceSlug,
    WorkflowCommandErrorRecord, WorkflowEntryLifecycleStateRecord, WorkflowOutcomeRecord,
    WorkflowOwnedDefinitionRecord, WorkflowSlug, WorkflowTransitionEndpoint,
    WorkflowTransitionEvidenceRecord,
};
use emc::core::workflow::NewWorkflow;
use emc::io::dto::{
    parse_automation_name, parse_automation_reaction_description, parse_automation_trigger_name,
    parse_bit_encoding_semantics, parse_board_connection_endpoint,
    parse_board_connection_endpoint_kind, parse_board_element_declared_name,
    parse_board_element_kind, parse_board_element_name, parse_board_lane_id,
    parse_command_error_name, parse_command_error_names, parse_command_error_recovery_kind,
    parse_command_input_source_description, parse_command_input_source_kind, parse_command_name,
    parse_connection_kind, parse_contract_kind_name, parse_control_name,
    parse_control_recovery_behavior, parse_covered_definition_name, parse_data_flow_source,
    parse_data_flow_target, parse_datum_name, parse_datum_names, parse_event_attribute_name,
    parse_event_attribute_source_field, parse_event_attribute_source_kind,
    parse_event_attribute_source_name, parse_event_name, parse_event_names, parse_gherkin_suite,
    parse_model_description, parse_model_name, parse_navigation_target_name,
    parse_navigation_target_names, parse_navigation_target_type, parse_outcome_label_name,
    parse_payload_contract_name, parse_project_name, parse_provenance_description,
    parse_read_model_derivation_rule, parse_read_model_field_source_kind, parse_read_model_name,
    parse_read_model_transitive_rule, parse_review_timestamp, parse_reviewer_id,
    parse_scenario_name, parse_scenario_step_text, parse_singleton_repeat_behavior,
    parse_sketch_token, parse_slice_kind, parse_slice_slug, parse_source_chain_hops,
    parse_stream_name, parse_stream_names, parse_transformation_semantics,
    parse_transition_trigger_name, parse_translation_external_event_name, parse_translation_name,
    parse_view_field_name, parse_view_field_source_kind, parse_view_name,
    parse_workflow_entry_lifecycle_evidence_text, parse_workflow_entry_lifecycle_state_name,
    parse_workflow_owned_definition_kind, parse_workflow_owned_definition_name,
    parse_workflow_slug, parse_workflow_transition_endpoint,
    parse_workflow_transition_evidence_text, parse_workflow_transition_kind,
};
use emc::mcp::{serve_http, serve_stdio};
use emc::shell::{ShellError, interpret};

struct Cli {
    command: Command,
}

enum Command {
    AddAutomationDefinition {
        automation: NewAutomationDefinition,
    },
    AddBitLevelDataFlow {
        data_flow: NewBitLevelDataFlow,
    },
    AddBoardConnection {
        connection: NewBoardConnection,
    },
    AddBoardElement {
        element: NewBoardElement,
    },
    AddCommandDefinition {
        command: NewCommandDefinition,
    },
    AddEventDefinition {
        event: NewEventDefinition,
    },
    AddExternalPayloadDefinition {
        external_payload: NewExternalPayloadDefinition,
    },
    AddOutcomeDefinition {
        outcome: NewOutcomeDefinition,
    },
    AddReadModelDefinition {
        read_model: NewReadModelDefinition,
    },
    AddViewDefinition {
        view: NewViewDefinition,
    },
    AddSlice {
        slice: NewSlice,
    },
    AddSliceScenario {
        scenario: NewSliceScenario,
    },
    AddTranslationDefinition {
        translation: NewTranslationDefinition,
    },
    AddWorkflow {
        workflow: NewWorkflow,
    },
    AddWorkflowCommandError {
        workflow_slug: WorkflowSlug,
        error: WorkflowCommandErrorRecord,
    },
    AddWorkflowOwnedDefinition {
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
    Init {
        name: ProjectName,
    },
    ListSlices,
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
    match parse_cli(env::args().skip(1).collect()).and_then(run) {
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
            interpret(command::add_automation_definition(automation))
        }
        Command::AddBitLevelDataFlow { data_flow } => {
            interpret(command::add_bit_level_data_flow(data_flow))
        }
        Command::AddBoardConnection { connection } => {
            interpret(command::add_board_connection(connection))
        }
        Command::AddBoardElement { element } => interpret(command::add_board_element(element)),
        Command::AddCommandDefinition {
            command: definition,
        } => interpret(command::add_command_definition(definition)),
        Command::AddEventDefinition { event } => interpret(command::add_event_definition(event)),
        Command::AddExternalPayloadDefinition { external_payload } => {
            interpret(command::add_external_payload_definition(external_payload))
        }
        Command::AddOutcomeDefinition { outcome } => {
            interpret(command::add_outcome_definition(outcome))
        }
        Command::AddReadModelDefinition { read_model } => {
            interpret(command::add_read_model_definition(read_model))
        }
        Command::AddViewDefinition { view } => interpret(command::add_view_definition(view)),
        Command::AddSlice { slice } => interpret(command::add_slice(slice)),
        Command::AddSliceScenario { scenario } => interpret(command::add_slice_scenario(scenario)),
        Command::AddTranslationDefinition { translation } => {
            interpret(command::add_translation_definition(translation))
        }
        Command::AddWorkflow { workflow } => interpret(command::add_workflow(workflow)),
        Command::AddWorkflowCommandError {
            workflow_slug,
            error,
        } => interpret(command::add_workflow_command_error(workflow_slug, error)),
        Command::AddWorkflowOwnedDefinition {
            workflow_slug,
            definition,
        } => interpret(command::add_workflow_owned_definition(
            workflow_slug,
            definition,
        )),
        Command::AddWorkflowTransitionEvidence {
            workflow_slug,
            evidence,
        } => interpret(command::add_workflow_transition_evidence(
            workflow_slug,
            evidence,
        )),
        Command::AddWorkflowEntryLifecycleState {
            workflow_slug,
            coverage,
        } => interpret(command::add_workflow_entry_lifecycle_state(
            workflow_slug,
            coverage,
        )),
        Command::AddWorkflowOutcome {
            workflow_slug,
            outcome,
        } => interpret(command::add_workflow_outcome(workflow_slug, outcome)),
        Command::Check => interpret(command::check_project()),
        Command::ConnectWorkflow { connection } => interpret(command::connect_workflow(connection)),
        Command::GherkinList { suite } => interpret(command::gherkin_list(suite)),
        Command::GherkinRunAll => interpret(command::gherkin_run_all()),
        Command::GherkinRun { suite } => interpret(command::gherkin_run(suite)),
        Command::Help => print_help(),
        Command::Init { name } => interpret(command::init(name)),
        Command::ListSlices => interpret(command::list_slices()),
        Command::ListTransitions => interpret(command::list_transitions()),
        Command::ListWorkflows => interpret(command::list_workflows()),
        Command::RequireWorkflowEntryLifecycleCoverage { workflow_slug } => interpret(
            command::require_workflow_entry_lifecycle_coverage(workflow_slug),
        ),
        Command::McpHttp {
            host,
            port,
            once,
            auth_token,
        } => serve_http(&host, port, once, auth_token.as_deref()),
        Command::McpStdio => serve_stdio(),
        Command::ReviewGate { slug } => interpret(command::review_gate_for_workflow(slug)),
        Command::RecordCleanReview {
            slug,
            reviewer,
            reviewed_at,
        } => interpret(command::record_clean_review(slug, reviewer, reviewed_at)),
        Command::RemoveSlice { slug } => interpret(command::remove_slice(slug)),
        Command::RemoveTransition { removal } => interpret(command::remove_transition(removal)),
        Command::RemoveWorkflow { slug } => interpret(command::remove_workflow(slug)),
        Command::ShowSlice { slug } => interpret(command::show_slice(slug)),
        Command::ShowWorkflow { slug } => interpret(command::show_workflow(slug)),
        Command::UpdateSliceDescription { slug, description } => {
            interpret(command::update_slice_description(slug, description))
        }
        Command::UpdateSliceKind { slug, kind } => {
            interpret(command::update_slice_kind(slug, kind))
        }
        Command::UpdateSliceName { slug, name } => {
            interpret(command::update_slice_name(slug, name))
        }
        Command::UpdateWorkflowDescription { slug, description } => {
            interpret(command::update_workflow_description(slug, description))
        }
        Command::UpdateWorkflowName { slug, name } => {
            interpret(command::update_workflow_name(slug, name))
        }
        Command::Verify => interpret(command::verify()),
    }
}

fn parse_cli(arguments: Vec<String>) -> Result<Cli, ShellError> {
    match arguments.as_slice() {
        [] => Ok(Cli {
            command: Command::Help,
        }),
        [flag] if flag == "--help" || flag == "-h" => Ok(Cli {
            command: Command::Help,
        }),
        [
            command,
            subject,
            slice_flag,
            slice,
            datum_flag,
            datum,
            source_flag,
            source,
            transformation_flag,
            transformation,
            target_flag,
            target,
            bit_encoding_flag,
            bit_encoding,
        ] if command == "add"
            && subject == "data-flow"
            && slice_flag == "--slice"
            && datum_flag == "--datum"
            && source_flag == "--source"
            && transformation_flag == "--transformation"
            && target_flag == "--target"
            && bit_encoding_flag == "--bit-encoding" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let datum =
                parse_datum_name(datum).map_err(|error| ShellError::message(error.to_string()))?;
            let source = parse_data_flow_source(source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let transformation = parse_transformation_semantics(transformation)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target = parse_data_flow_target(target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let bit_encoding = parse_bit_encoding_semantics(bit_encoding)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddBitLevelDataFlow {
                    data_flow: NewBitLevelDataFlow::new(
                        slice_slug,
                        datum,
                        source,
                        transformation,
                        target,
                        bit_encoding,
                    ),
                },
            })
        }
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
            control_flag,
            control,
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
            external_system_flag,
            external_system,
            handoff_contract_flag,
            handoff_contract,
        ] if command == "add"
            && subject == "view"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && read_model_flag == "--read-model"
            && field_flag == "--field"
            && source_field_flag == "--source-field"
            && sketch_token_flag == "--sketch-token"
            && field_provenance_flag == "--field-provenance"
            && bit_encoding_flag == "--bit-encoding"
            && control_flag == "--control"
            && control_command_flag == "--control-command"
            && control_input_flag == "--control-input"
            && control_input_source_flag == "--control-input-source"
            && control_input_description_flag == "--control-input-description"
            && control_input_sketch_token_flag == "--control-input-sketch-token"
            && control_input_visible_flag == "--control-input-visible"
            && control_input_decision_flag == "--control-input-decision"
            && handled_errors_flag == "--handled-errors"
            && recovery_behavior_flag == "--recovery-behavior"
            && control_sketch_token_flag == "--control-sketch-token"
            && navigation_type_flag == "--navigation-type"
            && navigation_target_flag == "--navigation-target"
            && external_system_flag == "--external-system"
            && handoff_contract_flag == "--handoff-contract" =>
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
            let control_name = parse_control_name(control)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_command = parse_command_name(control_command)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input = parse_datum_name(control_input)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_source = parse_command_input_source_kind(control_input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_description =
                parse_command_input_source_description(control_input_description)
                    .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_sketch_token = parse_sketch_token(control_input_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_visible = parse_bool_flag(control_input_visible)?;
            let control_input_decision = parse_bool_flag(control_input_decision)?;
            let handled_errors = parse_command_error_names(handled_errors)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let recovery_behavior = parse_control_recovery_behavior(recovery_behavior)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_sketch_token = parse_sketch_token(control_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let navigation_type = parse_navigation_target_type(navigation_type)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let navigation_target = parse_navigation_target_name(navigation_target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let external_system = parse_navigation_target_name(external_system)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let handoff_contract = parse_payload_contract_name(handoff_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddViewDefinition {
                    view: NewViewDefinition::new(
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
                    )
                    .with_controls(ViewControls::from_controls([
                        NewControlDefinition::new(
                            control_name,
                            control_command,
                            NewControlInputProvision::new(
                                control_input,
                                control_input_source,
                                control_input_description,
                                control_input_sketch_token,
                                control_input_visible,
                                control_input_decision,
                            ),
                            CommandErrorNames::from_names(handled_errors),
                            recovery_behavior,
                            control_sketch_token,
                            NewNavigationTarget::new(navigation_type, navigation_target)
                                .with_external_system(external_system, handoff_contract),
                        ),
                    ])),
                },
            })
        }
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
            control_flag,
            control,
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
            local_states_flag,
            local_states,
            filters_flag,
            filters,
        ] if command == "add"
            && subject == "view"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && read_model_flag == "--read-model"
            && field_flag == "--field"
            && source_field_flag == "--source-field"
            && sketch_token_flag == "--sketch-token"
            && field_provenance_flag == "--field-provenance"
            && bit_encoding_flag == "--bit-encoding"
            && control_flag == "--control"
            && control_command_flag == "--control-command"
            && control_input_flag == "--control-input"
            && control_input_source_flag == "--control-input-source"
            && control_input_description_flag == "--control-input-description"
            && control_input_sketch_token_flag == "--control-input-sketch-token"
            && control_input_visible_flag == "--control-input-visible"
            && control_input_decision_flag == "--control-input-decision"
            && handled_errors_flag == "--handled-errors"
            && recovery_behavior_flag == "--recovery-behavior"
            && control_sketch_token_flag == "--control-sketch-token"
            && navigation_type_flag == "--navigation-type"
            && navigation_target_flag == "--navigation-target"
            && local_states_flag == "--local-states"
            && filters_flag == "--filters" =>
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
            let control_name = parse_control_name(control)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_command = parse_command_name(control_command)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input = parse_datum_name(control_input)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_source = parse_command_input_source_kind(control_input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_description =
                parse_command_input_source_description(control_input_description)
                    .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_sketch_token = parse_sketch_token(control_input_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_visible = parse_bool_flag(control_input_visible)?;
            let control_input_decision = parse_bool_flag(control_input_decision)?;
            let handled_errors = parse_command_error_names(handled_errors)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let recovery_behavior = parse_control_recovery_behavior(recovery_behavior)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_sketch_token = parse_sketch_token(control_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let navigation_type = parse_navigation_target_type(navigation_type)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let navigation_target = parse_navigation_target_name(navigation_target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let local_states = parse_navigation_target_names(local_states)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let filters = parse_navigation_target_names(filters)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddViewDefinition {
                    view: NewViewDefinition::new(
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
                    )
                    .with_local_states(ViewLocalStates::from_targets(local_states))
                    .with_filters(ViewFilters::from_targets(filters))
                    .with_controls(ViewControls::from_controls([NewControlDefinition::new(
                        control_name,
                        control_command,
                        NewControlInputProvision::new(
                            control_input,
                            control_input_source,
                            control_input_description,
                            control_input_sketch_token,
                            control_input_visible,
                            control_input_decision,
                        ),
                        CommandErrorNames::from_names(handled_errors),
                        recovery_behavior,
                        control_sketch_token,
                        NewNavigationTarget::new(navigation_type, navigation_target),
                    )])),
                },
            })
        }
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
            control_flag,
            control,
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
        ] if command == "add"
            && subject == "view"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && read_model_flag == "--read-model"
            && field_flag == "--field"
            && source_field_flag == "--source-field"
            && sketch_token_flag == "--sketch-token"
            && field_provenance_flag == "--field-provenance"
            && bit_encoding_flag == "--bit-encoding"
            && control_flag == "--control"
            && control_command_flag == "--control-command"
            && control_input_flag == "--control-input"
            && control_input_source_flag == "--control-input-source"
            && control_input_description_flag == "--control-input-description"
            && control_input_sketch_token_flag == "--control-input-sketch-token"
            && control_input_visible_flag == "--control-input-visible"
            && control_input_decision_flag == "--control-input-decision"
            && handled_errors_flag == "--handled-errors"
            && recovery_behavior_flag == "--recovery-behavior"
            && control_sketch_token_flag == "--control-sketch-token"
            && navigation_type_flag == "--navigation-type"
            && navigation_target_flag == "--navigation-target" =>
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
            let control_name = parse_control_name(control)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_command = parse_command_name(control_command)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input = parse_datum_name(control_input)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_source = parse_command_input_source_kind(control_input_source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_description =
                parse_command_input_source_description(control_input_description)
                    .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_sketch_token = parse_sketch_token(control_input_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_input_visible = parse_bool_flag(control_input_visible)?;
            let control_input_decision = parse_bool_flag(control_input_decision)?;
            let handled_errors = parse_command_error_names(handled_errors)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let recovery_behavior = parse_control_recovery_behavior(recovery_behavior)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let control_sketch_token = parse_sketch_token(control_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let navigation_type = parse_navigation_target_type(navigation_type)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let navigation_target = parse_navigation_target_name(navigation_target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddViewDefinition {
                    view: NewViewDefinition::new(
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
                    )
                    .with_controls(ViewControls::from_controls([
                        NewControlDefinition::new(
                            control_name,
                            control_command,
                            NewControlInputProvision::new(
                                control_input,
                                control_input_source,
                                control_input_description,
                                control_input_sketch_token,
                                control_input_visible,
                                control_input_decision,
                            ),
                            CommandErrorNames::from_names(handled_errors),
                            recovery_behavior,
                            control_sketch_token,
                            NewNavigationTarget::new(navigation_type, navigation_target),
                        ),
                    ])),
                },
            })
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
        ] if command == "add"
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
                    field_source_kind,
                    source_event,
                    source_attribute,
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
            Ok(Cli {
                command: Command::AddReadModelDefinition { read_model },
            })
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
            field_source_flag,
            field_source,
            derivation_rule_flag,
            derivation_rule,
            derivation_scenario_flag,
            derivation_scenario,
            field_provenance_flag,
            field_provenance,
        ] if command == "add"
            && subject == "read-model"
            && slice_flag == "--slice"
            && name_flag == "--name"
            && field_flag == "--field"
            && field_source_flag == "--field-source"
            && derivation_rule_flag == "--derivation-rule"
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
            let derivation_scenario = parse_scenario_name(derivation_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let provenance_description = parse_provenance_description(field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddReadModelDefinition {
                    read_model: NewReadModelDefinition::new(
                        slice_slug,
                        read_model_name,
                        NewReadModelField::new_derivation(
                            field_name,
                            field_source_kind,
                            derivation_rule,
                            derivation_scenario,
                            provenance_description,
                        ),
                    ),
                },
            })
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
            field_source_flag,
            field_source,
            absence_event_flag,
            absence_event,
            absence_scenario_flag,
            absence_scenario,
            field_provenance_flag,
            field_provenance,
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddReadModelDefinition {
                    read_model: NewReadModelDefinition::new(
                        slice_slug,
                        read_model_name,
                        NewReadModelField::new_absence_default(
                            field_name,
                            field_source_kind,
                            absence_event,
                            absence_scenario,
                            provenance_description,
                        ),
                    ),
                },
            })
        }
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
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddBoardElement {
                    element: NewBoardElement::new(
                        slice_slug,
                        element_name,
                        element_kind,
                        lane_id,
                        declared_name,
                        main_path,
                    ),
                },
            })
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
        ] if command == "add"
            && subject == "board-connection"
            && slice_flag == "--slice"
            && source_flag == "--source"
            && source_kind_flag == "--source-kind"
            && target_flag == "--target"
            && target_kind_flag == "--target-kind" =>
        {
            let slice_slug =
                parse_slice_slug(slice).map_err(|error| ShellError::message(error.to_string()))?;
            let source = parse_board_connection_endpoint(source)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_kind = parse_board_connection_endpoint_kind(source_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target = parse_board_connection_endpoint(target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target_kind = parse_board_connection_endpoint_kind(target_kind)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddBoardConnection {
                    connection: NewBoardConnection::new(
                        slice_slug,
                        source,
                        source_kind,
                        target,
                        target_kind,
                    ),
                },
            })
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
            field_provenance_flag,
            field_provenance,
            bit_encoding_flag,
            bit_encoding,
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddExternalPayloadDefinition {
                    external_payload: NewExternalPayloadDefinition::new(
                        slice_slug,
                        payload_name,
                        payload_field,
                        field_provenance,
                        bit_encoding,
                    ),
                },
            })
        }
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
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddOutcomeDefinition {
                    outcome: NewOutcomeDefinition::new(
                        slice_slug,
                        label,
                        OutcomeEventNames::from_events(events),
                        externally_relevant,
                    ),
                },
            })
        }
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
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddViewDefinition {
                    view: NewViewDefinition::new(
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
                },
            })
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
            field_source_flag,
            field_source,
            source_event_flag,
            source_event,
            source_attribute_flag,
            source_attribute,
            field_provenance_flag,
            field_provenance,
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddReadModelDefinition {
                    read_model: NewReadModelDefinition::new(
                        slice_slug,
                        read_model_name,
                        NewReadModelField::new(
                            field_name,
                            field_source_kind,
                            source_event,
                            source_attribute,
                            provenance_description,
                        ),
                    ),
                },
            })
        }
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
            if scenario_kind != "contract" {
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
            if scenario_kind != "contract" {
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
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddAutomationDefinition {
                    automation: NewAutomationDefinition::new(
                        slice_slug,
                        automation_name,
                        trigger_name,
                        command_name,
                        CommandErrorNames::from_names(handled_errors),
                        reaction_description,
                    ),
                },
            })
        }
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
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddTranslationDefinition {
                    translation: NewTranslationDefinition::new(
                        slice_slug,
                        translation_name,
                        external_event_name,
                        payload_contract_name,
                        command_name,
                    ),
                },
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
            if scenario_kind != "contract" {
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
            if scenario_kind != "contract" {
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
                            input_source,
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
                    input_source,
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
                            input_source,
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    )
                    .with_observed_streams(CommandObservedStreams::from_streams(observed_streams)),
                },
            })
        }
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
        ] if command == "add"
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
            Ok(Cli {
                command: Command::AddCommandDefinition {
                    command: NewCommandDefinition::new(
                        slice_slug,
                        command_name,
                        NewCommandInput::new(
                            input_name,
                            input_source,
                            input_description,
                            CommandInputProvenanceChain::from_hops(provenance_chain),
                        ),
                        EmittedEventNames::from_events(emitted_events),
                    ),
                },
            })
        }
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
        ] if command == "add"
            && subject == "workflow-outcome"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && label_flag == "--label"
            && externally_relevant_flag == "--externally-relevant" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let label = parse_outcome_label_name(label)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let externally_relevant = parse_bool_flag(externally_relevant)?;
            Ok(Cli {
                command: Command::AddWorkflowOutcome {
                    workflow_slug,
                    outcome: WorkflowOutcomeRecord::new(source_slice, label, externally_relevant),
                },
            })
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
        ] if command == "add"
            && subject == "workflow-command-error"
            && workflow_flag == "--workflow"
            && source_slice_flag == "--source-slice"
            && command_flag == "--command"
            && error_flag == "--error" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slice = WorkflowTransitionEndpoint::try_new(source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command_name = parse_command_name(command_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let error_name = parse_command_error_name(error_name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflowCommandError {
                    workflow_slug,
                    error: WorkflowCommandErrorRecord::new(source_slice, command_name, error_name),
                },
            })
        }
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
            let trigger = parse_transition_trigger_name(trigger)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_evidence = parse_workflow_transition_evidence_text(source_evidence)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let target_evidence = parse_workflow_transition_evidence_text(target_evidence)
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
    match scenario_kind {
        "acceptance" => Ok(NewSliceScenario::new(
            slice_slug,
            ScenarioKind::acceptance(),
            name,
            given,
            when,
            then,
        )),
        "contract" => Err(ShellError::message(
            "contract scenarios require --contract-kind and --covered-definition",
        )),
        _ => Err(ShellError::message(format!(
            "invalid scenario kind: {scenario_kind}"
        ))),
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
    ClapCommand::new("emc")
        .about("Event Model Compiler")
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .subcommand(
            ClapCommand::new("init")
                .about("Create a deterministic EMC project")
                .arg(Arg::new("name").long("name").value_name("PROJECT_NAME")),
        )
        .subcommand(
            ClapCommand::new("list").about("Read model indexes").subcommand(
                ClapCommand::new("workflows").about("List modeled workflows in the project"),
            )
            .subcommand(ClapCommand::new("slices").about("List modeled slices in the project"))
            .subcommand(
                ClapCommand::new("transitions")
                    .about("List modeled workflow transitions in the project"),
            ),
        )
        .subcommand(
            ClapCommand::new("show")
                .about("Read modeled artifacts")
                .subcommand(ClapCommand::new("workflow").about("Show a workflow by slug"))
                .subcommand(ClapCommand::new("slice").about("Show a slice by slug")),
        )
        .subcommand(
            ClapCommand::new("add")
                .about("Create modeled business artifacts")
                .subcommand(
                    ClapCommand::new("workflow")
                        .about("Add a workflow and synchronized formal artifacts"),
                )
                .subcommand(
                    ClapCommand::new("workflow-outcome")
                        .about("Add a workflow composition outcome fact to formal artifacts"),
                )
                .subcommand(
                    ClapCommand::new("workflow-command-error").about(
                        "Add a workflow composition command-error fact to formal artifacts",
                    ),
                )
                .subcommand(
                    ClapCommand::new("workflow-owned-definition").about(
                        "Add a workflow composition ownership fact to formal artifacts",
                    ),
                )
                .subcommand(
                    ClapCommand::new("workflow-transition-evidence").about(
                        "Add workflow transition legality evidence to formal artifacts",
                    ),
                )
                .subcommand(
                    ClapCommand::new("slice")
                        .about("Add a slice and synchronized formal artifacts"),
                )
                .subcommand(
                    ClapCommand::new("scenario")
                        .about("Add an acceptance or contract scenario to formal slice artifacts"),
                )
                .subcommand(
                    ClapCommand::new("command")
                        .about("Add a command definition to formal slice artifacts"),
                )
                .subcommand(
                    ClapCommand::new("event")
                        .about("Add an event definition to formal slice artifacts"),
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
                    ClapCommand::new("view")
                        .about("Add a view field projection to formal slice artifacts"),
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
                ),
        )
        .subcommand(
            ClapCommand::new("update")
                .about("Modify modeled business artifacts")
                .subcommand(
                    ClapCommand::new("workflow")
                        .about("Update a workflow and synchronized formal artifacts"),
                )
                .subcommand(
                    ClapCommand::new("slice")
                        .about("Update a slice and synchronized formal artifacts"),
                ),
        )
        .subcommand(
            ClapCommand::new("connect")
                .about("Connect modeled workflow steps")
                .subcommand(
                    ClapCommand::new("workflow")
                        .about("Add a workflow transition and synchronized formal artifacts"),
                ),
        )
        .subcommand(ClapCommand::new("verify").about("Run Lean4 and Quint verification"))
        .subcommand(ClapCommand::new("check").about("Check project artifact synchronization"))
        .subcommand(
            ClapCommand::new("gherkin")
                .about("List or run checked-in event-model rule suites")
                .subcommand(ClapCommand::new("list").about("List configured feature files"))
                .subcommand(ClapCommand::new("run").about("Run configured rule-suite coverage")),
        )
        .subcommand(
            ClapCommand::new("review")
                .about("Evaluate review gates")
                .subcommand(ClapCommand::new("gate").about("Check a workflow review gate"))
                .subcommand(
                    ClapCommand::new("record").about("Record a clean workflow review"),
                ),
        )
        .subcommand(
            ClapCommand::new("mcp")
                .about("Serve EMC tools over MCP")
                .subcommand(ClapCommand::new("stdio").about("Serve MCP over stdio"))
                .subcommand(ClapCommand::new("http").about("Serve MCP over HTTP")),
        )
        .after_help(
            "Common commands:
  emc init --name <project-name>
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
  emc add workflow-owned-definition --workflow <workflow> --source-slice <slice> --definition-kind <kind> --definition-name <name> [--definition-stream <stream> --source-provenance <text>]
  emc add workflow-transition-evidence --workflow <workflow> --from <step> --to <step> --via <kind> --name <trigger> --source-evidence <text> --target-evidence <text>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --observes <stream[,stream]>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --singleton <true|false> --repeat-behavior <already_exists_error|idempotent>
  emc add command --slice <slice> --name <name> --input <datum> --input-source <kind> --input-description <text> --input-provenance <hop[,hop]> --emits <event[,event]> --error <name> --error-scenario <scenario> --error-recovery <kind>
  emc add external-payload --slice <slice> --name <name> --field <field> --field-provenance <text> --bit-encoding <semantics>
  emc add event --slice <slice> --name <event> --stream <stream> --attribute <name> --attribute-source <kind> --attribute-source-name <name> --attribute-source-field <field> --attribute-provenance <text> [--observed true]
  emc add outcome --slice <slice> --label <label> --events <event[,event]> --externally-relevant <true|false>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --source-event <event> --source-attribute <attribute> --field-provenance <text>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --source-event <event> --source-attribute <attribute> --field-provenance <text> --transitive <true|false> --relationship-fields <field[,field]> --transitive-rule <rule> --example-scenario <scenario>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --derivation-rule <rule> --derivation-scenario <scenario> --field-provenance <text>
  emc add read-model --slice <slice> --name <read-model> --field <name> --field-source <kind> --absence-event <event> --absence-scenario <scenario> --field-provenance <text>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics> --control <control> --control-command <command> --control-input <datum> --control-input-source <kind> --control-input-description <text> --control-input-sketch-token <token> --control-input-visible <true|false> --control-input-decision <true|false> --handled-errors <error[,error]> --recovery-behavior <kind> --control-sketch-token <token> --navigation-type <type> --navigation-target <target>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics> --control <control> --control-command <command> --control-input <datum> --control-input-source <kind> --control-input-description <text> --control-input-sketch-token <token> --control-input-visible <true|false> --control-input-decision <true|false> --handled-errors <error[,error]> --recovery-behavior <kind> --control-sketch-token <token> --navigation-type local_view_state --navigation-target <target> --local-states <state[,state]> --filters <filter[,filter]>
  emc add view --slice <slice> --name <view> --read-model <read-model> --field <name> --source-field <field> --sketch-token <token> --field-provenance <text> --bit-encoding <semantics> --control <control> --control-command <command> --control-input <datum> --control-input-source <kind> --control-input-description <text> --control-input-sketch-token <token> --control-input-visible <true|false> --control-input-decision <true|false> --handled-errors <error[,error]> --recovery-behavior <kind> --control-sketch-token <token> --navigation-type external_system --navigation-target <target> --external-system <name> --handoff-contract <contract>
  emc add automation --slice <slice> --name <name> --trigger <event-or-signal> --command <command> --handled-errors <error[,error]> --reaction <semantics>
  emc add translation --slice <slice> --name <name> --external-event <event-or-signal> --payload-contract <payload> --command <command>
  emc add data-flow --slice <slice> --datum <name> --source <source> --transformation <semantics> --target <target> --bit-encoding <semantics>
  emc update slice --slug <slice> --description <text>
  emc update slice --slug <slice> --type <kind>
  emc update slice --slug <slice> --name <name>
  emc remove slice --slug <slice>
  emc connect workflow --workflow <workflow> --from <slice> --to <slice> --via <kind> --name <trigger> [--payload-contract <contract>]
  emc remove transition --workflow <workflow> --from <slice> --to <slice> --via <kind> --name <trigger>
  emc remove transition --workflow <workflow> --from <slice> --to-workflow <workflow> --via outcome --name <trigger>
  emc list workflows
  emc list slices
  emc list transitions
  emc show workflow <workflow>
  emc show workflow --slug <workflow>
  emc show slice <slice>
  emc show slice --slug <slice>
  emc verify
  emc check
  emc gherkin list --suite <suite>
  emc gherkin run --suite <suite>
  emc gherkin run --all
  emc review gate --workflow <workflow>
  emc review record --workflow <workflow> --reviewer <reviewer> --reviewed-at <timestamp>
  emc mcp stdio
  emc mcp http --host 127.0.0.1 --port 7331
  emc mcp http --host 0.0.0.0 --port 7331 --auth-token <token>",
        )
}

fn parse_port(port: &str) -> Result<u16, ShellError> {
    port.parse::<u16>()
        .map_err(|error| ShellError::message(format!("invalid MCP HTTP port: {error}")))
}
