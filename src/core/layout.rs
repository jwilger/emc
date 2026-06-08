// Copyright 2026 John Wilger

use std::fmt::Display;

use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
use crate::core::effect::{
    ArtifactFileExtension, CanonicalDeclarationMarker, CanonicalDeclarationPrefix, Effect,
    EffectPlan, FileContents, ProjectPath, ReportLine,
};
use crate::core::emit::lean::emit_slice_module as emit_lean_slice_module;
use crate::core::emit::quint::emit_slice_module as emit_quint_slice_module;
use crate::core::formal_graph::{FormalWorkflowGraph, FormalWorkflowGraphs};
use crate::core::formal_model::{
    FormalModelCommand, FormalModelCommandError, FormalModelCommandInput,
    FormalModelCommandInputFields, FormalModelDataFlow, FormalModelDataFlowFields,
    FormalModelOutcome, FormalModelScenario, FormalModelScenarioDefinition,
    FormalModelScenarioDefinitionFields, FormalModelSlice, FormalModelSliceModule,
    lean_model_command_error_list as render_lean_model_command_error_list,
    lean_model_command_input_list as render_lean_model_command_input_list,
    lean_model_command_list as render_lean_model_command_list,
    lean_model_data_flow_list as render_lean_model_data_flow_list,
    lean_model_outcome_list as render_lean_model_outcome_list,
    lean_model_scenario_definition_list as render_lean_model_scenario_definition_list,
    lean_model_scenario_list as render_lean_model_scenario_list,
    lean_model_slice_list as render_lean_model_slice_list,
    lean_model_slice_module_list as render_lean_model_slice_module_list,
    quint_model_command_error_list as render_quint_model_command_error_list,
    quint_model_command_input_list as render_quint_model_command_input_list,
    quint_model_command_list as render_quint_model_command_list,
    quint_model_data_flow_list as render_quint_model_data_flow_list,
    quint_model_outcome_list as render_quint_model_outcome_list,
    quint_model_scenario_definition_list as render_quint_model_scenario_definition_list,
    quint_model_scenario_list as render_quint_model_scenario_list,
    quint_model_slice_list as render_quint_model_slice_list,
    quint_model_slice_module_list as render_quint_model_slice_module_list,
};
use crate::core::formal_project_facts::{
    ProjectAutomation, ProjectAutomationDefinition, ProjectBoardConnection, ProjectBoardElement,
    ProjectCommand, ProjectCommandError, ProjectCommandInput, ProjectDataFlow, ProjectEvent,
    ProjectEventAttribute, ProjectExternalPayload, ProjectExternalPayloadField, ProjectOutcome,
    ProjectReadModel, ProjectReadModelDefinition, ProjectReadModelField, ProjectScenario,
    ProjectScenarioDefinition, ProjectStream, ProjectTranslation, ProjectTranslationDefinition,
    ProjectView, ProjectViewControl, ProjectViewDefinition, ProjectViewField,
};
use crate::core::formal_slice_facts::ScenarioKind;
use crate::core::project::ProjectName;
use crate::core::types::{
    BitEncodingSemantics, CommandErrorName, CommandErrorRecoveryKind,
    CommandInputSourceDescription, CommandInputSourceKind, CommandName, ContractKindName,
    CoveredDefinitionName, DataFlowSource, DataFlowSourceKind, DataFlowTarget, DatumName,
    EventAttributeName, EventAttributeSourceField, EventAttributeSourceName, EventName,
    LeanModuleName, ModelDescription, ModelName, OutcomeLabelName, QuintModuleName, ScenarioName,
    ScenarioStepText, SliceSlug, SourceChainHop, StreamName, TransformationSemantics,
    WorkflowSliceDetail, WorkflowSlug, WorkflowTransitionRecord,
};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ModeledWorkflowLayout {
    name: ModelName,
    description: ModelDescription,
    slug: WorkflowSlug,
}

impl ModeledWorkflowLayout {
    pub(crate) fn new(name: ModelName, description: ModelDescription, slug: WorkflowSlug) -> Self {
        Self {
            name,
            description,
            slug,
        }
    }

    pub(crate) fn name(&self) -> &ModelName {
        &self.name
    }

    pub(crate) fn description(&self) -> &ModelDescription {
        &self.description
    }

    pub(crate) fn slug(&self) -> &WorkflowSlug {
        &self.slug
    }

    pub(crate) fn lean_artifact_path(&self) -> ProjectPath {
        let module_name = module_name_from_model(self.name.clone());
        project_path(format!("model/lean/{module_name}.lean"))
    }

    pub(crate) fn quint_artifact_path(&self) -> ProjectPath {
        let module_name = module_name_from_model(self.name.clone());
        project_path(format!("model/quint/{module_name}.qnt"))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ModeledWorkflowLayouts {
    workflows: Vec<ModeledWorkflowLayout>,
}

impl ModeledWorkflowLayouts {
    pub(crate) fn new(workflows: Vec<ModeledWorkflowLayout>) -> Self {
        Self { workflows }
    }

    pub(crate) fn as_slice(&self) -> &[ModeledWorkflowLayout] {
        &self.workflows
    }

    pub(crate) fn into_inner(self) -> Vec<ModeledWorkflowLayout> {
        self.workflows
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ModeledWorkflowSliceDetails {
    slices: Vec<WorkflowSliceDetail>,
}

impl ModeledWorkflowSliceDetails {
    pub(crate) fn new(slices: Vec<WorkflowSliceDetail>) -> Self {
        Self { slices }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ModeledWorkflowTransitions {
    transitions: Vec<WorkflowTransitionRecord>,
}

impl ModeledWorkflowTransitions {
    pub(crate) fn new(transitions: Vec<WorkflowTransitionRecord>) -> Self {
        Self { transitions }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ModeledProjectRootInventories {
    scenarios: Vec<ProjectScenario>,
    scenario_definitions: Vec<ProjectScenarioDefinition>,
    data_flows: Vec<ProjectDataFlow>,
    outcomes: Vec<ProjectOutcome>,
    command_errors: Vec<ProjectCommandError>,
    commands: Vec<ProjectCommand>,
    command_inputs: Vec<ProjectCommandInput>,
    read_models: Vec<ProjectReadModel>,
    read_model_definitions: Vec<ProjectReadModelDefinition>,
    read_model_fields: Vec<ProjectReadModelField>,
    views: Vec<ProjectView>,
    view_definitions: Vec<ProjectViewDefinition>,
    view_controls: Vec<ProjectViewControl>,
    board_elements: Vec<ProjectBoardElement>,
    board_connections: Vec<ProjectBoardConnection>,
    view_fields: Vec<ProjectViewField>,
    automations: Vec<ProjectAutomation>,
    automation_definitions: Vec<ProjectAutomationDefinition>,
    translations: Vec<ProjectTranslation>,
    translation_definitions: Vec<ProjectTranslationDefinition>,
    external_payloads: Vec<ProjectExternalPayload>,
    external_payload_fields: Vec<ProjectExternalPayloadField>,
    streams: Vec<ProjectStream>,
    events: Vec<ProjectEvent>,
    event_attributes: Vec<ProjectEventAttribute>,
}

pub(crate) struct ModeledProjectRootInventoryParts {
    pub(crate) scenarios: Vec<ProjectScenario>,
    pub(crate) scenario_definitions: Vec<ProjectScenarioDefinition>,
    pub(crate) data_flows: Vec<ProjectDataFlow>,
    pub(crate) outcomes: Vec<ProjectOutcome>,
    pub(crate) command_errors: Vec<ProjectCommandError>,
    pub(crate) commands: Vec<ProjectCommand>,
    pub(crate) command_inputs: Vec<ProjectCommandInput>,
    pub(crate) read_models: Vec<ProjectReadModel>,
    pub(crate) read_model_definitions: Vec<ProjectReadModelDefinition>,
    pub(crate) read_model_fields: Vec<ProjectReadModelField>,
    pub(crate) views: Vec<ProjectView>,
    pub(crate) view_definitions: Vec<ProjectViewDefinition>,
    pub(crate) view_controls: Vec<ProjectViewControl>,
    pub(crate) board_elements: Vec<ProjectBoardElement>,
    pub(crate) board_connections: Vec<ProjectBoardConnection>,
    pub(crate) view_fields: Vec<ProjectViewField>,
    pub(crate) automations: Vec<ProjectAutomation>,
    pub(crate) automation_definitions: Vec<ProjectAutomationDefinition>,
    pub(crate) translations: Vec<ProjectTranslation>,
    pub(crate) translation_definitions: Vec<ProjectTranslationDefinition>,
    pub(crate) external_payloads: Vec<ProjectExternalPayload>,
    pub(crate) external_payload_fields: Vec<ProjectExternalPayloadField>,
    pub(crate) streams: Vec<ProjectStream>,
    pub(crate) events: Vec<ProjectEvent>,
    pub(crate) event_attributes: Vec<ProjectEventAttribute>,
}

impl ModeledProjectRootInventories {
    pub(crate) fn from_parts(parts: ModeledProjectRootInventoryParts) -> Self {
        Self {
            scenarios: parts.scenarios,
            scenario_definitions: parts.scenario_definitions,
            data_flows: parts.data_flows,
            outcomes: parts.outcomes,
            command_errors: parts.command_errors,
            commands: parts.commands,
            command_inputs: parts.command_inputs,
            read_models: parts.read_models,
            read_model_definitions: parts.read_model_definitions,
            read_model_fields: parts.read_model_fields,
            views: parts.views,
            view_definitions: parts.view_definitions,
            view_controls: parts.view_controls,
            board_elements: parts.board_elements,
            board_connections: parts.board_connections,
            view_fields: parts.view_fields,
            automations: parts.automations,
            automation_definitions: parts.automation_definitions,
            translations: parts.translations,
            translation_definitions: parts.translation_definitions,
            external_payloads: parts.external_payloads,
            external_payload_fields: parts.external_payload_fields,
            streams: parts.streams,
            events: parts.events,
            event_attributes: parts.event_attributes,
        }
    }
}

pub(crate) fn check_project(
    project_name: ProjectName,
    formal_workflows: FormalWorkflowGraphs,
    project_inventories: ModeledProjectRootInventories,
) -> EffectPlan {
    let module_name = module_name(&project_name);
    let mut formal_workflows = formal_workflows.into_inner();
    formal_workflows.sort_by(|left, right| left.slug().as_ref().cmp(right.slug().as_ref()));
    let modeled_workflows = formal_workflows
        .iter()
        .map(modeled_workflow_layout)
        .collect::<Vec<_>>();
    let root_effects = project_root_effects(
        &project_name,
        &module_name,
        &modeled_workflows,
        &formal_workflows,
        &ProjectRootInventories {
            scenarios: &project_inventories.scenarios,
            scenario_definitions: &project_inventories.scenario_definitions,
            data_flows: &project_inventories.data_flows,
            outcomes: &project_inventories.outcomes,
            command_errors: &project_inventories.command_errors,
            commands: &project_inventories.commands,
            command_inputs: &project_inventories.command_inputs,
            read_models: &project_inventories.read_models,
            read_model_definitions: &project_inventories.read_model_definitions,
            read_model_fields: &project_inventories.read_model_fields,
            views: &project_inventories.views,
            view_definitions: &project_inventories.view_definitions,
            view_controls: &project_inventories.view_controls,
            board_elements: &project_inventories.board_elements,
            board_connections: &project_inventories.board_connections,
            view_fields: &project_inventories.view_fields,
            automations: &project_inventories.automations,
            automation_definitions: &project_inventories.automation_definitions,
            translations: &project_inventories.translations,
            translation_definitions: &project_inventories.translation_definitions,
            external_payloads: &project_inventories.external_payloads,
            external_payload_fields: &project_inventories.external_payload_fields,
            streams: &project_inventories.streams,
            events: &project_inventories.events,
            event_attributes: &project_inventories.event_attributes,
        },
    );
    let lean_artifact_paths = modeled_artifact_paths(
        [
            project_path("model/lean/lakefile.lean"),
            project_path(format!("model/lean/{module_name}.lean")),
        ],
        &modeled_workflows,
        ModeledWorkflowLayout::lean_artifact_path,
    );
    let quint_artifact_paths = modeled_artifact_paths(
        [project_path(format!("model/quint/{module_name}.qnt"))],
        &modeled_workflows,
        ModeledWorkflowLayout::quint_artifact_path,
    );
    let lean_slice_artifact_paths =
        modeled_slice_artifact_paths(&formal_workflows, "model/lean/slices", ".lean");
    let quint_slice_artifact_paths =
        modeled_slice_artifact_paths(&formal_workflows, "model/quint/slices", ".qnt");
    let modeled_effects = formal_workflows
        .iter()
        .flat_map(formal_workflow_effects)
        .collect::<Vec<_>>();

    EffectPlan::new(
        [
            vec![
                Effect::RequireFile(project_path("emc.toml")),
                Effect::RequireFile(project_path("model/lean/lakefile.lean")),
                Effect::RequireFile(project_path("model/lean/lean-toolchain")),
                Effect::RequireFile(project_path(format!("model/lean/{module_name}.lean"))),
                Effect::RequireFile(project_path("model/lean/slices/.gitkeep")),
                Effect::RequireFile(project_path(format!("model/quint/{module_name}.qnt"))),
                Effect::RequireFile(project_path("model/quint/slices/.gitkeep")),
                Effect::require_only_modeled_artifacts(
                    project_path("model/lean"),
                    artifact_file_extension(".lean"),
                    lean_artifact_paths,
                    report_line("Lean model artifact drift"),
                ),
                Effect::require_only_modeled_artifacts(
                    project_path("model/quint"),
                    artifact_file_extension(".qnt"),
                    quint_artifact_paths,
                    report_line("Quint model artifact drift"),
                ),
                Effect::require_only_modeled_artifacts(
                    project_path("model/lean/slices"),
                    artifact_file_extension(".lean"),
                    lean_slice_artifact_paths,
                    report_line("Lean slice artifact drift"),
                ),
                Effect::require_only_modeled_artifacts(
                    project_path("model/quint/slices"),
                    artifact_file_extension(".qnt"),
                    quint_slice_artifact_paths,
                    report_line("Quint slice artifact drift"),
                ),
                Effect::RequireFile(project_path("reviews/.gitkeep")),
            ],
            root_effects,
            modeled_effects,
            vec![Effect::Report(report_line("project layout is complete"))],
        ]
        .concat(),
    )
}

fn modeled_workflow_layout(workflow: &FormalWorkflowGraph) -> ModeledWorkflowLayout {
    ModeledWorkflowLayout::new(
        workflow.name().clone(),
        workflow.description().clone(),
        workflow.slug().clone(),
    )
}

fn modeled_slice_artifact_paths(
    formal_workflows: &[FormalWorkflowGraph],
    artifact_directory: &str,
    extension: &str,
) -> Vec<ProjectPath> {
    formal_workflows
        .iter()
        .flat_map(|workflow| {
            workflow.slice_details().as_slice().iter().map(|slice| {
                project_path(format!(
                    "{}/{}{}",
                    artifact_directory,
                    module_name_from_model(slice.name().clone()),
                    extension
                ))
            })
        })
        .collect()
}

struct ProjectRootInventories<'a> {
    scenarios: &'a [ProjectScenario],
    scenario_definitions: &'a [ProjectScenarioDefinition],
    data_flows: &'a [ProjectDataFlow],
    outcomes: &'a [ProjectOutcome],
    command_errors: &'a [ProjectCommandError],
    commands: &'a [ProjectCommand],
    command_inputs: &'a [ProjectCommandInput],
    read_models: &'a [ProjectReadModel],
    read_model_definitions: &'a [ProjectReadModelDefinition],
    read_model_fields: &'a [ProjectReadModelField],
    views: &'a [ProjectView],
    view_definitions: &'a [ProjectViewDefinition],
    view_controls: &'a [ProjectViewControl],
    board_elements: &'a [ProjectBoardElement],
    board_connections: &'a [ProjectBoardConnection],
    view_fields: &'a [ProjectViewField],
    automations: &'a [ProjectAutomation],
    automation_definitions: &'a [ProjectAutomationDefinition],
    translations: &'a [ProjectTranslation],
    translation_definitions: &'a [ProjectTranslationDefinition],
    external_payloads: &'a [ProjectExternalPayload],
    external_payload_fields: &'a [ProjectExternalPayloadField],
    streams: &'a [ProjectStream],
    events: &'a [ProjectEvent],
    event_attributes: &'a [ProjectEventAttribute],
}

fn project_root_effects(
    project_name: &ProjectName,
    module_name: &str,
    modeled_workflows: &[ModeledWorkflowLayout],
    formal_workflows: &[FormalWorkflowGraph],
    inventories: &ProjectRootInventories<'_>,
) -> Vec<Effect> {
    let project_name_text = project_name.as_ref();
    let model_version = "0.1.0";
    let workflow_slug_list = workflow_slug_list(modeled_workflows);
    let workflow_count = modeled_workflows.len();
    let model_slices = formal_model_slices(formal_workflows);
    let model_slice_modules = formal_model_slice_modules(formal_workflows);
    let model_scenarios = formal_model_scenarios(inventories.scenarios);
    let model_scenario_definitions =
        formal_model_scenario_definitions(inventories.scenario_definitions);
    let model_data_flows = formal_model_data_flows(inventories.data_flows);
    let model_outcomes = formal_model_outcomes(inventories.outcomes);
    let model_command_errors = formal_model_command_errors(inventories.command_errors);
    let model_commands = formal_model_commands(inventories.commands);
    let model_command_inputs = formal_model_command_inputs(inventories.command_inputs);
    let lean_model_slice_list = render_lean_model_slice_list(&model_slices);
    let lean_model_slice_module_list = render_lean_model_slice_module_list(&model_slice_modules);
    let lean_model_scenario_list = render_lean_model_scenario_list(&model_scenarios);
    let lean_model_scenario_definition_list =
        render_lean_model_scenario_definition_list(&model_scenario_definitions);
    let lean_model_data_flow_list = render_lean_model_data_flow_list(&model_data_flows);
    let lean_model_outcome_list = render_lean_model_outcome_list(&model_outcomes);
    let lean_model_command_error_list = render_lean_model_command_error_list(&model_command_errors);
    let lean_model_command_list = render_lean_model_command_list(&model_commands);
    let lean_model_command_input_list = render_lean_model_command_input_list(&model_command_inputs);
    let lean_model_read_model_list = lean_model_read_model_list(inventories.read_models);
    let lean_model_read_model_definition_list =
        lean_model_read_model_definition_list(inventories.read_model_definitions);
    let lean_model_read_model_field_list =
        lean_model_read_model_field_list(inventories.read_model_fields);
    let lean_model_view_list = lean_model_view_list(inventories.views);
    let lean_model_view_definition_list =
        lean_model_view_definition_list(inventories.view_definitions);
    let lean_model_view_control_list = lean_model_view_control_list(inventories.view_controls);
    let lean_model_board_element_list = lean_model_board_element_list(inventories.board_elements);
    let lean_model_board_connection_list =
        lean_model_board_connection_list(inventories.board_connections);
    let lean_model_view_field_list = lean_model_view_field_list(inventories.view_fields);
    let lean_model_automation_list = lean_model_automation_list(inventories.automations);
    let lean_model_automation_definition_list =
        lean_model_automation_definition_list(inventories.automation_definitions);
    let lean_model_translation_list = lean_model_translation_list(inventories.translations);
    let lean_model_translation_definition_list =
        lean_model_translation_definition_list(inventories.translation_definitions);
    let lean_model_external_payload_list =
        lean_model_external_payload_list(inventories.external_payloads);
    let lean_model_external_payload_field_list =
        lean_model_external_payload_field_list(inventories.external_payload_fields);
    let lean_model_stream_list = lean_model_stream_list(inventories.streams);
    let lean_model_event_list = lean_model_event_list(inventories.events);
    let lean_model_event_attribute_list =
        lean_model_event_attribute_list(inventories.event_attributes);
    let quint_model_slice_list = render_quint_model_slice_list(&model_slices);
    let quint_model_slice_module_list = render_quint_model_slice_module_list(&model_slice_modules);
    let quint_model_scenario_list = render_quint_model_scenario_list(&model_scenarios);
    let quint_model_scenario_definition_list =
        render_quint_model_scenario_definition_list(&model_scenario_definitions);
    let quint_model_data_flow_list = render_quint_model_data_flow_list(&model_data_flows);
    let quint_model_outcome_list = render_quint_model_outcome_list(&model_outcomes);
    let quint_model_command_error_list =
        render_quint_model_command_error_list(&model_command_errors);
    let quint_model_command_list = render_quint_model_command_list(&model_commands);
    let quint_model_command_input_list =
        render_quint_model_command_input_list(&model_command_inputs);
    let quint_model_read_model_list = quint_model_read_model_list(inventories.read_models);
    let quint_model_read_model_definition_list =
        quint_model_read_model_definition_list(inventories.read_model_definitions);
    let quint_model_read_model_field_list =
        quint_model_read_model_field_list(inventories.read_model_fields);
    let quint_model_view_list = quint_model_view_list(inventories.views);
    let quint_model_view_definition_list =
        quint_model_view_definition_list(inventories.view_definitions);
    let quint_model_view_control_list = quint_model_view_control_list(inventories.view_controls);
    let quint_model_board_element_list = quint_model_board_element_list(inventories.board_elements);
    let quint_model_board_connection_list =
        quint_model_board_connection_list(inventories.board_connections);
    let quint_model_view_field_list = quint_model_view_field_list(inventories.view_fields);
    let quint_model_automation_list = quint_model_automation_list(inventories.automations);
    let quint_model_automation_definition_list =
        quint_model_automation_definition_list(inventories.automation_definitions);
    let quint_model_translation_list = quint_model_translation_list(inventories.translations);
    let quint_model_translation_definition_list =
        quint_model_translation_definition_list(inventories.translation_definitions);
    let quint_model_external_payload_list =
        quint_model_external_payload_list(inventories.external_payloads);
    let quint_model_external_payload_field_list =
        quint_model_external_payload_field_list(inventories.external_payload_fields);
    let quint_model_stream_list = quint_model_stream_list(inventories.streams);
    let quint_model_event_list = quint_model_event_list(inventories.events);
    let quint_model_event_attribute_list =
        quint_model_event_attribute_list(inventories.event_attributes);
    let model_digest = model_digest(
        project_name,
        modeled_workflows,
        formal_workflows,
        inventories,
    );
    let slice_count = formal_workflows
        .iter()
        .map(|workflow| workflow.slice_details().as_slice().len())
        .sum::<usize>();
    let scenario_count = inventories.scenarios.len();
    let scenario_definition_count = inventories.scenario_definitions.len();
    let data_flow_count = inventories.data_flows.len();
    let outcome_count = inventories.outcomes.len();
    let command_error_count = inventories.command_errors.len();
    let stream_count = inventories.streams.len();
    let command_count = inventories.commands.len();
    let command_input_count = inventories.command_inputs.len();
    let read_model_count = inventories.read_models.len();
    let read_model_definition_count = inventories.read_model_definitions.len();
    let read_model_field_count = inventories.read_model_fields.len();
    let view_count = inventories.views.len();
    let view_definition_count = inventories.view_definitions.len();
    let view_control_count = inventories.view_controls.len();
    let board_element_count = inventories.board_elements.len();
    let board_connection_count = inventories.board_connections.len();
    let view_field_count = inventories.view_fields.len();
    let automation_count = inventories.automations.len();
    let automation_definition_count = inventories.automation_definitions.len();
    let translation_count = inventories.translations.len();
    let translation_definition_count = inventories.translation_definitions.len();
    let external_payload_count = inventories.external_payloads.len();
    let external_payload_field_count = inventories.external_payload_fields.len();
    let event_count = inventories.events.len();
    let event_attribute_count = inventories.event_attributes.len();
    let manifest_path = project_path("emc.toml");
    let lean_path = project_path(format!("model/lean/{module_name}.lean"));
    let lakefile_path = project_path("model/lean/lakefile.lean");
    let lean_toolchain_path = project_path("model/lean/lean-toolchain");
    let quint_path = project_path(format!("model/quint/{module_name}.qnt"));
    let manifest_message = report_line(format!("project manifest drift for {project_name_text}"));
    let lean_message = report_line(format!("Lean project root drift for {project_name_text}"));
    let lean_config_message =
        report_line(format!("Lean project config drift for {project_name_text}"));
    let quint_message = report_line(format!("Quint project root drift for {project_name_text}"));
    let quint_module_close_prefix = canonical_declaration_prefix("}");
    let quint_module_close_marker = canonical_declaration_marker("}");

    vec![
        Effect::require_canonical_declaration(
            manifest_path.clone(),
            canonical_declaration_prefix("name ="),
            canonical_declaration_marker(format!("name = {}", json_string(project_name_text))),
            manifest_message.clone(),
        ),
        Effect::require_canonical_declaration(
            manifest_path.clone(),
            canonical_declaration_prefix("version ="),
            canonical_declaration_marker(format!("version = {}", json_string(model_version))),
            manifest_message.clone(),
        ),
        Effect::require_canonical_declaration(
            manifest_path.clone(),
            canonical_declaration_prefix("lean_module ="),
            canonical_declaration_marker(format!("lean_module = \"{module_name}\"")),
            manifest_message.clone(),
        ),
        Effect::require_canonical_declaration(
            manifest_path,
            canonical_declaration_prefix("quint_module ="),
            canonical_declaration_marker(format!("quint_module = \"{module_name}\"")),
            manifest_message,
        ),
        Effect::require_canonical_declaration(
            lakefile_path.clone(),
            canonical_declaration_prefix("import Lake"),
            canonical_declaration_marker("import Lake"),
            lean_config_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lakefile_path.clone(),
            canonical_declaration_prefix("open Lake"),
            canonical_declaration_marker("open Lake DSL"),
            lean_config_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lakefile_path,
            canonical_declaration_prefix("package "),
            canonical_declaration_marker("package EMCModel where"),
            lean_config_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_toolchain_path,
            canonical_declaration_prefix("leanprover/lean4:"),
            canonical_declaration_marker("leanprover/lean4:4.29.1"),
            lean_config_message,
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("namespace "),
            canonical_declaration_marker(format!("namespace {module_name}")),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelVersion :="),
            canonical_declaration_marker(format!(
                "def modelVersion := {}",
                json_string(model_version)
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelName :="),
            canonical_declaration_marker(format!(
                "def modelName := {}",
                json_string(project_name_text)
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDigest :="),
            canonical_declaration_marker(format!(
                "def modelDigest := {}",
                json_string(&model_digest)
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelWorkflows :"),
            canonical_declaration_marker(format!(
                "def modelWorkflows : List String := {workflow_slug_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelSlice where"),
            canonical_declaration_marker("structure ModelSlice where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelSliceModule where"),
            canonical_declaration_marker("structure ModelSliceModule where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  formalModule : String"),
            canonical_declaration_marker("  formalModule : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelSlices :"),
            canonical_declaration_marker(format!(
                "def modelSlices : List ModelSlice := {lean_model_slice_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelSliceModules :"),
            canonical_declaration_marker(format!(
                "def modelSliceModules : List ModelSliceModule := {lean_model_slice_module_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelSliceBelongsToDeclaredWorkflow"),
            canonical_declaration_marker(
                "def modelSliceBelongsToDeclaredWorkflow (slice : ModelSlice) : Bool := modelWorkflows.any (fun workflow => workflow == slice.workflow)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelSliceHasModule"),
            canonical_declaration_marker(
                "def modelSliceHasModule (slice : ModelSlice) : Bool := modelSliceModules.any (fun sliceModule => sliceModule.workflow == slice.workflow && sliceModule.slice == slice.slice && sliceModule.formalModule.isEmpty == false)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelSliceModuleBelongsToDeclaredSlice"),
            canonical_declaration_marker(
                "def modelSliceModuleBelongsToDeclaredSlice (sliceModule : ModelSliceModule) : Bool := sliceModule.formalModule.isEmpty == false && modelSlices.any (fun slice => slice.workflow == sliceModule.workflow && slice.slice == sliceModule.slice)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelWorkflowSlicesHaveModules"),
            canonical_declaration_marker(
                "def modelWorkflowSlicesHaveModules (workflow : String) : Bool := modelSlices.all (fun slice => slice.workflow != workflow || modelSliceHasModule slice)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelWorkflowHasCompositionStructure"),
            canonical_declaration_marker(
                "def modelWorkflowHasCompositionStructure (workflow : String) : Bool := modelWorkflowSlicesHaveModules workflow",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelScenario where"),
            canonical_declaration_marker("structure ModelScenario where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelScenarioDefinition where"),
            canonical_declaration_marker("structure ModelScenarioDefinition where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  errorReferences : List String"),
            canonical_declaration_marker("  errorReferences : List String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelDataFlow where"),
            canonical_declaration_marker("structure ModelDataFlow where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  transformation : String"),
            canonical_declaration_marker("  transformation : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelOutcome where"),
            canonical_declaration_marker("structure ModelOutcome where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  externallyRelevant : Bool"),
            canonical_declaration_marker("  externallyRelevant : Bool"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelCommandError where"),
            canonical_declaration_marker("structure ModelCommandError where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  recovery : String"),
            canonical_declaration_marker("  recovery : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelCommand where"),
            canonical_declaration_marker("structure ModelCommand where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelCommandInput where"),
            canonical_declaration_marker("structure ModelCommandInput where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  invocationArgumentSourceField : String"),
            canonical_declaration_marker("  invocationArgumentSourceField : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelReadModel where"),
            canonical_declaration_marker("structure ModelReadModel where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelReadModelDefinition where"),
            canonical_declaration_marker("structure ModelReadModelDefinition where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelReadModelField where"),
            canonical_declaration_marker("structure ModelReadModelField where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  absenceScenarioName : String"),
            canonical_declaration_marker("  absenceScenarioName : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelView where"),
            canonical_declaration_marker("structure ModelView where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelViewDefinition where"),
            canonical_declaration_marker("structure ModelViewDefinition where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelViewControl where"),
            canonical_declaration_marker("structure ModelViewControl where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  handoffContract : String"),
            canonical_declaration_marker("  handoffContract : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelBoardElement where"),
            canonical_declaration_marker("structure ModelBoardElement where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  mainPath : Bool"),
            canonical_declaration_marker("  mainPath : Bool"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelBoardConnection where"),
            canonical_declaration_marker("structure ModelBoardConnection where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelViewField where"),
            canonical_declaration_marker("structure ModelViewField where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  sourceReadModel : String"),
            canonical_declaration_marker("  sourceReadModel : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelExternalPayload where"),
            canonical_declaration_marker("structure ModelExternalPayload where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelExternalPayloadField where"),
            canonical_declaration_marker("structure ModelExternalPayloadField where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelStream where"),
            canonical_declaration_marker("structure ModelStream where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelEvent where"),
            canonical_declaration_marker("structure ModelEvent where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("structure ModelEventAttribute where"),
            canonical_declaration_marker("structure ModelEventAttribute where"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("  generatedSourceKind : String"),
            canonical_declaration_marker("  generatedSourceKind : String"),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelScenarios :"),
            canonical_declaration_marker(format!(
                "def modelScenarios : List ModelScenario := {lean_model_scenario_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelScenarioDefinitions :"),
            canonical_declaration_marker(format!(
                "def modelScenarioDefinitions : List ModelScenarioDefinition := {lean_model_scenario_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlows :"),
            canonical_declaration_marker(format!(
                "def modelDataFlows : List ModelDataFlow := {lean_model_data_flow_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelOutcomes :"),
            canonical_declaration_marker(format!(
                "def modelOutcomes : List ModelOutcome := {lean_model_outcome_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelCommandErrors :"),
            canonical_declaration_marker(format!(
                "def modelCommandErrors : List ModelCommandError := {lean_model_command_error_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelCommands :"),
            canonical_declaration_marker(format!(
                "def modelCommands : List ModelCommand := {lean_model_command_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelCommandInputs :"),
            canonical_declaration_marker(format!(
                "def modelCommandInputs : List ModelCommandInput := {lean_model_command_input_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelReadModels :"),
            canonical_declaration_marker(format!(
                "def modelReadModels : List ModelReadModel := {lean_model_read_model_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelReadModelDefinitions :"),
            canonical_declaration_marker(format!(
                "def modelReadModelDefinitions : List ModelReadModelDefinition := {lean_model_read_model_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelReadModelFields :"),
            canonical_declaration_marker(format!(
                "def modelReadModelFields : List ModelReadModelField := {lean_model_read_model_field_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViews :"),
            canonical_declaration_marker(format!(
                "def modelViews : List ModelView := {lean_model_view_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewDefinitions :"),
            canonical_declaration_marker(format!(
                "def modelViewDefinitions : List ModelViewDefinition := {lean_model_view_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewControls :"),
            canonical_declaration_marker(format!(
                "def modelViewControls : List ModelViewControl := {lean_model_view_control_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelBoardElements :"),
            canonical_declaration_marker(format!(
                "def modelBoardElements : List ModelBoardElement := {lean_model_board_element_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelBoardConnections :"),
            canonical_declaration_marker(format!(
                "def modelBoardConnections : List ModelBoardConnection := {lean_model_board_connection_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewFields :"),
            canonical_declaration_marker(format!(
                "def modelViewFields : List ModelViewField := {lean_model_view_field_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelAutomations :"),
            canonical_declaration_marker(format!(
                "def modelAutomations : List (String × String × String) := {lean_model_automation_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelAutomationDefinitions :"),
            canonical_declaration_marker(format!(
                "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := {lean_model_automation_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelTranslations :"),
            canonical_declaration_marker(format!(
                "def modelTranslations : List ModelTranslation := {lean_model_translation_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelTranslationDefinitions :"),
            canonical_declaration_marker(format!(
                "def modelTranslationDefinitions : List ModelTranslationDefinition := {lean_model_translation_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelExternalPayloads :"),
            canonical_declaration_marker(format!(
                "def modelExternalPayloads : List ModelExternalPayload := {lean_model_external_payload_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelExternalPayloadFields :"),
            canonical_declaration_marker(format!(
                "def modelExternalPayloadFields : List ModelExternalPayloadField := {lean_model_external_payload_field_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelStreams :"),
            canonical_declaration_marker(format!(
                "def modelStreams : List ModelStream := {lean_model_stream_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelEvents :"),
            canonical_declaration_marker(format!(
                "def modelEvents : List ModelEvent := {lean_model_event_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelEventAttributes :"),
            canonical_declaration_marker(format!(
                "def modelEventAttributes : List ModelEventAttribute := {lean_model_event_attribute_list}"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelScenarioDefinitionHasGwt"),
            canonical_declaration_marker(
                "def modelScenarioDefinitionHasGwt (scenario : ModelScenarioDefinition) : Bool := scenario.given.isEmpty == false && scenario.when.isEmpty == false && scenario.thenStep.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelScenarioKindIsFirstClass"),
            canonical_declaration_marker(
                "def modelScenarioKindIsFirstClass (scenario : ModelScenarioDefinition) : Bool := scenario.scenarioKind == \"acceptance\" || scenario.scenarioKind == \"contract\"",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowIsBitComplete"),
            canonical_declaration_marker(
                "def modelDataFlowIsBitComplete (dataFlow : ModelDataFlow) : Bool := dataFlow.datum.isEmpty == false && dataFlow.sourceKind.isEmpty == false && dataFlow.source.isEmpty == false && dataFlow.transformation.isEmpty == false && dataFlow.target.isEmpty == false && dataFlow.bitEncoding.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowCoversDatumTarget"),
            canonical_declaration_marker(
                "def modelDataFlowCoversDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) : Bool := modelDataFlows.any (fun dataFlow => dataFlow.workflow == workflow && dataFlow.slice == slice && dataFlow.datum == datum && dataFlow.target == target && modelDataFlowIsBitComplete dataFlow)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowBitEncodingMatchesDatumTarget"),
            canonical_declaration_marker(
                "def modelDataFlowBitEncodingMatchesDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) (bitEncoding : String) : Bool := modelDataFlows.any (fun dataFlow => dataFlow.workflow == workflow && dataFlow.slice == slice && dataFlow.datum == datum && dataFlow.target == target && dataFlow.bitEncoding == bitEncoding && modelDataFlowIsBitComplete dataFlow)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowSourceBitEncodingMatchesModeledSource"),
            canonical_declaration_marker(
                "def modelDataFlowSourceBitEncodingMatchesModeledSource (dataFlow : ModelDataFlow) : Bool := (modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source) == false) || modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && sourceFlow.bitEncoding == dataFlow.bitEncoding && modelDataFlowIsBitComplete sourceFlow)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowHasModeledTransformationSemantics"),
            canonical_declaration_marker(
                "def modelDataFlowHasModeledTransformationSemantics (dataFlow : ModelDataFlow) : Bool := dataFlow.transformation == \"identity\" || dataFlow.transformation == \"projection\" || dataFlow.transformation == \"derivation\" || dataFlow.transformation == \"default\" || dataFlow.transformation == \"absence\" || dataFlow.transformation == \"transformation\"",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowHasModeledSourceKind"),
            canonical_declaration_marker(
                "def modelDataFlowHasModeledSourceKind (dataFlow : ModelDataFlow) : Bool := (dataFlow.sourceKind == \"original\" && dataFlow.source.isEmpty == false) || (dataFlow.sourceKind == \"modeled_target\" && dataFlow.source.isEmpty == false)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowModeledSourceResolves"),
            canonical_declaration_marker(
                "def modelDataFlowModeledSourceResolves (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind != \"modeled_target\" || modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && modelDataFlowIsBitComplete sourceFlow)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelSameDataFlowTarget"),
            canonical_declaration_marker(
                "def modelSameDataFlowTarget (left : ModelDataFlow) (right : ModelDataFlow) : Bool := left.workflow == right.workflow && left.slice == right.slice && left.datum == right.datum && left.target == right.target",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowTargetsFromReachable"),
            canonical_declaration_marker(
                "def modelDataFlowTargetsFromReachable (reachable : List ModelDataFlow) : List ModelDataFlow := modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"modeled_target\" && reachable.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && modelDataFlowIsBitComplete sourceFlow))",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowsReachableFromOriginalsAfterFuel"),
            canonical_declaration_marker(
                "def modelDataFlowsReachableFromOriginalsAfterFuel : Nat -> List ModelDataFlow -> List ModelDataFlow",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowsReachableFromOriginals :"),
            canonical_declaration_marker(
                "def modelDataFlowsReachableFromOriginals : List ModelDataFlow := modelDataFlowsReachableFromOriginalsAfterFuel modelDataFlows.length (modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"original\" && modelDataFlowIsBitComplete dataFlow))",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowHasOriginalSourceChain"),
            canonical_declaration_marker(
                "def modelDataFlowHasOriginalSourceChain (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind == \"original\" || modelDataFlowsReachableFromOriginals.any (fun reachableFlow => modelSameDataFlowTarget reachableFlow dataFlow)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowTargetsFromBitPreservingReachable"),
            canonical_declaration_marker(
                "def modelDataFlowTargetsFromBitPreservingReachable (reachable : List ModelDataFlow) : List ModelDataFlow := modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"modeled_target\" && reachable.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && sourceFlow.bitEncoding == dataFlow.bitEncoding && modelDataFlowIsBitComplete sourceFlow))",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix(
                "def modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel",
            ),
            canonical_declaration_marker(
                "def modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel : Nat -> List ModelDataFlow -> List ModelDataFlow",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix(
                "def modelDataFlowsReachableFromOriginalsWithPreservedBits :",
            ),
            canonical_declaration_marker(
                "def modelDataFlowsReachableFromOriginalsWithPreservedBits : List ModelDataFlow := modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel modelDataFlows.length (modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"original\" && modelDataFlowIsBitComplete dataFlow))",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDataFlowHasBitPreservingOriginalSourceChain"),
            canonical_declaration_marker(
                "def modelDataFlowHasBitPreservingOriginalSourceChain (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind == \"original\" || modelDataFlowsReachableFromOriginalsWithPreservedBits.any (fun reachableFlow => modelSameDataFlowTarget reachableFlow dataFlow)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelCommandInputHasModeledDataFlow"),
            canonical_declaration_marker(
                "def modelCommandInputHasModeledDataFlow (input : ModelCommandInput) : Bool := modelDataFlowCoversDatumTarget input.workflow input.slice input.input input.command",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelEventAttributeHasModeledDataFlow"),
            canonical_declaration_marker(
                "def modelEventAttributeHasModeledDataFlow (eventAttribute : ModelEventAttribute) : Bool := modelDataFlowCoversDatumTarget eventAttribute.workflow eventAttribute.slice eventAttribute.attributeName eventAttribute.event",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelReadModelFieldHasModeledDataFlow"),
            canonical_declaration_marker(
                "def modelReadModelFieldHasModeledDataFlow (field : ModelReadModelField) : Bool := modelDataFlowCoversDatumTarget field.workflow field.slice field.field field.readModel",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewFieldHasModeledDataFlow"),
            canonical_declaration_marker(
                "def modelViewFieldHasModeledDataFlow (field : ModelViewField) : Bool := modelDataFlowCoversDatumTarget field.workflow field.slice field.field field.view",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewFieldBitEncodingMatchesDataFlow"),
            canonical_declaration_marker(
                "def modelViewFieldBitEncodingMatchesDataFlow (field : ModelViewField) : Bool := modelDataFlowBitEncodingMatchesDatumTarget field.workflow field.slice field.field field.view field.bitEncoding",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelExternalPayloadFieldHasModeledDataFlow"),
            canonical_declaration_marker(
                "def modelExternalPayloadFieldHasModeledDataFlow (field : ModelExternalPayloadField) : Bool := modelDataFlowCoversDatumTarget field.workflow field.slice field.field field.externalPayload",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelExternalPayloadFieldBitEncodingMatchesDataFlow"),
            canonical_declaration_marker(
                "def modelExternalPayloadFieldBitEncodingMatchesDataFlow (field : ModelExternalPayloadField) : Bool := modelDataFlowBitEncodingMatchesDatumTarget field.workflow field.slice field.field field.externalPayload field.bitEncoding",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelMeaningfulDataHasModeledDataFlows"),
            canonical_declaration_marker(
                "def modelMeaningfulDataHasModeledDataFlows : Bool := modelCommandInputs.all modelCommandInputHasModeledDataFlow && modelEventAttributes.all modelEventAttributeHasModeledDataFlow && modelReadModelFields.all modelReadModelFieldHasModeledDataFlow && modelViewFields.all modelViewFieldHasModeledDataFlow && modelExternalPayloadFields.all modelExternalPayloadFieldHasModeledDataFlow",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelCommandInputHasProvenance"),
            canonical_declaration_marker(
                "def modelCommandInputHasProvenance (input : ModelCommandInput) : Bool := input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelCommandInputTracesToInvocationSource"),
            canonical_declaration_marker(
                "def modelCommandInputTracesToInvocationSource (input : ModelCommandInput) : Bool := input.sourceKind == \"actor\" || (input.sourceKind == \"event_stream_state\" && input.eventStreamSourceEvent.isEmpty == false && input.eventStreamSourceAttribute.isEmpty == false) || (input.sourceKind == \"external_payload\" && input.externalPayloadSourceName.isEmpty == false && input.externalPayloadSourceField.isEmpty == false) || (input.sourceKind == \"generated\" && input.generatedSourceName.isEmpty == false && input.generatedSourceField.isEmpty == false) || (input.sourceKind == \"session\" && input.sessionSourceName.isEmpty == false && input.sessionSourceField.isEmpty == false) || (input.sourceKind == \"invocation_argument\" && input.invocationArgumentSourceName.isEmpty == false && input.invocationArgumentSourceField.isEmpty == false)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelEventAttributeSourceIsComplete"),
            canonical_declaration_marker(
                "def modelEventAttributeSourceIsComplete (eventAttribute : ModelEventAttribute) : Bool := eventAttribute.provenance.isEmpty == false && ((eventAttribute.sourceKind == \"command_input\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false) || (eventAttribute.sourceKind == \"external_payload\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false) || (eventAttribute.sourceKind == \"generated\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.generatedSourceKind.isEmpty == false) || (eventAttribute.sourceKind == \"session\" && eventAttribute.sourceName.isEmpty == false) || (eventAttribute.sourceKind == \"derivation\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false))",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelReadModelFieldSourceIsComplete"),
            canonical_declaration_marker(
                "def modelReadModelFieldSourceIsComplete (field : ModelReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false && field.derivationSourceFields.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewFieldSourceIsComplete"),
            canonical_declaration_marker(
                "def modelViewFieldSourceIsComplete (field : ModelViewField) : Bool := field.sourceKind == \"read_model\" && field.sourceReadModel.isEmpty == false && field.sourceField.isEmpty == false && field.provenance.isEmpty == false && field.bitEncoding.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelReadModelFieldTracesToOriginalProvenance"),
            canonical_declaration_marker(
                "def modelReadModelFieldTracesToOriginalProvenance (field : ModelReadModelField) : Bool := field.provenance.isEmpty == false && ((field.sourceKind == \"event_attribute\" && modelEventAttributes.any (fun eventAttribute => eventAttribute.workflow == field.workflow && eventAttribute.slice == field.slice && eventAttribute.event == field.sourceEvent && eventAttribute.attributeName == field.sourceAttribute && modelEventAttributeSourceIsComplete eventAttribute)) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false && field.derivationSourceFields.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false))",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewFieldReadModelFieldSourceResolves"),
            canonical_declaration_marker(
                "def modelViewFieldReadModelFieldSourceResolves (viewField : ModelViewField) : Bool := modelViewFieldSourceIsComplete viewField && modelReadModelFields.any (fun readModelField => readModelField.workflow == viewField.workflow && readModelField.slice == viewField.slice && readModelField.readModel == viewField.sourceReadModel && readModelField.field == viewField.sourceField && modelReadModelFieldSourceIsComplete readModelField)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelDisplayedDatumTracesToOriginalProvenance"),
            canonical_declaration_marker(
                "def modelDisplayedDatumTracesToOriginalProvenance (viewField : ModelViewField) : Bool := modelViewFieldReadModelFieldSourceResolves viewField && modelReadModelFields.any (fun readModelField => readModelField.workflow == viewField.workflow && readModelField.slice == viewField.slice && readModelField.readModel == viewField.sourceReadModel && readModelField.field == viewField.sourceField && modelReadModelFieldTracesToOriginalProvenance readModelField)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelExternalPayloadFieldHasProvenance"),
            canonical_declaration_marker(
                "def modelExternalPayloadFieldHasProvenance (field : ModelExternalPayloadField) : Bool := field.provenance.isEmpty == false && field.bitEncoding.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelControlProvidesCommandInput"),
            canonical_declaration_marker(
                "def modelControlProvidesCommandInput (control : ModelViewControl) (input : ModelCommandInput) : Bool := control.workflow == input.workflow && control.command == input.command && control.input == input.input",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewControlProvidesEveryCommandInput"),
            canonical_declaration_marker(
                "def modelViewControlProvidesEveryCommandInput (control : ModelViewControl) : Bool := modelCommandInputs.all (fun input => input.workflow != control.workflow || input.command != control.command || modelViewControls.any (fun providedInput => providedInput.workflow == control.workflow && providedInput.slice == control.slice && providedInput.view == control.view && providedInput.control == control.control && providedInput.command == control.command && modelControlProvidesCommandInput providedInput input))",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelOutcomeBranchIsModeled"),
            canonical_declaration_marker(
                "def modelOutcomeBranchIsModeled (outcome : ModelOutcome) : Bool := outcome.outcome.isEmpty == false && outcome.events.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelCommandErrorRecoveryIsModeled"),
            canonical_declaration_marker(
                "def modelCommandErrorRecoveryIsModeled (commandError : ModelCommandError) : Bool := commandError.command.isEmpty == false && commandError.error.isEmpty == false && commandError.scenario.isEmpty == false && commandError.recovery.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelViewControlNavigationTargetIsModeled"),
            canonical_declaration_marker(
                "def modelViewControlNavigationTargetIsModeled (control : ModelViewControl) : Bool := control.navigationType.isEmpty || ((control.navigationType == \"modeled_view\" || control.navigationType == \"local_view_state\") && control.navigationTarget.isEmpty == false) || (control.navigationType == \"external_workflow\" && control.externalWorkflow.isEmpty == false) || (control.navigationType == \"external_system\" && control.externalSystem.isEmpty == false && control.handoffContract.isEmpty == false)",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelExternalBoundaryContractIsModeled"),
            canonical_declaration_marker(
                "def modelExternalBoundaryContractIsModeled (translation : ModelTranslationDefinition) : Bool := translation.translation.isEmpty == false && translation.externalEvent.isEmpty == false && translation.payloadContract.isEmpty == false && translation.command.isEmpty == false",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("def modelWorkflowBehaviorSurfaceIsComplete"),
            canonical_declaration_marker(
                "def modelWorkflowBehaviorSurfaceIsComplete : Bool := modelOutcomes.all modelOutcomeBranchIsModeled && modelCommandErrors.all modelCommandErrorRecoveryIsModeled && modelViewControls.all modelViewControlNavigationTargetIsModeled && modelTranslationDefinitions.all modelExternalBoundaryContractIsModeled",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelIdentityIsStable"),
            canonical_declaration_marker(format!(
                "theorem modelIdentityIsStable : modelName = {} := rfl",
                json_string(project_name_text)
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelVersionIsStable"),
            canonical_declaration_marker(format!(
                "theorem modelVersionIsStable : modelVersion = {} := rfl",
                json_string(model_version)
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDigestIsStable"),
            canonical_declaration_marker(format!(
                "theorem modelDigestIsStable : modelDigest = {} := rfl",
                json_string(&model_digest)
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelWorkflowsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelWorkflowsAreDeclared : modelWorkflows.length = {workflow_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelSlicesAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelSlicesAreDeclared : modelSlices.length = {slice_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelSliceModulesAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelSliceModulesAreDeclared : modelSliceModules.length = {slice_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelWorkflowCompositionStructureComplete"),
            canonical_declaration_marker(
                "theorem modelWorkflowCompositionStructureComplete : (modelSlices.all modelSliceBelongsToDeclaredWorkflow && modelSlices.all modelSliceHasModule && modelSliceModules.all modelSliceModuleBelongsToDeclaredSlice && modelWorkflows.all modelWorkflowHasCompositionStructure) = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelWorkflowBehaviorSurfaceIsCompleteIsStable"),
            canonical_declaration_marker(
                "theorem modelWorkflowBehaviorSurfaceIsCompleteIsStable : modelWorkflowBehaviorSurfaceIsComplete = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelScenariosAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelScenariosAreDeclared : modelScenarios.length = {scenario_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelScenarioDefinitionsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelScenarioDefinitionsAreDeclared : modelScenarioDefinitions.length = {scenario_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelScenarioDefinitionsHaveGwt"),
            canonical_declaration_marker(
                "theorem modelScenarioDefinitionsHaveGwt : modelScenarioDefinitions.all modelScenarioDefinitionHasGwt = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelScenarioKindsAreFirstClass"),
            canonical_declaration_marker(
                "theorem modelScenarioKindsAreFirstClass : modelScenarioDefinitions.all modelScenarioKindIsFirstClass = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDataFlowsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelDataFlowsAreDeclared : modelDataFlows.length = {data_flow_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDataFlowsAreBitComplete"),
            canonical_declaration_marker(
                "theorem modelDataFlowsAreBitComplete : modelDataFlows.all modelDataFlowIsBitComplete = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDataFlowSourceKindsAreModeled"),
            canonical_declaration_marker(
                "theorem modelDataFlowSourceKindsAreModeled : modelDataFlows.all modelDataFlowHasModeledSourceKind = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDataFlowModeledSourcesResolve"),
            canonical_declaration_marker(
                "theorem modelDataFlowModeledSourcesResolve : modelDataFlows.all modelDataFlowModeledSourceResolves = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDataFlowSourceChainsReachOriginals"),
            canonical_declaration_marker(
                "theorem modelDataFlowSourceChainsReachOriginals : modelDataFlows.all modelDataFlowHasOriginalSourceChain = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix(
                "theorem modelDataFlowSourceChainsPreserveBitEncodingSemantics",
            ),
            canonical_declaration_marker(
                "theorem modelDataFlowSourceChainsPreserveBitEncodingSemantics : modelDataFlows.all modelDataFlowHasBitPreservingOriginalSourceChain = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDataFlowTransformationsAreModeled"),
            canonical_declaration_marker(
                "theorem modelDataFlowTransformationsAreModeled : modelDataFlows.all modelDataFlowHasModeledTransformationSemantics = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelMeaningfulDataFlowsAreCovered"),
            canonical_declaration_marker(
                "theorem modelMeaningfulDataFlowsAreCovered : modelMeaningfulDataHasModeledDataFlows = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix(
                "theorem modelDataFlowSourceBitEncodingsMatchModeledSources",
            ),
            canonical_declaration_marker(
                "theorem modelDataFlowSourceBitEncodingsMatchModeledSources : modelDataFlows.all modelDataFlowSourceBitEncodingMatchesModeledSource = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewFieldBitEncodingsMatchDataFlows"),
            canonical_declaration_marker(
                "theorem modelViewFieldBitEncodingsMatchDataFlows : modelViewFields.all modelViewFieldBitEncodingMatchesDataFlow = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix(
                "theorem modelExternalPayloadFieldBitEncodingsMatchDataFlows",
            ),
            canonical_declaration_marker(
                "theorem modelExternalPayloadFieldBitEncodingsMatchDataFlows : modelExternalPayloadFields.all modelExternalPayloadFieldBitEncodingMatchesDataFlow = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelOutcomesAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelOutcomesAreDeclared : modelOutcomes.length = {outcome_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelCommandErrorsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelCommandErrorsAreDeclared : modelCommandErrors.length = {command_error_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelCommandsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelCommandsAreDeclared : modelCommands.length = {command_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelCommandInputsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelCommandInputsAreDeclared : modelCommandInputs.length = {command_input_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelCommandInputsHaveProvenance"),
            canonical_declaration_marker(
                "theorem modelCommandInputsHaveProvenance : modelCommandInputs.all modelCommandInputHasProvenance = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelCommandInputsTraceToInvocationSources"),
            canonical_declaration_marker(
                "theorem modelCommandInputsTraceToInvocationSources : modelCommandInputs.all modelCommandInputTracesToInvocationSource = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelEventAttributeSourcesAreComplete"),
            canonical_declaration_marker(
                "theorem modelEventAttributeSourcesAreComplete : modelEventAttributes.all modelEventAttributeSourceIsComplete = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelReadModelsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelReadModelsAreDeclared : modelReadModels.length = {read_model_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelReadModelDefinitionsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelReadModelDefinitionsAreDeclared : modelReadModelDefinitions.length = {read_model_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelReadModelFieldsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelReadModelFieldsAreDeclared : modelReadModelFields.length = {read_model_field_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelReadModelFieldSourcesAreComplete"),
            canonical_declaration_marker(
                "theorem modelReadModelFieldSourcesAreComplete : modelReadModelFields.all modelReadModelFieldSourceIsComplete = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewFieldSourcesAreComplete"),
            canonical_declaration_marker(
                "theorem modelViewFieldSourcesAreComplete : modelViewFields.all modelViewFieldSourceIsComplete = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewFieldReadModelFieldSourcesResolve"),
            canonical_declaration_marker(
                "theorem modelViewFieldReadModelFieldSourcesResolve : modelViewFields.all modelViewFieldReadModelFieldSourceResolves = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelDisplayedDataTraceToOriginalProvenance"),
            canonical_declaration_marker(
                "theorem modelDisplayedDataTraceToOriginalProvenance : modelViewFields.all modelDisplayedDatumTracesToOriginalProvenance = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelViewsAreDeclared : modelViews.length = {view_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewDefinitionsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelViewDefinitionsAreDeclared : modelViewDefinitions.length = {view_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewControlsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelViewControlsAreDeclared : modelViewControls.length = {view_control_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewControlsProvideCommandInputs"),
            canonical_declaration_marker(
                "theorem modelViewControlsProvideCommandInputs : modelViewControls.all modelViewControlProvidesEveryCommandInput = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelBoardElementsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelBoardElementsAreDeclared : modelBoardElements.length = {board_element_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelBoardConnectionsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelBoardConnectionsAreDeclared : modelBoardConnections.length = {board_connection_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelViewFieldsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelViewFieldsAreDeclared : modelViewFields.length = {view_field_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelAutomationsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelAutomationsAreDeclared : modelAutomations.length = {automation_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelAutomationDefinitionsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelAutomationDefinitionsAreDeclared : modelAutomationDefinitions.length = {automation_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelTranslationsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelTranslationsAreDeclared : modelTranslations.length = {translation_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelTranslationDefinitionsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelTranslationDefinitionsAreDeclared : modelTranslationDefinitions.length = {translation_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelExternalPayloadsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelExternalPayloadsAreDeclared : modelExternalPayloads.length = {external_payload_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelExternalPayloadFieldsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelExternalPayloadFieldsAreDeclared : modelExternalPayloadFields.length = {external_payload_field_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelExternalPayloadFieldsHaveProvenance"),
            canonical_declaration_marker(
                "theorem modelExternalPayloadFieldsHaveProvenance : modelExternalPayloadFields.all modelExternalPayloadFieldHasProvenance = true := rfl",
            ),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelStreamsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelStreamsAreDeclared : modelStreams.length = {stream_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelEventsAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelEventsAreDeclared : modelEvents.length = {event_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            canonical_declaration_prefix("theorem modelEventAttributesAreDeclared"),
            canonical_declaration_marker(format!(
                "theorem modelEventAttributesAreDeclared : modelEventAttributes.length = {event_attribute_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::require_canonical_declaration(
            lean_path,
            canonical_declaration_prefix("end "),
            canonical_declaration_marker(format!("end {module_name}")),
            lean_message,
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("module "),
            canonical_declaration_marker(format!("module {module_name} {{")),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelSlice ="),
            canonical_declaration_marker("  type ModelSlice = { workflow: str, slice: str }"),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelSliceModule ="),
            canonical_declaration_marker(
                "  type ModelSliceModule = { workflow: str, slice: str, formalModule: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelScenario ="),
            canonical_declaration_marker(
                "  type ModelScenario = { workflow: str, slice: str, scenarioKind: str, scenario: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelScenarioDefinition ="),
            canonical_declaration_marker(
                "  type ModelScenarioDefinition = { workflow: str, slice: str, scenarioKind: str, scenario: str, given: str, when: str, then: str, readStreams: List[str], writtenStreams: List[str], contractKind: str, coveredDefinition: str, errorReferences: List[str] }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelDataFlow ="),
            canonical_declaration_marker(
                "  type ModelDataFlow = { workflow: str, slice: str, datum: str, sourceKind: str, source: str, transformation: str, target: str, bitEncoding: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelOutcome ="),
            canonical_declaration_marker(
                "  type ModelOutcome = { workflow: str, slice: str, outcome: str, events: List[str], externallyRelevant: bool }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelCommandError ="),
            canonical_declaration_marker(
                "  type ModelCommandError = { workflow: str, slice: str, command: str, error: str, scenario: str, recovery: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelCommand ="),
            canonical_declaration_marker(
                "  type ModelCommand = { workflow: str, slice: str, command: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelCommandInput ="),
            canonical_declaration_marker(
                "  type ModelCommandInput = { workflow: str, slice: str, command: str, input: str, sourceKind: str, sourceDescription: str, provenanceChain: List[str], eventStreamSourceEvent: str, eventStreamSourceAttribute: str, externalPayloadSourceName: str, externalPayloadSourceField: str, generatedSourceName: str, generatedSourceField: str, sessionSourceName: str, sessionSourceField: str, invocationArgumentSourceName: str, invocationArgumentSourceField: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelReadModel ="),
            canonical_declaration_marker(
                "  type ModelReadModel = { workflow: str, slice: str, readModel: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelReadModelDefinition ="),
            canonical_declaration_marker(
                "  type ModelReadModelDefinition = { workflow: str, slice: str, readModel: str, transitive: bool, relationshipFields: List[str], transitiveRule: str, exampleScenarioName: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelReadModelField ="),
            canonical_declaration_marker(
                "  type ModelReadModelField = { workflow: str, slice: str, readModel: str, field: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, derivationSourceFields: List[str], absenceEvent: str, derivationScenarioName: str, absenceScenarioName: str, provenance: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelView ="),
            canonical_declaration_marker(
                "  type ModelView = { workflow: str, slice: str, view: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelViewDefinition ="),
            canonical_declaration_marker(
                "  type ModelViewDefinition = { workflow: str, slice: str, view: str, readModels: List[str], sketchTokens: List[str], localStates: List[str], filters: List[str] }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelViewControl ="),
            canonical_declaration_marker(
                "  type ModelViewControl = { workflow: str, slice: str, view: str, control: str, command: str, input: str, inputSourceKind: str, inputSourceDescription: str, inputSketchToken: str, inputVisibleToActor: bool, inputDecisionField: bool, handledErrors: List[str], recoveryBehavior: str, controlSketchToken: str, navigationType: str, navigationTarget: str, externalWorkflow: str, externalSystem: str, handoffContract: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelBoardElement ="),
            canonical_declaration_marker(
                "  type ModelBoardElement = { workflow: str, slice: str, element: str, kind: str, lane: str, declaredName: str, mainPath: bool }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelBoardConnection ="),
            canonical_declaration_marker(
                "  type ModelBoardConnection = { workflow: str, slice: str, source: str, sourceKind: str, target: str, targetKind: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelViewField ="),
            canonical_declaration_marker(
                "  type ModelViewField = { workflow: str, slice: str, view: str, field: str, sourceKind: str, sourceReadModel: str, sourceField: str, provenance: str, bitEncoding: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelAutomation ="),
            canonical_declaration_marker(
                "  type ModelAutomation = { workflow: str, slice: str, automation: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelAutomationDefinition ="),
            canonical_declaration_marker(
                "  type ModelAutomationDefinition = { workflow: str, slice: str, automation: str, trigger: str, command: str, handledErrors: List[str], reaction: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelTranslation ="),
            canonical_declaration_marker(
                "  type ModelTranslation = { workflow: str, slice: str, translation: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelTranslationDefinition ="),
            canonical_declaration_marker(
                "  type ModelTranslationDefinition = { workflow: str, slice: str, translation: str, externalEvent: str, payloadContract: str, command: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelExternalPayload ="),
            canonical_declaration_marker(
                "  type ModelExternalPayload = { workflow: str, slice: str, externalPayload: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelExternalPayloadField ="),
            canonical_declaration_marker(
                "  type ModelExternalPayloadField = { workflow: str, slice: str, externalPayload: str, field: str, provenance: str, bitEncoding: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelStream ="),
            canonical_declaration_marker(
                "  type ModelStream = { workflow: str, slice: str, stream: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelEvent ="),
            canonical_declaration_marker(
                "  type ModelEvent = { workflow: str, slice: str, event: str, stream: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  type ModelEventAttribute ="),
            canonical_declaration_marker(
                "  type ModelEventAttribute = { workflow: str, slice: str, event: str, attribute: str, sourceKind: str, sourceName: str, sourceField: str, generatedSourceKind: str, provenance: str }",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelVersion ="),
            canonical_declaration_marker(format!(
                "  val modelVersion = {}",
                json_string(model_version)
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelName ="),
            canonical_declaration_marker(format!(
                "  val modelName = {}",
                json_string(project_name_text)
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDigest ="),
            canonical_declaration_marker(format!(
                "  val modelDigest = {}",
                json_string(&model_digest)
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelWorkflows:"),
            canonical_declaration_marker(format!(
                "  val modelWorkflows: List[str] = {workflow_slug_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelSlices:"),
            canonical_declaration_marker(format!(
                "  val modelSlices: List[ModelSlice] = {quint_model_slice_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelSliceModules:"),
            canonical_declaration_marker(format!(
                "  val modelSliceModules: List[ModelSliceModule] = {quint_model_slice_module_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelScenarios:"),
            canonical_declaration_marker(format!(
                "  val modelScenarios: List[ModelScenario] = {quint_model_scenario_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelScenarioDefinitions:"),
            canonical_declaration_marker(format!(
                "  val modelScenarioDefinitions: List[ModelScenarioDefinition] = {quint_model_scenario_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlows:"),
            canonical_declaration_marker(format!(
                "  val modelDataFlows: List[ModelDataFlow] = {quint_model_data_flow_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowCount ="),
            canonical_declaration_marker(format!("  val modelDataFlowCount = {data_flow_count}")),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelOutcomes:"),
            canonical_declaration_marker(format!(
                "  val modelOutcomes: List[ModelOutcome] = {quint_model_outcome_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommandErrors:"),
            canonical_declaration_marker(format!(
                "  val modelCommandErrors: List[ModelCommandError] = {quint_model_command_error_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommands:"),
            canonical_declaration_marker(format!(
                "  val modelCommands: List[ModelCommand] = {quint_model_command_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommandInputs:"),
            canonical_declaration_marker(format!(
                "  val modelCommandInputs: List[ModelCommandInput] = {quint_model_command_input_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelReadModels:"),
            canonical_declaration_marker(format!(
                "  val modelReadModels: List[ModelReadModel] = {quint_model_read_model_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelReadModelDefinitions:"),
            canonical_declaration_marker(format!(
                "  val modelReadModelDefinitions: List[ModelReadModelDefinition] = {quint_model_read_model_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelReadModelFields:"),
            canonical_declaration_marker(format!(
                "  val modelReadModelFields: List[ModelReadModelField] = {quint_model_read_model_field_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViews:"),
            canonical_declaration_marker(format!(
                "  val modelViews: List[ModelView] = {quint_model_view_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewDefinitions:"),
            canonical_declaration_marker(format!(
                "  val modelViewDefinitions: List[ModelViewDefinition] = {quint_model_view_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewControls:"),
            canonical_declaration_marker(format!(
                "  val modelViewControls: List[ModelViewControl] = {quint_model_view_control_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelBoardElements:"),
            canonical_declaration_marker(format!(
                "  val modelBoardElements: List[ModelBoardElement] = {quint_model_board_element_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelBoardConnections:"),
            canonical_declaration_marker(format!(
                "  val modelBoardConnections: List[ModelBoardConnection] = {quint_model_board_connection_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewFields:"),
            canonical_declaration_marker(format!(
                "  val modelViewFields: List[ModelViewField] = {quint_model_view_field_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelAutomations:"),
            canonical_declaration_marker(format!(
                "  val modelAutomations: List[ModelAutomation] = {quint_model_automation_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelAutomationDefinitions:"),
            canonical_declaration_marker(format!(
                "  val modelAutomationDefinitions: List[ModelAutomationDefinition] = {quint_model_automation_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelTranslations:"),
            canonical_declaration_marker(format!(
                "  val modelTranslations: List[ModelTranslation] = {quint_model_translation_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelTranslationDefinitions:"),
            canonical_declaration_marker(format!(
                "  val modelTranslationDefinitions: List[ModelTranslationDefinition] = {quint_model_translation_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelExternalPayloads:"),
            canonical_declaration_marker(format!(
                "  val modelExternalPayloads: List[ModelExternalPayload] = {quint_model_external_payload_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelExternalPayloadFields:"),
            canonical_declaration_marker(format!(
                "  val modelExternalPayloadFields: List[ModelExternalPayloadField] = {quint_model_external_payload_field_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelStreams:"),
            canonical_declaration_marker(format!(
                "  val modelStreams: List[ModelStream] = {quint_model_stream_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelEvents:"),
            canonical_declaration_marker(format!(
                "  val modelEvents: List[ModelEvent] = {quint_model_event_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelEventAttributes:"),
            canonical_declaration_marker(format!(
                "  val modelEventAttributes: List[ModelEventAttribute] = {quint_model_event_attribute_list}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelIdentityStable ="),
            canonical_declaration_marker(format!(
                "  val modelIdentityStable = modelName == {}",
                json_string(project_name_text)
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelVersionStable ="),
            canonical_declaration_marker(format!(
                "  val modelVersionStable = modelVersion == {}",
                json_string(model_version)
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDigestStable ="),
            canonical_declaration_marker(format!(
                "  val modelDigestStable = modelDigest == {}",
                json_string(&model_digest)
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelWorkflowsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelWorkflowsAreDeclared = modelWorkflows.length() == {workflow_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelSlicesAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelSlicesAreDeclared = modelSlices.length() == {slice_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelSliceModulesAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelSliceModulesAreDeclared = modelSliceModules.length() == {slice_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelSliceBelongsToDeclaredWorkflow"),
            canonical_declaration_marker(
                "  def modelSliceBelongsToDeclaredWorkflow(modelSlice) = modelWorkflows.select(workflow => workflow == modelSlice.workflow).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelSliceHasModule"),
            canonical_declaration_marker(
                "  def modelSliceHasModule(modelSlice) = modelSliceModules.select(sliceModule => sliceModule.workflow == modelSlice.workflow and sliceModule.slice == modelSlice.slice and sliceModule.formalModule != \"\").length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelSliceModuleBelongsToDeclaredSlice"),
            canonical_declaration_marker(
                "  def modelSliceModuleBelongsToDeclaredSlice(sliceModule) = sliceModule.formalModule != \"\" and modelSlices.select(modelSlice => modelSlice.workflow == sliceModule.workflow and modelSlice.slice == sliceModule.slice).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelWorkflowSlicesHaveModules"),
            canonical_declaration_marker(
                "  def modelWorkflowSlicesHaveModules(workflow) = modelSlices.select(modelSlice => modelSlice.workflow == workflow and not(modelSliceHasModule(modelSlice))).length() == 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelWorkflowHasCompositionStructure"),
            canonical_declaration_marker(
                "  def modelWorkflowHasCompositionStructure(workflow) = modelWorkflowSlicesHaveModules(workflow)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelWorkflowCompositionStructureComplete ="),
            canonical_declaration_marker(
                "  val modelWorkflowCompositionStructureComplete = modelSlices.select(modelSlice => modelSliceBelongsToDeclaredWorkflow(modelSlice)).length() == modelSlices.length() and modelSlices.select(modelSlice => modelSliceHasModule(modelSlice)).length() == modelSlices.length() and modelSliceModules.select(sliceModule => modelSliceModuleBelongsToDeclaredSlice(sliceModule)).length() == modelSliceModules.length() and modelWorkflows.select(workflow => modelWorkflowHasCompositionStructure(workflow)).length() == modelWorkflows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelScenariosAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelScenariosAreDeclared = modelScenarios.length() == {scenario_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelScenarioDefinitionsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelScenarioDefinitionsAreDeclared = modelScenarioDefinitions.length() == {scenario_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelScenarioDefinitionHasGwt"),
            canonical_declaration_marker(
                "  def modelScenarioDefinitionHasGwt(scenario) = scenario.given != \"\" and scenario.when != \"\" and scenario.then != \"\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelScenarioKindIsFirstClass"),
            canonical_declaration_marker(
                "  def modelScenarioKindIsFirstClass(scenario) = scenario.scenarioKind == \"acceptance\" or scenario.scenarioKind == \"contract\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelScenarioDefinitionsHaveGwt ="),
            canonical_declaration_marker(
                "  val modelScenarioDefinitionsHaveGwt = modelScenarioDefinitions.select(scenario => modelScenarioDefinitionHasGwt(scenario)).length() == modelScenarioDefinitions.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelScenarioKindsAreFirstClass ="),
            canonical_declaration_marker(
                "  val modelScenarioKindsAreFirstClass = modelScenarioDefinitions.select(scenario => modelScenarioKindIsFirstClass(scenario)).length() == modelScenarioDefinitions.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowIsBitComplete"),
            canonical_declaration_marker(
                "  def modelDataFlowIsBitComplete(dataFlow) = dataFlow.datum != \"\" and dataFlow.sourceKind != \"\" and dataFlow.source != \"\" and dataFlow.transformation != \"\" and dataFlow.target != \"\" and dataFlow.bitEncoding != \"\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowCoversDatumTarget"),
            canonical_declaration_marker(
                "  def modelDataFlowCoversDatumTarget(workflow, sliceName, datum, target) = modelDataFlows.select(dataFlow => dataFlow.workflow == workflow and dataFlow.slice == sliceName and dataFlow.datum == datum and dataFlow.target == target and modelDataFlowIsBitComplete(dataFlow)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowBitEncodingMatchesDatumTarget"),
            canonical_declaration_marker(
                "  def modelDataFlowBitEncodingMatchesDatumTarget(workflow, sliceName, datum, target, bitEncoding) = modelDataFlows.select(dataFlow => dataFlow.workflow == workflow and dataFlow.slice == sliceName and dataFlow.datum == datum and dataFlow.target == target and dataFlow.bitEncoding == bitEncoding and modelDataFlowIsBitComplete(dataFlow)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix(
                "  def modelDataFlowSourceBitEncodingMatchesModeledSource",
            ),
            canonical_declaration_marker(
                "  def modelDataFlowSourceBitEncodingMatchesModeledSource(dataFlow) = modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source).length() == 0 or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and sourceFlow.bitEncoding == dataFlow.bitEncoding and modelDataFlowIsBitComplete(sourceFlow)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowHasModeledTransformationSemantics"),
            canonical_declaration_marker(
                "  def modelDataFlowHasModeledTransformationSemantics(dataFlow) = dataFlow.transformation == \"identity\" or dataFlow.transformation == \"projection\" or dataFlow.transformation == \"derivation\" or dataFlow.transformation == \"default\" or dataFlow.transformation == \"absence\" or dataFlow.transformation == \"transformation\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowHasModeledSourceKind"),
            canonical_declaration_marker(
                "  def modelDataFlowHasModeledSourceKind(dataFlow) = (dataFlow.sourceKind == \"original\" and dataFlow.source != \"\") or (dataFlow.sourceKind == \"modeled_target\" and dataFlow.source != \"\")",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowModeledSourceResolves"),
            canonical_declaration_marker(
                "  def modelDataFlowModeledSourceResolves(dataFlow) = dataFlow.sourceKind != \"modeled_target\" or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and modelDataFlowIsBitComplete(sourceFlow)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelSameDataFlowTarget"),
            canonical_declaration_marker(
                "  def modelSameDataFlowTarget(left, right) = left.workflow == right.workflow and left.slice == right.slice and left.datum == right.datum and left.target == right.target",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowTargetsFromReachable"),
            canonical_declaration_marker(
                "  def modelDataFlowTargetsFromReachable(reachable) = modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"modeled_target\" and reachable.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and modelDataFlowIsBitComplete(sourceFlow)).length() > 0)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowsReachableFromOriginalsAfterFuel"),
            canonical_declaration_marker(
                "  def modelDataFlowsReachableFromOriginalsAfterFuel(fuel, reachable) = range(0, fuel).foldl(reachable, (currentReachable, _) => currentReachable.concat(modelDataFlowTargetsFromReachable(currentReachable)))",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowsReachableFromOriginals ="),
            canonical_declaration_marker(
                "  val modelDataFlowsReachableFromOriginals = modelDataFlowsReachableFromOriginalsAfterFuel(modelDataFlowCount, modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"original\" and modelDataFlowIsBitComplete(dataFlow)))",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowHasOriginalSourceChain"),
            canonical_declaration_marker(
                "  def modelDataFlowHasOriginalSourceChain(dataFlow) = dataFlow.sourceKind == \"original\" or modelDataFlowsReachableFromOriginals.select(reachableFlow => modelSameDataFlowTarget(reachableFlow, dataFlow)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowTargetsFromBitPreservingReachable"),
            canonical_declaration_marker(
                "  def modelDataFlowTargetsFromBitPreservingReachable(reachable) = modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"modeled_target\" and reachable.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and sourceFlow.bitEncoding == dataFlow.bitEncoding and modelDataFlowIsBitComplete(sourceFlow)).length() > 0)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix(
                "  def modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel",
            ),
            canonical_declaration_marker(
                "  def modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel(fuel, reachable) = range(0, fuel).foldl(reachable, (currentReachable, _) => currentReachable.concat(modelDataFlowTargetsFromBitPreservingReachable(currentReachable)))",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix(
                "  val modelDataFlowsReachableFromOriginalsWithPreservedBits",
            ),
            canonical_declaration_marker(
                "  val modelDataFlowsReachableFromOriginalsWithPreservedBits = modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel(modelDataFlowCount, modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"original\" and modelDataFlowIsBitComplete(dataFlow)))",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDataFlowHasBitPreservingOriginalSourceChain"),
            canonical_declaration_marker(
                "  def modelDataFlowHasBitPreservingOriginalSourceChain(dataFlow) = dataFlow.sourceKind == \"original\" or modelDataFlowsReachableFromOriginalsWithPreservedBits.select(reachableFlow => modelSameDataFlowTarget(reachableFlow, dataFlow)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelCommandInputHasModeledDataFlow"),
            canonical_declaration_marker(
                "  def modelCommandInputHasModeledDataFlow(input) = modelDataFlowCoversDatumTarget(input.workflow, input.slice, input.input, input.command)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelEventAttributeHasModeledDataFlow"),
            canonical_declaration_marker(
                "  def modelEventAttributeHasModeledDataFlow(eventAttr) = modelDataFlowCoversDatumTarget(eventAttr.workflow, eventAttr.slice, eventAttr.attribute, eventAttr.event)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelReadModelFieldHasModeledDataFlow"),
            canonical_declaration_marker(
                "  def modelReadModelFieldHasModeledDataFlow(readModelField) = modelDataFlowCoversDatumTarget(readModelField.workflow, readModelField.slice, readModelField.field, readModelField.readModel)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelViewFieldHasModeledDataFlow"),
            canonical_declaration_marker(
                "  def modelViewFieldHasModeledDataFlow(viewField) = modelDataFlowCoversDatumTarget(viewField.workflow, viewField.slice, viewField.field, viewField.view)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelViewFieldBitEncodingMatchesDataFlow"),
            canonical_declaration_marker(
                "  def modelViewFieldBitEncodingMatchesDataFlow(viewField) = modelDataFlowBitEncodingMatchesDatumTarget(viewField.workflow, viewField.slice, viewField.field, viewField.view, viewField.bitEncoding)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelExternalPayloadFieldHasModeledDataFlow"),
            canonical_declaration_marker(
                "  def modelExternalPayloadFieldHasModeledDataFlow(externalPayloadField) = modelDataFlowCoversDatumTarget(externalPayloadField.workflow, externalPayloadField.slice, externalPayloadField.field, externalPayloadField.externalPayload)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix(
                "  def modelExternalPayloadFieldBitEncodingMatchesDataFlow",
            ),
            canonical_declaration_marker(
                "  def modelExternalPayloadFieldBitEncodingMatchesDataFlow(externalPayloadField) = modelDataFlowBitEncodingMatchesDatumTarget(externalPayloadField.workflow, externalPayloadField.slice, externalPayloadField.field, externalPayloadField.externalPayload, externalPayloadField.bitEncoding)",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelDataFlowsAreDeclared = modelDataFlows.length() == {data_flow_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowsAreBitComplete ="),
            canonical_declaration_marker(
                "  val modelDataFlowsAreBitComplete = modelDataFlows.select(dataFlow => modelDataFlowIsBitComplete(dataFlow)).length() == modelDataFlows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowSourceKindsAreModeled ="),
            canonical_declaration_marker(
                "  val modelDataFlowSourceKindsAreModeled = modelDataFlows.select(dataFlow => modelDataFlowHasModeledSourceKind(dataFlow)).length() == modelDataFlows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowModeledSourcesResolve ="),
            canonical_declaration_marker(
                "  val modelDataFlowModeledSourcesResolve = modelDataFlows.select(dataFlow => modelDataFlowModeledSourceResolves(dataFlow)).length() == modelDataFlows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowSourceChainsReachOriginals ="),
            canonical_declaration_marker(
                "  val modelDataFlowSourceChainsReachOriginals = modelDataFlows.select(dataFlow => modelDataFlowHasOriginalSourceChain(dataFlow)).length() == modelDataFlows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix(
                "  val modelDataFlowSourceChainsPreserveBitEncodingSemantics =",
            ),
            canonical_declaration_marker(
                "  val modelDataFlowSourceChainsPreserveBitEncodingSemantics = modelDataFlows.select(dataFlow => modelDataFlowHasBitPreservingOriginalSourceChain(dataFlow)).length() == modelDataFlows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDataFlowTransformationsAreModeled ="),
            canonical_declaration_marker(
                "  val modelDataFlowTransformationsAreModeled = modelDataFlows.select(dataFlow => modelDataFlowHasModeledTransformationSemantics(dataFlow)).length() == modelDataFlows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelMeaningfulDataHasModeledDataFlows ="),
            canonical_declaration_marker(
                "  val modelMeaningfulDataHasModeledDataFlows = modelCommandInputs.select(input => modelCommandInputHasModeledDataFlow(input)).length() == modelCommandInputs.length() and modelEventAttributes.select(eventAttr => modelEventAttributeHasModeledDataFlow(eventAttr)).length() == modelEventAttributes.length() and modelReadModelFields.select(readModelField => modelReadModelFieldHasModeledDataFlow(readModelField)).length() == modelReadModelFields.length() and modelViewFields.select(viewField => modelViewFieldHasModeledDataFlow(viewField)).length() == modelViewFields.length() and modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldHasModeledDataFlow(externalPayloadField)).length() == modelExternalPayloadFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelMeaningfulDataFlowsAreCovered ="),
            canonical_declaration_marker(
                "  val modelMeaningfulDataFlowsAreCovered = modelMeaningfulDataHasModeledDataFlows",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix(
                "  val modelDataFlowSourceBitEncodingsMatchModeledSources =",
            ),
            canonical_declaration_marker(
                "  val modelDataFlowSourceBitEncodingsMatchModeledSources = modelDataFlows.select(dataFlow => modelDataFlowSourceBitEncodingMatchesModeledSource(dataFlow)).length() == modelDataFlows.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewFieldBitEncodingsMatchDataFlows ="),
            canonical_declaration_marker(
                "  val modelViewFieldBitEncodingsMatchDataFlows = modelViewFields.select(viewField => modelViewFieldBitEncodingMatchesDataFlow(viewField)).length() == modelViewFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix(
                "  val modelExternalPayloadFieldBitEncodingsMatchDataFlows =",
            ),
            canonical_declaration_marker(
                "  val modelExternalPayloadFieldBitEncodingsMatchDataFlows = modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldBitEncodingMatchesDataFlow(externalPayloadField)).length() == modelExternalPayloadFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelOutcomesAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelOutcomesAreDeclared = modelOutcomes.length() == {outcome_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommandErrorsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelCommandErrorsAreDeclared = modelCommandErrors.length() == {command_error_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommandsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelCommandsAreDeclared = modelCommands.length() == {command_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommandInputsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelCommandInputsAreDeclared = modelCommandInputs.length() == {command_input_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelCommandInputHasProvenance"),
            canonical_declaration_marker(
                "  def modelCommandInputHasProvenance(input) = input.sourceDescription != \"\" and input.provenanceChain.length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelCommandInputTracesToInvocationSource"),
            canonical_declaration_marker(
                "  def modelCommandInputTracesToInvocationSource(input) = input.sourceKind == \"actor\" or (input.sourceKind == \"event_stream_state\" and input.eventStreamSourceEvent != \"\" and input.eventStreamSourceAttribute != \"\") or (input.sourceKind == \"external_payload\" and input.externalPayloadSourceName != \"\" and input.externalPayloadSourceField != \"\") or (input.sourceKind == \"generated\" and input.generatedSourceName != \"\" and input.generatedSourceField != \"\") or (input.sourceKind == \"session\" and input.sessionSourceName != \"\" and input.sessionSourceField != \"\") or (input.sourceKind == \"invocation_argument\" and input.invocationArgumentSourceName != \"\" and input.invocationArgumentSourceField != \"\")",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommandInputsHaveProvenance ="),
            canonical_declaration_marker(
                "  val modelCommandInputsHaveProvenance = modelCommandInputs.select(input => modelCommandInputHasProvenance(input)).length() == modelCommandInputs.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelCommandInputsTraceToInvocationSources ="),
            canonical_declaration_marker(
                "  val modelCommandInputsTraceToInvocationSources = modelCommandInputs.select(input => modelCommandInputTracesToInvocationSource(input)).length() == modelCommandInputs.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelEventAttributeSourceIsComplete"),
            canonical_declaration_marker(
                "  def modelEventAttributeSourceIsComplete(eventAttr) = eventAttr.provenance != \"\" and ((eventAttr.sourceKind == \"command_input\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\") or (eventAttr.sourceKind == \"external_payload\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\") or (eventAttr.sourceKind == \"generated\" and eventAttr.sourceName != \"\" and eventAttr.generatedSourceKind != \"\") or (eventAttr.sourceKind == \"session\" and eventAttr.sourceName != \"\") or (eventAttr.sourceKind == \"derivation\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\"))",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelEventAttributeSourcesAreComplete ="),
            canonical_declaration_marker(
                "  val modelEventAttributeSourcesAreComplete = modelEventAttributes.select(eventAttr => modelEventAttributeSourceIsComplete(eventAttr)).length() == modelEventAttributes.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelReadModelFieldSourceIsComplete"),
            canonical_declaration_marker(
                "  def modelReadModelFieldSourceIsComplete(readModelField) = (readModelField.sourceKind == \"event_attribute\" and readModelField.sourceEvent != \"\" and readModelField.sourceAttribute != \"\") or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\" and readModelField.derivationSourceFields.length() > 0) or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\")",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelReadModelFieldTracesToOriginalProvenance"),
            canonical_declaration_marker(
                "  def modelReadModelFieldTracesToOriginalProvenance(readModelField) = readModelField.provenance != \"\" and ((readModelField.sourceKind == \"event_attribute\" and modelEventAttributes.select(eventAttr => eventAttr.workflow == readModelField.workflow and eventAttr.slice == readModelField.slice and eventAttr.event == readModelField.sourceEvent and eventAttr.attribute == readModelField.sourceAttribute and modelEventAttributeSourceIsComplete(eventAttr)).length() > 0) or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\" and readModelField.derivationSourceFields.length() > 0) or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\"))",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelReadModelsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelReadModelsAreDeclared = modelReadModels.length() == {read_model_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelReadModelDefinitionsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelReadModelDefinitionsAreDeclared = modelReadModelDefinitions.length() == {read_model_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelReadModelFieldSourcesAreComplete ="),
            canonical_declaration_marker(
                "  val modelReadModelFieldSourcesAreComplete = modelReadModelFields.select(readModelField => modelReadModelFieldSourceIsComplete(readModelField)).length() == modelReadModelFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelViewFieldSourceIsComplete"),
            canonical_declaration_marker(
                "  def modelViewFieldSourceIsComplete(viewField) = viewField.sourceKind == \"read_model\" and viewField.sourceReadModel != \"\" and viewField.sourceField != \"\" and viewField.provenance != \"\" and viewField.bitEncoding != \"\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelViewFieldReadModelFieldSourceResolves"),
            canonical_declaration_marker(
                "  def modelViewFieldReadModelFieldSourceResolves(viewField) = modelViewFieldSourceIsComplete(viewField) and modelReadModelFields.select(readModelField => readModelField.workflow == viewField.workflow and readModelField.slice == viewField.slice and readModelField.readModel == viewField.sourceReadModel and readModelField.field == viewField.sourceField and modelReadModelFieldSourceIsComplete(readModelField)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewFieldReadModelFieldSourcesResolve ="),
            canonical_declaration_marker(
                "  val modelViewFieldReadModelFieldSourcesResolve = modelViewFields.select(viewField => modelViewFieldReadModelFieldSourceResolves(viewField)).length() == modelViewFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelDisplayedDatumTracesToOriginalProvenance"),
            canonical_declaration_marker(
                "  def modelDisplayedDatumTracesToOriginalProvenance(viewField) = modelViewFieldReadModelFieldSourceResolves(viewField) and modelReadModelFields.select(readModelField => readModelField.workflow == viewField.workflow and readModelField.slice == viewField.slice and readModelField.readModel == viewField.sourceReadModel and readModelField.field == viewField.sourceField and modelReadModelFieldTracesToOriginalProvenance(readModelField)).length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelDisplayedDataTraceToOriginalProvenance ="),
            canonical_declaration_marker(
                "  val modelDisplayedDataTraceToOriginalProvenance = modelViewFields.select(viewField => modelDisplayedDatumTracesToOriginalProvenance(viewField)).length() == modelViewFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewFieldSourcesAreComplete ="),
            canonical_declaration_marker(
                "  val modelViewFieldSourcesAreComplete = modelViewFields.select(viewField => modelViewFieldSourceIsComplete(viewField)).length() == modelViewFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelExternalPayloadFieldHasProvenance"),
            canonical_declaration_marker(
                "  def modelExternalPayloadFieldHasProvenance(externalPayloadField) = externalPayloadField.provenance != \"\" and externalPayloadField.bitEncoding != \"\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelExternalPayloadFieldsHaveProvenance ="),
            canonical_declaration_marker(
                "  val modelExternalPayloadFieldsHaveProvenance = modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldHasProvenance(externalPayloadField)).length() == modelExternalPayloadFields.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelReadModelFieldsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == {read_model_field_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelViewsAreDeclared = modelViews.length() == {view_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewDefinitionsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelViewDefinitionsAreDeclared = modelViewDefinitions.length() == {view_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewControlsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelViewControlsAreDeclared = modelViewControls.length() == {view_control_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelControlProvidesCommandInput"),
            canonical_declaration_marker(
                "  def modelControlProvidesCommandInput(control, input) = control.workflow == input.workflow and control.command == input.command and control.input == input.input",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelViewControlProvidesEveryCommandInput"),
            canonical_declaration_marker(
                "  def modelViewControlProvidesEveryCommandInput(control) = modelCommandInputs.select(input => input.workflow != control.workflow or input.command != control.command or modelViewControls.select(providedInput => providedInput.workflow == control.workflow and providedInput.slice == control.slice and providedInput.view == control.view and providedInput.control == control.control and providedInput.command == control.command and modelControlProvidesCommandInput(providedInput, input)).length() > 0).length() == modelCommandInputs.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewControlsProvideCommandInputs ="),
            canonical_declaration_marker(
                "  val modelViewControlsProvideCommandInputs = modelViewControls.select(control => modelViewControlProvidesEveryCommandInput(control)).length() == modelViewControls.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelOutcomeBranchIsModeled"),
            canonical_declaration_marker(
                "  def modelOutcomeBranchIsModeled(outcome) = outcome.outcome != \"\" and outcome.events.length() > 0",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelCommandErrorRecoveryIsModeled"),
            canonical_declaration_marker(
                "  def modelCommandErrorRecoveryIsModeled(commandError) = commandError.command != \"\" and commandError.error != \"\" and commandError.scenario != \"\" and commandError.recovery != \"\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelViewControlNavigationTargetIsModeled"),
            canonical_declaration_marker(
                "  def modelViewControlNavigationTargetIsModeled(control) = control.navigationType == \"\" or ((control.navigationType == \"modeled_view\" or control.navigationType == \"local_view_state\") and control.navigationTarget != \"\") or (control.navigationType == \"external_workflow\" and control.externalWorkflow != \"\") or (control.navigationType == \"external_system\" and control.externalSystem != \"\" and control.handoffContract != \"\")",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  def modelExternalBoundaryContractIsModeled"),
            canonical_declaration_marker(
                "  def modelExternalBoundaryContractIsModeled(translation) = translation.translation != \"\" and translation.externalEvent != \"\" and translation.payloadContract != \"\" and translation.command != \"\"",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelWorkflowBehaviorSurfaceIsComplete ="),
            canonical_declaration_marker(
                "  val modelWorkflowBehaviorSurfaceIsComplete = modelOutcomes.select(outcome => modelOutcomeBranchIsModeled(outcome)).length() == modelOutcomes.length() and modelCommandErrors.select(commandError => modelCommandErrorRecoveryIsModeled(commandError)).length() == modelCommandErrors.length() and modelViewControls.select(control => modelViewControlNavigationTargetIsModeled(control)).length() == modelViewControls.length() and modelTranslationDefinitions.select(translation => modelExternalBoundaryContractIsModeled(translation)).length() == modelTranslationDefinitions.length()",
            ),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelBoardElementsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelBoardElementsAreDeclared = modelBoardElements.length() == {board_element_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelBoardConnectionsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelBoardConnectionsAreDeclared = modelBoardConnections.length() == {board_connection_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelViewFieldsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelViewFieldsAreDeclared = modelViewFields.length() == {view_field_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelAutomationsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelAutomationsAreDeclared = modelAutomations.length() == {automation_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelAutomationDefinitionsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelAutomationDefinitionsAreDeclared = modelAutomationDefinitions.length() == {automation_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelTranslationsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelTranslationsAreDeclared = modelTranslations.length() == {translation_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelTranslationDefinitionsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelTranslationDefinitionsAreDeclared = modelTranslationDefinitions.length() == {translation_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelExternalPayloadsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelExternalPayloadsAreDeclared = modelExternalPayloads.length() == {external_payload_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelExternalPayloadFieldsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelExternalPayloadFieldsAreDeclared = modelExternalPayloadFields.length() == {external_payload_field_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelStreamsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelStreamsAreDeclared = modelStreams.length() == {stream_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelEventsAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelEventsAreDeclared = modelEvents.length() == {event_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            canonical_declaration_prefix("  val modelEventAttributesAreDeclared ="),
            canonical_declaration_marker(format!(
                "  val modelEventAttributesAreDeclared = modelEventAttributes.length() == {event_attribute_count}"
            )),
            quint_message.clone(),
        ),
        Effect::require_canonical_declaration(
            quint_path,
            quint_module_close_prefix,
            quint_module_close_marker,
            quint_message,
        ),
    ]
}

fn modeled_artifact_paths<const N: usize>(
    required_paths: [ProjectPath; N],
    modeled_workflows: &[ModeledWorkflowLayout],
    workflow_path: fn(&ModeledWorkflowLayout) -> ProjectPath,
) -> Vec<ProjectPath> {
    required_paths
        .into_iter()
        .chain(modeled_workflows.iter().map(workflow_path))
        .collect()
}

pub(crate) fn list_workflows(modeled_workflows: ModeledWorkflowLayouts) -> EffectPlan {
    EffectPlan::new(
        modeled_workflows
            .into_inner()
            .into_iter()
            .map(|workflow| Effect::Report(report_line(workflow.name.as_ref().to_owned())))
            .collect(),
    )
}

pub(crate) fn list_slices(modeled_slices: ModeledWorkflowSliceDetails) -> EffectPlan {
    EffectPlan::new(
        modeled_slices
            .slices
            .into_iter()
            .map(|slice| Effect::Report(report_line(slice.name().as_ref().to_owned())))
            .collect(),
    )
}

pub(crate) fn list_transitions(modeled_transitions: ModeledWorkflowTransitions) -> EffectPlan {
    EffectPlan::new(
        modeled_transitions
            .transitions
            .into_iter()
            .map(|transition| {
                Effect::Report(report_line(format!(
                    "{} -> {} [{} {}]",
                    transition.source().as_ref(),
                    transition.target().as_ref(),
                    transition.kind().as_ref(),
                    transition.trigger().as_ref()
                )))
            })
            .collect(),
    )
}

pub(crate) fn show_workflow(workflow_document: FileContents) -> EffectPlan {
    show_document(workflow_document)
}

pub(crate) fn show_document(document: FileContents) -> EffectPlan {
    EffectPlan::new(vec![Effect::ReportDocument(document)])
}

fn formal_workflow_effects(workflow: &FormalWorkflowGraph) -> Vec<Effect> {
    let workflow_name = workflow.name().as_ref().to_owned();
    let lean_name_marker = canonical_declaration_marker(format!(
        "def workflowName := {}",
        json_string(workflow.name().as_ref())
    ));
    let lean_name_prefix = canonical_declaration_prefix("def workflowName :=");
    let lean_slug_marker = canonical_declaration_marker(format!(
        "def workflowSlug := {}",
        json_string(workflow.slug().as_ref())
    ));
    let lean_slug_prefix = canonical_declaration_prefix("def workflowSlug :=");
    let lean_description_marker = canonical_declaration_marker(format!(
        "def workflowDescription := {}",
        json_string(workflow.description().as_ref())
    ));
    let lean_description_prefix = canonical_declaration_prefix("def workflowDescription :=");
    let quint_name_marker = canonical_declaration_marker(format!(
        "val workflowName = {}",
        json_string(workflow.name().as_ref())
    ));
    let quint_name_prefix = canonical_declaration_prefix("val workflowName =");
    let quint_slug_marker = canonical_declaration_marker(format!(
        "val workflowSlug = {}",
        json_string(workflow.slug().as_ref())
    ));
    let quint_slug_prefix = canonical_declaration_prefix("val workflowSlug =");
    let quint_description_marker = canonical_declaration_marker(format!(
        "val workflowDescription = {}",
        json_string(workflow.description().as_ref())
    ));
    let quint_description_prefix = canonical_declaration_prefix("val workflowDescription =");
    let lean_slice_marker = lean_workflow_slice_marker(workflow);
    let lean_slice_detail_marker = lean_workflow_slice_detail_marker(workflow);
    let lean_slice_module_marker = lean_workflow_slice_module_marker(workflow);
    let lean_transition_marker = lean_workflow_transition_marker(workflow);
    let lean_exit_target_marker = lean_workflow_exit_target_marker(workflow);
    let quint_slice_marker = quint_workflow_slice_marker(workflow);
    let quint_slice_detail_marker = quint_workflow_slice_detail_marker(workflow);
    let quint_slice_module_marker = quint_workflow_slice_module_marker(workflow);
    let quint_transition_marker = quint_workflow_transition_marker(workflow);
    let quint_exit_target_marker = quint_workflow_exit_target_marker(workflow);
    let lean_slice_prefix = canonical_declaration_prefix("def workflowSlices : List String :=");
    let lean_slice_detail_prefix =
        canonical_declaration_prefix("def workflowSliceDetails : List WorkflowSliceDetail :=");
    let lean_slice_module_prefix =
        canonical_declaration_prefix("def workflowSliceModules : List WorkflowSliceModule :=");
    let lean_transition_prefix =
        canonical_declaration_prefix("def workflowTransitions : List WorkflowTransition :=");
    let lean_exit_target_prefix =
        canonical_declaration_prefix("def workflowExitTargets : List String :=");
    let quint_slice_prefix = canonical_declaration_prefix("val workflowSlices:");
    let quint_slice_detail_prefix = canonical_declaration_prefix("val workflowSliceDetails:");
    let quint_slice_module_prefix = canonical_declaration_prefix("val workflowSliceModules:");
    let quint_transition_prefix = canonical_declaration_prefix("val workflowTransitions:");
    let quint_exit_target_prefix = canonical_declaration_prefix("val workflowExitTargets:");
    let lean_identity_invariant_marker = canonical_declaration_marker(format!(
        "theorem workflowIdentityIsStable : workflowName = {} := rfl",
        json_string(workflow.name().as_ref())
    ));
    let lean_identity_invariant_prefix =
        canonical_declaration_prefix("theorem workflowIdentityIsStable :");
    let lean_slice_detail_invariant_marker = canonical_declaration_marker(
        "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl",
    );
    let lean_slice_detail_invariant_prefix =
        canonical_declaration_prefix("theorem workflowSlicesHaveDetails :");
    let lean_slice_module_invariant_marker = canonical_declaration_marker(
        "theorem workflowSlicesHaveModuleReferences : workflowSlices.length = workflowSliceModules.length := rfl",
    );
    let lean_slice_module_invariant_prefix =
        canonical_declaration_prefix("theorem workflowSlicesHaveModuleReferences :");
    let lean_transition_invariant_marker = canonical_declaration_marker(
        "theorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.source.isEmpty == false && transition.target.isEmpty == false && transition.kind.isEmpty == false && transition.trigger.isEmpty == false) = true := rfl",
    );
    let lean_transition_invariant_prefix =
        canonical_declaration_prefix("theorem workflowTransitionsAreStructured :");
    let lean_transition_source_resolution_marker = canonical_declaration_marker(
        "theorem workflowTransitionSourcesResolve : workflowTransitions.all (fun transition => workflowSlices.contains transition.source) = true := rfl",
    );
    let lean_transition_source_resolution_prefix =
        canonical_declaration_prefix("theorem workflowTransitionSourcesResolve :");
    let lean_transition_target_resolution_marker = canonical_declaration_marker(
        "theorem workflowTransitionTargetsResolve : workflowTransitions.all (fun transition => workflowSlices.contains transition.target || workflowExitTargets.contains transition.target) = true := rfl",
    );
    let lean_transition_target_resolution_prefix =
        canonical_declaration_prefix("theorem workflowTransitionTargetsResolve :");
    let quint_identity_invariant_marker = canonical_declaration_marker(format!(
        "val workflowIdentityStable = workflowName == {}",
        json_string(workflow.name().as_ref())
    ));
    let quint_identity_invariant_prefix =
        canonical_declaration_prefix("val workflowIdentityStable =");
    let quint_slice_detail_invariant_marker = canonical_declaration_marker(
        "val workflowSlicesHaveDetails = length(workflowSlices) == length(workflowSliceDetails)",
    );
    let quint_slice_detail_invariant_prefix =
        canonical_declaration_prefix("val workflowSlicesHaveDetails =");
    let quint_slice_detail_complete_marker = canonical_declaration_marker(
        "val workflowSliceDetailsComplete = workflowSlicesHaveDetails",
    );
    let quint_slice_detail_complete_prefix =
        canonical_declaration_prefix("val workflowSliceDetailsComplete =");
    let quint_slice_module_complete_marker = canonical_declaration_marker(
        "val workflowSliceModulesComplete = workflowSlices.length() == workflowSliceModules.length()",
    );
    let quint_slice_module_complete_prefix =
        canonical_declaration_prefix("val workflowSliceModulesComplete =");
    let quint_transition_invariant_marker = canonical_declaration_marker(
        "val workflowTransitionsStructured = workflowTransitions.select(transition => transition.source != \"\" and transition.target != \"\" and transition.kind != \"\" and transition.trigger != \"\").length() == workflowTransitions.length()",
    );
    let quint_transition_invariant_prefix =
        canonical_declaration_prefix("val workflowTransitionsStructured =");
    let quint_transition_source_resolution_marker = canonical_declaration_marker(
        "val workflowTransitionSourcesResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.source).length() > 0).length() == workflowTransitions.length()",
    );
    let quint_transition_source_resolution_prefix =
        canonical_declaration_prefix("val workflowTransitionSourcesResolve =");
    let quint_transition_target_resolution_marker = canonical_declaration_marker(
        "val workflowTransitionTargetsResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.target).length() > 0 or workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() > 0).length() == workflowTransitions.length()",
    );
    let quint_transition_target_resolution_prefix =
        canonical_declaration_prefix("val workflowTransitionTargetsResolve =");
    let module_name = module_name_from_model(workflow.name().clone());
    let lean_module_marker = canonical_declaration_marker(format!("namespace {module_name}"));
    let lean_module_prefix = canonical_declaration_prefix("namespace ");
    let lean_module_end_marker = canonical_declaration_marker(format!("end {module_name}"));
    let lean_module_end_prefix = canonical_declaration_prefix("end ");
    let quint_module_marker = canonical_declaration_marker(format!("module {module_name} {{"));
    let quint_module_prefix = canonical_declaration_prefix("module ");
    let quint_module_close_prefix = canonical_declaration_prefix("}");
    let quint_module_close_marker = canonical_declaration_marker("}");
    let lean_path = project_path(format!("model/lean/{module_name}.lean"));
    let quint_path = project_path(format!("model/quint/{module_name}.qnt"));
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow.name().clone(),
        workflow_slug: workflow.slug().clone(),
        workflow_description: workflow.description().clone(),
        workflow_slice_details: workflow.slice_details().clone(),
        workflow_transitions: workflow.transitions().clone(),
        workflow_outcomes: workflow.outcomes().clone(),
        workflow_command_errors: workflow.command_errors().clone(),
        workflow_owned_definitions: workflow.owned_definitions().clone(),
        workflow_transition_evidences: workflow.transition_evidences().clone(),
        workflow_requires_entry_lifecycle_coverage: workflow.entry_lifecycle_required(),
        workflow_entry_lifecycle_states: workflow.entry_lifecycle_states().clone(),
    });

    let workflow_effects = vec![
        Effect::RequireFile(lean_path.clone()),
        Effect::RequireFile(quint_path.clone()),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_module_prefix,
            lean_module_marker,
            report_line(format!(
                "Lean workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_module_end_prefix,
            lean_module_end_marker,
            report_line(format!(
                "Lean workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_module_prefix,
            quint_module_marker,
            report_line(format!(
                "Quint workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_module_close_prefix,
            quint_module_close_marker,
            report_line(format!(
                "Quint workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_name_prefix,
            lean_name_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_slug_prefix,
            lean_slug_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_description_prefix,
            lean_description_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_name_prefix,
            quint_name_marker,
            report_line(format!(
                "Quint workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_slug_prefix,
            quint_slug_marker,
            report_line(format!(
                "Quint workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_description_prefix,
            quint_description_marker,
            report_line(format!(
                "Quint workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_slice_prefix,
            lean_slice_marker,
            report_line(format!(
                "Lean workflow slice drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_slice_detail_prefix,
            lean_slice_detail_marker,
            report_line(format!(
                "Lean workflow slice detail drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_slice_module_prefix,
            lean_slice_module_marker,
            report_line(format!(
                "Lean workflow slice module drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_slice_prefix,
            quint_slice_marker,
            report_line(format!(
                "Quint workflow slice drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_slice_detail_prefix,
            quint_slice_detail_marker,
            report_line(format!(
                "Quint workflow slice detail drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_slice_module_prefix,
            quint_slice_module_marker,
            report_line(format!(
                "Quint workflow slice module drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_transition_prefix,
            lean_transition_marker,
            report_line(format!(
                "Lean workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_exit_target_prefix,
            lean_exit_target_marker,
            report_line(format!(
                "Lean workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_transition_prefix,
            quint_transition_marker,
            report_line(format!(
                "Quint workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_exit_target_prefix,
            quint_exit_target_marker,
            report_line(format!(
                "Quint workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_identity_invariant_prefix,
            lean_identity_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_identity_invariant_prefix,
            quint_identity_invariant_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_slice_detail_invariant_prefix,
            lean_slice_detail_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_slice_module_invariant_prefix,
            lean_slice_module_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_slice_detail_invariant_prefix,
            quint_slice_detail_invariant_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_slice_detail_complete_prefix,
            quint_slice_detail_complete_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_slice_module_complete_prefix,
            quint_slice_module_complete_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_transition_invariant_prefix,
            lean_transition_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_transition_invariant_prefix,
            quint_transition_invariant_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_transition_source_resolution_prefix,
            lean_transition_source_resolution_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            lean_path.clone(),
            lean_transition_target_resolution_prefix,
            lean_transition_target_resolution_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_transition_source_resolution_prefix,
            quint_transition_source_resolution_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_canonical_declaration(
            quint_path.clone(),
            quint_transition_target_resolution_prefix,
            quint_transition_target_resolution_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::require_digest(
            lean_path,
            digest.clone(),
            report_line(format!(
                "artifact digest mismatch for workflow {workflow_name}"
            )),
        ),
        Effect::require_digest(
            quint_path,
            digest,
            report_line(format!(
                "artifact digest mismatch for workflow {workflow_name}"
            )),
        ),
    ];

    workflow_effects
        .into_iter()
        .chain(formal_slice_effects(workflow))
        .collect()
}

fn formal_slice_effects(workflow: &FormalWorkflowGraph) -> Vec<Effect> {
    let workflow_name = workflow.name().as_ref().to_owned();
    workflow
        .slice_details()
        .as_slice()
        .iter()
        .flat_map(|slice| {
            let module_name = module_name_from_model(slice.name().clone());
            let slice_digest = slice_artifact_digest(
                slice.name().clone(),
                slice.slug().clone(),
                *slice.kind(),
                slice.description().clone(),
            );
            let lean_slice_path = project_path(format!("model/lean/slices/{module_name}.lean"));
            let quint_slice_path = project_path(format!("model/quint/slices/{module_name}.qnt"));

            [
                Effect::require_file_contents_with_authored_formal_facts(
                    lean_slice_path,
                    emit_lean_slice_module(
                        lean_module_name(module_name.clone()),
                        slice.name().clone(),
                        slice.description().clone(),
                        slice.slug().clone(),
                        *slice.kind(),
                        slice_digest.clone(),
                    ),
                    report_line(format!(
                        "Lean slice artifact drift for workflow {workflow_name}"
                    )),
                ),
                Effect::require_file_contents_with_authored_formal_facts(
                    quint_slice_path,
                    emit_quint_slice_module(
                        quint_module_name(module_name),
                        slice.name().clone(),
                        slice.description().clone(),
                        slice.slug().clone(),
                        *slice.kind(),
                        slice_digest,
                    ),
                    report_line(format!(
                        "Quint slice artifact drift for workflow {workflow_name}"
                    )),
                ),
            ]
        })
        .collect()
}

fn lean_workflow_slice_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "def workflowSlices : List String := [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| json_string(slice.slug().as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn lean_workflow_slice_detail_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "def workflowSliceDetails : List WorkflowSliceDetail := [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| {
                format!(
                    "{{ slug := {}, name := {}, kind := {}, description := {} }}",
                    json_string(slice.slug().as_ref()),
                    json_string(slice.name().as_ref()),
                    json_string(slice.kind().as_ref()),
                    json_string(slice.description().as_ref())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn lean_workflow_slice_module_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "def workflowSliceModules : List WorkflowSliceModule := [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| {
                format!(
                    "{{ slice := {}, formalModule := {} }}",
                    json_string(slice.slug().as_ref()),
                    json_string(&module_name_from_model(slice.name().clone()))
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn lean_workflow_transition_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "def workflowTransitions : List WorkflowTransition := [{}]",
        workflow
            .transitions()
            .as_slice()
            .iter()
            .map(|transition| {
                format!(
                    "{{ source := {}, target := {}, kind := {}, trigger := {}, rationale := {}, payloadContract := {} }}",
                    json_string(transition.source().as_ref()),
                    json_string(transition.target().as_ref()),
                    json_string(transition.kind().as_ref()),
                    json_string(transition.trigger().as_ref()),
                    json_string(
                        transition
                            .rationale()
                            .map_or("", |rationale| rationale.as_ref())
                    ),
                    json_string(
                        transition
                            .payload_contract()
                            .map_or("", |payload_contract| payload_contract.as_ref())
                    )
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn lean_workflow_exit_target_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "def workflowExitTargets : List String := [{}]",
        workflow_exit_targets(workflow).join(",")
    ))
}

fn quint_workflow_slice_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "val workflowSlices: List[str] = [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| json_string(slice.slug().as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn quint_workflow_slice_detail_marker(
    workflow: &FormalWorkflowGraph,
) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "val workflowSliceDetails: List[WorkflowSliceDetail] = [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| {
                format!(
                    "{{ slug: {}, name: {}, kind: {}, description: {} }}",
                    json_string(slice.slug().as_ref()),
                    json_string(slice.name().as_ref()),
                    json_string(slice.kind().as_ref()),
                    json_string(slice.description().as_ref())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn quint_workflow_slice_module_marker(
    workflow: &FormalWorkflowGraph,
) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "val workflowSliceModules: List[WorkflowSliceModule] = [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| {
                format!(
                    "{{ slice: {}, formalModule: {} }}",
                    json_string(slice.slug().as_ref()),
                    json_string(&module_name_from_model(slice.name().clone()))
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn quint_workflow_transition_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "val workflowTransitions: List[WorkflowTransition] = [{}]",
        workflow
            .transitions()
            .as_slice()
            .iter()
            .map(|transition| {
                format!(
                    "{{ source: {}, target: {}, kind: {}, trigger: {}, rationale: {}, payloadContract: {} }}",
                    json_string(transition.source().as_ref()),
                    json_string(transition.target().as_ref()),
                    json_string(transition.kind().as_ref()),
                    json_string(transition.trigger().as_ref()),
                    json_string(
                        transition
                            .rationale()
                            .map_or("", |rationale| rationale.as_ref())
                    ),
                    json_string(
                        transition
                            .payload_contract()
                            .map_or("", |payload_contract| payload_contract.as_ref())
                    )
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn quint_workflow_exit_target_marker(workflow: &FormalWorkflowGraph) -> CanonicalDeclarationMarker {
    canonical_declaration_marker(format!(
        "val workflowExitTargets: List[str] = [{}]",
        workflow_exit_targets(workflow).join(",")
    ))
}

fn workflow_exit_targets(workflow: &FormalWorkflowGraph) -> Vec<String> {
    workflow
        .transitions()
        .as_slice()
        .iter()
        .filter(|transition| transition.kind().is_workflow_exit())
        .map(|transition| json_string(transition.target().as_ref()))
        .collect()
}

fn workflow_slug_list(modeled_workflows: &[ModeledWorkflowLayout]) -> String {
    format!(
        "[{}]",
        modeled_workflows
            .iter()
            .map(|workflow| json_string(workflow.slug().as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn formal_model_slices(formal_workflows: &[FormalWorkflowGraph]) -> Vec<FormalModelSlice> {
    formal_workflows
        .iter()
        .flat_map(|workflow| {
            workflow
                .slice_details()
                .as_slice()
                .iter()
                .map(|slice| FormalModelSlice::new(workflow.slug().clone(), slice.slug().clone()))
        })
        .collect()
}

fn formal_model_slice_modules(
    formal_workflows: &[FormalWorkflowGraph],
) -> Vec<FormalModelSliceModule> {
    formal_workflows
        .iter()
        .flat_map(|workflow| {
            workflow.slice_details().as_slice().iter().map(|slice| {
                FormalModelSliceModule::new(
                    workflow.slug().clone(),
                    slice.slug().clone(),
                    lean_module_name(module_name_from_model(slice.name().clone())),
                )
            })
        })
        .collect()
}

fn formal_model_scenarios(project_scenarios: &[ProjectScenario]) -> Vec<FormalModelScenario> {
    project_scenarios
        .iter()
        .map(|scenario| {
            FormalModelScenario::new(
                semantic_value(
                    "workflow slug",
                    scenario.workflow_slug(),
                    WorkflowSlug::try_new,
                ),
                semantic_value("slice slug", scenario.slice_slug(), SliceSlug::try_new),
                semantic_value(
                    "scenario kind",
                    scenario.scenario_kind(),
                    ScenarioKind::try_new,
                ),
                semantic_value("scenario name", scenario.scenario(), ScenarioName::try_new),
            )
        })
        .collect()
}

fn formal_model_scenario_definitions(
    scenario_definitions: &[ProjectScenarioDefinition],
) -> Vec<FormalModelScenarioDefinition> {
    scenario_definitions
        .iter()
        .map(|scenario| {
            FormalModelScenarioDefinition::new(FormalModelScenarioDefinitionFields {
                workflow: semantic_value(
                    "workflow slug",
                    scenario.workflow_slug(),
                    WorkflowSlug::try_new,
                ),
                slice: semantic_value("slice slug", scenario.slice_slug(), SliceSlug::try_new),
                scenario_kind: semantic_value(
                    "scenario kind",
                    scenario.scenario_kind(),
                    ScenarioKind::try_new,
                ),
                scenario: semantic_value(
                    "scenario name",
                    scenario.scenario(),
                    ScenarioName::try_new,
                ),
                given: semantic_value(
                    "scenario given step",
                    scenario.given(),
                    ScenarioStepText::try_new,
                ),
                when: semantic_value(
                    "scenario when step",
                    scenario.when(),
                    ScenarioStepText::try_new,
                ),
                then: semantic_value(
                    "scenario then step",
                    scenario.then(),
                    ScenarioStepText::try_new,
                ),
                read_streams: semantic_values(
                    "scenario read stream",
                    scenario.read_streams(),
                    StreamName::try_new,
                ),
                written_streams: semantic_values(
                    "scenario written stream",
                    scenario.written_streams(),
                    StreamName::try_new,
                ),
                contract_kind: optional_semantic_value(
                    "scenario contract kind",
                    scenario.contract_kind(),
                    ContractKindName::try_new,
                ),
                covered_definition: optional_semantic_value(
                    "scenario covered definition",
                    scenario.covered_definition(),
                    CoveredDefinitionName::try_new,
                ),
                error_references: semantic_values(
                    "scenario error reference",
                    scenario.error_references(),
                    CommandErrorName::try_new,
                ),
            })
        })
        .collect()
}

fn formal_model_data_flows(project_data_flows: &[ProjectDataFlow]) -> Vec<FormalModelDataFlow> {
    project_data_flows
        .iter()
        .map(|data_flow| {
            FormalModelDataFlow::new(FormalModelDataFlowFields {
                workflow: semantic_value(
                    "workflow slug",
                    data_flow.workflow_slug(),
                    WorkflowSlug::try_new,
                ),
                slice: semantic_value("slice slug", data_flow.slice_slug(), SliceSlug::try_new),
                datum: semantic_value("data-flow datum", data_flow.datum(), DatumName::try_new),
                source_kind: semantic_value(
                    "data-flow source kind",
                    data_flow.source_kind(),
                    DataFlowSourceKind::try_new,
                ),
                source: semantic_value(
                    "data-flow source",
                    data_flow.source(),
                    DataFlowSource::try_new,
                ),
                transformation: semantic_value(
                    "data-flow transformation",
                    data_flow.transformation(),
                    TransformationSemantics::try_new,
                ),
                target: semantic_value(
                    "data-flow target",
                    data_flow.target(),
                    DataFlowTarget::try_new,
                ),
                bit_encoding: semantic_value(
                    "data-flow bit encoding",
                    data_flow.bit_encoding(),
                    BitEncodingSemantics::try_new,
                ),
            })
        })
        .collect()
}

fn formal_model_outcomes(project_outcomes: &[ProjectOutcome]) -> Vec<FormalModelOutcome> {
    project_outcomes
        .iter()
        .map(|outcome| {
            FormalModelOutcome::new(
                semantic_value(
                    "workflow slug",
                    outcome.workflow_slug(),
                    WorkflowSlug::try_new,
                ),
                semantic_value("slice slug", outcome.slice_slug(), SliceSlug::try_new),
                semantic_value(
                    "outcome label",
                    outcome.outcome(),
                    OutcomeLabelName::try_new,
                ),
                semantic_values("outcome event", outcome.events(), EventName::try_new),
                outcome.externally_relevant(),
            )
        })
        .collect()
}

fn formal_model_command_errors(
    project_command_errors: &[ProjectCommandError],
) -> Vec<FormalModelCommandError> {
    project_command_errors
        .iter()
        .map(|command_error| {
            FormalModelCommandError::new(
                semantic_value(
                    "workflow slug",
                    command_error.workflow_slug(),
                    WorkflowSlug::try_new,
                ),
                semantic_value("slice slug", command_error.slice_slug(), SliceSlug::try_new),
                semantic_value(
                    "command name",
                    command_error.command(),
                    CommandName::try_new,
                ),
                semantic_value(
                    "command error name",
                    command_error.error(),
                    CommandErrorName::try_new,
                ),
                semantic_value(
                    "scenario name",
                    command_error.scenario(),
                    ScenarioName::try_new,
                ),
                semantic_value(
                    "command error recovery",
                    command_error.recovery(),
                    CommandErrorRecoveryKind::try_new,
                ),
            )
        })
        .collect()
}

fn formal_model_commands(project_commands: &[ProjectCommand]) -> Vec<FormalModelCommand> {
    project_commands
        .iter()
        .map(|command| {
            FormalModelCommand::new(
                semantic_value(
                    "workflow slug",
                    command.workflow_slug(),
                    WorkflowSlug::try_new,
                ),
                semantic_value("slice slug", command.slice_slug(), SliceSlug::try_new),
                semantic_value("command name", command.command(), CommandName::try_new),
            )
        })
        .collect()
}

fn formal_model_command_inputs(
    project_command_inputs: &[ProjectCommandInput],
) -> Vec<FormalModelCommandInput> {
    project_command_inputs
        .iter()
        .map(|command_input| {
            FormalModelCommandInput::new(FormalModelCommandInputFields {
                workflow: semantic_value(
                    "workflow slug",
                    command_input.workflow_slug(),
                    WorkflowSlug::try_new,
                ),
                slice: semantic_value("slice slug", command_input.slice_slug(), SliceSlug::try_new),
                command: semantic_value(
                    "command name",
                    command_input.command(),
                    CommandName::try_new,
                ),
                input: semantic_value(
                    "command input name",
                    command_input.input(),
                    DatumName::try_new,
                ),
                source_kind: semantic_value(
                    "command input source kind",
                    command_input.source_kind(),
                    CommandInputSourceKind::try_new,
                ),
                source_description: semantic_value(
                    "command input source description",
                    command_input.source_description(),
                    CommandInputSourceDescription::try_new,
                ),
                provenance_chain: semantic_values(
                    "command input provenance chain hop",
                    command_input.provenance_chain(),
                    SourceChainHop::try_new,
                ),
                event_stream_source_event: optional_semantic_value(
                    "command input event stream source event",
                    command_input.event_stream_source_event(),
                    EventName::try_new,
                ),
                event_stream_source_attribute: optional_semantic_value(
                    "command input event stream source attribute",
                    command_input.event_stream_source_attribute(),
                    EventAttributeName::try_new,
                ),
                external_payload_source_name: optional_semantic_value(
                    "command input external payload source name",
                    command_input.external_payload_source_name(),
                    EventAttributeSourceName::try_new,
                ),
                external_payload_source_field: optional_semantic_value(
                    "command input external payload source field",
                    command_input.external_payload_source_field(),
                    EventAttributeSourceField::try_new,
                ),
                generated_source_name: optional_semantic_value(
                    "command input generated source name",
                    command_input.generated_source_name(),
                    EventAttributeSourceName::try_new,
                ),
                generated_source_field: optional_semantic_value(
                    "command input generated source field",
                    command_input.generated_source_field(),
                    EventAttributeSourceField::try_new,
                ),
                session_source_name: optional_semantic_value(
                    "command input session source name",
                    command_input.session_source_name(),
                    EventAttributeSourceName::try_new,
                ),
                session_source_field: optional_semantic_value(
                    "command input session source field",
                    command_input.session_source_field(),
                    EventAttributeSourceField::try_new,
                ),
                invocation_argument_source_name: optional_semantic_value(
                    "command input invocation argument source name",
                    command_input.invocation_argument_source_name(),
                    EventAttributeSourceName::try_new,
                ),
                invocation_argument_source_field: optional_semantic_value(
                    "command input invocation argument source field",
                    command_input.invocation_argument_source_field(),
                    EventAttributeSourceField::try_new,
                ),
            })
        })
        .collect()
}

fn semantic_values<T, E>(
    field: &str,
    values: &[String],
    parse: impl Fn(String) -> Result<T, E> + Copy,
) -> Vec<T>
where
    E: Display,
{
    values
        .iter()
        .map(|value| semantic_value(field, value, parse))
        .collect()
}

fn optional_semantic_value<T, E>(
    field: &str,
    value: &str,
    parse: impl FnOnce(String) -> Result<T, E>,
) -> Option<T>
where
    E: Display,
{
    if value.trim().is_empty() {
        None
    } else {
        Some(semantic_value(field, value, parse))
    }
}

fn semantic_value<T, E>(field: &str, value: &str, parse: impl FnOnce(String) -> Result<T, E>) -> T
where
    E: Display,
{
    parse(value.to_owned()).unwrap_or_else(|error| {
        unreachable!("EMC generated project inventory must carry valid {field}: {error}");
    })
}

fn lean_model_stream_list(project_streams: &[ProjectStream]) -> String {
    let mut project_streams = project_streams
        .iter()
        .map(|stream| (stream.workflow_slug(), stream.slice_slug(), stream.stream()))
        .collect::<Vec<_>>();
    project_streams.sort_unstable();
    format!(
        "[{}]",
        project_streams
            .into_iter()
            .map(|(workflow_slug, slice_slug, stream)| {
                format!(
                    "{{ workflow := {}, slice := {}, stream := {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(stream)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_stream_list(project_streams: &[ProjectStream]) -> String {
    let mut project_streams = project_streams
        .iter()
        .map(|stream| (stream.workflow_slug(), stream.slice_slug(), stream.stream()))
        .collect::<Vec<_>>();
    project_streams.sort_unstable();
    format!(
        "[{}]",
        project_streams
            .into_iter()
            .map(|(workflow_slug, slice_slug, stream)| {
                format!(
                    "{{ workflow: {}, slice: {}, stream: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(stream)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_read_model_list(project_read_models: &[ProjectReadModel]) -> String {
    let mut project_read_models = project_read_models
        .iter()
        .map(|read_model| {
            (
                read_model.workflow_slug(),
                read_model.slice_slug(),
                read_model.read_model(),
            )
        })
        .collect::<Vec<_>>();
    project_read_models.sort_unstable();
    format!(
        "[{}]",
        project_read_models
            .into_iter()
            .map(|(workflow_slug, slice_slug, read_model)| {
                format!(
                    "{{ workflow := {}, slice := {}, readModel := {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(read_model)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_read_model_list(project_read_models: &[ProjectReadModel]) -> String {
    let mut project_read_models = project_read_models
        .iter()
        .map(|read_model| {
            (
                read_model.workflow_slug(),
                read_model.slice_slug(),
                read_model.read_model(),
            )
        })
        .collect::<Vec<_>>();
    project_read_models.sort_unstable();
    format!(
        "[{}]",
        project_read_models
            .into_iter()
            .map(|(workflow_slug, slice_slug, read_model)| {
                format!(
                    "{{ workflow: {}, slice: {}, readModel: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(read_model)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_read_model_definition_list(
    project_read_model_definitions: &[ProjectReadModelDefinition],
) -> String {
    let mut project_read_model_definitions = project_read_model_definitions
        .iter()
        .map(|definition| {
            (
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.read_model(),
                definition.transitive(),
                definition.relationship_fields(),
                definition.transitive_rule(),
                definition.example_scenario_name(),
            )
        })
        .collect::<Vec<_>>();
    project_read_model_definitions.sort_unstable();
    format!(
        "[{}]",
        project_read_model_definitions
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    read_model,
                    transitive,
                    relationship_fields,
                    transitive_rule,
                    example_scenario_name,
                )| {
                    format!(
                        "{{ workflow := {}, slice := {}, readModel := {}, transitive := {}, relationshipFields := [{}], transitiveRule := {}, exampleScenarioName := {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(read_model),
                        transitive,
                        json_string_list(relationship_fields),
                        json_string(transitive_rule),
                        json_string(example_scenario_name)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_read_model_definition_list(
    project_read_model_definitions: &[ProjectReadModelDefinition],
) -> String {
    let mut project_read_model_definitions = project_read_model_definitions
        .iter()
        .map(|definition| {
            (
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.read_model(),
                definition.transitive(),
                definition.relationship_fields(),
                definition.transitive_rule(),
                definition.example_scenario_name(),
            )
        })
        .collect::<Vec<_>>();
    project_read_model_definitions.sort_unstable();
    format!(
        "[{}]",
        project_read_model_definitions
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    read_model,
                    transitive,
                    relationship_fields,
                    transitive_rule,
                    example_scenario_name,
                )| {
                    format!(
                        "{{ workflow: {}, slice: {}, readModel: {}, transitive: {}, relationshipFields: [{}], transitiveRule: {}, exampleScenarioName: {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(read_model),
                        transitive,
                        json_string_list(relationship_fields),
                        json_string(transitive_rule),
                        json_string(example_scenario_name)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_read_model_field_list(project_read_model_fields: &[ProjectReadModelField]) -> String {
    let mut project_read_model_fields = project_read_model_fields
        .iter()
        .map(|field| {
            (
                field.workflow_slug(),
                field.slice_slug(),
                field.read_model(),
                field.field(),
                field.source_kind(),
                field.source_event(),
                field.source_attribute(),
                field.derivation_rule(),
                field.derivation_source_fields(),
                field.absence_event(),
                field.derivation_scenario_name(),
                field.absence_scenario_name(),
                field.provenance(),
            )
        })
        .collect::<Vec<_>>();
    project_read_model_fields.sort_unstable_by(|left, right| {
        left.0
            .cmp(right.0)
            .then_with(|| left.1.cmp(right.1))
            .then_with(|| left.2.cmp(right.2))
            .then_with(|| left.3.cmp(right.3))
    });
    format!(
        "[{}]",
        project_read_model_fields
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    read_model,
                    field,
                    source_kind,
                    source_event,
                    source_attribute,
                    derivation_rule,
                    derivation_source_fields,
                    absence_event,
                    derivation_scenario_name,
                    absence_scenario_name,
                    provenance,
                )| {
                    format!(
                        "{{ workflow := {}, slice := {}, readModel := {}, field := {}, sourceKind := {}, sourceEvent := {}, sourceAttribute := {}, derivationRule := {}, derivationSourceFields := [{}], absenceEvent := {}, derivationScenarioName := {}, absenceScenarioName := {}, provenance := {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(read_model),
                        json_string(field),
                        json_string(source_kind),
                        json_string(source_event),
                        json_string(source_attribute),
                        json_string(derivation_rule),
                        json_string_list(derivation_source_fields),
                        json_string(absence_event),
                        json_string(derivation_scenario_name),
                        json_string(absence_scenario_name),
                        json_string(provenance)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_read_model_field_list(
    project_read_model_fields: &[ProjectReadModelField],
) -> String {
    let mut project_read_model_fields = project_read_model_fields
        .iter()
        .map(|field| {
            (
                field.workflow_slug(),
                field.slice_slug(),
                field.read_model(),
                field.field(),
                field.source_kind(),
                field.source_event(),
                field.source_attribute(),
                field.derivation_rule(),
                field.derivation_source_fields(),
                field.absence_event(),
                field.derivation_scenario_name(),
                field.absence_scenario_name(),
                field.provenance(),
            )
        })
        .collect::<Vec<_>>();
    project_read_model_fields.sort_unstable_by(|left, right| {
        left.0
            .cmp(right.0)
            .then_with(|| left.1.cmp(right.1))
            .then_with(|| left.2.cmp(right.2))
            .then_with(|| left.3.cmp(right.3))
    });
    format!(
        "[{}]",
        project_read_model_fields
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    read_model,
                    field,
                    source_kind,
                    source_event,
                    source_attribute,
                    derivation_rule,
                    derivation_source_fields,
                    absence_event,
                    derivation_scenario_name,
                    absence_scenario_name,
                    provenance,
                )| {
                    format!(
                        "{{ workflow: {}, slice: {}, readModel: {}, field: {}, sourceKind: {}, sourceEvent: {}, sourceAttribute: {}, derivationRule: {}, derivationSourceFields: [{}], absenceEvent: {}, derivationScenarioName: {}, absenceScenarioName: {}, provenance: {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(read_model),
                        json_string(field),
                        json_string(source_kind),
                        json_string(source_event),
                        json_string(source_attribute),
                        json_string(derivation_rule),
                        json_string_list(derivation_source_fields),
                        json_string(absence_event),
                        json_string(derivation_scenario_name),
                        json_string(absence_scenario_name),
                        json_string(provenance)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_view_list(project_views: &[ProjectView]) -> String {
    let mut project_views = project_views
        .iter()
        .map(|view| (view.workflow_slug(), view.slice_slug(), view.view()))
        .collect::<Vec<_>>();
    project_views.sort_unstable();
    format!(
        "[{}]",
        project_views
            .into_iter()
            .map(|(workflow_slug, slice_slug, view)| {
                format!(
                    "{{ workflow := {}, slice := {}, view := {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(view)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_view_list(project_views: &[ProjectView]) -> String {
    let mut project_views = project_views
        .iter()
        .map(|view| (view.workflow_slug(), view.slice_slug(), view.view()))
        .collect::<Vec<_>>();
    project_views.sort_unstable();
    format!(
        "[{}]",
        project_views
            .into_iter()
            .map(|(workflow_slug, slice_slug, view)| {
                format!(
                    "{{ workflow: {}, slice: {}, view: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(view)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_view_definition_list(project_view_definitions: &[ProjectViewDefinition]) -> String {
    let mut project_view_definitions = project_view_definitions.to_vec();
    project_view_definitions.sort();
    format!(
        "[{}]",
        project_view_definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "{{ workflow := {}, slice := {}, view := {}, readModels := [{}], sketchTokens := [{}], localStates := [{}], filters := [{}] }}",
                    json_string(definition.workflow_slug()),
                    json_string(definition.slice_slug()),
                    json_string(definition.view()),
                    json_string_list(definition.read_models()),
                    json_string_list(definition.sketch_tokens()),
                    json_string_list(definition.local_states()),
                    json_string_list(definition.filters())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_view_definition_list(project_view_definitions: &[ProjectViewDefinition]) -> String {
    let mut project_view_definitions = project_view_definitions.to_vec();
    project_view_definitions.sort();
    format!(
        "[{}]",
        project_view_definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "{{ workflow: {}, slice: {}, view: {}, readModels: [{}], sketchTokens: [{}], localStates: [{}], filters: [{}] }}",
                    json_string(definition.workflow_slug()),
                    json_string(definition.slice_slug()),
                    json_string(definition.view()),
                    json_string_list(definition.read_models()),
                    json_string_list(definition.sketch_tokens()),
                    json_string_list(definition.local_states()),
                    json_string_list(definition.filters())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_view_control_list(project_view_controls: &[ProjectViewControl]) -> String {
    let mut project_view_controls = project_view_controls.to_vec();
    project_view_controls.sort();
    format!(
        "[{}]",
        project_view_controls
            .into_iter()
            .map(|control| {
                format!(
                    "{{ workflow := {}, slice := {}, view := {}, control := {}, command := {}, input := {}, inputSourceKind := {}, inputSourceDescription := {}, inputSketchToken := {}, inputVisibleToActor := {}, inputDecisionField := {}, handledErrors := [{}], recoveryBehavior := {}, controlSketchToken := {}, navigationType := {}, navigationTarget := {}, externalWorkflow := {}, externalSystem := {}, handoffContract := {} }}",
                    json_string(control.workflow_slug()),
                    json_string(control.slice_slug()),
                    json_string(control.view()),
                    json_string(control.control()),
                    json_string(control.command()),
                    json_string(control.input()),
                    json_string(control.input_source_kind()),
                    json_string(control.input_source_description()),
                    json_string(control.input_sketch_token()),
                    control.input_visible_to_actor(),
                    control.input_decision_field(),
                    json_string_list(control.handled_errors()),
                    json_string(control.recovery_behavior()),
                    json_string(control.control_sketch_token()),
                    json_string(control.navigation_type()),
                    json_string(control.navigation_target()),
                    json_string(control.external_workflow()),
                    json_string(control.external_system()),
                    json_string(control.handoff_contract())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_view_control_list(project_view_controls: &[ProjectViewControl]) -> String {
    let mut project_view_controls = project_view_controls.to_vec();
    project_view_controls.sort();
    format!(
        "[{}]",
        project_view_controls
            .into_iter()
            .map(|control| {
                format!(
                    "{{ workflow: {}, slice: {}, view: {}, control: {}, command: {}, input: {}, inputSourceKind: {}, inputSourceDescription: {}, inputSketchToken: {}, inputVisibleToActor: {}, inputDecisionField: {}, handledErrors: [{}], recoveryBehavior: {}, controlSketchToken: {}, navigationType: {}, navigationTarget: {}, externalWorkflow: {}, externalSystem: {}, handoffContract: {} }}",
                    json_string(control.workflow_slug()),
                    json_string(control.slice_slug()),
                    json_string(control.view()),
                    json_string(control.control()),
                    json_string(control.command()),
                    json_string(control.input()),
                    json_string(control.input_source_kind()),
                    json_string(control.input_source_description()),
                    json_string(control.input_sketch_token()),
                    control.input_visible_to_actor(),
                    control.input_decision_field(),
                    json_string_list(control.handled_errors()),
                    json_string(control.recovery_behavior()),
                    json_string(control.control_sketch_token()),
                    json_string(control.navigation_type()),
                    json_string(control.navigation_target()),
                    json_string(control.external_workflow()),
                    json_string(control.external_system()),
                    json_string(control.handoff_contract())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_board_element_list(project_board_elements: &[ProjectBoardElement]) -> String {
    let mut project_board_elements = project_board_elements.to_vec();
    project_board_elements.sort();
    format!(
        "[{}]",
        project_board_elements
            .into_iter()
            .map(|element| {
                format!(
                    "{{ workflow := {}, slice := {}, element := {}, kind := {}, lane := {}, declaredName := {}, mainPath := {} }}",
                    json_string(element.workflow_slug()),
                    json_string(element.slice_slug()),
                    json_string(element.element()),
                    json_string(element.kind()),
                    json_string(element.lane()),
                    json_string(element.declared_name()),
                    element.main_path()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_board_element_list(project_board_elements: &[ProjectBoardElement]) -> String {
    let mut project_board_elements = project_board_elements.to_vec();
    project_board_elements.sort();
    format!(
        "[{}]",
        project_board_elements
            .into_iter()
            .map(|element| {
                format!(
                    "{{ workflow: {}, slice: {}, element: {}, kind: {}, lane: {}, declaredName: {}, mainPath: {} }}",
                    json_string(element.workflow_slug()),
                    json_string(element.slice_slug()),
                    json_string(element.element()),
                    json_string(element.kind()),
                    json_string(element.lane()),
                    json_string(element.declared_name()),
                    element.main_path()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_board_connection_list(
    project_board_connections: &[ProjectBoardConnection],
) -> String {
    let mut project_board_connections = project_board_connections.to_vec();
    project_board_connections.sort();
    format!(
        "[{}]",
        project_board_connections
            .into_iter()
            .map(|connection| {
                format!(
                    "{{ workflow := {}, slice := {}, source := {}, sourceKind := {}, target := {}, targetKind := {} }}",
                    json_string(connection.workflow_slug()),
                    json_string(connection.slice_slug()),
                    json_string(connection.source()),
                    json_string(connection.source_kind()),
                    json_string(connection.target()),
                    json_string(connection.target_kind())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_board_connection_list(
    project_board_connections: &[ProjectBoardConnection],
) -> String {
    let mut project_board_connections = project_board_connections.to_vec();
    project_board_connections.sort();
    format!(
        "[{}]",
        project_board_connections
            .into_iter()
            .map(|connection| {
                format!(
                    "{{ workflow: {}, slice: {}, source: {}, sourceKind: {}, target: {}, targetKind: {} }}",
                    json_string(connection.workflow_slug()),
                    json_string(connection.slice_slug()),
                    json_string(connection.source()),
                    json_string(connection.source_kind()),
                    json_string(connection.target()),
                    json_string(connection.target_kind())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_view_field_list(project_view_fields: &[ProjectViewField]) -> String {
    let mut project_view_fields = project_view_fields
        .iter()
        .map(|field| {
            (
                field.workflow_slug(),
                field.slice_slug(),
                field.view(),
                field.field(),
                field.source_kind(),
                field.source_read_model(),
                field.source_field(),
                field.provenance(),
                field.bit_encoding(),
            )
        })
        .collect::<Vec<_>>();
    project_view_fields.sort_unstable();
    format!(
        "[{}]",
        project_view_fields
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    view,
                    field,
                    source_kind,
                    source_read_model,
                    source_field,
                    provenance,
                    bit_encoding,
                )| {
                    format!(
                        "{{ workflow := {}, slice := {}, view := {}, field := {}, sourceKind := {}, sourceReadModel := {}, sourceField := {}, provenance := {}, bitEncoding := {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(view),
                        json_string(field),
                        json_string(source_kind),
                        json_string(source_read_model),
                        json_string(source_field),
                        json_string(provenance),
                        json_string(bit_encoding)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_view_field_list(project_view_fields: &[ProjectViewField]) -> String {
    let mut project_view_fields = project_view_fields
        .iter()
        .map(|field| {
            (
                field.workflow_slug(),
                field.slice_slug(),
                field.view(),
                field.field(),
                field.source_kind(),
                field.source_read_model(),
                field.source_field(),
                field.provenance(),
                field.bit_encoding(),
            )
        })
        .collect::<Vec<_>>();
    project_view_fields.sort_unstable();
    format!(
        "[{}]",
        project_view_fields
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    view,
                    field,
                    source_kind,
                    source_read_model,
                    source_field,
                    provenance,
                    bit_encoding,
                )| {
                    format!(
                        "{{ workflow: {}, slice: {}, view: {}, field: {}, sourceKind: {}, sourceReadModel: {}, sourceField: {}, provenance: {}, bitEncoding: {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(view),
                        json_string(field),
                        json_string(source_kind),
                        json_string(source_read_model),
                        json_string(source_field),
                        json_string(provenance),
                        json_string(bit_encoding)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_automation_list(project_automations: &[ProjectAutomation]) -> String {
    let mut project_automations = project_automations
        .iter()
        .map(|automation| {
            (
                automation.workflow_slug(),
                automation.slice_slug(),
                automation.automation(),
            )
        })
        .collect::<Vec<_>>();
    project_automations.sort_unstable();
    format!(
        "[{}]",
        project_automations
            .into_iter()
            .map(|(workflow_slug, slice_slug, automation)| {
                format!(
                    "({}, {}, {})",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(automation)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_automation_list(project_automations: &[ProjectAutomation]) -> String {
    let mut project_automations = project_automations
        .iter()
        .map(|automation| {
            (
                automation.workflow_slug(),
                automation.slice_slug(),
                automation.automation(),
            )
        })
        .collect::<Vec<_>>();
    project_automations.sort_unstable();
    format!(
        "[{}]",
        project_automations
            .into_iter()
            .map(|(workflow_slug, slice_slug, automation)| {
                format!(
                    "{{ workflow: {}, slice: {}, automation: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(automation)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_automation_definition_list(definitions: &[ProjectAutomationDefinition]) -> String {
    let mut definitions = definitions.iter().collect::<Vec<_>>();
    definitions.sort_unstable();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "({}, {}, {}, {}, {}, [{}], {})",
                    json_string(definition.workflow_slug()),
                    json_string(definition.slice_slug()),
                    json_string(definition.automation()),
                    json_string(definition.trigger()),
                    json_string(definition.command()),
                    json_string_list(definition.handled_errors()),
                    json_string(definition.reaction())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_automation_definition_list(definitions: &[ProjectAutomationDefinition]) -> String {
    let mut definitions = definitions.iter().collect::<Vec<_>>();
    definitions.sort_unstable();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "{{ workflow: {}, slice: {}, automation: {}, trigger: {}, command: {}, handledErrors: [{}], reaction: {} }}",
                    json_string(definition.workflow_slug()),
                    json_string(definition.slice_slug()),
                    json_string(definition.automation()),
                    json_string(definition.trigger()),
                    json_string(definition.command()),
                    json_string_list(definition.handled_errors()),
                    json_string(definition.reaction())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_translation_list(project_translations: &[ProjectTranslation]) -> String {
    let mut project_translations = project_translations
        .iter()
        .map(|translation| {
            (
                translation.workflow_slug(),
                translation.slice_slug(),
                translation.translation(),
            )
        })
        .collect::<Vec<_>>();
    project_translations.sort_unstable();
    format!(
        "[{}]",
        project_translations
            .into_iter()
            .map(|(workflow_slug, slice_slug, translation)| {
                format!(
                    "{{ workflow := {}, slice := {}, translation := {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(translation)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_translation_list(project_translations: &[ProjectTranslation]) -> String {
    let mut project_translations = project_translations
        .iter()
        .map(|translation| {
            (
                translation.workflow_slug(),
                translation.slice_slug(),
                translation.translation(),
            )
        })
        .collect::<Vec<_>>();
    project_translations.sort_unstable();
    format!(
        "[{}]",
        project_translations
            .into_iter()
            .map(|(workflow_slug, slice_slug, translation)| {
                format!(
                    "{{ workflow: {}, slice: {}, translation: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(translation)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_translation_definition_list(definitions: &[ProjectTranslationDefinition]) -> String {
    let mut definitions = definitions
        .iter()
        .map(|definition| {
            (
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.translation(),
                definition.external_event(),
                definition.payload_contract(),
                definition.command(),
            )
        })
        .collect::<Vec<_>>();
    definitions.sort_unstable();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    translation,
                    external_event,
                    payload_contract,
                    command,
                )| {
                    format!(
                        "{{ workflow := {}, slice := {}, translation := {}, externalEvent := {}, payloadContract := {}, command := {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(translation),
                        json_string(external_event),
                        json_string(payload_contract),
                        json_string(command)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_translation_definition_list(definitions: &[ProjectTranslationDefinition]) -> String {
    let mut definitions = definitions
        .iter()
        .map(|definition| {
            (
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.translation(),
                definition.external_event(),
                definition.payload_contract(),
                definition.command(),
            )
        })
        .collect::<Vec<_>>();
    definitions.sort_unstable();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(
                |(workflow_slug, slice_slug, translation, external_event, payload_contract, command)| {
                    format!(
                        "{{ workflow: {}, slice: {}, translation: {}, externalEvent: {}, payloadContract: {}, command: {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(translation),
                        json_string(external_event),
                        json_string(payload_contract),
                        json_string(command)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_external_payload_list(
    project_external_payloads: &[ProjectExternalPayload],
) -> String {
    let mut project_external_payloads = project_external_payloads
        .iter()
        .map(|external_payload| {
            (
                external_payload.workflow_slug(),
                external_payload.slice_slug(),
                external_payload.external_payload(),
            )
        })
        .collect::<Vec<_>>();
    project_external_payloads.sort_unstable();
    format!(
        "[{}]",
        project_external_payloads
            .into_iter()
            .map(|(workflow_slug, slice_slug, external_payload)| {
                format!(
                    "{{ workflow := {}, slice := {}, externalPayload := {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(external_payload)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_external_payload_list(
    project_external_payloads: &[ProjectExternalPayload],
) -> String {
    let mut project_external_payloads = project_external_payloads
        .iter()
        .map(|external_payload| {
            (
                external_payload.workflow_slug(),
                external_payload.slice_slug(),
                external_payload.external_payload(),
            )
        })
        .collect::<Vec<_>>();
    project_external_payloads.sort_unstable();
    format!(
        "[{}]",
        project_external_payloads
            .into_iter()
            .map(|(workflow_slug, slice_slug, external_payload)| {
                format!(
                    "{{ workflow: {}, slice: {}, externalPayload: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(external_payload)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_external_payload_field_list(
    project_external_payload_fields: &[ProjectExternalPayloadField],
) -> String {
    let mut fields = project_external_payload_fields.iter().collect::<Vec<_>>();
    fields.sort_unstable();
    format!(
        "[{}]",
        fields
            .into_iter()
            .map(|field| {
                format!(
                    "{{ workflow := {}, slice := {}, externalPayload := {}, field := {}, provenance := {}, bitEncoding := {} }}",
                    json_string(field.workflow_slug()),
                    json_string(field.slice_slug()),
                    json_string(field.external_payload()),
                    json_string(field.field()),
                    json_string(field.provenance()),
                    json_string(field.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_external_payload_field_list(
    project_external_payload_fields: &[ProjectExternalPayloadField],
) -> String {
    let mut fields = project_external_payload_fields.iter().collect::<Vec<_>>();
    fields.sort_unstable();
    format!(
        "[{}]",
        fields
            .into_iter()
            .map(|field| {
                format!(
                    "{{ workflow: {}, slice: {}, externalPayload: {}, field: {}, provenance: {}, bitEncoding: {} }}",
                    json_string(field.workflow_slug()),
                    json_string(field.slice_slug()),
                    json_string(field.external_payload()),
                    json_string(field.field()),
                    json_string(field.provenance()),
                    json_string(field.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_event_list(project_events: &[ProjectEvent]) -> String {
    let mut project_events = project_events
        .iter()
        .map(|event| {
            (
                event.workflow_slug(),
                event.slice_slug(),
                event.event(),
                event.stream(),
            )
        })
        .collect::<Vec<_>>();
    project_events.sort_unstable();
    format!(
        "[{}]",
        project_events
            .into_iter()
            .map(|(workflow_slug, slice_slug, event, stream)| {
                format!(
                    "{{ workflow := {}, slice := {}, event := {}, stream := {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(event),
                    json_string(stream)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_event_list(project_events: &[ProjectEvent]) -> String {
    let mut project_events = project_events
        .iter()
        .map(|event| {
            (
                event.workflow_slug(),
                event.slice_slug(),
                event.event(),
                event.stream(),
            )
        })
        .collect::<Vec<_>>();
    project_events.sort_unstable();
    format!(
        "[{}]",
        project_events
            .into_iter()
            .map(|(workflow_slug, slice_slug, event, stream)| {
                format!(
                    "{{ workflow: {}, slice: {}, event: {}, stream: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(event),
                    json_string(stream)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_event_attribute_list(project_event_attributes: &[ProjectEventAttribute]) -> String {
    let mut project_event_attributes = project_event_attributes
        .iter()
        .map(|attribute| {
            (
                attribute.workflow_slug(),
                attribute.slice_slug(),
                attribute.event(),
                attribute.attribute(),
                attribute.source_kind(),
                attribute.source_name(),
                attribute.source_field(),
                attribute.generated_source_kind(),
                attribute.provenance(),
            )
        })
        .collect::<Vec<_>>();
    project_event_attributes.sort_unstable();
    format!(
        "[{}]",
        project_event_attributes
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    event,
                    attribute,
                    source_kind,
                    source_name,
                    source_field,
                    generated_source_kind,
                    provenance,
                )| {
                    format!(
                        "{{ workflow := {}, slice := {}, event := {}, attributeName := {}, sourceKind := {}, sourceName := {}, sourceField := {}, generatedSourceKind := {}, provenance := {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(event),
                        json_string(attribute),
                        json_string(source_kind),
                        json_string(source_name),
                        json_string(source_field),
                        json_string(generated_source_kind),
                        json_string(provenance)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_event_attribute_list(project_event_attributes: &[ProjectEventAttribute]) -> String {
    let mut project_event_attributes = project_event_attributes
        .iter()
        .map(|attribute| {
            (
                attribute.workflow_slug(),
                attribute.slice_slug(),
                attribute.event(),
                attribute.attribute(),
                attribute.source_kind(),
                attribute.source_name(),
                attribute.source_field(),
                attribute.generated_source_kind(),
                attribute.provenance(),
            )
        })
        .collect::<Vec<_>>();
    project_event_attributes.sort_unstable();
    format!(
        "[{}]",
        project_event_attributes
            .into_iter()
            .map(
                |(
                    workflow_slug,
                    slice_slug,
                    event,
                    attribute,
                    source_kind,
                    source_name,
                    source_field,
                    generated_source_kind,
                    provenance,
                )| {
                    format!(
                        "{{ workflow: {}, slice: {}, event: {}, attribute: {}, sourceKind: {}, sourceName: {}, sourceField: {}, generatedSourceKind: {}, provenance: {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(event),
                        json_string(attribute),
                        json_string(source_kind),
                        json_string(source_name),
                        json_string(source_field),
                        json_string(generated_source_kind),
                        json_string(provenance)
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn model_digest(
    project_name: &ProjectName,
    modeled_workflows: &[ModeledWorkflowLayout],
    formal_workflows: &[FormalWorkflowGraph],
    inventories: &ProjectRootInventories<'_>,
) -> String {
    let canonical_source = format!(
        "project:name={};version=0.1.0;workflows={};slices={};scenarios={};scenario-definitions={};data-flows={};outcomes={};command-errors={};commands={};command-inputs={};read-models={};read-model-definitions={};read-model-fields={};views={};view-definitions={};view-controls={};board-elements={};board-connections={};view-fields={};automations={};automation-definitions={};translations={};translation-definitions={};external-payloads={};external-payload-fields={};streams={};events={};event-attributes={}",
        project_name.as_ref(),
        digest_workflows(modeled_workflows),
        digest_slices(formal_workflows),
        digest_scenarios(inventories.scenarios),
        digest_scenario_definitions(inventories.scenario_definitions),
        digest_data_flows(inventories.data_flows),
        digest_outcomes(inventories.outcomes),
        digest_command_errors(inventories.command_errors),
        digest_commands(inventories.commands),
        digest_command_inputs(inventories.command_inputs),
        digest_read_models(inventories.read_models),
        digest_read_model_definitions(inventories.read_model_definitions),
        digest_read_model_fields(inventories.read_model_fields),
        digest_views(inventories.views),
        digest_view_definitions(inventories.view_definitions),
        digest_view_controls(inventories.view_controls),
        digest_board_elements(inventories.board_elements),
        digest_board_connections(inventories.board_connections),
        digest_view_fields(inventories.view_fields),
        digest_automations(inventories.automations),
        digest_automation_definitions(inventories.automation_definitions),
        digest_translations(inventories.translations),
        digest_translation_definitions(inventories.translation_definitions),
        digest_external_payloads(inventories.external_payloads),
        digest_external_payload_fields(inventories.external_payload_fields),
        digest_streams(inventories.streams),
        digest_events(inventories.events),
        digest_event_attributes(inventories.event_attributes)
    );
    hex::encode(Sha256::digest(canonical_source.as_bytes()))
}

fn digest_outcomes(project_outcomes: &[ProjectOutcome]) -> String {
    let mut outcomes = project_outcomes.iter().collect::<Vec<_>>();
    outcomes.sort_unstable();
    outcomes
        .into_iter()
        .map(|outcome| {
            format!(
                "{}/{}/{}@{}#{}",
                outcome.workflow_slug(),
                outcome.slice_slug(),
                outcome.outcome(),
                outcome.events().join("+"),
                outcome.externally_relevant()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_workflows(modeled_workflows: &[ModeledWorkflowLayout]) -> String {
    let mut workflow_slugs = modeled_workflows
        .iter()
        .map(|workflow| workflow.slug().as_ref())
        .collect::<Vec<_>>();
    workflow_slugs.sort_unstable();
    workflow_slugs.join(",")
}

fn digest_slices(formal_workflows: &[FormalWorkflowGraph]) -> String {
    let mut memberships = formal_workflows
        .iter()
        .flat_map(|workflow| {
            workflow.slice_details().as_slice().iter().map(|slice| {
                (
                    workflow.slug().as_ref(),
                    slice.slug().as_ref(),
                    module_name_from_model(slice.name().clone()),
                )
            })
        })
        .collect::<Vec<_>>();
    memberships.sort_unstable();
    memberships
        .into_iter()
        .map(|(workflow_slug, slice_slug, slice_module)| {
            format!("{workflow_slug}/{slice_slug}@{slice_module}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_scenarios(project_scenarios: &[ProjectScenario]) -> String {
    let mut scenarios = project_scenarios
        .iter()
        .map(|scenario| {
            (
                scenario.workflow_slug(),
                scenario.slice_slug(),
                scenario.scenario_kind(),
                scenario.scenario(),
            )
        })
        .collect::<Vec<_>>();
    scenarios.sort_unstable();
    scenarios
        .into_iter()
        .map(|(workflow_slug, slice_slug, scenario_kind, scenario)| {
            format!("{workflow_slug}/{slice_slug}/{scenario_kind}/{scenario}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_scenario_definitions(scenario_definitions: &[ProjectScenarioDefinition]) -> String {
    let mut scenario_definitions = scenario_definitions.iter().collect::<Vec<_>>();
    scenario_definitions.sort_unstable();
    scenario_definitions
        .into_iter()
        .map(|scenario| {
            format!(
                "{}/{}/{}/{}@{}~{}~{}#{}#{}#{}#{}#{}",
                scenario.workflow_slug(),
                scenario.slice_slug(),
                scenario.scenario_kind(),
                scenario.scenario(),
                scenario.given(),
                scenario.when(),
                scenario.then(),
                scenario.read_streams().join("+"),
                scenario.written_streams().join("+"),
                scenario.contract_kind(),
                scenario.covered_definition(),
                scenario.error_references().join("+")
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_data_flows(project_data_flows: &[ProjectDataFlow]) -> String {
    project_data_flows
        .iter()
        .map(|data_flow| {
            format!(
                "{}/{}/{}@{}:{}~{}~{}#{}",
                data_flow.workflow_slug(),
                data_flow.slice_slug(),
                data_flow.datum(),
                data_flow.source_kind(),
                data_flow.source(),
                data_flow.transformation(),
                data_flow.target(),
                data_flow.bit_encoding()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_command_errors(project_command_errors: &[ProjectCommandError]) -> String {
    project_command_errors
        .iter()
        .map(|command_error| {
            format!(
                "{}/{}/{}/{}@{}#{}",
                command_error.workflow_slug(),
                command_error.slice_slug(),
                command_error.command(),
                command_error.error(),
                command_error.scenario(),
                command_error.recovery()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_streams(project_streams: &[ProjectStream]) -> String {
    let mut streams = project_streams
        .iter()
        .map(|stream| (stream.workflow_slug(), stream.slice_slug(), stream.stream()))
        .collect::<Vec<_>>();
    streams.sort_unstable();
    streams
        .into_iter()
        .map(|(workflow_slug, slice_slug, stream)| format!("{workflow_slug}/{slice_slug}/{stream}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_commands(project_commands: &[ProjectCommand]) -> String {
    let mut commands = project_commands
        .iter()
        .map(|command| {
            (
                command.workflow_slug(),
                command.slice_slug(),
                command.command(),
            )
        })
        .collect::<Vec<_>>();
    commands.sort_unstable();
    commands
        .into_iter()
        .map(|(workflow_slug, slice_slug, command)| {
            format!("{workflow_slug}/{slice_slug}/{command}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_command_inputs(project_command_inputs: &[ProjectCommandInput]) -> String {
    let mut command_inputs = project_command_inputs.iter().collect::<Vec<_>>();
    command_inputs.sort_unstable();
    command_inputs
        .into_iter()
        .map(|command_input| {
            format!(
                "{}/{}/{}/{}@{}#{}#{}#{}#{}#{}#{}#{}#{}#{}#{}#{}#{}",
                command_input.workflow_slug(),
                command_input.slice_slug(),
                command_input.command(),
                command_input.input(),
                command_input.source_kind(),
                command_input.source_description(),
                command_input.provenance_chain().join(" -> "),
                command_input.event_stream_source_event(),
                command_input.event_stream_source_attribute(),
                command_input.external_payload_source_name(),
                command_input.external_payload_source_field(),
                command_input.generated_source_name(),
                command_input.generated_source_field(),
                command_input.session_source_name(),
                command_input.session_source_field(),
                command_input.invocation_argument_source_name(),
                command_input.invocation_argument_source_field()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_read_models(project_read_models: &[ProjectReadModel]) -> String {
    let mut read_models = project_read_models
        .iter()
        .map(|read_model| {
            (
                read_model.workflow_slug(),
                read_model.slice_slug(),
                read_model.read_model(),
            )
        })
        .collect::<Vec<_>>();
    read_models.sort_unstable();
    read_models
        .into_iter()
        .map(|(workflow_slug, slice_slug, read_model)| {
            format!("{workflow_slug}/{slice_slug}/{read_model}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_read_model_definitions(
    project_read_model_definitions: &[ProjectReadModelDefinition],
) -> String {
    let mut read_model_definitions = project_read_model_definitions.iter().collect::<Vec<_>>();
    read_model_definitions.sort_unstable();
    read_model_definitions
        .into_iter()
        .map(|definition| {
            format!(
                "{}/{}/{}@{}#{}#{}#{}",
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.read_model(),
                definition.transitive(),
                definition.relationship_fields().join("+"),
                definition.transitive_rule(),
                definition.example_scenario_name()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_read_model_fields(project_read_model_fields: &[ProjectReadModelField]) -> String {
    let mut read_model_fields = project_read_model_fields
        .iter()
        .map(|field| {
            (
                field.workflow_slug(),
                field.slice_slug(),
                field.read_model(),
                field.field(),
                field.source_kind(),
                field.source_event(),
                field.source_attribute(),
                field.derivation_rule(),
                field.derivation_source_fields(),
                field.absence_event(),
                field.derivation_scenario_name(),
                field.absence_scenario_name(),
                field.provenance(),
            )
        })
        .collect::<Vec<_>>();
    read_model_fields.sort_unstable_by(|left, right| {
        left.0
            .cmp(right.0)
            .then_with(|| left.1.cmp(right.1))
            .then_with(|| left.2.cmp(right.2))
            .then_with(|| left.3.cmp(right.3))
    });
    read_model_fields
        .into_iter()
        .map(
            |(
                workflow_slug,
                slice_slug,
                read_model,
                field,
                source_kind,
                source_event,
                source_attribute,
                derivation_rule,
                derivation_source_fields,
                absence_event,
                derivation_scenario_name,
                absence_scenario_name,
                provenance,
            )| {
                format!(
                    "{workflow_slug}/{slice_slug}/{read_model}/{field}@{source_kind}#{source_event}.{source_attribute}#{derivation_rule}#{}#{absence_event}#{derivation_scenario_name}#{absence_scenario_name}#{provenance}",
                    derivation_source_fields.join("|")
                )
            },
        )
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_views(project_views: &[ProjectView]) -> String {
    let mut views = project_views
        .iter()
        .map(|view| (view.workflow_slug(), view.slice_slug(), view.view()))
        .collect::<Vec<_>>();
    views.sort_unstable();
    views
        .into_iter()
        .map(|(workflow_slug, slice_slug, view)| format!("{workflow_slug}/{slice_slug}/{view}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_view_definitions(project_view_definitions: &[ProjectViewDefinition]) -> String {
    let mut view_definitions = project_view_definitions.to_vec();
    view_definitions.sort();
    view_definitions
        .into_iter()
        .map(|definition| {
            format!(
                "{}/{}/{}@{}#{}#{}#{}",
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.view(),
                definition.read_models().join("|"),
                definition.sketch_tokens().join("|"),
                definition.local_states().join("|"),
                definition.filters().join("|")
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_view_controls(project_view_controls: &[ProjectViewControl]) -> String {
    let mut view_controls = project_view_controls.to_vec();
    view_controls.sort();
    view_controls
        .into_iter()
        .map(|control| {
            format!(
                "{}/{}/{}/{}@{}#{}:{}:{}:{}:{}:{}#{}#{}#{}#{}:{}:{}:{}:{}",
                control.workflow_slug(),
                control.slice_slug(),
                control.view(),
                control.control(),
                control.command(),
                control.input(),
                control.input_source_kind(),
                control.input_source_description(),
                control.input_sketch_token(),
                control.input_visible_to_actor(),
                control.input_decision_field(),
                control.handled_errors().join("|"),
                control.recovery_behavior(),
                control.control_sketch_token(),
                control.navigation_type(),
                control.navigation_target(),
                control.external_workflow(),
                control.external_system(),
                control.handoff_contract()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_board_elements(project_board_elements: &[ProjectBoardElement]) -> String {
    let mut board_elements = project_board_elements.to_vec();
    board_elements.sort();
    board_elements
        .into_iter()
        .map(|element| {
            format!(
                "{}/{}/{}@{}:{}:{}:{}",
                element.workflow_slug(),
                element.slice_slug(),
                element.element(),
                element.kind(),
                element.lane(),
                element.declared_name(),
                element.main_path()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_board_connections(project_board_connections: &[ProjectBoardConnection]) -> String {
    let mut board_connections = project_board_connections.to_vec();
    board_connections.sort();
    board_connections
        .into_iter()
        .map(|connection| {
            format!(
                "{}/{}:{}:{}->{}:{}",
                connection.workflow_slug(),
                connection.slice_slug(),
                connection.source(),
                connection.source_kind(),
                connection.target(),
                connection.target_kind()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_view_fields(project_view_fields: &[ProjectViewField]) -> String {
    let mut view_fields = project_view_fields.iter().collect::<Vec<_>>();
    view_fields.sort_unstable();
    view_fields
        .into_iter()
        .map(|field| {
            format!(
                "{}/{}/{}/{}@{}#{}.{}#{}#{}",
                field.workflow_slug(),
                field.slice_slug(),
                field.view(),
                field.field(),
                field.source_kind(),
                field.source_read_model(),
                field.source_field(),
                field.provenance(),
                field.bit_encoding()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_automations(project_automations: &[ProjectAutomation]) -> String {
    let mut automations = project_automations
        .iter()
        .map(|automation| {
            (
                automation.workflow_slug(),
                automation.slice_slug(),
                automation.automation(),
            )
        })
        .collect::<Vec<_>>();
    automations.sort_unstable();
    automations
        .into_iter()
        .map(|(workflow_slug, slice_slug, automation)| {
            format!("{workflow_slug}/{slice_slug}/{automation}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_automation_definitions(definitions: &[ProjectAutomationDefinition]) -> String {
    let mut definitions = definitions.iter().collect::<Vec<_>>();
    definitions.sort_unstable();
    definitions
        .into_iter()
        .map(|definition| {
            format!(
                "{}/{}/{}@{}#{}#{}#{}",
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.automation(),
                definition.trigger(),
                definition.command(),
                definition.handled_errors().join("|"),
                definition.reaction()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_translations(project_translations: &[ProjectTranslation]) -> String {
    let mut translations = project_translations
        .iter()
        .map(|translation| {
            (
                translation.workflow_slug(),
                translation.slice_slug(),
                translation.translation(),
            )
        })
        .collect::<Vec<_>>();
    translations.sort_unstable();
    translations
        .into_iter()
        .map(|(workflow_slug, slice_slug, translation)| {
            format!("{workflow_slug}/{slice_slug}/{translation}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_translation_definitions(definitions: &[ProjectTranslationDefinition]) -> String {
    let mut definitions = definitions
        .iter()
        .map(|definition| {
            (
                definition.workflow_slug(),
                definition.slice_slug(),
                definition.translation(),
                definition.external_event(),
                definition.payload_contract(),
                definition.command(),
            )
        })
        .collect::<Vec<_>>();
    definitions.sort_unstable();
    definitions
        .into_iter()
        .map(
            |(workflow_slug, slice_slug, translation, external_event, payload_contract, command)| {
                format!(
                    "{workflow_slug}/{slice_slug}/{translation}@{external_event}#{payload_contract}#{command}"
                )
            },
        )
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_external_payloads(project_external_payloads: &[ProjectExternalPayload]) -> String {
    let mut external_payloads = project_external_payloads
        .iter()
        .map(|external_payload| {
            (
                external_payload.workflow_slug(),
                external_payload.slice_slug(),
                external_payload.external_payload(),
            )
        })
        .collect::<Vec<_>>();
    external_payloads.sort_unstable();
    external_payloads
        .into_iter()
        .map(|(workflow_slug, slice_slug, external_payload)| {
            format!("{workflow_slug}/{slice_slug}/{external_payload}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_external_payload_fields(
    project_external_payload_fields: &[ProjectExternalPayloadField],
) -> String {
    let mut fields = project_external_payload_fields.iter().collect::<Vec<_>>();
    fields.sort_unstable();
    fields
        .into_iter()
        .map(|field| {
            format!(
                "{}/{}/{}/{}@{}#{}",
                field.workflow_slug(),
                field.slice_slug(),
                field.external_payload(),
                field.field(),
                field.provenance(),
                field.bit_encoding()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_events(project_events: &[ProjectEvent]) -> String {
    let mut events = project_events
        .iter()
        .map(|event| {
            (
                event.workflow_slug(),
                event.slice_slug(),
                event.event(),
                event.stream(),
            )
        })
        .collect::<Vec<_>>();
    events.sort_unstable();
    events
        .into_iter()
        .map(|(workflow_slug, slice_slug, event, stream)| {
            format!("{workflow_slug}/{slice_slug}/{event}@{stream}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_event_attributes(project_event_attributes: &[ProjectEventAttribute]) -> String {
    let mut event_attributes = project_event_attributes
        .iter()
        .map(|attribute| {
            (
                attribute.workflow_slug(),
                attribute.slice_slug(),
                attribute.event(),
                attribute.attribute(),
                attribute.source_kind(),
                attribute.source_name(),
                attribute.source_field(),
                attribute.generated_source_kind(),
                attribute.provenance(),
            )
        })
        .collect::<Vec<_>>();
    event_attributes.sort_unstable();
    event_attributes
        .into_iter()
        .map(
            |(
                workflow_slug,
                slice_slug,
                event,
                attribute,
                source_kind,
                source_name,
                source_field,
                generated_source_kind,
                provenance,
            )| {
                format!(
                    "{workflow_slug}/{slice_slug}/{event}/{attribute}@{source_kind}#{source_name}.{source_field}#{generated_source_kind}#{provenance}"
                )
            },
        )
        .collect::<Vec<_>>()
        .join(",")
}

fn module_name(project_name: &ProjectName) -> String {
    module_name_from_raw(project_name.as_ref())
}

fn module_name_from_model(model_name: ModelName) -> String {
    module_name_from_raw(model_name.as_ref())
}

fn module_name_from_raw(raw: &str) -> String {
    let mut capitalize_next = true;
    raw.chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                let next = if capitalize_next {
                    character.to_ascii_uppercase()
                } else {
                    character
                };
                capitalize_next = false;
                Some(next)
            } else {
                capitalize_next = true;
                None
            }
        })
        .collect()
}

fn lean_module_name(value: impl Into<String>) -> LeanModuleName {
    LeanModuleName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Lean4 module name must be valid: {error}");
    })
}

fn quint_module_name(value: impl Into<String>) -> QuintModuleName {
    QuintModuleName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Quint module name must be valid: {error}");
    })
}

fn project_path(value: impl Into<String>) -> ProjectPath {
    ProjectPath::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static project path must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static report line must be valid: {error}");
    })
}

fn canonical_declaration_prefix(value: impl Into<String>) -> CanonicalDeclarationPrefix {
    CanonicalDeclarationPrefix::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static canonical declaration prefix must be valid: {error}");
    })
}

fn canonical_declaration_marker(value: impl Into<String>) -> CanonicalDeclarationMarker {
    CanonicalDeclarationMarker::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static artifact marker must be valid: {error}");
    })
}

fn artifact_file_extension(value: impl Into<String>) -> ArtifactFileExtension {
    ArtifactFileExtension::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static artifact file extension must be valid: {error}");
    })
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated JSON string must be valid: {error}");
    })
}

fn json_string_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",")
}
