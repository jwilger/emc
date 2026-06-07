// Copyright 2026 John Wilger

use eventcore::{Command, CommandError, CommandLogic, Event, NewEvents, StreamId, require};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::connection::{ConnectionKind, WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::effect::ArtifactDigest;
use crate::core::events::EventDraft;
use crate::core::formal_slice_facts::{
    NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
    NewCommandDefinition, NewEventDefinition, NewExternalPayloadDefinition, NewOutcomeDefinition,
    NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition, NewViewDefinition,
};
use crate::core::project::ProjectName;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    ModelDescription, ModelName, PayloadContractName, ReviewRuleName, ReviewTimestamp, ReviewerId,
    SliceSlug, TransitionTriggerName, WorkflowCommandErrorRecord,
    WorkflowEntryLifecycleStateRecord, WorkflowOutcomeRecord, WorkflowOwnedDefinitionRecord,
    WorkflowSlug, WorkflowTransitionEvidenceRecord,
};

pub fn project_stream_id() -> Result<StreamId, String> {
    StreamId::try_new("project".to_owned()).map_err(|error| error.to_string())
}

pub fn workflow_stream_id(slug: &str) -> Result<StreamId, String> {
    StreamId::try_new(format!("workflow::{slug}")).map_err(|error| error.to_string())
}

pub fn slice_stream_id(slug: &str) -> Result<StreamId, String> {
    StreamId::try_new(format!("slice::{slug}")).map_err(|error| error.to_string())
}

pub fn review_stream_id(workflow: &str) -> Result<StreamId, String> {
    StreamId::try_new(format!("review::{workflow}")).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum EmcEvent {
    ProjectInitialized {
        stream_id: StreamId,
        name: String,
    },
    WorkflowAdded {
        stream_id: StreamId,
        slug: String,
        name: String,
        description: String,
    },
    WorkflowUpdated {
        stream_id: StreamId,
        slug: String,
        name: String,
        description: String,
    },
    WorkflowRemoved {
        stream_id: StreamId,
        slug: String,
    },
    WorkflowOutcomeAdded {
        stream_id: StreamId,
        workflow: String,
        source_slice: String,
        label: String,
        externally_relevant: bool,
    },
    WorkflowCommandErrorAdded {
        stream_id: StreamId,
        workflow: String,
        source_slice: String,
        command: String,
        error: String,
    },
    WorkflowOwnedDefinitionAdded {
        stream_id: StreamId,
        workflow: String,
        source_slice: String,
        definition_kind: String,
        definition_name: String,
        definition_stream: Option<String>,
        source_provenance: Option<String>,
        event_participation: Option<String>,
        view_role: Option<String>,
    },
    WorkflowTransitionEvidenceAdded {
        stream_id: StreamId,
        workflow: String,
        source: String,
        target: String,
        via: String,
        name: String,
        source_evidence: String,
        target_evidence: String,
    },
    WorkflowEntryLifecycleCoverageRequired {
        stream_id: StreamId,
        workflow: String,
    },
    WorkflowEntryLifecycleStateAdded {
        stream_id: StreamId,
        workflow: String,
        state: String,
        step: String,
        evidence: String,
    },
    WorkflowReadinessDeclared {
        stream_id: StreamId,
        workflow: String,
        projection_fingerprint: String,
        model_content_digest: String,
        verified_at: String,
        verified_by: String,
        review_event_id: Option<String>,
    },
    WorkflowConnected {
        stream_id: StreamId,
        workflow: String,
        source: String,
        target_slice: Option<String>,
        target_workflow: Option<String>,
        via: String,
        name: String,
        payload_contract: Option<String>,
        reason: Option<String>,
    },
    WorkflowTransitionRemoved {
        stream_id: StreamId,
        workflow: String,
        source: String,
        target_slice: Option<String>,
        target_workflow: Option<String>,
        via: String,
        name: String,
    },
    SliceAdded {
        stream_id: StreamId,
        workflow: String,
        slug: String,
        name: String,
        kind: String,
        description: String,
    },
    SliceUpdated {
        stream_id: StreamId,
        slug: String,
        name: String,
        kind: String,
        description: String,
    },
    SliceRemoved {
        stream_id: StreamId,
        slug: String,
    },
    SliceFactAdded {
        stream_id: StreamId,
        exported_event_type: String,
        payload: Value,
    },
    ReviewRecorded {
        stream_id: StreamId,
        workflow: String,
        model_content_digest: String,
        reviewer_id: String,
        reviewed_at: String,
        categories: Vec<String>,
    },
    ConflictResolved {
        stream_id: StreamId,
        conflict_id: String,
        chosen_event_id: String,
    },
}

impl Event for EmcEvent {
    fn stream_id(&self) -> &StreamId {
        match self {
            Self::ProjectInitialized { stream_id, .. } => stream_id,
            Self::WorkflowAdded { stream_id, .. } => stream_id,
            Self::WorkflowUpdated { stream_id, .. } => stream_id,
            Self::WorkflowRemoved { stream_id, .. } => stream_id,
            Self::WorkflowOutcomeAdded { stream_id, .. } => stream_id,
            Self::WorkflowCommandErrorAdded { stream_id, .. } => stream_id,
            Self::WorkflowOwnedDefinitionAdded { stream_id, .. } => stream_id,
            Self::WorkflowTransitionEvidenceAdded { stream_id, .. } => stream_id,
            Self::WorkflowEntryLifecycleCoverageRequired { stream_id, .. } => stream_id,
            Self::WorkflowEntryLifecycleStateAdded { stream_id, .. } => stream_id,
            Self::WorkflowReadinessDeclared { stream_id, .. } => stream_id,
            Self::WorkflowConnected { stream_id, .. } => stream_id,
            Self::WorkflowTransitionRemoved { stream_id, .. } => stream_id,
            Self::SliceAdded { stream_id, .. } => stream_id,
            Self::SliceUpdated { stream_id, .. } => stream_id,
            Self::SliceRemoved { stream_id, .. } => stream_id,
            Self::SliceFactAdded { stream_id, .. } => stream_id,
            Self::ReviewRecorded { stream_id, .. } => stream_id,
            Self::ConflictResolved { stream_id, .. } => stream_id,
        }
    }

    fn event_type_name() -> &'static str {
        "EmcEvent"
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct WorkflowCommandState {
    added: bool,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct SliceCommandState {
    added: bool,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct ProjectCommandState {
    initialized: bool,
}

fn apply_workflow_command_state(
    mut state: WorkflowCommandState,
    event: &EmcEvent,
) -> WorkflowCommandState {
    match event {
        EmcEvent::WorkflowAdded { .. } | EmcEvent::WorkflowUpdated { .. } => {
            state.added = true;
        }
        EmcEvent::WorkflowRemoved { .. } => {
            state.added = false;
        }
        _ => {}
    }
    state
}

fn apply_slice_command_state(mut state: SliceCommandState, event: &EmcEvent) -> SliceCommandState {
    match event {
        EmcEvent::SliceAdded { .. } | EmcEvent::SliceUpdated { .. } => {
            state.added = true;
        }
        EmcEvent::SliceRemoved { .. } => {
            state.added = false;
        }
        _ => {}
    }
    state
}

#[derive(Command)]
pub struct InitializeProjectCommand {
    #[stream]
    project_stream: StreamId,
    name: ProjectName,
}

impl InitializeProjectCommand {
    pub fn new(name: String) -> Result<Self, String> {
        Ok(Self {
            project_stream: project_stream_id()?,
            name: ProjectName::try_new(name).map_err(|error| error.to_string())?,
        })
    }
}

impl CommandLogic for InitializeProjectCommand {
    type Event = EmcEvent;
    type State = ProjectCommandState;

    fn apply(&self, mut state: Self::State, event: &Self::Event) -> Self::State {
        if let EmcEvent::ProjectInitialized { .. } = event {
            state.initialized = true;
        }
        state
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        if state.initialized {
            return Ok(Vec::new().into());
        }
        Ok(vec![EmcEvent::ProjectInitialized {
            stream_id: self.project_stream.clone(),
            name: self.name.as_ref().to_owned(),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct AddWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    slug: WorkflowSlug,
    name: ModelName,
    description: ModelDescription,
}

impl AddWorkflowCommand {
    pub fn new(slug: String, name: String, description: String) -> Result<Self, String> {
        let slug = WorkflowSlug::try_new(slug).map_err(|error| error.to_string())?;
        Ok(Self {
            workflow_stream: workflow_stream_id(slug.as_ref())?,
            slug,
            name: ModelName::try_new(name).map_err(|error| error.to_string())?,
            description: ModelDescription::try_new(description)
                .map_err(|error| error.to_string())?,
        })
    }
}

impl CommandLogic for AddWorkflowCommand {
    type Event = EmcEvent;
    type State = WorkflowCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_workflow_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(!state.added, "workflow stream has already been added");
        Ok(vec![EmcEvent::WorkflowAdded {
            stream_id: self.workflow_stream.clone(),
            slug: self.slug.as_ref().to_owned(),
            name: self.name.as_ref().to_owned(),
            description: self.description.as_ref().to_owned(),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct UpdateWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    slug: WorkflowSlug,
    name: ModelName,
    description: ModelDescription,
}

impl UpdateWorkflowCommand {
    pub fn new(slug: String, name: String, description: String) -> Result<Self, String> {
        let slug = WorkflowSlug::try_new(slug).map_err(|error| error.to_string())?;
        Ok(Self {
            workflow_stream: workflow_stream_id(slug.as_ref())?,
            slug,
            name: ModelName::try_new(name).map_err(|error| error.to_string())?,
            description: ModelDescription::try_new(description)
                .map_err(|error| error.to_string())?,
        })
    }
}

impl CommandLogic for UpdateWorkflowCommand {
    type Event = EmcEvent;
    type State = WorkflowCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_workflow_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "workflow stream must exist before update");
        Ok(vec![EmcEvent::WorkflowUpdated {
            stream_id: self.workflow_stream.clone(),
            slug: self.slug.as_ref().to_owned(),
            name: self.name.as_ref().to_owned(),
            description: self.description.as_ref().to_owned(),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct RemoveWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    slug: WorkflowSlug,
}

impl RemoveWorkflowCommand {
    pub fn new(slug: String) -> Result<Self, String> {
        let slug = WorkflowSlug::try_new(slug).map_err(|error| error.to_string())?;
        Ok(Self {
            workflow_stream: workflow_stream_id(slug.as_ref())?,
            slug,
        })
    }
}

impl CommandLogic for RemoveWorkflowCommand {
    type Event = EmcEvent;
    type State = WorkflowCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_workflow_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "workflow stream must exist before removal");
        Ok(vec![EmcEvent::WorkflowRemoved {
            stream_id: self.workflow_stream.clone(),
            slug: self.slug.as_ref().to_owned(),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct ConnectWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    connection: WorkflowConnection,
}

pub struct ConnectWorkflowInput {
    pub workflow: String,
    pub source: String,
    pub target_slice: Option<String>,
    pub target_workflow: Option<String>,
    pub via: String,
    pub name: String,
    pub payload_contract: Option<String>,
    pub reason: Option<String>,
}

impl ConnectWorkflowCommand {
    pub fn new(input: ConnectWorkflowInput) -> Result<Self, String> {
        let workflow = WorkflowSlug::try_new(input.workflow).map_err(|error| error.to_string())?;
        let source = SliceSlug::try_new(input.source).map_err(|error| error.to_string())?;
        let kind = connection_kind(&input.via)?;
        let trigger =
            TransitionTriggerName::try_new(input.name).map_err(|error| error.to_string())?;
        let connection = workflow_connection_from_input(WorkflowConnectionCommandInput {
            workflow: workflow.clone(),
            source,
            kind,
            trigger,
            target_slice: input.target_slice,
            target_workflow: input.target_workflow,
            payload_contract: input.payload_contract,
            reason: input.reason,
        })?;
        Ok(Self {
            workflow_stream: workflow_stream_id(workflow.as_ref())?,
            connection,
        })
    }
}

impl CommandLogic for ConnectWorkflowCommand {
    type Event = EmcEvent;
    type State = WorkflowCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_workflow_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "workflow stream must exist before connection");
        Ok(vec![EmcEvent::WorkflowConnected {
            stream_id: self.workflow_stream.clone(),
            workflow: self.connection.workflow_slug().as_ref().to_owned(),
            source: self.connection.source().as_ref().to_owned(),
            target_slice: self
                .connection
                .target()
                .slice_slug()
                .map(|slug| slug.as_ref().to_owned()),
            target_workflow: self
                .connection
                .target()
                .workflow_slug()
                .map(|slug| slug.as_ref().to_owned()),
            via: self.connection.kind().trigger_kind().to_owned(),
            name: self.connection.trigger().as_ref().to_owned(),
            payload_contract: self
                .connection
                .payload_contract()
                .map(|payload_contract| payload_contract.as_ref().to_owned()),
            reason: self
                .connection
                .target()
                .reason()
                .map(|reason| reason.as_ref().to_owned()),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct RemoveWorkflowTransitionCommand {
    #[stream]
    workflow_stream: StreamId,
    removal: WorkflowTransitionRemoval,
}

pub struct RemoveWorkflowTransitionInput {
    pub workflow: String,
    pub source: String,
    pub target_slice: Option<String>,
    pub target_workflow: Option<String>,
    pub via: String,
    pub name: String,
}

impl RemoveWorkflowTransitionCommand {
    pub fn new(input: RemoveWorkflowTransitionInput) -> Result<Self, String> {
        let workflow = WorkflowSlug::try_new(input.workflow).map_err(|error| error.to_string())?;
        let source = SliceSlug::try_new(input.source).map_err(|error| error.to_string())?;
        let kind = connection_kind(&input.via)?;
        let trigger =
            TransitionTriggerName::try_new(input.name).map_err(|error| error.to_string())?;
        let removal =
            workflow_transition_removal_from_input(WorkflowTransitionRemovalCommandInput {
                workflow: workflow.clone(),
                source,
                kind,
                trigger,
                target_slice: input.target_slice,
                target_workflow: input.target_workflow,
            })?;
        Ok(Self {
            workflow_stream: workflow_stream_id(workflow.as_ref())?,
            removal,
        })
    }
}

impl CommandLogic for RemoveWorkflowTransitionCommand {
    type Event = EmcEvent;
    type State = WorkflowCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_workflow_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "workflow stream must exist before transition removal"
        );
        Ok(vec![EmcEvent::WorkflowTransitionRemoved {
            stream_id: self.workflow_stream.clone(),
            workflow: self.removal.workflow_slug().as_ref().to_owned(),
            source: self.removal.source().as_ref().to_owned(),
            target_slice: self
                .removal
                .target()
                .slice_slug()
                .map(|slug| slug.as_ref().to_owned()),
            target_workflow: self
                .removal
                .target()
                .workflow_slug()
                .map(|slug| slug.as_ref().to_owned()),
            via: self.removal.kind().trigger_kind().to_owned(),
            name: self.removal.trigger().as_ref().to_owned(),
        }]
        .into())
    }
}

struct WorkflowConnectionCommandInput {
    workflow: WorkflowSlug,
    source: SliceSlug,
    kind: ConnectionKind,
    trigger: TransitionTriggerName,
    target_slice: Option<String>,
    target_workflow: Option<String>,
    payload_contract: Option<String>,
    reason: Option<String>,
}

fn workflow_connection_from_input(
    input: WorkflowConnectionCommandInput,
) -> Result<WorkflowConnection, String> {
    match (
        input.target_slice,
        input.target_workflow,
        input.payload_contract,
        input.reason,
    ) {
        (Some(target), None, None, None) => Ok(WorkflowConnection::new(
            input.workflow,
            input.source,
            SliceSlug::try_new(target).map_err(|error| error.to_string())?,
            input.kind,
            input.trigger,
        )),
        (Some(target), None, Some(payload_contract), None) => {
            Ok(WorkflowConnection::new_with_payload_contract(
                input.workflow,
                input.source,
                SliceSlug::try_new(target).map_err(|error| error.to_string())?,
                input.kind,
                input.trigger,
                PayloadContractName::try_new(payload_contract)
                    .map_err(|error| error.to_string())?,
            ))
        }
        (None, Some(target), None, Some(reason)) => Ok(WorkflowConnection::new_workflow_exit(
            input.workflow,
            input.source,
            WorkflowSlug::try_new(target).map_err(|error| error.to_string())?,
            input.kind,
            input.trigger,
            ModelDescription::try_new(reason).map_err(|error| error.to_string())?,
        )),
        (Some(_), Some(_), _, _) => {
            Err("workflow connection cannot target both slice and workflow".to_owned())
        }
        (None, None, _, _) => Err("workflow connection target is required".to_owned()),
        (None, Some(_), Some(_), _) => {
            Err("workflow exit connection cannot declare a payload contract".to_owned())
        }
        (None, Some(_), None, None) => Err("workflow exit connection requires a reason".to_owned()),
        (Some(_), None, _, Some(_)) => {
            Err("slice workflow connection cannot declare a workflow exit reason".to_owned())
        }
    }
}

struct WorkflowTransitionRemovalCommandInput {
    workflow: WorkflowSlug,
    source: SliceSlug,
    kind: ConnectionKind,
    trigger: TransitionTriggerName,
    target_slice: Option<String>,
    target_workflow: Option<String>,
}

fn workflow_transition_removal_from_input(
    input: WorkflowTransitionRemovalCommandInput,
) -> Result<WorkflowTransitionRemoval, String> {
    match (input.target_slice, input.target_workflow) {
        (Some(target), None) => Ok(WorkflowTransitionRemoval::new(
            input.workflow,
            input.source,
            SliceSlug::try_new(target).map_err(|error| error.to_string())?,
            input.kind,
            input.trigger,
        )),
        (None, Some(target)) => Ok(WorkflowTransitionRemoval::new_workflow_exit(
            input.workflow,
            input.source,
            WorkflowSlug::try_new(target).map_err(|error| error.to_string())?,
            input.kind,
            input.trigger,
        )),
        (Some(_), Some(_)) => {
            Err("workflow transition removal cannot target both slice and workflow".to_owned())
        }
        (None, None) => Err("workflow transition removal target is required".to_owned()),
    }
}

fn connection_kind(value: &str) -> Result<ConnectionKind, String> {
    match value {
        "command" => Ok(ConnectionKind::command()),
        "event" => Ok(ConnectionKind::event()),
        "navigation" => Ok(ConnectionKind::navigation()),
        "external_trigger" => Ok(ConnectionKind::external_trigger()),
        "outcome" => Ok(ConnectionKind::outcome()),
        _ => Err(format!("unknown workflow connection kind {value}")),
    }
}

#[derive(Command)]
pub struct AddWorkflowFactCommand {
    #[stream]
    workflow_stream: StreamId,
    fact: WorkflowFactInput,
}

pub enum WorkflowFactInput {
    OutcomeAdded {
        workflow: WorkflowSlug,
        outcome: WorkflowOutcomeRecord,
    },
    CommandErrorAdded {
        workflow: WorkflowSlug,
        error: WorkflowCommandErrorRecord,
    },
    OwnedDefinitionAdded {
        workflow: WorkflowSlug,
        definition: WorkflowOwnedDefinitionRecord,
    },
    TransitionEvidenceAdded {
        workflow: WorkflowSlug,
        evidence: WorkflowTransitionEvidenceRecord,
    },
    EntryLifecycleCoverageRequired {
        workflow: WorkflowSlug,
    },
    EntryLifecycleStateAdded {
        workflow: WorkflowSlug,
        state: WorkflowEntryLifecycleStateRecord,
    },
}

impl WorkflowFactInput {
    fn workflow(&self) -> &str {
        match self {
            Self::OutcomeAdded { workflow, .. }
            | Self::CommandErrorAdded { workflow, .. }
            | Self::OwnedDefinitionAdded { workflow, .. }
            | Self::TransitionEvidenceAdded { workflow, .. }
            | Self::EntryLifecycleCoverageRequired { workflow }
            | Self::EntryLifecycleStateAdded { workflow, .. } => workflow.as_ref(),
        }
    }

    fn to_event(&self, stream_id: StreamId) -> EmcEvent {
        match self {
            Self::OutcomeAdded { workflow, outcome } => EmcEvent::WorkflowOutcomeAdded {
                stream_id,
                workflow: workflow.as_ref().to_owned(),
                source_slice: outcome.source_slice().as_ref().to_owned(),
                label: outcome.label().as_ref().to_owned(),
                externally_relevant: outcome.externally_relevant(),
            },
            Self::CommandErrorAdded { workflow, error } => EmcEvent::WorkflowCommandErrorAdded {
                stream_id,
                workflow: workflow.as_ref().to_owned(),
                source_slice: error.source_slice().as_ref().to_owned(),
                command: error.command_name().as_ref().to_owned(),
                error: error.error_name().as_ref().to_owned(),
            },
            Self::OwnedDefinitionAdded {
                workflow,
                definition,
            } => EmcEvent::WorkflowOwnedDefinitionAdded {
                stream_id,
                workflow: workflow.as_ref().to_owned(),
                source_slice: definition.source_slice().as_ref().to_owned(),
                definition_kind: definition.definition_kind().as_ref().to_owned(),
                definition_name: definition.definition_name().as_ref().to_owned(),
                definition_stream: definition
                    .definition_stream()
                    .map(|stream| stream.as_ref().to_owned()),
                source_provenance: definition
                    .source_provenance()
                    .map(|provenance| provenance.as_ref().to_owned()),
                event_participation: definition
                    .event_participation()
                    .map(|participation| participation.as_ref().to_owned()),
                view_role: definition.view_role().map(|role| role.as_ref().to_owned()),
            },
            Self::TransitionEvidenceAdded { workflow, evidence } => {
                EmcEvent::WorkflowTransitionEvidenceAdded {
                    stream_id,
                    workflow: workflow.as_ref().to_owned(),
                    source: evidence.source().as_ref().to_owned(),
                    target: evidence.target().as_ref().to_owned(),
                    via: evidence.kind().as_ref().to_owned(),
                    name: evidence.trigger().as_ref().to_owned(),
                    source_evidence: evidence.source_evidence().as_ref().to_owned(),
                    target_evidence: evidence.target_evidence().as_ref().to_owned(),
                }
            }
            Self::EntryLifecycleCoverageRequired { workflow } => {
                EmcEvent::WorkflowEntryLifecycleCoverageRequired {
                    stream_id,
                    workflow: workflow.as_ref().to_owned(),
                }
            }
            Self::EntryLifecycleStateAdded { workflow, state } => {
                EmcEvent::WorkflowEntryLifecycleStateAdded {
                    stream_id,
                    workflow: workflow.as_ref().to_owned(),
                    state: state.state().as_ref().to_owned(),
                    step: state.step().as_ref().to_owned(),
                    evidence: state.evidence().as_ref().to_owned(),
                }
            }
        }
    }
}

impl AddWorkflowFactCommand {
    pub fn new(fact: WorkflowFactInput) -> Result<Self, String> {
        Ok(Self {
            workflow_stream: workflow_stream_id(fact.workflow())?,
            fact,
        })
    }
}

impl CommandLogic for AddWorkflowFactCommand {
    type Event = EmcEvent;
    type State = WorkflowCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_workflow_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "workflow stream must exist before adding fact");
        Ok(vec![self.fact.to_event(self.workflow_stream.clone())].into())
    }
}

#[derive(Command)]
pub struct AddSliceCommand {
    #[stream]
    slice_stream: StreamId,
    slice: NewSlice,
}

impl AddSliceCommand {
    pub fn new(
        workflow: String,
        slug: String,
        name: String,
        kind: String,
        description: String,
    ) -> Result<Self, String> {
        let slice = NewSlice::new(
            WorkflowSlug::try_new(workflow).map_err(|error| error.to_string())?,
            SliceSlug::try_new(slug).map_err(|error| error.to_string())?,
            ModelName::try_new(name).map_err(|error| error.to_string())?,
            ModelDescription::try_new(description).map_err(|error| error.to_string())?,
            slice_kind(&kind)?,
        );
        Ok(Self {
            slice_stream: slice_stream_id(slice.slug().as_ref())?,
            slice,
        })
    }
}

impl CommandLogic for AddSliceCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(!state.added, "slice stream has already been added");
        Ok(vec![EmcEvent::SliceAdded {
            stream_id: self.slice_stream.clone(),
            workflow: self.slice.workflow_slug().as_ref().to_owned(),
            slug: self.slice.slug().as_ref().to_owned(),
            name: self.slice.name().as_ref().to_owned(),
            kind: self.slice.kind().as_str().to_owned(),
            description: self.slice.description().as_ref().to_owned(),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct UpdateSliceCommand {
    #[stream]
    slice_stream: StreamId,
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKind,
    description: ModelDescription,
}

impl UpdateSliceCommand {
    pub fn new(
        slug: String,
        name: String,
        kind: String,
        description: String,
    ) -> Result<Self, String> {
        let slug = SliceSlug::try_new(slug).map_err(|error| error.to_string())?;
        Ok(Self {
            slice_stream: slice_stream_id(slug.as_ref())?,
            slug,
            name: ModelName::try_new(name).map_err(|error| error.to_string())?,
            kind: slice_kind(&kind)?,
            description: ModelDescription::try_new(description)
                .map_err(|error| error.to_string())?,
        })
    }
}

impl CommandLogic for UpdateSliceCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "slice stream must exist before update");
        Ok(vec![EmcEvent::SliceUpdated {
            stream_id: self.slice_stream.clone(),
            slug: self.slug.as_ref().to_owned(),
            name: self.name.as_ref().to_owned(),
            kind: self.kind.as_str().to_owned(),
            description: self.description.as_ref().to_owned(),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct RemoveSliceCommand {
    #[stream]
    slice_stream: StreamId,
    slug: SliceSlug,
}

impl RemoveSliceCommand {
    pub fn new(slug: String) -> Result<Self, String> {
        let slug = SliceSlug::try_new(slug).map_err(|error| error.to_string())?;
        Ok(Self {
            slice_stream: slice_stream_id(slug.as_ref())?,
            slug,
        })
    }
}

impl CommandLogic for RemoveSliceCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "slice stream must exist before removal");
        Ok(vec![EmcEvent::SliceRemoved {
            stream_id: self.slice_stream.clone(),
            slug: self.slug.as_ref().to_owned(),
        }]
        .into())
    }
}

fn slice_kind(value: &str) -> Result<SliceKind, String> {
    match value {
        "state_view" => Ok(SliceKind::state_view()),
        "state_change" => Ok(SliceKind::state_change()),
        "translation" => Ok(SliceKind::translation()),
        "automation" => Ok(SliceKind::automation()),
        _ => Err(format!("unknown slice kind {value}")),
    }
}

#[derive(Command)]
pub struct AddSliceFactCommand {
    #[stream]
    slice_stream: StreamId,
    fact: SliceFactInput,
}

impl AddSliceFactCommand {
    pub fn new(fact: SliceFactInput) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(fact.slice_slug().as_ref())?,
            fact,
        })
    }
}

impl CommandLogic for AddSliceFactCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "slice stream must exist before adding fact");
        let draft = self.fact.to_event_draft();
        Ok(vec![EmcEvent::SliceFactAdded {
            stream_id: self.slice_stream.clone(),
            exported_event_type: draft.event_type().to_owned(),
            payload: draft.payload().clone(),
        }]
        .into())
    }
}

pub enum SliceFactInput {
    Scenario(NewSliceScenario),
    Outcome(NewOutcomeDefinition),
    ExternalPayload(NewExternalPayloadDefinition),
    EventDefinition(NewEventDefinition),
    CommandDefinition(NewCommandDefinition),
    ReadModel(NewReadModelDefinition),
    View(NewViewDefinition),
    BitLevelDataFlow(NewBitLevelDataFlow),
    Translation(NewTranslationDefinition),
    Automation(NewAutomationDefinition),
    BoardElement(NewBoardElement),
    BoardConnection(NewBoardConnection),
}

impl SliceFactInput {
    fn slice_slug(&self) -> &SliceSlug {
        match self {
            Self::Scenario(fact) => fact.slice_slug(),
            Self::Outcome(fact) => fact.slice_slug(),
            Self::ExternalPayload(fact) => fact.slice_slug(),
            Self::EventDefinition(fact) => fact.slice_slug(),
            Self::CommandDefinition(fact) => fact.slice_slug(),
            Self::ReadModel(fact) => fact.slice_slug(),
            Self::View(fact) => fact.slice_slug(),
            Self::BitLevelDataFlow(fact) => fact.slice_slug(),
            Self::Translation(fact) => fact.slice_slug(),
            Self::Automation(fact) => fact.slice_slug(),
            Self::BoardElement(fact) => fact.slice_slug(),
            Self::BoardConnection(fact) => fact.slice_slug(),
        }
    }

    fn to_event_draft(&self) -> EventDraft {
        match self {
            Self::Scenario(fact) => EventDraft::slice_scenario_added(fact),
            Self::Outcome(fact) => EventDraft::slice_outcome_added(fact),
            Self::ExternalPayload(fact) => EventDraft::slice_external_payload_added(fact),
            Self::EventDefinition(fact) => EventDraft::slice_event_definition_added(fact),
            Self::CommandDefinition(fact) => EventDraft::slice_command_definition_added(fact),
            Self::ReadModel(fact) => EventDraft::slice_read_model_added(fact),
            Self::View(fact) => EventDraft::slice_view_added(fact),
            Self::BitLevelDataFlow(fact) => EventDraft::slice_bit_level_data_flow_added(fact),
            Self::Translation(fact) => EventDraft::slice_translation_added(fact),
            Self::Automation(fact) => EventDraft::slice_automation_added(fact),
            Self::BoardElement(fact) => EventDraft::slice_board_element_added(fact),
            Self::BoardConnection(fact) => EventDraft::slice_board_connection_added(fact),
        }
    }
}

#[derive(Command)]
pub struct RecordReviewCommand {
    #[stream]
    review_stream: StreamId,
    workflow: WorkflowSlug,
    model_content_digest: ArtifactDigest,
    reviewer_id: ReviewerId,
    reviewed_at: ReviewTimestamp,
    categories: Vec<ReviewRuleName>,
}

impl RecordReviewCommand {
    pub fn new(
        workflow: String,
        model_content_digest: String,
        reviewer_id: String,
        reviewed_at: String,
        categories: Vec<String>,
    ) -> Result<Self, String> {
        let workflow = WorkflowSlug::try_new(workflow).map_err(|error| error.to_string())?;
        Ok(Self {
            review_stream: review_stream_id(workflow.as_ref())?,
            workflow,
            model_content_digest: ArtifactDigest::try_new(model_content_digest)
                .map_err(|error| error.to_string())?,
            reviewer_id: ReviewerId::try_new(reviewer_id).map_err(|error| error.to_string())?,
            reviewed_at: ReviewTimestamp::try_new(reviewed_at)
                .map_err(|error| error.to_string())?,
            categories: categories
                .into_iter()
                .map(ReviewRuleName::try_new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?,
        })
    }
}

impl CommandLogic for RecordReviewCommand {
    type Event = EmcEvent;
    type State = ProjectCommandState;

    fn apply(&self, state: Self::State, _event: &Self::Event) -> Self::State {
        state
    }

    fn handle(&self, _state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        Ok(vec![EmcEvent::ReviewRecorded {
            stream_id: self.review_stream.clone(),
            workflow: self.workflow.as_ref().to_owned(),
            model_content_digest: self.model_content_digest.as_ref().to_owned(),
            reviewer_id: self.reviewer_id.as_ref().to_owned(),
            reviewed_at: self.reviewed_at.as_ref().to_owned(),
            categories: self
                .categories
                .iter()
                .map(|category| category.as_ref().to_owned())
                .collect(),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct DeclareWorkflowReadinessCommand {
    #[stream]
    workflow_stream: StreamId,
    workflow: WorkflowSlug,
    projection_fingerprint: ArtifactDigest,
    model_content_digest: ArtifactDigest,
    verified_at: ReviewTimestamp,
    verified_by: ReviewerId,
    review_event_id: Option<ArtifactDigest>,
}

impl DeclareWorkflowReadinessCommand {
    pub fn new(
        workflow: WorkflowSlug,
        projection_fingerprint: ArtifactDigest,
        model_content_digest: ArtifactDigest,
        verified_at: ReviewTimestamp,
        verified_by: ReviewerId,
        review_event_id: Option<ArtifactDigest>,
    ) -> Result<Self, String> {
        Ok(Self {
            workflow_stream: workflow_stream_id(workflow.as_ref())?,
            workflow,
            projection_fingerprint,
            model_content_digest,
            verified_at,
            verified_by,
            review_event_id,
        })
    }
}

impl CommandLogic for DeclareWorkflowReadinessCommand {
    type Event = EmcEvent;
    type State = WorkflowCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_workflow_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(state.added, "workflow stream must exist before readiness");
        Ok(vec![EmcEvent::WorkflowReadinessDeclared {
            stream_id: self.workflow_stream.clone(),
            workflow: self.workflow.as_ref().to_owned(),
            projection_fingerprint: self.projection_fingerprint.as_ref().to_owned(),
            model_content_digest: self.model_content_digest.as_ref().to_owned(),
            verified_at: self.verified_at.as_ref().to_owned(),
            verified_by: self.verified_by.as_ref().to_owned(),
            review_event_id: self
                .review_event_id
                .as_ref()
                .map(|event_id| event_id.as_ref().to_owned()),
        }]
        .into())
    }
}

#[derive(Command)]
pub struct ResolveConflictCommand {
    #[stream]
    project_stream: StreamId,
    conflict_id: ArtifactDigest,
    chosen_event_id: ArtifactDigest,
}

impl ResolveConflictCommand {
    pub fn new(conflict_id: String, chosen_event_id: String) -> Result<Self, String> {
        Ok(Self {
            project_stream: project_stream_id()?,
            conflict_id: ArtifactDigest::try_new(conflict_id).map_err(|error| error.to_string())?,
            chosen_event_id: ArtifactDigest::try_new(chosen_event_id)
                .map_err(|error| error.to_string())?,
        })
    }
}

impl CommandLogic for ResolveConflictCommand {
    type Event = EmcEvent;
    type State = ProjectCommandState;

    fn apply(&self, state: Self::State, _event: &Self::Event) -> Self::State {
        state
    }

    fn handle(&self, _state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        Ok(vec![EmcEvent::ConflictResolved {
            stream_id: self.project_stream.clone(),
            conflict_id: self.conflict_id.as_ref().to_owned(),
            chosen_event_id: self.chosen_event_id.as_ref().to_owned(),
        }]
        .into())
    }
}
