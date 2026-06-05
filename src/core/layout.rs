use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
use crate::core::effect::{
    ArtifactFileExtension, ArtifactMarker, Effect, EffectPlan, FileContents, ProjectPath,
    ReportLine,
};
use crate::core::emit::lean::emit_slice_module as emit_lean_slice_module;
use crate::core::emit::quint::emit_slice_module as emit_quint_slice_module;
use crate::core::formal_graph::{FormalWorkflowGraph, FormalWorkflowGraphs};
use crate::core::formal_project_facts::{ProjectCommand, ProjectEvent, ProjectStream};
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

pub fn check_project(
    project_name: ProjectName,
    formal_workflows: FormalWorkflowGraphs,
    project_commands: Vec<ProjectCommand>,
    project_streams: Vec<ProjectStream>,
    project_events: Vec<ProjectEvent>,
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
        &project_commands,
        &project_streams,
        &project_events,
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

fn project_root_effects(
    project_name: &ProjectName,
    module_name: &str,
    modeled_workflows: &[ModeledWorkflowLayout],
    formal_workflows: &[FormalWorkflowGraph],
    project_commands: &[ProjectCommand],
    project_streams: &[ProjectStream],
    project_events: &[ProjectEvent],
) -> Vec<Effect> {
    let project_name_text = project_name.as_ref();
    let model_version = "0.1.0";
    let workflow_slug_list = workflow_slug_list(modeled_workflows);
    let workflow_count = modeled_workflows.len();
    let lean_model_slice_list = lean_model_slice_list(formal_workflows);
    let lean_model_slice_module_list = lean_model_slice_module_list(formal_workflows);
    let lean_model_command_list = lean_model_command_list(project_commands);
    let lean_model_stream_list = lean_model_stream_list(project_streams);
    let lean_model_event_list = lean_model_event_list(project_events);
    let quint_model_slice_list = quint_model_slice_list(formal_workflows);
    let quint_model_slice_module_list = quint_model_slice_module_list(formal_workflows);
    let quint_model_command_list = quint_model_command_list(project_commands);
    let quint_model_stream_list = quint_model_stream_list(project_streams);
    let quint_model_event_list = quint_model_event_list(project_events);
    let model_digest = model_digest(
        project_name,
        modeled_workflows,
        formal_workflows,
        project_commands,
        project_streams,
        project_events,
    );
    let slice_count = formal_workflows
        .iter()
        .map(|workflow| workflow.slice_details().as_slice().len())
        .sum::<usize>();
    let stream_count = project_streams.len();
    let command_count = project_commands.len();
    let event_count = project_events.len();
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
            artifact_marker("def modelCommands :"),
            artifact_marker(format!(
                "def modelCommands : List (String × String × String) := {lean_model_command_list}"
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
            artifact_marker("theorem modelCommandsAreDeclared"),
            artifact_marker(format!(
                "theorem modelCommandsAreDeclared : modelCommands.length = {command_count} := rfl"
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
            artifact_marker("  type ModelCommand ="),
            artifact_marker("  type ModelCommand = { workflow: str, slice: str, command: str }"),
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
            artifact_marker("  val modelCommands:"),
            artifact_marker(format!(
                "  val modelCommands: List[ModelCommand] = {quint_model_command_list}"
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
            artifact_marker("  val modelCommandsAreDeclared ="),
            artifact_marker(format!(
                "  val modelCommandsAreDeclared = modelCommands.length() == {command_count}"
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

fn model_digest(
    project_name: &ProjectName,
    modeled_workflows: &[ModeledWorkflowLayout],
    formal_workflows: &[FormalWorkflowGraph],
    project_commands: &[ProjectCommand],
    project_streams: &[ProjectStream],
    project_events: &[ProjectEvent],
) -> String {
    format!(
        "project:name={};version=0.1.0;workflows={};slices={};commands={};streams={};events={}",
        project_name.as_ref(),
        digest_workflows(modeled_workflows),
        digest_slices(formal_workflows),
        digest_commands(project_commands),
        digest_streams(project_streams),
        digest_events(project_events)
    )
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
