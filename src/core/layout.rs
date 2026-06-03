use crate::core::digest::artifact_digest;
use crate::core::effect::{
    ArtifactDigest, Effect, EffectPlan, FileContents, ProjectPath, ReportLine,
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
    let digest = artifact_digest(workflow.name.clone());
    let browser_name_marker =
        artifact_digest_marker(format!("\"name\": {}", json_string(workflow.name.as_ref())));
    let browser_description_marker = artifact_digest_marker(format!(
        "\"description\": {}",
        json_string(workflow.description.as_ref())
    ));
    let lean_slug_marker = artifact_digest_marker(format!(
        "def workflowSlug := {}",
        json_string(workflow.slug.as_ref())
    ));
    let lean_description_marker = artifact_digest_marker(format!(
        "def workflowDescription := {}",
        json_string(workflow.description.as_ref())
    ));
    let quint_slug_marker = artifact_digest_marker(format!(
        "val workflowSlug = {}",
        json_string(workflow.slug.as_ref())
    ));
    let quint_description_marker = artifact_digest_marker(format!(
        "val workflowDescription = {}",
        json_string(workflow.description.as_ref())
    ));
    let lean_slice_marker = artifact_digest_marker("def workflowSlices : List String :=");
    let lean_transition_marker = artifact_digest_marker("def workflowTransitions : List String :=");
    let quint_slice_marker = artifact_digest_marker("val workflowSlices =");
    let quint_transition_marker = artifact_digest_marker("val workflowTransitions =");
    let module_name = module_name_from_model(workflow.name.clone());
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
        Effect::RequireDigest(
            workflow_path.clone(),
            browser_name_marker,
            report_line(format!(
                "browser workflow drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            workflow_path.clone(),
            browser_description_marker,
            report_line(format!(
                "browser workflow drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireFile(lean_path.clone()),
        Effect::RequireFile(quint_path.clone()),
        Effect::RequireDigest(
            lean_path.clone(),
            digest.clone(),
            report_line(format!(
                "artifact digest mismatch for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            quint_path.clone(),
            digest,
            report_line(format!(
                "artifact digest mismatch for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            lean_path.clone(),
            lean_slug_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            lean_path.clone(),
            lean_description_marker,
            report_line(format!(
                "Lean workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            quint_path.clone(),
            quint_slug_marker,
            report_line(format!(
                "Quint workflow field drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            quint_path.clone(),
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
        Effect::RequireWorkflowSlices(
            workflow_path.clone(),
            quint_path.clone(),
            quint_slice_marker,
            report_line(format!(
                "Quint workflow slice drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireWorkflowTransitions(
            workflow_path.clone(),
            lean_path,
            lean_transition_marker,
            report_line(format!(
                "Lean workflow transition drift for workflow {workflow_name}"
            )),
        ),
        Effect::RequireWorkflowTransitions(
            workflow_path,
            quint_path,
            quint_transition_marker,
            report_line(format!(
                "Quint workflow transition drift for workflow {workflow_name}"
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

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated JSON string must be valid: {error}");
    })
}
