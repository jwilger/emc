use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::digest::artifact_digest;
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, SliceSlug, TransitionTriggerName,
    WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionEndpoint, WorkflowTransitionFieldName,
    WorkflowTransitionKind, WorkflowTransitionRecord, WorkflowTransitionRecords,
};
use crate::core::workflow_document::{
    WorkflowDocument, WorkflowTransitionAddition, WorkflowTransitionTarget, workflow_path,
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

    fn trigger_kind(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Event => "event",
            Self::Navigation => "navigation",
            Self::ExternalTrigger => "external_trigger",
            Self::Outcome => "outcome",
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkflowTransitionRemovalTarget {
    Slice(SliceSlug),
    Workflow(WorkflowSlug),
}

impl WorkflowTransitionRemovalTarget {
    fn as_ref(&self) -> &str {
        match self {
            Self::Slice(slug) => slug.as_ref(),
            Self::Workflow(slug) => slug.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionRemoval {
    workflow_slug: WorkflowSlug,
    source: SliceSlug,
    target: WorkflowTransitionRemovalTarget,
    kind: ConnectionKind,
    trigger: TransitionTriggerName,
}

impl WorkflowTransitionRemoval {
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
            target: WorkflowTransitionRemovalTarget::Slice(target),
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
    ) -> Self {
        Self {
            workflow_slug,
            source,
            target: WorkflowTransitionRemovalTarget::Workflow(target),
            kind,
            trigger,
        }
    }

    pub fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }
}

pub fn connect_workflow(
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_document: FileContents,
    connection: WorkflowConnection,
) -> Result<EffectPlan, ConnectionMutationError> {
    let workflow_document = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let workflow_name = workflow_document
        .name()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    if workflow_name != indexed_workflow_name {
        return Err(ConnectionMutationError::new(format!(
            "workflow document name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_document
        .description()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    if workflow_description != indexed_workflow_description {
        return Err(ConnectionMutationError::new(format!(
            "workflow document description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }
    let workflow_document = workflow_document
        .with_connected_transition(WorkflowTransitionAddition::new(
            connection.source.clone(),
            workflow_transition_target(&connection.target),
            workflow_transition_field(connection.kind),
            connection.trigger.clone(),
        ))
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let module_name = module_name(workflow_name.as_ref());
    let workflow_slice_details = workflow_document
        .slice_details()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let workflow_transitions = workflow_document
        .transitions()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let digest = artifact_digest(
        workflow_name.clone(),
        connection.workflow_slug.clone(),
        workflow_description.clone(),
        WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
        WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
    );
    let workflow_json = workflow_document
        .contents()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let source = connection.source.as_ref();
    let target = connection.target.as_ref();

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(workflow_path(&connection.workflow_slug), workflow_json),
        Effect::WriteFile(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_workflow_module(
                lean_module_name(module_name.clone()),
                workflow_name.clone(),
                workflow_description.clone(),
                connection.workflow_slug.clone(),
                WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
                WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
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
                WorkflowSliceDetails::from_details(workflow_slice_details),
                WorkflowTransitionRecords::from_records(workflow_transitions),
                digest,
            ),
        ),
        Effect::Report(report_line(format!("connected {source} to {target}"))),
    ]))
}

pub fn remove_transition(
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_document: FileContents,
    removal: WorkflowTransitionRemoval,
) -> Result<EffectPlan, ConnectionMutationError> {
    let workflow_document = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let workflow_name = workflow_document
        .name()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    if workflow_name != indexed_workflow_name {
        return Err(ConnectionMutationError::new(format!(
            "workflow document name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_document
        .description()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    if workflow_description != indexed_workflow_description {
        return Err(ConnectionMutationError::new(format!(
            "workflow document description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }
    let workflow_document = workflow_document
        .with_removed_transition(removal_transition_record(&removal)?)
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let module_name = module_name(workflow_name.as_ref());
    let workflow_slice_details = workflow_document
        .slice_details()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let workflow_transitions = workflow_document
        .transitions()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let digest = artifact_digest(
        workflow_name.clone(),
        removal.workflow_slug.clone(),
        workflow_description.clone(),
        WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
        WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
    );
    let workflow_json = workflow_document
        .contents()
        .map_err(|error| ConnectionMutationError::new(error.to_string()))?;
    let source = removal.source.as_ref();
    let target = removal.target.as_ref();

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(workflow_path(&removal.workflow_slug), workflow_json),
        Effect::WriteFile(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_workflow_module(
                lean_module_name(module_name.clone()),
                workflow_name.clone(),
                workflow_description.clone(),
                removal.workflow_slug.clone(),
                WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
                WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
                digest.clone(),
            ),
        ),
        Effect::WriteFile(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_workflow_module(
                quint_module_name(module_name),
                workflow_name,
                workflow_description,
                removal.workflow_slug.clone(),
                WorkflowSliceDetails::from_details(workflow_slice_details),
                WorkflowTransitionRecords::from_records(workflow_transitions),
                digest,
            ),
        ),
        Effect::Report(report_line(format!(
            "removed transition {source} to {target}"
        ))),
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

fn workflow_transition_target(target: &WorkflowConnectionTarget) -> WorkflowTransitionTarget {
    match target {
        WorkflowConnectionTarget::Slice(slug) => WorkflowTransitionTarget::Slice(slug.clone()),
        WorkflowConnectionTarget::Workflow { slug, reason } => WorkflowTransitionTarget::Workflow {
            slug: slug.clone(),
            reason: reason.clone(),
        },
    }
}

fn workflow_transition_field(kind: ConnectionKind) -> WorkflowTransitionFieldName {
    WorkflowTransitionFieldName::try_new(kind.trigger_field().to_owned()).unwrap_or_else(|error| {
        unreachable!("EMC generated workflow transition field must be valid: {error}");
    })
}

fn removal_transition_record(
    removal: &WorkflowTransitionRemoval,
) -> Result<WorkflowTransitionRecord, ConnectionMutationError> {
    let kind = match removal.target {
        WorkflowTransitionRemovalTarget::Slice(_) => removal.kind.trigger_kind().to_owned(),
        WorkflowTransitionRemovalTarget::Workflow(_) => {
            format!("workflow_exit:{}", removal.kind.trigger_kind())
        }
    };
    Ok(WorkflowTransitionRecord::new(
        workflow_transition_endpoint("source", removal.source.as_ref())?,
        workflow_transition_endpoint("target", removal.target.as_ref())?,
        workflow_transition_kind(&kind)?,
        removal.trigger.clone(),
    ))
}

fn workflow_transition_endpoint(
    context: &str,
    raw: &str,
) -> Result<WorkflowTransitionEndpoint, ConnectionMutationError> {
    WorkflowTransitionEndpoint::try_new(raw.to_owned()).map_err(|error| {
        ConnectionMutationError::new(format!("invalid workflow transition {context}: {error}"))
    })
}

fn workflow_transition_kind(raw: &str) -> Result<WorkflowTransitionKind, ConnectionMutationError> {
    WorkflowTransitionKind::try_new(raw.to_owned()).map_err(|error| {
        ConnectionMutationError::new(format!("invalid workflow transition kind: {error}"))
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
