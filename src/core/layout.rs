use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
use crate::core::effect::{
    ArtifactFileExtension, ArtifactMarker, Effect, EffectPlan, FileContents, ProjectPath,
    ReportLine,
};
use crate::core::emit::lean::emit_slice_module as emit_lean_slice_module;
use crate::core::emit::quint::emit_slice_module as emit_quint_slice_module;
use crate::core::formal_graph::{FormalWorkflowGraph, FormalWorkflowGraphs};
use crate::core::formal_project_facts::{
    ProjectAutomation, ProjectAutomationDefinition, ProjectBoardConnection, ProjectBoardElement,
    ProjectCommand, ProjectCommandError, ProjectCommandInput, ProjectDataFlow, ProjectEvent,
    ProjectEventAttribute, ProjectExternalPayload, ProjectExternalPayloadField, ProjectOutcome,
    ProjectReadModel, ProjectReadModelDefinition, ProjectReadModelField, ProjectScenario,
    ProjectScenarioDefinition, ProjectStream, ProjectTranslation, ProjectTranslationDefinition,
    ProjectView, ProjectViewControl, ProjectViewDefinition, ProjectViewField,
};
use crate::core::project::ProjectName;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, WorkflowSliceDetail,
    WorkflowSlug, WorkflowTransitionRecord,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModeledWorkflowLayout {
    name: ModelName,
    description: ModelDescription,
    slug: WorkflowSlug,
}

impl ModeledWorkflowLayout {
    pub fn new(name: ModelName, description: ModelDescription, slug: WorkflowSlug) -> Self {
        Self {
            name,
            description,
            slug,
        }
    }

    pub fn name(&self) -> &ModelName {
        &self.name
    }

    pub fn description(&self) -> &ModelDescription {
        &self.description
    }

    pub fn slug(&self) -> &WorkflowSlug {
        &self.slug
    }

    pub fn lean_artifact_path(&self) -> ProjectPath {
        let module_name = module_name_from_model(self.name.clone());
        project_path(format!("model/lean/{module_name}.lean"))
    }

    pub fn quint_artifact_path(&self) -> ProjectPath {
        let module_name = module_name_from_model(self.name.clone());
        project_path(format!("model/quint/{module_name}.qnt"))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModeledWorkflowLayouts {
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
pub struct ModeledWorkflowSliceDetails {
    slices: Vec<WorkflowSliceDetail>,
}

impl ModeledWorkflowSliceDetails {
    pub(crate) fn new(slices: Vec<WorkflowSliceDetail>) -> Self {
        Self { slices }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModeledWorkflowTransitions {
    transitions: Vec<WorkflowTransitionRecord>,
}

impl ModeledWorkflowTransitions {
    pub(crate) fn new(transitions: Vec<WorkflowTransitionRecord>) -> Self {
        Self { transitions }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModeledProjectRootInventories {
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

pub fn check_project(
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
                Effect::RequireFile(project_path("model/quint/quint.json")),
                Effect::RequireFile(project_path(format!("model/quint/{module_name}.qnt"))),
                Effect::RequireFile(project_path("model/quint/slices/.gitkeep")),
                Effect::RequireOnlyModeledArtifacts(
                    project_path("model/lean"),
                    artifact_file_extension(".lean"),
                    lean_artifact_paths,
                    report_line("Lean model artifact drift"),
                ),
                Effect::RequireOnlyModeledArtifacts(
                    project_path("model/quint"),
                    artifact_file_extension(".qnt"),
                    quint_artifact_paths,
                    report_line("Quint model artifact drift"),
                ),
                Effect::RequireOnlyModeledArtifacts(
                    project_path("model/lean/slices"),
                    artifact_file_extension(".lean"),
                    lean_slice_artifact_paths,
                    report_line("Lean slice artifact drift"),
                ),
                Effect::RequireOnlyModeledArtifacts(
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
    let lean_model_slice_list = lean_model_slice_list(formal_workflows);
    let lean_model_slice_module_list = lean_model_slice_module_list(formal_workflows);
    let lean_model_scenario_list = lean_model_scenario_list(inventories.scenarios);
    let lean_model_scenario_definition_list =
        lean_model_scenario_definition_list(inventories.scenario_definitions);
    let lean_model_data_flow_list = lean_model_data_flow_list(inventories.data_flows);
    let lean_model_outcome_list = lean_model_outcome_list(inventories.outcomes);
    let lean_model_command_error_list = lean_model_command_error_list(inventories.command_errors);
    let lean_model_command_list = lean_model_command_list(inventories.commands);
    let lean_model_command_input_list = lean_model_command_input_list(inventories.command_inputs);
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
    let quint_model_slice_list = quint_model_slice_list(formal_workflows);
    let quint_model_slice_module_list = quint_model_slice_module_list(formal_workflows);
    let quint_model_scenario_list = quint_model_scenario_list(inventories.scenarios);
    let quint_model_scenario_definition_list =
        quint_model_scenario_definition_list(inventories.scenario_definitions);
    let quint_model_data_flow_list = quint_model_data_flow_list(inventories.data_flows);
    let quint_model_outcome_list = quint_model_outcome_list(inventories.outcomes);
    let quint_model_command_error_list = quint_model_command_error_list(inventories.command_errors);
    let quint_model_command_list = quint_model_command_list(inventories.commands);
    let quint_model_command_input_list = quint_model_command_input_list(inventories.command_inputs);
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
    let quint_config_path = project_path("model/quint/quint.json");
    let manifest_message = report_line(format!("project manifest drift for {project_name_text}"));
    let lean_message = report_line(format!("Lean project root drift for {project_name_text}"));
    let lean_config_message =
        report_line(format!("Lean project config drift for {project_name_text}"));
    let quint_message = report_line(format!("Quint project root drift for {project_name_text}"));
    let quint_config_message = report_line(format!(
        "Quint project config drift for {project_name_text}"
    ));
    let quint_module_close_marker = artifact_marker("}");

    vec![
        Effect::RequireCanonicalDeclaration(
            manifest_path.clone(),
            artifact_marker("name ="),
            artifact_marker(format!("name = {}", json_string(project_name_text))),
            manifest_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            manifest_path.clone(),
            artifact_marker("version ="),
            artifact_marker(format!("version = {}", json_string(model_version))),
            manifest_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            manifest_path.clone(),
            artifact_marker("lean_module ="),
            artifact_marker(format!("lean_module = \"{module_name}\"")),
            manifest_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            manifest_path,
            artifact_marker("quint_module ="),
            artifact_marker(format!("quint_module = \"{module_name}\"")),
            manifest_message,
        ),
        Effect::RequireCanonicalDeclaration(
            lakefile_path,
            artifact_marker("package "),
            artifact_marker("package EMCModel"),
            lean_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_toolchain_path,
            artifact_marker("leanprover/lean4:"),
            artifact_marker("leanprover/lean4:4.29.1"),
            lean_config_message,
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("namespace "),
            artifact_marker(format!("namespace {module_name}")),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelVersion :="),
            artifact_marker(format!(
                "def modelVersion := {}",
                json_string(model_version)
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelName :="),
            artifact_marker(format!(
                "def modelName := {}",
                json_string(project_name_text)
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelDigest :="),
            artifact_marker(format!("def modelDigest := {}", json_string(&model_digest))),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelWorkflows :"),
            artifact_marker(format!(
                "def modelWorkflows : List String := {workflow_slug_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelSlices :"),
            artifact_marker(format!(
                "def modelSlices : List (String × String) := {lean_model_slice_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelSliceModules :"),
            artifact_marker(format!(
                "def modelSliceModules : List (String × String × String) := {lean_model_slice_module_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelScenarios :"),
            artifact_marker(format!(
                "def modelScenarios : List (String × String × String × String) := {lean_model_scenario_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelScenarioDefinitions :"),
            artifact_marker(format!(
                "def modelScenarioDefinitions : List (String × String × String × String × String × String × String × List String × List String × String × String × List String) := {lean_model_scenario_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelDataFlows :"),
            artifact_marker(format!(
                "def modelDataFlows : List (String × String × String × String × String × String × String) := {lean_model_data_flow_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelOutcomes :"),
            artifact_marker(format!(
                "def modelOutcomes : List (String × String × String × List String × Bool) := {lean_model_outcome_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelCommandErrors :"),
            artifact_marker(format!(
                "def modelCommandErrors : List (String × String × String × String × String × String) := {lean_model_command_error_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelCommands :"),
            artifact_marker(format!(
                "def modelCommands : List (String × String × String) := {lean_model_command_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelCommandInputs :"),
            artifact_marker(format!(
                "def modelCommandInputs : List (String × String × String × String × String × String × List String × String × String) := {lean_model_command_input_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelReadModels :"),
            artifact_marker(format!(
                "def modelReadModels : List (String × String × String) := {lean_model_read_model_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelReadModelDefinitions :"),
            artifact_marker(format!(
                "def modelReadModelDefinitions : List (String × String × String × Bool × List String × String × String) := {lean_model_read_model_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelReadModelFields :"),
            artifact_marker(format!(
                "def modelReadModelFields : List (String × String × String × String × String × String × String × String × List String × String × String × String × String) := {lean_model_read_model_field_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelViews :"),
            artifact_marker(format!(
                "def modelViews : List (String × String × String) := {lean_model_view_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelViewDefinitions :"),
            artifact_marker(format!(
                "def modelViewDefinitions : List (String × String × String × List String × List String × List String × List String) := {lean_model_view_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelViewControls :"),
            artifact_marker(format!(
                "def modelViewControls : List (String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) := {lean_model_view_control_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelBoardElements :"),
            artifact_marker(format!(
                "def modelBoardElements : List (String × String × String × String × String × String × Bool) := {lean_model_board_element_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelBoardConnections :"),
            artifact_marker(format!(
                "def modelBoardConnections : List (String × String × String × String × String × String) := {lean_model_board_connection_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelViewFields :"),
            artifact_marker(format!(
                "def modelViewFields : List (String × String × String × String × String × String × String × String × String) := {lean_model_view_field_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelAutomations :"),
            artifact_marker(format!(
                "def modelAutomations : List (String × String × String) := {lean_model_automation_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelAutomationDefinitions :"),
            artifact_marker(format!(
                "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := {lean_model_automation_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelTranslations :"),
            artifact_marker(format!(
                "def modelTranslations : List (String × String × String) := {lean_model_translation_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelTranslationDefinitions :"),
            artifact_marker(format!(
                "def modelTranslationDefinitions : List (String × String × String × String × String × String) := {lean_model_translation_definition_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelExternalPayloads :"),
            artifact_marker(format!(
                "def modelExternalPayloads : List (String × String × String) := {lean_model_external_payload_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelExternalPayloadFields :"),
            artifact_marker(format!(
                "def modelExternalPayloadFields : List (String × String × String × String × String × String) := {lean_model_external_payload_field_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelStreams :"),
            artifact_marker(format!(
                "def modelStreams : List (String × String × String) := {lean_model_stream_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelEvents :"),
            artifact_marker(format!(
                "def modelEvents : List (String × String × String × String) := {lean_model_event_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("def modelEventAttributes :"),
            artifact_marker(format!(
                "def modelEventAttributes : List (String × String × String × String × String × String × String × String) := {lean_model_event_attribute_list}"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelIdentityIsStable"),
            artifact_marker(format!(
                "theorem modelIdentityIsStable : modelName = {} := rfl",
                json_string(project_name_text)
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelVersionIsStable"),
            artifact_marker(format!(
                "theorem modelVersionIsStable : modelVersion = {} := rfl",
                json_string(model_version)
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelDigestIsStable"),
            artifact_marker(format!(
                "theorem modelDigestIsStable : modelDigest = {} := rfl",
                json_string(&model_digest)
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelWorkflowsAreDeclared"),
            artifact_marker(format!(
                "theorem modelWorkflowsAreDeclared : modelWorkflows.length = {workflow_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelSlicesAreDeclared"),
            artifact_marker(format!(
                "theorem modelSlicesAreDeclared : modelSlices.length = {slice_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelSliceModulesAreDeclared"),
            artifact_marker(format!(
                "theorem modelSliceModulesAreDeclared : modelSliceModules.length = {slice_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelScenariosAreDeclared"),
            artifact_marker(format!(
                "theorem modelScenariosAreDeclared : modelScenarios.length = {scenario_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelScenarioDefinitionsAreDeclared"),
            artifact_marker(format!(
                "theorem modelScenarioDefinitionsAreDeclared : modelScenarioDefinitions.length = {scenario_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelDataFlowsAreDeclared"),
            artifact_marker(format!(
                "theorem modelDataFlowsAreDeclared : modelDataFlows.length = {data_flow_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelOutcomesAreDeclared"),
            artifact_marker(format!(
                "theorem modelOutcomesAreDeclared : modelOutcomes.length = {outcome_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelCommandErrorsAreDeclared"),
            artifact_marker(format!(
                "theorem modelCommandErrorsAreDeclared : modelCommandErrors.length = {command_error_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelCommandsAreDeclared"),
            artifact_marker(format!(
                "theorem modelCommandsAreDeclared : modelCommands.length = {command_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelCommandInputsAreDeclared"),
            artifact_marker(format!(
                "theorem modelCommandInputsAreDeclared : modelCommandInputs.length = {command_input_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelReadModelsAreDeclared"),
            artifact_marker(format!(
                "theorem modelReadModelsAreDeclared : modelReadModels.length = {read_model_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelReadModelDefinitionsAreDeclared"),
            artifact_marker(format!(
                "theorem modelReadModelDefinitionsAreDeclared : modelReadModelDefinitions.length = {read_model_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelReadModelFieldsAreDeclared"),
            artifact_marker(format!(
                "theorem modelReadModelFieldsAreDeclared : modelReadModelFields.length = {read_model_field_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelViewsAreDeclared"),
            artifact_marker(format!(
                "theorem modelViewsAreDeclared : modelViews.length = {view_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelViewDefinitionsAreDeclared"),
            artifact_marker(format!(
                "theorem modelViewDefinitionsAreDeclared : modelViewDefinitions.length = {view_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelViewControlsAreDeclared"),
            artifact_marker(format!(
                "theorem modelViewControlsAreDeclared : modelViewControls.length = {view_control_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelBoardElementsAreDeclared"),
            artifact_marker(format!(
                "theorem modelBoardElementsAreDeclared : modelBoardElements.length = {board_element_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelBoardConnectionsAreDeclared"),
            artifact_marker(format!(
                "theorem modelBoardConnectionsAreDeclared : modelBoardConnections.length = {board_connection_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelViewFieldsAreDeclared"),
            artifact_marker(format!(
                "theorem modelViewFieldsAreDeclared : modelViewFields.length = {view_field_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelAutomationsAreDeclared"),
            artifact_marker(format!(
                "theorem modelAutomationsAreDeclared : modelAutomations.length = {automation_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelAutomationDefinitionsAreDeclared"),
            artifact_marker(format!(
                "theorem modelAutomationDefinitionsAreDeclared : modelAutomationDefinitions.length = {automation_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelTranslationsAreDeclared"),
            artifact_marker(format!(
                "theorem modelTranslationsAreDeclared : modelTranslations.length = {translation_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelTranslationDefinitionsAreDeclared"),
            artifact_marker(format!(
                "theorem modelTranslationDefinitionsAreDeclared : modelTranslationDefinitions.length = {translation_definition_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelExternalPayloadsAreDeclared"),
            artifact_marker(format!(
                "theorem modelExternalPayloadsAreDeclared : modelExternalPayloads.length = {external_payload_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelExternalPayloadFieldsAreDeclared"),
            artifact_marker(format!(
                "theorem modelExternalPayloadFieldsAreDeclared : modelExternalPayloadFields.length = {external_payload_field_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelStreamsAreDeclared"),
            artifact_marker(format!(
                "theorem modelStreamsAreDeclared : modelStreams.length = {stream_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelEventsAreDeclared"),
            artifact_marker(format!(
                "theorem modelEventsAreDeclared : modelEvents.length = {event_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            artifact_marker("theorem modelEventAttributesAreDeclared"),
            artifact_marker(format!(
                "theorem modelEventAttributesAreDeclared : modelEventAttributes.length = {event_attribute_count} := rfl"
            )),
            lean_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path,
            artifact_marker("end "),
            artifact_marker(format!("end {module_name}")),
            lean_message,
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("module "),
            artifact_marker(format!("module {module_name} {{")),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelSlice ="),
            artifact_marker("  type ModelSlice = { workflow: str, slice: str }"),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelSliceModule ="),
            artifact_marker(
                "  type ModelSliceModule = { workflow: str, slice: str, formalModule: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelScenario ="),
            artifact_marker(
                "  type ModelScenario = { workflow: str, slice: str, scenarioKind: str, scenario: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelScenarioDefinition ="),
            artifact_marker(
                "  type ModelScenarioDefinition = { workflow: str, slice: str, scenarioKind: str, scenario: str, given: str, when: str, then: str, readStreams: List[str], writtenStreams: List[str], contractKind: str, coveredDefinition: str, errorReferences: List[str] }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelDataFlow ="),
            artifact_marker(
                "  type ModelDataFlow = { workflow: str, slice: str, datum: str, source: str, transformation: str, target: str, bitEncoding: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelOutcome ="),
            artifact_marker(
                "  type ModelOutcome = { workflow: str, slice: str, outcome: str, events: List[str], externallyRelevant: bool }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelCommandError ="),
            artifact_marker(
                "  type ModelCommandError = { workflow: str, slice: str, command: str, error: str, scenario: str, recovery: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelCommand ="),
            artifact_marker("  type ModelCommand = { workflow: str, slice: str, command: str }"),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelCommandInput ="),
            artifact_marker(
                "  type ModelCommandInput = { workflow: str, slice: str, command: str, input: str, sourceKind: str, sourceDescription: str, provenanceChain: List[str], eventStreamSourceEvent: str, eventStreamSourceAttribute: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelReadModel ="),
            artifact_marker(
                "  type ModelReadModel = { workflow: str, slice: str, readModel: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelReadModelDefinition ="),
            artifact_marker(
                "  type ModelReadModelDefinition = { workflow: str, slice: str, readModel: str, transitive: bool, relationshipFields: List[str], transitiveRule: str, exampleScenarioName: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelReadModelField ="),
            artifact_marker(
                "  type ModelReadModelField = { workflow: str, slice: str, readModel: str, field: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, derivationSourceFields: List[str], absenceEvent: str, derivationScenarioName: str, absenceScenarioName: str, provenance: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelView ="),
            artifact_marker("  type ModelView = { workflow: str, slice: str, view: str }"),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelViewDefinition ="),
            artifact_marker(
                "  type ModelViewDefinition = { workflow: str, slice: str, view: str, readModels: List[str], sketchTokens: List[str], localStates: List[str], filters: List[str] }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelViewControl ="),
            artifact_marker(
                "  type ModelViewControl = { workflow: str, slice: str, view: str, control: str, command: str, input: str, inputSourceKind: str, inputSourceDescription: str, inputSketchToken: str, inputVisibleToActor: bool, inputDecisionField: bool, handledErrors: List[str], recoveryBehavior: str, controlSketchToken: str, navigationType: str, navigationTarget: str, externalWorkflow: str, externalSystem: str, handoffContract: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelBoardElement ="),
            artifact_marker(
                "  type ModelBoardElement = { workflow: str, slice: str, element: str, kind: str, lane: str, declaredName: str, mainPath: bool }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelBoardConnection ="),
            artifact_marker(
                "  type ModelBoardConnection = { workflow: str, slice: str, source: str, sourceKind: str, target: str, targetKind: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelViewField ="),
            artifact_marker(
                "  type ModelViewField = { workflow: str, slice: str, view: str, field: str, sourceKind: str, sourceReadModel: str, sourceField: str, provenance: str, bitEncoding: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelAutomation ="),
            artifact_marker(
                "  type ModelAutomation = { workflow: str, slice: str, automation: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelAutomationDefinition ="),
            artifact_marker(
                "  type ModelAutomationDefinition = { workflow: str, slice: str, automation: str, trigger: str, command: str, handledErrors: List[str], reaction: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelTranslation ="),
            artifact_marker(
                "  type ModelTranslation = { workflow: str, slice: str, translation: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelTranslationDefinition ="),
            artifact_marker(
                "  type ModelTranslationDefinition = { workflow: str, slice: str, translation: str, externalEvent: str, payloadContract: str, command: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelExternalPayload ="),
            artifact_marker(
                "  type ModelExternalPayload = { workflow: str, slice: str, externalPayload: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelExternalPayloadField ="),
            artifact_marker(
                "  type ModelExternalPayloadField = { workflow: str, slice: str, externalPayload: str, field: str, provenance: str, bitEncoding: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelStream ="),
            artifact_marker("  type ModelStream = { workflow: str, slice: str, stream: str }"),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelEvent ="),
            artifact_marker(
                "  type ModelEvent = { workflow: str, slice: str, event: str, stream: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  type ModelEventAttribute ="),
            artifact_marker(
                "  type ModelEventAttribute = { workflow: str, slice: str, event: str, attribute: str, sourceKind: str, sourceName: str, sourceField: str, provenance: str }",
            ),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelVersion ="),
            artifact_marker(format!(
                "  val modelVersion = {}",
                json_string(model_version)
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelName ="),
            artifact_marker(format!(
                "  val modelName = {}",
                json_string(project_name_text)
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelDigest ="),
            artifact_marker(format!(
                "  val modelDigest = {}",
                json_string(&model_digest)
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelWorkflows:"),
            artifact_marker(format!(
                "  val modelWorkflows: List[str] = {workflow_slug_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelSlices:"),
            artifact_marker(format!(
                "  val modelSlices: List[ModelSlice] = {quint_model_slice_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelSliceModules:"),
            artifact_marker(format!(
                "  val modelSliceModules: List[ModelSliceModule] = {quint_model_slice_module_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelScenarios:"),
            artifact_marker(format!(
                "  val modelScenarios: List[ModelScenario] = {quint_model_scenario_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelScenarioDefinitions:"),
            artifact_marker(format!(
                "  val modelScenarioDefinitions: List[ModelScenarioDefinition] = {quint_model_scenario_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelDataFlows:"),
            artifact_marker(format!(
                "  val modelDataFlows: List[ModelDataFlow] = {quint_model_data_flow_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelOutcomes:"),
            artifact_marker(format!(
                "  val modelOutcomes: List[ModelOutcome] = {quint_model_outcome_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelCommandErrors:"),
            artifact_marker(format!(
                "  val modelCommandErrors: List[ModelCommandError] = {quint_model_command_error_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelCommands:"),
            artifact_marker(format!(
                "  val modelCommands: List[ModelCommand] = {quint_model_command_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelCommandInputs:"),
            artifact_marker(format!(
                "  val modelCommandInputs: List[ModelCommandInput] = {quint_model_command_input_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelReadModels:"),
            artifact_marker(format!(
                "  val modelReadModels: List[ModelReadModel] = {quint_model_read_model_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelReadModelDefinitions:"),
            artifact_marker(format!(
                "  val modelReadModelDefinitions: List[ModelReadModelDefinition] = {quint_model_read_model_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelReadModelFields:"),
            artifact_marker(format!(
                "  val modelReadModelFields: List[ModelReadModelField] = {quint_model_read_model_field_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViews:"),
            artifact_marker(format!(
                "  val modelViews: List[ModelView] = {quint_model_view_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViewDefinitions:"),
            artifact_marker(format!(
                "  val modelViewDefinitions: List[ModelViewDefinition] = {quint_model_view_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViewControls:"),
            artifact_marker(format!(
                "  val modelViewControls: List[ModelViewControl] = {quint_model_view_control_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelBoardElements:"),
            artifact_marker(format!(
                "  val modelBoardElements: List[ModelBoardElement] = {quint_model_board_element_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelBoardConnections:"),
            artifact_marker(format!(
                "  val modelBoardConnections: List[ModelBoardConnection] = {quint_model_board_connection_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViewFields:"),
            artifact_marker(format!(
                "  val modelViewFields: List[ModelViewField] = {quint_model_view_field_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelAutomations:"),
            artifact_marker(format!(
                "  val modelAutomations: List[ModelAutomation] = {quint_model_automation_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelAutomationDefinitions:"),
            artifact_marker(format!(
                "  val modelAutomationDefinitions: List[ModelAutomationDefinition] = {quint_model_automation_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelTranslations:"),
            artifact_marker(format!(
                "  val modelTranslations: List[ModelTranslation] = {quint_model_translation_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelTranslationDefinitions:"),
            artifact_marker(format!(
                "  val modelTranslationDefinitions: List[ModelTranslationDefinition] = {quint_model_translation_definition_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelExternalPayloads:"),
            artifact_marker(format!(
                "  val modelExternalPayloads: List[ModelExternalPayload] = {quint_model_external_payload_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelExternalPayloadFields:"),
            artifact_marker(format!(
                "  val modelExternalPayloadFields: List[ModelExternalPayloadField] = {quint_model_external_payload_field_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelStreams:"),
            artifact_marker(format!(
                "  val modelStreams: List[ModelStream] = {quint_model_stream_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelEvents:"),
            artifact_marker(format!(
                "  val modelEvents: List[ModelEvent] = {quint_model_event_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelEventAttributes:"),
            artifact_marker(format!(
                "  val modelEventAttributes: List[ModelEventAttribute] = {quint_model_event_attribute_list}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelIdentityStable ="),
            artifact_marker(format!(
                "  val modelIdentityStable = modelName == {}",
                json_string(project_name_text)
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelVersionStable ="),
            artifact_marker(format!(
                "  val modelVersionStable = modelVersion == {}",
                json_string(model_version)
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelDigestStable ="),
            artifact_marker(format!(
                "  val modelDigestStable = modelDigest == {}",
                json_string(&model_digest)
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelWorkflowsAreDeclared ="),
            artifact_marker(format!(
                "  val modelWorkflowsAreDeclared = modelWorkflows.length() == {workflow_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelSlicesAreDeclared ="),
            artifact_marker(format!(
                "  val modelSlicesAreDeclared = modelSlices.length() == {slice_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelSliceModulesAreDeclared ="),
            artifact_marker(format!(
                "  val modelSliceModulesAreDeclared = modelSliceModules.length() == {slice_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelScenariosAreDeclared ="),
            artifact_marker(format!(
                "  val modelScenariosAreDeclared = modelScenarios.length() == {scenario_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelScenarioDefinitionsAreDeclared ="),
            artifact_marker(format!(
                "  val modelScenarioDefinitionsAreDeclared = modelScenarioDefinitions.length() == {scenario_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelDataFlowsAreDeclared ="),
            artifact_marker(format!(
                "  val modelDataFlowsAreDeclared = modelDataFlows.length() == {data_flow_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelOutcomesAreDeclared ="),
            artifact_marker(format!(
                "  val modelOutcomesAreDeclared = modelOutcomes.length() == {outcome_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelCommandErrorsAreDeclared ="),
            artifact_marker(format!(
                "  val modelCommandErrorsAreDeclared = modelCommandErrors.length() == {command_error_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelCommandsAreDeclared ="),
            artifact_marker(format!(
                "  val modelCommandsAreDeclared = modelCommands.length() == {command_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelCommandInputsAreDeclared ="),
            artifact_marker(format!(
                "  val modelCommandInputsAreDeclared = modelCommandInputs.length() == {command_input_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelReadModelsAreDeclared ="),
            artifact_marker(format!(
                "  val modelReadModelsAreDeclared = modelReadModels.length() == {read_model_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelReadModelDefinitionsAreDeclared ="),
            artifact_marker(format!(
                "  val modelReadModelDefinitionsAreDeclared = modelReadModelDefinitions.length() == {read_model_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelReadModelFieldsAreDeclared ="),
            artifact_marker(format!(
                "  val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == {read_model_field_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViewsAreDeclared ="),
            artifact_marker(format!(
                "  val modelViewsAreDeclared = modelViews.length() == {view_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViewDefinitionsAreDeclared ="),
            artifact_marker(format!(
                "  val modelViewDefinitionsAreDeclared = modelViewDefinitions.length() == {view_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViewControlsAreDeclared ="),
            artifact_marker(format!(
                "  val modelViewControlsAreDeclared = modelViewControls.length() == {view_control_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelBoardElementsAreDeclared ="),
            artifact_marker(format!(
                "  val modelBoardElementsAreDeclared = modelBoardElements.length() == {board_element_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelBoardConnectionsAreDeclared ="),
            artifact_marker(format!(
                "  val modelBoardConnectionsAreDeclared = modelBoardConnections.length() == {board_connection_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelViewFieldsAreDeclared ="),
            artifact_marker(format!(
                "  val modelViewFieldsAreDeclared = modelViewFields.length() == {view_field_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelAutomationsAreDeclared ="),
            artifact_marker(format!(
                "  val modelAutomationsAreDeclared = modelAutomations.length() == {automation_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelAutomationDefinitionsAreDeclared ="),
            artifact_marker(format!(
                "  val modelAutomationDefinitionsAreDeclared = modelAutomationDefinitions.length() == {automation_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelTranslationsAreDeclared ="),
            artifact_marker(format!(
                "  val modelTranslationsAreDeclared = modelTranslations.length() == {translation_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelTranslationDefinitionsAreDeclared ="),
            artifact_marker(format!(
                "  val modelTranslationDefinitionsAreDeclared = modelTranslationDefinitions.length() == {translation_definition_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelExternalPayloadsAreDeclared ="),
            artifact_marker(format!(
                "  val modelExternalPayloadsAreDeclared = modelExternalPayloads.length() == {external_payload_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelExternalPayloadFieldsAreDeclared ="),
            artifact_marker(format!(
                "  val modelExternalPayloadFieldsAreDeclared = modelExternalPayloadFields.length() == {external_payload_field_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelStreamsAreDeclared ="),
            artifact_marker(format!(
                "  val modelStreamsAreDeclared = modelStreams.length() == {stream_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelEventsAreDeclared ="),
            artifact_marker(format!(
                "  val modelEventsAreDeclared = modelEvents.length() == {event_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            artifact_marker("  val modelEventAttributesAreDeclared ="),
            artifact_marker(format!(
                "  val modelEventAttributesAreDeclared = modelEventAttributes.length() == {event_attribute_count}"
            )),
            quint_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path,
            quint_module_close_marker.clone(),
            quint_module_close_marker,
            quint_message,
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("  \"main\":"),
            artifact_marker(format!("  \"main\": \"{module_name}.qnt\",")),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowIdentityStable\""),
            artifact_marker("    \"workflowIdentityStable\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowSliceDetailsComplete\""),
            artifact_marker("    \"workflowSliceDetailsComplete\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowSliceModulesComplete\""),
            artifact_marker("    \"workflowSliceModulesComplete\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowTransitionsStructured\""),
            artifact_marker("    \"workflowTransitionsStructured\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowTransitionSourcesResolve\""),
            artifact_marker("    \"workflowTransitionSourcesResolve\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowTransitionTargetsResolve\""),
            artifact_marker("    \"workflowTransitionTargetsResolve\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowStepRelationshipsAreAllowed\""),
            artifact_marker("    \"workflowStepRelationshipsAreAllowed\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowStepSlugsAreUnique\""),
            artifact_marker("    \"workflowStepSlugsAreUnique\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowHasExactlyOneEntryStep\""),
            artifact_marker("    \"workflowHasExactlyOneEntryStep\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowMainStepsHaveIncomingReachability\""),
            artifact_marker("    \"workflowMainStepsHaveIncomingReachability\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowNonSupportingStepsReachableFromEntry\""),
            artifact_marker("    \"workflowNonSupportingStepsReachableFromEntry\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\""),
            artifact_marker("    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowTransitionsHaveModeledKinds\""),
            artifact_marker("    \"workflowTransitionsHaveModeledKinds\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowExitsNameTargetsAndRationale\""),
            artifact_marker("    \"workflowExitsNameTargetsAndRationale\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowExternallyRelevantOutcomesHandled\""),
            artifact_marker("    \"workflowExternallyRelevantOutcomesHandled\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowOutcomesSourceResolve\""),
            artifact_marker("    \"workflowOutcomesSourceResolve\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowCommandErrorsSourceResolve\""),
            artifact_marker("    \"workflowCommandErrorsSourceResolve\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\""),
            artifact_marker("    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowNonEventDefinitionsAreUniquelyOwned\""),
            artifact_marker("    \"workflowNonEventDefinitionsAreUniquelyOwned\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowSharedEventDefinitionsHaveIdenticalIdentity\""),
            artifact_marker("    \"workflowSharedEventDefinitionsHaveIdenticalIdentity\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowCommandTransitionsResolveControlsAndCommands\""),
            artifact_marker("    \"workflowCommandTransitionsResolveControlsAndCommands\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowEventTransitionsAreSharedByEndpointSlices\""),
            artifact_marker("    \"workflowEventTransitionsAreSharedByEndpointSlices\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowNavigationTransitionsResolveControlsAndViews\""),
            artifact_marker("    \"workflowNavigationTransitionsResolveControlsAndViews\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowExternalTriggersDeclarePayloadContracts\""),
            artifact_marker("    \"workflowExternalTriggersDeclarePayloadContracts\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowExternalTriggerPayloadContractsHaveProvenance\""),
            artifact_marker("    \"workflowExternalTriggerPayloadContractsHaveProvenance\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path.clone(),
            artifact_marker("    \"workflowTransitionsHaveRequiredEvidence\""),
            artifact_marker("    \"workflowTransitionsHaveRequiredEvidence\","),
            quint_config_message.clone(),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_config_path,
            artifact_marker("    \"workflowEntryLifecycleStatesCoverRequiredStates\""),
            artifact_marker("    \"workflowEntryLifecycleStatesCoverRequiredStates\""),
            quint_config_message,
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

pub fn list_workflows(modeled_workflows: ModeledWorkflowLayouts) -> EffectPlan {
    EffectPlan::new(
        modeled_workflows
            .into_inner()
            .into_iter()
            .map(|workflow| Effect::Report(report_line(workflow.name.as_ref().to_owned())))
            .collect(),
    )
}

pub fn list_slices(modeled_slices: ModeledWorkflowSliceDetails) -> EffectPlan {
    EffectPlan::new(
        modeled_slices
            .slices
            .into_iter()
            .map(|slice| Effect::Report(report_line(slice.name().as_ref().to_owned())))
            .collect(),
    )
}

pub fn list_transitions(modeled_transitions: ModeledWorkflowTransitions) -> EffectPlan {
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

pub fn show_workflow(workflow_document: FileContents) -> EffectPlan {
    show_document(workflow_document)
}

pub fn show_document(document: FileContents) -> EffectPlan {
    EffectPlan::new(vec![Effect::ReportDocument(document)])
}

fn formal_workflow_effects(workflow: &FormalWorkflowGraph) -> Vec<Effect> {
    let workflow_name = workflow.name().as_ref().to_owned();
    let lean_name_marker = artifact_marker(format!(
        "def workflowName := {}",
        json_string(workflow.name().as_ref())
    ));
    let lean_name_prefix = artifact_marker("def workflowName :=");
    let lean_slug_marker = artifact_marker(format!(
        "def workflowSlug := {}",
        json_string(workflow.slug().as_ref())
    ));
    let lean_slug_prefix = artifact_marker("def workflowSlug :=");
    let lean_description_marker = artifact_marker(format!(
        "def workflowDescription := {}",
        json_string(workflow.description().as_ref())
    ));
    let lean_description_prefix = artifact_marker("def workflowDescription :=");
    let quint_name_marker = artifact_marker(format!(
        "val workflowName = {}",
        json_string(workflow.name().as_ref())
    ));
    let quint_name_prefix = artifact_marker("val workflowName =");
    let quint_slug_marker = artifact_marker(format!(
        "val workflowSlug = {}",
        json_string(workflow.slug().as_ref())
    ));
    let quint_slug_prefix = artifact_marker("val workflowSlug =");
    let quint_description_marker = artifact_marker(format!(
        "val workflowDescription = {}",
        json_string(workflow.description().as_ref())
    ));
    let quint_description_prefix = artifact_marker("val workflowDescription =");
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
    let lean_slice_prefix = artifact_marker("def workflowSlices : List String :=");
    let lean_slice_detail_prefix =
        artifact_marker("def workflowSliceDetails : List (String × String × String × String) :=");
    let lean_slice_module_prefix =
        artifact_marker("def workflowSliceModules : List (String × String) :=");
    let lean_transition_prefix =
        artifact_marker("def workflowTransitions : List WorkflowTransition :=");
    let lean_exit_target_prefix = artifact_marker("def workflowExitTargets : List String :=");
    let quint_slice_prefix = artifact_marker("val workflowSlices:");
    let quint_slice_detail_prefix = artifact_marker("val workflowSliceDetails:");
    let quint_slice_module_prefix = artifact_marker("val workflowSliceModules:");
    let quint_transition_prefix = artifact_marker("val workflowTransitions:");
    let quint_exit_target_prefix = artifact_marker("val workflowExitTargets:");
    let lean_identity_invariant_marker = artifact_marker(format!(
        "theorem workflowIdentityIsStable : workflowName = {} := rfl",
        json_string(workflow.name().as_ref())
    ));
    let lean_identity_invariant_prefix = artifact_marker("theorem workflowIdentityIsStable :");
    let lean_slice_detail_invariant_marker = artifact_marker(
        "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl",
    );
    let lean_slice_detail_invariant_prefix = artifact_marker("theorem workflowSlicesHaveDetails :");
    let lean_slice_module_invariant_marker = artifact_marker(
        "theorem workflowSlicesHaveModuleReferences : workflowSlices.length = workflowSliceModules.length := rfl",
    );
    let lean_slice_module_invariant_prefix =
        artifact_marker("theorem workflowSlicesHaveModuleReferences :");
    let lean_transition_invariant_marker = artifact_marker(
        "theorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.source.isEmpty == false && transition.target.isEmpty == false && transition.kind.isEmpty == false && transition.trigger.isEmpty == false) = true := rfl",
    );
    let lean_transition_invariant_prefix =
        artifact_marker("theorem workflowTransitionsAreStructured :");
    let lean_transition_source_resolution_marker = artifact_marker(
        "theorem workflowTransitionSourcesResolve : workflowTransitions.all (fun transition => workflowSlices.contains transition.source) = true := rfl",
    );
    let lean_transition_source_resolution_prefix =
        artifact_marker("theorem workflowTransitionSourcesResolve :");
    let lean_transition_target_resolution_marker = artifact_marker(
        "theorem workflowTransitionTargetsResolve : workflowTransitions.all (fun transition => workflowSlices.contains transition.target || workflowExitTargets.contains transition.target) = true := rfl",
    );
    let lean_transition_target_resolution_prefix =
        artifact_marker("theorem workflowTransitionTargetsResolve :");
    let quint_identity_invariant_marker = artifact_marker(format!(
        "val workflowIdentityStable = workflowName == {}",
        json_string(workflow.name().as_ref())
    ));
    let quint_identity_invariant_prefix = artifact_marker("val workflowIdentityStable =");
    let quint_slice_detail_invariant_marker = artifact_marker(
        "val workflowSlicesHaveDetails = length(workflowSlices) == length(workflowSliceDetails)",
    );
    let quint_slice_detail_invariant_prefix = artifact_marker("val workflowSlicesHaveDetails =");
    let quint_slice_detail_complete_marker =
        artifact_marker("val workflowSliceDetailsComplete = workflowSlicesHaveDetails");
    let quint_slice_detail_complete_prefix = artifact_marker("val workflowSliceDetailsComplete =");
    let quint_slice_module_complete_marker = artifact_marker(
        "val workflowSliceModulesComplete = workflowSlices.length() == workflowSliceModules.length()",
    );
    let quint_slice_module_complete_prefix = artifact_marker("val workflowSliceModulesComplete =");
    let quint_transition_invariant_marker = artifact_marker(
        "val workflowTransitionsStructured = workflowTransitions.select(transition => transition.source != \"\" and transition.target != \"\" and transition.kind != \"\" and transition.trigger != \"\").length() == workflowTransitions.length()",
    );
    let quint_transition_invariant_prefix = artifact_marker("val workflowTransitionsStructured =");
    let quint_transition_source_resolution_marker = artifact_marker(
        "val workflowTransitionSourcesResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.source).length() > 0).length() == workflowTransitions.length()",
    );
    let quint_transition_source_resolution_prefix =
        artifact_marker("val workflowTransitionSourcesResolve =");
    let quint_transition_target_resolution_marker = artifact_marker(
        "val workflowTransitionTargetsResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.target).length() > 0 or workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() > 0).length() == workflowTransitions.length()",
    );
    let quint_transition_target_resolution_prefix =
        artifact_marker("val workflowTransitionTargetsResolve =");
    let module_name = module_name_from_model(workflow.name().clone());
    let lean_module_marker = artifact_marker(format!("namespace {module_name}"));
    let lean_module_prefix = artifact_marker("namespace ");
    let lean_module_end_marker = artifact_marker(format!("end {module_name}"));
    let lean_module_end_prefix = artifact_marker("end ");
    let quint_module_marker = artifact_marker(format!("module {module_name} {{"));
    let quint_module_prefix = artifact_marker("module ");
    let quint_module_close_marker = artifact_marker("}");
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
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_module_prefix,
            lean_module_marker,
            report_line(format!(
                "Lean workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_module_end_prefix,
            lean_module_end_marker,
            report_line(format!(
                "Lean workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_module_prefix,
            quint_module_marker,
            report_line(format!(
                "Quint workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_module_close_marker.clone(),
            quint_module_close_marker,
            report_line(format!(
                "Quint workflow module drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_name_prefix,
            lean_name_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_slug_prefix,
            lean_slug_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_description_prefix,
            lean_description_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_name_prefix,
            quint_name_marker,
            report_line(format!(
                "Quint workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_slug_prefix,
            quint_slug_marker,
            report_line(format!(
                "Quint workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_description_prefix,
            quint_description_marker,
            report_line(format!(
                "Quint workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_slice_prefix,
            lean_slice_marker,
            report_line(format!(
                "Lean workflow slice drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_slice_detail_prefix,
            lean_slice_detail_marker,
            report_line(format!(
                "Lean workflow slice detail drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_slice_module_prefix,
            lean_slice_module_marker,
            report_line(format!(
                "Lean workflow slice module drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_slice_prefix,
            quint_slice_marker,
            report_line(format!(
                "Quint workflow slice drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_slice_detail_prefix,
            quint_slice_detail_marker,
            report_line(format!(
                "Quint workflow slice detail drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_slice_module_prefix,
            quint_slice_module_marker,
            report_line(format!(
                "Quint workflow slice module drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_transition_prefix,
            lean_transition_marker,
            report_line(format!(
                "Lean workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_exit_target_prefix,
            lean_exit_target_marker,
            report_line(format!(
                "Lean workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_transition_prefix,
            quint_transition_marker,
            report_line(format!(
                "Quint workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_exit_target_prefix,
            quint_exit_target_marker,
            report_line(format!(
                "Quint workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_identity_invariant_prefix,
            lean_identity_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_identity_invariant_prefix,
            quint_identity_invariant_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_slice_detail_invariant_prefix,
            lean_slice_detail_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_slice_module_invariant_prefix,
            lean_slice_module_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_slice_detail_invariant_prefix,
            quint_slice_detail_invariant_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_slice_detail_complete_prefix,
            quint_slice_detail_complete_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_slice_module_complete_prefix,
            quint_slice_module_complete_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_transition_invariant_prefix,
            lean_transition_invariant_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_transition_invariant_prefix,
            quint_transition_invariant_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_transition_source_resolution_prefix,
            lean_transition_source_resolution_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            lean_path.clone(),
            lean_transition_target_resolution_prefix,
            lean_transition_target_resolution_marker,
            report_line(format!(
                "Lean workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_transition_source_resolution_prefix,
            quint_transition_source_resolution_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireCanonicalDeclaration(
            quint_path.clone(),
            quint_transition_target_resolution_prefix,
            quint_transition_target_resolution_marker,
            report_line(format!(
                "Quint workflow invariant drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            lean_path,
            digest.clone(),
            report_line(format!(
                "artifact digest mismatch for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
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
                slice.kind().clone(),
                slice.description().clone(),
            );
            let lean_slice_path = project_path(format!("model/lean/slices/{module_name}.lean"));
            let quint_slice_path = project_path(format!("model/quint/slices/{module_name}.qnt"));

            [
                Effect::RequireFileContentsWithAuthoredFormalFacts(
                    lean_slice_path,
                    emit_lean_slice_module(
                        lean_module_name(module_name.clone()),
                        slice.name().clone(),
                        slice.description().clone(),
                        slice.slug().clone(),
                        slice.kind().clone(),
                        slice_digest.clone(),
                    ),
                    report_line(format!(
                        "Lean slice artifact drift for workflow {workflow_name}"
                    )),
                ),
                Effect::RequireFileContentsWithAuthoredFormalFacts(
                    quint_slice_path,
                    emit_quint_slice_module(
                        quint_module_name(module_name),
                        slice.name().clone(),
                        slice.description().clone(),
                        slice.slug().clone(),
                        slice.kind().clone(),
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

fn lean_workflow_slice_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
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

fn lean_workflow_slice_detail_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
        "def workflowSliceDetails : List (String × String × String × String) := [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| {
                format!(
                    "({}, {}, {}, {})",
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

fn lean_workflow_slice_module_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
        "def workflowSliceModules : List (String × String) := [{}]",
        workflow
            .slice_details()
            .as_slice()
            .iter()
            .map(|slice| {
                format!(
                    "({}, {})",
                    json_string(slice.slug().as_ref()),
                    json_string(&module_name_from_model(slice.name().clone()))
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn lean_workflow_transition_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
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

fn lean_workflow_exit_target_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
        "def workflowExitTargets : List String := [{}]",
        workflow_exit_targets(workflow).join(",")
    ))
}

fn quint_workflow_slice_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
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

fn quint_workflow_slice_detail_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
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

fn quint_workflow_slice_module_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
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

fn quint_workflow_transition_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
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

fn quint_workflow_exit_target_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
        "val workflowExitTargets: List[str] = [{}]",
        workflow_exit_targets(workflow).join(",")
    ))
}

fn workflow_exit_targets(workflow: &FormalWorkflowGraph) -> Vec<String> {
    workflow
        .transitions()
        .as_slice()
        .iter()
        .filter(|transition| transition.kind().as_ref().starts_with("workflow_exit:"))
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

fn lean_model_slice_list(formal_workflows: &[FormalWorkflowGraph]) -> String {
    let mut memberships = formal_workflows
        .iter()
        .flat_map(|workflow| {
            workflow
                .slice_details()
                .as_slice()
                .iter()
                .map(|slice| (workflow.slug().as_ref(), slice.slug().as_ref()))
        })
        .collect::<Vec<_>>();
    memberships.sort_unstable();
    format!(
        "[{}]",
        memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug)| {
                format!(
                    "({}, {})",
                    json_string(workflow_slug),
                    json_string(slice_slug)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_slice_list(formal_workflows: &[FormalWorkflowGraph]) -> String {
    let mut memberships = formal_workflows
        .iter()
        .flat_map(|workflow| {
            workflow
                .slice_details()
                .as_slice()
                .iter()
                .map(|slice| (workflow.slug().as_ref(), slice.slug().as_ref()))
        })
        .collect::<Vec<_>>();
    memberships.sort_unstable();
    format!(
        "[{}]",
        memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug)| {
                format!(
                    "{{ workflow: {}, slice: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn lean_model_slice_module_list(formal_workflows: &[FormalWorkflowGraph]) -> String {
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
    format!(
        "[{}]",
        memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug, slice_module)| {
                format!(
                    "({}, {}, {})",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(&slice_module)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_slice_module_list(formal_workflows: &[FormalWorkflowGraph]) -> String {
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
    format!(
        "[{}]",
        memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug, slice_module)| {
                format!(
                    "{{ workflow: {}, slice: {}, formalModule: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(&slice_module)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn lean_model_scenario_list(project_scenarios: &[ProjectScenario]) -> String {
    let mut project_scenarios = project_scenarios
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
    project_scenarios.sort_unstable();
    format!(
        "[{}]",
        project_scenarios
            .into_iter()
            .map(|(workflow_slug, slice_slug, scenario_kind, scenario)| {
                format!(
                    "({}, {}, {}, {})",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(scenario_kind),
                    json_string(scenario)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_scenario_list(project_scenarios: &[ProjectScenario]) -> String {
    let mut project_scenarios = project_scenarios
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
    project_scenarios.sort_unstable();
    format!(
        "[{}]",
        project_scenarios
            .into_iter()
            .map(|(workflow_slug, slice_slug, scenario_kind, scenario)| {
                format!(
                    "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(scenario_kind),
                    json_string(scenario)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_scenario_definition_list(
    scenario_definitions: &[ProjectScenarioDefinition],
) -> String {
    let mut scenario_definitions = scenario_definitions.to_vec();
    scenario_definitions.sort();
    format!(
        "[{}]",
        scenario_definitions
            .into_iter()
            .map(|scenario| {
                format!(
                    "({}, {}, {}, {}, {}, {}, {}, [{}], [{}], {}, {}, [{}])",
                    json_string(scenario.workflow_slug()),
                    json_string(scenario.slice_slug()),
                    json_string(scenario.scenario_kind()),
                    json_string(scenario.scenario()),
                    json_string(scenario.given()),
                    json_string(scenario.when()),
                    json_string(scenario.then()),
                    json_string_list(scenario.read_streams()),
                    json_string_list(scenario.written_streams()),
                    json_string(scenario.contract_kind()),
                    json_string(scenario.covered_definition()),
                    json_string_list(scenario.error_references())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_scenario_definition_list(
    scenario_definitions: &[ProjectScenarioDefinition],
) -> String {
    let mut scenario_definitions = scenario_definitions.to_vec();
    scenario_definitions.sort();
    format!(
        "[{}]",
        scenario_definitions
            .into_iter()
            .map(|scenario| {
                format!(
                    "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {}, given: {}, when: {}, then: {}, readStreams: [{}], writtenStreams: [{}], contractKind: {}, coveredDefinition: {}, errorReferences: [{}] }}",
                    json_string(scenario.workflow_slug()),
                    json_string(scenario.slice_slug()),
                    json_string(scenario.scenario_kind()),
                    json_string(scenario.scenario()),
                    json_string(scenario.given()),
                    json_string(scenario.when()),
                    json_string(scenario.then()),
                    json_string_list(scenario.read_streams()),
                    json_string_list(scenario.written_streams()),
                    json_string(scenario.contract_kind()),
                    json_string(scenario.covered_definition()),
                    json_string_list(scenario.error_references())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_data_flow_list(project_data_flows: &[ProjectDataFlow]) -> String {
    let mut project_data_flows = project_data_flows.iter().collect::<Vec<_>>();
    project_data_flows.sort_unstable();
    format!(
        "[{}]",
        project_data_flows
            .into_iter()
            .map(|data_flow| {
                format!(
                    "({}, {}, {}, {}, {}, {}, {})",
                    json_string(data_flow.workflow_slug()),
                    json_string(data_flow.slice_slug()),
                    json_string(data_flow.datum()),
                    json_string(data_flow.source()),
                    json_string(data_flow.transformation()),
                    json_string(data_flow.target()),
                    json_string(data_flow.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_data_flow_list(project_data_flows: &[ProjectDataFlow]) -> String {
    let mut project_data_flows = project_data_flows.iter().collect::<Vec<_>>();
    project_data_flows.sort_unstable();
    format!(
        "[{}]",
        project_data_flows
            .into_iter()
            .map(|data_flow| {
                format!(
                    "{{ workflow: {}, slice: {}, datum: {}, source: {}, transformation: {}, target: {}, bitEncoding: {} }}",
                    json_string(data_flow.workflow_slug()),
                    json_string(data_flow.slice_slug()),
                    json_string(data_flow.datum()),
                    json_string(data_flow.source()),
                    json_string(data_flow.transformation()),
                    json_string(data_flow.target()),
                    json_string(data_flow.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_outcome_list(project_outcomes: &[ProjectOutcome]) -> String {
    let mut project_outcomes = project_outcomes.iter().collect::<Vec<_>>();
    project_outcomes.sort_unstable();
    format!(
        "[{}]",
        project_outcomes
            .into_iter()
            .map(|outcome| {
                format!(
                    "({}, {}, {}, [{}], {})",
                    json_string(outcome.workflow_slug()),
                    json_string(outcome.slice_slug()),
                    json_string(outcome.outcome()),
                    outcome
                        .events()
                        .iter()
                        .map(|event| json_string(event))
                        .collect::<Vec<_>>()
                        .join(","),
                    outcome.externally_relevant()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_outcome_list(project_outcomes: &[ProjectOutcome]) -> String {
    let mut project_outcomes = project_outcomes.iter().collect::<Vec<_>>();
    project_outcomes.sort_unstable();
    format!(
        "[{}]",
        project_outcomes
            .into_iter()
            .map(|outcome| {
                format!(
                    "{{ workflow: {}, slice: {}, outcome: {}, events: [{}], externallyRelevant: {} }}",
                    json_string(outcome.workflow_slug()),
                    json_string(outcome.slice_slug()),
                    json_string(outcome.outcome()),
                    outcome
                        .events()
                        .iter()
                        .map(|event| json_string(event))
                        .collect::<Vec<_>>()
                        .join(","),
                    outcome.externally_relevant()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_command_error_list(project_command_errors: &[ProjectCommandError]) -> String {
    let mut project_command_errors = project_command_errors.iter().collect::<Vec<_>>();
    project_command_errors.sort_unstable();
    format!(
        "[{}]",
        project_command_errors
            .into_iter()
            .map(|command_error| {
                format!(
                    "({}, {}, {}, {}, {}, {})",
                    json_string(command_error.workflow_slug()),
                    json_string(command_error.slice_slug()),
                    json_string(command_error.command()),
                    json_string(command_error.error()),
                    json_string(command_error.scenario()),
                    json_string(command_error.recovery())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_command_error_list(project_command_errors: &[ProjectCommandError]) -> String {
    let mut project_command_errors = project_command_errors.iter().collect::<Vec<_>>();
    project_command_errors.sort_unstable();
    format!(
        "[{}]",
        project_command_errors
            .into_iter()
            .map(|command_error| {
                format!(
                    "{{ workflow: {}, slice: {}, command: {}, error: {}, scenario: {}, recovery: {} }}",
                    json_string(command_error.workflow_slug()),
                    json_string(command_error.slice_slug()),
                    json_string(command_error.command()),
                    json_string(command_error.error()),
                    json_string(command_error.scenario()),
                    json_string(command_error.recovery())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
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
                    "({}, {}, {})",
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

fn lean_model_command_list(project_commands: &[ProjectCommand]) -> String {
    let mut project_commands = project_commands
        .iter()
        .map(|command| {
            (
                command.workflow_slug(),
                command.slice_slug(),
                command.command(),
            )
        })
        .collect::<Vec<_>>();
    project_commands.sort_unstable();
    format!(
        "[{}]",
        project_commands
            .into_iter()
            .map(|(workflow_slug, slice_slug, command)| {
                format!(
                    "({}, {}, {})",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(command)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_command_list(project_commands: &[ProjectCommand]) -> String {
    let mut project_commands = project_commands
        .iter()
        .map(|command| {
            (
                command.workflow_slug(),
                command.slice_slug(),
                command.command(),
            )
        })
        .collect::<Vec<_>>();
    project_commands.sort_unstable();
    format!(
        "[{}]",
        project_commands
            .into_iter()
            .map(|(workflow_slug, slice_slug, command)| {
                format!(
                    "{{ workflow: {}, slice: {}, command: {} }}",
                    json_string(workflow_slug),
                    json_string(slice_slug),
                    json_string(command)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_model_command_input_list(project_command_inputs: &[ProjectCommandInput]) -> String {
    let mut project_command_inputs = project_command_inputs.iter().collect::<Vec<_>>();
    project_command_inputs.sort_unstable();
    format!(
        "[{}]",
        project_command_inputs
            .into_iter()
            .map(|command_input| {
                format!(
                    "({}, {}, {}, {}, {}, {}, [{}], {}, {})",
                    json_string(command_input.workflow_slug()),
                    json_string(command_input.slice_slug()),
                    json_string(command_input.command()),
                    json_string(command_input.input()),
                    json_string(command_input.source_kind()),
                    json_string(command_input.source_description()),
                    json_string_list(command_input.provenance_chain()),
                    json_string(command_input.event_stream_source_event()),
                    json_string(command_input.event_stream_source_attribute())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_model_command_input_list(project_command_inputs: &[ProjectCommandInput]) -> String {
    let mut project_command_inputs = project_command_inputs.iter().collect::<Vec<_>>();
    project_command_inputs.sort_unstable();
    format!(
        "[{}]",
        project_command_inputs
            .into_iter()
            .map(|command_input| {
                format!(
                    "{{ workflow: {}, slice: {}, command: {}, input: {}, sourceKind: {}, sourceDescription: {}, provenanceChain: [{}], eventStreamSourceEvent: {}, eventStreamSourceAttribute: {} }}",
                    json_string(command_input.workflow_slug()),
                    json_string(command_input.slice_slug()),
                    json_string(command_input.command()),
                    json_string(command_input.input()),
                    json_string(command_input.source_kind()),
                    json_string(command_input.source_description()),
                    json_string_list(command_input.provenance_chain()),
                    json_string(command_input.event_stream_source_event()),
                    json_string(command_input.event_stream_source_attribute())
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
                    "({}, {}, {})",
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
                        "({}, {}, {}, {}, [{}], {}, {})",
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
                        "({}, {}, {}, {}, {}, {}, {}, {}, [{}], {}, {}, {}, {})",
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
                    "({}, {}, {})",
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
                    "({}, {}, {}, [{}], [{}], [{}], [{}])",
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
                    "({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, [{}], {}, {}, {}, {}, {}, {}, {})",
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
                    "({}, {}, {}, {}, {}, {}, {})",
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
                    "({}, {}, {}, {}, {}, {})",
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
                        "({}, {}, {}, {}, {}, {}, {}, {}, {})",
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
                    "({}, {}, {})",
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
                        "({}, {}, {}, {}, {}, {})",
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
                    "({}, {}, {})",
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
                    "({}, {}, {}, {}, {}, {})",
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
                    "({}, {}, {}, {})",
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
                    provenance,
                )| {
                    format!(
                        "({}, {}, {}, {}, {}, {}, {}, {})",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(event),
                        json_string(attribute),
                        json_string(source_kind),
                        json_string(source_name),
                        json_string(source_field),
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
                    provenance,
                )| {
                    format!(
                        "{{ workflow: {}, slice: {}, event: {}, attribute: {}, sourceKind: {}, sourceName: {}, sourceField: {}, provenance: {} }}",
                        json_string(workflow_slug),
                        json_string(slice_slug),
                        json_string(event),
                        json_string(attribute),
                        json_string(source_kind),
                        json_string(source_name),
                        json_string(source_field),
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
    format!(
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
    )
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
                "{}/{}/{}@{}~{}~{}#{}",
                data_flow.workflow_slug(),
                data_flow.slice_slug(),
                data_flow.datum(),
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
                "{}/{}/{}/{}@{}#{}#{}#{}#{}",
                command_input.workflow_slug(),
                command_input.slice_slug(),
                command_input.command(),
                command_input.input(),
                command_input.source_kind(),
                command_input.source_description(),
                command_input.provenance_chain().join(" -> "),
                command_input.event_stream_source_event(),
                command_input.event_stream_source_attribute()
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
                provenance,
            )| {
                format!(
                    "{workflow_slug}/{slice_slug}/{event}/{attribute}@{source_kind}#{source_name}.{source_field}#{provenance}"
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

fn artifact_marker(value: impl Into<String>) -> ArtifactMarker {
    ArtifactMarker::try_new(value.into()).unwrap_or_else(|error| {
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
