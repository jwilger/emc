use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::digest::{artifact_digest, slice_artifact_digest};
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::lean::{
    emit_slice_module as emit_lean_slice_module, emit_workflow_module as emit_lean_workflow_module,
};
use crate::core::emit::quint::{
    emit_slice_module as emit_quint_slice_module,
    emit_workflow_module as emit_quint_workflow_module,
};
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, SliceKindName, SliceSlug,
    WorkflowSliceDetail, WorkflowSliceFileReference, WorkflowSlug,
};
use crate::core::workflow_document::{WorkflowDocument, WorkflowSliceAddition, workflow_path};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SliceKind {
    StateView,
    StateChange,
    Translation,
    Automation,
}

impl SliceKind {
    pub fn state_view() -> Self {
        Self::StateView
    }

    pub fn state_change() -> Self {
        Self::StateChange
    }

    pub fn translation() -> Self {
        Self::Translation
    }

    pub fn automation() -> Self {
        Self::Automation
    }

    fn as_ref(self) -> &'static str {
        match self {
            Self::StateView => "state_view",
            Self::StateChange => "state_change",
            Self::Translation => "translation",
            Self::Automation => "automation",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewSlice {
    workflow_slug: WorkflowSlug,
    slug: SliceSlug,
    name: ModelName,
    description: ModelDescription,
    kind: SliceKind,
}

impl NewSlice {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slug: SliceSlug,
        name: ModelName,
        description: ModelDescription,
        kind: SliceKind,
    ) -> Self {
        Self {
            workflow_slug,
            slug,
            name,
            description,
            kind,
        }
    }

    pub fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }
}

pub fn add_slice(
    workflow_document: FileContents,
    new_slice: NewSlice,
) -> Result<EffectPlan, SliceMutationError> {
    let workflow_document = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let existing_slice_details = workflow_document
        .slice_details()
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    existing_slice_details
        .iter()
        .find(|slice| slice.slug() == &new_slice.slug)
        .map_or(Ok(()), |_slice| {
            Err(SliceMutationError::new(format!(
                "slice {} already exists",
                new_slice.slug.as_ref()
            )))
        })?;
    let slice_module_name = module_name(new_slice.name.as_ref());
    existing_slice_details
        .iter()
        .find(|slice| module_name(slice.name().as_ref()) == slice_module_name)
        .map_or(Ok(()), |_slice| {
            Err(SliceMutationError::new(format!(
                "slice module {slice_module_name} already exists"
            )))
        })?;
    let relationship = workflow_document
        .next_slice_relationship()
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let workflow_document = workflow_document
        .with_added_slice(WorkflowSliceAddition::new(
            slice_file(&new_slice),
            workflow_slice_detail(&new_slice),
            relationship,
        ))
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let workflow_name = workflow_document
        .name()
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let workflow_description = workflow_document
        .description()
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let workflow_module_name = module_name(workflow_name.as_ref());
    let workflow_slice_details = workflow_document
        .slice_details()
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let workflow_transitions = workflow_document
        .transitions()
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let digest = artifact_digest(
        workflow_name.clone(),
        new_slice.workflow_slug.clone(),
        workflow_description.clone(),
        workflow_slice_details.clone(),
        workflow_transitions.clone(),
    );
    let workflow_json = workflow_document
        .contents()
        .map_err(|error| SliceMutationError::new(error.to_string()))?;
    let slice_json = slice_json(&new_slice);
    let slice_name = new_slice.name.as_ref();
    let slice_kind = slice_kind_name(new_slice.kind);
    let slice_digest = slice_artifact_digest(
        new_slice.name.clone(),
        new_slice.slug.clone(),
        slice_kind.clone(),
        new_slice.description.clone(),
    );

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(workflow_path(&new_slice.workflow_slug), workflow_json),
        Effect::WriteFile(
            project_path(format!(
                "model/browser/data/slices/{}.eventmodel.json",
                new_slice.slug.as_ref()
            )),
            file_contents(slice_json),
        ),
        Effect::WriteFile(
            project_path(format!("model/lean/slices/{slice_module_name}.lean")),
            emit_lean_slice_module(
                lean_module_name(slice_module_name.clone()),
                new_slice.name.clone(),
                new_slice.description.clone(),
                new_slice.slug.clone(),
                slice_kind.clone(),
                slice_digest.clone(),
            ),
        ),
        Effect::WriteFile(
            project_path(format!("model/quint/slices/{slice_module_name}.qnt")),
            emit_quint_slice_module(
                quint_module_name(slice_module_name),
                new_slice.name.clone(),
                new_slice.description.clone(),
                new_slice.slug.clone(),
                slice_kind,
                slice_digest,
            ),
        ),
        Effect::WriteFile(
            project_path(format!("model/lean/{workflow_module_name}.lean")),
            emit_lean_workflow_module(
                lean_module_name(workflow_module_name.clone()),
                workflow_name.clone(),
                workflow_description.clone(),
                new_slice.workflow_slug.clone(),
                workflow_slice_details.clone(),
                workflow_transitions.clone(),
                digest.clone(),
            ),
        ),
        Effect::WriteFile(
            project_path(format!("model/quint/{workflow_module_name}.qnt")),
            emit_quint_workflow_module(
                quint_module_name(workflow_module_name),
                workflow_name,
                workflow_description,
                new_slice.workflow_slug.clone(),
                workflow_slice_details,
                workflow_transitions,
                digest,
            ),
        ),
        Effect::Report(report_line(format!("added slice {slice_name}"))),
    ]))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceMutationError {
    message: String,
}

impl SliceMutationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for SliceMutationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for SliceMutationError {}

fn slice_json(new_slice: &NewSlice) -> String {
    format!(
        "{{\n  \"name\": {},\n  \"version\": \"0.1.0\",\n  \"description\": {},\n  \"type\": {},\n  \"board\": {{}},\n  \"streams\": [],\n  \"events\": [],\n  \"commands\": [],\n  \"read_models\": [],\n  \"views\": [],\n  \"slices\": [\n    {{\n      \"name\": {},\n      \"type\": {},\n      \"events\": [],\n      \"views\": [],\n      \"acceptance_scenarios\": [],\n      \"contract_scenarios\": []\n    }}\n  ]\n}}\n",
        json_string(new_slice.name.as_ref()),
        json_string(new_slice.description.as_ref()),
        json_string(new_slice.kind.as_ref()),
        json_string(new_slice.name.as_ref()),
        json_string(new_slice.kind.as_ref()),
    )
}

fn slice_file(new_slice: &NewSlice) -> WorkflowSliceFileReference {
    WorkflowSliceFileReference::try_new(format!(
        "../slices/{}.eventmodel.json",
        new_slice.slug.as_ref()
    ))
    .unwrap_or_else(|error| {
        unreachable!("EMC generated slice file reference must be valid: {error}")
    })
}

fn workflow_slice_detail(new_slice: &NewSlice) -> WorkflowSliceDetail {
    WorkflowSliceDetail::new(
        new_slice.slug.clone(),
        new_slice.name.clone(),
        slice_kind_name(new_slice.kind),
        new_slice.description.clone(),
    )
}

fn slice_kind_name(kind: SliceKind) -> SliceKindName {
    SliceKindName::try_new(kind.as_ref().to_owned())
        .unwrap_or_else(|error| unreachable!("EMC generated slice kind must be valid: {error}"))
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
