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
    NewCommandDefinition, NewControlDefinition, NewEventDefinition, NewExternalPayloadDefinition,
    NewOutcomeDefinition, NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition,
};
use crate::core::project::ProjectName;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    AutomationName, BoardElementName, CommandErrorName, CommandName, ControlName,
    EventAttributeSourceField, EventAttributeSourceName, EventName, ModelDescription, ModelName,
    OutcomeLabelName, PayloadContractName, ReadModelName, ReviewRuleName, ReviewTimestamp,
    ReviewerId, ScenarioName, SliceKindName, SliceSlug, StreamName, TransitionTriggerName,
    TranslationName, ViewName, WorkflowCommandErrorRecord, WorkflowEntryLifecycleEvidenceText,
    WorkflowEntryLifecycleStateName, WorkflowEntryLifecycleStateRecord, WorkflowEventParticipation,
    WorkflowOutcomeRecord, WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName,
    WorkflowOwnedDefinitionRecord, WorkflowSlug, WorkflowTransitionEndpoint,
    WorkflowTransitionEvidenceRecord, WorkflowTransitionKind, WorkflowTransitionSourceEvidenceText,
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
    SliceAutomationDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        automation: SliceAutomationDefinitionUpdateEvent,
    },
    SliceAutomationDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        automation: SliceAutomationDefinitionRemovalEvent,
    },
    SliceBoardElementUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        element: SliceBoardElementUpdateEvent,
    },
    SliceBoardElementRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        element: SliceBoardElementRemovalEvent,
    },
    SliceTranslationDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        translation: SliceTranslationDefinitionUpdateEvent,
    },
    SliceTranslationDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        translation: SliceTranslationDefinitionRemovalEvent,
    },
    SliceExternalPayloadDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        external_payload: SliceExternalPayloadDefinitionUpdateEvent,
    },
    SliceExternalPayloadDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        external_payload: SliceExternalPayloadDefinitionRemovalEvent,
    },
    SliceOutcomeDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        outcome: SliceOutcomeDefinitionUpdateEvent,
    },
    SliceOutcomeDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        outcome: SliceOutcomeDefinitionRemovalEvent,
    },
    SliceCommandDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        command: SliceCommandDefinitionUpdateEvent,
    },
    SliceCommandDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        command: SliceCommandDefinitionRemovalEvent,
    },
    SliceEventDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        event: SliceEventDefinitionUpdateEvent,
    },
    SliceEventDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        event: SliceEventDefinitionRemovalEvent,
    },
    SliceReadModelDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        read_model: SliceReadModelDefinitionUpdateEvent,
    },
    SliceReadModelDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        read_model: SliceReadModelDefinitionRemovalEvent,
    },
    SliceViewDefinitionUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        view: SliceViewDefinitionUpdateEvent,
    },
    SliceViewDefinitionRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        view: SliceViewDefinitionRemovalEvent,
    },
    SliceViewControlUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        control: SliceViewControlUpdateEvent,
    },
    SliceViewControlRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        control: SliceViewControlRemovalEvent,
    },
    SliceScenarioUpdated {
        stream_id: StreamId,
        #[serde(flatten)]
        scenario: SliceScenarioUpdateEvent,
    },
    SliceScenarioRemoved {
        stream_id: StreamId,
        #[serde(flatten)]
        scenario: SliceScenarioRemovalEvent,
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
            Self::ProjectInitialized { stream_id, .. }
            | Self::WorkflowAdded { stream_id, .. }
            | Self::WorkflowUpdated { stream_id, .. }
            | Self::WorkflowRemoved { stream_id, .. }
            | Self::WorkflowOutcomeAdded { stream_id, .. }
            | Self::WorkflowCommandErrorAdded { stream_id, .. }
            | Self::WorkflowOwnedDefinitionAdded { stream_id, .. }
            | Self::WorkflowTransitionEvidenceAdded { stream_id, .. }
            | Self::WorkflowEntryLifecycleCoverageRequired { stream_id, .. }
            | Self::WorkflowEntryLifecycleStateAdded { stream_id, .. }
            | Self::WorkflowReadinessDeclared { stream_id, .. }
            | Self::WorkflowConnected { stream_id, .. }
            | Self::WorkflowTransitionRemoved { stream_id, .. }
            | Self::SliceAdded { stream_id, .. }
            | Self::SliceUpdated { stream_id, .. }
            | Self::SliceRemoved { stream_id, .. }
            | Self::SliceFactAdded { stream_id, .. }
            | Self::SliceAutomationDefinitionUpdated { stream_id, .. }
            | Self::SliceAutomationDefinitionRemoved { stream_id, .. }
            | Self::SliceBoardElementUpdated { stream_id, .. }
            | Self::SliceBoardElementRemoved { stream_id, .. }
            | Self::SliceTranslationDefinitionUpdated { stream_id, .. }
            | Self::SliceTranslationDefinitionRemoved { stream_id, .. }
            | Self::SliceExternalPayloadDefinitionUpdated { stream_id, .. }
            | Self::SliceExternalPayloadDefinitionRemoved { stream_id, .. }
            | Self::SliceOutcomeDefinitionUpdated { stream_id, .. }
            | Self::SliceOutcomeDefinitionRemoved { stream_id, .. }
            | Self::SliceCommandDefinitionUpdated { stream_id, .. }
            | Self::SliceCommandDefinitionRemoved { stream_id, .. }
            | Self::SliceEventDefinitionUpdated { stream_id, .. }
            | Self::SliceEventDefinitionRemoved { stream_id, .. }
            | Self::SliceReadModelDefinitionUpdated { stream_id, .. }
            | Self::SliceReadModelDefinitionRemoved { stream_id, .. }
            | Self::SliceViewDefinitionUpdated { stream_id, .. }
            | Self::SliceViewDefinitionRemoved { stream_id, .. }
            | Self::SliceViewControlUpdated { stream_id, .. }
            | Self::SliceViewControlRemoved { stream_id, .. }
            | Self::SliceScenarioUpdated { stream_id, .. }
            | Self::SliceScenarioRemoved { stream_id, .. }
            | Self::ReviewRecorded { stream_id, .. }
            | Self::ConflictResolved { stream_id, .. } => stream_id,
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

#[derive(Command)]
pub(crate) struct UpdateBoardElementCommand {
    #[stream]
    slice_stream: StreamId,
    element: NewBoardElement,
}

impl UpdateBoardElementCommand {
    pub(crate) fn new(element: NewBoardElement) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(element.slice_slug().as_ref())?,
            element,
        })
    }
}

impl CommandLogic for UpdateBoardElementCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating board element"
        );
        Ok(vec![EmcEvent::SliceBoardElementUpdated {
            stream_id: self.slice_stream.clone(),
            element: SliceBoardElementUpdateEvent::new(self.element.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveBoardElementCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: BoardElementName,
}

impl RemoveBoardElementCommand {
    pub(crate) fn new(slice: SliceSlug, name: BoardElementName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveBoardElementCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing board element"
        );
        Ok(vec![EmcEvent::SliceBoardElementRemoved {
            stream_id: self.slice_stream.clone(),
            element: SliceBoardElementRemovalEvent::new(self.slice.clone(), self.name.clone()),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceBoardElementUpdateEvent {
    element: NewBoardElement,
}

impl SliceBoardElementUpdateEvent {
    pub(crate) fn new(element: NewBoardElement) -> Self {
        Self { element }
    }

    pub(crate) fn element(&self) -> NewBoardElement {
        self.element.clone()
    }
}

impl Serialize for SliceBoardElementUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_board_element_updated(&self.element)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceBoardElementUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceBoardElementUpdated { element } => Ok(Self::new(element)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceBoardElementUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceBoardElementRemovalEvent {
    slice: SliceSlug,
    name: BoardElementName,
}

impl SliceBoardElementRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: BoardElementName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &BoardElementName {
        &self.name
    }
}

impl Serialize for SliceBoardElementRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_board_element_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceBoardElementRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceBoardElementRemoved { slice, name } => {
                Ok(Self::new(slice, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceBoardElementRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateCommandDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    command: NewCommandDefinition,
}

impl UpdateCommandDefinitionCommand {
    pub(crate) fn new(command: NewCommandDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(command.slice_slug().as_ref())?,
            command,
        })
    }
}

impl CommandLogic for UpdateCommandDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating command definition"
        );
        Ok(vec![EmcEvent::SliceCommandDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            command: SliceCommandDefinitionUpdateEvent::new(self.command.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveCommandDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: CommandName,
}

impl RemoveCommandDefinitionCommand {
    pub(crate) fn new(slice: SliceSlug, name: CommandName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveCommandDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing command definition"
        );
        Ok(vec![EmcEvent::SliceCommandDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            command: SliceCommandDefinitionRemovalEvent::new(self.slice.clone(), self.name.clone()),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceCommandDefinitionUpdateEvent {
    command: NewCommandDefinition,
}

impl SliceCommandDefinitionUpdateEvent {
    pub(crate) fn new(command: NewCommandDefinition) -> Self {
        Self { command }
    }

    pub(crate) fn command(&self) -> NewCommandDefinition {
        self.command.clone()
    }
}

impl Serialize for SliceCommandDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_command_definition_updated(&self.command)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceCommandDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceCommandDefinitionUpdated { command } => Ok(Self::new(command)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceCommandDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceCommandDefinitionRemovalEvent {
    slice: SliceSlug,
    name: CommandName,
}

impl SliceCommandDefinitionRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: CommandName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &CommandName {
        &self.name
    }
}

impl Serialize for SliceCommandDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_command_definition_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceCommandDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceCommandDefinitionRemoved { slice, name } => {
                Ok(Self::new(slice, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceCommandDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateAutomationDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    automation: NewAutomationDefinition,
}

impl UpdateAutomationDefinitionCommand {
    pub(crate) fn new(automation: NewAutomationDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(automation.slice_slug().as_ref())?,
            automation,
        })
    }
}

impl CommandLogic for UpdateAutomationDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating automation definition"
        );
        Ok(vec![EmcEvent::SliceAutomationDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            automation: SliceAutomationDefinitionUpdateEvent::new(self.automation.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveAutomationDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: AutomationName,
}

impl RemoveAutomationDefinitionCommand {
    pub(crate) fn new(slice: SliceSlug, name: AutomationName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveAutomationDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing automation definition"
        );
        Ok(vec![EmcEvent::SliceAutomationDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            automation: SliceAutomationDefinitionRemovalEvent::new(
                self.slice.clone(),
                self.name.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceAutomationDefinitionUpdateEvent {
    automation: NewAutomationDefinition,
}

impl SliceAutomationDefinitionUpdateEvent {
    pub(crate) fn new(automation: NewAutomationDefinition) -> Self {
        Self { automation }
    }

    pub(crate) fn automation(&self) -> NewAutomationDefinition {
        self.automation.clone()
    }
}

impl Serialize for SliceAutomationDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_automation_definition_updated(&self.automation)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceAutomationDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceAutomationDefinitionUpdated { automation } => {
                Ok(Self::new(automation))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceAutomationDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceAutomationDefinitionRemovalEvent {
    slice: SliceSlug,
    name: AutomationName,
}

impl SliceAutomationDefinitionRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: AutomationName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &AutomationName {
        &self.name
    }
}

impl Serialize for SliceAutomationDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_automation_definition_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceAutomationDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceAutomationDefinitionRemoved { slice, name } => {
                Ok(Self::new(slice, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceAutomationDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateTranslationDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    translation: NewTranslationDefinition,
}

impl UpdateTranslationDefinitionCommand {
    pub(crate) fn new(translation: NewTranslationDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(translation.slice_slug().as_ref())?,
            translation,
        })
    }
}

impl CommandLogic for UpdateTranslationDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating translation definition"
        );
        Ok(vec![EmcEvent::SliceTranslationDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            translation: SliceTranslationDefinitionUpdateEvent::new(self.translation.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveTranslationDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: TranslationName,
}

impl RemoveTranslationDefinitionCommand {
    pub(crate) fn new(slice: SliceSlug, name: TranslationName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveTranslationDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing translation definition"
        );
        Ok(vec![EmcEvent::SliceTranslationDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            translation: SliceTranslationDefinitionRemovalEvent::new(
                self.slice.clone(),
                self.name.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceTranslationDefinitionUpdateEvent {
    translation: NewTranslationDefinition,
}

impl SliceTranslationDefinitionUpdateEvent {
    pub(crate) fn new(translation: NewTranslationDefinition) -> Self {
        Self { translation }
    }

    pub(crate) fn translation(&self) -> NewTranslationDefinition {
        self.translation.clone()
    }
}

impl Serialize for SliceTranslationDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_translation_definition_updated(&self.translation)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceTranslationDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceTranslationDefinitionUpdated { translation } => {
                Ok(Self::new(translation))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceTranslationDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceTranslationDefinitionRemovalEvent {
    slice: SliceSlug,
    name: TranslationName,
}

impl SliceTranslationDefinitionRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: TranslationName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &TranslationName {
        &self.name
    }
}

impl Serialize for SliceTranslationDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_translation_definition_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceTranslationDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceTranslationDefinitionRemoved { slice, name } => {
                Ok(Self::new(slice, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceTranslationDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateExternalPayloadDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    external_payload: NewExternalPayloadDefinition,
}

impl UpdateExternalPayloadDefinitionCommand {
    pub(crate) fn new(external_payload: NewExternalPayloadDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(external_payload.slice_slug().as_ref())?,
            external_payload,
        })
    }
}

impl CommandLogic for UpdateExternalPayloadDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating external payload definition"
        );
        Ok(vec![EmcEvent::SliceExternalPayloadDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            external_payload: SliceExternalPayloadDefinitionUpdateEvent::new(
                self.external_payload.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveExternalPayloadDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: EventAttributeSourceName,
    field: EventAttributeSourceField,
}

impl RemoveExternalPayloadDefinitionCommand {
    pub(crate) fn new(
        slice: SliceSlug,
        name: EventAttributeSourceName,
        field: EventAttributeSourceField,
    ) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
            field,
        })
    }
}

impl CommandLogic for RemoveExternalPayloadDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing external payload definition"
        );
        Ok(vec![EmcEvent::SliceExternalPayloadDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            external_payload: SliceExternalPayloadDefinitionRemovalEvent::new(
                self.slice.clone(),
                self.name.clone(),
                self.field.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceExternalPayloadDefinitionUpdateEvent {
    external_payload: NewExternalPayloadDefinition,
}

impl SliceExternalPayloadDefinitionUpdateEvent {
    pub(crate) fn new(external_payload: NewExternalPayloadDefinition) -> Self {
        Self { external_payload }
    }

    pub(crate) fn external_payload(&self) -> NewExternalPayloadDefinition {
        self.external_payload.clone()
    }
}

impl Serialize for SliceExternalPayloadDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_external_payload_definition_updated(&self.external_payload)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceExternalPayloadDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceExternalPayloadDefinitionUpdated { external_payload } => {
                Ok(Self::new(external_payload))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceExternalPayloadDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceExternalPayloadDefinitionRemovalEvent {
    slice: SliceSlug,
    name: EventAttributeSourceName,
    field: EventAttributeSourceField,
}

impl SliceExternalPayloadDefinitionRemovalEvent {
    pub(crate) fn new(
        slice: SliceSlug,
        name: EventAttributeSourceName,
        field: EventAttributeSourceField,
    ) -> Self {
        Self { slice, name, field }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &EventAttributeSourceName {
        &self.name
    }

    pub(crate) fn field(&self) -> &EventAttributeSourceField {
        &self.field
    }
}

impl Serialize for SliceExternalPayloadDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_external_payload_definition_removed(
            &self.slice,
            &self.name,
            &self.field,
        )
        .body()
        .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceExternalPayloadDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceExternalPayloadDefinitionRemoved { slice, name, field } => {
                Ok(Self::new(slice, name, field))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceExternalPayloadDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateOutcomeDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    outcome: NewOutcomeDefinition,
}

impl UpdateOutcomeDefinitionCommand {
    pub(crate) fn new(outcome: NewOutcomeDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(outcome.slice_slug().as_ref())?,
            outcome,
        })
    }
}

impl CommandLogic for UpdateOutcomeDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating outcome definition"
        );
        Ok(vec![EmcEvent::SliceOutcomeDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            outcome: SliceOutcomeDefinitionUpdateEvent::new(self.outcome.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveOutcomeDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    label: OutcomeLabelName,
}

impl RemoveOutcomeDefinitionCommand {
    pub(crate) fn new(slice: SliceSlug, label: OutcomeLabelName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            label,
        })
    }
}

impl CommandLogic for RemoveOutcomeDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing outcome definition"
        );
        Ok(vec![EmcEvent::SliceOutcomeDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            outcome: SliceOutcomeDefinitionRemovalEvent::new(
                self.slice.clone(),
                self.label.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceOutcomeDefinitionUpdateEvent {
    outcome: NewOutcomeDefinition,
}

impl SliceOutcomeDefinitionUpdateEvent {
    pub(crate) fn new(outcome: NewOutcomeDefinition) -> Self {
        Self { outcome }
    }

    pub(crate) fn outcome(&self) -> NewOutcomeDefinition {
        self.outcome.clone()
    }
}

impl Serialize for SliceOutcomeDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_outcome_definition_updated(&self.outcome)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceOutcomeDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceOutcomeDefinitionUpdated { outcome } => Ok(Self::new(outcome)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceOutcomeDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceOutcomeDefinitionRemovalEvent {
    slice: SliceSlug,
    label: OutcomeLabelName,
}

impl SliceOutcomeDefinitionRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, label: OutcomeLabelName) -> Self {
        Self { slice, label }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn label(&self) -> &OutcomeLabelName {
        &self.label
    }
}

impl Serialize for SliceOutcomeDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_outcome_definition_removed(&self.slice, &self.label)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceOutcomeDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceOutcomeDefinitionRemoved { slice, label } => {
                Ok(Self::new(slice, label))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceOutcomeDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateEventDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    event: NewEventDefinition,
}

impl UpdateEventDefinitionCommand {
    pub(crate) fn new(event: NewEventDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(event.slice_slug().as_ref())?,
            event,
        })
    }
}

impl CommandLogic for UpdateEventDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating event definition"
        );
        Ok(vec![EmcEvent::SliceEventDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            event: SliceEventDefinitionUpdateEvent::new(self.event.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveEventDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: EventName,
}

impl RemoveEventDefinitionCommand {
    pub(crate) fn new(slice: SliceSlug, name: EventName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveEventDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing event definition"
        );
        Ok(vec![EmcEvent::SliceEventDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            event: SliceEventDefinitionRemovalEvent::new(self.slice.clone(), self.name.clone()),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceEventDefinitionUpdateEvent {
    event: NewEventDefinition,
}

impl SliceEventDefinitionUpdateEvent {
    pub(crate) fn new(event: NewEventDefinition) -> Self {
        Self { event }
    }

    pub(crate) fn event(&self) -> NewEventDefinition {
        self.event.clone()
    }
}

impl Serialize for SliceEventDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_event_definition_updated(&self.event)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceEventDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceEventDefinitionUpdated { event } => Ok(Self::new(event)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceEventDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceEventDefinitionRemovalEvent {
    slice: SliceSlug,
    name: EventName,
}

impl SliceEventDefinitionRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: EventName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &EventName {
        &self.name
    }
}

impl Serialize for SliceEventDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_event_definition_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceEventDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceEventDefinitionRemoved { slice, name } => {
                Ok(Self::new(slice, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceEventDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateReadModelDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    read_model: NewReadModelDefinition,
}

impl UpdateReadModelDefinitionCommand {
    pub(crate) fn new(read_model: NewReadModelDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(read_model.slice_slug().as_ref())?,
            read_model,
        })
    }
}

impl CommandLogic for UpdateReadModelDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating read model definition"
        );
        Ok(vec![EmcEvent::SliceReadModelDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            read_model: SliceReadModelDefinitionUpdateEvent::new(self.read_model.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveReadModelDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: ReadModelName,
}

impl RemoveReadModelDefinitionCommand {
    pub(crate) fn new(slice: SliceSlug, name: ReadModelName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveReadModelDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing read model definition"
        );
        Ok(vec![EmcEvent::SliceReadModelDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            read_model: SliceReadModelDefinitionRemovalEvent::new(
                self.slice.clone(),
                self.name.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceReadModelDefinitionUpdateEvent {
    read_model: NewReadModelDefinition,
}

impl SliceReadModelDefinitionUpdateEvent {
    pub(crate) fn new(read_model: NewReadModelDefinition) -> Self {
        Self { read_model }
    }

    pub(crate) fn read_model(&self) -> NewReadModelDefinition {
        self.read_model.clone()
    }
}

impl Serialize for SliceReadModelDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_read_model_definition_updated(&self.read_model)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceReadModelDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceReadModelDefinitionUpdated { read_model } => {
                Ok(Self::new(read_model))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceReadModelDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceReadModelDefinitionRemovalEvent {
    slice: SliceSlug,
    name: ReadModelName,
}

impl SliceReadModelDefinitionRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: ReadModelName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &ReadModelName {
        &self.name
    }
}

impl Serialize for SliceReadModelDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_read_model_definition_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceReadModelDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceReadModelDefinitionRemoved { slice, name } => {
                Ok(Self::new(slice, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceReadModelDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateViewDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    view: NewViewDefinition,
}

impl UpdateViewDefinitionCommand {
    pub(crate) fn new(view: NewViewDefinition) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(view.slice_slug().as_ref())?,
            view,
        })
    }
}

impl CommandLogic for UpdateViewDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating view definition"
        );
        Ok(vec![EmcEvent::SliceViewDefinitionUpdated {
            stream_id: self.slice_stream.clone(),
            view: SliceViewDefinitionUpdateEvent::new(self.view.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveViewDefinitionCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: ViewName,
}

impl RemoveViewDefinitionCommand {
    pub(crate) fn new(slice: SliceSlug, name: ViewName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveViewDefinitionCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing view definition"
        );
        Ok(vec![EmcEvent::SliceViewDefinitionRemoved {
            stream_id: self.slice_stream.clone(),
            view: SliceViewDefinitionRemovalEvent::new(self.slice.clone(), self.name.clone()),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceViewDefinitionUpdateEvent {
    view: NewViewDefinition,
}

impl SliceViewDefinitionUpdateEvent {
    pub(crate) fn new(view: NewViewDefinition) -> Self {
        Self { view }
    }

    pub(crate) fn view(&self) -> NewViewDefinition {
        self.view.clone()
    }
}

impl Serialize for SliceViewDefinitionUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_view_definition_updated(&self.view)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceViewDefinitionUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceViewDefinitionUpdated { view } => Ok(Self::new(view)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceViewDefinitionUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceViewDefinitionRemovalEvent {
    slice: SliceSlug,
    name: ViewName,
}

impl SliceViewDefinitionRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: ViewName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &ViewName {
        &self.name
    }
}

impl Serialize for SliceViewDefinitionRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_view_definition_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceViewDefinitionRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceViewDefinitionRemoved { slice, name } => {
                Ok(Self::new(slice, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceViewDefinitionRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateViewControlCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    view: ViewName,
    control: NewControlDefinition,
}

impl UpdateViewControlCommand {
    pub(crate) fn new(
        slice: SliceSlug,
        view: ViewName,
        control: NewControlDefinition,
    ) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            view,
            control,
        })
    }
}

impl CommandLogic for UpdateViewControlCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating view control"
        );
        Ok(vec![EmcEvent::SliceViewControlUpdated {
            stream_id: self.slice_stream.clone(),
            control: SliceViewControlUpdateEvent::new(
                self.slice.clone(),
                self.view.clone(),
                self.control.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveViewControlCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    view: ViewName,
    name: ControlName,
}

impl RemoveViewControlCommand {
    pub(crate) fn new(slice: SliceSlug, view: ViewName, name: ControlName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            view,
            name,
        })
    }
}

impl CommandLogic for RemoveViewControlCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing view control"
        );
        Ok(vec![EmcEvent::SliceViewControlRemoved {
            stream_id: self.slice_stream.clone(),
            control: SliceViewControlRemovalEvent::new(
                self.slice.clone(),
                self.view.clone(),
                self.name.clone(),
            ),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceViewControlUpdateEvent {
    slice: SliceSlug,
    view: ViewName,
    control: NewControlDefinition,
}

impl SliceViewControlUpdateEvent {
    pub(crate) fn new(slice: SliceSlug, view: ViewName, control: NewControlDefinition) -> Self {
        Self {
            slice,
            view,
            control,
        }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn view(&self) -> &ViewName {
        &self.view
    }

    pub(crate) fn control(&self) -> NewControlDefinition {
        self.control.clone()
    }
}

impl Serialize for SliceViewControlUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_view_control_updated(&self.slice, &self.view, &self.control)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceViewControlUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceViewControlUpdated {
                slice,
                view,
                control,
            } => Ok(Self::new(slice, view, control)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceViewControlUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceViewControlRemovalEvent {
    slice: SliceSlug,
    view: ViewName,
    name: ControlName,
}

impl SliceViewControlRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, view: ViewName, name: ControlName) -> Self {
        Self { slice, view, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn view(&self) -> &ViewName {
        &self.view
    }

    pub(crate) fn name(&self) -> &ControlName {
        &self.name
    }
}

impl Serialize for SliceViewControlRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_view_control_removed(&self.slice, &self.view, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceViewControlRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceViewControlRemoved { slice, view, name } => {
                Ok(Self::new(slice, view, name))
            }
            other => Err(DeserializeError::custom(format!(
                "expected SliceViewControlRemoved event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Command)]
pub(crate) struct UpdateSliceScenarioCommand {
    #[stream]
    slice_stream: StreamId,
    scenario: NewSliceScenario,
}

impl UpdateSliceScenarioCommand {
    pub(crate) fn new(scenario: NewSliceScenario) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(scenario.slice_slug().as_ref())?,
            scenario,
        })
    }
}

impl CommandLogic for UpdateSliceScenarioCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before updating scenario"
        );
        Ok(vec![EmcEvent::SliceScenarioUpdated {
            stream_id: self.slice_stream.clone(),
            scenario: SliceScenarioUpdateEvent::new(self.scenario.clone()),
        }]
        .into())
    }
}

#[derive(Command)]
pub(crate) struct RemoveSliceScenarioCommand {
    #[stream]
    slice_stream: StreamId,
    slice: SliceSlug,
    name: ScenarioName,
}

impl RemoveSliceScenarioCommand {
    pub(crate) fn new(slice: SliceSlug, name: ScenarioName) -> Result<Self, String> {
        Ok(Self {
            slice_stream: slice_stream_id(slice.as_ref())?,
            slice,
            name,
        })
    }
}

impl CommandLogic for RemoveSliceScenarioCommand {
    type Event = EmcEvent;
    type State = SliceCommandState;

    fn apply(&self, state: Self::State, event: &Self::Event) -> Self::State {
        apply_slice_command_state(state, event)
    }

    fn handle(&self, state: Self::State) -> Result<NewEvents<Self::Event>, CommandError> {
        require!(
            state.added,
            "slice stream must exist before removing scenario"
        );
        Ok(vec![EmcEvent::SliceScenarioRemoved {
            stream_id: self.slice_stream.clone(),
            scenario: SliceScenarioRemovalEvent::new(self.slice.clone(), self.name.clone()),
        }]
        .into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceScenarioUpdateEvent {
    scenario: NewSliceScenario,
}

impl SliceScenarioUpdateEvent {
    pub(crate) fn new(scenario: NewSliceScenario) -> Self {
        Self { scenario }
    }

    pub(crate) fn scenario(&self) -> NewSliceScenario {
        self.scenario.clone()
    }
}

impl Serialize for SliceScenarioUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_scenario_updated(&self.scenario)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceScenarioUpdateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceScenarioUpdated { scenario } => Ok(Self::new(scenario)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceScenarioUpdated event body, got {}",
                other.event_type()
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceScenarioRemovalEvent {
    slice: SliceSlug,
    name: ScenarioName,
}

impl SliceScenarioRemovalEvent {
    pub(crate) fn new(slice: SliceSlug, name: ScenarioName) -> Self {
        Self { slice, name }
    }

    pub(crate) fn slice(&self) -> &SliceSlug {
        &self.slice
    }

    pub(crate) fn name(&self) -> &ScenarioName {
        &self.name
    }
}

impl Serialize for SliceScenarioRemovalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let body = EventDraft::slice_scenario_removed(&self.slice, &self.name)
            .body()
            .clone();
        serialize_event_body(serializer, &body)
    }
}

impl<'de> Deserialize<'de> for SliceScenarioRemovalEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_event_body(deserializer)? {
            ExportedEventBody::SliceScenarioRemoved { slice, name } => Ok(Self::new(slice, name)),
            other => Err(DeserializeError::custom(format!(
                "expected SliceScenarioRemoved event body, got {}",
                other.event_type()
            ))),
        }
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
        serialize_tagged_event_body(serializer, &body.to_json_value())
    }
}

fn serialize_event_body<S>(serializer: S, body: &ExportedEventBody) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serialize_tagged_event_body(serializer, &body.tagged_json_value())
}

fn serialize_tagged_event_body<S>(serializer: S, body: &Value) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("EventBody", 1)?;
    state.serialize_field("body", body)?;
    state.end()
}

fn deserialize_event_body<'de, D>(deserializer: D) -> Result<ExportedEventBody, D::Error>
where
    D: Deserializer<'de>,
{
    let serialized = SerializedSliceFactEvent::deserialize(deserializer)?;
    serialized
        .slice_fact_body()
        .map(SliceFactEventBody::into_body)
        .map_err(DeserializeError::custom)
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

    fn into_body(self) -> ExportedEventBody {
        self.body
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
            semantic("command", |value| BoardElementKind::try_new(&value))?,
            semantic("actions", |value| BoardLaneId::try_new(&value))?,
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
