use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::{FileContents, ProjectPath};
use crate::core::types::{
    ModelDescription, ModelName, ReviewRuleName, ReviewStatus, SliceKindName, SliceSlug,
    TransitionTriggerName, WorkflowBranchDetail, WorkflowBranchLabel, WorkflowReviewOverlayDetail,
    WorkflowSliceDetail, WorkflowSliceFileReference, WorkflowSlug, WorkflowStepName,
    WorkflowStepRelationshipName, WorkflowTransitionDetail, WorkflowTransitionFieldName,
    WorkflowTransitionKind, WorkflowTransitionLabel, WorkflowTransitionName,
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

    pub fn next_slice_relationship(
        &self,
    ) -> Result<WorkflowStepRelationshipName, WorkflowDocumentError> {
        let relationship = if self.steps()?.is_empty() {
            "entry"
        } else {
            "main"
        };
        WorkflowStepRelationshipName::try_new(relationship.to_owned()).map_err(|error| {
            WorkflowDocumentError::new(format!("invalid workflow step relationship: {error}"))
        })
    }

    pub fn with_added_slice(
        &self,
        addition: WorkflowSliceAddition,
    ) -> Result<Self, WorkflowDocumentError> {
        let mut next = self.object()?.clone();
        next.insert(
            "slice_files".to_owned(),
            Value::Array(appended_slice_files(
                next.get("slice_files").and_then(Value::as_array),
                addition.slice_file.as_ref(),
            )),
        );
        next.insert(
            "steps".to_owned(),
            Value::Array(appended_steps(
                next.get("steps").and_then(Value::as_array),
                addition,
            )),
        );
        Ok(Self {
            value: Value::Object(next),
        })
    }

    pub fn with_connected_transition(
        &self,
        addition: WorkflowTransitionAddition,
    ) -> Result<Self, WorkflowDocumentError> {
        reject_unknown_transition_target(self.steps()?, &addition)?;
        reject_duplicate_transition(self.steps()?, &addition)?;
        let mut next = self.object()?.clone();
        next.insert(
            "steps".to_owned(),
            Value::Array(connected_steps(self.steps()?, addition)?),
        );
        Ok(Self {
            value: Value::Object(next),
        })
    }

    pub fn with_description(
        &self,
        description: &ModelDescription,
    ) -> Result<Self, WorkflowDocumentError> {
        let mut next = self.object()?.clone();
        next.insert(
            "description".to_owned(),
            Value::String(description.as_ref().to_owned()),
        );
        Ok(Self {
            value: Value::Object(next),
        })
    }

    pub fn slice_details(&self) -> Result<Vec<WorkflowSliceDetail>, WorkflowDocumentError> {
        self.steps()?.iter().map(workflow_slice_detail).collect()
    }

    pub fn main_path_step_names(&self) -> Result<Vec<WorkflowStepName>, WorkflowDocumentError> {
        self.steps()?
            .iter()
            .filter(|step| {
                step.get("relationship")
                    .and_then(Value::as_str)
                    .is_some_and(|relationship| relationship == "entry" || relationship == "main")
            })
            .filter_map(|step| step.get("name").and_then(Value::as_str))
            .map(workflow_step_name)
            .collect()
    }

    pub fn branch_details(&self) -> Result<Vec<WorkflowBranchDetail>, WorkflowDocumentError> {
        let steps = self.steps()?;

        steps
            .iter()
            .filter(|step| {
                step.get("relationship")
                    .and_then(Value::as_str)
                    .is_some_and(|relationship| relationship != "entry" && relationship != "main")
            })
            .filter_map(|step| {
                step.get("name")
                    .and_then(Value::as_str)
                    .zip(step.get("relationship").and_then(Value::as_str))
                    .zip(Some(step))
            })
            .map(|((name, relationship), step)| {
                Ok(WorkflowBranchDetail::new(
                    workflow_step_name(name)?,
                    workflow_branch_label(steps, step, relationship)?,
                ))
            })
            .collect()
    }

    pub fn slice_files(&self) -> Result<Vec<WorkflowSliceFileReference>, WorkflowDocumentError> {
        self.optional_slice_files()?
            .ok_or_else(|| WorkflowDocumentError::new("workflow document is missing slice_files"))
    }

    pub fn optional_slice_files(
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

    pub fn transitions(&self) -> Result<Vec<WorkflowTransitionLabel>, WorkflowDocumentError> {
        workflow_transitions(self.steps()?)
    }

    pub fn transition_details(
        &self,
    ) -> Result<Vec<WorkflowTransitionDetail>, WorkflowDocumentError> {
        let steps = self.steps()?;

        steps
            .iter()
            .map(|step| step_transition_details(steps, step))
            .collect::<Result<Vec<_>, _>>()
            .map(|details| details.into_iter().flatten().collect())
    }

    pub fn review_overlay_details(
        &self,
    ) -> Result<Vec<WorkflowReviewOverlayDetail>, WorkflowDocumentError> {
        self.object()?
            .get("review_diagnostics")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|diagnostic| {
                Some((
                    diagnostic.get("step").and_then(Value::as_str)?,
                    diagnostic.get("status").and_then(Value::as_str)?,
                    diagnostic.get("missing_rule").and_then(Value::as_str)?,
                ))
            })
            .map(|(step, status, missing_rule)| {
                Ok(WorkflowReviewOverlayDetail::new(
                    workflow_step_name(step)?,
                    review_status(status)?,
                    review_rule_name(missing_rule)?,
                ))
            })
            .collect()
    }

    pub fn contents(&self) -> Result<FileContents, WorkflowDocumentError> {
        serde_json::to_string_pretty(&self.value)
            .map(|json| format!("{json}\n"))
            .map_err(|error| WorkflowDocumentError::new(format!("invalid workflow JSON: {error}")))
            .and_then(|json| {
                FileContents::try_new(json).map_err(|error| {
                    WorkflowDocumentError::new(format!("invalid workflow contents: {error}"))
                })
            })
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
pub struct WorkflowSliceAddition {
    slice_file: WorkflowSliceFileReference,
    detail: WorkflowSliceDetail,
    relationship: WorkflowStepRelationshipName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionAddition {
    source: SliceSlug,
    target: WorkflowTransitionTarget,
    trigger_field: WorkflowTransitionFieldName,
    trigger: TransitionTriggerName,
}

impl WorkflowTransitionAddition {
    pub fn new(
        source: SliceSlug,
        target: WorkflowTransitionTarget,
        trigger_field: WorkflowTransitionFieldName,
        trigger: TransitionTriggerName,
    ) -> Self {
        Self {
            source,
            target,
            trigger_field,
            trigger,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkflowTransitionTarget {
    Slice(SliceSlug),
    Workflow {
        slug: WorkflowSlug,
        reason: ModelDescription,
    },
}

impl WorkflowSliceAddition {
    pub fn new(
        slice_file: WorkflowSliceFileReference,
        detail: WorkflowSliceDetail,
        relationship: WorkflowStepRelationshipName,
    ) -> Self {
        Self {
            slice_file,
            detail,
            relationship,
        }
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

pub fn workflow_path(slug: &WorkflowSlug) -> ProjectPath {
    ProjectPath::try_new(format!(
        "model/browser/data/workflows/{}.eventmodel.json",
        slug.as_ref()
    ))
    .unwrap_or_else(|error| unreachable!("EMC generated workflow path must be valid: {error}"))
}

fn appended_slice_files(existing: Option<&Vec<Value>>, new_value: &str) -> Vec<Value> {
    let mut values = existing.cloned().unwrap_or_default();
    if !values.iter().any(|value| value.as_str() == Some(new_value)) {
        values.push(Value::String(new_value.to_owned()));
    }
    values
}

fn appended_steps(existing: Option<&Vec<Value>>, addition: WorkflowSliceAddition) -> Vec<Value> {
    let mut values = existing.cloned().unwrap_or_default();
    values.push(Value::Object(
        [
            (
                "slice".to_owned(),
                Value::String(addition.detail.slug().as_ref().to_owned()),
            ),
            (
                "name".to_owned(),
                Value::String(addition.detail.name().as_ref().to_owned()),
            ),
            (
                "type".to_owned(),
                Value::String(addition.detail.kind().as_ref().to_owned()),
            ),
            (
                "description".to_owned(),
                Value::String(addition.detail.description().as_ref().to_owned()),
            ),
            (
                "relationship".to_owned(),
                Value::String(addition.relationship.as_ref().to_owned()),
            ),
            ("transitions".to_owned(), Value::Array(Vec::new())),
        ]
        .into_iter()
        .collect(),
    ));
    values
}

fn connected_steps(
    existing: &[Value],
    addition: WorkflowTransitionAddition,
) -> Result<Vec<Value>, WorkflowDocumentError> {
    let mut found_source = false;
    let next_steps = existing
        .iter()
        .map(|step| {
            if step.get("slice").and_then(Value::as_str) != Some(addition.source.as_ref()) {
                return Ok(step.clone());
            }
            found_source = true;
            append_transition(step, &addition)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if found_source {
        Ok(next_steps)
    } else {
        Err(WorkflowDocumentError::new(format!(
            "unknown workflow step {}",
            addition.source.as_ref()
        )))
    }
}

fn reject_unknown_transition_target(
    existing: &[Value],
    addition: &WorkflowTransitionAddition,
) -> Result<(), WorkflowDocumentError> {
    match &addition.target {
        WorkflowTransitionTarget::Slice(target)
            if !existing.iter().any(|step| {
                step.get("slice")
                    .and_then(Value::as_str)
                    .is_some_and(|slice| slice == target.as_ref())
            }) =>
        {
            Err(WorkflowDocumentError::new(format!(
                "unknown workflow step {}",
                target.as_ref()
            )))
        }
        WorkflowTransitionTarget::Slice(_) | WorkflowTransitionTarget::Workflow { .. } => Ok(()),
    }
}

fn reject_duplicate_transition(
    existing: &[Value],
    addition: &WorkflowTransitionAddition,
) -> Result<(), WorkflowDocumentError> {
    let addition_label = transition_addition_label(addition)?;
    if workflow_transitions(existing)?
        .iter()
        .any(|existing_label| existing_label == &addition_label)
    {
        return Err(WorkflowDocumentError::new(format!(
            "workflow transition {} already exists",
            addition_label.as_ref()
        )));
    }
    Ok(())
}

fn append_transition(
    step: &Value,
    addition: &WorkflowTransitionAddition,
) -> Result<Value, WorkflowDocumentError> {
    let object = step
        .as_object()
        .ok_or_else(|| WorkflowDocumentError::new("workflow step must be a JSON object"))?;
    let mut next = object.clone();
    let mut transitions = object
        .get("transitions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    transitions.push(transition_value(addition));
    next.insert("transitions".to_owned(), Value::Array(transitions));
    Ok(Value::Object(next))
}

fn transition_addition_label(
    addition: &WorkflowTransitionAddition,
) -> Result<WorkflowTransitionLabel, WorkflowDocumentError> {
    let transition = transition_value(addition);
    let label = match &addition.target {
        WorkflowTransitionTarget::Slice(target) => {
            transition_label(addition.source.as_ref(), target.as_ref(), &transition)
        }
        WorkflowTransitionTarget::Workflow { slug, reason: _ } => {
            workflow_exit_transition_label(addition.source.as_ref(), slug.as_ref(), &transition)
        }
    }
    .ok_or_else(|| WorkflowDocumentError::new("workflow transition addition is missing trigger"))?;
    workflow_transition_label_value(&label)
}

fn transition_value(addition: &WorkflowTransitionAddition) -> Value {
    let mut transition = serde_json::Map::from_iter([(
        addition.trigger_field.as_ref().to_owned(),
        Value::String(addition.trigger.as_ref().to_owned()),
    )]);
    match &addition.target {
        WorkflowTransitionTarget::Slice(target) => {
            transition.insert("to".to_owned(), Value::String(target.as_ref().to_owned()));
        }
        WorkflowTransitionTarget::Workflow { slug, reason } => {
            transition.insert(
                "to_workflow".to_owned(),
                Value::String(slug.as_ref().to_owned()),
            );
            transition.insert(
                "exit_reason".to_owned(),
                Value::String(reason.as_ref().to_owned()),
            );
        }
    }
    Value::Object(transition)
}

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
    Ok(WorkflowSliceDetail::new(slug, name, kind, description))
}

fn workflow_transitions(
    steps: &[Value],
) -> Result<Vec<WorkflowTransitionLabel>, WorkflowDocumentError> {
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
                WorkflowDocumentError::new(format!("invalid workflow transition: {error}"))
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
        ("exit_reason", "reason"),
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
    ]
    .into_iter()
    .find_map(|(field, kind)| {
        transition
            .get(field)
            .and_then(Value::as_str)
            .map(|trigger| format!("{source}->{target}:workflow_exit:{kind}:{trigger}"))
    })
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

fn workflow_step_name(raw: &str) -> Result<WorkflowStepName, WorkflowDocumentError> {
    WorkflowStepName::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid workflow step name: {error}")))
}

fn workflow_branch_label(
    steps: &[Value],
    step: &Value,
    relationship: &str,
) -> Result<WorkflowBranchLabel, WorkflowDocumentError> {
    let label = step
        .get("slice")
        .and_then(Value::as_str)
        .filter(|slice| {
            relationship == "alternate" && has_incoming_outcome_transition(steps, slice)
        })
        .map_or_else(
            || relationship.replace('_', " "),
            |_| "alternate outcome".to_owned(),
        );
    WorkflowBranchLabel::try_new(label).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow branch label: {error}"))
    })
}

fn has_incoming_outcome_transition(steps: &[Value], target_slice: &str) -> bool {
    steps
        .iter()
        .flat_map(|step| {
            step.get("transitions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .any(|transition| {
            transition
                .get("to")
                .and_then(Value::as_str)
                .is_some_and(|target| target == target_slice)
                && transition
                    .get("via_outcome")
                    .and_then(Value::as_str)
                    .is_some()
        })
}

fn step_transition_details(
    steps: &[Value],
    step: &Value,
) -> Result<Vec<WorkflowTransitionDetail>, WorkflowDocumentError> {
    let source = step.get("name").and_then(Value::as_str);

    step.get("transitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|transition| {
            let (kind, label) = transition_kind_and_label(transition)?;
            Some((
                transition_display_name(transition, label),
                source?,
                transition_target_name(steps, transition)?,
                (kind, label),
            ))
        })
        .map(|(name, source, target, (kind, label))| {
            Ok(WorkflowTransitionDetail::new(
                workflow_transition_name(name)?,
                workflow_step_name(source)?,
                workflow_step_name(target)?,
                workflow_transition_kind(kind)?,
                workflow_transition_label_value(label)?,
            ))
        })
        .collect()
}

fn transition_target_name<'a>(steps: &'a [Value], transition: &'a Value) -> Option<&'a str> {
    transition
        .get("to")
        .and_then(Value::as_str)
        .map(|target_slice| {
            workflow_step_name_for_slice(steps, target_slice).unwrap_or(target_slice)
        })
        .or_else(|| transition.get("target_name").and_then(Value::as_str))
        .or_else(|| transition.get("to_workflow").and_then(Value::as_str))
}

fn workflow_step_name_for_slice<'a>(steps: &'a [Value], slice: &str) -> Option<&'a str> {
    steps
        .iter()
        .find(|step| {
            step.get("slice")
                .and_then(Value::as_str)
                .is_some_and(|step_slice| step_slice == slice)
        })
        .and_then(|step| step.get("name").and_then(Value::as_str))
}

fn transition_kind_and_label(transition: &Value) -> Option<(&'static str, &str)> {
    transition
        .get("retry")
        .and_then(Value::as_bool)
        .filter(|retry| *retry)
        .map(|_| ("retry", "retry"))
        .or_else(|| {
            transition
                .get("via_navigation")
                .and_then(Value::as_str)
                .map(|label| ("navigation", label))
        })
        .or_else(|| {
            transition
                .get("via_command")
                .and_then(Value::as_str)
                .map(|label| ("command", label))
        })
        .or_else(|| {
            transition
                .get("via_event")
                .and_then(Value::as_str)
                .map(|label| ("event", label))
        })
        .or_else(|| {
            transition
                .get("via_external_trigger")
                .and_then(Value::as_str)
                .map(|label| ("external trigger", label))
        })
        .or_else(|| {
            transition
                .get("via_outcome")
                .and_then(Value::as_str)
                .map(|label| ("workflow exit", label))
        })
}

fn transition_display_name<'a>(transition: &'a Value, label: &'a str) -> &'a str {
    transition
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(label)
}

fn workflow_transition_name(raw: &str) -> Result<WorkflowTransitionName, WorkflowDocumentError> {
    WorkflowTransitionName::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow transition name: {error}"))
    })
}

fn workflow_transition_kind(raw: &str) -> Result<WorkflowTransitionKind, WorkflowDocumentError> {
    WorkflowTransitionKind::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow transition kind: {error}"))
    })
}

fn workflow_transition_label_value(
    raw: &str,
) -> Result<WorkflowTransitionLabel, WorkflowDocumentError> {
    WorkflowTransitionLabel::try_new(raw.to_owned()).map_err(|error| {
        WorkflowDocumentError::new(format!("invalid workflow transition label: {error}"))
    })
}

fn review_status(raw: &str) -> Result<ReviewStatus, WorkflowDocumentError> {
    ReviewStatus::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid review status: {error}")))
}

fn review_rule_name(raw: &str) -> Result<ReviewRuleName, WorkflowDocumentError> {
    ReviewRuleName::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid review rule name: {error}")))
}

fn model_description(context: &str, raw: &str) -> Result<ModelDescription, WorkflowDocumentError> {
    ModelDescription::try_new(raw.to_owned())
        .map_err(|error| WorkflowDocumentError::new(format!("invalid {context}: {error}")))
}
