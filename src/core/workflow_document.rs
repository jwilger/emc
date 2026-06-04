use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::FileContents;
use crate::core::types::{
    ModelDescription, ModelName, PayloadContractName, SliceKindName, SliceSlug,
    TransitionTriggerName, WorkflowSliceDetail, WorkflowSliceFileReference,
    WorkflowStepRelationshipName, WorkflowTransitionEndpoint, WorkflowTransitionKind,
    WorkflowTransitionRecord,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowDocument {
    value: Value,
}

impl WorkflowDocument {
    pub fn parse(contents: &FileContents) -> Result<Self, WorkflowDocumentError> {
        Self::parse_optional(contents)?
            .ok_or_else(|| WorkflowDocumentError::new("workflow document must be a JSON object"))
    }

    pub fn parse_optional(contents: &FileContents) -> Result<Option<Self>, WorkflowDocumentError> {
        let value = serde_json::from_str::<Value>(contents.as_ref()).map_err(|error| {
            WorkflowDocumentError::new(format!("invalid workflow JSON: {error}"))
        })?;
        if value.as_object().is_none() {
            return Ok(None);
        }
        Ok(Some(Self { value }))
    }

    pub fn name(&self) -> Result<ModelName, WorkflowDocumentError> {
        self.string_field("name")
            .and_then(|name| model_name("workflow name", name))
    }

    pub fn description(&self) -> Result<ModelDescription, WorkflowDocumentError> {
        self.string_field("description")
            .and_then(|description| model_description("workflow description", description))
    }

    pub(crate) fn slice_details(&self) -> Result<Vec<WorkflowSliceDetail>, WorkflowDocumentError> {
        self.steps()?.iter().map(workflow_slice_detail).collect()
    }

    pub(crate) fn slice_files(
        &self,
    ) -> Result<Vec<WorkflowSliceFileReference>, WorkflowDocumentError> {
        self.optional_slice_files()?
            .ok_or_else(|| WorkflowDocumentError::new("workflow document is missing slice_files"))
    }

    pub(crate) fn optional_slice_files(
        &self,
    ) -> Result<Option<Vec<WorkflowSliceFileReference>>, WorkflowDocumentError> {
        self.object()?
            .get("slice_files")
            .and_then(Value::as_array)
            .map(|slice_files| {
                slice_files
                    .iter()
                    .filter_map(Value::as_str)
                    .map(workflow_slice_file_reference)
                    .collect()
            })
            .transpose()
    }

    pub(crate) fn transitions(
        &self,
    ) -> Result<Vec<WorkflowTransitionRecord>, WorkflowDocumentError> {
        workflow_transitions(self.steps()?)
    }

    fn object(&self) -> Result<&serde_json::Map<String, Value>, WorkflowDocumentError> {
        self.value
            .as_object()
            .ok_or_else(|| WorkflowDocumentError::new("workflow document must be a JSON object"))
    }

    fn string_field(&self, field: &str) -> Result<&str, WorkflowDocumentError> {
        self.object()?
            .get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| {
                WorkflowDocumentError::new(format!("workflow document is missing {field}"))
            })
    }

    fn steps(&self) -> Result<&[Value], WorkflowDocumentError> {
        self.object()?
            .get("steps")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .ok_or_else(|| WorkflowDocumentError::new("workflow document is missing steps"))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowDocumentError {
    message: String,
}

impl WorkflowDocumentError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for WorkflowDocumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowDocumentError {}

fn workflow_slice_detail(step: &Value) -> Result<WorkflowSliceDetail, WorkflowDocumentError> {
    let slug = step
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| WorkflowDocumentError::new("workflow step is missing slice"))
        .and_then(slice_slug)?;
    let name = step
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| WorkflowDocumentError::new("workflow step is missing name"))
        .and_then(|raw| model_name("slice name", raw))?;
    let kind = step
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| WorkflowDocumentError::new("workflow step is missing type"))
        .and_then(slice_kind)?;
    let description = step
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| WorkflowDocumentError::new("workflow step is missing description"))
        .and_then(|raw| model_description("slice description", raw))?;
    let relationship = step
        .get("relationship")
        .and_then(Value::as_str)
        .map(workflow_step_relationship_name)
        .transpose()?
        .unwrap_or_else(default_workflow_step_relationship);
    Ok(WorkflowSliceDetail::new_with_relationship(
        slug,
        name,
        kind,
        description,
        relationship,
    ))
}

fn workflow_transitions(
    steps: &[Value],
) -> Result<Vec<WorkflowTransitionRecord>, WorkflowDocumentError> {
    steps
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
                    .and_then(|target| transition_record(source, target, transition))
                    .or_else(|| {
                        transition
                            .get("to_workflow")
                            .and_then(Value::as_str)
                            .and_then(|target| {
                                workflow_exit_transition_record(source, target, transition)
                            })
                    })
            })
        })
        .map(workflow_transition_record)
        .collect()
}

fn transition_record<'a>(
    source: &'a str,
    target: &'a str,
    transition: &'a Value,
) -> Option<RawTransitionRecord<'a>> {
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
            .map(|trigger| RawTransitionRecord {
                source,
                target,
                kind: RawTransitionKind::Plain(kind),
                trigger,
                rationale: None,
                payload_contract: transition.get("payload_contract").and_then(Value::as_str),
            })
    })
}

fn workflow_exit_transition_record<'a>(
    source: &'a str,
    target: &'a str,
    transition: &'a Value,
) -> Option<RawTransitionRecord<'a>> {
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
            .map(|trigger| RawTransitionRecord {
                source,
                target,
                kind: RawTransitionKind::WorkflowExit(kind),
                trigger,
                rationale: transition.get("exit_reason").and_then(Value::as_str),
                payload_contract: transition.get("payload_contract").and_then(Value::as_str),
            })
    })
}

struct RawTransitionRecord<'a> {
    source: &'a str,
    target: &'a str,
    kind: RawTransitionKind<'a>,
    trigger: &'a str,
    rationale: Option<&'a str>,
    payload_contract: Option<&'a str>,
}

enum RawTransitionKind<'a> {
    Plain(&'a str),
    WorkflowExit(&'a str),
}

fn slice_slug(raw: &str) -> Result<SliceSlug, WorkflowDocumentError> {
    SliceSlug::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid slice slug: {error}")))
}

fn model_name(context: &str, raw: &str) -> Result<ModelName, WorkflowDocumentError> {
    ModelName::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid {context}: {error}")))
}

fn slice_kind(raw: &str) -> Result<SliceKindName, WorkflowDocumentError> {
    SliceKindName::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid slice kind: {error}")))
}

fn workflow_slice_file_reference(
    raw: &str,
) -> Result<WorkflowSliceFileReference, WorkflowDocumentError> {
    WorkflowSliceFileReference::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow slice file reference: {error}"))
    })
}

fn workflow_transition_record(
    raw: RawTransitionRecord<'_>,
) -> Result<WorkflowTransitionRecord, WorkflowDocumentError> {
    let kind = match raw.kind {
        RawTransitionKind::Plain(kind) => kind.to_owned(),
        RawTransitionKind::WorkflowExit(kind) => format!("workflow_exit:{kind}"),
    };
    let source = workflow_transition_endpoint("source", raw.source)?;
    let target = workflow_transition_endpoint("target", raw.target)?;
    let kind = workflow_transition_kind(&kind)?;
    let trigger = transition_trigger_name(raw.trigger)?;
    match (raw.rationale, raw.payload_contract) {
        (None, Some(payload_contract)) => Ok(WorkflowTransitionRecord::new_with_payload_contract(
            source,
            target,
            kind,
            trigger,
            payload_contract_name(payload_contract)?,
        )),
        (Some(rationale), _) => Ok(WorkflowTransitionRecord::new_with_rationale(
            source,
            target,
            kind,
            trigger,
            model_description("workflow transition rationale", rationale)?,
        )),
        (None, None) => Ok(WorkflowTransitionRecord::new(source, target, kind, trigger)),
    }
}

fn workflow_transition_endpoint(
    context: &str,
    raw: &str,
) -> Result<WorkflowTransitionEndpoint, WorkflowDocumentError> {
    WorkflowTransitionEndpoint::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow transition {context}: {error}"))
    })
}

fn transition_trigger_name(raw: &str) -> Result<TransitionTriggerName, WorkflowDocumentError> {
    TransitionTriggerName::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow transition trigger: {error}"))
    })
}

fn payload_contract_name(raw: &str) -> Result<PayloadContractName, WorkflowDocumentError> {
    PayloadContractName::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid payload contract: {error}")))
}

fn workflow_transition_kind(raw: &str) -> Result<WorkflowTransitionKind, WorkflowDocumentError> {
    WorkflowTransitionKind::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow transition kind: {error}"))
    })
}

fn workflow_step_relationship_name(
    raw: &str,
) -> Result<WorkflowStepRelationshipName, WorkflowDocumentError> {
    WorkflowStepRelationshipName::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow step relationship: {error}"))
    })
}

fn default_workflow_step_relationship() -> WorkflowStepRelationshipName {
    WorkflowStepRelationshipName::try_new("main".to_owned()).unwrap_or_else(|error| {
        unreachable!("EMC generated workflow step relationship must be valid: {error}");
    })
}

fn model_description(context: &str, raw: &str) -> Result<ModelDescription, WorkflowDocumentError> {
    ModelDescription::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid {context}: {error}")))
}
