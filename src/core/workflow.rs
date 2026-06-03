use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::digest::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::layout::{ModeledWorkflowLayout, ModeledWorkflowLayouts};
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, WorkflowSliceDetail,
    WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionRecord, WorkflowTransitionRecords,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexedWorkflowDocument {
    slug: WorkflowSlug,
    contents: FileContents,
}

impl IndexedWorkflowDocument {
    pub fn new(slug: WorkflowSlug, contents: FileContents) -> Self {
        Self { slug, contents }
    }

    pub fn slug(&self) -> &WorkflowSlug {
        &self.slug
    }

    pub fn contents(&self) -> &FileContents {
        &self.contents
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexedWorkflowDocuments {
    documents: Vec<IndexedWorkflowDocument>,
}

impl IndexedWorkflowDocuments {
    pub(crate) fn new(documents: Vec<IndexedWorkflowDocument>) -> Self {
        Self { documents }
    }

    fn as_slice(&self) -> &[IndexedWorkflowDocument] {
        &self.documents
    }
}

pub fn add_workflow(
    existing_workflows: ModeledWorkflowLayouts,
    workflow: NewWorkflow,
) -> Result<EffectPlan, WorkflowMutationError> {
    reject_workflow_slug_collision(existing_workflows.as_slice(), &workflow)?;
    reject_workflow_module_collision(existing_workflows.as_slice(), &workflow)?;
    Ok(workflow_effect_plan(
        existing_workflows.into_inner(),
        workflow,
    ))
}

pub fn update_workflow_description(
    existing_workflows: ModeledWorkflowLayouts,
    workflow_document: FileContents,
    slug: WorkflowSlug,
    description: ModelDescription,
) -> Result<EffectPlan, WorkflowMutationError> {
    let existing_workflow = existing_workflows
        .as_slice()
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
        existing_workflows.into_inner(),
        NewWorkflow::new(workflow_name, description, slug),
        workflow_json,
        workflow_slice_details,
        workflow_transitions,
        None,
    ))
}

pub fn update_workflow_name(
    existing_workflows: ModeledWorkflowLayouts,
    workflow_document: FileContents,
    slug: WorkflowSlug,
    name: ModelName,
) -> Result<EffectPlan, WorkflowMutationError> {
    let existing_workflow = existing_workflows
        .as_slice()
        .iter()
        .find(|existing| existing.slug() == &slug)
        .cloned()
        .ok_or_else(|| WorkflowMutationError::new(format!("unknown workflow {}", slug.as_ref())))?;
    let updated_workflow = NewWorkflow::new(
        name.clone(),
        existing_workflow.description().clone(),
        slug.clone(),
    );
    reject_workflow_module_collision(existing_workflows.as_slice(), &updated_workflow)?;

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
    let workflow_description = workflow_document
        .description()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    if workflow_description != *existing_workflow.description() {
        return Err(WorkflowMutationError::new(format!(
            "workflow document description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            existing_workflow.description().as_ref()
        )));
    }

    let workflow_document = workflow_document
        .with_name(&name)
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
        existing_workflows.into_inner(),
        updated_workflow,
        workflow_json,
        workflow_slice_details,
        workflow_transitions,
        Some(module_name(existing_workflow.name().as_ref())),
    ))
}

pub fn remove_workflow(
    existing_workflows: ModeledWorkflowLayouts,
    workflow_documents: IndexedWorkflowDocuments,
    slug: WorkflowSlug,
) -> Result<EffectPlan, WorkflowMutationError> {
    let removed_workflow = existing_workflows
        .as_slice()
        .iter()
        .find(|existing| existing.slug() == &slug)
        .cloned()
        .ok_or_else(|| WorkflowMutationError::new(format!("unknown workflow {}", slug.as_ref())))?;
    reject_incoming_workflow_references(workflow_documents.as_slice(), &slug)?;
    let workflow_document = workflow_documents
        .as_slice()
        .iter()
        .find(|document| document.slug() == &slug)
        .ok_or_else(|| {
            WorkflowMutationError::new(format!("workflow {} document is missing", slug.as_ref()))
        })
        .and_then(|document| {
            WorkflowDocument::parse(document.contents())
                .map_err(|error| WorkflowMutationError::new(error.to_string()))
        })?;
    let workflow_name = workflow_document
        .name()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    if workflow_name != *removed_workflow.name() {
        return Err(WorkflowMutationError::new(format!(
            "workflow document name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            removed_workflow.name().as_ref()
        )));
    }

    let remaining_workflows = existing_workflows
        .into_inner()
        .into_iter()
        .filter(|existing| existing.slug() != &slug)
        .collect::<Vec<_>>();
    let removed_slice_details = workflow_document
        .slice_details()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    let workflow_module_name = module_name(removed_workflow.name().as_ref());
    let workflow_name = removed_workflow.name().as_ref().to_owned();

    let remove_slice_effects = removed_slice_details
        .into_iter()
        .flat_map(remove_slice_artifact_effects);
    let effects = [
        Effect::WriteFile(
            project_path("model/browser/data/index.json"),
            file_contents(browser_index(remaining_workflows)),
        ),
        Effect::RemoveFile(workflow_path(&slug)),
        Effect::RemoveFile(project_path(format!(
            "model/lean/{workflow_module_name}.lean"
        ))),
        Effect::RemoveFile(project_path(format!(
            "model/quint/{workflow_module_name}.qnt"
        ))),
    ]
    .into_iter()
    .chain(remove_slice_effects)
    .chain([Effect::Report(report_line(format!(
        "removed workflow {workflow_name}"
    )))])
    .collect::<Vec<_>>();

    Ok(EffectPlan::new(effects))
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
        WorkflowSliceDetails::from_details([]),
        WorkflowTransitionRecords::from_records([]),
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
                WorkflowSliceDetails::from_details([]),
                WorkflowTransitionRecords::from_records([]),
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
                WorkflowSliceDetails::from_details([]),
                WorkflowTransitionRecords::from_records([]),
                digest,
            ),
        ),
        Effect::Report(report_line(format!("added workflow {workflow_name}"))),
    ])
}

fn reject_incoming_workflow_references(
    workflow_documents: &[IndexedWorkflowDocument],
    removed_slug: &WorkflowSlug,
) -> Result<(), WorkflowMutationError> {
    workflow_documents
        .iter()
        .filter(|document| document.slug() != removed_slug)
        .find_map(|document| incoming_workflow_reference(document, removed_slug).transpose())
        .transpose()
        .map(|reference| {
            reference.map_or(Ok(()), |referencing_slug| {
                Err(WorkflowMutationError::new(format!(
                    "workflow {} is referenced by workflow {}",
                    removed_slug.as_ref(),
                    referencing_slug.as_ref()
                )))
            })
        })?
}

fn incoming_workflow_reference(
    document: &IndexedWorkflowDocument,
    removed_slug: &WorkflowSlug,
) -> Result<Option<WorkflowSlug>, WorkflowMutationError> {
    let workflow_document = WorkflowDocument::parse(document.contents())
        .map_err(|error| WorkflowMutationError::new(error.to_string()))?;
    workflow_document
        .transitions()
        .map_err(|error| WorkflowMutationError::new(error.to_string()))
        .map(|transitions| {
            transitions
                .into_iter()
                .any(|transition| {
                    transition.kind().as_ref().starts_with("workflow_exit:")
                        && transition.target().as_ref() == removed_slug.as_ref()
                })
                .then(|| document.slug().clone())
        })
}

fn remove_slice_artifact_effects(slice: WorkflowSliceDetail) -> [Effect; 3] {
    let slice_slug = slice.slug().as_ref();
    let module_name = module_name(slice.name().as_ref());
    [
        Effect::RemoveFile(project_path(format!(
            "model/browser/data/slices/{slice_slug}.eventmodel.json"
        ))),
        Effect::RemoveFile(project_path(format!(
            "model/lean/slices/{module_name}.lean"
        ))),
        Effect::RemoveFile(project_path(format!(
            "model/quint/slices/{module_name}.qnt"
        ))),
    ]
}

fn update_workflow_effect_plan(
    existing_workflows: Vec<ModeledWorkflowLayout>,
    workflow: NewWorkflow,
    workflow_json: FileContents,
    workflow_slice_details: Vec<WorkflowSliceDetail>,
    workflow_transitions: Vec<WorkflowTransitionRecord>,
    previous_module_name: Option<String>,
) -> EffectPlan {
    let workflow_name = workflow.name.as_ref();
    let module_name = module_name(workflow.name.as_ref());
    let lean_module_name = lean_module_name(module_name.clone());
    let quint_module_name = quint_module_name(module_name.clone());
    let digest = artifact_digest(
        workflow.name.clone(),
        workflow.slug.clone(),
        workflow.description.clone(),
        WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
        WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
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

    let cleanup_effects = previous_module_name
        .filter(|previous_module_name| previous_module_name != &module_name)
        .into_iter()
        .flat_map(|previous_module_name| {
            [
                Effect::RemoveFile(project_path(format!(
                    "model/lean/{previous_module_name}.lean"
                ))),
                Effect::RemoveFile(project_path(format!(
                    "model/quint/{previous_module_name}.qnt"
                ))),
            ]
        });

    EffectPlan::new(
        cleanup_effects
            .chain([
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
                        WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
                        WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
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
                        WorkflowSliceDetails::from_details(workflow_slice_details),
                        WorkflowTransitionRecords::from_records(workflow_transitions),
                        digest,
                    ),
                ),
                Effect::Report(report_line(format!("updated workflow {workflow_name}"))),
            ])
            .collect(),
    )
}

fn reject_workflow_module_collision(
    existing_workflows: &[ModeledWorkflowLayout],
    workflow: &NewWorkflow,
) -> Result<(), WorkflowMutationError> {
    let generated_module_name = module_name(workflow.name.as_ref());
    existing_workflows
        .iter()
        .filter(|existing| existing.slug() != &workflow.slug)
        .find(|existing| module_name(existing.name().as_ref()) == generated_module_name)
        .map_or(Ok(()), |_existing| {
            Err(WorkflowMutationError::new(format!(
                "workflow module {generated_module_name} already exists"
            )))
        })
}

fn reject_workflow_slug_collision(
    existing_workflows: &[ModeledWorkflowLayout],
    workflow: &NewWorkflow,
) -> Result<(), WorkflowMutationError> {
    existing_workflows
        .iter()
        .find(|existing| existing.slug() == &workflow.slug)
        .map_or(Ok(()), |_existing| {
            Err(WorkflowMutationError::new(format!(
                "workflow {} already exists",
                workflow.slug.as_ref()
            )))
        })
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
