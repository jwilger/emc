use crate::core::emc::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, ProjectPath, ReportLine};
use crate::core::project::ProjectName;
use crate::core::types::{ModelName, WorkflowSlug};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ImportedWorkflowLayout {
    name: ModelName,
    slug: WorkflowSlug,
}

impl ImportedWorkflowLayout {
    pub fn new(name: ModelName, slug: WorkflowSlug) -> Self {
        Self { name, slug }
    }
}

pub fn check_project(
    project_name: ProjectName,
    imported_workflows: Vec<ImportedWorkflowLayout>,
) -> EffectPlan {
    let module_name = module_name(&project_name);
    let imported_effects = imported_workflows
        .into_iter()
        .flat_map(imported_workflow_effects)
        .collect::<Vec<_>>();

    EffectPlan::new(
        [
            vec![
                Effect::RequireFile(project_path("emc.toml")),
                Effect::RequireFile(project_path(format!("model/lean/{module_name}.lean"))),
                Effect::RequireFile(project_path(format!("model/quint/{module_name}.qnt"))),
                Effect::RequireFile(project_path("model/browser/data/index.json")),
                Effect::RequireFile(project_path("model/browser/data/workflows/.gitkeep")),
                Effect::RequireFile(project_path("model/browser/data/slices/.gitkeep")),
                Effect::RequireFile(project_path("reviews/.gitkeep")),
            ],
            imported_effects,
            vec![Effect::Report(report_line("project layout is complete"))],
        ]
        .concat(),
    )
}

pub fn list_workflows(imported_workflows: Vec<ImportedWorkflowLayout>) -> EffectPlan {
    EffectPlan::new(
        imported_workflows
            .into_iter()
            .map(|workflow| Effect::Report(report_line(workflow.name.as_ref().to_owned())))
            .collect(),
    )
}

fn imported_workflow_effects(workflow: ImportedWorkflowLayout) -> Vec<Effect> {
    let workflow_name = workflow.name.as_ref().to_owned();
    let digest = artifact_digest(workflow.name.clone());
    let module_name = module_name_from_model(workflow.name);
    let workflow_slug = workflow.slug.as_ref();

    vec![
        Effect::RequireFile(project_path(format!(
            "model/browser/data/workflows/{workflow_slug}.eventmodel.json"
        ))),
        Effect::RequireFile(project_path(format!("model/lean/{module_name}.lean"))),
        Effect::RequireFile(project_path(format!("model/quint/{module_name}.qnt"))),
        Effect::RequireDigest(
            project_path(format!("model/lean/{module_name}.lean")),
            digest.clone(),
            report_line(format!(
                "artifact digest mismatch for workflow {workflow_name}"
            )),
        ),
        Effect::RequireDigest(
            project_path(format!("model/quint/{module_name}.qnt")),
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
