use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::{Value, json};

use crate::core::digest::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, SliceSlug, WorkflowSlug,
    WorkflowTransitionLabel,
};

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
    let workflow_value = serde_json::from_str::<Value>(workflow_document.as_ref())
        .map_err(|error| SliceMutationError::new(format!("invalid workflow JSON: {error}")))?;
    let workflow_object = workflow_value
        .as_object()
        .ok_or_else(|| SliceMutationError::new("workflow document must be a JSON object"))?;
    let slice_file = slice_file(&new_slice);
    let relationship = workflow_object
        .get("steps")
        .and_then(Value::as_array)
        .filter(|steps| !steps.is_empty())
        .map_or("entry", |_| "main");
    let slice_files = appended_array_strings(
        workflow_object.get("slice_files").and_then(Value::as_array),
        slice_file.as_ref(),
    );
    let steps = appended_array_values(
        workflow_object.get("steps").and_then(Value::as_array),
        json!({
            "slice": new_slice.slug.as_ref(),
            "name": new_slice.name.as_ref(),
            "type": new_slice.kind.as_ref(),
            "relationship": relationship,
            "transitions": []
        }),
    );
    let workflow_name = workflow_name(workflow_object)?;
    let workflow_description = workflow_description(workflow_object)?;
    let module_name = module_name(workflow_name.as_ref());
    let workflow_slices = workflow_slices(&steps)?;
    let workflow_transitions = workflow_transitions(&steps)?;
    let digest = artifact_digest(workflow_name.clone());
    let workflow_json = workflow_json(workflow_object, slice_files, steps)?;
    let slice_json = slice_json(&new_slice);
    let slice_name = new_slice.name.as_ref();

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(
            project_path(format!(
                "model/browser/data/workflows/{}.eventmodel.json",
                new_slice.workflow_slug.as_ref()
            )),
            file_contents(workflow_json),
        ),
        Effect::WriteFile(
            project_path(format!(
                "model/browser/data/slices/{}.eventmodel.json",
                new_slice.slug.as_ref()
            )),
            file_contents(slice_json),
        ),
        Effect::WriteFile(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_workflow_module(
                lean_module_name(module_name.clone()),
                workflow_name.clone(),
                workflow_description.clone(),
                new_slice.workflow_slug.clone(),
                workflow_slices.clone(),
                workflow_transitions.clone(),
                digest.clone(),
            ),
        ),
        Effect::WriteFile(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_workflow_module(
                quint_module_name(module_name),
                workflow_name,
                workflow_description,
                new_slice.workflow_slug.clone(),
                workflow_slices,
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

fn workflow_json(
    workflow_object: &serde_json::Map<String, Value>,
    slice_files: Vec<Value>,
    steps: Vec<Value>,
) -> Result<String, SliceMutationError> {
    let mut next = workflow_object.clone();
    next.insert("slice_files".to_owned(), Value::Array(slice_files));
    next.insert("steps".to_owned(), Value::Array(steps));
    serde_json::to_string_pretty(&Value::Object(next))
        .map(|json| format!("{json}\n"))
        .map_err(|error| SliceMutationError::new(format!("invalid workflow JSON: {error}")))
}

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

fn appended_array_strings(existing: Option<&Vec<Value>>, new_value: &str) -> Vec<Value> {
    let mut values = existing.cloned().unwrap_or_default();
    if !values.iter().any(|value| value.as_str() == Some(new_value)) {
        values.push(Value::String(new_value.to_owned()));
    }
    values
}

fn appended_array_values(existing: Option<&Vec<Value>>, new_value: Value) -> Vec<Value> {
    let mut values = existing.cloned().unwrap_or_default();
    values.push(new_value);
    values
}

fn slice_file(new_slice: &NewSlice) -> String {
    format!("../slices/{}.eventmodel.json", new_slice.slug.as_ref())
}

fn workflow_name(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<ModelName, SliceMutationError> {
    workflow_object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| SliceMutationError::new("workflow document is missing name"))
        .and_then(|name| {
            ModelName::try_new(name.to_owned())
                .map_err(|error| SliceMutationError::new(format!("invalid workflow name: {error}")))
        })
}

fn workflow_description(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<ModelDescription, SliceMutationError> {
    workflow_object
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| SliceMutationError::new("workflow document is missing description"))
        .and_then(|description| {
            ModelDescription::try_new(description.to_owned()).map_err(|error| {
                SliceMutationError::new(format!("invalid workflow description: {error}"))
            })
        })
}

fn workflow_slices(steps: &[Value]) -> Result<Vec<SliceSlug>, SliceMutationError> {
    steps
        .iter()
        .filter_map(|step| step.get("slice").and_then(Value::as_str))
        .map(|slice| {
            SliceSlug::try_new(slice.to_owned())
                .map_err(|error| SliceMutationError::new(format!("invalid slice slug: {error}")))
        })
        .collect()
}

fn workflow_transitions(
    steps: &[Value],
) -> Result<Vec<WorkflowTransitionLabel>, SliceMutationError> {
    steps
        .iter()
        .filter_map(|step| {
            let source = step.get("slice").and_then(Value::as_str)?;
            let transitions = step.get("transitions").and_then(Value::as_array)?;
            Some((source, transitions))
        })
        .flat_map(|(source, transitions)| {
            transitions.iter().filter_map(move |transition| {
                let target = transition.get("to").and_then(Value::as_str)?;
                transition
                    .get("via_navigation")
                    .and_then(Value::as_str)
                    .map(|trigger| format!("{source}->{target}:navigation:{trigger}"))
            })
        })
        .map(|label| {
            WorkflowTransitionLabel::try_new(label).map_err(|error| {
                SliceMutationError::new(format!("invalid workflow transition: {error}"))
            })
        })
        .collect()
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
