use crate::core::digest::{artifact_digest, slice_artifact_digest};
use crate::core::effect::{
    ArtifactFileExtension, ArtifactMarker, Effect, EffectPlan, FileContents, ProjectPath,
    ReportLine,
};
use crate::core::emit::lean::emit_slice_module as emit_lean_slice_module;
use crate::core::emit::quint::emit_slice_module as emit_quint_slice_module;
use crate::core::formal_graph::{FormalWorkflowGraph, FormalWorkflowGraphs};
use crate::core::formal_projection::{
    project_browser_index_document, project_slice_browser_document,
    project_workflow_browser_document,
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

    pub fn browser_data_path(&self) -> ProjectPath {
        project_path(format!(
            "data/workflows/{}.eventmodel.json",
            self.slug.as_ref()
        ))
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
) -> EffectPlan {
    let module_name = module_name(&project_name);
    let mut formal_workflows = formal_workflows.into_inner();
    formal_workflows.sort_by(|left, right| left.slug().as_ref().cmp(right.slug().as_ref()));
    let modeled_workflows = formal_workflows
        .iter()
        .map(modeled_workflow_layout)
        .collect::<Vec<_>>();
    let root_effects = project_root_effects(&project_name, &module_name);
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
    let browser_workflow_paths = formal_workflows
        .iter()
        .map(|workflow| {
            project_path(format!(
                "model/browser/data/workflows/{}.eventmodel.json",
                workflow.slug().as_ref()
            ))
        })
        .collect::<Vec<_>>();
    let browser_slice_paths = formal_workflows
        .iter()
        .flat_map(|workflow| {
            workflow.slice_details().as_slice().iter().map(|slice| {
                project_path(format!(
                    "model/browser/data/slices/{}.eventmodel.json",
                    slice.slug().as_ref()
                ))
            })
        })
        .collect::<Vec<_>>();
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
                Effect::RequireFile(project_path("model/browser/data/index.json")),
                Effect::RequireJsonObjectKeysUnique(
                    project_path("model/browser/data/index.json"),
                    report_line("browser index drift"),
                ),
                Effect::RequireIndexedWorkflowFiles(
                    project_path("model/browser/data/index.json"),
                    project_path("model/browser/data/workflows"),
                    report_line("browser workflow index drift"),
                ),
                Effect::RequireFileContents(
                    project_path("model/browser/data/index.json"),
                    project_browser_index_document(&formal_workflows),
                    report_line("browser index projection drift"),
                ),
                Effect::RequireReferencedSliceFiles(
                    project_path("model/browser/data/workflows"),
                    project_path("model/browser/data/slices"),
                    report_line("browser slice reference drift"),
                ),
                Effect::RequireReferencedSliceFileIdentities(
                    project_path("model/browser/data/workflows"),
                    report_line("browser slice identity drift"),
                ),
                Effect::RequireOnlyModeledArtifacts(
                    project_path("model/browser/data/workflows"),
                    artifact_file_extension(".eventmodel.json"),
                    browser_workflow_paths,
                    report_line("browser workflow projection drift"),
                ),
                Effect::RequireOnlyModeledArtifacts(
                    project_path("model/browser/data/slices"),
                    artifact_file_extension(".eventmodel.json"),
                    browser_slice_paths,
                    report_line("browser slice projection drift"),
                ),
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
                Effect::RequireFile(project_path("model/browser/data/workflows/.gitkeep")),
                Effect::RequireFile(project_path("model/browser/data/slices/.gitkeep")),
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

fn project_root_effects(project_name: &ProjectName, module_name: &str) -> Vec<Effect> {
    let project_name_text = project_name.as_ref();
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
            quint_config_path,
            artifact_marker("    \"workflowTransitionsStructured\""),
            artifact_marker("    \"workflowTransitionsStructured\""),
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
    let browser_name_marker = artifact_marker(format!(
        "  \"name\": {},",
        json_string(workflow.name().as_ref())
    ));
    let browser_name_prefix = artifact_marker("  \"name\":");
    let browser_description_marker = artifact_marker(format!(
        "  \"description\": {},",
        json_string(workflow.description().as_ref())
    ));
    let browser_description_prefix = artifact_marker("  \"description\":");
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
    let lean_transition_marker = lean_workflow_transition_marker(workflow);
    let quint_slice_marker = quint_workflow_slice_marker(workflow);
    let quint_slice_detail_marker = quint_workflow_slice_detail_marker(workflow);
    let quint_transition_marker = quint_workflow_transition_marker(workflow);
    let lean_slice_prefix = artifact_marker("def workflowSlices : List String :=");
    let lean_slice_detail_prefix =
        artifact_marker("def workflowSliceDetails : List (String × String × String × String) :=");
    let lean_transition_prefix =
        artifact_marker("def workflowTransitions : List WorkflowTransition :=");
    let quint_slice_prefix = artifact_marker("val workflowSlices =");
    let quint_slice_detail_prefix = artifact_marker("val workflowSliceDetails =");
    let quint_transition_prefix = artifact_marker("val workflowTransitions =");
    let lean_identity_invariant_marker = artifact_marker(format!(
        "theorem workflowIdentityIsStable : workflowName = {} := rfl",
        json_string(workflow.name().as_ref())
    ));
    let lean_identity_invariant_prefix = artifact_marker("theorem workflowIdentityIsStable :");
    let lean_slice_detail_invariant_marker = artifact_marker(
        "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl",
    );
    let lean_slice_detail_invariant_prefix = artifact_marker("theorem workflowSlicesHaveDetails :");
    let lean_transition_invariant_marker = artifact_marker(
        "theorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.source.isEmpty == false && transition.target.isEmpty == false && transition.kind.isEmpty == false && transition.trigger.isEmpty == false) = true := rfl",
    );
    let lean_transition_invariant_prefix =
        artifact_marker("theorem workflowTransitionsAreStructured :");
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
    let quint_transition_invariant_marker = artifact_marker(
        "val workflowTransitionsStructured = workflowTransitions.select(transition => transition.source != \"\" and transition.target != \"\" and transition.kind != \"\" and transition.trigger != \"\").length() == workflowTransitions.length()",
    );
    let quint_transition_invariant_prefix = artifact_marker("val workflowTransitionsStructured =");
    let module_name = module_name_from_model(workflow.name().clone());
    let lean_module_marker = artifact_marker(format!("namespace {module_name}"));
    let lean_module_prefix = artifact_marker("namespace ");
    let lean_module_end_marker = artifact_marker(format!("end {module_name}"));
    let lean_module_end_prefix = artifact_marker("end ");
    let quint_module_marker = artifact_marker(format!("module {module_name} {{"));
    let quint_module_prefix = artifact_marker("module ");
    let quint_module_close_marker = artifact_marker("}");
    let workflow_slug = workflow.slug().as_ref();
    let workflow_path = project_path(format!(
        "model/browser/data/workflows/{workflow_slug}.eventmodel.json"
    ));
    let lean_path = project_path(format!("model/lean/{module_name}.lean"));
    let quint_path = project_path(format!("model/quint/{module_name}.qnt"));
    let digest = artifact_digest(
        workflow.name().clone(),
        workflow.slug().clone(),
        workflow.description().clone(),
        workflow.slice_details().clone(),
        workflow.transitions().clone(),
    );

    let workflow_effects = [
        vec![
            Effect::RequireFile(workflow_path.clone()),
            Effect::RequireWorkflowProjection(
                workflow_path.clone(),
                project_workflow_browser_document(workflow),
                report_line(format!(
                    "browser workflow drift for workflow {workflow_name}"
                )),
            ),
            Effect::RequireWorkflowSliceFiles(
                workflow_path.clone(),
                report_line(format!(
                    "workflow {workflow_name} references missing slice artifact"
                )),
            ),
            Effect::RequireWorkflowSliceJsonObjects(
                workflow_path.clone(),
                report_line(format!("browser slice drift for workflow {workflow_name}")),
            ),
            Effect::RequireWorkflowSliceJsonObjectKeysUnique(
                workflow_path.clone(),
                report_line(format!("browser slice drift for workflow {workflow_name}")),
            ),
            Effect::RequireJsonObjectKeysUnique(
                workflow_path.clone(),
                report_line(format!(
                    "browser workflow drift for workflow {workflow_name}"
                )),
            ),
            Effect::RequireCanonicalDeclaration(
                workflow_path.clone(),
                browser_name_prefix,
                browser_name_marker,
                report_line(format!(
                    "browser workflow drift for workflow {workflow_name}"
                )),
            ),
            Effect::RequireCanonicalDeclaration(
                workflow_path.clone(),
                browser_description_prefix,
                browser_description_marker,
                report_line(format!(
                    "browser workflow drift for workflow {workflow_name}"
                )),
            ),
        ],
        Vec::new(),
        vec![
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
                lean_path.clone(),
                lean_transition_prefix,
                lean_transition_marker,
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
        ],
    ]
    .concat();

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
            let browser_slice_path = project_path(format!(
                "model/browser/data/slices/{}.eventmodel.json",
                slice.slug().as_ref()
            ));
            let lean_slice_path = project_path(format!("model/lean/slices/{module_name}.lean"));
            let quint_slice_path = project_path(format!("model/quint/slices/{module_name}.qnt"));

            [
                Effect::RequireFileContents(
                    browser_slice_path,
                    project_slice_browser_document(workflow, slice),
                    report_line(format!(
                        "browser slice projection drift for slice {}",
                        slice.name().as_ref()
                    )),
                ),
                Effect::RequireFileContents(
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
                Effect::RequireFileContents(
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

fn lean_workflow_transition_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
        "def workflowTransitions : List WorkflowTransition := [{}]",
        workflow
            .transitions()
            .as_slice()
            .iter()
            .map(|transition| {
                format!(
                    "{{ source := {}, target := {}, kind := {}, trigger := {}, rationale := {} }}",
                    json_string(transition.source().as_ref()),
                    json_string(transition.target().as_ref()),
                    json_string(transition.kind().as_ref()),
                    json_string(transition.trigger().as_ref()),
                    json_string(
                        transition
                            .rationale()
                            .map_or("", |rationale| rationale.as_ref())
                    )
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn quint_workflow_slice_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
        "val workflowSlices = [{}]",
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
        "val workflowSliceDetails = [{}]",
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

fn quint_workflow_transition_marker(workflow: &FormalWorkflowGraph) -> ArtifactMarker {
    artifact_marker(format!(
        "val workflowTransitions = [{}]",
        workflow
            .transitions()
            .as_slice()
            .iter()
            .map(|transition| {
                format!(
                    "{{ source: {}, target: {}, kind: {}, trigger: {}, rationale: {} }}",
                    json_string(transition.source().as_ref()),
                    json_string(transition.target().as_ref()),
                    json_string(transition.kind().as_ref()),
                    json_string(transition.trigger().as_ref()),
                    json_string(
                        transition
                            .rationale()
                            .map_or("", |rationale| rationale.as_ref())
                    )
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
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
