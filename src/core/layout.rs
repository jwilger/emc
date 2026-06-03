use crate::core::digest::artifact_digest;
use crate::core::effect::{
    ArtifactDigest, ArtifactFileExtension, ArtifactMarker, Effect, EffectPlan, FileContents,
    ProjectPath, ReportLine,
};
use crate::core::project::ProjectName;
use crate::core::types::{ModelDescription, ModelName, WorkflowSlug};

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

pub fn check_project(
    project_name: ProjectName,
    modeled_workflows: Vec<ModeledWorkflowLayout>,
) -> EffectPlan {
    let module_name = module_name(&project_name);
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
    let modeled_effects = modeled_workflows
        .into_iter()
        .flat_map(modeled_workflow_effects)
        .collect::<Vec<_>>();

    EffectPlan::new(
        [
            vec![
                Effect::RequireFile(project_path("emc.toml")),
                Effect::RequireFile(project_path("model/lean/lakefile.lean")),
                Effect::RequireFile(project_path("model/lean/lean-toolchain")),
                Effect::RequireFile(project_path(format!("model/lean/{module_name}.lean"))),
                Effect::RequireFile(project_path("model/quint/quint.json")),
                Effect::RequireFile(project_path(format!("model/quint/{module_name}.qnt"))),
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
                Effect::RequireReferencedSliceFiles(
                    project_path("model/browser/data/workflows"),
                    project_path("model/browser/data/slices"),
                    report_line("browser slice reference drift"),
                ),
                Effect::RequireReferencedSliceFileIdentities(
                    project_path("model/browser/data/workflows"),
                    report_line("browser slice identity drift"),
                ),
                Effect::RequireFile(project_path("model/browser/data/workflows/.gitkeep")),
                Effect::RequireFile(project_path("model/browser/data/slices/.gitkeep")),
                Effect::RequireFile(project_path("reviews/.gitkeep")),
            ],
            modeled_effects,
            vec![Effect::Report(report_line("project layout is complete"))],
        ]
        .concat(),
    )
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

pub fn list_workflows(modeled_workflows: Vec<ModeledWorkflowLayout>) -> EffectPlan {
    EffectPlan::new(
        modeled_workflows
            .into_iter()
            .map(|workflow| Effect::Report(report_line(workflow.name.as_ref().to_owned())))
            .collect(),
    )
}

pub fn show_workflow(workflow_document: FileContents) -> EffectPlan {
    EffectPlan::new(vec![Effect::ReportDocument(workflow_document)])
}

fn modeled_workflow_effects(workflow: ModeledWorkflowLayout) -> Vec<Effect> {
    let workflow_name = workflow.name.as_ref().to_owned();
    let digest = artifact_digest(
        workflow.name.clone(),
        workflow.slug.clone(),
        workflow.description.clone(),
    );
    let browser_name_marker = artifact_marker(format!(
        "  \"name\": {},",
        json_string(workflow.name.as_ref())
    ));
    let browser_name_prefix = artifact_marker("  \"name\":");
    let browser_description_marker = artifact_marker(format!(
        "  \"description\": {},",
        json_string(workflow.description.as_ref())
    ));
    let browser_description_prefix = artifact_marker("  \"description\":");
    let lean_name_marker = artifact_marker(format!(
        "def workflowName := {}",
        json_string(workflow.name.as_ref())
    ));
    let lean_name_prefix = artifact_marker("def workflowName :=");
    let lean_slug_marker = artifact_marker(format!(
        "def workflowSlug := {}",
        json_string(workflow.slug.as_ref())
    ));
    let lean_slug_prefix = artifact_marker("def workflowSlug :=");
    let lean_description_marker = artifact_marker(format!(
        "def workflowDescription := {}",
        json_string(workflow.description.as_ref())
    ));
    let lean_description_prefix = artifact_marker("def workflowDescription :=");
    let quint_name_marker = artifact_marker(format!(
        "val workflowName = {}",
        json_string(workflow.name.as_ref())
    ));
    let quint_name_prefix = artifact_marker("val workflowName =");
    let quint_slug_marker = artifact_marker(format!(
        "val workflowSlug = {}",
        json_string(workflow.slug.as_ref())
    ));
    let quint_slug_prefix = artifact_marker("val workflowSlug =");
    let quint_description_marker = artifact_marker(format!(
        "val workflowDescription = {}",
        json_string(workflow.description.as_ref())
    ));
    let quint_description_prefix = artifact_marker("val workflowDescription =");
    let lean_slice_marker = artifact_digest_marker("def workflowSlices : List String :=");
    let lean_slice_detail_marker = artifact_digest_marker(
        "def workflowSliceDetails : List (String × String × String × String) :=",
    );
    let lean_transition_marker = artifact_digest_marker("def workflowTransitions : List String :=");
    let lean_identity_invariant_marker = artifact_marker(format!(
        "theorem workflowIdentityIsStable : workflowName = {} := rfl",
        json_string(workflow.name.as_ref())
    ));
    let lean_identity_invariant_prefix = artifact_marker("theorem workflowIdentityIsStable :");
    let lean_slice_detail_invariant_marker = artifact_marker(
        "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl",
    );
    let lean_slice_detail_invariant_prefix = artifact_marker("theorem workflowSlicesHaveDetails :");
    let quint_slice_marker = artifact_digest_marker("val workflowSlices =");
    let quint_slice_detail_marker = artifact_digest_marker("val workflowSliceDetails =");
    let quint_transition_marker = artifact_digest_marker("val workflowTransitions =");
    let quint_identity_invariant_marker = artifact_marker(format!(
        "val workflowIdentityStable = workflowName == {}",
        json_string(workflow.name.as_ref())
    ));
    let quint_identity_invariant_prefix = artifact_marker("val workflowIdentityStable =");
    let quint_slice_detail_invariant_marker = artifact_marker(
        "val workflowSlicesHaveDetails = length(workflowSlices) == length(workflowSliceDetails)",
    );
    let quint_slice_detail_invariant_prefix = artifact_marker("val workflowSlicesHaveDetails =");
    let quint_slice_detail_complete_marker =
        artifact_marker("val workflowSliceDetailsComplete = workflowSlicesHaveDetails");
    let quint_slice_detail_complete_prefix = artifact_marker("val workflowSliceDetailsComplete =");
    let module_name = module_name_from_model(workflow.name.clone());
    let lean_module_marker = artifact_marker(format!("namespace {module_name}"));
    let lean_module_prefix = artifact_marker("namespace ");
    let lean_module_end_marker = artifact_marker(format!("end {module_name}"));
    let lean_module_end_prefix = artifact_marker("end ");
    let quint_module_marker = artifact_marker(format!("module {module_name} {{"));
    let quint_module_prefix = artifact_marker("module ");
    let workflow_slug = workflow.slug.as_ref();
    let workflow_path = project_path(format!(
        "model/browser/data/workflows/{workflow_slug}.eventmodel.json"
    ));
    let lean_path = project_path(format!("model/lean/{module_name}.lean"));
    let quint_path = project_path(format!("model/quint/{module_name}.qnt"));

    vec![
        Effect::RequireFile(workflow_path.clone()),
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
        Effect::RequireWorkflowSlices(
            workflow_path.clone(),
            lean_path.clone(),
            lean_slice_marker,
            report_line(format!(
                "Lean workflow slice drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireWorkflowSliceDetails(
            workflow_path.clone(),
            lean_path.clone(),
            lean_slice_detail_marker,
            report_line(format!(
                "Lean workflow slice detail drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireWorkflowSlices(
            workflow_path.clone(),
            quint_path.clone(),
            quint_slice_marker,
            report_line(format!(
                "Quint workflow slice drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireWorkflowSliceDetails(
            workflow_path.clone(),
            quint_path.clone(),
            quint_slice_detail_marker,
            report_line(format!(
                "Quint workflow slice detail drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireWorkflowTransitions(
            workflow_path.clone(),
            lean_path.clone(),
            lean_transition_marker,
            report_line(format!(
                "Lean workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireWorkflowTransitions(
            workflow_path,
            quint_path.clone(),
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
    ]
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

fn artifact_digest_marker(value: impl Into<String>) -> ArtifactDigest {
    ArtifactDigest::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static artifact marker must be valid: {error}");
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
