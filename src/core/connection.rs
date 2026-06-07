// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest};
use crate::core::effect::{Effect, EffectPlan, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::formal_graph::FormalWorkflowGraph;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, PayloadContractName, QuintModuleName, SliceSlug,
    TransitionTriggerName, WorkflowCommandErrorRecords, WorkflowModuleData, WorkflowOutcomeRecords,
    WorkflowOwnedDefinitionRecords, WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug,
    WorkflowTransitionEndpoint, WorkflowTransitionKind, WorkflowTransitionRecord,
    WorkflowTransitionRecords,
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

    pub fn try_new(value: String) -> Result<Self, ConnectionKindError> {
        match value.trim() {
            "command" => Ok(Self::Command),
            "event" => Ok(Self::Event),
            "navigation" => Ok(Self::Navigation),
            "external_trigger" => Ok(Self::ExternalTrigger),
            "outcome" => Ok(Self::Outcome),
            _ => Err(ConnectionKindError::new(value)),
        }
    }

    pub(crate) fn trigger_kind(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Event => "event",
            Self::Navigation => "navigation",
            Self::ExternalTrigger => "external_trigger",
            Self::Outcome => "outcome",
        }
    }
}

impl AsRef<str> for ConnectionKind {
    fn as_ref(&self) -> &str {
        self.trigger_kind()
    }
}

impl Display for ConnectionKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for ConnectionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for ConnectionKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConnectionKindError {
    message: String,
}

impl ConnectionKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled workflow connection kind, got '{value}'"),
        }
    }
}

impl Display for ConnectionKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ConnectionKindError {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkflowConnectionTarget {
    Slice(SliceSlug),
    Workflow {
        slug: WorkflowSlug,
        reason: ModelDescription,
    },
}

impl WorkflowConnectionTarget {
    pub(crate) fn as_ref(&self) -> &str {
        match self {
            Self::Slice(slug) => slug.as_ref(),
            Self::Workflow { slug, reason: _ } => slug.as_ref(),
        }
    }

    pub(crate) fn slice_slug(&self) -> Option<&SliceSlug> {
        match self {
            Self::Slice(slug) => Some(slug),
            Self::Workflow { slug: _, reason: _ } => None,
        }
    }

    pub(crate) fn workflow_slug(&self) -> Option<&WorkflowSlug> {
        match self {
            Self::Slice(_) => None,
            Self::Workflow { slug, reason: _ } => Some(slug),
        }
    }

    pub(crate) fn reason(&self) -> Option<&ModelDescription> {
        match self {
            Self::Slice(_) => None,
            Self::Workflow { slug: _, reason } => Some(reason),
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
    payload_contract: Option<PayloadContractName>,
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
            payload_contract: None,
        }
    }

    pub fn new_with_payload_contract(
        workflow_slug: WorkflowSlug,
        source: SliceSlug,
        target: SliceSlug,
        kind: ConnectionKind,
        trigger: TransitionTriggerName,
        payload_contract: PayloadContractName,
    ) -> Self {
        Self {
            workflow_slug,
            source,
            target: WorkflowConnectionTarget::Slice(target),
            kind,
            trigger,
            payload_contract: Some(payload_contract),
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
            payload_contract: None,
        }
    }

    pub fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub fn source(&self) -> &SliceSlug {
        &self.source
    }

    pub fn target(&self) -> &WorkflowConnectionTarget {
        &self.target
    }

    pub fn kind(&self) -> ConnectionKind {
        self.kind
    }

    pub fn trigger(&self) -> &TransitionTriggerName {
        &self.trigger
    }

    pub fn payload_contract(&self) -> Option<&PayloadContractName> {
        self.payload_contract.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkflowTransitionRemovalTarget {
    Slice(SliceSlug),
    Workflow(WorkflowSlug),
}

impl WorkflowTransitionRemovalTarget {
    pub(crate) fn as_ref(&self) -> &str {
        match self {
            Self::Slice(slug) => slug.as_ref(),
            Self::Workflow(slug) => slug.as_ref(),
        }
    }

    pub(crate) fn slice_slug(&self) -> Option<&SliceSlug> {
        match self {
            Self::Slice(slug) => Some(slug),
            Self::Workflow(_) => None,
        }
    }

    pub(crate) fn workflow_slug(&self) -> Option<&WorkflowSlug> {
        match self {
            Self::Slice(_) => None,
            Self::Workflow(slug) => Some(slug),
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

    pub fn source(&self) -> &SliceSlug {
        &self.source
    }

    pub fn target(&self) -> &WorkflowTransitionRemovalTarget {
        &self.target
    }

    pub fn kind(&self) -> ConnectionKind {
        self.kind
    }

    pub fn trigger(&self) -> &TransitionTriggerName {
        &self.trigger
    }
}

pub(crate) fn connect_workflow(
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_graph: FormalWorkflowGraph,
    connection: WorkflowConnection,
) -> Result<EffectPlan, ConnectionMutationError> {
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != indexed_workflow_name {
        return Err(ConnectionMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != indexed_workflow_description {
        return Err(ConnectionMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }
    let module_name = module_name(workflow_name.as_ref());
    let workflow_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    let workflow_outcomes = workflow_graph.outcomes().as_slice().to_owned();
    let workflow_command_errors = workflow_graph.command_errors().as_slice().to_owned();
    let workflow_owned_definitions = workflow_graph.owned_definitions().as_slice().to_owned();
    let workflow_transition_evidences = workflow_graph.transition_evidences().clone();
    let workflow_entry_lifecycle_required = workflow_graph.entry_lifecycle_required();
    let workflow_entry_lifecycle_states = workflow_graph.entry_lifecycle_states().clone();
    reject_unknown_transition_source(&workflow_slice_details, &connection.source)?;
    reject_unknown_transition_target(&workflow_slice_details, &connection.target)?;
    let mut workflow_transitions = workflow_graph.transitions().as_slice().to_owned();
    let transition = connection_transition_record(&connection)?;
    reject_duplicate_transition(&workflow_transitions, &transition)?;
    workflow_transitions.push(transition);
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow_name.clone(),
        workflow_slug: connection.workflow_slug.clone(),
        workflow_description: workflow_description.clone(),
        workflow_slice_details: WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
        workflow_transitions: WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
        workflow_outcomes: WorkflowOutcomeRecords::from_records(workflow_outcomes.clone()),
        workflow_command_errors: WorkflowCommandErrorRecords::from_records(
            workflow_command_errors.clone(),
        ),
        workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records(
            workflow_owned_definitions.clone(),
        ),
        workflow_transition_evidences: workflow_transition_evidences.clone(),
        workflow_requires_entry_lifecycle_coverage: workflow_entry_lifecycle_required,
        workflow_entry_lifecycle_states: workflow_entry_lifecycle_states.clone(),
    });
    let source = connection.source.as_ref();
    let target = connection.target.as_ref();
    let effects = vec![
        Effect::write_file(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_workflow_module(
                lean_module_name(module_name.clone()),
                WorkflowModuleData::new(
                    workflow_name.clone(),
                    workflow_description.clone(),
                    connection.workflow_slug.clone(),
                    digest.clone(),
                )
                .with_slice_details(WorkflowSliceDetails::from_details(
                    workflow_slice_details.clone(),
                ))
                .with_transitions(WorkflowTransitionRecords::from_records(
                    workflow_transitions.clone(),
                ))
                .with_outcomes(WorkflowOutcomeRecords::from_records(
                    workflow_outcomes.clone(),
                ))
                .with_command_errors(WorkflowCommandErrorRecords::from_records(
                    workflow_command_errors.clone(),
                ))
                .with_owned_definitions(WorkflowOwnedDefinitionRecords::from_records(
                    workflow_owned_definitions.clone(),
                ))
                .with_transition_evidences(workflow_transition_evidences.clone())
                .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                .with_entry_lifecycle_states(workflow_entry_lifecycle_states.clone()),
            ),
        ),
        Effect::write_file(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_workflow_module(
                quint_module_name(module_name),
                WorkflowModuleData::new(
                    workflow_name,
                    workflow_description,
                    connection.workflow_slug.clone(),
                    digest,
                )
                .with_slice_details(WorkflowSliceDetails::from_details(workflow_slice_details))
                .with_transitions(WorkflowTransitionRecords::from_records(
                    workflow_transitions,
                ))
                .with_outcomes(WorkflowOutcomeRecords::from_records(workflow_outcomes))
                .with_command_errors(WorkflowCommandErrorRecords::from_records(
                    workflow_command_errors,
                ))
                .with_owned_definitions(WorkflowOwnedDefinitionRecords::from_records(
                    workflow_owned_definitions,
                ))
                .with_transition_evidences(workflow_transition_evidences)
                .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                .with_entry_lifecycle_states(workflow_entry_lifecycle_states),
            ),
        ),
        Effect::Report(report_line(format!("connected {source} to {target}"))),
    ];

    Ok(EffectPlan::new(effects))
}

pub(crate) fn remove_transition(
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_graph: FormalWorkflowGraph,
    removal: WorkflowTransitionRemoval,
) -> Result<EffectPlan, ConnectionMutationError> {
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != indexed_workflow_name {
        return Err(ConnectionMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != indexed_workflow_description {
        return Err(ConnectionMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }
    let module_name = module_name(workflow_name.as_ref());
    let workflow_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    let workflow_outcomes = workflow_graph.outcomes().as_slice().to_owned();
    let workflow_command_errors = workflow_graph.command_errors().as_slice().to_owned();
    let workflow_owned_definitions = workflow_graph.owned_definitions().as_slice().to_owned();
    let workflow_transition_evidences = workflow_graph.transition_evidences().clone();
    let workflow_entry_lifecycle_required = workflow_graph.entry_lifecycle_required();
    let workflow_entry_lifecycle_states = workflow_graph.entry_lifecycle_states().clone();
    let removal_record = removal_transition_record(&removal)?;
    let mut removed_transition = false;
    let workflow_transitions = workflow_graph
        .transitions()
        .as_slice()
        .iter()
        .filter_map(|transition| {
            if same_transition_identity(transition, &removal_record) {
                removed_transition = true;
                None
            } else {
                Some(transition.clone())
            }
        })
        .collect::<Vec<_>>();
    if !removed_transition {
        return Err(ConnectionMutationError::new(format!(
            "workflow transition {} does not exist",
            transition_record_label(&removal_record)
        )));
    }
    reject_main_slices_without_incoming_transitions(
        &workflow_slice_details,
        &workflow_transitions,
    )?;
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow_name.clone(),
        workflow_slug: removal.workflow_slug.clone(),
        workflow_description: workflow_description.clone(),
        workflow_slice_details: WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
        workflow_transitions: WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
        workflow_outcomes: WorkflowOutcomeRecords::from_records(workflow_outcomes.clone()),
        workflow_command_errors: WorkflowCommandErrorRecords::from_records(
            workflow_command_errors.clone(),
        ),
        workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records(
            workflow_owned_definitions.clone(),
        ),
        workflow_transition_evidences: workflow_transition_evidences.clone(),
        workflow_requires_entry_lifecycle_coverage: workflow_entry_lifecycle_required,
        workflow_entry_lifecycle_states: workflow_entry_lifecycle_states.clone(),
    });
    let source = removal.source.as_ref();
    let target = removal.target.as_ref();

    Ok(EffectPlan::new(vec![
        Effect::write_file(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_workflow_module(
                lean_module_name(module_name.clone()),
                WorkflowModuleData::new(
                    workflow_name.clone(),
                    workflow_description.clone(),
                    removal.workflow_slug.clone(),
                    digest.clone(),
                )
                .with_slice_details(WorkflowSliceDetails::from_details(
                    workflow_slice_details.clone(),
                ))
                .with_transitions(WorkflowTransitionRecords::from_records(
                    workflow_transitions.clone(),
                ))
                .with_outcomes(WorkflowOutcomeRecords::from_records(
                    workflow_outcomes.clone(),
                ))
                .with_command_errors(WorkflowCommandErrorRecords::from_records(
                    workflow_command_errors.clone(),
                ))
                .with_owned_definitions(WorkflowOwnedDefinitionRecords::from_records(
                    workflow_owned_definitions.clone(),
                ))
                .with_transition_evidences(workflow_transition_evidences.clone())
                .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                .with_entry_lifecycle_states(workflow_entry_lifecycle_states.clone()),
            ),
        ),
        Effect::write_file(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_workflow_module(
                quint_module_name(module_name),
                WorkflowModuleData::new(
                    workflow_name,
                    workflow_description,
                    removal.workflow_slug.clone(),
                    digest,
                )
                .with_slice_details(WorkflowSliceDetails::from_details(workflow_slice_details))
                .with_transitions(WorkflowTransitionRecords::from_records(
                    workflow_transitions,
                ))
                .with_outcomes(WorkflowOutcomeRecords::from_records(workflow_outcomes))
                .with_command_errors(WorkflowCommandErrorRecords::from_records(
                    workflow_command_errors,
                ))
                .with_owned_definitions(WorkflowOwnedDefinitionRecords::from_records(
                    workflow_owned_definitions,
                ))
                .with_transition_evidences(workflow_transition_evidences)
                .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                .with_entry_lifecycle_states(workflow_entry_lifecycle_states),
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

fn removal_transition_record(
    removal: &WorkflowTransitionRemoval,
) -> Result<WorkflowTransitionRecord, ConnectionMutationError> {
    let kind = match removal.target {
        WorkflowTransitionRemovalTarget::Slice(_) => transition_kind(removal.kind),
        WorkflowTransitionRemovalTarget::Workflow(_) => workflow_exit_transition_kind(removal.kind),
    };
    Ok(WorkflowTransitionRecord::new(
        workflow_transition_endpoint("source", removal.source.as_ref())?,
        workflow_transition_endpoint("target", removal.target.as_ref())?,
        kind,
        removal.trigger.clone(),
    ))
}

fn connection_transition_record(
    connection: &WorkflowConnection,
) -> Result<WorkflowTransitionRecord, ConnectionMutationError> {
    let source = workflow_transition_endpoint("source", connection.source.as_ref())?;
    let target = workflow_transition_endpoint("target", connection.target.as_ref())?;
    let kind = match connection.target {
        WorkflowConnectionTarget::Slice(_) => transition_kind(connection.kind),
        WorkflowConnectionTarget::Workflow { .. } => workflow_exit_transition_kind(connection.kind),
    };
    match &connection.target {
        WorkflowConnectionTarget::Slice(_) => {
            if let Some(payload_contract) = &connection.payload_contract {
                Ok(WorkflowTransitionRecord::new_with_payload_contract(
                    source,
                    target,
                    kind,
                    connection.trigger.clone(),
                    payload_contract.clone(),
                ))
            } else {
                Ok(WorkflowTransitionRecord::new(
                    source,
                    target,
                    kind,
                    connection.trigger.clone(),
                ))
            }
        }
        WorkflowConnectionTarget::Workflow { slug: _, reason } => {
            Ok(WorkflowTransitionRecord::new_with_rationale(
                source,
                target,
                kind,
                connection.trigger.clone(),
                reason.clone(),
            ))
        }
    }
}

fn reject_unknown_transition_source(
    slices: &[WorkflowSliceDetail],
    source: &SliceSlug,
) -> Result<(), ConnectionMutationError> {
    if slices.iter().any(|slice| slice.slug() == source) {
        Ok(())
    } else {
        Err(ConnectionMutationError::new(format!(
            "unknown workflow step {}",
            source.as_ref()
        )))
    }
}

fn reject_unknown_transition_target(
    slices: &[WorkflowSliceDetail],
    target: &WorkflowConnectionTarget,
) -> Result<(), ConnectionMutationError> {
    match target {
        WorkflowConnectionTarget::Slice(target)
            if !slices.iter().any(|slice| slice.slug() == target) =>
        {
            Err(ConnectionMutationError::new(format!(
                "unknown workflow step {}",
                target.as_ref()
            )))
        }
        WorkflowConnectionTarget::Slice(_) | WorkflowConnectionTarget::Workflow { .. } => Ok(()),
    }
}

fn reject_duplicate_transition(
    transitions: &[WorkflowTransitionRecord],
    addition: &WorkflowTransitionRecord,
) -> Result<(), ConnectionMutationError> {
    if transitions
        .iter()
        .any(|existing| same_transition_identity(existing, addition))
    {
        return Err(ConnectionMutationError::new(format!(
            "workflow transition {} already exists",
            transition_record_label(addition)
        )));
    }
    Ok(())
}

fn reject_main_slices_without_incoming_transitions(
    slices: &[WorkflowSliceDetail],
    transitions: &[WorkflowTransitionRecord],
) -> Result<(), ConnectionMutationError> {
    slices
        .iter()
        .skip(1)
        .find(|slice| {
            !transitions
                .iter()
                .any(|transition| transition.target().as_ref() == slice.slug().as_ref())
        })
        .map_or(Ok(()), |slice| {
            Err(ConnectionMutationError::new(format!(
                "removing transition would leave workflow step '{}' without an incoming transition",
                slice.slug().as_ref()
            )))
        })
}

fn same_transition_identity(
    left: &WorkflowTransitionRecord,
    right: &WorkflowTransitionRecord,
) -> bool {
    left.source() == right.source()
        && left.target() == right.target()
        && left.kind() == right.kind()
        && left.trigger() == right.trigger()
}

fn transition_record_label(transition: &WorkflowTransitionRecord) -> String {
    format!(
        "{}->{}:{}:{}",
        transition.source().as_ref(),
        transition.target().as_ref(),
        transition.kind().as_ref(),
        transition.trigger().as_ref()
    )
}

fn workflow_transition_endpoint(
    context: &str,
    raw: &str,
) -> Result<WorkflowTransitionEndpoint, ConnectionMutationError> {
    WorkflowTransitionEndpoint::try_new(raw.to_owned()).map_err(|error| {
        ConnectionMutationError::new(format!("invalid workflow transition {context}: {error}"))
    })
}

fn transition_kind(kind: ConnectionKind) -> WorkflowTransitionKind {
    match kind {
        ConnectionKind::Command => WorkflowTransitionKind::Command,
        ConnectionKind::Event => WorkflowTransitionKind::Event,
        ConnectionKind::Navigation => WorkflowTransitionKind::Navigation,
        ConnectionKind::ExternalTrigger => WorkflowTransitionKind::ExternalTrigger,
        ConnectionKind::Outcome => WorkflowTransitionKind::Outcome,
    }
}

fn workflow_exit_transition_kind(kind: ConnectionKind) -> WorkflowTransitionKind {
    match kind {
        ConnectionKind::Command => WorkflowTransitionKind::WorkflowExitCommand,
        ConnectionKind::Event => WorkflowTransitionKind::WorkflowExitEvent,
        ConnectionKind::Navigation => WorkflowTransitionKind::WorkflowExitNavigation,
        ConnectionKind::ExternalTrigger => WorkflowTransitionKind::WorkflowExitExternalTrigger,
        ConnectionKind::Outcome => WorkflowTransitionKind::WorkflowExitOutcome,
    }
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
