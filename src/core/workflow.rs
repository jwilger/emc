use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::emc::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::layout::ImportedWorkflowLayout;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, WorkflowSlug,
};

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
    workflow_effect_plan(existing_workflows, workflow, MutationReport::Added)
}

pub fn update_workflow_description(
    existing_workflows: Vec<ImportedWorkflowLayout>,
    slug: WorkflowSlug,
    description: ModelDescription,
) -> Result<EffectPlan, WorkflowMutationError> {
    let workflow = existing_workflows
        .iter()
        .find(|existing| existing.slug() == &slug)
        .map(|existing| {
            NewWorkflow::new(existing.name().clone(), description.clone(), slug.clone())
        })
        .ok_or_else(|| WorkflowMutationError::new(format!("unknown workflow {}", slug.as_ref())))?;

    Ok(workflow_effect_plan(
        existing_workflows,
        workflow,
        MutationReport::Updated,
    ))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowMutationError {
    message: String,
}

impl WorkflowMutationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for WorkflowMutationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowMutationError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MutationReport {
    Added,
    Updated,
}

fn workflow_effect_plan(
    existing_workflows: Vec<ImportedWorkflowLayout>,
    workflow: NewWorkflow,
    mutation_report: MutationReport,
) -> EffectPlan {
    let workflow_name = workflow.name.as_ref();
    let workflow_description = workflow.description.as_ref();
    let workflow_slug = workflow.slug.as_ref();
    let module_name = module_name(workflow.name.as_ref());
    let lean_module_name = lean_module_name(module_name.clone());
    let quint_module_name = quint_module_name(module_name.clone());
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
            emit_lean_workflow_module(
                lean_module_name,
                workflow.name.clone(),
                workflow.description.clone(),
                workflow.slug.clone(),
                Vec::new(),
                Vec::new(),
                digest.clone(),
            ),
        ),
        Effect::WriteFile(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_workflow_module(
                quint_module_name,
                workflow.name.clone(),
                workflow.description.clone(),
                workflow.slug.clone(),
                Vec::new(),
                Vec::new(),
                digest,
            ),
        ),
        Effect::Report(report_line(format!(
            "{} workflow {workflow_name}",
            mutation_report.as_ref()
        ))),
    ])
}

impl MutationReport {
    fn as_ref(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Updated => "updated",
        }
    }
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
