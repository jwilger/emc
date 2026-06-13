// Copyright 2026 John Wilger

use eventcore::{Command, CommandError, CommandLogic, Event, NewEvents, StreamId, require};
use serde::de::Error as DeserializeError;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::core::connection::{ConnectionKind, WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::effect::{
    ChosenEventId, EventConflictId, ModelContentDigest, ProjectionFingerprint, ReviewEventReference,
};
use crate::core::events::{EventDraft, ExportedEventBody, ExportedEventType};
use crate::core::formal_slice_facts::{
    NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
    NewCommandDefinition, NewEventDefinition, NewExternalPayloadDefinition, NewOutcomeDefinition,
    NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition, NewViewDefinition,
};
use crate::core::project::ProjectName;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    CommandErrorName, CommandName, ModelDescription, ModelName, OutcomeLabelName,
    PayloadContractName, ReviewRuleName, ReviewTimestamp, ReviewerId, SliceKindName, SliceSlug,
    StreamName, TransitionTriggerName, WorkflowCommandErrorRecord,
    WorkflowEntryLifecycleEvidenceText, WorkflowEntryLifecycleStateName,
    WorkflowEntryLifecycleStateRecord, WorkflowEventParticipation, WorkflowOutcomeRecord,
    WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord,
    WorkflowSlug, WorkflowTransitionEndpoint, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionKind, WorkflowTransitionSourceEvidenceText,
    WorkflowTransitionTargetEvidenceText, WorkflowViewRole,
};

pub(crate) fn project_stream_id() -> Result<StreamId, String> {
    StreamId::try_new("project".to_owned()).map_err(|error| error.to_string())
}

pub(crate) fn workflow_stream_id(slug: &str) -> Result<StreamId, String> {
    StreamId::try_new(format!("workflow::{slug}")).map_err(|error| error.to_string())
}

pub(crate) fn slice_stream_id(slug: &str) -> Result<StreamId, String> {
    StreamId::try_new(format!("slice::{slug}")).map_err(|error| error.to_string())
}

pub(crate) fn review_stream_id(workflow: &str) -> Result<StreamId, String> {
    StreamId::try_new(format!("review::{workflow}")).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum EmcEvent {
    ProjectInitialized {
        stream_id: StreamId,
        name: ProjectName,
    },
    WorkflowAdded {
        stream_id: StreamId,
        slug: WorkflowSlug,
        name: ModelName,
        description: ModelDescription,
    },
    WorkflowUpdated {
        stream_id: StreamId,
        slug: WorkflowSlug,
        name: ModelName,
        description: ModelDescription,
    },
    WorkflowRemoved {
        stream_id: StreamId,
        slug: WorkflowSlug,
    },
    WorkflowOutcomeAdded {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        source_slice: WorkflowTransitionEndpoint,
        label: OutcomeLabelName,
        externally_relevant: bool,
    },
    WorkflowCommandErrorAdded {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        source_slice: WorkflowTransitionEndpoint,
        command: CommandName,
        error: CommandErrorName,
    },
    WorkflowOwnedDefinitionAdded {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        source_slice: WorkflowTransitionEndpoint,
        definition_kind: WorkflowOwnedDefinitionKind,
        definition_name: WorkflowOwnedDefinitionName,
        definition_stream: Option<StreamName>,
        source_provenance: Option<ModelDescription>,
        event_participation: Option<WorkflowEventParticipation>,
        view_role: Option<WorkflowViewRole>,
    },
    WorkflowTransitionEvidenceAdded {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        via: WorkflowTransitionKind,
        name: TransitionTriggerName,
        source_evidence: WorkflowTransitionSourceEvidenceText,
        target_evidence: WorkflowTransitionTargetEvidenceText,
        source_control: Option<TransitionTriggerName>,
        target_view: Option<WorkflowOwnedDefinitionName>,
    },
    WorkflowEntryLifecycleCoverageRequired {
        stream_id: StreamId,
        workflow: WorkflowSlug,
    },
    WorkflowEntryLifecycleStateAdded {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        state: WorkflowEntryLifecycleStateName,
        step: WorkflowTransitionEndpoint,
        evidence: WorkflowEntryLifecycleEvidenceText,
    },
    WorkflowReadinessDeclared {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        projection_fingerprint: ProjectionFingerprint,
        model_content_digest: ModelContentDigest,
        verified_at: ReviewTimestamp,
        verified_by: ReviewerId,
        #[serde(rename = "review_event_id")]
        review_event: ReviewEventReference,
    },
    WorkflowConnected {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        source: SliceSlug,
        target_slice: Option<SliceSlug>,
        target_workflow: Option<WorkflowSlug>,
        via: ConnectionKind,
        name: TransitionTriggerName,
        payload_contract: Option<PayloadContractName>,
        reason: Option<ModelDescription>,
        source_control: Option<TransitionTriggerName>,
        target_view: Option<WorkflowOwnedDefinitionName>,
    },
    WorkflowTransitionRemoved {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        source: SliceSlug,
        target_slice: Option<SliceSlug>,
        target_workflow: Option<WorkflowSlug>,
        via: ConnectionKind,
        name: TransitionTriggerName,
    },
    SliceAdded {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
    },
    SliceUpdated {
        stream_id: StreamId,
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
    },
    SliceRemoved {
        stream_id: StreamId,
        slug: SliceSlug,
    },
    SliceFactAdded {
        stream_id: StreamId,
        #[serde(flatten)]
        fact: SliceFactEvent,
    },
    ReviewRecorded {
        stream_id: StreamId,
        workflow: WorkflowSlug,
        model_content_digest: ModelContentDigest,
        reviewer_id: ReviewerId,
        reviewed_at: ReviewTimestamp,
        categories: Vec<ReviewRuleName>,
    },
    ConflictResolved {
        stream_id: StreamId,
        conflict_id: EventConflictId,
        chosen_event_id: ChosenEventId,
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
pub(crate) struct WorkflowCommandState {
    added: bool,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub(crate) struct SliceCommandState {
    added: bool,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub(crate) struct ProjectCommandState {
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
pub(crate) struct InitializeProjectCommand {
    #[stream]
    project_stream: StreamId,
    name: ProjectName,
}

impl InitializeProjectCommand {
    pub(crate) fn from_semantic(name: ProjectName) -> Result<Self, String> {
        Ok(Self {
            project_stream: project_stream_id()?,
            name,
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
            name: self.name.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct AddWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    slug: WorkflowSlug,
    name: ModelName,
    description: ModelDescription,
}

impl AddWorkflowCommand {
    pub(crate) fn from_semantic(
        slug: WorkflowSlug,
        name: ModelName,
        description: ModelDescription,
    ) -> Result<Self, String> {
        Ok(Self {
            workflow_stream: workflow_stream_id(slug.as_ref())?,
            slug,
            name,
            description,
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
            slug: self.slug.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct UpdateWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    slug: WorkflowSlug,
    name: ModelName,
    description: ModelDescription,
}

impl UpdateWorkflowCommand {
    pub(crate) fn from_semantic(
        slug: WorkflowSlug,
        name: ModelName,
        description: ModelDescription,
    ) -> Result<Self, String> {
        Ok(Self {
            workflow_stream: workflow_stream_id(slug.as_ref())?,
            slug,
            name,
            description,
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
            slug: self.slug.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    slug: WorkflowSlug,
}

impl RemoveWorkflowCommand {
    pub(crate) fn from_semantic(slug: WorkflowSlug) -> Result<Self, String> {
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
            slug: self.slug.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct ConnectWorkflowCommand {
    #[stream]
    workflow_stream: StreamId,
    connection: WorkflowConnection,
}

impl ConnectWorkflowCommand {
    pub(crate) fn from_connection(connection: WorkflowConnection) -> Result<Self, String> {
        Ok(Self {
            workflow_stream: workflow_stream_id(connection.workflow_slug().as_ref())?,
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
            workflow: self.connection.workflow_slug().clone(),
            source: self.connection.source().clone(),
            target_slice: self.connection.target().slice_slug().cloned(),
            target_workflow: self.connection.target().workflow_slug().cloned(),
            via: self.connection.kind(),
            name: self.connection.trigger().clone(),
            payload_contract: self.connection.payload_contract().cloned(),
            reason: self.connection.target().reason().cloned(),
            source_control: self.connection.source_control().cloned(),
            target_view: self.connection.target_view().cloned(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveWorkflowTransitionCommand {
    #[stream]
    workflow_stream: StreamId,
    removal: WorkflowTransitionRemoval,
}

impl RemoveWorkflowTransitionCommand {
    pub(crate) fn from_removal(removal: WorkflowTransitionRemoval) -> Result<Self, String> {
        Ok(Self {
            workflow_stream: workflow_stream_id(removal.workflow_slug().as_ref())?,
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
            workflow: self.removal.workflow_slug().clone(),
            source: self.removal.source().clone(),
            target_slice: self.removal.target().slice_slug().cloned(),
            target_workflow: self.removal.target().workflow_slug().cloned(),
            via: self.removal.kind(),
            name: self.removal.trigger().clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct AddWorkflowFactCommand {
    #[stream]
    workflow_stream: StreamId,
    fact: WorkflowFactInput,
}

pub(crate) enum WorkflowFactInput {
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
                workflow: workflow.clone(),
                source_slice: outcome.source_slice().clone(),
                label: outcome.label().clone(),
                externally_relevant: outcome.externally_relevant(),
            },
            Self::CommandErrorAdded { workflow, error } => EmcEvent::WorkflowCommandErrorAdded {
                stream_id,
                workflow: workflow.clone(),
                source_slice: error.source_slice().clone(),
                command: error.command_name().clone(),
                error: error.error_name().clone(),
            },
            Self::OwnedDefinitionAdded {
                workflow,
                definition,
            } => EmcEvent::WorkflowOwnedDefinitionAdded {
                stream_id,
                workflow: workflow.clone(),
                source_slice: definition.source_slice().clone(),
                definition_kind: *definition.definition_kind(),
                definition_name: definition.definition_name().clone(),
                definition_stream: definition.definition_stream().cloned(),
                source_provenance: definition.source_provenance().cloned(),
                event_participation: definition.event_participation().copied(),
                view_role: definition.view_role().copied(),
            },
            Self::TransitionEvidenceAdded { workflow, evidence } => {
                EmcEvent::WorkflowTransitionEvidenceAdded {
                    stream_id,
                    workflow: workflow.clone(),
                    source: evidence.source().clone(),
                    target: evidence.target().clone(),
                    via: *evidence.kind(),
                    name: evidence.trigger().clone(),
                    source_evidence: evidence.source_evidence().clone(),
                    target_evidence: evidence.target_evidence().clone(),
                    source_control: evidence.source_control().cloned(),
                    target_view: evidence.target_view().cloned(),
                }
            }
            Self::EntryLifecycleCoverageRequired { workflow } => {
                EmcEvent::WorkflowEntryLifecycleCoverageRequired {
                    stream_id,
                    workflow: workflow.clone(),
                }
            }
            Self::EntryLifecycleStateAdded { workflow, state } => {
                EmcEvent::WorkflowEntryLifecycleStateAdded {
                    stream_id,
                    workflow: workflow.clone(),
                    state: *state.state(),
                    step: state.step().clone(),
                    evidence: state.evidence().clone(),
                }
            }
        }
    }
}

impl AddWorkflowFactCommand {
    pub(crate) fn new(fact: WorkflowFactInput) -> Result<Self, String> {
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
pub(crate) struct AddSliceCommand {
    #[stream]
    slice_stream: StreamId,
    slice: NewSlice,
}

impl AddSliceCommand {
    pub(crate) fn from_semantic(
        workflow: WorkflowSlug,
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
    ) -> Result<Self, String> {
        let slice = NewSlice::new(workflow, slug, name, description, kind.into());
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
            workflow: self.slice.workflow_slug().clone(),
            slug: self.slice.slug().clone(),
            name: self.slice.name().clone(),
            kind: self.slice.kind().into(),
            description: self.slice.description().clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct UpdateSliceCommand {
    #[stream]
    slice_stream: StreamId,
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKind,
    description: ModelDescription,
}

impl UpdateSliceCommand {
    pub(crate) fn from_semantic(
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
    ) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slug.as_ref())?,
            slug,
            name,
            kind: kind.into(),
            description,
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
            slug: self.slug.clone(),
            name: self.name.clone(),
            kind: self.kind.into(),
            description: self.description.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveSliceCommand {
    #[stream]
    slice_stream: StreamId,
    slug: SliceSlug,
}

impl RemoveSliceCommand {
    pub(crate) fn from_semantic(slug: SliceSlug) -> Result<Self, String> {
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
            slug: self.slug.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct AddSliceFactCommand {
    #[stream]
    slice_stream: StreamId,
    fact: SliceFactInput,
}

impl AddSliceFactCommand {
    pub(crate) fn new(fact: SliceFactInput) -> Result<Self, String> {
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
        Ok(vec![EmcEvent::SliceFactAdded {
            stream_id: self.slice_stream.clone(),
            fact: SliceFactEvent::new(self.fact.clone()),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceFactEvent {
    fact: Box<SliceFactInput>,
}

impl SliceFactEvent {
    pub(crate) fn new(fact: SliceFactInput) -> Self {
        Self {
            fact: Box::new(fact),
        }
    }

    /// The full-fidelity `ExportedEventBody` this slice fact carries, for the
    /// projection read path.
    pub(crate) fn to_event_body(&self) -> ExportedEventBody {
        self.fact.to_event_body()
    }
}

impl Serialize for SliceFactEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = SliceFactEventBody::from_slice_fact(self.fact.as_ref());
        let mut state = serializer.serialize_struct("SliceFactEvent", 1)?;
        state.serialize_field("body", &body.to_json_value())?;
        state.end()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceFactEventBody {
    body: ExportedEventBody,
}

impl SliceFactEventBody {
    fn from_slice_fact(fact: &SliceFactInput) -> Self {
        Self {
            body: fact.to_event_body(),
        }
    }

    fn from_tagged_json_value(value: &Value) -> Result<Self, String> {
        Ok(Self {
            body: ExportedEventBody::from_tagged_json_value(value)?,
        })
    }

    fn from_legacy_payload(event_type: ExportedEventType, payload: &Value) -> Result<Self, String> {
        Ok(Self {
            body: ExportedEventBody::from_event_type_and_payload(event_type, payload)?,
        })
    }

    fn into_slice_fact(self) -> Result<SliceFactInput, String> {
        SliceFactInput::from_event_body(&self.body)
    }

    fn to_json_value(&self) -> Value {
        self.body.tagged_json_value()
    }
}

#[derive(Deserialize)]
struct SerializedSliceFactEvent {
    body: Option<Value>,
    exported_event_type: Option<ExportedEventType>,
    payload: Option<Value>,
}

impl<'de> Deserialize<'de> for SliceFactEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serialized = SerializedSliceFactEvent::deserialize(deserializer)?;
        let body = serialized
            .slice_fact_body()
            .map_err(DeserializeError::custom)?;
        let fact = body.into_slice_fact().map_err(DeserializeError::custom)?;
        Ok(Self::new(fact))
    }
}

impl SerializedSliceFactEvent {
    fn slice_fact_body(self) -> Result<SliceFactEventBody, String> {
        match (self.body, self.exported_event_type, self.payload) {
            (Some(body), None, None) => SliceFactEventBody::from_tagged_json_value(&body),
            (None, Some(event_type), Some(payload)) => {
                SliceFactEventBody::from_legacy_payload(event_type, &payload)
            }
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => Err(
                "slice fact event must use either body or legacy exported_event_type/payload"
                    .to_owned(),
            ),
            (None, Some(_), None) => Err("missing slice fact payload".to_owned()),
            (None, None, Some(_)) => Err("missing slice fact exported event type".to_owned()),
            (None, None, None) => Err("missing slice fact event body".to_owned()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SliceFactInput {
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

    fn to_event_body(&self) -> ExportedEventBody {
        self.to_event_draft().body().clone()
    }

    pub(crate) fn from_event_body(body: &ExportedEventBody) -> Result<Self, String> {
        match body {
            ExportedEventBody::SliceScenarioAdded { scenario } => {
                Ok(Self::Scenario(scenario.clone()))
            }
            ExportedEventBody::SliceOutcomeAdded { outcome } => Ok(Self::Outcome(outcome.clone())),
            ExportedEventBody::SliceExternalPayloadAdded { external_payload } => {
                Ok(Self::ExternalPayload(external_payload.clone()))
            }
            ExportedEventBody::SliceEventDefinitionAdded { event } => {
                Ok(Self::EventDefinition(event.clone()))
            }
            ExportedEventBody::SliceCommandDefinitionAdded { command } => {
                Ok(Self::CommandDefinition(command.clone()))
            }
            ExportedEventBody::SliceReadModelAdded { read_model } => {
                Ok(Self::ReadModel(read_model.clone()))
            }
            ExportedEventBody::SliceViewAdded { view } => Ok(Self::View(view.clone())),
            ExportedEventBody::SliceBitLevelDataFlowAdded { data_flow } => {
                Ok(Self::BitLevelDataFlow(data_flow.clone()))
            }
            ExportedEventBody::SliceTranslationAdded { translation } => {
                Ok(Self::Translation(translation.clone()))
            }
            ExportedEventBody::SliceAutomationAdded { automation } => {
                Ok(Self::Automation(automation.clone()))
            }
            ExportedEventBody::SliceBoardElementAdded { element } => {
                Ok(Self::BoardElement(element.clone()))
            }
            ExportedEventBody::SliceBoardConnectionAdded { connection } => {
                Ok(Self::BoardConnection(connection.clone()))
            }
            body => Err(format!(
                "unsupported slice fact event type {}",
                body.event_type()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use serde_json::json;

    use super::*;
    use crate::core::types::{
        BoardElementDeclaredName, BoardElementKind, BoardElementName, BoardLaneId,
    };

    #[test]
    fn slice_fact_event_body_round_trips_current_tagged_json() -> Result<(), String> {
        let fact = board_element_fact()?;
        let body = SliceFactEventBody::from_slice_fact(&fact);

        let serialized = body.to_json_value();

        assert_eq!(
            serialized,
            json!({
                "SliceBoardElementAdded": {
                    "slice": "capture-ticket",
                    "name": "CaptureTicket",
                    "kind": "command",
                    "lane": "actions",
                    "declared_name": "CaptureTicket",
                    "main_path": true
                }
            })
        );
        assert_eq!(
            SliceFactEventBody::from_tagged_json_value(&serialized)?.into_slice_fact()?,
            fact
        );

        Ok(())
    }

    #[test]
    fn slice_fact_event_body_reads_legacy_event_type_and_payload() -> Result<(), String> {
        let legacy_payload = json!({
            "slice": "capture-ticket",
            "name": "CaptureTicket",
            "kind": "command",
            "lane": "actions",
            "declared_name": "CaptureTicket",
            "main_path": true
        });

        let decoded = SliceFactEventBody::from_legacy_payload(
            ExportedEventType::SliceBoardElementAdded,
            &legacy_payload,
        )?
        .into_slice_fact()?;

        assert_eq!(decoded, board_element_fact()?);

        Ok(())
    }

    fn board_element_fact() -> Result<SliceFactInput, String> {
        Ok(SliceFactInput::BoardElement(NewBoardElement::new(
            semantic("capture-ticket", SliceSlug::try_new)?,
            semantic("CaptureTicket", BoardElementName::try_new)?,
            semantic("command", BoardElementKind::try_new)?,
            semantic("actions", BoardLaneId::try_new)?,
            semantic("CaptureTicket", BoardElementDeclaredName::try_new)?,
            true,
        )))
    }

    fn semantic<T, Error>(
        value: &str,
        parse: impl FnOnce(String) -> Result<T, Error>,
    ) -> Result<T, String>
    where
        Error: Display,
    {
        parse(value.to_owned()).map_err(|error| error.to_string())
    }
}

#[derive(Command)]
pub(crate) struct RecordReviewCommand {
    #[stream]
    review_stream: StreamId,
    workflow: WorkflowSlug,
    model_content_digest: ModelContentDigest,
    reviewer_id: ReviewerId,
    reviewed_at: ReviewTimestamp,
    categories: Vec<ReviewRuleName>,
}

impl RecordReviewCommand {
    pub(crate) fn from_semantic(
        workflow: WorkflowSlug,
        model_content_digest: ModelContentDigest,
        reviewer_id: ReviewerId,
        reviewed_at: ReviewTimestamp,
        categories: Vec<ReviewRuleName>,
    ) -> Result<Self, String> {
        Ok(Self {
            review_stream: review_stream_id(workflow.as_ref())?,
            workflow,
            model_content_digest,
            reviewer_id,
            reviewed_at,
            categories,
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
            workflow: self.workflow.clone(),
            model_content_digest: self.model_content_digest.clone(),
            reviewer_id: self.reviewer_id.clone(),
            reviewed_at: self.reviewed_at.clone(),
            categories: self.categories.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct DeclareWorkflowReadinessCommand {
    #[stream]
    workflow_stream: StreamId,
    workflow: WorkflowSlug,
    projection_fingerprint: ProjectionFingerprint,
    model_content_digest: ModelContentDigest,
    verified_at: ReviewTimestamp,
    verified_by: ReviewerId,
    review_event: ReviewEventReference,
}

impl DeclareWorkflowReadinessCommand {
    pub(crate) fn new(
        workflow: WorkflowSlug,
        projection_fingerprint: ProjectionFingerprint,
        model_content_digest: ModelContentDigest,
        verified_at: ReviewTimestamp,
        verified_by: ReviewerId,
        review_event: ReviewEventReference,
    ) -> Result<Self, String> {
        Ok(Self {
            workflow_stream: workflow_stream_id(workflow.as_ref())?,
            workflow,
            projection_fingerprint,
            model_content_digest,
            verified_at,
            verified_by,
            review_event,
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
            workflow: self.workflow.clone(),
            projection_fingerprint: self.projection_fingerprint.clone(),
            model_content_digest: self.model_content_digest.clone(),
            verified_at: self.verified_at.clone(),
            verified_by: self.verified_by.clone(),
            review_event: self.review_event.clone(),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct ResolveConflictCommand {
    #[stream]
    project_stream: StreamId,
    conflict_id: EventConflictId,
    chosen_event_id: ChosenEventId,
}

impl ResolveConflictCommand {
    pub(crate) fn from_semantic(
        conflict_id: EventConflictId,
        chosen_event_id: ChosenEventId,
    ) -> Result<Self, String> {
        Ok(Self {
            project_stream: project_stream_id()?,
            conflict_id,
            chosen_event_id,
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
            conflict_id: self.conflict_id.clone(),
            chosen_event_id: self.chosen_event_id.clone(),
        }]
        .into())
    }
}
