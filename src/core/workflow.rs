use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::digest::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::layout::ModeledWorkflowLayout;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, WorkflowSliceDetail,
    WorkflowSlug, WorkflowTransitionLabel,
};
use crate::core::workflow_document::{WorkflowDocument, workflow_path};

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
    existing_workflows: Vec<ModeledWorkflowLayout>,
    workflow: NewWorkflow,
) -> EffectPlan {
    workflow_effect_plan(existing_workflows, workflow)
}

pub fn update_workflow_description(
    existing_workflows: Vec<ModeledWorkflowLayout>,
    workflow_document: FileContents,
    slug: WorkflowSlug,
    description: ModelDescription,
) -> Result<EffectPlan, WorkflowMutationError> {
    let existing_workflow = existing_workflows
        .iter()
        .find(|existing| existing.slug() == &slug)
        .cloned()
        .ok_or_else(|| WorkflowMutationError::new(format!("unknown workflow {}", slug.as_ref())))?;
    let workflow_document = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    let workflow_name = workflow_document
        .name()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    if workflow_name != *existing_workflow.name() {
        return Err(WorkflowMutationError::new(format!(
            "workflow document name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            existing_workflow.name().as_ref()
        )));
    }
    let workflow_document = workflow_document
        .with_description(&description)
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    let workflow_json = workflow_document
        .contents()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    let workflow_slice_details = workflow_document
        .slice_details()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    let workflow_transitions = workflow_document
        .transitions()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;

    Ok(update_workflow_effect_plan(
        existing_workflows,
        NewWorkflow::new(workflow_name, description, slug),
        workflow_json,
        workflow_slice_details,
        workflow_transitions,
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

fn workflow_effect_plan(
    existing_workflows: Vec<ModeledWorkflowLayout>,
    workflow: NewWorkflow,
) -> EffectPlan {
    let workflow_name = workflow.name.as_ref();
    let workflow_description = workflow.description.as_ref();
    let module_name = module_name(workflow.name.as_ref());
    let lean_module_name = lean_module_name(module_name.clone());
    let quint_module_name = quint_module_name(module_name.clone());
    let digest = artifact_digest(
        workflow.name.clone(),
        workflow.slug.clone(),
        workflow.description.clone(),
        Vec::new(),
        Vec::new(),
    );
    let workflow_layout = ModeledWorkflowLayout::new(
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
            workflow_path(&workflow.slug),
            file_contents(format!(
                "{{\n  \"name\": {},\n  \"version\": \"0.1.0\",\n  \"description\": {},\n  \"board\": {{}},\n  \"streams\": [],\n  \"events\": [],\n  \"commands\": [],\n  \"read_models\": [],\n  \"slices\": [],\n  \"slice_files\": [],\n  \"steps\": []\n}}\n",
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
        Effect::Report(report_line(format!("added workflow {workflow_name}"))),
    ])
}

fn update_workflow_effect_plan(
    existing_workflows: Vec<ModeledWorkflowLayout>,
    workflow: NewWorkflow,
    workflow_json: FileContents,
    workflow_slice_details: Vec<WorkflowSliceDetail>,
    workflow_transitions: Vec<WorkflowTransitionLabel>,
) -> EffectPlan {
    let workflow_name = workflow.name.as_ref();
    let module_name = module_name(workflow.name.as_ref());
    let lean_module_name = lean_module_name(module_name.clone());
    let quint_module_name = quint_module_name(module_name.clone());
    let digest = artifact_digest(
        workflow.name.clone(),
        workflow.slug.clone(),
        workflow.description.clone(),
        workflow_slice_details.clone(),
        workflow_transitions.clone(),
    );
    let workflow_layout = ModeledWorkflowLayout::new(
        workflow.name.clone(),
        workflow.description.clone(),
        workflow.slug.clone(),
    );
    let updated_slug = workflow.slug.clone();
    let workflows = existing_workflows
        .into_iter()
        .filter(|existing| existing.slug() != &updated_slug)
        .chain([workflow_layout])
        .collect::<Vec<_>>();

    EffectPlan::new(vec![
        Effect::WriteFile(workflow_path(&workflow.slug), workflow_json),
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
                workflow_slice_details.clone(),
                workflow_transitions.clone(),
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
                workflow_slice_details,
                workflow_transitions,
                digest,
            ),
        ),
        Effect::Report(report_line(format!("updated workflow {workflow_name}"))),
    ])
}

fn browser_index(mut workflows: Vec<ModeledWorkflowLayout>) -> String {
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

fn workflow_index_entry(workflow: &ModeledWorkflowLayout) -> String {
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
