use crate::core::emc::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::layout::ImportedWorkflowLayout;
use crate::core::types::{ModelDescription, ModelName, WorkflowSlug};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewWorkflow {
    name: ModelName,
    description: ModelDescription,
    slug: WorkflowSlug,
}

impl NewWorkflow {
    pub fn new(name: ModelName, description: ModelDescription, slug: WorkflowSlug) -> Self {
        Self {
            name,
            description,
            slug,
        }
    }
}

pub fn add_workflow(
    existing_workflows: Vec<ImportedWorkflowLayout>,
    workflow: NewWorkflow,
) -> EffectPlan {
    let workflow_name = workflow.name.as_ref();
    let workflow_description = workflow.description.as_ref();
    let workflow_slug = workflow.slug.as_ref();
    let module_name = module_name(workflow.name.as_ref());
    let digest = artifact_digest(workflow.name.clone());
    let workflow_layout = ImportedWorkflowLayout::new(
        workflow.name.clone(),
        workflow.description.clone(),
        workflow.slug.clone(),
    );
    let added_slug = workflow.slug.clone();
    let workflows = existing_workflows
        .into_iter()
        .filter(|existing| existing.slug() != &added_slug)
        .chain([workflow_layout])
        .collect::<Vec<_>>();

    EffectPlan::new(vec![
        Effect::WriteFile(
            project_path(format!(
                "model/browser/data/workflows/{workflow_slug}.eventmodel.json"
            )),
            file_contents(format!(
                "{{\n  \"name\": {},\n  \"version\": \"0.1.0\",\n  \"description\": {},\n  \"slice_files\": [],\n  \"steps\": []\n}}\n",
                json_string(workflow_name),
                json_string(workflow_description)
            )),
        ),
        Effect::WriteFile(
            project_path("model/browser/data/index.json"),
            file_contents(browser_index(workflows)),
        ),
        Effect::WriteFile(
            project_path(format!("model/lean/{module_name}.lean")),
            file_contents(format!(
                "namespace {module_name}\n\n-- EMC-DIGEST: {}\n-- EMC generated Lean4 business workflow model.\ndef workflowName := \"{workflow_name}\"\n\nend {module_name}\n",
                digest.as_ref()
            )),
        ),
        Effect::WriteFile(
            project_path(format!("model/quint/{module_name}.qnt")),
            file_contents(format!(
                "module {module_name} {{\n  // EMC-DIGEST: {}\n  const workflowName = \"{workflow_name}\"\n}}\n",
                digest.as_ref()
            )),
        ),
        Effect::Report(report_line(format!("added workflow {workflow_name}"))),
    ])
}

fn browser_index(mut workflows: Vec<ImportedWorkflowLayout>) -> String {
    workflows.sort_by(|left, right| left.slug().as_ref().cmp(right.slug().as_ref()));
    let entries = workflows
        .iter()
        .map(workflow_index_entry)
        .collect::<Vec<_>>()
        .join(",\n");
    if entries.is_empty() {
        "{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": []\n}\n".to_owned()
    } else {
        format!(
            "{{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": [\n{entries}\n  ]\n}}\n"
        )
    }
}

fn workflow_index_entry(workflow: &ImportedWorkflowLayout) -> String {
    format!(
        "    {{\n      \"name\": {},\n      \"path\": \"data/workflows/{}.eventmodel.json\",\n      \"description\": {}\n    }}",
        json_string(workflow.name().as_ref()),
        workflow.slug().as_ref(),
        json_string(workflow.description().as_ref())
    )
}

fn module_name(raw: &str) -> String {
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
        unreachable!("EMC generated project path must be valid: {error}");
    })
}

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated file contents must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated report line must be valid: {error}");
    })
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated JSON string must be valid: {error}");
    })
}
