// Copyright 2026 John Wilger

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::core::connection::{ConnectionKind, WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
use crate::core::effect::{
    ArtifactDigest, ChosenEventId, Effect, EffectPlan, EventConflictId, FileContents,
    ModelContentDigest, ProjectPath, ProjectionFingerprint, ReportLine, ReviewEventId,
    ReviewEventReference,
};
use crate::core::emit::lean::{
    emit_slice_module as emit_lean_slice_module, emit_workflow_module as emit_lean_workflow_module,
};
use crate::core::emit::quint::{
    emit_slice_module as emit_quint_slice_module,
    emit_workflow_module as emit_quint_workflow_module,
};
use crate::core::formal_slice_facts::{
    CommandErrorDefinitions, CommandErrorNames, CommandInputProvenanceChain, CommandInputSource,
    CommandObservedStreams, EmittedEventNames, NewAutomationDefinition, NewBitLevelDataFlow,
    NewBoardConnection, NewBoardElement, NewCommandDefinition, NewCommandErrorDefinition,
    NewCommandInput, NewControlDefinition, NewControlInputProvision, NewEventAttribute,
    NewEventDefinition, NewExternalPayloadDefinition, NewNavigationTarget, NewOutcomeDefinition,
    NewReadModelDefinition, NewReadModelField, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition, NewViewField, OutcomeEventNames, ReadModelDerivationSourceFields,
    ReadModelFieldSource, ReadModelRelationshipFields, ScenarioKind, ScenarioStreamNames,
    ViewControls, ViewFilters, ViewLocalStates,
};
use crate::core::project::{ProjectName, ProjectSliceMembership, project_root_effects};
use crate::core::slice::NewSlice;
use crate::core::types::{
    AutomationName, AutomationReactionDescription, AutomationTriggerName, BitEncodingSemantics,
    BoardConnectionEndpoint, BoardConnectionEndpointKind, BoardElementDeclaredName,
    BoardElementKind, BoardElementName, BoardLaneId, CommandErrorName, CommandErrorRecoveryKind,
    CommandInputSourceDescription, CommandInputSourceKind, CommandName, ContractKindName,
    ControlName, ControlRecoveryBehavior, CoveredDefinitionName, DataFlowSource,
    DataFlowSourceKind, DataFlowTarget, DatumName, EventAttributeName, EventAttributeSourceField,
    EventAttributeSourceKind, EventAttributeSourceName, EventName,
    GeneratedEventAttributeSourceKind, LeanModuleName, ModelDescription, ModelName,
    NavigationTargetName, NavigationTargetType, OutcomeLabelName, PayloadContractName,
    ProvenanceDescription, QuintModuleName, ReadModelDerivationRule, ReadModelFieldSourceKind,
    ReadModelName, ReadModelTransitiveRule, ReviewRuleName, ReviewTimestamp, ReviewerId,
    ScenarioName, ScenarioStepText, SingletonRepeatBehavior, SketchToken, SliceKindName, SliceSlug,
    SourceChainHop, StreamName, TransformationSemantics, TransitionTriggerName,
    TranslationExternalEventName, TranslationName, ViewFieldName, ViewFieldSourceKind, ViewName,
    WorkflowCommandErrorRecord, WorkflowCommandErrorRecords, WorkflowEntryLifecycleEvidenceText,
    WorkflowEntryLifecycleStateName, WorkflowEntryLifecycleStateRecord,
    WorkflowEntryLifecycleStateRecords, WorkflowEventParticipation, WorkflowModuleData,
    WorkflowOutcomeRecord, WorkflowOutcomeRecords, WorkflowOwnedDefinitionKind,
    WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord, WorkflowOwnedDefinitionRecords,
    WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug, WorkflowStepRelationshipName,
    WorkflowTransitionEndpoint, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionKind, WorkflowTransitionRecord,
    WorkflowTransitionRecords, WorkflowTransitionSourceEvidenceText,
    WorkflowTransitionTargetEvidenceText, WorkflowViewRole,
};
use crate::core::workflow::NewWorkflow;

const SCHEMA_VERSION: &str = "emc.events.v1";
const EVENT_EXPORT_DIRECTORY: &str = "model/events/v1";
const PROJECTION_FINGERPRINT_PATH: &str = "model/events/projection.fingerprint";

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct EventDraft {
    stream_id: EventStreamId,
    body: ExportedEventBody,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExportedEventMeta {
    event_id: ExportedEventId,
    command_id: ExportedCommandId,
    command_ordinal: usize,
    stream_id: EventStreamId,
    parents: Vec<ExportedEventId>,
}

impl ExportedEventMeta {
    fn from_draft(
        draft: &EventDraft,
        parents: Vec<ExportedEventId>,
        command_ordinal: usize,
    ) -> Result<Self, String> {
        let event_id = event_id(draft, &parents, command_ordinal)?;
        let command_id = ExportedCommandId::from_event_id(&event_id);
        Ok(Self {
            event_id,
            command_id,
            command_ordinal,
            stream_id: draft.stream_id.clone(),
            parents,
        })
    }

    fn from_json_value(value: &Value) -> Result<Self, String> {
        Ok(Self {
            event_id: required_exported_event_id(value, "event_id")?,
            command_id: required_exported_command_id(value, "command_id")?,
            command_ordinal: required_usize(value, "command_ordinal")?,
            stream_id: required_event_stream_id(value, "stream_id")?,
            parents: parents(value)?,
        })
    }

    fn event_id(&self) -> &ExportedEventId {
        &self.event_id
    }

    fn stream_id(&self) -> &EventStreamId {
        &self.stream_id
    }

    fn parents(&self) -> &[ExportedEventId] {
        &self.parents
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ExportedEventId {
    value: ArtifactDigest,
}

impl ExportedEventId {
    fn try_new(value: String) -> Result<Self, String> {
        ArtifactDigest::try_new(value)
            .map(|value| Self { value })
            .map_err(|error| error.to_string())
    }
}

impl AsRef<str> for ExportedEventId {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl Ord for ExportedEventId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl PartialOrd for ExportedEventId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExportedCommandId {
    value: ArtifactDigest,
}

impl ExportedCommandId {
    fn try_new(value: String) -> Result<Self, String> {
        ArtifactDigest::try_new(value)
            .map(|value| Self { value })
            .map_err(|error| error.to_string())
    }

    fn from_event_id(event_id: &ExportedEventId) -> Self {
        Self {
            value: ArtifactDigest::try_new(event_id.as_ref().to_owned()).unwrap_or_else(|error| {
                unreachable!("exported event id must be a valid command id: {error}");
            }),
        }
    }
}

impl AsRef<str> for ExportedCommandId {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct EventStreamId {
    value: String,
}

impl EventStreamId {
    pub(crate) fn project() -> Self {
        Self {
            value: "project".to_owned(),
        }
    }

    pub(crate) fn workflow(slug: &WorkflowSlug) -> Self {
        Self {
            value: format!("workflow::{}", slug.as_ref()),
        }
    }

    pub(crate) fn slice(slug: &SliceSlug) -> Self {
        Self {
            value: format!("slice::{}", slug.as_ref()),
        }
    }

    pub(crate) fn review(workflow: &WorkflowSlug) -> Self {
        Self {
            value: format!("review::{}", workflow.as_ref()),
        }
    }

    pub(crate) fn try_new(value: String) -> Result<Self, EventStreamIdError> {
        let trimmed = value.trim().to_owned();
        let trimmed = trimmed.as_str();
        if trimmed == "project" {
            return Ok(Self::project());
        }

        if let Some(slug) = trimmed.strip_prefix("workflow::") {
            return Self::workflow_stream(value, slug);
        }

        if let Some(slug) = trimmed.strip_prefix("slice::") {
            return Self::slice_stream(value, slug);
        }

        if let Some(workflow) = trimmed.strip_prefix("review::") {
            return Self::review_stream(value, workflow);
        }

        Err(EventStreamIdError::new(value))
    }

    fn workflow_stream(value: String, slug: &str) -> Result<Self, EventStreamIdError> {
        if slug.contains("::") {
            return Err(EventStreamIdError::new(value));
        }

        WorkflowSlug::try_new(slug.to_owned())
            .map(|slug| Self::workflow(&slug))
            .map_err(|_error| EventStreamIdError::new(value))
    }

    fn slice_stream(value: String, slug: &str) -> Result<Self, EventStreamIdError> {
        if slug.contains("::") {
            return Err(EventStreamIdError::new(value));
        }

        SliceSlug::try_new(slug.to_owned())
            .map(|slug| Self::slice(&slug))
            .map_err(|_error| EventStreamIdError::new(value))
    }

    fn review_stream(value: String, workflow: &str) -> Result<Self, EventStreamIdError> {
        if workflow.contains("::") {
            return Err(EventStreamIdError::new(value));
        }

        WorkflowSlug::try_new(workflow.to_owned())
            .map(|workflow| Self::review(&workflow))
            .map_err(|_error| EventStreamIdError::new(value))
    }
}

impl AsRef<str> for EventStreamId {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl Display for EventStreamId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct EventStreamIdError {
    message: String,
}

impl EventStreamIdError {
    fn new(value: String) -> Self {
        Self {
            message: format!(
                "expected a project, workflow, slice, or review event stream id, got '{value}'"
            ),
        }
    }
}

impl Display for EventStreamIdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for EventStreamIdError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum ExportedEventType {
    ProjectInitialized,
    WorkflowAdded,
    WorkflowUpdated,
    WorkflowRemoved,
    WorkflowOutcomeAdded,
    WorkflowCommandErrorAdded,
    WorkflowOwnedDefinitionAdded,
    WorkflowTransitionEvidenceAdded,
    WorkflowEntryLifecycleCoverageRequired,
    WorkflowEntryLifecycleStateAdded,
    WorkflowReadinessDeclared,
    WorkflowConnected,
    WorkflowTransitionRemoved,
    SliceAdded,
    SliceUpdated,
    SliceRemoved,
    SliceScenarioAdded,
    SliceOutcomeAdded,
    SliceExternalPayloadAdded,
    SliceEventDefinitionAdded,
    SliceCommandDefinitionAdded,
    SliceReadModelAdded,
    SliceViewAdded,
    SliceBitLevelDataFlowAdded,
    SliceTranslationAdded,
    SliceAutomationAdded,
    SliceBoardElementAdded,
    SliceBoardConnectionAdded,
    ReviewRecorded,
    ConflictResolved,
}

impl ExportedEventType {
    pub(crate) fn try_new(value: String) -> Result<Self, ExportedEventTypeError> {
        match value.trim() {
            "ProjectInitialized" => Ok(Self::ProjectInitialized),
            "WorkflowAdded" => Ok(Self::WorkflowAdded),
            "WorkflowUpdated" => Ok(Self::WorkflowUpdated),
            "WorkflowRemoved" => Ok(Self::WorkflowRemoved),
            "WorkflowOutcomeAdded" => Ok(Self::WorkflowOutcomeAdded),
            "WorkflowCommandErrorAdded" => Ok(Self::WorkflowCommandErrorAdded),
            "WorkflowOwnedDefinitionAdded" => Ok(Self::WorkflowOwnedDefinitionAdded),
            "WorkflowTransitionEvidenceAdded" => Ok(Self::WorkflowTransitionEvidenceAdded),
            "WorkflowEntryLifecycleCoverageRequired" => {
                Ok(Self::WorkflowEntryLifecycleCoverageRequired)
            }
            "WorkflowEntryLifecycleStateAdded" => Ok(Self::WorkflowEntryLifecycleStateAdded),
            "WorkflowReadinessDeclared" => Ok(Self::WorkflowReadinessDeclared),
            "WorkflowConnected" => Ok(Self::WorkflowConnected),
            "WorkflowTransitionRemoved" => Ok(Self::WorkflowTransitionRemoved),
            "SliceAdded" => Ok(Self::SliceAdded),
            "SliceUpdated" => Ok(Self::SliceUpdated),
            "SliceRemoved" => Ok(Self::SliceRemoved),
            "SliceScenarioAdded" => Ok(Self::SliceScenarioAdded),
            "SliceOutcomeAdded" => Ok(Self::SliceOutcomeAdded),
            "SliceExternalPayloadAdded" => Ok(Self::SliceExternalPayloadAdded),
            "SliceEventDefinitionAdded" => Ok(Self::SliceEventDefinitionAdded),
            "SliceCommandDefinitionAdded" => Ok(Self::SliceCommandDefinitionAdded),
            "SliceReadModelAdded" => Ok(Self::SliceReadModelAdded),
            "SliceViewAdded" => Ok(Self::SliceViewAdded),
            "SliceBitLevelDataFlowAdded" => Ok(Self::SliceBitLevelDataFlowAdded),
            "SliceTranslationAdded" => Ok(Self::SliceTranslationAdded),
            "SliceAutomationAdded" => Ok(Self::SliceAutomationAdded),
            "SliceBoardElementAdded" => Ok(Self::SliceBoardElementAdded),
            "SliceBoardConnectionAdded" => Ok(Self::SliceBoardConnectionAdded),
            "ReviewRecorded" => Ok(Self::ReviewRecorded),
            "ConflictResolved" => Ok(Self::ConflictResolved),
            _ => Err(ExportedEventTypeError::new(value)),
        }
    }

    pub(crate) fn is_slice_fact(self) -> bool {
        matches!(
            self,
            Self::SliceScenarioAdded
                | Self::SliceOutcomeAdded
                | Self::SliceExternalPayloadAdded
                | Self::SliceEventDefinitionAdded
                | Self::SliceCommandDefinitionAdded
                | Self::SliceReadModelAdded
                | Self::SliceViewAdded
                | Self::SliceBitLevelDataFlowAdded
                | Self::SliceTranslationAdded
                | Self::SliceAutomationAdded
                | Self::SliceBoardElementAdded
                | Self::SliceBoardConnectionAdded
        )
    }
}

impl AsRef<str> for ExportedEventType {
    fn as_ref(&self) -> &str {
        match self {
            Self::ProjectInitialized => "ProjectInitialized",
            Self::WorkflowAdded => "WorkflowAdded",
            Self::WorkflowUpdated => "WorkflowUpdated",
            Self::WorkflowRemoved => "WorkflowRemoved",
            Self::WorkflowOutcomeAdded => "WorkflowOutcomeAdded",
            Self::WorkflowCommandErrorAdded => "WorkflowCommandErrorAdded",
            Self::WorkflowOwnedDefinitionAdded => "WorkflowOwnedDefinitionAdded",
            Self::WorkflowTransitionEvidenceAdded => "WorkflowTransitionEvidenceAdded",
            Self::WorkflowEntryLifecycleCoverageRequired => {
                "WorkflowEntryLifecycleCoverageRequired"
            }
            Self::WorkflowEntryLifecycleStateAdded => "WorkflowEntryLifecycleStateAdded",
            Self::WorkflowReadinessDeclared => "WorkflowReadinessDeclared",
            Self::WorkflowConnected => "WorkflowConnected",
            Self::WorkflowTransitionRemoved => "WorkflowTransitionRemoved",
            Self::SliceAdded => "SliceAdded",
            Self::SliceUpdated => "SliceUpdated",
            Self::SliceRemoved => "SliceRemoved",
            Self::SliceScenarioAdded => "SliceScenarioAdded",
            Self::SliceOutcomeAdded => "SliceOutcomeAdded",
            Self::SliceExternalPayloadAdded => "SliceExternalPayloadAdded",
            Self::SliceEventDefinitionAdded => "SliceEventDefinitionAdded",
            Self::SliceCommandDefinitionAdded => "SliceCommandDefinitionAdded",
            Self::SliceReadModelAdded => "SliceReadModelAdded",
            Self::SliceViewAdded => "SliceViewAdded",
            Self::SliceBitLevelDataFlowAdded => "SliceBitLevelDataFlowAdded",
            Self::SliceTranslationAdded => "SliceTranslationAdded",
            Self::SliceAutomationAdded => "SliceAutomationAdded",
            Self::SliceBoardElementAdded => "SliceBoardElementAdded",
            Self::SliceBoardConnectionAdded => "SliceBoardConnectionAdded",
            Self::ReviewRecorded => "ReviewRecorded",
            Self::ConflictResolved => "ConflictResolved",
        }
    }
}

impl Display for ExportedEventType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ExportedEventTypeError {
    message: String,
}

impl ExportedEventTypeError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled exported event type, got '{value}'"),
        }
    }
}

impl Display for ExportedEventTypeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ExportedEventTypeError {}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowEventPayload {
    slug: WorkflowSlug,
    name: ModelName,
    description: ModelDescription,
}

impl WorkflowEventPayload {
    fn from_workflow(workflow: &NewWorkflow) -> Self {
        Self {
            slug: workflow.slug().clone(),
            name: workflow.name().clone(),
            description: workflow.description().clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            slug: workflow_slug(required_str(payload, "slug")?)?,
            name: model_name(required_str(payload, "name")?)?,
            description: model_description(required_str(payload, "description")?)?,
        })
    }

    fn into_workflow(self) -> NewWorkflow {
        NewWorkflow::new(self.name, self.description, self.slug)
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slug": self.slug.as_ref(),
            "name": self.name.as_ref(),
            "description": self.description.as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ExportedEventBody {
    ProjectInitialized {
        name: ProjectName,
    },
    WorkflowAdded {
        workflow: NewWorkflow,
    },
    WorkflowUpdated {
        workflow: NewWorkflow,
    },
    WorkflowRemoved {
        slug: WorkflowSlug,
    },
    WorkflowOutcomeAdded {
        workflow: WorkflowSlug,
        outcome: WorkflowOutcomeRecord,
    },
    WorkflowCommandErrorAdded {
        workflow: WorkflowSlug,
        error: WorkflowCommandErrorRecord,
    },
    WorkflowOwnedDefinitionAdded {
        workflow: WorkflowSlug,
        definition: WorkflowOwnedDefinitionRecord,
    },
    WorkflowTransitionEvidenceAdded {
        workflow: WorkflowSlug,
        evidence: WorkflowTransitionEvidenceRecord,
    },
    WorkflowEntryLifecycleCoverageRequired {
        workflow: WorkflowSlug,
    },
    WorkflowEntryLifecycleStateAdded {
        workflow: WorkflowSlug,
        coverage: WorkflowEntryLifecycleStateRecord,
    },
    WorkflowReadinessDeclared {
        workflow: WorkflowSlug,
        projection_fingerprint: ProjectionFingerprint,
        model_content_digest: ModelContentDigest,
        verified_at: ReviewTimestamp,
        verified_by: ReviewerId,
        review_event: ReviewEventReference,
    },
    WorkflowConnected {
        connection: WorkflowConnection,
    },
    WorkflowTransitionRemoved {
        removal: WorkflowTransitionRemoval,
    },
    SliceAdded {
        slice: NewSlice,
    },
    SliceUpdated {
        slice: WorkflowSliceDetail,
    },
    SliceRemoved {
        slug: SliceSlug,
    },
    SliceScenarioAdded {
        scenario: NewSliceScenario,
    },
    SliceOutcomeAdded {
        outcome: NewOutcomeDefinition,
    },
    SliceExternalPayloadAdded {
        external_payload: NewExternalPayloadDefinition,
    },
    SliceEventDefinitionAdded {
        event: NewEventDefinition,
    },
    SliceCommandDefinitionAdded {
        command: NewCommandDefinition,
    },
    SliceReadModelAdded {
        read_model: NewReadModelDefinition,
    },
    SliceViewAdded {
        view: NewViewDefinition,
    },
    SliceBitLevelDataFlowAdded {
        data_flow: NewBitLevelDataFlow,
    },
    SliceTranslationAdded {
        translation: NewTranslationDefinition,
    },
    SliceAutomationAdded {
        automation: NewAutomationDefinition,
    },
    SliceBoardElementAdded {
        element: NewBoardElement,
    },
    SliceBoardConnectionAdded {
        connection: NewBoardConnection,
    },
    ReviewRecorded {
        workflow_slug: WorkflowSlug,
        model_content_digest: ModelContentDigest,
        reviewer_id: ReviewerId,
        reviewed_at: ReviewTimestamp,
        categories: Vec<ReviewRuleName>,
    },
    ConflictResolved {
        conflict_id: EventConflictId,
        chosen_event_id: ChosenEventId,
    },
}

impl ExportedEventBody {
    pub(crate) fn event_type(&self) -> ExportedEventType {
        match self {
            Self::ProjectInitialized { .. } => ExportedEventType::ProjectInitialized,
            Self::WorkflowAdded { .. } => ExportedEventType::WorkflowAdded,
            Self::WorkflowUpdated { .. } => ExportedEventType::WorkflowUpdated,
            Self::WorkflowRemoved { .. } => ExportedEventType::WorkflowRemoved,
            Self::WorkflowOutcomeAdded { .. } => ExportedEventType::WorkflowOutcomeAdded,
            Self::WorkflowCommandErrorAdded { .. } => ExportedEventType::WorkflowCommandErrorAdded,
            Self::WorkflowOwnedDefinitionAdded { .. } => {
                ExportedEventType::WorkflowOwnedDefinitionAdded
            }
            Self::WorkflowTransitionEvidenceAdded { .. } => {
                ExportedEventType::WorkflowTransitionEvidenceAdded
            }
            Self::WorkflowEntryLifecycleCoverageRequired { .. } => {
                ExportedEventType::WorkflowEntryLifecycleCoverageRequired
            }
            Self::WorkflowEntryLifecycleStateAdded { .. } => {
                ExportedEventType::WorkflowEntryLifecycleStateAdded
            }
            Self::WorkflowReadinessDeclared { .. } => ExportedEventType::WorkflowReadinessDeclared,
            Self::WorkflowConnected { .. } => ExportedEventType::WorkflowConnected,
            Self::WorkflowTransitionRemoved { .. } => ExportedEventType::WorkflowTransitionRemoved,
            Self::SliceAdded { .. } => ExportedEventType::SliceAdded,
            Self::SliceUpdated { .. } => ExportedEventType::SliceUpdated,
            Self::SliceRemoved { .. } => ExportedEventType::SliceRemoved,
            Self::SliceScenarioAdded { .. } => ExportedEventType::SliceScenarioAdded,
            Self::SliceOutcomeAdded { .. } => ExportedEventType::SliceOutcomeAdded,
            Self::SliceExternalPayloadAdded { .. } => ExportedEventType::SliceExternalPayloadAdded,
            Self::SliceEventDefinitionAdded { .. } => ExportedEventType::SliceEventDefinitionAdded,
            Self::SliceCommandDefinitionAdded { .. } => {
                ExportedEventType::SliceCommandDefinitionAdded
            }
            Self::SliceReadModelAdded { .. } => ExportedEventType::SliceReadModelAdded,
            Self::SliceViewAdded { .. } => ExportedEventType::SliceViewAdded,
            Self::SliceBitLevelDataFlowAdded { .. } => {
                ExportedEventType::SliceBitLevelDataFlowAdded
            }
            Self::SliceTranslationAdded { .. } => ExportedEventType::SliceTranslationAdded,
            Self::SliceAutomationAdded { .. } => ExportedEventType::SliceAutomationAdded,
            Self::SliceBoardElementAdded { .. } => ExportedEventType::SliceBoardElementAdded,
            Self::SliceBoardConnectionAdded { .. } => ExportedEventType::SliceBoardConnectionAdded,
            Self::ReviewRecorded { .. } => ExportedEventType::ReviewRecorded,
            Self::ConflictResolved { .. } => ExportedEventType::ConflictResolved,
        }
    }

    pub(crate) fn payload_json(&self) -> Value {
        match self {
            Self::ProjectInitialized { name } => json!({ "name": name.as_ref() }),
            Self::WorkflowAdded { workflow } | Self::WorkflowUpdated { workflow } => {
                WorkflowEventPayload::from_workflow(workflow).to_json_value()
            }
            Self::WorkflowRemoved { slug } => json!({ "slug": slug.as_ref() }),
            Self::WorkflowOutcomeAdded { workflow, outcome } => json!({
                "workflow": workflow.as_ref(),
                "source_slice": outcome.source_slice().as_ref(),
                "label": outcome.label().as_ref(),
                "externally_relevant": outcome.externally_relevant(),
            }),
            Self::WorkflowCommandErrorAdded { workflow, error } => json!({
                "workflow": workflow.as_ref(),
                "source_slice": error.source_slice().as_ref(),
                "command": error.command_name().as_ref(),
                "error": error.error_name().as_ref(),
            }),
            Self::WorkflowOwnedDefinitionAdded {
                workflow,
                definition,
            } => json!({
                "workflow": workflow.as_ref(),
                "source_slice": definition.source_slice().as_ref(),
                "definition_kind": definition.definition_kind().as_ref(),
                "definition_name": definition.definition_name().as_ref(),
                "definition_stream": definition.definition_stream().map(|stream| stream.as_ref()),
                "source_provenance": definition
                    .source_provenance()
                    .map(|provenance| provenance.as_ref()),
                "event_participation": definition
                    .event_participation()
                    .map(|participation| participation.as_ref()),
                "view_role": definition.view_role().map(|role| role.as_ref()),
            }),
            Self::WorkflowTransitionEvidenceAdded { workflow, evidence } => json!({
                "workflow": workflow.as_ref(),
                "from": evidence.source().as_ref(),
                "to": evidence.target().as_ref(),
                "via": evidence.kind().as_ref(),
                "name": evidence.trigger().as_ref(),
                "source_evidence": evidence.source_evidence().as_ref(),
                "target_evidence": evidence.target_evidence().as_ref(),
            }),
            Self::WorkflowEntryLifecycleCoverageRequired { workflow } => {
                json!({ "workflow": workflow.as_ref() })
            }
            Self::WorkflowEntryLifecycleStateAdded { workflow, coverage } => json!({
                "workflow": workflow.as_ref(),
                "state": coverage.state().as_ref(),
                "step": coverage.step().as_ref(),
                "evidence": coverage.evidence().as_ref(),
            }),
            Self::WorkflowReadinessDeclared {
                workflow,
                projection_fingerprint,
                model_content_digest,
                verified_at,
                verified_by,
                review_event,
            } => json!({
                "workflow": workflow.as_ref(),
                "projection_fingerprint": projection_fingerprint.as_ref(),
                "model_content_digest": model_content_digest.as_ref(),
                "verified_at": verified_at.as_ref(),
                "verified_by": verified_by.as_ref(),
                "review_event_id": review_event
                    .as_review_event_id()
                    .map(|event_id| event_id.as_ref()),
            }),
            Self::WorkflowConnected { connection } => json!({
                "workflow": connection.workflow_slug().as_ref(),
                "from": connection.source().as_ref(),
                "to": connection.target().slice_slug().map(|target| target.as_ref()),
                "to_workflow": connection
                    .target()
                    .workflow_slug()
                    .map(|target| target.as_ref()),
                "via": connection.kind().trigger_kind(),
                "name": connection.trigger().as_ref(),
                "payload_contract": connection.payload_contract().map(|contract| contract.as_ref()),
                "reason": connection.target().reason().map(|reason| reason.as_ref()),
            }),
            Self::WorkflowTransitionRemoved { removal } => json!({
                "workflow": removal.workflow_slug().as_ref(),
                "from": removal.source().as_ref(),
                "to": removal.target().slice_slug().map(|target| target.as_ref()),
                "to_workflow": removal.target().workflow_slug().map(|target| target.as_ref()),
                "via": removal.kind().trigger_kind(),
                "name": removal.trigger().as_ref(),
            }),
            Self::SliceAdded { slice } => json!({
                "workflow": slice.workflow_slug().as_ref(),
                "slug": slice.slug().as_ref(),
                "name": slice.name().as_ref(),
                "kind": slice.kind().as_str(),
                "description": slice.description().as_ref(),
            }),
            Self::SliceUpdated { slice } => json!({
                "slug": slice.slug().as_ref(),
                "name": slice.name().as_ref(),
                "kind": slice.kind().as_ref(),
                "description": slice.description().as_ref(),
            }),
            Self::SliceRemoved { slug } => json!({ "slug": slug.as_ref() }),
            Self::SliceScenarioAdded { scenario } => slice_scenario_payload(scenario),
            Self::SliceOutcomeAdded { outcome } => slice_outcome_payload(outcome),
            Self::SliceExternalPayloadAdded { external_payload } => {
                slice_external_payload_payload(external_payload)
            }
            Self::SliceEventDefinitionAdded { event } => slice_event_definition_payload(event),
            Self::SliceCommandDefinitionAdded { command } => {
                slice_command_definition_payload(command)
            }
            Self::SliceReadModelAdded { read_model } => slice_read_model_payload(read_model),
            Self::SliceViewAdded { view } => slice_view_payload(view),
            Self::SliceBitLevelDataFlowAdded { data_flow } => {
                slice_bit_level_data_flow_payload(data_flow)
            }
            Self::SliceTranslationAdded { translation } => slice_translation_payload(translation),
            Self::SliceAutomationAdded { automation } => slice_automation_payload(automation),
            Self::SliceBoardElementAdded { element } => slice_board_element_payload(element),
            Self::SliceBoardConnectionAdded { connection } => {
                slice_board_connection_payload(connection)
            }
            Self::ReviewRecorded {
                workflow_slug,
                model_content_digest,
                reviewer_id,
                reviewed_at,
                categories,
            } => json!({
                "workflow": workflow_slug.as_ref(),
                "model_content_digest": model_content_digest.as_ref(),
                "reviewer_id": reviewer_id.as_ref(),
                "reviewed_at": reviewed_at.as_ref(),
                "categories": categories.iter().map(ReviewRuleName::as_ref).collect::<Vec<_>>(),
            }),
            Self::ConflictResolved {
                conflict_id,
                chosen_event_id,
            } => json!({
                "conflict_id": conflict_id.as_ref(),
                "chosen_event_id": chosen_event_id.as_ref(),
            }),
        }
    }

    pub(crate) fn tagged_json_value(&self) -> Value {
        let mut body = Map::new();
        body.insert(self.event_type().as_ref().to_owned(), self.payload_json());
        Value::Object(body)
    }

    pub(crate) fn from_tagged_json_value(value: &Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "expected exported event body object".to_owned())?;
        let mut entries = object.iter();
        let (event_type, payload) = entries
            .next()
            .ok_or_else(|| "expected exported event body event type".to_owned())?;
        if entries.next().is_some() {
            return Err("expected exported event body with exactly one event type".to_owned());
        }
        let event_type =
            ExportedEventType::try_new(event_type.clone()).map_err(|error| error.to_string())?;
        Self::from_event_type_and_payload(event_type, payload)
    }

    pub(crate) fn from_event_type_and_payload(
        event_type: ExportedEventType,
        payload: &Value,
    ) -> Result<Self, String> {
        match event_type {
            ExportedEventType::ProjectInitialized => Ok(Self::ProjectInitialized {
                name: project_name(required_str(payload, "name")?)?,
            }),
            ExportedEventType::WorkflowAdded => Ok(Self::WorkflowAdded {
                workflow: WorkflowEventPayload::from_json_value(payload)?.into_workflow(),
            }),
            ExportedEventType::WorkflowUpdated => Ok(Self::WorkflowUpdated {
                workflow: WorkflowEventPayload::from_json_value(payload)?.into_workflow(),
            }),
            ExportedEventType::WorkflowRemoved => Ok(Self::WorkflowRemoved {
                slug: workflow_slug(required_str(payload, "slug")?)?,
            }),
            ExportedEventType::WorkflowOutcomeAdded => Ok(Self::WorkflowOutcomeAdded {
                workflow: workflow_slug(required_str(payload, "workflow")?)?,
                outcome: workflow_outcome_from_payload(payload)?,
            }),
            ExportedEventType::WorkflowCommandErrorAdded => Ok(Self::WorkflowCommandErrorAdded {
                workflow: workflow_slug(required_str(payload, "workflow")?)?,
                error: workflow_command_error_from_payload(payload)?,
            }),
            ExportedEventType::WorkflowOwnedDefinitionAdded => {
                Ok(Self::WorkflowOwnedDefinitionAdded {
                    workflow: workflow_slug(required_str(payload, "workflow")?)?,
                    definition: workflow_owned_definition_from_payload(payload)?,
                })
            }
            ExportedEventType::WorkflowTransitionEvidenceAdded => {
                Ok(Self::WorkflowTransitionEvidenceAdded {
                    workflow: workflow_slug(required_str(payload, "workflow")?)?,
                    evidence: workflow_transition_evidence_from_payload(payload)?,
                })
            }
            ExportedEventType::WorkflowEntryLifecycleCoverageRequired => {
                Ok(Self::WorkflowEntryLifecycleCoverageRequired {
                    workflow: workflow_slug(required_str(payload, "workflow")?)?,
                })
            }
            ExportedEventType::WorkflowEntryLifecycleStateAdded => {
                Ok(Self::WorkflowEntryLifecycleStateAdded {
                    workflow: workflow_slug(required_str(payload, "workflow")?)?,
                    coverage: workflow_entry_lifecycle_state_from_payload(payload)?,
                })
            }
            ExportedEventType::WorkflowReadinessDeclared => Ok(Self::WorkflowReadinessDeclared {
                workflow: workflow_slug(required_str(payload, "workflow")?)?,
                projection_fingerprint: projection_fingerprint_from_payload(payload)?,
                model_content_digest: model_content_digest_from_payload(payload)?,
                verified_at: review_timestamp(required_str(payload, "verified_at")?)?,
                verified_by: reviewer_id(required_str(payload, "verified_by")?)?,
                review_event: review_event_reference_from_payload(payload)?,
            }),
            ExportedEventType::WorkflowConnected => Ok(Self::WorkflowConnected {
                connection: workflow_connection_from_payload(payload)?,
            }),
            ExportedEventType::WorkflowTransitionRemoved => Ok(Self::WorkflowTransitionRemoved {
                removal: workflow_transition_removal_from_payload(payload)?,
            }),
            ExportedEventType::SliceAdded => Ok(Self::SliceAdded {
                slice: slice_from_payload(payload)?,
            }),
            ExportedEventType::SliceUpdated => Ok(Self::SliceUpdated {
                slice: workflow_slice_detail_from_payload(payload)?,
            }),
            ExportedEventType::SliceRemoved => Ok(Self::SliceRemoved {
                slug: slice_slug(required_str(payload, "slug")?)?,
            }),
            ExportedEventType::SliceScenarioAdded => Ok(Self::SliceScenarioAdded {
                scenario: slice_scenario_from_payload(payload)?,
            }),
            ExportedEventType::SliceOutcomeAdded => Ok(Self::SliceOutcomeAdded {
                outcome: slice_outcome_from_payload(payload)?,
            }),
            ExportedEventType::SliceExternalPayloadAdded => Ok(Self::SliceExternalPayloadAdded {
                external_payload: slice_external_payload_from_payload(payload)?,
            }),
            ExportedEventType::SliceEventDefinitionAdded => Ok(Self::SliceEventDefinitionAdded {
                event: slice_event_definition_from_payload(payload)?,
            }),
            ExportedEventType::SliceCommandDefinitionAdded => {
                Ok(Self::SliceCommandDefinitionAdded {
                    command: slice_command_definition_from_payload(payload)?,
                })
            }
            ExportedEventType::SliceReadModelAdded => Ok(Self::SliceReadModelAdded {
                read_model: slice_read_model_from_payload(payload)?,
            }),
            ExportedEventType::SliceViewAdded => Ok(Self::SliceViewAdded {
                view: slice_view_from_payload(payload)?,
            }),
            ExportedEventType::SliceBitLevelDataFlowAdded => Ok(Self::SliceBitLevelDataFlowAdded {
                data_flow: slice_bit_level_data_flow_from_payload(payload)?,
            }),
            ExportedEventType::SliceTranslationAdded => Ok(Self::SliceTranslationAdded {
                translation: slice_translation_from_payload(payload)?,
            }),
            ExportedEventType::SliceAutomationAdded => Ok(Self::SliceAutomationAdded {
                automation: slice_automation_from_payload(payload)?,
            }),
            ExportedEventType::SliceBoardElementAdded => Ok(Self::SliceBoardElementAdded {
                element: slice_board_element_from_payload(payload)?,
            }),
            ExportedEventType::SliceBoardConnectionAdded => Ok(Self::SliceBoardConnectionAdded {
                connection: slice_board_connection_from_payload(payload)?,
            }),
            ExportedEventType::ReviewRecorded => Ok(Self::ReviewRecorded {
                workflow_slug: workflow_slug(required_str(payload, "workflow")?)?,
                model_content_digest: ModelContentDigest::new(artifact_digest_from_str(
                    required_str(payload, "model_content_digest")?,
                )?),
                reviewer_id: reviewer_id(required_str(payload, "reviewer_id")?)?,
                reviewed_at: review_timestamp(required_str(payload, "reviewed_at")?)?,
                categories: review_rule_names_from_payload(payload)?,
            }),
            ExportedEventType::ConflictResolved => Ok(Self::ConflictResolved {
                conflict_id: EventConflictId::new(artifact_digest_from_str(required_str(
                    payload,
                    "conflict_id",
                )?)?),
                chosen_event_id: ChosenEventId::new(artifact_digest_from_str(required_str(
                    payload,
                    "chosen_event_id",
                )?)?),
            }),
        }
    }
}

impl EventDraft {
    pub(crate) fn project_initialized(name: &ProjectName) -> Self {
        Self {
            stream_id: EventStreamId::project(),
            body: ExportedEventBody::ProjectInitialized { name: name.clone() },
        }
    }

    pub(crate) fn event_type(&self) -> ExportedEventType {
        self.body.event_type()
    }

    pub(crate) fn stream_id(&self) -> &str {
        self.stream_id.as_ref()
    }

    pub(crate) fn body(&self) -> &ExportedEventBody {
        &self.body
    }

    pub(crate) fn payload_json(&self) -> Value {
        self.body.payload_json()
    }

    pub(crate) fn workflow_added(workflow: &NewWorkflow) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow.slug()),
            body: ExportedEventBody::WorkflowAdded {
                workflow: workflow.clone(),
            },
        }
    }

    pub(crate) fn workflow_updated(workflow: &NewWorkflow) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow.slug()),
            body: ExportedEventBody::WorkflowUpdated {
                workflow: workflow.clone(),
            },
        }
    }

    pub(crate) fn workflow_removed(workflow: &WorkflowSlug) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowRemoved {
                slug: workflow.clone(),
            },
        }
    }

    pub(crate) fn workflow_outcome_added(
        workflow: &WorkflowSlug,
        outcome: &WorkflowOutcomeRecord,
    ) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowOutcomeAdded {
                workflow: workflow.clone(),
                outcome: outcome.clone(),
            },
        }
    }

    pub(crate) fn workflow_command_error_added(
        workflow: &WorkflowSlug,
        error: &WorkflowCommandErrorRecord,
    ) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowCommandErrorAdded {
                workflow: workflow.clone(),
                error: error.clone(),
            },
        }
    }

    pub(crate) fn workflow_owned_definition_added(
        workflow: &WorkflowSlug,
        definition: &WorkflowOwnedDefinitionRecord,
    ) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowOwnedDefinitionAdded {
                workflow: workflow.clone(),
                definition: definition.clone(),
            },
        }
    }

    pub(crate) fn workflow_transition_evidence_added(
        workflow: &WorkflowSlug,
        evidence: &WorkflowTransitionEvidenceRecord,
    ) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowTransitionEvidenceAdded {
                workflow: workflow.clone(),
                evidence: evidence.clone(),
            },
        }
    }

    pub(crate) fn workflow_entry_lifecycle_coverage_required(workflow: &WorkflowSlug) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowEntryLifecycleCoverageRequired {
                workflow: workflow.clone(),
            },
        }
    }

    pub(crate) fn workflow_entry_lifecycle_state_added(
        workflow: &WorkflowSlug,
        coverage: &WorkflowEntryLifecycleStateRecord,
    ) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowEntryLifecycleStateAdded {
                workflow: workflow.clone(),
                coverage: coverage.clone(),
            },
        }
    }

    pub(crate) fn workflow_readiness_declared(
        workflow: &WorkflowSlug,
        projection_fingerprint: &ProjectionFingerprint,
        model_content_digest: &ModelContentDigest,
        verified_at: &ReviewTimestamp,
        verified_by: &ReviewerId,
        review_event: &ReviewEventReference,
    ) -> Self {
        Self {
            stream_id: EventStreamId::workflow(workflow),
            body: ExportedEventBody::WorkflowReadinessDeclared {
                workflow: workflow.clone(),
                projection_fingerprint: projection_fingerprint.clone(),
                model_content_digest: model_content_digest.clone(),
                verified_at: verified_at.clone(),
                verified_by: verified_by.clone(),
                review_event: review_event.clone(),
            },
        }
    }

    pub(crate) fn workflow_connected(connection: &WorkflowConnection) -> Self {
        Self {
            stream_id: EventStreamId::workflow(connection.workflow_slug()),
            body: ExportedEventBody::WorkflowConnected {
                connection: connection.clone(),
            },
        }
    }

    pub(crate) fn workflow_transition_removed(removal: &WorkflowTransitionRemoval) -> Self {
        Self {
            stream_id: EventStreamId::workflow(removal.workflow_slug()),
            body: ExportedEventBody::WorkflowTransitionRemoved {
                removal: removal.clone(),
            },
        }
    }

    pub(crate) fn slice_added(slice: &NewSlice) -> Self {
        Self {
            stream_id: EventStreamId::slice(slice.slug()),
            body: ExportedEventBody::SliceAdded {
                slice: slice.clone(),
            },
        }
    }

    pub(crate) fn slice_updated(slice: &WorkflowSliceDetail) -> Self {
        Self {
            stream_id: EventStreamId::slice(slice.slug()),
            body: ExportedEventBody::SliceUpdated {
                slice: slice.clone(),
            },
        }
    }

    pub(crate) fn slice_removed(slice: &WorkflowSliceDetail) -> Self {
        Self {
            stream_id: EventStreamId::slice(slice.slug()),
            body: ExportedEventBody::SliceRemoved {
                slug: slice.slug().clone(),
            },
        }
    }

    pub(crate) fn slice_scenario_added(scenario: &NewSliceScenario) -> Self {
        Self {
            stream_id: EventStreamId::slice(scenario.slice_slug()),
            body: ExportedEventBody::SliceScenarioAdded {
                scenario: scenario.clone(),
            },
        }
    }

    pub(crate) fn slice_outcome_added(outcome: &NewOutcomeDefinition) -> Self {
        Self {
            stream_id: EventStreamId::slice(outcome.slice_slug()),
            body: ExportedEventBody::SliceOutcomeAdded {
                outcome: outcome.clone(),
            },
        }
    }

    pub(crate) fn slice_external_payload_added(
        external_payload: &NewExternalPayloadDefinition,
    ) -> Self {
        Self {
            stream_id: EventStreamId::slice(external_payload.slice_slug()),
            body: ExportedEventBody::SliceExternalPayloadAdded {
                external_payload: external_payload.clone(),
            },
        }
    }

    pub(crate) fn slice_event_definition_added(event: &NewEventDefinition) -> Self {
        Self {
            stream_id: EventStreamId::slice(event.slice_slug()),
            body: ExportedEventBody::SliceEventDefinitionAdded {
                event: event.clone(),
            },
        }
    }

    pub(crate) fn slice_command_definition_added(command: &NewCommandDefinition) -> Self {
        Self {
            stream_id: EventStreamId::slice(command.slice_slug()),
            body: ExportedEventBody::SliceCommandDefinitionAdded {
                command: command.clone(),
            },
        }
    }

    pub(crate) fn slice_read_model_added(read_model: &NewReadModelDefinition) -> Self {
        Self {
            stream_id: EventStreamId::slice(read_model.slice_slug()),
            body: ExportedEventBody::SliceReadModelAdded {
                read_model: read_model.clone(),
            },
        }
    }

    pub(crate) fn slice_view_added(view: &NewViewDefinition) -> Self {
        Self {
            stream_id: EventStreamId::slice(view.slice_slug()),
            body: ExportedEventBody::SliceViewAdded { view: view.clone() },
        }
    }

    pub(crate) fn slice_bit_level_data_flow_added(data_flow: &NewBitLevelDataFlow) -> Self {
        Self {
            stream_id: EventStreamId::slice(data_flow.slice_slug()),
            body: ExportedEventBody::SliceBitLevelDataFlowAdded {
                data_flow: data_flow.clone(),
            },
        }
    }

    pub(crate) fn slice_translation_added(translation: &NewTranslationDefinition) -> Self {
        Self {
            stream_id: EventStreamId::slice(translation.slice_slug()),
            body: ExportedEventBody::SliceTranslationAdded {
                translation: translation.clone(),
            },
        }
    }

    pub(crate) fn slice_automation_added(automation: &NewAutomationDefinition) -> Self {
        Self {
            stream_id: EventStreamId::slice(automation.slice_slug()),
            body: ExportedEventBody::SliceAutomationAdded {
                automation: automation.clone(),
            },
        }
    }

    pub(crate) fn slice_board_element_added(element: &NewBoardElement) -> Self {
        Self {
            stream_id: EventStreamId::slice(element.slice_slug()),
            body: ExportedEventBody::SliceBoardElementAdded {
                element: element.clone(),
            },
        }
    }

    pub(crate) fn slice_board_connection_added(connection: &NewBoardConnection) -> Self {
        Self {
            stream_id: EventStreamId::slice(connection.slice_slug()),
            body: ExportedEventBody::SliceBoardConnectionAdded {
                connection: connection.clone(),
            },
        }
    }

    pub(crate) fn conflict_resolved(
        conflict_id: &EventConflictId,
        chosen_event_id: &ChosenEventId,
    ) -> Self {
        Self {
            stream_id: EventStreamId::project(),
            body: ExportedEventBody::ConflictResolved {
                conflict_id: conflict_id.clone(),
                chosen_event_id: chosen_event_id.clone(),
            },
        }
    }

    pub(crate) fn review_recorded(
        workflow_slug: &WorkflowSlug,
        model_content_digest: &ModelContentDigest,
        reviewer_id: &ReviewerId,
        reviewed_at: &ReviewTimestamp,
        categories: &[ReviewRuleName],
    ) -> Self {
        Self {
            stream_id: EventStreamId::review(workflow_slug),
            body: ExportedEventBody::ReviewRecorded {
                workflow_slug: workflow_slug.clone(),
                model_content_digest: model_content_digest.clone(),
                reviewer_id: reviewer_id.clone(),
                reviewed_at: reviewed_at.clone(),
                categories: categories.to_vec(),
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ExportedEvent {
    meta: ExportedEventMeta,
    body: ExportedEventBody,
}

impl ExportedEvent {
    pub(crate) fn from_draft(
        draft: &EventDraft,
        parents: Vec<ExportedEventId>,
        command_ordinal: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            meta: ExportedEventMeta::from_draft(draft, parents, command_ordinal)?,
            body: draft.body.clone(),
        })
    }

    #[cfg(test)]
    pub(crate) fn from_draft_for_test(draft: &EventDraft) -> Result<Self, String> {
        Self::from_draft(draft, Vec::new(), 0)
    }

    fn from_json_value(value: &Value) -> Result<Self, String> {
        let schema_version = required_str(value, "schema_version")?;
        if schema_version != SCHEMA_VERSION {
            return Err(format!(
                "unsupported exported event schema version {schema_version}"
            ));
        }
        let event_type = ExportedEventType::try_new(required_str(value, "type")?.to_owned())
            .map_err(|error| error.to_string())?;
        let body = ExportedEventBody::from_event_type_and_payload(
            event_type,
            required_object(value, "payload")?,
        )?;
        Ok(Self {
            meta: ExportedEventMeta::from_json_value(value)?,
            body,
        })
    }

    pub(crate) fn from_json_str(contents: &str) -> Result<Self, String> {
        let value = serde_json::from_str::<Value>(contents).map_err(|error| error.to_string())?;
        Self::from_json_value(&value)
    }

    #[cfg(test)]
    pub(crate) fn from_json_for_test(value: &Value) -> Result<Self, String> {
        Self::from_json_value(value)
    }

    pub(crate) fn event_type(&self) -> ExportedEventType {
        self.body.event_type()
    }

    pub(crate) fn event_id(&self) -> &ExportedEventId {
        self.meta.event_id()
    }

    pub(crate) fn stream_id(&self) -> &EventStreamId {
        self.meta.stream_id()
    }

    pub(crate) fn parents(&self) -> &[ExportedEventId] {
        self.meta.parents()
    }

    pub(crate) fn body(&self) -> &ExportedEventBody {
        &self.body
    }

    pub(crate) fn payload_json(&self) -> Value {
        self.body.payload_json()
    }

    fn to_json_value(&self) -> Value {
        json!({
            "schema_version": SCHEMA_VERSION,
            "event_id": self.meta.event_id.as_ref(),
            "command_id": self.meta.command_id.as_ref(),
            "command_ordinal": self.meta.command_ordinal,
            "stream_id": self.meta.stream_id.as_ref(),
            "parents": self.meta.parents.iter().map(|parent| parent.as_ref()).collect::<Vec<_>>(),
            "type": self.event_type().as_ref(),
            "payload": self.payload_json(),
        })
    }

    pub(crate) fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string(&self.to_json_value()).map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExportedEventHeader {
    event_id: ExportedEventId,
    stream_id: Option<EventStreamId>,
    parents: Vec<ExportedEventId>,
    event_type: ExportedEventType,
}

impl ExportedEventHeader {
    fn from_json_value(value: &Value) -> Result<Self, String> {
        Ok(Self {
            event_id: required_exported_event_id(value, "event_id")?,
            stream_id: optional_str(value, "stream_id")?
                .map(|stream_id| EventStreamId::try_new(stream_id.to_owned()))
                .transpose()
                .map_err(|error| error.to_string())?,
            parents: parents(value)?,
            event_type: ExportedEventType::try_new(required_str(value, "type")?.to_owned())
                .map_err(|error| error.to_string())?,
        })
    }

    fn parent_count(&self) -> usize {
        self.parents.len()
    }
}

trait ExportedEventFrontier {
    fn frontier_event_id(&self) -> &ExportedEventId;
    fn frontier_event_type(&self) -> ExportedEventType;
}

impl ExportedEventFrontier for ExportedEvent {
    fn frontier_event_id(&self) -> &ExportedEventId {
        self.event_id()
    }

    fn frontier_event_type(&self) -> ExportedEventType {
        self.event_type()
    }
}

impl ExportedEventFrontier for ExportedEventHeader {
    fn frontier_event_id(&self) -> &ExportedEventId {
        &self.event_id
    }

    fn frontier_event_type(&self) -> ExportedEventType {
        self.event_type
    }
}

fn slice_scenario_payload(scenario: &NewSliceScenario) -> Value {
    json!({
        "slice": scenario.slice_slug().as_ref(),
        "kind": scenario.kind().as_str(),
        "name": scenario.name().as_ref(),
        "given": scenario.given().as_ref(),
        "when": scenario.when().as_ref(),
        "then": scenario.then().as_ref(),
        "read_streams": scenario
            .read_streams()
            .as_slice()
            .iter()
            .map(|stream| stream.as_ref())
            .collect::<Vec<_>>(),
        "written_streams": scenario
            .written_streams()
            .as_slice()
            .iter()
            .map(|stream| stream.as_ref())
            .collect::<Vec<_>>(),
        "contract_kind": scenario.contract_kind().map(|kind| kind.as_ref()),
        "covered_definition": scenario
            .covered_definition()
            .map(|definition| definition.as_ref()),
        "error_references": scenario
            .error_references()
            .as_slice()
            .iter()
            .map(|error| error.as_ref())
            .collect::<Vec<_>>(),
    })
}

fn slice_outcome_payload(outcome: &NewOutcomeDefinition) -> Value {
    json!({
        "slice": outcome.slice_slug().as_ref(),
        "label": outcome.label().as_ref(),
        "events": outcome
            .event_set()
            .as_slice()
            .iter()
            .map(|event| event.as_ref())
            .collect::<Vec<_>>(),
        "externally_relevant": outcome.externally_relevant(),
    })
}

fn slice_external_payload_payload(external_payload: &NewExternalPayloadDefinition) -> Value {
    json!({
        "slice": external_payload.slice_slug().as_ref(),
        "name": external_payload.name().as_ref(),
        "field": external_payload.field().as_ref(),
        "field_provenance": external_payload.field_provenance().as_ref(),
        "bit_encoding": external_payload.bit_encoding().as_ref(),
    })
}

fn slice_event_definition_payload(event: &NewEventDefinition) -> Value {
    let attribute = event.attribute();
    json!({
        "slice": event.slice_slug().as_ref(),
        "name": event.name().as_ref(),
        "stream": event.stream().as_ref(),
        "attribute": {
            "name": attribute.name().as_ref(),
            "source_kind": attribute.source_kind().as_ref(),
            "source_name": attribute.source_name().as_ref(),
            "source_field": attribute.source_field().as_ref(),
            "generated_source_kind": attribute
                .generated_source_kind()
                .map(|source_kind| source_kind.as_ref()),
            "provenance": attribute.provenance_description().as_ref(),
        },
        "observed": event.observed(),
        "shared": event.shared(),
    })
}

fn slice_command_definition_payload(command: &NewCommandDefinition) -> Value {
    json!({
        "slice": command.slice_slug().as_ref(),
        "name": command.name().as_ref(),
        "input": command_input_payload(command.input()),
        "emitted_events": command
            .emitted_events()
            .as_slice()
            .iter()
            .map(|event| event.as_ref())
            .collect::<Vec<_>>(),
        "observed_streams": command
            .observed_streams()
            .as_slice()
            .iter()
            .map(|stream| stream.as_ref())
            .collect::<Vec<_>>(),
        "errors": command
            .errors()
            .as_slice()
            .iter()
            .map(|error| {
                json!({
                    "name": error.name().as_ref(),
                    "scenario": error.scenario_name().as_ref(),
                    "recovery": error.recovery_kind().as_ref(),
                })
            })
            .collect::<Vec<_>>(),
        "singleton_repeat_behavior": command
            .singleton_repeat_behavior()
            .map(|repeat_behavior| repeat_behavior.as_ref()),
    })
}

fn slice_read_model_payload(read_model: &NewReadModelDefinition) -> Value {
    json!({
        "slice": read_model.slice_slug().as_ref(),
        "name": read_model.name().as_ref(),
        "field": read_model_field_payload(read_model.field()),
        "transitive": read_model.transitive(),
        "relationship_fields": read_model
            .relationship_fields()
            .as_slice()
            .iter()
            .map(|field| field.as_ref())
            .collect::<Vec<_>>(),
        "transitive_rule": read_model
            .transitive_rule()
            .map(|rule| rule.as_ref()),
        "example_scenario": read_model
            .example_scenario_name()
            .map(|scenario| scenario.as_ref()),
    })
}

fn slice_view_payload(view: &NewViewDefinition) -> Value {
    json!({
        "slice": view.slice_slug().as_ref(),
        "name": view.name().as_ref(),
        "field": view_field_payload(view.field()),
        "controls": view
            .controls()
            .as_slice()
            .iter()
            .map(control_payload)
            .collect::<Vec<_>>(),
        "local_states": view
            .local_states()
            .as_slice()
            .iter()
            .map(|state| state.as_ref())
            .collect::<Vec<_>>(),
        "filters": view
            .filters()
            .as_slice()
            .iter()
            .map(|filter| filter.as_ref())
            .collect::<Vec<_>>(),
    })
}

fn slice_bit_level_data_flow_payload(data_flow: &NewBitLevelDataFlow) -> Value {
    json!({
        "slice": data_flow.slice_slug().as_ref(),
        "datum": data_flow.datum().as_ref(),
        "source": data_flow.source().as_ref(),
        "source_kind": data_flow.source_kind().as_ref(),
        "transformation": data_flow.transformation().as_ref(),
        "target": data_flow.target().as_ref(),
        "bit_encoding": data_flow.bit_encoding().as_ref(),
    })
}

fn slice_translation_payload(translation: &NewTranslationDefinition) -> Value {
    json!({
        "slice": translation.slice_slug().as_ref(),
        "name": translation.name().as_ref(),
        "external_event": translation.external_event_name().as_ref(),
        "payload_contract": translation.payload_contract_name().as_ref(),
        "command": translation.command_name().as_ref(),
    })
}

fn slice_automation_payload(automation: &NewAutomationDefinition) -> Value {
    json!({
        "slice": automation.slice_slug().as_ref(),
        "name": automation.name().as_ref(),
        "trigger": automation.trigger_name().as_ref(),
        "command": automation.command_name().as_ref(),
        "handled_errors": automation
            .handled_errors()
            .as_slice()
            .iter()
            .map(|error| error.as_ref())
            .collect::<Vec<_>>(),
        "reaction": automation.reaction_description().as_ref(),
    })
}

fn slice_board_element_payload(element: &NewBoardElement) -> Value {
    json!({
        "slice": element.slice_slug().as_ref(),
        "name": element.name().as_ref(),
        "kind": element.kind().as_ref(),
        "lane": element.lane().as_ref(),
        "declared_name": element.declared_name().as_ref(),
        "main_path": element.main_path(),
    })
}

fn slice_board_connection_payload(connection: &NewBoardConnection) -> Value {
    json!({
        "slice": connection.slice_slug().as_ref(),
        "source": connection.source().as_ref(),
        "source_kind": connection.source_kind().as_ref(),
        "target": connection.target().as_ref(),
        "target_kind": connection.target_kind().as_ref(),
    })
}

fn command_input_payload(input: &NewCommandInput) -> Value {
    json!({
        "name": input.name().as_ref(),
        "source_kind": input.source_kind().as_ref(),
        "source_description": input.source_description().as_ref(),
        "provenance_chain": input
            .provenance_chain()
            .as_slice()
            .iter()
            .map(|hop| hop.as_ref())
            .collect::<Vec<_>>(),
        "event_stream_source_event": input
            .event_stream_source_event()
            .map(|event| event.as_ref()),
        "event_stream_source_attribute": input
            .event_stream_source_attribute()
            .map(|attribute| attribute.as_ref()),
        "external_payload_source_name": input
            .external_payload_source_name()
            .map(|name| name.as_ref()),
        "external_payload_source_field": input
            .external_payload_source_field()
            .map(|field| field.as_ref()),
        "generated_source_name": input
            .generated_source_name()
            .map(|name| name.as_ref()),
        "generated_source_field": input
            .generated_source_field()
            .map(|field| field.as_ref()),
        "session_source_name": input
            .session_source_name()
            .map(|name| name.as_ref()),
        "session_source_field": input
            .session_source_field()
            .map(|field| field.as_ref()),
        "invocation_argument_source_name": input
            .invocation_argument_source_name()
            .map(|name| name.as_ref()),
        "invocation_argument_source_field": input
            .invocation_argument_source_field()
            .map(|field| field.as_ref()),
    })
}

fn read_model_field_payload(field: &NewReadModelField) -> Value {
    json!({
        "name": field.name().as_ref(),
        "source_kind": field.source_kind().as_ref(),
        "source_event": field.source_event().map(|event| event.as_ref()),
        "source_attribute": field
            .source_attribute()
            .map(|attribute| attribute.as_ref()),
        "derivation_rule": field.derivation_rule().map(|rule| rule.as_ref()),
        "derivation_source_fields": field
            .derivation_source_fields()
            .as_slice()
            .iter()
            .map(|field| field.as_ref())
            .collect::<Vec<_>>(),
        "absence_event": field.absence_event().map(|event| event.as_ref()),
        "derivation_scenario": field
            .derivation_scenario_name()
            .map(|scenario| scenario.as_ref()),
        "absence_scenario": field
            .absence_scenario_name()
            .map(|scenario| scenario.as_ref()),
        "provenance": field.provenance_description().as_ref(),
    })
}

fn view_field_payload(field: &NewViewField) -> Value {
    json!({
        "name": field.name().as_ref(),
        "source_kind": field.source_kind().as_ref(),
        "source_read_model": field.source_read_model().as_ref(),
        "source_field": field.source_field().as_ref(),
        "sketch_token": field.sketch_token().as_ref(),
        "provenance": field.provenance_description().as_ref(),
        "bit_encoding": field.bit_encoding().as_ref(),
    })
}

fn control_payload(control: &NewControlDefinition) -> Value {
    json!({
        "name": control.name().as_ref(),
        "command": control.command_name().as_ref(),
        "input": control_input_payload(control.input()),
        "handled_errors": control
            .handled_errors()
            .as_slice()
            .iter()
            .map(|error| error.as_ref())
            .collect::<Vec<_>>(),
        "recovery_behavior": control.recovery_behavior().as_ref(),
        "sketch_token": control.sketch_token().as_ref(),
        "navigation": navigation_payload(control.navigation()),
    })
}

fn control_input_payload(input: &NewControlInputProvision) -> Value {
    json!({
        "name": input.name().as_ref(),
        "source_kind": input.source_kind().as_ref(),
        "source_description": input.source_description().as_ref(),
        "sketch_token": input.sketch_token().as_ref(),
        "visible_to_actor": input.visible_to_actor(),
        "decision_field": input.decision_field(),
    })
}

fn navigation_payload(navigation: &NewNavigationTarget) -> Value {
    json!({
        "type": navigation.target_type().as_ref(),
        "target": navigation.target_name().as_ref(),
        "external_workflow": navigation
            .external_workflow_name()
            .map(|workflow| workflow.as_ref()),
        "external_system": navigation
            .external_system_name()
            .map(|system| system.as_ref()),
        "handoff_contract": navigation
            .handoff_contract()
            .map(|contract| contract.as_ref()),
    })
}

pub(crate) fn export_event_file_contents(
    draft: &EventDraft,
) -> Result<(String, FileContents), String> {
    let parents = exported_event_ids()?;
    let command_ordinal = command_ordinal_for_stream(draft.stream_id.as_ref())?;
    let exported = ExportedEvent::from_draft(draft, parents, command_ordinal)?;
    let json = serde_json::to_string_pretty(&exported.to_json_value())
        .map_err(|error| error.to_string())?;
    let contents = FileContents::try_new(format!("{json}\n")).map_err(|error| error.to_string())?;
    Ok((
        format!(
            "{EVENT_EXPORT_DIRECTORY}/{}.json",
            exported.event_id().as_ref()
        ),
        contents,
    ))
}

pub(crate) fn project_exported_events() -> Result<Option<EffectPlan>, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(None);
    }

    let events = exported_events_in_topological_order(path)?;
    if events.is_empty() {
        return Ok(None);
    }

    let fingerprint = projection_fingerprint(&events)?;
    ProjectedModel::from_events(events)
        .and_then(|model| model.effects(fingerprint))
        .map(Some)
}

pub(crate) fn exported_events_projection_fingerprint() -> Result<Option<String>, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(None);
    }

    let events = exported_event_headers_in_topological_order(path)?;
    if events.is_empty() {
        return Ok(None);
    }

    projection_fingerprint(&events).map(Some)
}

pub(crate) fn list_event_conflicts() -> Result<EffectPlan, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(EffectPlan::new(vec![Effect::Report(report_line(
            "no event conflicts",
        )?)]));
    }

    let conflicts = event_conflicts(path)?;
    if conflicts.is_empty() {
        return Ok(EffectPlan::new(vec![Effect::Report(report_line(
            "no event conflicts",
        )?)]));
    }

    let effects = conflicts
        .into_iter()
        .map(|conflict| {
            let event_ids = conflict
                .event_ids
                .into_iter()
                .map(|event_id| event_id.as_ref().to_owned())
                .collect::<Vec<_>>()
                .join(",");
            report_line(format!(
                "conflict {} {} {} {} id {}",
                conflict.stream_id,
                conflict.event_type,
                conflict.semantic_key,
                event_ids,
                conflict.id
            ))
            .map(Effect::Report)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(EffectPlan::new(effects))
}

pub(crate) fn list_stale_workflow_readiness() -> Result<EffectPlan, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(EffectPlan::new(Vec::new()));
    }

    let events = exported_events_in_topological_order(path)?;
    let current_fingerprint = projection_fingerprint_digest(&events)?;
    let latest_readiness = events.into_iter().try_fold(
        BTreeMap::<WorkflowSlug, WorkflowReadinessDeclaration>::new(),
        |mut declarations, event| -> Result<_, String> {
            if let ExportedEventBody::WorkflowReadinessDeclared {
                workflow,
                projection_fingerprint,
                ..
            } = event.body()
            {
                let declaration = WorkflowReadinessDeclaration {
                    workflow: workflow.clone(),
                    projection_fingerprint: projection_fingerprint.clone(),
                };
                declarations.insert(declaration.workflow.clone(), declaration);
            }
            Ok(declarations)
        },
    )?;

    let effects = latest_readiness
        .into_values()
        .filter(|declaration| declaration.projection_fingerprint != current_fingerprint)
        .map(|declaration| {
            report_line(format!(
                "workflow {} readiness is stale for current event frontier",
                declaration.workflow.as_ref()
            ))
            .map(Effect::Report)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(EffectPlan::new(effects))
}

pub(crate) fn resolve_event_conflict(
    conflict_id: EventConflictId,
    chosen_event_id: ChosenEventId,
) -> Result<EffectPlan, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Err(format!("unknown event conflict {}", conflict_id.as_ref()));
    }

    let conflict = event_conflicts(path)?
        .into_iter()
        .find(|conflict| conflict.id == conflict_id.as_ref())
        .ok_or_else(|| format!("unknown event conflict {}", conflict_id.as_ref()))?;
    let chosen_exported_event_id = ExportedEventId::try_new(chosen_event_id.as_ref().to_owned())?;
    if !conflict.event_ids.contains(&chosen_exported_event_id) {
        return Err(format!(
            "event {} is not part of conflict {}",
            chosen_exported_event_id.as_ref(),
            conflict_id.as_ref()
        ));
    }

    Ok(EffectPlan::new(vec![
        Effect::ExportEvent(EventDraft::conflict_resolved(
            &conflict_id,
            &chosen_event_id,
        )),
        Effect::Report(report_line(format!(
            "resolved conflict {}",
            conflict_id.as_ref()
        ))?),
    ]))
}

pub(crate) fn unresolved_event_conflicts_exist() -> Result<bool, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(false);
    }

    event_conflicts(path).map(|conflicts| !conflicts.is_empty())
}

pub(crate) fn reject_legacy_artifact_only_project() -> Result<(), String> {
    let has_project_manifest = Path::new("emc.toml").exists();
    let has_generated_artifacts =
        Path::new("model/lean").exists() || Path::new("model/quint").exists();
    let has_event_export = Path::new(EVENT_EXPORT_DIRECTORY).exists()
        && exported_event_ids().is_ok_and(|event_ids| !event_ids.is_empty());

    if has_project_manifest && has_generated_artifacts && !has_event_export {
        return Err(format!(
            "pre-release upgrade required: generated artifacts exist without exported events in {EVENT_EXPORT_DIRECTORY}"
        ));
    }

    Ok(())
}

fn exported_event_ids() -> Result<Vec<ExportedEventId>, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut event_ids = exported_event_headers_in_topological_order(path)?
        .into_iter()
        .map(|event| event.event_id)
        .collect::<Vec<_>>();
    event_ids.sort();
    Ok(event_ids)
}

#[derive(Debug)]
struct EventConflict {
    id: String,
    stream_id: String,
    event_type: String,
    semantic_key: String,
    event_ids: BTreeSet<ExportedEventId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ConflictKey {
    stream_id: String,
    event_type: String,
    semantic_key: String,
    parents: Vec<ExportedEventId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ConflictPayload {
    event_id: ExportedEventId,
    payload: String,
}

fn event_conflicts(path: &Path) -> Result<Vec<EventConflict>, String> {
    let events = exported_events_in_topological_order(path)?;
    let resolved_conflicts = resolved_conflict_ids(&events)?;
    let mut grouped = BTreeMap::<ConflictKey, BTreeSet<ConflictPayload>>::new();

    for event in events {
        if let Some(key) = conflict_key(&event)? {
            grouped.entry(key).or_default().insert(ConflictPayload {
                event_id: event.event_id().clone(),
                payload: canonical_payload(&event)?,
            });
        }
    }

    Ok(grouped
        .into_iter()
        .filter_map(|(key, payloads)| {
            let id = conflict_id(&key);
            let distinct_payload_count = payloads
                .iter()
                .map(|payload| payload.payload.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            (distinct_payload_count > 1 && !resolved_conflicts.contains(&id)).then(|| {
                EventConflict {
                    id,
                    stream_id: key.stream_id,
                    event_type: key.event_type,
                    semantic_key: key.semantic_key,
                    event_ids: payloads
                        .into_iter()
                        .map(|payload| payload.event_id)
                        .collect::<BTreeSet<_>>(),
                }
            })
        })
        .collect())
}

fn resolved_conflict_ids(events: &[ExportedEvent]) -> Result<BTreeSet<String>, String> {
    Ok(events
        .iter()
        .filter_map(|event| match event.body() {
            ExportedEventBody::ConflictResolved { conflict_id, .. } => {
                Some(conflict_id.as_ref().to_owned())
            }
            _ => None,
        })
        .collect())
}

fn conflict_id(key: &ConflictKey) -> String {
    let parent_ids = key
        .parents
        .iter()
        .map(|parent| parent.as_ref())
        .collect::<Vec<_>>();
    hex::encode(Sha256::digest(
        serde_json::to_vec(&json!({
            "stream_id": key.stream_id,
            "type": key.event_type,
            "semantic_key": key.semantic_key,
            "parents": parent_ids,
        }))
        .unwrap_or_default(),
    ))
}

fn conflict_key(event: &ExportedEvent) -> Result<Option<ConflictKey>, String> {
    let semantic_key = match event.body() {
        ExportedEventBody::WorkflowUpdated { workflow } => workflow.slug().as_ref().to_owned(),
        ExportedEventBody::SliceUpdated { slice } => slice.slug().as_ref().to_owned(),
        _ => return Ok(None),
    };

    Ok(Some(ConflictKey {
        stream_id: event.stream_id().as_ref().to_owned(),
        event_type: event.event_type().as_ref().to_owned(),
        semantic_key,
        parents: event.parents().to_vec(),
    }))
}

fn parents(event: &Value) -> Result<Vec<ExportedEventId>, String> {
    let mut parents = event
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| "exported event is missing parents".to_owned())?
        .iter()
        .map(|parent| {
            parent
                .as_str()
                .ok_or_else(|| "exported event parent must be a string".to_owned())
                .and_then(|value| exported_event_id(value, "parents"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    parents.sort();
    Ok(parents)
}

fn parent_refs(parents: &[ExportedEventId]) -> Vec<&str> {
    parents
        .iter()
        .map(|parent| parent.as_ref())
        .collect::<Vec<_>>()
}

fn canonical_payload(event: &ExportedEvent) -> Result<String, String> {
    serde_json::to_string(&event.payload_json()).map_err(|error| error.to_string())
}

fn exported_events_in_topological_order(path: &Path) -> Result<Vec<ExportedEvent>, String> {
    let mut events = fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .map(|entry| {
            entry
                .map_err(|error| error.to_string())
                .and_then(|entry| read_event_file(&entry.path()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    events.sort_by(|left, right| {
        left.parents()
            .len()
            .cmp(&right.parents().len())
            .then(left.event_id().cmp(right.event_id()))
    });
    Ok(events)
}

fn exported_event_headers_in_topological_order(
    path: &Path,
) -> Result<Vec<ExportedEventHeader>, String> {
    let mut events = fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .map(|entry| {
            entry
                .map_err(|error| error.to_string())
                .and_then(|entry| read_event_header_file(&entry.path()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    events.sort_by(|left, right| {
        left.parent_count()
            .cmp(&right.parent_count())
            .then(left.event_id.cmp(&right.event_id))
    });
    Ok(events)
}

fn read_event_file(path: &Path) -> Result<ExportedEvent, String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    ExportedEvent::from_json_str(&contents)
}

fn read_event_header_file(path: &Path) -> Result<ExportedEventHeader, String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let value = serde_json::from_str::<Value>(&contents).map_err(|error| error.to_string())?;
    ExportedEventHeader::from_json_value(&value)
}

fn projection_fingerprint<T: ExportedEventFrontier>(events: &[T]) -> Result<String, String> {
    projection_fingerprint_digest(events).map(|digest| digest.as_ref().to_owned())
}

fn projection_fingerprint_digest<T: ExportedEventFrontier>(
    events: &[T],
) -> Result<ProjectionFingerprint, String> {
    let event_ids = events
        .iter()
        .filter(|event| event.frontier_event_type() != ExportedEventType::WorkflowReadinessDeclared)
        .map(ExportedEventFrontier::frontier_event_id)
        .map(|event_id| event_id.as_ref())
        .collect::<Vec<_>>();
    ArtifactDigest::try_new(hex::encode(Sha256::digest(
        serde_json::to_vec(&event_ids).map_err(|error| error.to_string())?,
    )))
    .map(ProjectionFingerprint::new)
    .map_err(|error| error.to_string())
}

fn command_ordinal_for_stream(stream_id: &str) -> Result<usize, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(0);
    }

    exported_event_headers_in_topological_order(path)?
        .into_iter()
        .try_fold(0_usize, |count, event| {
            let increments_count = event
                .stream_id
                .as_ref()
                .is_some_and(|event_stream_id| event_stream_id.as_ref() == stream_id);
            Ok(count + usize::from(increments_count))
        })
}

fn event_id(
    draft: &EventDraft,
    parents: &[ExportedEventId],
    command_ordinal: usize,
) -> Result<ExportedEventId, String> {
    let payload = draft.payload_json();
    let canonical = serde_json::to_vec(&json!({
        "schema_version": SCHEMA_VERSION,
        "command_ordinal": command_ordinal,
        "stream_id": draft.stream_id.as_ref(),
        "parents": parent_refs(parents),
        "type": draft.event_type().as_ref(),
        "payload": payload,
    }))
    .map_err(|error| error.to_string())?;
    ExportedEventId::try_new(hex::encode(Sha256::digest(canonical)))
}

#[derive(Debug)]
struct ProjectedModel {
    project_name: ProjectName,
    workflows: Vec<ProjectedWorkflow>,
    reviews: Vec<ProjectedReview>,
}

impl ProjectedModel {
    fn from_events(events: Vec<ExportedEvent>) -> Result<Self, String> {
        events
            .into_iter()
            .try_fold(
                None::<Self>,
                |model, event| -> Result<Option<Self>, String> {
                    match event.body {
                        ExportedEventBody::ProjectInitialized { name } => {
                            let (workflows, reviews) = model
                                .map(|model| (model.workflows, model.reviews))
                                .unwrap_or_default();
                            Ok(Some(Self {
                                project_name: name,
                                workflows,
                                reviews,
                            }))
                        }
                        ExportedEventBody::WorkflowAdded { workflow } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowAdded appeared before project initialization".to_owned()
                            })?;
                            model.workflows.push(ProjectedWorkflow {
                                slug: workflow.slug().clone(),
                                name: workflow.name().clone(),
                                description: workflow.description().clone(),
                                slices: Vec::new(),
                                command_errors: Vec::new(),
                                outcomes: Vec::new(),
                                owned_definitions: Vec::new(),
                                transitions: Vec::new(),
                                transition_evidences: Vec::new(),
                                requires_entry_lifecycle_coverage: false,
                                entry_lifecycle_states: Vec::new(),
                            });
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowUpdated { workflow } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowUpdated appeared before project initialization".to_owned()
                            })?;
                            let projected_workflow = model
                                .workflows
                                .iter_mut()
                                .find(|existing| existing.slug == *workflow.slug())
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowUpdated references unknown workflow {}",
                                        workflow.slug().as_ref()
                                    )
                                })?;
                            projected_workflow.name = workflow.name().clone();
                            projected_workflow.description = workflow.description().clone();
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowRemoved { slug } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowRemoved appeared before project initialization".to_owned()
                            })?;
                            let before = model.workflows.len();
                            model
                                .workflows
                                .retain(|workflow| workflow.slug != slug);
                            if model.workflows.len() == before {
                                return Err(format!(
                                    "WorkflowRemoved references unknown workflow {}",
                                    slug.as_ref()
                                ));
                            }
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceAdded { slice } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceAdded appeared before project initialization".to_owned()
                            })?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug == *slice.workflow_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceAdded references unknown workflow {}",
                                        slice.workflow_slug().as_ref()
                                    )
                                })?;
                            let relationship = if workflow.slices.is_empty() {
                                WorkflowStepRelationshipName::Entry
                            } else {
                                WorkflowStepRelationshipName::Main
                            };
                            workflow.slices.push(ProjectedSlice {
                                slug: slice.slug().clone(),
                                name: slice.name().clone(),
                                kind: slice.kind().into(),
                                description: slice.description().clone(),
                                relationship,
                                scenarios: Vec::new(),
                                outcomes: Vec::new(),
                                external_payloads: Vec::new(),
                                event_definitions: Vec::new(),
                                command_definitions: Vec::new(),
                                read_models: Vec::new(),
                                bit_level_data_flows: Vec::new(),
                                views: Vec::new(),
                                translations: Vec::new(),
                                automations: Vec::new(),
                                board_elements: Vec::new(),
                                board_connections: Vec::new(),
                            });
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceUpdated { slice } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceUpdated appeared before project initialization".to_owned()
                            })?;
                            let projected_slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|existing| existing.slug == *slice.slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceUpdated references unknown slice {}",
                                        slice.slug().as_ref()
                                    )
                                })?;
                            projected_slice.name = slice.name().clone();
                            projected_slice.kind = *slice.kind();
                            projected_slice.description = slice.description().clone();
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceRemoved { slug } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceRemoved appeared before project initialization".to_owned()
                            })?;
                            let removed_count = model
                                .workflows
                                .iter_mut()
                                .map(|workflow| {
                                    let before = workflow.slices.len();
                                    workflow
                                        .slices
                                        .retain(|slice| slice.slug != slug);
                                    workflow.transitions.retain(|transition| {
                                        transition.source().as_ref() != slug.as_ref()
                                            && transition.target().as_ref() != slug.as_ref()
                                    });
                                    before - workflow.slices.len()
                                })
                                .sum::<usize>();
                            if removed_count == 0 {
                                return Err(format!(
                                    "SliceRemoved references unknown slice {}",
                                    slug.as_ref()
                                ));
                            }
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceOutcomeAdded { outcome } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceOutcomeAdded appeared before project initialization".to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *outcome.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceOutcomeAdded references unknown slice {}",
                                        outcome.slice_slug().as_ref()
                                    )
                                })?;
                            slice.outcomes.push(outcome);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceScenarioAdded { scenario } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceScenarioAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *scenario.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceScenarioAdded references unknown slice {}",
                                        scenario.slice_slug().as_ref()
                                    )
                                })?;
                            slice.scenarios.push(scenario);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceExternalPayloadAdded { external_payload } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceExternalPayloadAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *external_payload.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceExternalPayloadAdded references unknown slice {}",
                                        external_payload.slice_slug().as_ref()
                                    )
                                })?;
                            slice.external_payloads.push(external_payload);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceEventDefinitionAdded { event } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceEventDefinitionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *event.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceEventDefinitionAdded references unknown slice {}",
                                        event.slice_slug().as_ref()
                                    )
                                })?;
                            slice.event_definitions.push(event);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceCommandDefinitionAdded { command } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceCommandDefinitionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *command.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceCommandDefinitionAdded references unknown slice {}",
                                        command.slice_slug().as_ref()
                                    )
                                })?;
                            slice.command_definitions.push(command);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceReadModelAdded { read_model } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceReadModelAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *read_model.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceReadModelAdded references unknown slice {}",
                                        read_model.slice_slug().as_ref()
                                    )
                                })?;
                            slice.read_models.push(read_model);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceBitLevelDataFlowAdded { data_flow } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceBitLevelDataFlowAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *data_flow.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceBitLevelDataFlowAdded references unknown slice {}",
                                        data_flow.slice_slug().as_ref()
                                    )
                                })?;
                            slice.bit_level_data_flows.push(data_flow);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceViewAdded { view } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceViewAdded appeared before project initialization".to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *view.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceViewAdded references unknown slice {}",
                                        view.slice_slug().as_ref()
                                    )
                                })?;
                            slice.views.push(view);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceTranslationAdded { translation } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceTranslationAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *translation.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceTranslationAdded references unknown slice {}",
                                        translation.slice_slug().as_ref()
                                    )
                                })?;
                            slice.translations.push(translation);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceAutomationAdded { automation } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceAutomationAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *automation.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceAutomationAdded references unknown slice {}",
                                        automation.slice_slug().as_ref()
                                    )
                                })?;
                            slice.automations.push(automation);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceBoardElementAdded { element } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceBoardElementAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *element.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceBoardElementAdded references unknown slice {}",
                                        element.slice_slug().as_ref()
                                    )
                                })?;
                            slice.board_elements.push(element);
                            Ok(Some(model))
                        }
                        ExportedEventBody::SliceBoardConnectionAdded { connection } => {
                            let mut model = model.ok_or_else(|| {
                                "SliceBoardConnectionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug == *connection.slice_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "SliceBoardConnectionAdded references unknown slice {}",
                                        connection.slice_slug().as_ref()
                                    )
                                })?;
                            slice.board_connections.push(connection);
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowReadinessDeclared { .. } => {
                            let model = model.ok_or_else(|| {
                                "WorkflowReadinessDeclared appeared before project initialization"
                                    .to_owned()
                            })?;
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowConnected { connection } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowConnected appeared before project initialization"
                                    .to_owned()
                            })?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug == *connection.workflow_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowConnected references unknown workflow {}",
                                        connection.workflow_slug().as_ref()
                                    )
                                })?;
                            workflow
                                .transitions
                                .push(workflow_transition_record_from_connection(&connection)?);
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowOutcomeAdded { workflow, outcome } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowOutcomeAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let projected_workflow = model
                                .workflows
                                .iter_mut()
                                .find(|existing| existing.slug == workflow)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowOutcomeAdded references unknown workflow {}",
                                        workflow.as_ref()
                                    )
                                })?;
                            projected_workflow.outcomes.push(outcome);
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowCommandErrorAdded { workflow, error } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowCommandErrorAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let projected_workflow = model
                                .workflows
                                .iter_mut()
                                .find(|existing| existing.slug == workflow)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowCommandErrorAdded references unknown workflow {}",
                                        workflow.as_ref()
                                    )
                                })?;
                            projected_workflow.command_errors.push(error);
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowOwnedDefinitionAdded {
                            workflow,
                            definition,
                        } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowOwnedDefinitionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let projected_workflow = model
                                .workflows
                                .iter_mut()
                                .find(|existing| existing.slug == workflow)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowOwnedDefinitionAdded references unknown workflow {}",
                                        workflow.as_ref()
                                    )
                                })?;
                            projected_workflow.owned_definitions.push(definition);
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowTransitionEvidenceAdded { workflow, evidence } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowTransitionEvidenceAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let projected_workflow = model
                                .workflows
                                .iter_mut()
                                .find(|existing| existing.slug == workflow)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowTransitionEvidenceAdded references unknown workflow {}",
                                        workflow.as_ref()
                                    )
                                })?;
                            projected_workflow.transition_evidences.push(evidence);
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowEntryLifecycleCoverageRequired { workflow } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowEntryLifecycleCoverageRequired appeared before project initialization"
                                    .to_owned()
                            })?;
                            let projected_workflow = model
                                .workflows
                                .iter_mut()
                                .find(|existing| existing.slug == workflow)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowEntryLifecycleCoverageRequired references unknown workflow {}",
                                        workflow.as_ref()
                                    )
                                })?;
                            projected_workflow.requires_entry_lifecycle_coverage = true;
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowEntryLifecycleStateAdded { workflow, coverage } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowEntryLifecycleStateAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let projected_workflow = model
                                .workflows
                                .iter_mut()
                                .find(|existing| existing.slug == workflow)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowEntryLifecycleStateAdded references unknown workflow {}",
                                        workflow.as_ref()
                                    )
                                })?;
                            projected_workflow.entry_lifecycle_states.push(coverage);
                            Ok(Some(model))
                        }
                        ExportedEventBody::WorkflowTransitionRemoved { removal } => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowTransitionRemoved appeared before project initialization"
                                    .to_owned()
                            })?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug == *removal.workflow_slug())
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowTransitionRemoved references unknown workflow {}",
                                        removal.workflow_slug().as_ref()
                                    )
                                })?;
                            let removed_transition =
                                workflow_transition_record_from_removal(&removal)?;
                            workflow
                                .transitions
                                .retain(|transition| !same_transition(transition, &removed_transition));
                            Ok(Some(model))
                        }
                        ExportedEventBody::ReviewRecorded {
                            workflow_slug,
                            model_content_digest,
                            reviewer_id,
                            reviewed_at,
                            categories,
                        } => {
                            let mut model = model.ok_or_else(|| {
                                "ReviewRecorded appeared before project initialization".to_owned()
                            })?;
                            if !model
                                .workflows
                                .iter()
                                .any(|workflow| workflow.slug == workflow_slug)
                            {
                                return Err(format!(
                                    "ReviewRecorded references unknown workflow {}",
                                    workflow_slug.as_ref()
                                ));
                            }
                            let review = ProjectedReview {
                                workflow_slug,
                                model_content_digest,
                                reviewer_id: reviewer_id.as_ref().to_owned(),
                                reviewed_at: reviewed_at.as_ref().to_owned(),
                                categories: categories
                                    .into_iter()
                                    .map(|category| category.as_ref().to_owned())
                                    .collect(),
                            };
                            model.reviews.retain(|existing| {
                                existing.workflow_slug.as_ref() != review.workflow_slug.as_ref()
                            });
                            model.reviews.push(review);
                            Ok(Some(model))
                        }
                        ExportedEventBody::ConflictResolved { .. } => Ok(model),
                    }
                },
            )?
            .ok_or_else(|| "exported events are missing ProjectInitialized".to_owned())
    }

    fn effects(self, projection_fingerprint: String) -> Result<EffectPlan, String> {
        let project_module_name = module_name(self.project_name.as_ref());
        let workflow_slugs = self
            .workflows
            .iter()
            .map(|workflow| workflow.slug.clone())
            .collect::<Vec<_>>();
        let slice_memberships = self
            .workflows
            .iter()
            .flat_map(ProjectedWorkflow::slice_memberships)
            .collect::<Vec<_>>();

        let mut effects = vec![
            Effect::write_file(
                project_path(PROJECTION_FINGERPRINT_PATH)?,
                file_contents(format!("{projection_fingerprint}\n"))?,
            ),
            Effect::write_file(
                project_path("emc.toml")?,
                file_contents(format!(
                    "[project]\nname = \"{}\"\nversion = \"0.1.0\"\nlean_module = \"{project_module_name}\"\nquint_module = \"{project_module_name}\"\n",
                    self.project_name.as_ref()
                ))?,
            ),
            Effect::EnsureDirectory(project_path("model/lean")?),
            Effect::write_file(
                project_path("model/lean/lean-toolchain")?,
                file_contents("leanprover/lean4:4.29.1\n")?,
            ),
            Effect::write_file(
                project_path("model/lean/lakefile.lean")?,
                file_contents("import Lake\nopen Lake DSL\npackage EMCModel where\n")?,
            ),
            Effect::EnsureDirectory(project_path("model/lean/slices")?),
            Effect::write_file(
                project_path("model/lean/slices/.gitkeep")?,
                file_contents("\n")?,
            ),
            Effect::EnsureDirectory(project_path("model/quint")?),
            Effect::EnsureDirectory(project_path("model/quint/slices")?),
            Effect::write_file(
                project_path("model/quint/slices/.gitkeep")?,
                file_contents("\n")?,
            ),
            Effect::EnsureDirectory(project_path("reviews")?),
            Effect::write_file(project_path("reviews/.gitkeep")?, file_contents("\n")?),
        ];

        effects.extend(project_root_effects(
            self.project_name,
            &workflow_slugs,
            &slice_memberships,
        ));
        effects.extend(
            self.workflows
                .into_iter()
                .map(ProjectedWorkflow::effects)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten(),
        );
        effects.extend(
            self.reviews
                .into_iter()
                .map(ProjectedReview::effect)
                .collect::<Result<Vec<_>, _>>()?,
        );

        Ok(EffectPlan::new(effects))
    }
}

#[derive(Debug)]
struct ProjectedReview {
    workflow_slug: WorkflowSlug,
    model_content_digest: ModelContentDigest,
    reviewer_id: String,
    reviewed_at: String,
    categories: Vec<String>,
}

#[derive(Debug)]
struct WorkflowReadinessDeclaration {
    workflow: WorkflowSlug,
    projection_fingerprint: ProjectionFingerprint,
}

impl ProjectedReview {
    fn effect(self) -> Result<Effect, String> {
        let category_results =
            self.categories
                .into_iter()
                .fold(Map::new(), |mut results, category| {
                    results.insert(category, json!("clean"));
                    results
                });
        let document = json!({
            "workflow_slug": self.workflow_slug.as_ref(),
            "model_content_digest": self.model_content_digest.as_ref(),
            "reviewer_id": self.reviewer_id,
            "status": "clean",
            "category_results": category_results,
            "mandatory_findings": [],
            "reviewed_at": self.reviewed_at
        });
        let contents =
            serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?;
        Ok(Effect::write_file(
            project_path(format!(
                "reviews/{}.review.json",
                self.workflow_slug.as_ref()
            ))?,
            file_contents(format!("{contents}\n"))?,
        ))
    }
}

#[derive(Debug)]
struct ProjectedWorkflow {
    slug: WorkflowSlug,
    name: ModelName,
    description: ModelDescription,
    slices: Vec<ProjectedSlice>,
    command_errors: Vec<WorkflowCommandErrorRecord>,
    outcomes: Vec<WorkflowOutcomeRecord>,
    owned_definitions: Vec<WorkflowOwnedDefinitionRecord>,
    transitions: Vec<WorkflowTransitionRecord>,
    transition_evidences: Vec<WorkflowTransitionEvidenceRecord>,
    requires_entry_lifecycle_coverage: bool,
    entry_lifecycle_states: Vec<WorkflowEntryLifecycleStateRecord>,
}

impl ProjectedWorkflow {
    fn slice_details(&self) -> Vec<WorkflowSliceDetail> {
        self.slices
            .iter()
            .map(ProjectedSlice::slice_detail)
            .collect()
    }

    fn slice_memberships(&self) -> Vec<ProjectSliceMembership> {
        self.slices
            .iter()
            .map(|slice| {
                ProjectSliceMembership::new(
                    self.slug.clone(),
                    slice.slug.clone(),
                    lean_module_name(module_name(slice.name.as_ref())),
                )
            })
            .collect()
    }

    fn effects(self) -> Result<Vec<Effect>, String> {
        let module_name = module_name(self.name.as_ref());
        let slice_details = WorkflowSliceDetails::from_details(self.slice_details());
        let digest = artifact_digest(WorkflowArtifactDigestInput {
            workflow_name: self.name.clone(),
            workflow_slug: self.slug.clone(),
            workflow_description: self.description.clone(),
            workflow_slice_details: slice_details.clone(),
            workflow_transitions: WorkflowTransitionRecords::from_records(self.transitions.clone()),
            workflow_outcomes: WorkflowOutcomeRecords::from_records(self.outcomes.clone()),
            workflow_command_errors: WorkflowCommandErrorRecords::from_records(
                self.command_errors.clone(),
            ),
            workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records(
                self.owned_definitions.clone(),
            ),
            workflow_transition_evidences: WorkflowTransitionEvidenceRecords::from_records(
                self.transition_evidences.clone(),
            ),
            workflow_requires_entry_lifecycle_coverage: self.requires_entry_lifecycle_coverage,
            workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords::from_records(
                self.entry_lifecycle_states.clone(),
            ),
        });
        let workflow_data = WorkflowModuleData::new(self.name, self.description, self.slug, digest)
            .with_slice_details(slice_details)
            .with_transitions(WorkflowTransitionRecords::from_records(self.transitions))
            .with_outcomes(WorkflowOutcomeRecords::from_records(self.outcomes))
            .with_command_errors(WorkflowCommandErrorRecords::from_records(
                self.command_errors,
            ))
            .with_owned_definitions(WorkflowOwnedDefinitionRecords::from_records(
                self.owned_definitions,
            ))
            .with_transition_evidences(WorkflowTransitionEvidenceRecords::from_records(
                self.transition_evidences,
            ))
            .with_entry_lifecycle_required(self.requires_entry_lifecycle_coverage)
            .with_entry_lifecycle_states(WorkflowEntryLifecycleStateRecords::from_records(
                self.entry_lifecycle_states,
            ));
        let mut effects = vec![
            Effect::write_file(
                project_path(format!("model/lean/{module_name}.lean"))?,
                emit_lean_workflow_module(
                    lean_module_name(module_name.clone()),
                    workflow_data.clone(),
                ),
            ),
            Effect::write_file(
                project_path(format!("model/quint/{module_name}.qnt"))?,
                emit_quint_workflow_module(quint_module_name(module_name), workflow_data),
            ),
        ];

        effects.extend(
            self.slices
                .into_iter()
                .map(ProjectedSlice::effects)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten(),
        );

        Ok(effects)
    }
}

#[derive(Debug)]
struct ProjectedSlice {
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKindName,
    description: ModelDescription,
    relationship: WorkflowStepRelationshipName,
    scenarios: Vec<NewSliceScenario>,
    outcomes: Vec<NewOutcomeDefinition>,
    external_payloads: Vec<NewExternalPayloadDefinition>,
    event_definitions: Vec<NewEventDefinition>,
    command_definitions: Vec<NewCommandDefinition>,
    read_models: Vec<NewReadModelDefinition>,
    bit_level_data_flows: Vec<NewBitLevelDataFlow>,
    views: Vec<NewViewDefinition>,
    translations: Vec<NewTranslationDefinition>,
    automations: Vec<NewAutomationDefinition>,
    board_elements: Vec<NewBoardElement>,
    board_connections: Vec<NewBoardConnection>,
}

impl ProjectedSlice {
    fn slice_detail(&self) -> WorkflowSliceDetail {
        WorkflowSliceDetail::new_with_relationship(
            self.slug.clone(),
            self.name.clone(),
            self.kind,
            self.description.clone(),
            self.relationship,
        )
    }

    fn effects(self) -> Result<Vec<Effect>, String> {
        let module_name = module_name(self.name.as_ref());
        let digest = slice_artifact_digest(
            self.name.clone(),
            self.slug.clone(),
            self.kind,
            self.description.clone(),
        );
        let mut effects = vec![
            Effect::write_file(
                project_path(format!("model/lean/slices/{module_name}.lean"))?,
                emit_lean_slice_module(
                    lean_module_name(module_name.clone()),
                    self.name.clone(),
                    self.description.clone(),
                    self.slug.clone(),
                    self.kind,
                    digest.clone(),
                ),
            ),
            Effect::write_file(
                project_path(format!("model/quint/slices/{module_name}.qnt"))?,
                emit_quint_slice_module(
                    quint_module_name(module_name),
                    self.name,
                    self.description,
                    self.slug,
                    self.kind,
                    digest,
                ),
            ),
        ];
        effects.extend(
            self.scenarios
                .into_iter()
                .map(Effect::AddSliceScenarioFromSlice),
        );
        effects.extend(
            self.outcomes
                .into_iter()
                .map(Effect::AddOutcomeDefinitionFromSlice),
        );
        effects.extend(
            self.external_payloads
                .into_iter()
                .map(Effect::AddExternalPayloadDefinitionFromSlice),
        );
        effects.extend(
            self.event_definitions
                .into_iter()
                .map(Effect::AddEventDefinitionFromSlice),
        );
        effects.extend(
            self.command_definitions
                .into_iter()
                .map(Effect::AddCommandDefinitionFromSlice),
        );
        effects.extend(
            self.read_models
                .into_iter()
                .map(Effect::AddReadModelDefinitionFromSlice),
        );
        effects.extend(
            self.bit_level_data_flows
                .into_iter()
                .map(Effect::AddBitLevelDataFlowFromSlice),
        );
        effects.extend(
            self.views
                .into_iter()
                .map(Effect::AddViewDefinitionFromSlice),
        );
        effects.extend(
            self.translations
                .into_iter()
                .map(Effect::AddTranslationDefinitionFromSlice),
        );
        effects.extend(
            self.automations
                .into_iter()
                .map(Effect::AddAutomationDefinitionFromSlice),
        );
        effects.extend(
            self.board_elements
                .into_iter()
                .map(Effect::AddBoardElementFromSlice),
        );
        effects.extend(
            self.board_connections
                .into_iter()
                .map(Effect::AddBoardConnectionFromSlice),
        );
        Ok(effects)
    }
}

fn required_object<'event>(event: &'event Value, field: &str) -> Result<&'event Value, String> {
    event
        .get(field)
        .filter(|value| value.is_object())
        .ok_or_else(|| format!("exported event is missing object field {field}"))
}

fn required_str<'event>(event: &'event Value, field: &str) -> Result<&'event str, String> {
    event
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("exported event is missing string field {field}"))
}

fn required_string_array(event: &Value, field: &str) -> Result<Vec<String>, String> {
    event
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("exported event is missing string array field {field}"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("exported event field {field} must contain only strings"))
        })
        .collect()
}

fn required_array<'event>(event: &'event Value, field: &str) -> Result<&'event Vec<Value>, String> {
    event
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("exported event is missing array field {field}"))
}

fn optional_str<'event>(event: &'event Value, field: &str) -> Result<Option<&'event str>, String> {
    match event.get(field) {
        Some(Value::Null) | None => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or_else(|| format!("exported event field {field} must be a string or null")),
    }
}

fn required_bool(event: &Value, field: &str) -> Result<bool, String> {
    event
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("exported event is missing boolean field {field}"))
}

fn required_usize(event: &Value, field: &str) -> Result<usize, String> {
    let value = event
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("exported event is missing unsigned integer field {field}"))?;
    usize::try_from(value).map_err(|_| format!("exported event field {field} is too large"))
}

fn required_event_stream_id(event: &Value, field: &str) -> Result<EventStreamId, String> {
    EventStreamId::try_new(required_str(event, field)?.to_owned()).map_err(|error| {
        format!("exported event JSON field {field} contains an invalid stream id: {error}")
    })
}

fn required_exported_event_id(event: &Value, field: &str) -> Result<ExportedEventId, String> {
    exported_event_id(required_str(event, field)?, field)
}

fn exported_event_id(value: &str, field: &str) -> Result<ExportedEventId, String> {
    ExportedEventId::try_new(value.to_owned()).map_err(|error| {
        format!("exported event JSON field {field} contains an invalid id: {error}")
    })
}

fn required_exported_command_id(event: &Value, field: &str) -> Result<ExportedCommandId, String> {
    ExportedCommandId::try_new(required_str(event, field)?.to_owned()).map_err(|error| {
        format!("exported event JSON field {field} contains an invalid id: {error}")
    })
}

fn slice_from_payload(payload: &Value) -> Result<NewSlice, String> {
    Ok(NewSlice::new(
        workflow_slug(required_str(payload, "workflow")?)?,
        slice_slug(required_str(payload, "slug")?)?,
        model_name(required_str(payload, "name")?)?,
        model_description(required_str(payload, "description")?)?,
        slice_kind_name(required_str(payload, "kind")?)?.into(),
    ))
}

fn workflow_slice_detail_from_payload(payload: &Value) -> Result<WorkflowSliceDetail, String> {
    Ok(WorkflowSliceDetail::new(
        slice_slug(required_str(payload, "slug")?)?,
        model_name(required_str(payload, "name")?)?,
        slice_kind_name(required_str(payload, "kind")?)?,
        model_description(required_str(payload, "description")?)?,
    ))
}

fn projection_fingerprint_from_payload(payload: &Value) -> Result<ProjectionFingerprint, String> {
    Ok(ProjectionFingerprint::new(artifact_digest_from_str(
        required_str(payload, "projection_fingerprint")?,
    )?))
}

fn model_content_digest_from_payload(payload: &Value) -> Result<ModelContentDigest, String> {
    Ok(ModelContentDigest::new(artifact_digest_from_str(
        required_str(payload, "model_content_digest")?,
    )?))
}

fn review_event_reference_from_payload(payload: &Value) -> Result<ReviewEventReference, String> {
    let review_event_id = optional_str(payload, "review_event_id")?
        .map(artifact_digest_from_str)
        .transpose()?
        .map(ReviewEventId::new);
    Ok(ReviewEventReference::from_optional(review_event_id))
}

fn review_rule_names_from_payload(payload: &Value) -> Result<Vec<ReviewRuleName>, String> {
    required_string_array(payload, "categories")?
        .into_iter()
        .map(|category| ReviewRuleName::try_new(category).map_err(|error| error.to_string()))
        .collect()
}

fn workflow_connection_from_payload(payload: &Value) -> Result<WorkflowConnection, String> {
    let workflow = workflow_slug(required_str(payload, "workflow")?)?;
    let source = slice_slug(required_str(payload, "from")?)?;
    let target_slice = optional_str(payload, "to")?.map(slice_slug).transpose()?;
    let target_workflow = optional_str(payload, "to_workflow")?
        .map(workflow_slug)
        .transpose()?;
    let kind = connection_kind(required_str(payload, "via")?)?;
    let trigger = transition_trigger_name(required_str(payload, "name")?)?;
    let payload_contract = optional_str(payload, "payload_contract")?
        .map(payload_contract_name)
        .transpose()?;
    let reason = optional_str(payload, "reason")?
        .map(model_description)
        .transpose()?;

    match (target_slice, target_workflow, payload_contract, reason) {
        (Some(target), None, None, None) => Ok(WorkflowConnection::new(
            workflow, source, target, kind, trigger,
        )),
        (Some(target), None, Some(payload_contract), None) => {
            Ok(WorkflowConnection::new_with_payload_contract(
                workflow,
                source,
                target,
                kind,
                trigger,
                payload_contract,
            ))
        }
        (None, Some(target), None, Some(reason)) => Ok(WorkflowConnection::new_workflow_exit(
            workflow, source, target, kind, trigger, reason,
        )),
        _ => Err("WorkflowConnected has incompatible target fields".to_owned()),
    }
}

fn workflow_transition_removal_from_payload(
    payload: &Value,
) -> Result<WorkflowTransitionRemoval, String> {
    let workflow = workflow_slug(required_str(payload, "workflow")?)?;
    let source = slice_slug(required_str(payload, "from")?)?;
    let target_slice = optional_str(payload, "to")?.map(slice_slug).transpose()?;
    let target_workflow = optional_str(payload, "to_workflow")?
        .map(workflow_slug)
        .transpose()?;
    let kind = connection_kind(required_str(payload, "via")?)?;
    let trigger = transition_trigger_name(required_str(payload, "name")?)?;

    match (target_slice, target_workflow) {
        (Some(target), None) => Ok(WorkflowTransitionRemoval::new(
            workflow, source, target, kind, trigger,
        )),
        (None, Some(target)) => Ok(WorkflowTransitionRemoval::new_workflow_exit(
            workflow, source, target, kind, trigger,
        )),
        _ => Err("WorkflowTransitionRemoved has incompatible target fields".to_owned()),
    }
}

fn workflow_transition_record_from_connection(
    connection: &WorkflowConnection,
) -> Result<WorkflowTransitionRecord, String> {
    let source = workflow_transition_endpoint(connection.source().as_ref())?;
    let target = workflow_transition_endpoint(connection.target().as_ref())?;
    let kind = workflow_transition_kind_from_connection(
        connection.target().workflow_slug().is_some(),
        connection.kind(),
    )?;

    match (connection.payload_contract(), connection.target().reason()) {
        (Some(payload_contract), None) => Ok(WorkflowTransitionRecord::new_with_payload_contract(
            source,
            target,
            kind,
            connection.trigger().clone(),
            payload_contract.clone(),
        )),
        (None, Some(reason)) => Ok(WorkflowTransitionRecord::new_with_rationale(
            source,
            target,
            kind,
            connection.trigger().clone(),
            reason.clone(),
        )),
        (None, None) => Ok(WorkflowTransitionRecord::new(
            source,
            target,
            kind,
            connection.trigger().clone(),
        )),
        (Some(_), Some(_)) => {
            Err("WorkflowConnected cannot project both rationale and payload contract".to_owned())
        }
    }
}

fn workflow_transition_record_from_removal(
    removal: &WorkflowTransitionRemoval,
) -> Result<WorkflowTransitionRecord, String> {
    Ok(WorkflowTransitionRecord::new(
        workflow_transition_endpoint(removal.source().as_ref())?,
        workflow_transition_endpoint(removal.target().as_ref())?,
        workflow_transition_kind_from_connection(
            removal.target().workflow_slug().is_some(),
            removal.kind(),
        )?,
        removal.trigger().clone(),
    ))
}

fn workflow_transition_kind_from_connection(
    workflow_exit: bool,
    kind: ConnectionKind,
) -> Result<WorkflowTransitionKind, String> {
    let raw_kind = if workflow_exit {
        format!("workflow_exit:{}", kind.trigger_kind())
    } else {
        kind.trigger_kind().to_owned()
    };
    workflow_transition_kind(&raw_kind)
}

fn workflow_outcome_from_payload(payload: &Value) -> Result<WorkflowOutcomeRecord, String> {
    Ok(WorkflowOutcomeRecord::new(
        workflow_transition_endpoint(required_str(payload, "source_slice")?)?,
        outcome_label_name(required_str(payload, "label")?)?,
        required_bool(payload, "externally_relevant")?,
    ))
}

pub(crate) fn slice_outcome_from_payload(payload: &Value) -> Result<NewOutcomeDefinition, String> {
    Ok(NewOutcomeDefinition::new(
        slice_slug(required_str(payload, "slice")?)?,
        outcome_label_name(required_str(payload, "label")?)?,
        OutcomeEventNames::from_events(
            required_string_array(payload, "events")?
                .into_iter()
                .map(|event| event_name(&event))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        required_bool(payload, "externally_relevant")?,
    ))
}

pub(crate) fn slice_scenario_from_payload(payload: &Value) -> Result<NewSliceScenario, String> {
    let kind = scenario_kind(required_str(payload, "kind")?)?;
    let slice_slug = slice_slug(required_str(payload, "slice")?)?;
    let name = scenario_name(required_str(payload, "name")?)?;
    let given = scenario_step_text(required_str(payload, "given")?)?;
    let when = scenario_step_text(required_str(payload, "when")?)?;
    let then = scenario_step_text(required_str(payload, "then")?)?;
    let scenario = match (
        kind,
        optional_str(payload, "contract_kind")?,
        optional_str(payload, "covered_definition")?,
    ) {
        (ScenarioKind::Acceptance, None, None) => {
            NewSliceScenario::new(slice_slug, kind, name, given, when, then)
        }
        (ScenarioKind::Contract, Some(contract_kind), Some(covered_definition)) => {
            NewSliceScenario::new_contract(
                slice_slug,
                name,
                given,
                when,
                then,
                contract_kind_name(contract_kind)?,
                covered_definition_name(covered_definition)?,
            )
        }
        _ => return Err("SliceScenarioAdded has incompatible scenario kind fields".to_owned()),
    };

    Ok(scenario
        .with_streams(
            scenario_stream_names(required_string_array(payload, "read_streams")?)?,
            scenario_stream_names(required_string_array(payload, "written_streams")?)?,
        )
        .with_error_references(command_error_names(required_string_array(
            payload,
            "error_references",
        )?)?))
}

pub(crate) fn slice_external_payload_from_payload(
    payload: &Value,
) -> Result<NewExternalPayloadDefinition, String> {
    Ok(NewExternalPayloadDefinition::new(
        slice_slug(required_str(payload, "slice")?)?,
        event_attribute_source_name(required_str(payload, "name")?)?,
        event_attribute_source_field(required_str(payload, "field")?)?,
        provenance_description(required_str(payload, "field_provenance")?)?,
        bit_encoding_semantics(required_str(payload, "bit_encoding")?)?,
    ))
}

pub(crate) fn slice_event_definition_from_payload(
    payload: &Value,
) -> Result<NewEventDefinition, String> {
    let attribute_payload = required_object(payload, "attribute")?;
    let generated_source_kind = optional_str(attribute_payload, "generated_source_kind")?
        .map(generated_event_attribute_source_kind)
        .transpose()?;
    let attribute = match generated_source_kind {
        Some(generated_source_kind) => NewEventAttribute::new_with_generated_source_kind(
            event_attribute_name(required_str(attribute_payload, "name")?)?,
            event_attribute_source_kind(required_str(attribute_payload, "source_kind")?)?,
            event_attribute_source_name(required_str(attribute_payload, "source_name")?)?,
            event_attribute_source_field(required_str(attribute_payload, "source_field")?)?,
            generated_source_kind,
            provenance_description(required_str(attribute_payload, "provenance")?)?,
        ),
        None => NewEventAttribute::new(
            event_attribute_name(required_str(attribute_payload, "name")?)?,
            event_attribute_source_kind(required_str(attribute_payload, "source_kind")?)?,
            event_attribute_source_name(required_str(attribute_payload, "source_name")?)?,
            event_attribute_source_field(required_str(attribute_payload, "source_field")?)?,
            provenance_description(required_str(attribute_payload, "provenance")?)?,
        ),
    };
    let slice_slug = slice_slug(required_str(payload, "slice")?)?;
    let name = event_name(required_str(payload, "name")?)?;
    let stream = stream_name(required_str(payload, "stream")?)?;

    match (
        required_bool(payload, "observed")?,
        required_bool(payload, "shared")?,
    ) {
        (false, false) => Ok(NewEventDefinition::new(slice_slug, name, stream, attribute)),
        (true, false) => Ok(NewEventDefinition::new_observed(
            slice_slug, name, stream, attribute,
        )),
        (false, true) => Ok(NewEventDefinition::new_shared(
            slice_slug, name, stream, attribute,
        )),
        (true, true) => {
            Err("SliceEventDefinitionAdded cannot be both observed and shared".to_owned())
        }
    }
}

pub(crate) fn slice_command_definition_from_payload(
    payload: &Value,
) -> Result<NewCommandDefinition, String> {
    let input_payload = required_object(payload, "input")?;
    let source_kind = command_input_source_kind(required_str(input_payload, "source_kind")?)?;
    let input = NewCommandInput::new(
        datum_name(required_str(input_payload, "name")?)?,
        command_input_source_from_payload(source_kind, input_payload)?,
        command_input_source_description(required_str(input_payload, "source_description")?)?,
        CommandInputProvenanceChain::from_hops(
            required_string_array(input_payload, "provenance_chain")?
                .into_iter()
                .map(|hop| source_chain_hop(&hop))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );

    let mut command = NewCommandDefinition::new(
        slice_slug(required_str(payload, "slice")?)?,
        command_name(required_str(payload, "name")?)?,
        input,
        EmittedEventNames::from_events(
            required_string_array(payload, "emitted_events")?
                .into_iter()
                .map(|event| event_name(&event))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    )
    .with_observed_streams(CommandObservedStreams::from_streams(
        required_string_array(payload, "observed_streams")?
            .into_iter()
            .map(|stream| stream_name(&stream))
            .collect::<Result<Vec<_>, _>>()?,
    ))
    .with_errors(CommandErrorDefinitions::from_errors(
        required_array(payload, "errors")?
            .iter()
            .map(command_error_definition_from_payload)
            .collect::<Result<Vec<_>, _>>()?,
    ));

    if let Some(repeat_behavior) = optional_str(payload, "singleton_repeat_behavior")? {
        command =
            command.with_singleton_repeat_behavior(singleton_repeat_behavior(repeat_behavior)?);
    }

    Ok(command)
}

fn command_input_source_from_payload(
    source_kind: CommandInputSourceKind,
    input_payload: &Value,
) -> Result<CommandInputSource, String> {
    let event_stream = optional_event_stream_source(input_payload)?;
    let external_payload = optional_source_field_pair(
        input_payload,
        "external_payload_source_name",
        "external_payload_source_field",
    )?;
    let generated = optional_source_field_pair(
        input_payload,
        "generated_source_name",
        "generated_source_field",
    )?;
    let session =
        optional_source_field_pair(input_payload, "session_source_name", "session_source_field")?;
    let invocation_argument = optional_source_field_pair(
        input_payload,
        "invocation_argument_source_name",
        "invocation_argument_source_field",
    )?;

    match (
        source_kind,
        event_stream,
        external_payload,
        generated,
        session,
        invocation_argument,
    ) {
        (CommandInputSourceKind::Actor, None, None, None, None, None) => {
            Ok(CommandInputSource::actor())
        }
        (
            CommandInputSourceKind::EventStreamState,
            Some((event, attribute)),
            None,
            None,
            None,
            None,
        ) => Ok(CommandInputSource::event_stream_state(event, attribute)),
        (
            CommandInputSourceKind::ExternalPayload,
            None,
            Some((payload, field)),
            None,
            None,
            None,
        ) => Ok(CommandInputSource::external_payload(payload, field)),
        (CommandInputSourceKind::Generated, None, None, Some((source, field)), None, None) => {
            Ok(CommandInputSource::generated(source, field))
        }
        (CommandInputSourceKind::Session, None, None, None, Some((source, field)), None) => {
            Ok(CommandInputSource::session(source, field))
        }
        (
            CommandInputSourceKind::InvocationArgument,
            None,
            None,
            None,
            None,
            Some((argument, field)),
        ) => Ok(CommandInputSource::invocation_argument(argument, field)),
        _ => Err(
            "command input source kind and source coordinates must describe the same source"
                .to_owned(),
        ),
    }
}

fn optional_event_stream_source(
    payload: &Value,
) -> Result<Option<(EventName, EventAttributeName)>, String> {
    match (
        optional_str(payload, "event_stream_source_event")?,
        optional_str(payload, "event_stream_source_attribute")?,
    ) {
        (Some(event), Some(attribute)) => {
            Ok(Some((event_name(event)?, event_attribute_name(attribute)?)))
        }
        (None, None) => Ok(None),
        _ => Err("command input event stream source fields must be supplied together".to_owned()),
    }
}

fn optional_source_field_pair(
    payload: &Value,
    source_field: &str,
    field_field: &str,
) -> Result<Option<(EventAttributeSourceName, EventAttributeSourceField)>, String> {
    match (
        optional_str(payload, source_field)?,
        optional_str(payload, field_field)?,
    ) {
        (Some(source), Some(field)) => Ok(Some((
            event_attribute_source_name(source)?,
            event_attribute_source_field(field)?,
        ))),
        (None, None) => Ok(None),
        _ => Err(format!(
            "command input source fields {source_field} and {field_field} must be supplied together"
        )),
    }
}

fn command_error_definition_from_payload(
    payload: &Value,
) -> Result<NewCommandErrorDefinition, String> {
    Ok(NewCommandErrorDefinition::new(
        command_error_name(required_str(payload, "name")?)?,
        scenario_name(required_str(payload, "scenario")?)?,
        command_error_recovery_kind(required_str(payload, "recovery")?)?,
    ))
}

pub(crate) fn slice_read_model_from_payload(
    payload: &Value,
) -> Result<NewReadModelDefinition, String> {
    let field = read_model_field_from_payload(required_object(payload, "field")?)?;
    let mut read_model = NewReadModelDefinition::new(
        slice_slug(required_str(payload, "slice")?)?,
        read_model_name(required_str(payload, "name")?)?,
        field,
    );

    match (
        required_bool(payload, "transitive")?,
        required_string_array(payload, "relationship_fields")?,
        optional_str(payload, "transitive_rule")?,
        optional_str(payload, "example_scenario")?,
    ) {
        (false, relationship_fields, None, None) if relationship_fields.is_empty() => {}
        (true, relationship_fields, Some(transitive_rule), Some(example_scenario)) => {
            read_model = read_model.with_transitive_semantics(
                ReadModelRelationshipFields::from_fields(
                    relationship_fields
                        .into_iter()
                        .map(|field| datum_name(&field))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                read_model_transitive_rule(transitive_rule)?,
                scenario_name(example_scenario)?,
            );
        }
        _ => {
            return Err("SliceReadModelAdded has incompatible transitive fields".to_owned());
        }
    }

    Ok(read_model)
}

fn read_model_field_from_payload(payload: &Value) -> Result<NewReadModelField, String> {
    let name = datum_name(required_str(payload, "name")?)?;
    let source_kind = read_model_field_source_kind(required_str(payload, "source_kind")?)?;
    let provenance = provenance_description(required_str(payload, "provenance")?)?;
    let source = match (
        source_kind,
        optional_str(payload, "source_event")?,
        optional_str(payload, "source_attribute")?,
        optional_str(payload, "derivation_rule")?,
        required_string_array(payload, "derivation_source_fields")?,
        optional_str(payload, "absence_event")?,
        optional_str(payload, "derivation_scenario")?,
        optional_str(payload, "absence_scenario")?,
    ) {
        (
            ReadModelFieldSourceKind::EventAttribute,
            Some(source_event),
            Some(source_attribute),
            None,
            derivation_source_fields,
            None,
            None,
            None,
        ) if derivation_source_fields.is_empty() => ReadModelFieldSource::event_attribute(
            event_name(source_event)?,
            event_attribute_name(source_attribute)?,
        ),
        (
            ReadModelFieldSourceKind::Derivation,
            None,
            None,
            Some(derivation_rule),
            derivation_source_fields,
            None,
            Some(derivation_scenario),
            None,
        ) => ReadModelFieldSource::derivation(
            read_model_derivation_rule(derivation_rule)?,
            ReadModelDerivationSourceFields::from_fields(
                derivation_source_fields
                    .into_iter()
                    .map(|field| datum_name(&field))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            scenario_name(derivation_scenario)?,
        ),
        (
            ReadModelFieldSourceKind::AbsenceDefault,
            None,
            None,
            None,
            derivation_source_fields,
            Some(absence_event),
            None,
            Some(absence_scenario),
        ) if derivation_source_fields.is_empty() => ReadModelFieldSource::absence_default(
            event_name(absence_event)?,
            scenario_name(absence_scenario)?,
        ),
        _ => return Err("SliceReadModelAdded has incompatible field source fields".to_owned()),
    };

    Ok(NewReadModelField::new(name, source, provenance))
}

pub(crate) fn slice_bit_level_data_flow_from_payload(
    payload: &Value,
) -> Result<NewBitLevelDataFlow, String> {
    Ok(NewBitLevelDataFlow::new(
        slice_slug(required_str(payload, "slice")?)?,
        datum_name(required_str(payload, "datum")?)?,
        data_flow_source_kind(required_str(payload, "source_kind")?)?,
        data_flow_source(required_str(payload, "source")?)?,
        transformation_semantics(required_str(payload, "transformation")?)?,
        data_flow_target(required_str(payload, "target")?)?,
        bit_encoding_semantics(required_str(payload, "bit_encoding")?)?,
    ))
}

pub(crate) fn slice_view_from_payload(payload: &Value) -> Result<NewViewDefinition, String> {
    let mut view = NewViewDefinition::new(
        slice_slug(required_str(payload, "slice")?)?,
        view_name(required_str(payload, "name")?)?,
        view_field_from_payload(required_object(payload, "field")?)?,
    )
    .with_controls(ViewControls::from_controls(
        required_array(payload, "controls")?
            .iter()
            .map(control_definition_from_payload)
            .collect::<Result<Vec<_>, _>>()?,
    ))
    .with_local_states(ViewLocalStates::from_targets(
        required_string_array(payload, "local_states")?
            .into_iter()
            .map(|target| navigation_target_name(&target))
            .collect::<Result<Vec<_>, _>>()?,
    ));
    view = view.with_filters(ViewFilters::from_targets(
        required_string_array(payload, "filters")?
            .into_iter()
            .map(|target| navigation_target_name(&target))
            .collect::<Result<Vec<_>, _>>()?,
    ));
    Ok(view)
}

fn view_field_from_payload(payload: &Value) -> Result<NewViewField, String> {
    Ok(NewViewField::new(
        view_field_name(required_str(payload, "name")?)?,
        view_field_source_kind(required_str(payload, "source_kind")?)?,
        read_model_name(required_str(payload, "source_read_model")?)?,
        view_field_name(required_str(payload, "source_field")?)?,
        sketch_token(required_str(payload, "sketch_token")?)?,
        provenance_description(required_str(payload, "provenance")?)?,
        bit_encoding_semantics(required_str(payload, "bit_encoding")?)?,
    ))
}

fn control_definition_from_payload(payload: &Value) -> Result<NewControlDefinition, String> {
    Ok(NewControlDefinition::new(
        control_name(required_str(payload, "name")?)?,
        command_name(required_str(payload, "command")?)?,
        control_input_from_payload(required_object(payload, "input")?)?,
        command_error_names(required_string_array(payload, "handled_errors")?)?,
        control_recovery_behavior(required_str(payload, "recovery_behavior")?)?,
        sketch_token(required_str(payload, "sketch_token")?)?,
        navigation_from_payload(required_object(payload, "navigation")?)?,
    ))
}

fn control_input_from_payload(payload: &Value) -> Result<NewControlInputProvision, String> {
    Ok(NewControlInputProvision::new(
        datum_name(required_str(payload, "name")?)?,
        command_input_source_kind(required_str(payload, "source_kind")?)?,
        command_input_source_description(required_str(payload, "source_description")?)?,
        sketch_token(required_str(payload, "sketch_token")?)?,
        required_bool(payload, "visible_to_actor")?,
        required_bool(payload, "decision_field")?,
    ))
}

fn navigation_from_payload(payload: &Value) -> Result<NewNavigationTarget, String> {
    let navigation = NewNavigationTarget::new(
        navigation_target_type(required_str(payload, "type")?)?,
        navigation_target_name(required_str(payload, "target")?)?,
    );
    match (
        optional_str(payload, "external_workflow")?,
        optional_str(payload, "external_system")?,
        optional_str(payload, "handoff_contract")?,
    ) {
        (None, None, None) => Ok(navigation),
        (Some(external_workflow), None, None) => {
            Ok(navigation.with_external_workflow(navigation_target_name(external_workflow)?))
        }
        (None, Some(external_system), Some(handoff_contract)) => Ok(navigation
            .with_external_system(
                navigation_target_name(external_system)?,
                payload_contract_name(handoff_contract)?,
            )),
        _ => Err("SliceViewAdded has incompatible navigation target fields".to_owned()),
    }
}

pub(crate) fn slice_translation_from_payload(
    payload: &Value,
) -> Result<NewTranslationDefinition, String> {
    Ok(NewTranslationDefinition::new(
        slice_slug(required_str(payload, "slice")?)?,
        translation_name(required_str(payload, "name")?)?,
        translation_external_event_name(required_str(payload, "external_event")?)?,
        payload_contract_name(required_str(payload, "payload_contract")?)?,
        command_name(required_str(payload, "command")?)?,
    ))
}

pub(crate) fn slice_automation_from_payload(
    payload: &Value,
) -> Result<NewAutomationDefinition, String> {
    Ok(NewAutomationDefinition::new(
        slice_slug(required_str(payload, "slice")?)?,
        automation_name(required_str(payload, "name")?)?,
        automation_trigger_name(required_str(payload, "trigger")?)?,
        command_name(required_str(payload, "command")?)?,
        command_error_names(required_string_array(payload, "handled_errors")?)?,
        automation_reaction_description(required_str(payload, "reaction")?)?,
    ))
}

pub(crate) fn slice_board_element_from_payload(payload: &Value) -> Result<NewBoardElement, String> {
    Ok(NewBoardElement::new(
        slice_slug(required_str(payload, "slice")?)?,
        board_element_name(required_str(payload, "name")?)?,
        board_element_kind(required_str(payload, "kind")?)?,
        board_lane_id(required_str(payload, "lane")?)?,
        board_element_declared_name(required_str(payload, "declared_name")?)?,
        required_bool(payload, "main_path")?,
    ))
}

pub(crate) fn slice_board_connection_from_payload(
    payload: &Value,
) -> Result<NewBoardConnection, String> {
    Ok(NewBoardConnection::new(
        slice_slug(required_str(payload, "slice")?)?,
        board_connection_endpoint(required_str(payload, "source")?)?,
        board_connection_endpoint_kind(required_str(payload, "source_kind")?)?,
        board_connection_endpoint(required_str(payload, "target")?)?,
        board_connection_endpoint_kind(required_str(payload, "target_kind")?)?,
    ))
}

fn workflow_command_error_from_payload(
    payload: &Value,
) -> Result<WorkflowCommandErrorRecord, String> {
    Ok(WorkflowCommandErrorRecord::new(
        workflow_transition_endpoint(required_str(payload, "source_slice")?)?,
        command_name(required_str(payload, "command")?)?,
        command_error_name(required_str(payload, "error")?)?,
    ))
}

fn workflow_owned_definition_from_payload(
    payload: &Value,
) -> Result<WorkflowOwnedDefinitionRecord, String> {
    let source_slice = workflow_transition_endpoint(required_str(payload, "source_slice")?)?;
    let definition_kind =
        workflow_owned_definition_kind(required_str(payload, "definition_kind")?)?;
    let definition_name =
        workflow_owned_definition_name(required_str(payload, "definition_name")?)?;
    let definition_stream = optional_str(payload, "definition_stream")?
        .map(stream_name)
        .transpose()?;
    let source_provenance = optional_str(payload, "source_provenance")?
        .map(model_description)
        .transpose()?;
    let event_participation = optional_str(payload, "event_participation")?
        .map(workflow_event_participation)
        .transpose()?;
    let view_role = optional_str(payload, "view_role")?
        .map(workflow_view_role)
        .transpose()?;

    match (
        definition_stream,
        source_provenance,
        event_participation,
        view_role,
    ) {
        (None, None, None, None) => Ok(WorkflowOwnedDefinitionRecord::new(
            source_slice,
            definition_kind,
            definition_name,
        )),
        (None, None, None, Some(view_role)) => WorkflowOwnedDefinitionRecord::new_with_view_role(
            source_slice,
            definition_kind,
            definition_name,
            view_role,
        )
        .ok_or_else(|| "view role requires a view owned-definition kind".to_owned()),
        (Some(definition_stream), Some(source_provenance), None, None) => {
            Ok(WorkflowOwnedDefinitionRecord::new_with_event_identity(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
            ))
        }
        (Some(definition_stream), Some(source_provenance), Some(event_participation), None) => Ok(
            WorkflowOwnedDefinitionRecord::new_with_event_identity_and_participation(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
                event_participation,
            ),
        ),
        _ => Err("WorkflowOwnedDefinitionAdded has incompatible optional fields".to_owned()),
    }
}

fn workflow_transition_evidence_from_payload(
    payload: &Value,
) -> Result<WorkflowTransitionEvidenceRecord, String> {
    Ok(WorkflowTransitionEvidenceRecord::new(
        workflow_transition_endpoint(required_str(payload, "from")?)?,
        workflow_transition_endpoint(required_str(payload, "to")?)?,
        workflow_transition_kind(required_str(payload, "via")?)?,
        transition_trigger_name(required_str(payload, "name")?)?,
        workflow_transition_source_evidence_text(required_str(payload, "source_evidence")?)?,
        workflow_transition_target_evidence_text(required_str(payload, "target_evidence")?)?,
    ))
}

fn workflow_entry_lifecycle_state_from_payload(
    payload: &Value,
) -> Result<WorkflowEntryLifecycleStateRecord, String> {
    Ok(WorkflowEntryLifecycleStateRecord::new(
        workflow_entry_lifecycle_state_name(required_str(payload, "state")?)?,
        workflow_transition_endpoint(required_str(payload, "step")?)?,
        workflow_entry_lifecycle_evidence_text(required_str(payload, "evidence")?)?,
    ))
}

fn same_transition(left: &WorkflowTransitionRecord, right: &WorkflowTransitionRecord) -> bool {
    left.source().as_ref() == right.source().as_ref()
        && left.target().as_ref() == right.target().as_ref()
        && left.kind().as_ref() == right.kind().as_ref()
        && left.trigger().as_ref() == right.trigger().as_ref()
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

fn project_name(value: &str) -> Result<ProjectName, String> {
    ProjectName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn model_name(value: &str) -> Result<ModelName, String> {
    ModelName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn model_description(value: &str) -> Result<ModelDescription, String> {
    ModelDescription::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_slug(value: &str) -> Result<WorkflowSlug, String> {
    WorkflowSlug::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn artifact_digest_from_str(value: &str) -> Result<ArtifactDigest, String> {
    ArtifactDigest::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn connection_kind(value: &str) -> Result<ConnectionKind, String> {
    ConnectionKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn review_timestamp(value: &str) -> Result<ReviewTimestamp, String> {
    ReviewTimestamp::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn reviewer_id(value: &str) -> Result<ReviewerId, String> {
    ReviewerId::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn slice_slug(value: &str) -> Result<SliceSlug, String> {
    SliceSlug::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn slice_kind_name(value: &str) -> Result<SliceKindName, String> {
    SliceKindName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_transition_endpoint(value: &str) -> Result<WorkflowTransitionEndpoint, String> {
    WorkflowTransitionEndpoint::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_transition_kind(value: &str) -> Result<WorkflowTransitionKind, String> {
    WorkflowTransitionKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn transition_trigger_name(value: &str) -> Result<TransitionTriggerName, String> {
    TransitionTriggerName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn payload_contract_name(value: &str) -> Result<PayloadContractName, String> {
    PayloadContractName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn outcome_label_name(value: &str) -> Result<OutcomeLabelName, String> {
    OutcomeLabelName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn command_name(value: &str) -> Result<CommandName, String> {
    CommandName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn event_name(value: &str) -> Result<EventName, String> {
    EventName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn translation_name(value: &str) -> Result<TranslationName, String> {
    TranslationName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn translation_external_event_name(value: &str) -> Result<TranslationExternalEventName, String> {
    TranslationExternalEventName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn automation_name(value: &str) -> Result<AutomationName, String> {
    AutomationName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn automation_trigger_name(value: &str) -> Result<AutomationTriggerName, String> {
    AutomationTriggerName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn automation_reaction_description(value: &str) -> Result<AutomationReactionDescription, String> {
    AutomationReactionDescription::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn board_element_name(value: &str) -> Result<BoardElementName, String> {
    BoardElementName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn board_element_kind(value: &str) -> Result<BoardElementKind, String> {
    BoardElementKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn board_lane_id(value: &str) -> Result<BoardLaneId, String> {
    BoardLaneId::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn board_element_declared_name(value: &str) -> Result<BoardElementDeclaredName, String> {
    BoardElementDeclaredName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn board_connection_endpoint(value: &str) -> Result<BoardConnectionEndpoint, String> {
    BoardConnectionEndpoint::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn board_connection_endpoint_kind(value: &str) -> Result<BoardConnectionEndpointKind, String> {
    BoardConnectionEndpointKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn datum_name(value: &str) -> Result<DatumName, String> {
    DatumName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn read_model_name(value: &str) -> Result<ReadModelName, String> {
    ReadModelName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn read_model_field_source_kind(value: &str) -> Result<ReadModelFieldSourceKind, String> {
    ReadModelFieldSourceKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn read_model_derivation_rule(value: &str) -> Result<ReadModelDerivationRule, String> {
    ReadModelDerivationRule::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn read_model_transitive_rule(value: &str) -> Result<ReadModelTransitiveRule, String> {
    ReadModelTransitiveRule::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn data_flow_source_kind(value: &str) -> Result<DataFlowSourceKind, String> {
    DataFlowSourceKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn data_flow_source(value: &str) -> Result<DataFlowSource, String> {
    DataFlowSource::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn transformation_semantics(value: &str) -> Result<TransformationSemantics, String> {
    TransformationSemantics::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn data_flow_target(value: &str) -> Result<DataFlowTarget, String> {
    DataFlowTarget::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn view_name(value: &str) -> Result<ViewName, String> {
    ViewName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn view_field_name(value: &str) -> Result<ViewFieldName, String> {
    ViewFieldName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn view_field_source_kind(value: &str) -> Result<ViewFieldSourceKind, String> {
    ViewFieldSourceKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn sketch_token(value: &str) -> Result<SketchToken, String> {
    SketchToken::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn control_name(value: &str) -> Result<ControlName, String> {
    ControlName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn control_recovery_behavior(value: &str) -> Result<ControlRecoveryBehavior, String> {
    ControlRecoveryBehavior::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn navigation_target_type(value: &str) -> Result<NavigationTargetType, String> {
    NavigationTargetType::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn navigation_target_name(value: &str) -> Result<NavigationTargetName, String> {
    NavigationTargetName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn command_input_source_kind(value: &str) -> Result<CommandInputSourceKind, String> {
    CommandInputSourceKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn command_input_source_description(value: &str) -> Result<CommandInputSourceDescription, String> {
    CommandInputSourceDescription::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn source_chain_hop(value: &str) -> Result<SourceChainHop, String> {
    SourceChainHop::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn command_error_recovery_kind(value: &str) -> Result<CommandErrorRecoveryKind, String> {
    CommandErrorRecoveryKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn singleton_repeat_behavior(value: &str) -> Result<SingletonRepeatBehavior, String> {
    SingletonRepeatBehavior::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn scenario_kind(value: &str) -> Result<ScenarioKind, String> {
    ScenarioKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn scenario_name(value: &str) -> Result<ScenarioName, String> {
    ScenarioName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn scenario_step_text(value: &str) -> Result<ScenarioStepText, String> {
    ScenarioStepText::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn contract_kind_name(value: &str) -> Result<ContractKindName, String> {
    ContractKindName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn covered_definition_name(value: &str) -> Result<CoveredDefinitionName, String> {
    CoveredDefinitionName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn event_attribute_name(value: &str) -> Result<EventAttributeName, String> {
    EventAttributeName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn event_attribute_source_kind(value: &str) -> Result<EventAttributeSourceKind, String> {
    EventAttributeSourceKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn generated_event_attribute_source_kind(
    value: &str,
) -> Result<GeneratedEventAttributeSourceKind, String> {
    GeneratedEventAttributeSourceKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn event_attribute_source_name(value: &str) -> Result<EventAttributeSourceName, String> {
    EventAttributeSourceName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn event_attribute_source_field(value: &str) -> Result<EventAttributeSourceField, String> {
    EventAttributeSourceField::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn provenance_description(value: &str) -> Result<ProvenanceDescription, String> {
    ProvenanceDescription::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn bit_encoding_semantics(value: &str) -> Result<BitEncodingSemantics, String> {
    BitEncodingSemantics::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn scenario_stream_names(values: Vec<String>) -> Result<ScenarioStreamNames, String> {
    values
        .into_iter()
        .map(|value| stream_name(&value))
        .collect::<Result<Vec<_>, _>>()
        .map(ScenarioStreamNames::from_streams)
}

fn command_error_names(values: Vec<String>) -> Result<CommandErrorNames, String> {
    values
        .into_iter()
        .map(|value| command_error_name(&value))
        .collect::<Result<Vec<_>, _>>()
        .map(CommandErrorNames::from_names)
}

fn command_error_name(value: &str) -> Result<CommandErrorName, String> {
    CommandErrorName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_owned_definition_kind(value: &str) -> Result<WorkflowOwnedDefinitionKind, String> {
    WorkflowOwnedDefinitionKind::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_owned_definition_name(value: &str) -> Result<WorkflowOwnedDefinitionName, String> {
    WorkflowOwnedDefinitionName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn stream_name(value: &str) -> Result<StreamName, String> {
    StreamName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_event_participation(value: &str) -> Result<WorkflowEventParticipation, String> {
    WorkflowEventParticipation::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_view_role(value: &str) -> Result<WorkflowViewRole, String> {
    WorkflowViewRole::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_transition_source_evidence_text(
    value: &str,
) -> Result<WorkflowTransitionSourceEvidenceText, String> {
    WorkflowTransitionSourceEvidenceText::try_new(value.to_owned())
        .map_err(|error| error.to_string())
}

fn workflow_transition_target_evidence_text(
    value: &str,
) -> Result<WorkflowTransitionTargetEvidenceText, String> {
    WorkflowTransitionTargetEvidenceText::try_new(value.to_owned())
        .map_err(|error| error.to_string())
}

fn workflow_entry_lifecycle_state_name(
    value: &str,
) -> Result<WorkflowEntryLifecycleStateName, String> {
    WorkflowEntryLifecycleStateName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_entry_lifecycle_evidence_text(
    value: &str,
) -> Result<WorkflowEntryLifecycleEvidenceText, String> {
    WorkflowEntryLifecycleEvidenceText::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn lean_module_name(value: impl Into<String>) -> LeanModuleName {
    LeanModuleName::try_new(value.into())
        .unwrap_or_else(|error| unreachable!("generated Lean module name must be valid: {error}"))
}

fn quint_module_name(value: impl Into<String>) -> QuintModuleName {
    QuintModuleName::try_new(value.into())
        .unwrap_or_else(|error| unreachable!("generated Quint module name must be valid: {error}"))
}

fn project_path(value: impl Into<String>) -> Result<ProjectPath, String> {
    ProjectPath::try_new(value.into()).map_err(|error| error.to_string())
}

fn file_contents(value: impl Into<String>) -> Result<FileContents, String> {
    FileContents::try_new(value.into()).map_err(|error| error.to_string())
}

fn report_line(value: impl Into<String>) -> Result<ReportLine, String> {
    ReportLine::try_new(value.into()).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_event_payload_round_trips_between_semantic_workflow_and_json_boundary()
    -> Result<(), String> {
        let workflow = NewWorkflow::new(
            model_name("Open ticket")?,
            model_description("Actor opens a repair ticket.")?,
            workflow_slug("open-ticket")?,
        );

        let payload = WorkflowEventPayload::from_workflow(&workflow);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slug": "open-ticket",
                "name": "Open ticket",
                "description": "Actor opens a repair ticket.",
            })
        );
        assert_eq!(
            WorkflowEventPayload::from_json_value(&json)?.into_workflow(),
            workflow
        );

        Ok(())
    }
}
