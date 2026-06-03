use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::{Value, json};

use crate::core::digest::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, SliceKindName, SliceSlug,
    TransitionTriggerName, WorkflowSliceDetail, WorkflowSlug, WorkflowTransitionLabel,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConnectionKind {
    Command,
    Event,
    Navigation,
    ExternalTrigger,
    Outcome,
}

impl ConnectionKind {
    pub fn command() -> Self {
        Self::Command
    }

    pub fn event() -> Self {
        Self::Event
    }

    pub fn navigation() -> Self {
        Self::Navigation
    }

    pub fn external_trigger() -> Self {
        Self::ExternalTrigger
    }

    pub fn outcome() -> Self {
        Self::Outcome
    }

    fn trigger_field(self) -> &'static str {
        match self {
            Self::Command => "via_command",
            Self::Event => "via_event",
            Self::Navigation => "via_navigation",
            Self::ExternalTrigger => "via_external_trigger",
            Self::Outcome => "via_outcome",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkflowConnectionTarget {
    Slice(SliceSlug),
    Workflow {
        slug: WorkflowSlug,
        reason: ModelDescription,
    },
}

impl WorkflowConnectionTarget {
    fn as_ref(&self) -> &str {
        match self {
            Self::Slice(slug) => slug.as_ref(),
            Self::Workflow { slug, reason: _ } => slug.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowConnection {
    workflow_slug: WorkflowSlug,
    source: SliceSlug,
    target: WorkflowConnectionTarget,
    kind: ConnectionKind,
    trigger: TransitionTriggerName,
}

impl WorkflowConnection {
    pub fn new(
        workflow_slug: WorkflowSlug,
        source: SliceSlug,
        target: SliceSlug,
        kind: ConnectionKind,
        trigger: TransitionTriggerName,
    ) -> Self {
        Self {
            workflow_slug,
            source,
            target: WorkflowConnectionTarget::Slice(target),
            kind,
            trigger,
        }
    }

    pub fn new_workflow_exit(
        workflow_slug: WorkflowSlug,
        source: SliceSlug,
        target: WorkflowSlug,
        kind: ConnectionKind,
        trigger: TransitionTriggerName,
        reason: ModelDescription,
    ) -> Self {
        Self {
            workflow_slug,
            source,
            target: WorkflowConnectionTarget::Workflow {
                slug: target,
                reason,
            },
            kind,
            trigger,
        }
    }

    pub fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }
}

pub fn connect_workflow(
    workflow_document: FileContents,
    connection: WorkflowConnection,
) -> Result<EffectPlan, ConnectionMutationError> {
    let workflow_value = serde_json::from_str::<Value>(workflow_document.as_ref())
        .map_err(|error| ConnectionMutationError::new(format!("invalid workflow JSON: {error}")))?;
    let workflow_object = workflow_value
        .as_object()
        .ok_or_else(|| ConnectionMutationError::new("workflow document must be a JSON object"))?;
    let workflow_name = workflow_name(workflow_object)?;
    let workflow_description = workflow_description(workflow_object)?;
    let module_name = module_name(workflow_name.as_ref());
    let steps = connected_steps(workflow_object, &connection)?;
    let workflow_slice_details = workflow_slice_details(&steps)?;
    let workflow_transitions = workflow_transitions(&steps)?;
    let digest = artifact_digest(
        workflow_name.clone(),
        connection.workflow_slug.clone(),
        workflow_description.clone(),
        workflow_slice_details.clone(),
        workflow_transitions.clone(),
    );
    let workflow_json = workflow_json(workflow_object, steps)?;
    let source = connection.source.as_ref();
    let target = connection.target.as_ref();

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(
            project_path(format!(
                "model/browser/data/workflows/{}.eventmodel.json",
                connection.workflow_slug.as_ref()
            )),
            file_contents(workflow_json),
        ),
        Effect::WriteFile(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_workflow_module(
                lean_module_name(module_name.clone()),
                workflow_name.clone(),
                workflow_description.clone(),
                connection.workflow_slug.clone(),
                workflow_slice_details.clone(),
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
                connection.workflow_slug.clone(),
                workflow_slice_details,
                workflow_transitions,
                digest,
            ),
        ),
        Effect::Report(report_line(format!("connected {source} to {target}"))),
    ]))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConnectionMutationError {
    message: String,
}

impl ConnectionMutationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ConnectionMutationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ConnectionMutationError {}

fn connected_steps(
    workflow_object: &serde_json::Map<String, Value>,
    connection: &WorkflowConnection,
) -> Result<Vec<Value>, ConnectionMutationError> {
    let steps = workflow_object
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| ConnectionMutationError::new("workflow document is missing steps"))?;
    let mut found_source = false;
    let next_steps = steps
        .iter()
        .map(|step| {
            if step.get("slice").and_then(Value::as_str) != Some(connection.source.as_ref()) {
                return Ok(step.clone());
            }
            found_source = true;
            append_transition(step, connection)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if found_source {
        Ok(next_steps)
    } else {
        Err(ConnectionMutationError::new(format!(
            "unknown workflow step {}",
            connection.source.as_ref()
        )))
    }
}

fn append_transition(
    step: &Value,
    connection: &WorkflowConnection,
) -> Result<Value, ConnectionMutationError> {
    let object = step
        .as_object()
        .ok_or_else(|| ConnectionMutationError::new("workflow step must be a JSON object"))?;
    let mut next = object.clone();
    let mut transitions = object
        .get("transitions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    transitions.push(json!({
        connection.kind.trigger_field(): connection.trigger.as_ref()
    }));
    let transition = transitions
        .last_mut()
        .and_then(Value::as_object_mut)
        .ok_or_else(|| ConnectionMutationError::new("workflow transition must be a JSON object"))?;
    match &connection.target {
        WorkflowConnectionTarget::Slice(target) => {
            transition.insert("to".to_owned(), json!(target.as_ref()));
        }
        WorkflowConnectionTarget::Workflow { slug, reason } => {
            transition.insert("to_workflow".to_owned(), json!(slug.as_ref()));
            transition.insert("exit_reason".to_owned(), json!(reason.as_ref()));
        }
    }
    next.insert("transitions".to_owned(), Value::Array(transitions));
    Ok(Value::Object(next))
}

fn workflow_json(
    workflow_object: &serde_json::Map<String, Value>,
    steps: Vec<Value>,
) -> Result<String, ConnectionMutationError> {
    let mut next = workflow_object.clone();
    next.insert("steps".to_owned(), Value::Array(steps));
    serde_json::to_string_pretty(&Value::Object(next))
        .map(|json| format!("{json}\n"))
        .map_err(|error| ConnectionMutationError::new(format!("invalid workflow JSON: {error}")))
}

fn workflow_name(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<ModelName, ConnectionMutationError> {
    workflow_object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ConnectionMutationError::new("workflow document is missing name"))
        .and_then(|name| {
            ModelName::try_new(name.to_owned()).map_err(|error| {
                ConnectionMutationError::new(format!("invalid workflow name: {error}"))
            })
        })
}

fn workflow_description(
    workflow_object: &serde_json::Map<String, Value>,
) -> Result<ModelDescription, ConnectionMutationError> {
    workflow_object
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ConnectionMutationError::new("workflow document is missing description"))
        .and_then(|description| {
            ModelDescription::try_new(description.to_owned()).map_err(|error| {
                ConnectionMutationError::new(format!("invalid workflow description: {error}"))
            })
        })
}

fn workflow_slice_details(
    steps: &[Value],
) -> Result<Vec<WorkflowSliceDetail>, ConnectionMutationError> {
    steps.iter().map(workflow_slice_detail).collect()
}

fn workflow_slice_detail(step: &Value) -> Result<WorkflowSliceDetail, ConnectionMutationError> {
    let slug = step
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ConnectionMutationError::new("workflow step is missing slice"))
        .and_then(|slice| {
            SliceSlug::try_new(slice.to_owned()).map_err(|error| {
                ConnectionMutationError::new(format!("invalid slice slug: {error}"))
            })
        })?;
    let name = step
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ConnectionMutationError::new("workflow step is missing name"))
        .and_then(|name| {
            ModelName::try_new(name.to_owned()).map_err(|error| {
                ConnectionMutationError::new(format!("invalid slice name: {error}"))
            })
        })?;
    let kind = step
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| ConnectionMutationError::new("workflow step is missing type"))
        .and_then(|kind| {
            SliceKindName::try_new(kind.to_owned()).map_err(|error| {
                ConnectionMutationError::new(format!("invalid slice kind: {error}"))
            })
        })?;
    let description = step
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ConnectionMutationError::new("workflow step is missing description"))
        .and_then(|description| {
            ModelDescription::try_new(description.to_owned()).map_err(|error| {
                ConnectionMutationError::new(format!("invalid slice description: {error}"))
            })
        })?;
    Ok(WorkflowSliceDetail::new(slug, name, kind, description))
}

fn workflow_transitions(
    steps: &[Value],
) -> Result<Vec<WorkflowTransitionLabel>, ConnectionMutationError> {
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
                ConnectionMutationError::new(format!("invalid workflow transition: {error}"))
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
