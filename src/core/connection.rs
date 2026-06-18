// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest};
use crate::core::effect::{ArtifactDigest, Effect, EffectPlan, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::formal_graph::FormalWorkflowGraph;
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, PayloadContractName, QuintModuleName, SliceSlug,
    TransitionTriggerName, WorkflowCommandErrorRecords, WorkflowEntryLifecycleStateRecords,
    WorkflowModuleData, WorkflowOutcomeRecords, WorkflowOwnedDefinitionName,
    WorkflowOwnedDefinitionRecords, WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug,
    WorkflowTransitionEndpoint, WorkflowTransitionEvidenceRecords, WorkflowTransitionKind,
    WorkflowTransitionRecord, WorkflowTransitionRecords,
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

    pub fn try_new(value: &str) -> Result<Self, ConnectionKindError> {
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
        Self::try_new(&value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConnectionKindError {
    message: String,
}

impl ConnectionKindError {
    fn new(value: &str) -> Self {
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
    source_control: Option<TransitionTriggerName>,
    target_view: Option<WorkflowOwnedDefinitionName>,
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
            source_control: None,
            target_view: None,
            payload_contract: None,
        }
    }

    pub fn new_with_navigation_endpoints(
        workflow_slug: WorkflowSlug,
        source: SliceSlug,
        target: SliceSlug,
        kind: ConnectionKind,
        trigger: TransitionTriggerName,
        source_control: TransitionTriggerName,
        target_view: WorkflowOwnedDefinitionName,
    ) -> Self {
        Self {
            workflow_slug,
            source,
            target: WorkflowConnectionTarget::Slice(target),
            kind,
            trigger,
            source_control: Some(source_control),
            target_view: Some(target_view),
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
            source_control: None,
            target_view: None,
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
            source_control: None,
            target_view: None,
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

    pub fn source_control(&self) -> Option<&TransitionTriggerName> {
        self.source_control.as_ref()
    }

    pub fn target_view(&self) -> Option<&WorkflowOwnedDefinitionName> {
        self.target_view.as_ref()
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
    indexed_workflow_name: &ModelName,
    indexed_workflow_description: &ModelDescription,
    workflow_graph: &FormalWorkflowGraph,
    connection: &WorkflowConnection,
) -> Result<EffectPlan, ConnectionMutationError> {
    validate_workflow_identity(
        indexed_workflow_name,
        indexed_workflow_description,
        workflow_graph,
    )?;
    let mut artifact = WorkflowArtifactInputs::collect(workflow_graph);
    reject_unknown_transition_source(artifact.slice_details_slice(), &connection.source)?;
    reject_unknown_transition_target(artifact.slice_details_slice(), &connection.target)?;
    let transition = connection_transition_record(connection)?;
    reject_duplicate_transition(artifact.transitions_slice(), &transition)?;
    artifact.push_transition(transition);
    let digest = artifact.digest(&connection.workflow_slug);
    let source = connection.source.as_ref();
    let target = connection.target.as_ref();
    Ok(EffectPlan::new(artifact.module_effects(
        &connection.workflow_slug,
        &digest,
        report_line(format!("connected {source} to {target}")),
    )))
}

pub(crate) fn remove_transition(
    indexed_workflow_name: &ModelName,
    indexed_workflow_description: &ModelDescription,
    workflow_graph: &FormalWorkflowGraph,
    removal: &WorkflowTransitionRemoval,
) -> Result<EffectPlan, ConnectionMutationError> {
    validate_workflow_identity(
        indexed_workflow_name,
        indexed_workflow_description,
        workflow_graph,
    )?;
    let mut artifact = WorkflowArtifactInputs::collect(workflow_graph);
    let removal_record = removal_transition_record(removal)?;
    let remaining = retain_transitions_excluding(artifact.transitions_slice(), &removal_record)?;
    reject_main_slices_without_incoming_transitions(artifact.slice_details_slice(), &remaining)?;
    artifact.replace_transitions(remaining);
    let digest = artifact.digest(&removal.workflow_slug);
    let source = removal.source.as_ref();
    let target = removal.target.as_ref();
    Ok(EffectPlan::new(artifact.module_effects(
        &removal.workflow_slug,
        &digest,
        report_line(format!("removed transition {source} to {target}")),
    )))
}

fn validate_workflow_identity(
    indexed_workflow_name: &ModelName,
    indexed_workflow_description: &ModelDescription,
    workflow_graph: &FormalWorkflowGraph,
) -> Result<(), ConnectionMutationError> {
    if workflow_graph.name() != indexed_workflow_name {
        return Err(ConnectionMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_graph.name().as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    if workflow_graph.description() != indexed_workflow_description {
        return Err(ConnectionMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_graph.description().as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }
    Ok(())
}

fn retain_transitions_excluding(
    transitions: &[WorkflowTransitionRecord],
    removal_record: &WorkflowTransitionRecord,
) -> Result<Vec<WorkflowTransitionRecord>, ConnectionMutationError> {
    let mut removed_transition = false;
    let remaining = transitions
        .iter()
        .filter_map(|transition| {
            if same_transition_identity(transition, removal_record) {
                removed_transition = true;
                None
            } else {
                Some(transition.clone())
            }
        })
        .collect::<Vec<_>>();
    if removed_transition {
        Ok(remaining)
    } else {
        Err(ConnectionMutationError::new(format!(
            "workflow transition {} does not exist",
            transition_record_label(removal_record)
        )))
    }
}

/// Owned snapshot of the workflow graph fields shared by every artifact emission.
struct WorkflowArtifactInputs {
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    module_name: String,
    slice_details: WorkflowSliceDetails,
    transitions: WorkflowTransitionRecords,
    outcomes: WorkflowOutcomeRecords,
    command_errors: WorkflowCommandErrorRecords,
    owned_definitions: WorkflowOwnedDefinitionRecords,
    transition_evidences: WorkflowTransitionEvidenceRecords,
    entry_lifecycle_required: bool,
    entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
}

impl WorkflowArtifactInputs {
    fn collect(workflow_graph: &FormalWorkflowGraph) -> Self {
        let workflow_name = workflow_graph.name().clone();
        let module_name = module_name(workflow_name.as_ref());
        Self {
            workflow_name,
            workflow_description: workflow_graph.description().clone(),
            module_name,
            slice_details: workflow_graph.slice_details().clone(),
            transitions: workflow_graph.transitions().clone(),
            outcomes: workflow_graph.outcomes().clone(),
            command_errors: workflow_graph.command_errors().clone(),
            owned_definitions: workflow_graph.owned_definitions().clone(),
            transition_evidences: workflow_graph.transition_evidences().clone(),
            entry_lifecycle_required: workflow_graph.entry_lifecycle_required(),
            entry_lifecycle_states: workflow_graph.entry_lifecycle_states().clone(),
        }
    }

    fn slice_details_slice(&self) -> &[WorkflowSliceDetail] {
        self.slice_details.as_slice()
    }

    fn transitions_slice(&self) -> &[WorkflowTransitionRecord] {
        self.transitions.as_slice()
    }

    fn push_transition(&mut self, transition: WorkflowTransitionRecord) {
        let mut records = self.transitions.as_slice().to_owned();
        records.push(transition);
        self.transitions = WorkflowTransitionRecords::from_records(records);
    }

    fn replace_transitions(&mut self, transitions: Vec<WorkflowTransitionRecord>) {
        self.transitions = WorkflowTransitionRecords::from_records(transitions);
    }

    fn digest(&self, workflow_slug: &WorkflowSlug) -> ArtifactDigest {
        artifact_digest(&WorkflowArtifactDigestInput {
            workflow_name: self.workflow_name.clone(),
            workflow_slug: workflow_slug.clone(),
            workflow_description: self.workflow_description.clone(),
            workflow_slice_details: self.slice_details.clone(),
            workflow_transitions: self.transitions.clone(),
            workflow_outcomes: self.outcomes.clone(),
            workflow_command_errors: self.command_errors.clone(),
            workflow_owned_definitions: self.owned_definitions.clone(),
            workflow_transition_evidences: self.transition_evidences.clone(),
            workflow_requires_entry_lifecycle_coverage: self.entry_lifecycle_required,
            workflow_entry_lifecycle_states: self.entry_lifecycle_states.clone(),
        })
    }

    fn module_data(
        &self,
        workflow_slug: &WorkflowSlug,
        digest: &ArtifactDigest,
    ) -> WorkflowModuleData {
        WorkflowModuleData::new(
            self.workflow_name.clone(),
            self.workflow_description.clone(),
            workflow_slug.clone(),
            digest.clone(),
        )
        .with_slice_details(self.slice_details.clone())
        .with_transitions(self.transitions.clone())
        .with_outcomes(self.outcomes.clone())
        .with_command_errors(self.command_errors.clone())
        .with_owned_definitions(self.owned_definitions.clone())
        .with_transition_evidences(self.transition_evidences.clone())
        .with_entry_lifecycle_required(self.entry_lifecycle_required)
        .with_entry_lifecycle_states(self.entry_lifecycle_states.clone())
    }

    fn module_effects(
        &self,
        workflow_slug: &WorkflowSlug,
        digest: &ArtifactDigest,
        report: ReportLine,
    ) -> Vec<Effect> {
        vec![
            Effect::write_file(
                project_path(format!("model/lean/{}.lean", self.module_name)),
                emit_lean_workflow_module(
                    &lean_module_name(self.module_name.clone()),
                    &self.module_data(workflow_slug, digest),
                ),
            ),
            Effect::write_file(
                project_path(format!("model/quint/{}.qnt", self.module_name)),
                emit_quint_workflow_module(
                    &quint_module_name(self.module_name.clone()),
                    &self.module_data(workflow_slug, digest),
                ),
            ),
            Effect::Report(report),
        ]
    }
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
            if kind == WorkflowTransitionKind::Navigation
                && let (Some(source_control), Some(target_view)) =
                    (&connection.source_control, &connection.target_view)
            {
                return Ok(WorkflowTransitionRecord::new_with_navigation_endpoints(
                    source,
                    target,
                    kind,
                    connection.trigger.clone(),
                    source_control.clone(),
                    target_view.clone(),
                ));
            }
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
