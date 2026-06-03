use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{
    ModelDescription, ModelName, SliceKindName, SliceSlug, WorkflowSliceDetail, WorkflowSlug,
    WorkflowTransitionLabel,
};

pub fn artifact_digest(
    workflow_name: ModelName,
    workflow_slug: WorkflowSlug,
    workflow_description: ModelDescription,
    workflow_slice_details: Vec<WorkflowSliceDetail>,
    workflow_transitions: Vec<WorkflowTransitionLabel>,
) -> ArtifactDigest {
    ArtifactDigest::try_new(format!(
        "workflow:name={workflow_name};slug={workflow_slug};description={workflow_description};slices={};transitions={}",
        slice_details_digest(workflow_slice_details.as_slice()),
        transitions_digest(workflow_transitions.as_slice())
    ))
    .unwrap_or_else(|error| {
        unreachable!("EMC generated artifact digest must be valid: {error}");
    })
}

pub fn artifact_digest_from_workflow_document(
    workflow_slug: WorkflowSlug,
    workflow_document: FileContents,
) -> Result<ArtifactDigest, ArtifactDigestError> {
    let workflow_value = serde_json::from_str::<Value>(workflow_document.as_ref())
        .map_err(|error| ArtifactDigestError::new(format!("invalid workflow JSON: {error}")))?;
    let workflow_object = workflow_value
        .as_object()
        .ok_or_else(|| ArtifactDigestError::new("workflow document must be a JSON object"))?;

    Ok(artifact_digest(
        workflow_name(workflow_object)?,
        workflow_slug,
        workflow_description(workflow_object)?,
        workflow_slice_details(workflow_object)?,
        workflow_transitions(workflow_object)?,
    ))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ArtifactDigestError {
    message: String,
}

impl ArtifactDigestError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ArtifactDigestError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ArtifactDigestError {}

fn workflow_name(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<ModelName, ArtifactDigestError> {
    workflow_object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ArtifactDigestError::new("workflow document is missing name"))
        .and_then(|name| {
            ModelName::try_new(name.to_owned()).map_err(|error| {
                ArtifactDigestError::new(format!("invalid workflow name: {error}"))
            })
        })
}

fn workflow_description(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<ModelDescription, ArtifactDigestError> {
    workflow_object
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ArtifactDigestError::new("workflow document is missing description"))
        .and_then(|description| {
            ModelDescription::try_new(description.to_owned()).map_err(|error| {
                ArtifactDigestError::new(format!("invalid workflow description: {error}"))
            })
        })
}

fn workflow_slice_details(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<Vec<WorkflowSliceDetail>, ArtifactDigestError> {
    workflow_object
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| ArtifactDigestError::new("workflow document is missing steps"))?
        .iter()
        .map(workflow_slice_detail)
        .collect()
}

fn workflow_slice_detail(step: &Value) -> Result<WorkflowSliceDetail, ArtifactDigestError> {
    let slug = step
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ArtifactDigestError::new("workflow step is missing slice"))
        .and_then(|slice| {
            SliceSlug::try_new(slice.to_owned())
                .map_err(|error| ArtifactDigestError::new(format!("invalid slice slug: {error}")))
        })?;
    let name = step
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ArtifactDigestError::new("workflow step is missing name"))
        .and_then(|name| {
            ModelName::try_new(name.to_owned())
                .map_err(|error| ArtifactDigestError::new(format!("invalid slice name: {error}")))
        })?;
    let kind = step
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| ArtifactDigestError::new("workflow step is missing type"))
        .and_then(|kind| {
            SliceKindName::try_new(kind.to_owned())
                .map_err(|error| ArtifactDigestError::new(format!("invalid slice kind: {error}")))
        })?;
    let description = step
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ArtifactDigestError::new("workflow step is missing description"))
        .and_then(|description| {
            ModelDescription::try_new(description.to_owned()).map_err(|error| {
                ArtifactDigestError::new(format!("invalid slice description: {error}"))
            })
        })?;
    Ok(WorkflowSliceDetail::new(slug, name, kind, description))
}

fn workflow_transitions(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<Vec<WorkflowTransitionLabel>, ArtifactDigestError> {
    workflow_object
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| ArtifactDigestError::new("workflow document is missing steps"))?
        .iter()
        .filter_map(|step| {
            let source = step.get("slice").and_then(Value::as_str)?;
            let transitions = step.get("transitions").and_then(Value::as_array)?;
            Some((source, transitions))
        })
        .flat_map(|(source, transitions)| {
            transitions.iter().filter_map(move |transition| {
                transition
                    .get("to")
                    .and_then(Value::as_str)
                    .and_then(|target| transition_label(source, target, transition))
                    .or_else(|| {
                        transition
                            .get("to_workflow")
                            .and_then(Value::as_str)
                            .and_then(|target| {
                                workflow_exit_transition_label(source, target, transition)
                            })
                    })
            })
        })
        .map(|label| {
            WorkflowTransitionLabel::try_new(label).map_err(|error| {
                ArtifactDigestError::new(format!("invalid workflow transition: {error}"))
            })
        })
        .collect()
}

fn transition_label(source: &str, target: &str, transition: &Value) -> Option<String> {
    [
        ("via_command", "command"),
        ("via_event", "event"),
        ("via_navigation", "navigation"),
        ("via_external_trigger", "external_trigger"),
        ("via_outcome", "outcome"),
    ]
    .into_iter()
    .find_map(|(field, kind)| {
        transition
            .get(field)
            .and_then(Value::as_str)
            .map(|trigger| format!("{source}->{target}:{kind}:{trigger}"))
    })
}

fn workflow_exit_transition_label(
    source: &str,
    target: &str,
    transition: &Value,
) -> Option<String> {
    [
        ("via_command", "command"),
        ("via_event", "event"),
        ("via_navigation", "navigation"),
        ("via_external_trigger", "external_trigger"),
        ("via_outcome", "outcome"),
        ("exit_reason", "reason"),
    ]
    .into_iter()
    .find_map(|(field, kind)| {
        transition
            .get(field)
            .and_then(Value::as_str)
            .map(|trigger| format!("{source}->{target}:workflow_exit:{kind}:{trigger}"))
    })
}

fn slice_details_digest(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    workflow_slice_details
        .iter()
        .map(|slice| {
            [
                slice.slug().as_ref(),
                slice.name().as_ref(),
                slice.kind().as_ref(),
                slice.description().as_ref(),
            ]
            .join("|")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn transitions_digest(workflow_transitions: &[WorkflowTransitionLabel]) -> String {
    workflow_transitions
        .iter()
        .map(WorkflowTransitionLabel::as_ref)
        .collect::<Vec<_>>()
        .join(",")
}
