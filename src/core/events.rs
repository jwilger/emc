// Copyright 2026 John Wilger

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::core::connection::{
    ConnectionKind, WorkflowConnection, WorkflowConnectionTarget, WorkflowTransitionRemoval,
    WorkflowTransitionRemovalTarget,
};
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
use crate::core::event_commands::EmcEvent;
use crate::core::event_runtime::{list_forks, read_all_emc_events, reconcile_choose_branch};
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
    WorkflowTransitionEndpoint, WorkflowTransitionEvidenceNavigationEndpoints,
    WorkflowTransitionEvidenceRecord, WorkflowTransitionEvidenceRecords, WorkflowTransitionKind,
    WorkflowTransitionRecord, WorkflowTransitionRecords, WorkflowTransitionSourceEvidenceText,
    WorkflowTransitionTargetEvidenceText, WorkflowViewRole,
};
use crate::core::workflow::NewWorkflow;

const PROJECTION_FINGERPRINT_PATH: &str = "model/events/projection.fingerprint";

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct EventDraft {
    stream_id: EventStreamId,
    body: ExportedEventBody,
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
struct SliceAddedEventPayload {
    workflow_slug: WorkflowSlug,
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKindName,
    description: ModelDescription,
}

impl SliceAddedEventPayload {
    fn from_slice(slice: &NewSlice) -> Self {
        Self {
            workflow_slug: slice.workflow_slug().clone(),
            slug: slice.slug().clone(),
            name: slice.name().clone(),
            kind: SliceKindName::from(slice.kind()),
            description: slice.description().clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            workflow_slug: workflow_slug(required_str(payload, "workflow")?)?,
            slug: slice_slug(required_str(payload, "slug")?)?,
            name: model_name(required_str(payload, "name")?)?,
            kind: slice_kind_name(required_str(payload, "kind")?)?,
            description: model_description(required_str(payload, "description")?)?,
        })
    }

    fn into_slice(self) -> NewSlice {
        NewSlice::new(
            self.workflow_slug,
            self.slug,
            self.name,
            self.description,
            self.kind.into(),
        )
    }

    fn to_json_value(&self) -> Value {
        json!({
            "workflow": self.workflow_slug.as_ref(),
            "slug": self.slug.as_ref(),
            "name": self.name.as_ref(),
            "kind": self.kind.as_ref(),
            "description": self.description.as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceUpdatedEventPayload {
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKindName,
    description: ModelDescription,
}

impl SliceUpdatedEventPayload {
    fn from_slice_detail(slice: &WorkflowSliceDetail) -> Self {
        Self {
            slug: slice.slug().clone(),
            name: slice.name().clone(),
            kind: *slice.kind(),
            description: slice.description().clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            slug: slice_slug(required_str(payload, "slug")?)?,
            name: model_name(required_str(payload, "name")?)?,
            kind: slice_kind_name(required_str(payload, "kind")?)?,
            description: model_description(required_str(payload, "description")?)?,
        })
    }

    fn into_slice_detail(self) -> WorkflowSliceDetail {
        WorkflowSliceDetail::new(self.slug, self.name, self.kind, self.description)
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slug": self.slug.as_ref(),
            "name": self.name.as_ref(),
            "kind": self.kind.as_ref(),
            "description": self.description.as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceScenarioAddedEventPayload {
    scenario: NewSliceScenario,
}

impl SliceScenarioAddedEventPayload {
    fn from_scenario(scenario: &NewSliceScenario) -> Self {
        Self {
            scenario: scenario.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
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

        Ok(Self {
            scenario: scenario
                .with_streams(
                    scenario_stream_names(required_string_array(payload, "read_streams")?)?,
                    scenario_stream_names(required_string_array(payload, "written_streams")?)?,
                )
                .with_error_references(command_error_names(required_string_array(
                    payload,
                    "error_references",
                )?)?),
        })
    }

    fn into_scenario(self) -> NewSliceScenario {
        self.scenario
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.scenario.slice_slug().as_ref(),
            "kind": self.scenario.kind().as_str(),
            "name": self.scenario.name().as_ref(),
            "given": self.scenario.given().as_ref(),
            "when": self.scenario.when().as_ref(),
            "then": self.scenario.then().as_ref(),
            "read_streams": self
                .scenario
                .read_streams()
                .as_slice()
                .iter()
                .map(|stream| stream.as_ref())
                .collect::<Vec<_>>(),
            "written_streams": self
                .scenario
                .written_streams()
                .as_slice()
                .iter()
                .map(|stream| stream.as_ref())
                .collect::<Vec<_>>(),
            "contract_kind": self.scenario.contract_kind().map(|kind| kind.as_ref()),
            "covered_definition": self
                .scenario
                .covered_definition()
                .map(|definition| definition.as_ref()),
            "error_references": self
                .scenario
                .error_references()
                .as_slice()
                .iter()
                .map(|error| error.as_ref())
                .collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceOutcomeAddedEventPayload {
    outcome: NewOutcomeDefinition,
}

impl SliceOutcomeAddedEventPayload {
    fn from_outcome(outcome: &NewOutcomeDefinition) -> Self {
        Self {
            outcome: outcome.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            outcome: NewOutcomeDefinition::new(
                slice_slug(required_str(payload, "slice")?)?,
                outcome_label_name(required_str(payload, "label")?)?,
                OutcomeEventNames::from_events(
                    required_string_array(payload, "events")?
                        .into_iter()
                        .map(|event| event_name(&event))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                required_bool(payload, "externally_relevant")?,
            ),
        })
    }

    fn into_outcome(self) -> NewOutcomeDefinition {
        self.outcome
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.outcome.slice_slug().as_ref(),
            "label": self.outcome.label().as_ref(),
            "events": self
                .outcome
                .event_set()
                .as_slice()
                .iter()
                .map(|event| event.as_ref())
                .collect::<Vec<_>>(),
            "externally_relevant": self.outcome.externally_relevant(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceExternalPayloadAddedEventPayload {
    external_payload: NewExternalPayloadDefinition,
}

impl SliceExternalPayloadAddedEventPayload {
    fn from_external_payload(external_payload: &NewExternalPayloadDefinition) -> Self {
        Self {
            external_payload: external_payload.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            external_payload: NewExternalPayloadDefinition::new(
                slice_slug(required_str(payload, "slice")?)?,
                event_attribute_source_name(required_str(payload, "name")?)?,
                event_attribute_source_field(required_str(payload, "field")?)?,
                provenance_description(required_str(payload, "field_provenance")?)?,
                bit_encoding_semantics(required_str(payload, "bit_encoding")?)?,
            ),
        })
    }

    fn into_external_payload(self) -> NewExternalPayloadDefinition {
        self.external_payload
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.external_payload.slice_slug().as_ref(),
            "name": self.external_payload.name().as_ref(),
            "field": self.external_payload.field().as_ref(),
            "field_provenance": self.external_payload.field_provenance().as_ref(),
            "bit_encoding": self.external_payload.bit_encoding().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceEventDefinitionAddedEventPayload {
    event: NewEventDefinition,
}

impl SliceEventDefinitionAddedEventPayload {
    fn from_event(event: &NewEventDefinition) -> Self {
        Self {
            event: event.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
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

        let event = match (
            required_bool(payload, "observed")?,
            required_bool(payload, "shared")?,
        ) {
            (false, false) => NewEventDefinition::new(slice_slug, name, stream, attribute),
            (true, false) => NewEventDefinition::new_observed(slice_slug, name, stream, attribute),
            (false, true) => NewEventDefinition::new_shared(slice_slug, name, stream, attribute),
            (true, true) => {
                return Err(
                    "SliceEventDefinitionAdded cannot be both observed and shared".to_owned(),
                );
            }
        };

        Ok(Self { event })
    }

    fn into_event(self) -> NewEventDefinition {
        self.event
    }

    fn to_json_value(&self) -> Value {
        let attribute = self.event.attribute();
        json!({
            "slice": self.event.slice_slug().as_ref(),
            "name": self.event.name().as_ref(),
            "stream": self.event.stream().as_ref(),
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
            "observed": self.event.observed(),
            "shared": self.event.shared(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceCommandDefinitionAddedEventPayload {
    command: NewCommandDefinition,
}

impl SliceCommandDefinitionAddedEventPayload {
    fn from_command(command: &NewCommandDefinition) -> Self {
        Self {
            command: command.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        let input = Self::command_input_from_json_value(required_object(payload, "input")?)?;

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
                .map(Self::command_error_from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
        ));

        if let Some(repeat_behavior) = optional_str(payload, "singleton_repeat_behavior")? {
            command =
                command.with_singleton_repeat_behavior(singleton_repeat_behavior(repeat_behavior)?);
        }

        Ok(Self { command })
    }

    fn into_command(self) -> NewCommandDefinition {
        self.command
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.command.slice_slug().as_ref(),
            "name": self.command.name().as_ref(),
            "input": Self::command_input_to_json_value(self.command.input()),
            "emitted_events": self
                .command
                .emitted_events()
                .as_slice()
                .iter()
                .map(|event| event.as_ref())
                .collect::<Vec<_>>(),
            "observed_streams": self
                .command
                .observed_streams()
                .as_slice()
                .iter()
                .map(|stream| stream.as_ref())
                .collect::<Vec<_>>(),
            "errors": self
                .command
                .errors()
                .as_slice()
                .iter()
                .map(Self::command_error_to_json_value)
                .collect::<Vec<_>>(),
            "singleton_repeat_behavior": self
                .command
                .singleton_repeat_behavior()
                .map(|repeat_behavior| repeat_behavior.as_ref()),
        })
    }

    fn command_input_from_json_value(input_payload: &Value) -> Result<NewCommandInput, String> {
        let source_kind = command_input_source_kind(required_str(input_payload, "source_kind")?)?;
        Ok(NewCommandInput::new(
            datum_name(required_str(input_payload, "name")?)?,
            Self::command_input_source_from_json_value(source_kind, input_payload)?,
            command_input_source_description(required_str(input_payload, "source_description")?)?,
            CommandInputProvenanceChain::from_hops(
                required_string_array(input_payload, "provenance_chain")?
                    .into_iter()
                    .map(|hop| source_chain_hop(&hop))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ))
    }

    fn command_input_to_json_value(input: &NewCommandInput) -> Value {
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

    fn command_input_source_from_json_value(
        source_kind: CommandInputSourceKind,
        input_payload: &Value,
    ) -> Result<CommandInputSource, String> {
        let event_stream = Self::optional_event_stream_source(input_payload)?;
        let external_payload = Self::optional_source_field_pair(
            input_payload,
            "external_payload_source_name",
            "external_payload_source_field",
        )?;
        let generated = Self::optional_source_field_pair(
            input_payload,
            "generated_source_name",
            "generated_source_field",
        )?;
        let session = Self::optional_source_field_pair(
            input_payload,
            "session_source_name",
            "session_source_field",
        )?;
        let invocation_argument = Self::optional_source_field_pair(
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
            _ => {
                Err("command input event stream source fields must be supplied together".to_owned())
            }
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

    fn command_error_from_json_value(payload: &Value) -> Result<NewCommandErrorDefinition, String> {
        Ok(NewCommandErrorDefinition::new(
            command_error_name(required_str(payload, "name")?)?,
            scenario_name(required_str(payload, "scenario")?)?,
            command_error_recovery_kind(required_str(payload, "recovery")?)?,
        ))
    }

    fn command_error_to_json_value(error: &NewCommandErrorDefinition) -> Value {
        json!({
            "name": error.name().as_ref(),
            "scenario": error.scenario_name().as_ref(),
            "recovery": error.recovery_kind().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceReadModelAddedEventPayload {
    read_model: NewReadModelDefinition,
}

impl SliceReadModelAddedEventPayload {
    fn from_read_model(read_model: &NewReadModelDefinition) -> Self {
        Self {
            read_model: read_model.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        let field = Self::read_model_field_from_json_value(required_object(payload, "field")?)?;
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

        Ok(Self { read_model })
    }

    fn into_read_model(self) -> NewReadModelDefinition {
        self.read_model
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.read_model.slice_slug().as_ref(),
            "name": self.read_model.name().as_ref(),
            "field": Self::read_model_field_to_json_value(self.read_model.field()),
            "transitive": self.read_model.transitive(),
            "relationship_fields": self
                .read_model
                .relationship_fields()
                .as_slice()
                .iter()
                .map(|field| field.as_ref())
                .collect::<Vec<_>>(),
            "transitive_rule": self
                .read_model
                .transitive_rule()
                .map(|rule| rule.as_ref()),
            "example_scenario": self
                .read_model
                .example_scenario_name()
                .map(|scenario| scenario.as_ref()),
        })
    }

    fn read_model_field_from_json_value(payload: &Value) -> Result<NewReadModelField, String> {
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

    fn read_model_field_to_json_value(field: &NewReadModelField) -> Value {
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceViewAddedEventPayload {
    view: NewViewDefinition,
}

impl SliceViewAddedEventPayload {
    fn from_view(view: &NewViewDefinition) -> Self {
        Self { view: view.clone() }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        let mut view = NewViewDefinition::new(
            slice_slug(required_str(payload, "slice")?)?,
            view_name(required_str(payload, "name")?)?,
            Self::view_field_from_json_value(required_object(payload, "field")?)?,
        )
        .with_controls(ViewControls::from_controls(
            required_array(payload, "controls")?
                .iter()
                .map(Self::control_definition_from_json_value)
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
        Ok(Self { view })
    }

    fn into_view(self) -> NewViewDefinition {
        self.view
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.view.slice_slug().as_ref(),
            "name": self.view.name().as_ref(),
            "field": Self::view_field_to_json_value(self.view.field()),
            "controls": self
                .view
                .controls()
                .as_slice()
                .iter()
                .map(Self::control_to_json_value)
                .collect::<Vec<_>>(),
            "local_states": self
                .view
                .local_states()
                .as_slice()
                .iter()
                .map(|state| state.as_ref())
                .collect::<Vec<_>>(),
            "filters": self
                .view
                .filters()
                .as_slice()
                .iter()
                .map(|filter| filter.as_ref())
                .collect::<Vec<_>>(),
        })
    }

    fn view_field_from_json_value(payload: &Value) -> Result<NewViewField, String> {
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

    fn view_field_to_json_value(field: &NewViewField) -> Value {
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

    fn control_definition_from_json_value(payload: &Value) -> Result<NewControlDefinition, String> {
        Ok(NewControlDefinition::new(
            control_name(required_str(payload, "name")?)?,
            command_name(required_str(payload, "command")?)?,
            Self::control_input_from_json_value(required_object(payload, "input")?)?,
            command_error_names(required_string_array(payload, "handled_errors")?)?,
            control_recovery_behavior(required_str(payload, "recovery_behavior")?)?,
            sketch_token(required_str(payload, "sketch_token")?)?,
            Self::navigation_from_json_value(required_object(payload, "navigation")?)?,
        ))
    }

    fn control_to_json_value(control: &NewControlDefinition) -> Value {
        json!({
            "name": control.name().as_ref(),
            "command": control.command_name().as_ref(),
            "input": Self::control_input_to_json_value(control.input()),
            "handled_errors": control
                .handled_errors()
                .as_slice()
                .iter()
                .map(|error| error.as_ref())
                .collect::<Vec<_>>(),
            "recovery_behavior": control.recovery_behavior().as_ref(),
            "sketch_token": control.sketch_token().as_ref(),
            "navigation": Self::navigation_to_json_value(control.navigation()),
        })
    }

    fn control_input_from_json_value(payload: &Value) -> Result<NewControlInputProvision, String> {
        Ok(NewControlInputProvision::new(
            datum_name(required_str(payload, "name")?)?,
            command_input_source_kind(required_str(payload, "source_kind")?)?,
            command_input_source_description(required_str(payload, "source_description")?)?,
            sketch_token(required_str(payload, "sketch_token")?)?,
            required_bool(payload, "visible_to_actor")?,
            required_bool(payload, "decision_field")?,
        ))
    }

    fn control_input_to_json_value(input: &NewControlInputProvision) -> Value {
        json!({
            "name": input.name().as_ref(),
            "source_kind": input.source_kind().as_ref(),
            "source_description": input.source_description().as_ref(),
            "sketch_token": input.sketch_token().as_ref(),
            "visible_to_actor": input.visible_to_actor(),
            "decision_field": input.decision_field(),
        })
    }

    fn navigation_from_json_value(payload: &Value) -> Result<NewNavigationTarget, String> {
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

    fn navigation_to_json_value(navigation: &NewNavigationTarget) -> Value {
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceBitLevelDataFlowAddedEventPayload {
    data_flow: NewBitLevelDataFlow,
}

impl SliceBitLevelDataFlowAddedEventPayload {
    fn from_data_flow(data_flow: &NewBitLevelDataFlow) -> Self {
        Self {
            data_flow: data_flow.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            data_flow: NewBitLevelDataFlow::new(
                slice_slug(required_str(payload, "slice")?)?,
                datum_name(required_str(payload, "datum")?)?,
                data_flow_source_kind(required_str(payload, "source_kind")?)?,
                data_flow_source(required_str(payload, "source")?)?,
                transformation_semantics(required_str(payload, "transformation")?)?,
                data_flow_target(required_str(payload, "target")?)?,
                bit_encoding_semantics(required_str(payload, "bit_encoding")?)?,
            ),
        })
    }

    fn into_data_flow(self) -> NewBitLevelDataFlow {
        self.data_flow
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.data_flow.slice_slug().as_ref(),
            "datum": self.data_flow.datum().as_ref(),
            "source": self.data_flow.source().as_ref(),
            "source_kind": self.data_flow.source_kind().as_ref(),
            "transformation": self.data_flow.transformation().as_ref(),
            "target": self.data_flow.target().as_ref(),
            "bit_encoding": self.data_flow.bit_encoding().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceTranslationAddedEventPayload {
    translation: NewTranslationDefinition,
}

impl SliceTranslationAddedEventPayload {
    fn from_translation(translation: &NewTranslationDefinition) -> Self {
        Self {
            translation: translation.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            translation: NewTranslationDefinition::new(
                slice_slug(required_str(payload, "slice")?)?,
                translation_name(required_str(payload, "name")?)?,
                translation_external_event_name(required_str(payload, "external_event")?)?,
                payload_contract_name(required_str(payload, "payload_contract")?)?,
                command_name(required_str(payload, "command")?)?,
            ),
        })
    }

    fn into_translation(self) -> NewTranslationDefinition {
        self.translation
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.translation.slice_slug().as_ref(),
            "name": self.translation.name().as_ref(),
            "external_event": self.translation.external_event_name().as_ref(),
            "payload_contract": self.translation.payload_contract_name().as_ref(),
            "command": self.translation.command_name().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceAutomationAddedEventPayload {
    automation: NewAutomationDefinition,
}

impl SliceAutomationAddedEventPayload {
    fn from_automation(automation: &NewAutomationDefinition) -> Self {
        Self {
            automation: automation.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            automation: NewAutomationDefinition::new(
                slice_slug(required_str(payload, "slice")?)?,
                automation_name(required_str(payload, "name")?)?,
                automation_trigger_name(required_str(payload, "trigger")?)?,
                command_name(required_str(payload, "command")?)?,
                command_error_names(required_string_array(payload, "handled_errors")?)?,
                automation_reaction_description(required_str(payload, "reaction")?)?,
            ),
        })
    }

    fn into_automation(self) -> NewAutomationDefinition {
        self.automation
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.automation.slice_slug().as_ref(),
            "name": self.automation.name().as_ref(),
            "trigger": self.automation.trigger_name().as_ref(),
            "command": self.automation.command_name().as_ref(),
            "handled_errors": self
                .automation
                .handled_errors()
                .as_slice()
                .iter()
                .map(|error| error.as_ref())
                .collect::<Vec<_>>(),
            "reaction": self.automation.reaction_description().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceBoardElementAddedEventPayload {
    element: NewBoardElement,
}

impl SliceBoardElementAddedEventPayload {
    fn from_element(element: &NewBoardElement) -> Self {
        Self {
            element: element.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            element: NewBoardElement::new(
                slice_slug(required_str(payload, "slice")?)?,
                board_element_name(required_str(payload, "name")?)?,
                board_element_kind(required_str(payload, "kind")?)?,
                board_lane_id(required_str(payload, "lane")?)?,
                board_element_declared_name(required_str(payload, "declared_name")?)?,
                required_bool(payload, "main_path")?,
            ),
        })
    }

    fn into_element(self) -> NewBoardElement {
        self.element
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.element.slice_slug().as_ref(),
            "name": self.element.name().as_ref(),
            "kind": self.element.kind().as_ref(),
            "lane": self.element.lane().as_ref(),
            "declared_name": self.element.declared_name().as_ref(),
            "main_path": self.element.main_path(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceBoardConnectionAddedEventPayload {
    connection: NewBoardConnection,
}

impl SliceBoardConnectionAddedEventPayload {
    fn from_connection(connection: &NewBoardConnection) -> Self {
        Self {
            connection: connection.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            connection: NewBoardConnection::new(
                slice_slug(required_str(payload, "slice")?)?,
                board_connection_endpoint(required_str(payload, "source")?)?,
                board_connection_endpoint_kind(required_str(payload, "source_kind")?)?,
                board_connection_endpoint(required_str(payload, "target")?)?,
                board_connection_endpoint_kind(required_str(payload, "target_kind")?)?,
            ),
        })
    }

    fn into_connection(self) -> NewBoardConnection {
        self.connection
    }

    fn to_json_value(&self) -> Value {
        json!({
            "slice": self.connection.slice_slug().as_ref(),
            "source": self.connection.source().as_ref(),
            "source_kind": self.connection.source_kind().as_ref(),
            "target": self.connection.target().as_ref(),
            "target_kind": self.connection.target_kind().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ReviewRecordedEventPayload {
    workflow_slug: WorkflowSlug,
    model_content_digest: ModelContentDigest,
    reviewer_id: ReviewerId,
    reviewed_at: ReviewTimestamp,
    categories: Vec<ReviewRuleName>,
}

impl ReviewRecordedEventPayload {
    fn from_parts(
        workflow_slug: &WorkflowSlug,
        model_content_digest: &ModelContentDigest,
        reviewer_id: &ReviewerId,
        reviewed_at: &ReviewTimestamp,
        categories: &[ReviewRuleName],
    ) -> Self {
        Self {
            workflow_slug: workflow_slug.clone(),
            model_content_digest: model_content_digest.clone(),
            reviewer_id: reviewer_id.clone(),
            reviewed_at: reviewed_at.clone(),
            categories: categories.to_vec(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            workflow_slug: workflow_slug(required_str(payload, "workflow")?)?,
            model_content_digest: ModelContentDigest::new(artifact_digest_from_str(required_str(
                payload,
                "model_content_digest",
            )?)?),
            reviewer_id: reviewer_id(required_str(payload, "reviewer_id")?)?,
            reviewed_at: review_timestamp(required_str(payload, "reviewed_at")?)?,
            categories: Self::categories_from_json_value(payload)?,
        })
    }

    fn categories_from_json_value(payload: &Value) -> Result<Vec<ReviewRuleName>, String> {
        required_string_array(payload, "categories")?
            .into_iter()
            .map(|category| ReviewRuleName::try_new(category).map_err(|error| error.to_string()))
            .collect()
    }

    fn into_body(self) -> ExportedEventBody {
        ExportedEventBody::ReviewRecorded {
            workflow_slug: self.workflow_slug,
            model_content_digest: self.model_content_digest,
            reviewer_id: self.reviewer_id,
            reviewed_at: self.reviewed_at,
            categories: self.categories,
        }
    }

    fn to_json_value(&self) -> Value {
        json!({
            "workflow": self.workflow_slug.as_ref(),
            "model_content_digest": self.model_content_digest.as_ref(),
            "reviewer_id": self.reviewer_id.as_ref(),
            "reviewed_at": self.reviewed_at.as_ref(),
            "categories": self
                .categories
                .iter()
                .map(ReviewRuleName::as_ref)
                .collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowConnectedEventPayload {
    workflow_slug: WorkflowSlug,
    source: SliceSlug,
    target: WorkflowConnectedEventTarget,
    kind: ConnectionKind,
    trigger: TransitionTriggerName,
    source_control: Option<TransitionTriggerName>,
    target_view: Option<WorkflowOwnedDefinitionName>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum WorkflowConnectedEventTarget {
    Slice {
        slug: SliceSlug,
        payload_contract: Option<PayloadContractName>,
    },
    WorkflowExit {
        slug: WorkflowSlug,
        reason: ModelDescription,
    },
}

impl WorkflowConnectedEventPayload {
    fn from_connection(connection: &WorkflowConnection) -> Self {
        let target = match connection.target() {
            WorkflowConnectionTarget::Slice(slug) => WorkflowConnectedEventTarget::Slice {
                slug: slug.clone(),
                payload_contract: connection.payload_contract().cloned(),
            },
            WorkflowConnectionTarget::Workflow { slug, reason } => {
                WorkflowConnectedEventTarget::WorkflowExit {
                    slug: slug.clone(),
                    reason: reason.clone(),
                }
            }
        };
        Self {
            workflow_slug: connection.workflow_slug().clone(),
            source: connection.source().clone(),
            target,
            kind: connection.kind(),
            trigger: connection.trigger().clone(),
            source_control: connection.source_control().cloned(),
            target_view: connection.target_view().cloned(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
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
        let source_control = optional_str(payload, "source_control")?
            .map(transition_trigger_name)
            .transpose()?;
        let target_view = optional_str(payload, "target_view")?
            .map(workflow_owned_definition_name)
            .transpose()?;
        let reason = optional_str(payload, "reason")?
            .map(model_description)
            .transpose()?;

        let target = match (target_slice, target_workflow, payload_contract, reason) {
            (Some(slug), None, payload_contract, None) => WorkflowConnectedEventTarget::Slice {
                slug,
                payload_contract,
            },
            (None, Some(slug), None, Some(reason)) => {
                WorkflowConnectedEventTarget::WorkflowExit { slug, reason }
            }
            _ => return Err("WorkflowConnected has incompatible target fields".to_owned()),
        };

        Ok(Self {
            workflow_slug: workflow,
            source,
            target,
            kind,
            trigger,
            source_control,
            target_view,
        })
    }

    fn into_connection(self) -> WorkflowConnection {
        match self.target {
            WorkflowConnectedEventTarget::Slice {
                slug,
                payload_contract: None,
            } => {
                if let (Some(source_control), Some(target_view)) =
                    (self.source_control, self.target_view)
                {
                    WorkflowConnection::new_with_navigation_endpoints(
                        self.workflow_slug,
                        self.source,
                        slug,
                        self.kind,
                        self.trigger,
                        source_control,
                        target_view,
                    )
                } else {
                    WorkflowConnection::new(
                        self.workflow_slug,
                        self.source,
                        slug,
                        self.kind,
                        self.trigger,
                    )
                }
            }
            WorkflowConnectedEventTarget::Slice {
                slug,
                payload_contract: Some(payload_contract),
            } => WorkflowConnection::new_with_payload_contract(
                self.workflow_slug,
                self.source,
                slug,
                self.kind,
                self.trigger,
                payload_contract,
            ),
            WorkflowConnectedEventTarget::WorkflowExit { slug, reason } => {
                WorkflowConnection::new_workflow_exit(
                    self.workflow_slug,
                    self.source,
                    slug,
                    self.kind,
                    self.trigger,
                    reason,
                )
            }
        }
    }

    fn to_json_value(&self) -> Value {
        let (to, to_workflow, payload_contract, reason): (
            Option<&str>,
            Option<&str>,
            Option<&str>,
            Option<&str>,
        ) = match &self.target {
            WorkflowConnectedEventTarget::Slice {
                slug,
                payload_contract,
            } => (
                Some(slug.as_ref()),
                None,
                payload_contract.as_ref().map(|contract| contract.as_ref()),
                None,
            ),
            WorkflowConnectedEventTarget::WorkflowExit { slug, reason } => {
                (None, Some(slug.as_ref()), None, Some(reason.as_ref()))
            }
        };

        let mut payload = json!({
            "workflow": self.workflow_slug.as_ref(),
            "from": self.source.as_ref(),
            "to": to,
            "to_workflow": to_workflow,
            "via": self.kind.trigger_kind(),
            "name": self.trigger.as_ref(),
            "payload_contract": payload_contract,
            "reason": reason,
        });
        if let Value::Object(fields) = &mut payload {
            if let Some(source_control) = &self.source_control {
                fields.insert(
                    "source_control".to_owned(),
                    Value::String(source_control.as_ref().to_owned()),
                );
            }
            if let Some(target_view) = &self.target_view {
                fields.insert(
                    "target_view".to_owned(),
                    Value::String(target_view.as_ref().to_owned()),
                );
            }
        }
        payload
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowTransitionRemovedEventPayload {
    workflow_slug: WorkflowSlug,
    source: SliceSlug,
    target: WorkflowTransitionRemovedEventTarget,
    kind: ConnectionKind,
    trigger: TransitionTriggerName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum WorkflowTransitionRemovedEventTarget {
    Slice(SliceSlug),
    WorkflowExit(WorkflowSlug),
}

impl WorkflowTransitionRemovedEventPayload {
    fn from_removal(removal: &WorkflowTransitionRemoval) -> Self {
        let target = match removal.target() {
            WorkflowTransitionRemovalTarget::Slice(slug) => {
                WorkflowTransitionRemovedEventTarget::Slice(slug.clone())
            }
            WorkflowTransitionRemovalTarget::Workflow(slug) => {
                WorkflowTransitionRemovedEventTarget::WorkflowExit(slug.clone())
            }
        };
        Self {
            workflow_slug: removal.workflow_slug().clone(),
            source: removal.source().clone(),
            target,
            kind: removal.kind(),
            trigger: removal.trigger().clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        let workflow = workflow_slug(required_str(payload, "workflow")?)?;
        let source = slice_slug(required_str(payload, "from")?)?;
        let target_slice = optional_str(payload, "to")?.map(slice_slug).transpose()?;
        let target_workflow = optional_str(payload, "to_workflow")?
            .map(workflow_slug)
            .transpose()?;
        let kind = connection_kind(required_str(payload, "via")?)?;
        let trigger = transition_trigger_name(required_str(payload, "name")?)?;

        let target = match (target_slice, target_workflow) {
            (Some(slug), None) => WorkflowTransitionRemovedEventTarget::Slice(slug),
            (None, Some(slug)) => WorkflowTransitionRemovedEventTarget::WorkflowExit(slug),
            _ => return Err("WorkflowTransitionRemoved has incompatible target fields".to_owned()),
        };

        Ok(Self {
            workflow_slug: workflow,
            source,
            target,
            kind,
            trigger,
        })
    }

    fn into_removal(self) -> WorkflowTransitionRemoval {
        match self.target {
            WorkflowTransitionRemovedEventTarget::Slice(slug) => WorkflowTransitionRemoval::new(
                self.workflow_slug,
                self.source,
                slug,
                self.kind,
                self.trigger,
            ),
            WorkflowTransitionRemovedEventTarget::WorkflowExit(slug) => {
                WorkflowTransitionRemoval::new_workflow_exit(
                    self.workflow_slug,
                    self.source,
                    slug,
                    self.kind,
                    self.trigger,
                )
            }
        }
    }

    fn to_json_value(&self) -> Value {
        let (to, to_workflow): (Option<&str>, Option<&str>) = match &self.target {
            WorkflowTransitionRemovedEventTarget::Slice(slug) => (Some(slug.as_ref()), None),
            WorkflowTransitionRemovedEventTarget::WorkflowExit(slug) => (None, Some(slug.as_ref())),
        };

        json!({
            "workflow": self.workflow_slug.as_ref(),
            "from": self.source.as_ref(),
            "to": to,
            "to_workflow": to_workflow,
            "via": self.kind.trigger_kind(),
            "name": self.trigger.as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowOutcomeEventPayload {
    workflow: WorkflowSlug,
    outcome: WorkflowOutcomeRecord,
}

impl WorkflowOutcomeEventPayload {
    fn from_parts(workflow: &WorkflowSlug, outcome: &WorkflowOutcomeRecord) -> Self {
        Self {
            workflow: workflow.clone(),
            outcome: outcome.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            workflow: workflow_slug(required_str(payload, "workflow")?)?,
            outcome: WorkflowOutcomeRecord::new(
                workflow_transition_endpoint(required_str(payload, "source_slice")?)?,
                outcome_label_name(required_str(payload, "label")?)?,
                required_bool(payload, "externally_relevant")?,
            ),
        })
    }

    fn into_parts(self) -> (WorkflowSlug, WorkflowOutcomeRecord) {
        (self.workflow, self.outcome)
    }

    fn to_json_value(&self) -> Value {
        json!({
            "workflow": self.workflow.as_ref(),
            "source_slice": self.outcome.source_slice().as_ref(),
            "label": self.outcome.label().as_ref(),
            "externally_relevant": self.outcome.externally_relevant(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowCommandErrorEventPayload {
    workflow: WorkflowSlug,
    error: WorkflowCommandErrorRecord,
}

impl WorkflowCommandErrorEventPayload {
    fn from_parts(workflow: &WorkflowSlug, error: &WorkflowCommandErrorRecord) -> Self {
        Self {
            workflow: workflow.clone(),
            error: error.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            workflow: workflow_slug(required_str(payload, "workflow")?)?,
            error: WorkflowCommandErrorRecord::new(
                workflow_transition_endpoint(required_str(payload, "source_slice")?)?,
                command_name(required_str(payload, "command")?)?,
                command_error_name(required_str(payload, "error")?)?,
            ),
        })
    }

    fn into_parts(self) -> (WorkflowSlug, WorkflowCommandErrorRecord) {
        (self.workflow, self.error)
    }

    fn to_json_value(&self) -> Value {
        json!({
            "workflow": self.workflow.as_ref(),
            "source_slice": self.error.source_slice().as_ref(),
            "command": self.error.command_name().as_ref(),
            "error": self.error.error_name().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowOwnedDefinitionEventPayload {
    workflow: WorkflowSlug,
    definition: WorkflowOwnedDefinitionRecord,
}

impl WorkflowOwnedDefinitionEventPayload {
    fn from_parts(workflow: &WorkflowSlug, definition: &WorkflowOwnedDefinitionRecord) -> Self {
        Self {
            workflow: workflow.clone(),
            definition: definition.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        let workflow = workflow_slug(required_str(payload, "workflow")?)?;
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

        let definition = match (
            definition_stream,
            source_provenance,
            event_participation,
            view_role,
        ) {
            (None, None, None, None) => {
                WorkflowOwnedDefinitionRecord::new(source_slice, definition_kind, definition_name)
            }
            (None, None, None, Some(view_role)) => {
                WorkflowOwnedDefinitionRecord::new_with_view_role(
                    source_slice,
                    definition_kind,
                    definition_name,
                    view_role,
                )
                .ok_or_else(|| "view role requires a view owned-definition kind".to_owned())?
            }
            (Some(definition_stream), Some(source_provenance), None, None) => {
                WorkflowOwnedDefinitionRecord::new_with_event_identity(
                    source_slice,
                    definition_kind,
                    definition_name,
                    definition_stream,
                    source_provenance,
                )
            }
            (Some(definition_stream), Some(source_provenance), Some(event_participation), None) => {
                WorkflowOwnedDefinitionRecord::new_with_event_identity_and_participation(
                    source_slice,
                    definition_kind,
                    definition_name,
                    definition_stream,
                    source_provenance,
                    event_participation,
                )
            }
            _ => {
                return Err(
                    "WorkflowOwnedDefinitionAdded has incompatible optional fields".to_owned(),
                );
            }
        };

        Ok(Self {
            workflow,
            definition,
        })
    }

    fn into_parts(self) -> (WorkflowSlug, WorkflowOwnedDefinitionRecord) {
        (self.workflow, self.definition)
    }

    fn to_json_value(&self) -> Value {
        json!({
            "workflow": self.workflow.as_ref(),
            "source_slice": self.definition.source_slice().as_ref(),
            "definition_kind": self.definition.definition_kind().as_ref(),
            "definition_name": self.definition.definition_name().as_ref(),
            "definition_stream": self
                .definition
                .definition_stream()
                .map(|stream| stream.as_ref()),
            "source_provenance": self
                .definition
                .source_provenance()
                .map(|provenance| provenance.as_ref()),
            "event_participation": self
                .definition
                .event_participation()
                .map(|participation| participation.as_ref()),
            "view_role": self.definition.view_role().map(|role| role.as_ref()),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowTransitionEvidenceEventPayload {
    workflow: WorkflowSlug,
    evidence: WorkflowTransitionEvidenceRecord,
}

impl WorkflowTransitionEvidenceEventPayload {
    fn from_parts(workflow: &WorkflowSlug, evidence: &WorkflowTransitionEvidenceRecord) -> Self {
        Self {
            workflow: workflow.clone(),
            evidence: evidence.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        let source_control = optional_str(payload, "source_control")?
            .map(transition_trigger_name)
            .transpose()?;
        let target_view = optional_str(payload, "target_view")?
            .map(workflow_owned_definition_name)
            .transpose()?;
        let source = workflow_transition_endpoint(required_str(payload, "from")?)?;
        let target = workflow_transition_endpoint(required_str(payload, "to")?)?;
        let kind = workflow_transition_kind(required_str(payload, "via")?)?;
        let trigger = transition_trigger_name(required_str(payload, "name")?)?;
        let source_evidence =
            workflow_transition_source_evidence_text(required_str(payload, "source_evidence")?)?;
        let target_evidence =
            workflow_transition_target_evidence_text(required_str(payload, "target_evidence")?)?;
        let evidence =
            if let (Some(source_control), Some(target_view)) = (source_control, target_view) {
                WorkflowTransitionEvidenceRecord::new_with_navigation_endpoints(
                    source,
                    target,
                    kind,
                    trigger,
                    WorkflowTransitionEvidenceNavigationEndpoints::new(source_control, target_view),
                    source_evidence,
                    target_evidence,
                )
            } else {
                WorkflowTransitionEvidenceRecord::new(
                    source,
                    target,
                    kind,
                    trigger,
                    source_evidence,
                    target_evidence,
                )
            };
        Ok(Self {
            workflow: workflow_slug(required_str(payload, "workflow")?)?,
            evidence,
        })
    }

    fn into_parts(self) -> (WorkflowSlug, WorkflowTransitionEvidenceRecord) {
        (self.workflow, self.evidence)
    }

    fn to_json_value(&self) -> Value {
        let mut payload = json!({
            "workflow": self.workflow.as_ref(),
            "from": self.evidence.source().as_ref(),
            "to": self.evidence.target().as_ref(),
            "via": self.evidence.kind().as_ref(),
            "name": self.evidence.trigger().as_ref(),
            "source_evidence": self.evidence.source_evidence().as_ref(),
            "target_evidence": self.evidence.target_evidence().as_ref(),
        });
        if let Value::Object(fields) = &mut payload {
            if let Some(source_control) = self.evidence.source_control() {
                fields.insert(
                    "source_control".to_owned(),
                    Value::String(source_control.as_ref().to_owned()),
                );
            }
            if let Some(target_view) = self.evidence.target_view() {
                fields.insert(
                    "target_view".to_owned(),
                    Value::String(target_view.as_ref().to_owned()),
                );
            }
        }
        payload
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowEntryLifecycleStateEventPayload {
    workflow: WorkflowSlug,
    coverage: WorkflowEntryLifecycleStateRecord,
}

impl WorkflowEntryLifecycleStateEventPayload {
    fn from_parts(workflow: &WorkflowSlug, coverage: &WorkflowEntryLifecycleStateRecord) -> Self {
        Self {
            workflow: workflow.clone(),
            coverage: coverage.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            workflow: workflow_slug(required_str(payload, "workflow")?)?,
            coverage: WorkflowEntryLifecycleStateRecord::new(
                workflow_entry_lifecycle_state_name(required_str(payload, "state")?)?,
                workflow_transition_endpoint(required_str(payload, "step")?)?,
                workflow_entry_lifecycle_evidence_text(required_str(payload, "evidence")?)?,
            ),
        })
    }

    fn into_parts(self) -> (WorkflowSlug, WorkflowEntryLifecycleStateRecord) {
        (self.workflow, self.coverage)
    }

    fn to_json_value(&self) -> Value {
        json!({
            "workflow": self.workflow.as_ref(),
            "state": self.coverage.state().as_ref(),
            "step": self.coverage.step().as_ref(),
            "evidence": self.coverage.evidence().as_ref(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowReadinessDeclaredEventPayload {
    workflow: WorkflowSlug,
    projection_fingerprint: ProjectionFingerprint,
    model_content_digest: ModelContentDigest,
    verified_at: ReviewTimestamp,
    verified_by: ReviewerId,
    review_event: ReviewEventReference,
}

impl WorkflowReadinessDeclaredEventPayload {
    fn from_parts(
        workflow: &WorkflowSlug,
        projection_fingerprint: &ProjectionFingerprint,
        model_content_digest: &ModelContentDigest,
        verified_at: &ReviewTimestamp,
        verified_by: &ReviewerId,
        review_event: &ReviewEventReference,
    ) -> Self {
        Self {
            workflow: workflow.clone(),
            projection_fingerprint: projection_fingerprint.clone(),
            model_content_digest: model_content_digest.clone(),
            verified_at: verified_at.clone(),
            verified_by: verified_by.clone(),
            review_event: review_event.clone(),
        }
    }

    fn from_json_value(payload: &Value) -> Result<Self, String> {
        let review_event_id = optional_str(payload, "review_event_id")?
            .map(artifact_digest_from_str)
            .transpose()?
            .map(ReviewEventId::new);

        Ok(Self {
            workflow: workflow_slug(required_str(payload, "workflow")?)?,
            projection_fingerprint: ProjectionFingerprint::new(artifact_digest_from_str(
                required_str(payload, "projection_fingerprint")?,
            )?),
            model_content_digest: ModelContentDigest::new(artifact_digest_from_str(required_str(
                payload,
                "model_content_digest",
            )?)?),
            verified_at: review_timestamp(required_str(payload, "verified_at")?)?,
            verified_by: reviewer_id(required_str(payload, "verified_by")?)?,
            review_event: ReviewEventReference::from_optional(review_event_id),
        })
    }

    fn to_json_value(&self) -> Value {
        json!({
            "workflow": self.workflow.as_ref(),
            "projection_fingerprint": self.projection_fingerprint.as_ref(),
            "model_content_digest": self.model_content_digest.as_ref(),
            "verified_at": self.verified_at.as_ref(),
            "verified_by": self.verified_by.as_ref(),
            "review_event_id": self
                .review_event
                .as_review_event_id()
                .map(|event_id| event_id.as_ref()),
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
            Self::WorkflowOutcomeAdded { workflow, outcome } => {
                WorkflowOutcomeEventPayload::from_parts(workflow, outcome).to_json_value()
            }
            Self::WorkflowCommandErrorAdded { workflow, error } => {
                WorkflowCommandErrorEventPayload::from_parts(workflow, error).to_json_value()
            }
            Self::WorkflowOwnedDefinitionAdded {
                workflow,
                definition,
            } => WorkflowOwnedDefinitionEventPayload::from_parts(workflow, definition)
                .to_json_value(),
            Self::WorkflowTransitionEvidenceAdded { workflow, evidence } => {
                WorkflowTransitionEvidenceEventPayload::from_parts(workflow, evidence)
                    .to_json_value()
            }
            Self::WorkflowEntryLifecycleCoverageRequired { workflow } => {
                json!({ "workflow": workflow.as_ref() })
            }
            Self::WorkflowEntryLifecycleStateAdded { workflow, coverage } => {
                WorkflowEntryLifecycleStateEventPayload::from_parts(workflow, coverage)
                    .to_json_value()
            }
            Self::WorkflowReadinessDeclared {
                workflow,
                projection_fingerprint,
                model_content_digest,
                verified_at,
                verified_by,
                review_event,
            } => WorkflowReadinessDeclaredEventPayload::from_parts(
                workflow,
                projection_fingerprint,
                model_content_digest,
                verified_at,
                verified_by,
                review_event,
            )
            .to_json_value(),
            Self::WorkflowConnected { connection } => {
                WorkflowConnectedEventPayload::from_connection(connection).to_json_value()
            }
            Self::WorkflowTransitionRemoved { removal } => {
                WorkflowTransitionRemovedEventPayload::from_removal(removal).to_json_value()
            }
            Self::SliceAdded { slice } => SliceAddedEventPayload::from_slice(slice).to_json_value(),
            Self::SliceUpdated { slice } => {
                SliceUpdatedEventPayload::from_slice_detail(slice).to_json_value()
            }
            Self::SliceRemoved { slug } => json!({ "slug": slug.as_ref() }),
            Self::SliceScenarioAdded { scenario } => {
                SliceScenarioAddedEventPayload::from_scenario(scenario).to_json_value()
            }
            Self::SliceOutcomeAdded { outcome } => {
                SliceOutcomeAddedEventPayload::from_outcome(outcome).to_json_value()
            }
            Self::SliceExternalPayloadAdded { external_payload } => {
                SliceExternalPayloadAddedEventPayload::from_external_payload(external_payload)
                    .to_json_value()
            }
            Self::SliceEventDefinitionAdded { event } => {
                SliceEventDefinitionAddedEventPayload::from_event(event).to_json_value()
            }
            Self::SliceCommandDefinitionAdded { command } => {
                SliceCommandDefinitionAddedEventPayload::from_command(command).to_json_value()
            }
            Self::SliceReadModelAdded { read_model } => {
                SliceReadModelAddedEventPayload::from_read_model(read_model).to_json_value()
            }
            Self::SliceViewAdded { view } => {
                SliceViewAddedEventPayload::from_view(view).to_json_value()
            }
            Self::SliceBitLevelDataFlowAdded { data_flow } => {
                SliceBitLevelDataFlowAddedEventPayload::from_data_flow(data_flow).to_json_value()
            }
            Self::SliceTranslationAdded { translation } => {
                SliceTranslationAddedEventPayload::from_translation(translation).to_json_value()
            }
            Self::SliceAutomationAdded { automation } => {
                SliceAutomationAddedEventPayload::from_automation(automation).to_json_value()
            }
            Self::SliceBoardElementAdded { element } => {
                SliceBoardElementAddedEventPayload::from_element(element).to_json_value()
            }
            Self::SliceBoardConnectionAdded { connection } => {
                SliceBoardConnectionAddedEventPayload::from_connection(connection).to_json_value()
            }
            Self::ReviewRecorded {
                workflow_slug,
                model_content_digest,
                reviewer_id,
                reviewed_at,
                categories,
            } => ReviewRecordedEventPayload::from_parts(
                workflow_slug,
                model_content_digest,
                reviewer_id,
                reviewed_at,
                categories,
            )
            .to_json_value(),
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
            ExportedEventType::WorkflowOutcomeAdded => {
                let (workflow, outcome) =
                    WorkflowOutcomeEventPayload::from_json_value(payload)?.into_parts();
                Ok(Self::WorkflowOutcomeAdded { workflow, outcome })
            }
            ExportedEventType::WorkflowCommandErrorAdded => {
                let (workflow, error) =
                    WorkflowCommandErrorEventPayload::from_json_value(payload)?.into_parts();
                Ok(Self::WorkflowCommandErrorAdded { workflow, error })
            }
            ExportedEventType::WorkflowOwnedDefinitionAdded => {
                let (workflow, definition) =
                    WorkflowOwnedDefinitionEventPayload::from_json_value(payload)?.into_parts();
                Ok(Self::WorkflowOwnedDefinitionAdded {
                    workflow,
                    definition,
                })
            }
            ExportedEventType::WorkflowTransitionEvidenceAdded => {
                let (workflow, evidence) =
                    WorkflowTransitionEvidenceEventPayload::from_json_value(payload)?.into_parts();
                Ok(Self::WorkflowTransitionEvidenceAdded { workflow, evidence })
            }
            ExportedEventType::WorkflowEntryLifecycleCoverageRequired => {
                Ok(Self::WorkflowEntryLifecycleCoverageRequired {
                    workflow: workflow_slug(required_str(payload, "workflow")?)?,
                })
            }
            ExportedEventType::WorkflowEntryLifecycleStateAdded => {
                let (workflow, coverage) =
                    WorkflowEntryLifecycleStateEventPayload::from_json_value(payload)?.into_parts();
                Ok(Self::WorkflowEntryLifecycleStateAdded { workflow, coverage })
            }
            ExportedEventType::WorkflowReadinessDeclared => {
                let readiness = WorkflowReadinessDeclaredEventPayload::from_json_value(payload)?;
                Ok(Self::WorkflowReadinessDeclared {
                    workflow: readiness.workflow,
                    projection_fingerprint: readiness.projection_fingerprint,
                    model_content_digest: readiness.model_content_digest,
                    verified_at: readiness.verified_at,
                    verified_by: readiness.verified_by,
                    review_event: readiness.review_event,
                })
            }
            ExportedEventType::WorkflowConnected => Ok(Self::WorkflowConnected {
                connection: WorkflowConnectedEventPayload::from_json_value(payload)?
                    .into_connection(),
            }),
            ExportedEventType::WorkflowTransitionRemoved => Ok(Self::WorkflowTransitionRemoved {
                removal: WorkflowTransitionRemovedEventPayload::from_json_value(payload)?
                    .into_removal(),
            }),
            ExportedEventType::SliceAdded => Ok(Self::SliceAdded {
                slice: SliceAddedEventPayload::from_json_value(payload)?.into_slice(),
            }),
            ExportedEventType::SliceUpdated => Ok(Self::SliceUpdated {
                slice: SliceUpdatedEventPayload::from_json_value(payload)?.into_slice_detail(),
            }),
            ExportedEventType::SliceRemoved => Ok(Self::SliceRemoved {
                slug: slice_slug(required_str(payload, "slug")?)?,
            }),
            ExportedEventType::SliceScenarioAdded => Ok(Self::SliceScenarioAdded {
                scenario: SliceScenarioAddedEventPayload::from_json_value(payload)?.into_scenario(),
            }),
            ExportedEventType::SliceOutcomeAdded => Ok(Self::SliceOutcomeAdded {
                outcome: SliceOutcomeAddedEventPayload::from_json_value(payload)?.into_outcome(),
            }),
            ExportedEventType::SliceExternalPayloadAdded => Ok(Self::SliceExternalPayloadAdded {
                external_payload: SliceExternalPayloadAddedEventPayload::from_json_value(payload)?
                    .into_external_payload(),
            }),
            ExportedEventType::SliceEventDefinitionAdded => Ok(Self::SliceEventDefinitionAdded {
                event: SliceEventDefinitionAddedEventPayload::from_json_value(payload)?
                    .into_event(),
            }),
            ExportedEventType::SliceCommandDefinitionAdded => {
                Ok(Self::SliceCommandDefinitionAdded {
                    command: SliceCommandDefinitionAddedEventPayload::from_json_value(payload)?
                        .into_command(),
                })
            }
            ExportedEventType::SliceReadModelAdded => Ok(Self::SliceReadModelAdded {
                read_model: SliceReadModelAddedEventPayload::from_json_value(payload)?
                    .into_read_model(),
            }),
            ExportedEventType::SliceViewAdded => Ok(Self::SliceViewAdded {
                view: SliceViewAddedEventPayload::from_json_value(payload)?.into_view(),
            }),
            ExportedEventType::SliceBitLevelDataFlowAdded => Ok(Self::SliceBitLevelDataFlowAdded {
                data_flow: SliceBitLevelDataFlowAddedEventPayload::from_json_value(payload)?
                    .into_data_flow(),
            }),
            ExportedEventType::SliceTranslationAdded => Ok(Self::SliceTranslationAdded {
                translation: SliceTranslationAddedEventPayload::from_json_value(payload)?
                    .into_translation(),
            }),
            ExportedEventType::SliceAutomationAdded => Ok(Self::SliceAutomationAdded {
                automation: SliceAutomationAddedEventPayload::from_json_value(payload)?
                    .into_automation(),
            }),
            ExportedEventType::SliceBoardElementAdded => Ok(Self::SliceBoardElementAdded {
                element: SliceBoardElementAddedEventPayload::from_json_value(payload)?
                    .into_element(),
            }),
            ExportedEventType::SliceBoardConnectionAdded => Ok(Self::SliceBoardConnectionAdded {
                connection: SliceBoardConnectionAddedEventPayload::from_json_value(payload)?
                    .into_connection(),
            }),
            ExportedEventType::ReviewRecorded => {
                Ok(ReviewRecordedEventPayload::from_json_value(payload)?.into_body())
            }
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

    pub(crate) fn body(&self) -> &ExportedEventBody {
        &self.body
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

pub(crate) fn project_exported_events() -> Result<Option<EffectPlan>, String> {
    let events = read_all_emc_events(Path::new("."))?;
    if events.is_empty() {
        return Ok(None);
    }

    let fingerprint = projection_fingerprint(&events)?;
    ProjectedModel::from_events(events)
        .and_then(|model| model.effects(fingerprint))
        .map(Some)
}

/// The command definitions currently projected for `slice_slug` from the
/// authoritative event log. Empty when the project has no events yet or the
/// slice is unknown. Used for write-time validation that resolves references
/// against the source-of-truth model rather than the regenerated artifacts.
pub(crate) fn projected_slice_command_definitions(
    slice_slug: &SliceSlug,
) -> Result<Vec<NewCommandDefinition>, String> {
    let events = read_all_emc_events(Path::new("."))?;
    if events.is_empty() {
        return Ok(Vec::new());
    }
    let model = ProjectedModel::from_events(events)?;
    Ok(model.slice_command_definitions(slice_slug))
}

pub(crate) fn exported_events_projection_fingerprint() -> Result<Option<String>, String> {
    let events = read_all_emc_events(Path::new("."))?;
    if events.is_empty() {
        return Ok(None);
    }

    projection_fingerprint(&events).map(Some)
}

pub(crate) fn list_event_conflicts() -> Result<EffectPlan, String> {
    let forks = list_forks(Path::new("."))?;
    if forks.is_empty() {
        return Ok(EffectPlan::new(vec![Effect::Report(report_line(
            "no event conflicts",
        )?)]));
    }

    let effects = forks
        .into_iter()
        .map(|fork| {
            let branches = fork
                .transactions()
                .iter()
                .map(transaction_id_string)
                .collect::<Vec<_>>()
                .join(",");
            report_line(format!(
                "conflict {} base {} branches {}",
                fork.stream_id().as_ref(),
                usize::from(fork.base_version()),
                branches
            ))
            .map(Effect::Report)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(EffectPlan::new(effects))
}

pub(crate) fn list_stale_workflow_readiness() -> Result<EffectPlan, String> {
    let events = read_all_emc_events(Path::new("."))?;
    if events.is_empty() {
        return Ok(EffectPlan::new(Vec::new()));
    }

    let current_fingerprint = projection_fingerprint_digest(&events)?;
    let mut latest_readiness = BTreeMap::<WorkflowSlug, ProjectionFingerprint>::new();
    for event in &events {
        if let EmcEvent::WorkflowReadinessDeclared {
            workflow,
            projection_fingerprint,
            ..
        } = event
        {
            latest_readiness.insert(workflow.clone(), projection_fingerprint.clone());
        }
    }

    let effects = latest_readiness
        .into_iter()
        .filter(|(_, fingerprint)| *fingerprint != current_fingerprint)
        .map(|(workflow, _)| {
            report_line(format!(
                "workflow {} readiness is stale for current event frontier",
                workflow.as_ref()
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
    // In the eventcore-fs merge model a "conflict" is a fork on a stream
    // (`conflict_id` carries the forked stream id) and the resolution chooses
    // one divergent branch (`chosen_event_id` carries that branch's
    // transaction id). Reconcile records a merge transaction keeping it.
    let resolved = reconcile_choose_branch(
        Path::new("."),
        conflict_id.as_ref(),
        chosen_event_id.as_ref(),
    )?;
    if resolved == 0 {
        return Err(format!(
            "no unresolved conflict for stream {} branch {}",
            conflict_id.as_ref(),
            chosen_event_id.as_ref()
        ));
    }

    Ok(EffectPlan::new(vec![Effect::Report(report_line(
        format!("resolved conflict {}", conflict_id.as_ref()),
    )?)]))
}

pub(crate) fn unresolved_event_conflicts_exist() -> Result<bool, String> {
    Ok(!list_forks(Path::new("."))?.is_empty())
}

pub(crate) fn reject_legacy_artifact_only_project() -> Result<(), String> {
    let has_project_manifest = Path::new("emc.toml").exists();
    let has_generated_artifacts =
        Path::new("model/lean").exists() || Path::new("model/quint").exists();
    let has_event_store = !read_all_emc_events(Path::new("."))?.is_empty();

    if has_project_manifest && has_generated_artifacts && !has_event_store {
        return Err(
            "pre-release upgrade required: generated artifacts exist without a populated event store at model/events"
                .to_owned(),
        );
    }

    Ok(())
}

fn transaction_id_string(id: &eventcore_fs::TransactionId) -> String {
    serde_json::to_value(id)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn projection_fingerprint(events: &[EmcEvent]) -> Result<String, String> {
    projection_fingerprint_digest(events).map(|digest| digest.as_ref().to_owned())
}

fn projection_fingerprint_digest(events: &[EmcEvent]) -> Result<ProjectionFingerprint, String> {
    // The fingerprint identifies the event set behind the current artifacts.
    // Readiness declarations are excluded so declaring readiness does not make
    // a workflow immediately stale. Per-event content digests are sorted so the
    // fingerprint is independent of local-ingestion order (which can differ
    // across replicas after a git merge).
    let mut event_digests = events
        .iter()
        .filter(|event| !matches!(event, EmcEvent::WorkflowReadinessDeclared { .. }))
        .map(|event| {
            serde_json::to_vec(event)
                .map(|bytes| hex::encode(Sha256::digest(bytes)))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    event_digests.sort();
    ArtifactDigest::try_new(hex::encode(Sha256::digest(
        serde_json::to_vec(&event_digests).map_err(|error| error.to_string())?,
    )))
    .map(ProjectionFingerprint::new)
    .map_err(|error| error.to_string())
}

#[derive(Debug)]
struct ProjectedModel {
    project_name: ProjectName,
    workflows: Vec<ProjectedWorkflow>,
    reviews: Vec<ProjectedReview>,
}

impl ProjectedModel {
    fn from_events(events: Vec<EmcEvent>) -> Result<Self, String> {
        events
            .into_iter()
            .try_fold(None::<Self>, Self::apply_event)?
            .ok_or_else(|| "exported events are missing ProjectInitialized".to_owned())
    }

    fn require(model: Option<Self>, event: &str) -> Result<Self, String> {
        model.ok_or_else(|| format!("{event} appeared before project initialization"))
    }

    fn workflow_mut(
        &mut self,
        slug: &WorkflowSlug,
        event: &str,
    ) -> Result<&mut ProjectedWorkflow, String> {
        self.workflows
            .iter_mut()
            .find(|workflow| workflow.slug == *slug)
            .ok_or_else(|| format!("{event} references unknown workflow {}", slug.as_ref()))
    }

    fn slice_mut(&mut self, slug: &SliceSlug, event: &str) -> Result<&mut ProjectedSlice, String> {
        self.workflows
            .iter_mut()
            .flat_map(|workflow| workflow.slices.iter_mut())
            .find(|slice| slice.slug == *slug)
            .ok_or_else(|| format!("{event} references unknown slice {}", slug.as_ref()))
    }

    fn slice_command_definitions(&self, slug: &SliceSlug) -> Vec<NewCommandDefinition> {
        self.workflows
            .iter()
            .flat_map(|workflow| workflow.slices.iter())
            .find(|slice| slice.slug == *slug)
            .map(|slice| slice.command_definitions.clone())
            .unwrap_or_default()
    }

    fn apply_event(model: Option<Self>, event: EmcEvent) -> Result<Option<Self>, String> {
        match event {
            EmcEvent::ProjectInitialized { name, .. } => {
                let (workflows, reviews) = model
                    .map(|model| (model.workflows, model.reviews))
                    .unwrap_or_default();
                Ok(Some(Self {
                    project_name: name,
                    workflows,
                    reviews,
                }))
            }
            EmcEvent::WorkflowAdded {
                slug,
                name,
                description,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowAdded")?;
                model.workflows.push(ProjectedWorkflow {
                    slug,
                    name,
                    description,
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
            EmcEvent::WorkflowUpdated {
                slug,
                name,
                description,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowUpdated")?;
                let workflow = model.workflow_mut(&slug, "WorkflowUpdated")?;
                workflow.name = name;
                workflow.description = description;
                Ok(Some(model))
            }
            EmcEvent::WorkflowRemoved { slug, .. } => {
                let mut model = Self::require(model, "WorkflowRemoved")?;
                let before = model.workflows.len();
                model.workflows.retain(|workflow| workflow.slug != slug);
                if model.workflows.len() == before {
                    return Err(format!(
                        "WorkflowRemoved references unknown workflow {}",
                        slug.as_ref()
                    ));
                }
                Ok(Some(model))
            }
            EmcEvent::SliceAdded {
                workflow,
                slug,
                name,
                kind,
                description,
                ..
            } => {
                let mut model = Self::require(model, "SliceAdded")?;
                let workflow = model.workflow_mut(&workflow, "SliceAdded")?;
                let relationship = if workflow.slices.is_empty() {
                    WorkflowStepRelationshipName::Entry
                } else {
                    WorkflowStepRelationshipName::Main
                };
                workflow.slices.push(ProjectedSlice {
                    slug,
                    name,
                    kind,
                    description,
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
            EmcEvent::SliceUpdated {
                slug,
                name,
                kind,
                description,
                ..
            } => {
                let mut model = Self::require(model, "SliceUpdated")?;
                let projected_slice = model.slice_mut(&slug, "SliceUpdated")?;
                projected_slice.name = name;
                projected_slice.kind = kind;
                projected_slice.description = description;
                Ok(Some(model))
            }
            EmcEvent::SliceRemoved { slug, .. } => {
                let mut model = Self::require(model, "SliceRemoved")?;
                let removed_count = model
                    .workflows
                    .iter_mut()
                    .map(|workflow| {
                        let before = workflow.slices.len();
                        workflow.slices.retain(|slice| slice.slug != slug);
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
            EmcEvent::SliceFactAdded { fact, .. } => {
                let mut model = Self::require(model, "SliceFactAdded")?;
                model.apply_slice_fact_body(fact.to_event_body())?;
                Ok(Some(model))
            }
            EmcEvent::WorkflowReadinessDeclared { .. } => {
                Ok(Some(Self::require(model, "WorkflowReadinessDeclared")?))
            }
            EmcEvent::WorkflowConnected {
                workflow,
                source,
                target_slice,
                target_workflow,
                via,
                name,
                payload_contract,
                reason,
                source_control,
                target_view,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowConnected")?;
                let target_ref = target_slice
                    .as_ref()
                    .map(AsRef::as_ref)
                    .or_else(|| target_workflow.as_ref().map(WorkflowSlug::as_ref))
                    .ok_or_else(|| "WorkflowConnected is missing a target".to_owned())?;
                let workflow_exit = target_workflow.is_some();
                let source_endpoint = workflow_transition_endpoint(source.as_ref())?;
                let target_endpoint = workflow_transition_endpoint(target_ref)?;
                let kind = workflow_transition_kind_from_connection(workflow_exit, via)?;
                let record = match (payload_contract, reason) {
                    (Some(payload_contract), None) => {
                        WorkflowTransitionRecord::new_with_payload_contract(
                            source_endpoint,
                            target_endpoint,
                            kind,
                            name,
                            payload_contract,
                        )
                    }
                    (None, Some(reason)) => WorkflowTransitionRecord::new_with_rationale(
                        source_endpoint,
                        target_endpoint,
                        kind,
                        name,
                        reason,
                    ),
                    (None, None) => {
                        if kind == WorkflowTransitionKind::Navigation
                            && let (Some(source_control), Some(target_view)) =
                                (source_control, target_view)
                        {
                            WorkflowTransitionRecord::new_with_navigation_endpoints(
                                source_endpoint,
                                target_endpoint,
                                kind,
                                name,
                                source_control,
                                target_view,
                            )
                        } else {
                            WorkflowTransitionRecord::new(
                                source_endpoint,
                                target_endpoint,
                                kind,
                                name,
                            )
                        }
                    }
                    (Some(_), Some(_)) => {
                        return Err(
                            "WorkflowConnected has both payload contract and rationale".to_owned()
                        );
                    }
                };
                let workflow = model.workflow_mut(&workflow, "WorkflowConnected")?;
                workflow.transitions.push(record);
                Ok(Some(model))
            }
            EmcEvent::WorkflowTransitionRemoved {
                workflow,
                source,
                target_slice,
                target_workflow,
                via,
                name,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowTransitionRemoved")?;
                let target_ref = target_slice
                    .as_ref()
                    .map(AsRef::as_ref)
                    .or_else(|| target_workflow.as_ref().map(WorkflowSlug::as_ref))
                    .ok_or_else(|| "WorkflowTransitionRemoved is missing a target".to_owned())?;
                let workflow_exit = target_workflow.is_some();
                let removed_transition = WorkflowTransitionRecord::new(
                    workflow_transition_endpoint(source.as_ref())?,
                    workflow_transition_endpoint(target_ref)?,
                    workflow_transition_kind_from_connection(workflow_exit, via)?,
                    name,
                );
                let workflow = model.workflow_mut(&workflow, "WorkflowTransitionRemoved")?;
                workflow
                    .transitions
                    .retain(|transition| !same_transition(transition, &removed_transition));
                Ok(Some(model))
            }
            EmcEvent::WorkflowOutcomeAdded {
                workflow,
                source_slice,
                label,
                externally_relevant,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowOutcomeAdded")?;
                let outcome = WorkflowOutcomeRecord::new(source_slice, label, externally_relevant);
                model
                    .workflow_mut(&workflow, "WorkflowOutcomeAdded")?
                    .outcomes
                    .push(outcome);
                Ok(Some(model))
            }
            EmcEvent::WorkflowCommandErrorAdded {
                workflow,
                source_slice,
                command,
                error,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowCommandErrorAdded")?;
                let record = WorkflowCommandErrorRecord::new(source_slice, command, error);
                model
                    .workflow_mut(&workflow, "WorkflowCommandErrorAdded")?
                    .command_errors
                    .push(record);
                Ok(Some(model))
            }
            EmcEvent::WorkflowOwnedDefinitionAdded {
                workflow,
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
                event_participation,
                view_role,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowOwnedDefinitionAdded")?;
                let definition = WorkflowOwnedDefinitionRecord::from_parts(
                    source_slice,
                    definition_kind,
                    definition_name,
                    definition_stream,
                    source_provenance,
                    event_participation,
                    view_role,
                );
                model
                    .workflow_mut(&workflow, "WorkflowOwnedDefinitionAdded")?
                    .owned_definitions
                    .push(definition);
                Ok(Some(model))
            }
            EmcEvent::WorkflowTransitionEvidenceAdded {
                workflow,
                source,
                target,
                via,
                name,
                source_evidence,
                target_evidence,
                source_control,
                target_view,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowTransitionEvidenceAdded")?;
                let evidence = match (source_control, target_view) {
                    (Some(source_control), Some(target_view)) => {
                        WorkflowTransitionEvidenceRecord::new_with_navigation_endpoints(
                            source,
                            target,
                            via,
                            name,
                            WorkflowTransitionEvidenceNavigationEndpoints::new(
                                source_control,
                                target_view,
                            ),
                            source_evidence,
                            target_evidence,
                        )
                    }
                    _ => WorkflowTransitionEvidenceRecord::new(
                        source,
                        target,
                        via,
                        name,
                        source_evidence,
                        target_evidence,
                    ),
                };
                model
                    .workflow_mut(&workflow, "WorkflowTransitionEvidenceAdded")?
                    .transition_evidences
                    .push(evidence);
                Ok(Some(model))
            }
            EmcEvent::WorkflowEntryLifecycleCoverageRequired { workflow, .. } => {
                let mut model = Self::require(model, "WorkflowEntryLifecycleCoverageRequired")?;
                model
                    .workflow_mut(&workflow, "WorkflowEntryLifecycleCoverageRequired")?
                    .requires_entry_lifecycle_coverage = true;
                Ok(Some(model))
            }
            EmcEvent::WorkflowEntryLifecycleStateAdded {
                workflow,
                state,
                step,
                evidence,
                ..
            } => {
                let mut model = Self::require(model, "WorkflowEntryLifecycleStateAdded")?;
                let record = WorkflowEntryLifecycleStateRecord::new(state, step, evidence);
                model
                    .workflow_mut(&workflow, "WorkflowEntryLifecycleStateAdded")?
                    .entry_lifecycle_states
                    .push(record);
                Ok(Some(model))
            }
            EmcEvent::ReviewRecorded {
                workflow,
                model_content_digest,
                reviewer_id,
                reviewed_at,
                categories,
                ..
            } => {
                let mut model = Self::require(model, "ReviewRecorded")?;
                if !model
                    .workflows
                    .iter()
                    .any(|existing| existing.slug == workflow)
                {
                    return Err(format!(
                        "ReviewRecorded references unknown workflow {}",
                        workflow.as_ref()
                    ));
                }
                let review = ProjectedReview {
                    workflow_slug: workflow,
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
            EmcEvent::ConflictResolved { .. } => Ok(model),
        }
    }

    fn apply_slice_fact_body(&mut self, body: ExportedEventBody) -> Result<(), String> {
        match body {
            ExportedEventBody::SliceOutcomeAdded { outcome } => {
                self.slice_mut(outcome.slice_slug(), "SliceOutcomeAdded")?
                    .outcomes
                    .push(outcome);
            }
            ExportedEventBody::SliceScenarioAdded { scenario } => {
                self.slice_mut(scenario.slice_slug(), "SliceScenarioAdded")?
                    .scenarios
                    .push(scenario);
            }
            ExportedEventBody::SliceExternalPayloadAdded { external_payload } => {
                self.slice_mut(external_payload.slice_slug(), "SliceExternalPayloadAdded")?
                    .external_payloads
                    .push(external_payload);
            }
            ExportedEventBody::SliceEventDefinitionAdded { event } => {
                self.slice_mut(event.slice_slug(), "SliceEventDefinitionAdded")?
                    .event_definitions
                    .push(event);
            }
            ExportedEventBody::SliceCommandDefinitionAdded { command } => {
                self.slice_mut(command.slice_slug(), "SliceCommandDefinitionAdded")?
                    .command_definitions
                    .push(command);
            }
            ExportedEventBody::SliceReadModelAdded { read_model } => {
                self.slice_mut(read_model.slice_slug(), "SliceReadModelAdded")?
                    .read_models
                    .push(read_model);
            }
            ExportedEventBody::SliceBitLevelDataFlowAdded { data_flow } => {
                self.slice_mut(data_flow.slice_slug(), "SliceBitLevelDataFlowAdded")?
                    .bit_level_data_flows
                    .push(data_flow);
            }
            ExportedEventBody::SliceViewAdded { view } => {
                self.slice_mut(view.slice_slug(), "SliceViewAdded")?
                    .views
                    .push(view);
            }
            ExportedEventBody::SliceTranslationAdded { translation } => {
                self.slice_mut(translation.slice_slug(), "SliceTranslationAdded")?
                    .translations
                    .push(translation);
            }
            ExportedEventBody::SliceAutomationAdded { automation } => {
                self.slice_mut(automation.slice_slug(), "SliceAutomationAdded")?
                    .automations
                    .push(automation);
            }
            ExportedEventBody::SliceBoardElementAdded { element } => {
                self.slice_mut(element.slice_slug(), "SliceBoardElementAdded")?
                    .board_elements
                    .push(element);
            }
            ExportedEventBody::SliceBoardConnectionAdded { connection } => {
                self.slice_mut(connection.slice_slug(), "SliceBoardConnectionAdded")?
                    .board_connections
                    .push(connection);
            }
            other => {
                return Err(format!(
                    "unexpected slice fact event body {}",
                    other.event_type()
                ));
            }
        }
        Ok(())
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

    #[test]
    fn slice_added_event_payload_round_trips_between_semantic_slice_and_json_boundary()
    -> Result<(), String> {
        let slice = NewSlice::new(
            workflow_slug("open-ticket")?,
            slice_slug("capture-ticket")?,
            model_name("Capture ticket")?,
            model_description("Actor captures repair ticket details.")?,
            slice_kind_name("state_view")?.into(),
        );

        let payload = SliceAddedEventPayload::from_slice(&slice);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "workflow": "open-ticket",
                "slug": "capture-ticket",
                "name": "Capture ticket",
                "kind": "state_view",
                "description": "Actor captures repair ticket details.",
            })
        );
        assert_eq!(
            SliceAddedEventPayload::from_json_value(&json)?.into_slice(),
            slice
        );

        Ok(())
    }

    #[test]
    fn slice_updated_event_payload_round_trips_between_semantic_slice_detail_and_json_boundary()
    -> Result<(), String> {
        let slice = WorkflowSliceDetail::new(
            slice_slug("capture-ticket")?,
            model_name("Capture ticket")?,
            slice_kind_name("state_change")?,
            model_description("Actor captures updated repair ticket details.")?,
        );

        let payload = SliceUpdatedEventPayload::from_slice_detail(&slice);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slug": "capture-ticket",
                "name": "Capture ticket",
                "kind": "state_change",
                "description": "Actor captures updated repair ticket details.",
            })
        );
        assert_eq!(
            SliceUpdatedEventPayload::from_json_value(&json)?.into_slice_detail(),
            slice
        );

        Ok(())
    }

    #[test]
    fn slice_read_model_added_event_payload_round_trips_derivation_and_transitive_semantics()
    -> Result<(), String> {
        let read_model = NewReadModelDefinition::new(
            slice_slug("capture-ticket")?,
            read_model_name("ticket_state")?,
            NewReadModelField::new(
                datum_name("status_summary")?,
                ReadModelFieldSource::derivation(
                    read_model_derivation_rule("combine status and priority")?,
                    ReadModelDerivationSourceFields::from_fields([
                        datum_name("status")?,
                        datum_name("priority")?,
                    ]),
                    scenario_name("Ticket summary is derived")?,
                ),
                provenance_description("TicketCaptured.status + TicketPrioritized.priority")?,
            ),
        )
        .with_transitive_semantics(
            ReadModelRelationshipFields::from_fields([
                datum_name("ticket_id")?,
                datum_name("customer_id")?,
            ]),
            read_model_transitive_rule("ticket lineage follows customer ownership")?,
            scenario_name("Merged duplicate tickets are visible")?,
        );

        let payload = SliceReadModelAddedEventPayload::from_read_model(&read_model);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "ticket_state",
                "field": {
                    "name": "status_summary",
                    "source_kind": "derivation",
                    "source_event": null,
                    "source_attribute": null,
                    "derivation_rule": "combine status and priority",
                    "derivation_source_fields": ["status", "priority"],
                    "absence_event": null,
                    "derivation_scenario": "Ticket summary is derived",
                    "absence_scenario": null,
                    "provenance": "TicketCaptured.status + TicketPrioritized.priority",
                },
                "transitive": true,
                "relationship_fields": ["ticket_id", "customer_id"],
                "transitive_rule": "ticket lineage follows customer ownership",
                "example_scenario": "Merged duplicate tickets are visible",
            })
        );
        assert_eq!(
            SliceReadModelAddedEventPayload::from_json_value(&json)?.into_read_model(),
            read_model
        );

        Ok(())
    }

    #[test]
    fn slice_view_added_event_payload_round_trips_controls_navigation_and_filters()
    -> Result<(), String> {
        let view = NewViewDefinition::new(
            slice_slug("capture-ticket")?,
            view_name("ticket_summary")?,
            NewViewField::new(
                view_field_name("ticket_title")?,
                view_field_source_kind("read_model")?,
                read_model_name("ticket_state")?,
                view_field_name("title")?,
                sketch_token("title-label")?,
                provenance_description("ticket_state.title")?,
                bit_encoding_semantics("UTF-8 string")?,
            ),
        )
        .with_controls(ViewControls::from_controls([NewControlDefinition::new(
            control_name("AssignTicket")?,
            command_name("AssignTicket")?,
            NewControlInputProvision::new(
                datum_name("assignee_id")?,
                command_input_source_kind("actor")?,
                command_input_source_description("selected assignee")?,
                sketch_token("assignee-picker")?,
                true,
                false,
            ),
            CommandErrorNames::from_names([command_error_name("MissingAssignee")?]),
            control_recovery_behavior("stay_on_screen")?,
            sketch_token("assign-button")?,
            NewNavigationTarget::new(
                navigation_target_type("external_system")?,
                navigation_target_name("dispatch")?,
            )
            .with_external_system(
                navigation_target_name("dispatch-crm")?,
                payload_contract_name("assignment_handoff")?,
            ),
        )]))
        .with_local_states(ViewLocalStates::from_targets([navigation_target_name(
            "assignee_picker",
        )?]))
        .with_filters(ViewFilters::from_targets([navigation_target_name(
            "open_tickets",
        )?]));

        let payload = SliceViewAddedEventPayload::from_view(&view);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "ticket_summary",
                "field": {
                    "name": "ticket_title",
                    "source_kind": "read_model",
                    "source_read_model": "ticket_state",
                    "source_field": "title",
                    "sketch_token": "title-label",
                    "provenance": "ticket_state.title",
                    "bit_encoding": "UTF-8 string",
                },
                "controls": [
                    {
                        "name": "AssignTicket",
                        "command": "AssignTicket",
                        "input": {
                            "name": "assignee_id",
                            "source_kind": "actor",
                            "source_description": "selected assignee",
                            "sketch_token": "assignee-picker",
                            "visible_to_actor": true,
                            "decision_field": false,
                        },
                        "handled_errors": ["MissingAssignee"],
                        "recovery_behavior": "stay_on_screen",
                        "sketch_token": "assign-button",
                        "navigation": {
                            "type": "external_system",
                            "target": "dispatch",
                            "external_workflow": null,
                            "external_system": "dispatch-crm",
                            "handoff_contract": "assignment_handoff",
                        },
                    },
                ],
                "local_states": ["assignee_picker"],
                "filters": ["open_tickets"],
            })
        );
        assert_eq!(
            SliceViewAddedEventPayload::from_json_value(&json)?.into_view(),
            view
        );

        Ok(())
    }

    #[test]
    fn slice_bit_level_data_flow_added_event_payload_round_trips_semantic_data_flow()
    -> Result<(), String> {
        let data_flow = NewBitLevelDataFlow::new(
            slice_slug("capture-ticket")?,
            datum_name("ticket_id")?,
            data_flow_source_kind("modeled_target")?,
            data_flow_source("TicketCaptured.ticket_id")?,
            transformation_semantics("projection")?,
            data_flow_target("ticket_summary.ticket_id")?,
            bit_encoding_semantics("UUID as UTF-8 string")?,
        );

        let payload = SliceBitLevelDataFlowAddedEventPayload::from_data_flow(&data_flow);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "datum": "ticket_id",
                "source_kind": "modeled_target",
                "source": "TicketCaptured.ticket_id",
                "transformation": "projection",
                "target": "ticket_summary.ticket_id",
                "bit_encoding": "UUID as UTF-8 string",
            })
        );
        assert_eq!(
            SliceBitLevelDataFlowAddedEventPayload::from_json_value(&json)?.into_data_flow(),
            data_flow
        );

        Ok(())
    }

    #[test]
    fn slice_translation_added_event_payload_round_trips_external_event_contract_and_command()
    -> Result<(), String> {
        let translation = NewTranslationDefinition::new(
            slice_slug("capture-ticket")?,
            translation_name("capture_ticket_from_webhook")?,
            translation_external_event_name("TicketWebhookReceived")?,
            payload_contract_name("TicketWebhookPayload")?,
            command_name("CaptureTicket")?,
        );

        let payload = SliceTranslationAddedEventPayload::from_translation(&translation);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "capture_ticket_from_webhook",
                "external_event": "TicketWebhookReceived",
                "payload_contract": "TicketWebhookPayload",
                "command": "CaptureTicket",
            })
        );
        assert_eq!(
            SliceTranslationAddedEventPayload::from_json_value(&json)?.into_translation(),
            translation
        );

        Ok(())
    }

    #[test]
    fn slice_automation_added_event_payload_round_trips_trigger_command_errors_and_reaction()
    -> Result<(), String> {
        let automation = NewAutomationDefinition::new(
            slice_slug("capture-ticket")?,
            automation_name("assign_duplicate_ticket")?,
            automation_trigger_name("DuplicateTicketDetected")?,
            command_name("AssignTicket")?,
            CommandErrorNames::from_names([
                command_error_name("MissingAssignee")?,
                command_error_name("AssigneeUnavailable")?,
            ]),
            automation_reaction_description("route duplicate tickets to manual assignment")?,
        );

        let payload = SliceAutomationAddedEventPayload::from_automation(&automation);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "assign_duplicate_ticket",
                "trigger": "DuplicateTicketDetected",
                "command": "AssignTicket",
                "handled_errors": ["MissingAssignee", "AssigneeUnavailable"],
                "reaction": "route duplicate tickets to manual assignment",
            })
        );
        assert_eq!(
            SliceAutomationAddedEventPayload::from_json_value(&json)?.into_automation(),
            automation
        );

        Ok(())
    }

    #[test]
    fn slice_board_element_added_event_payload_round_trips_board_shape_and_main_path()
    -> Result<(), String> {
        let element = NewBoardElement::new(
            slice_slug("capture-ticket")?,
            board_element_name("capture-ticket-card")?,
            board_element_kind("command")?,
            board_lane_id("actions")?,
            board_element_declared_name("Capture ticket")?,
            true,
        );

        let payload = SliceBoardElementAddedEventPayload::from_element(&element);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "capture-ticket-card",
                "kind": "command",
                "lane": "actions",
                "declared_name": "Capture ticket",
                "main_path": true,
            })
        );
        assert_eq!(
            SliceBoardElementAddedEventPayload::from_json_value(&json)?.into_element(),
            element
        );

        Ok(())
    }

    #[test]
    fn slice_board_connection_added_event_payload_round_trips_endpoints_and_kinds()
    -> Result<(), String> {
        let connection = NewBoardConnection::new(
            slice_slug("capture-ticket")?,
            board_connection_endpoint("actor-submit")?,
            board_connection_endpoint_kind("workflow_trigger")?,
            board_connection_endpoint("CaptureTicket")?,
            board_connection_endpoint_kind("command")?,
        );

        let payload = SliceBoardConnectionAddedEventPayload::from_connection(&connection);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "source": "actor-submit",
                "source_kind": "workflow_trigger",
                "target": "CaptureTicket",
                "target_kind": "command",
            })
        );
        assert_eq!(
            SliceBoardConnectionAddedEventPayload::from_json_value(&json)?.into_connection(),
            connection
        );

        Ok(())
    }

    #[test]
    fn review_recorded_event_payload_round_trips_review_metadata_and_categories()
    -> Result<(), String> {
        let payload = ReviewRecordedEventPayload::from_parts(
            &workflow_slug("open-ticket")?,
            &ModelContentDigest::new(artifact_digest_from_str(
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )?),
            &reviewer_id("event-model-reviewer")?,
            &review_timestamp("2026-06-03T00:00:00.000Z")?,
            &[
                ReviewRuleName::LifecycleEntry,
                ReviewRuleName::BoardConnections,
                ReviewRuleName::ScenarioCoverage,
            ],
        );
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "workflow": "open-ticket",
                "model_content_digest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "reviewer_id": "event-model-reviewer",
                "reviewed_at": "2026-06-03T00:00:00.000Z",
                "categories": [
                    "lifecycle-entry",
                    "board-connections",
                    "scenario-coverage",
                ],
            })
        );
        assert_eq!(ReviewRecordedEventPayload::from_json_value(&json)?, payload);

        Ok(())
    }

    #[test]
    fn slice_scenario_added_event_payload_preserves_scenario_kind_shape() -> Result<(), String> {
        let acceptance = NewSliceScenario::new(
            slice_slug("capture-ticket")?,
            ScenarioKind::acceptance(),
            scenario_name("Actor captures ticket")?,
            scenario_step_text("ticket intake screen is open")?,
            scenario_step_text("the actor submits ticket details")?,
            scenario_step_text("the ticket details are visible for review")?,
        )
        .with_streams(
            ScenarioStreamNames::from_streams([stream_name("ticket-events")?]),
            ScenarioStreamNames::from_streams([stream_name("ticket-events")?]),
        );
        let contract = NewSliceScenario::new_contract(
            slice_slug("capture-ticket")?,
            scenario_name("Duplicate ticket is rejected")?,
            scenario_step_text("tickets stream already contains duplicate title")?,
            scenario_step_text("CaptureTicket handles the duplicate title")?,
            scenario_step_text("DuplicateTicket is returned")?,
            contract_kind_name("command")?,
            covered_definition_name("CaptureTicket")?,
        )
        .with_error_references(CommandErrorNames::from_names([command_error_name(
            "DuplicateTicket",
        )?]));

        let acceptance_payload = SliceScenarioAddedEventPayload::from_scenario(&acceptance);
        let acceptance_json = acceptance_payload.to_json_value();
        assert_eq!(
            acceptance_json,
            serde_json::json!({
                "slice": "capture-ticket",
                "kind": "acceptance",
                "name": "Actor captures ticket",
                "given": "ticket intake screen is open",
                "when": "the actor submits ticket details",
                "then": "the ticket details are visible for review",
                "read_streams": ["ticket-events"],
                "written_streams": ["ticket-events"],
                "contract_kind": null,
                "covered_definition": null,
                "error_references": [],
            })
        );
        assert_eq!(
            SliceScenarioAddedEventPayload::from_json_value(&acceptance_json)?.into_scenario(),
            acceptance
        );

        let contract_payload = SliceScenarioAddedEventPayload::from_scenario(&contract);
        let contract_json = contract_payload.to_json_value();
        assert_eq!(
            contract_json,
            serde_json::json!({
                "slice": "capture-ticket",
                "kind": "contract",
                "name": "Duplicate ticket is rejected",
                "given": "tickets stream already contains duplicate title",
                "when": "CaptureTicket handles the duplicate title",
                "then": "DuplicateTicket is returned",
                "read_streams": [],
                "written_streams": [],
                "contract_kind": "command",
                "covered_definition": "CaptureTicket",
                "error_references": ["DuplicateTicket"],
            })
        );
        assert_eq!(
            SliceScenarioAddedEventPayload::from_json_value(&contract_json)?.into_scenario(),
            contract
        );

        let incompatible_json = serde_json::json!({
            "slice": "capture-ticket",
            "kind": "acceptance",
            "name": "Actor captures ticket",
            "given": "ticket intake screen is open",
            "when": "the actor submits ticket details",
            "then": "the ticket details are visible for review",
            "read_streams": [],
            "written_streams": [],
            "contract_kind": "command",
            "covered_definition": "CaptureTicket",
            "error_references": [],
        });
        let error = match SliceScenarioAddedEventPayload::from_json_value(&incompatible_json) {
            Ok(_) => return Err("acceptance scenario with contract fields must fail".to_owned()),
            Err(error) => error,
        };
        assert_eq!(
            error,
            "SliceScenarioAdded has incompatible scenario kind fields"
        );

        Ok(())
    }

    #[test]
    fn slice_outcome_added_event_payload_round_trips_between_semantic_outcome_and_json_boundary()
    -> Result<(), String> {
        let outcome = NewOutcomeDefinition::new(
            slice_slug("capture-ticket")?,
            outcome_label_name("ticket_captured")?,
            OutcomeEventNames::from_events([
                event_name("TicketCaptured")?,
                event_name("TicketQueued")?,
            ]),
            true,
        );

        let payload = SliceOutcomeAddedEventPayload::from_outcome(&outcome);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "label": "ticket_captured",
                "events": ["TicketCaptured", "TicketQueued"],
                "externally_relevant": true,
            })
        );
        assert_eq!(
            SliceOutcomeAddedEventPayload::from_json_value(&json)?.into_outcome(),
            outcome
        );

        Ok(())
    }

    #[test]
    fn slice_external_payload_added_event_payload_round_trips_between_semantic_payload_and_json_boundary()
    -> Result<(), String> {
        let external_payload = NewExternalPayloadDefinition::new(
            slice_slug("capture-ticket")?,
            event_attribute_source_name("TicketForm")?,
            event_attribute_source_field("title")?,
            provenance_description("Submitted by the ticket intake form.")?,
            bit_encoding_semantics("utf8")?,
        );

        let payload =
            SliceExternalPayloadAddedEventPayload::from_external_payload(&external_payload);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "TicketForm",
                "field": "title",
                "field_provenance": "Submitted by the ticket intake form.",
                "bit_encoding": "utf8",
            })
        );
        assert_eq!(
            SliceExternalPayloadAddedEventPayload::from_json_value(&json)?.into_external_payload(),
            external_payload
        );

        Ok(())
    }

    #[test]
    fn slice_event_definition_added_event_payload_round_trips_generated_observed_event_definition()
    -> Result<(), String> {
        let event = NewEventDefinition::new_observed(
            slice_slug("capture-ticket")?,
            event_name("TicketCaptured")?,
            stream_name("tickets")?,
            NewEventAttribute::new_with_generated_source_kind(
                event_attribute_name("ticket_title")?,
                event_attribute_source_kind("generated")?,
                event_attribute_source_name("upstream_event_store")?,
                event_attribute_source_field("ticket_title")?,
                generated_event_attribute_source_kind("event_store_observation")?,
                provenance_description("TicketCaptured.ticket_title")?,
            ),
        );

        let payload = SliceEventDefinitionAddedEventPayload::from_event(&event);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "TicketCaptured",
                "stream": "tickets",
                "attribute": {
                    "name": "ticket_title",
                    "source_kind": "generated",
                    "source_name": "upstream_event_store",
                    "source_field": "ticket_title",
                    "generated_source_kind": "event_store_observation",
                    "provenance": "TicketCaptured.ticket_title",
                },
                "observed": true,
                "shared": false,
            })
        );
        assert_eq!(
            SliceEventDefinitionAddedEventPayload::from_json_value(&json)?.into_event(),
            event
        );

        Ok(())
    }

    #[test]
    fn slice_command_definition_added_event_payload_round_trips_generated_input_errors_and_repeat_behavior()
    -> Result<(), String> {
        let command = NewCommandDefinition::new(
            slice_slug("capture-ticket")?,
            command_name("CaptureTicket")?,
            NewCommandInput::new(
                datum_name("ticket_id")?,
                CommandInputSource::generated(
                    event_attribute_source_name("ticket_id_generator")?,
                    event_attribute_source_field("uuid")?,
                ),
                command_input_source_description("ticket id allocated by generator")?,
                CommandInputProvenanceChain::from_hops([
                    source_chain_hop("ticket_id_generator.uuid")?,
                    source_chain_hop("CaptureTicket.ticket_id")?,
                ]),
            ),
            EmittedEventNames::from_events([
                event_name("TicketCaptured")?,
                event_name("TicketQueued")?,
            ]),
        )
        .with_observed_streams(CommandObservedStreams::from_streams([stream_name(
            "tickets",
        )?]))
        .with_errors(CommandErrorDefinitions::from_errors([
            NewCommandErrorDefinition::new(
                command_error_name("DuplicateTicket")?,
                scenario_name("Duplicate ticket is rejected")?,
                command_error_recovery_kind("retry")?,
            ),
        ]))
        .with_singleton_repeat_behavior(singleton_repeat_behavior("idempotent")?);

        let payload = SliceCommandDefinitionAddedEventPayload::from_command(&command);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "slice": "capture-ticket",
                "name": "CaptureTicket",
                "input": {
                    "name": "ticket_id",
                    "source_kind": "generated",
                    "source_description": "ticket id allocated by generator",
                    "provenance_chain": [
                        "ticket_id_generator.uuid",
                        "CaptureTicket.ticket_id",
                    ],
                    "event_stream_source_event": null,
                    "event_stream_source_attribute": null,
                    "external_payload_source_name": null,
                    "external_payload_source_field": null,
                    "generated_source_name": "ticket_id_generator",
                    "generated_source_field": "uuid",
                    "session_source_name": null,
                    "session_source_field": null,
                    "invocation_argument_source_name": null,
                    "invocation_argument_source_field": null,
                },
                "emitted_events": ["TicketCaptured", "TicketQueued"],
                "observed_streams": ["tickets"],
                "errors": [
                    {
                        "name": "DuplicateTicket",
                        "scenario": "Duplicate ticket is rejected",
                        "recovery": "retry",
                    },
                ],
                "singleton_repeat_behavior": "idempotent",
            })
        );
        assert_eq!(
            SliceCommandDefinitionAddedEventPayload::from_json_value(&json)?.into_command(),
            command
        );

        Ok(())
    }

    #[test]
    fn workflow_connected_event_payload_round_trips_slice_target_with_payload_contract()
    -> Result<(), String> {
        let connection = WorkflowConnection::new_with_payload_contract(
            workflow_slug("open-ticket")?,
            slice_slug("capture-ticket")?,
            slice_slug("review-ticket")?,
            ConnectionKind::navigation(),
            transition_trigger_name("review-ticket-screen")?,
            payload_contract_name("ReviewTicketPayload")?,
        );

        let payload = WorkflowConnectedEventPayload::from_connection(&connection);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "workflow": "open-ticket",
                "from": "capture-ticket",
                "to": "review-ticket",
                "to_workflow": null,
                "via": "navigation",
                "name": "review-ticket-screen",
                "payload_contract": "ReviewTicketPayload",
                "reason": null,
            })
        );
        assert_eq!(
            WorkflowConnectedEventPayload::from_json_value(&json)?.into_connection(),
            connection
        );

        Ok(())
    }

    #[test]
    fn workflow_connected_event_payload_round_trips_workflow_exit_target() -> Result<(), String> {
        let connection = WorkflowConnection::new_workflow_exit(
            workflow_slug("open-ticket")?,
            slice_slug("capture-ticket")?,
            workflow_slug("close-ticket")?,
            ConnectionKind::outcome(),
            transition_trigger_name("ticket-closed")?,
            model_description("Resolved tickets continue in the close-ticket workflow.")?,
        );

        let payload = WorkflowConnectedEventPayload::from_connection(&connection);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "workflow": "open-ticket",
                "from": "capture-ticket",
                "to": null,
                "to_workflow": "close-ticket",
                "via": "outcome",
                "name": "ticket-closed",
                "payload_contract": null,
                "reason": "Resolved tickets continue in the close-ticket workflow.",
            })
        );
        assert_eq!(
            WorkflowConnectedEventPayload::from_json_value(&json)?.into_connection(),
            connection
        );

        Ok(())
    }

    #[test]
    fn workflow_connected_event_payload_rejects_mixed_transition_targets() -> Result<(), String> {
        let json = serde_json::json!({
            "workflow": "open-ticket",
            "from": "capture-ticket",
            "to": "review-ticket",
            "to_workflow": "close-ticket",
            "via": "navigation",
            "name": "review-ticket-screen",
            "payload_contract": null,
            "reason": "Only workflow exits may carry reasons.",
        });

        let error = match WorkflowConnectedEventPayload::from_json_value(&json) {
            Ok(_) => return Err("mixed slice and workflow targets must be rejected".to_owned()),
            Err(error) => error,
        };
        assert_eq!(error, "WorkflowConnected has incompatible target fields");

        Ok(())
    }

    #[test]
    fn workflow_transition_removed_event_payload_round_trips_workflow_exit_target()
    -> Result<(), String> {
        let removal = WorkflowTransitionRemoval::new_workflow_exit(
            workflow_slug("open-ticket")?,
            slice_slug("capture-ticket")?,
            workflow_slug("close-ticket")?,
            ConnectionKind::outcome(),
            transition_trigger_name("ticket-closed")?,
        );

        let payload = WorkflowTransitionRemovedEventPayload::from_removal(&removal);
        let json = payload.to_json_value();

        assert_eq!(
            json,
            serde_json::json!({
                "workflow": "open-ticket",
                "from": "capture-ticket",
                "to": null,
                "to_workflow": "close-ticket",
                "via": "outcome",
                "name": "ticket-closed",
            })
        );
        assert_eq!(
            WorkflowTransitionRemovedEventPayload::from_json_value(&json)?.into_removal(),
            removal
        );

        Ok(())
    }

    #[test]
    fn workflow_owned_definition_event_payload_preserves_optional_field_invariants()
    -> Result<(), String> {
        let workflow = workflow_slug("open-ticket")?;
        let basic_definition = WorkflowOwnedDefinitionRecord::new(
            workflow_transition_endpoint("capture-ticket")?,
            workflow_owned_definition_kind("command")?,
            workflow_owned_definition_name("CaptureTicket")?,
        );
        let event_definition =
            WorkflowOwnedDefinitionRecord::new_with_event_identity_and_participation(
                workflow_transition_endpoint("capture-ticket")?,
                workflow_owned_definition_kind("event")?,
                workflow_owned_definition_name("TicketCaptured")?,
                stream_name("ticket-events")?,
                model_description("CaptureTicket emits TicketCaptured.")?,
                workflow_event_participation("emitted")?,
            );
        let entry_view = WorkflowOwnedDefinitionRecord::new_with_view_role(
            workflow_transition_endpoint("capture-ticket")?,
            workflow_owned_definition_kind("view")?,
            workflow_owned_definition_name("CaptureTicketScreen")?,
            workflow_view_role("entry")?,
        )
        .ok_or_else(|| "view owned definition should accept entry view role".to_owned())?;

        let basic_payload =
            WorkflowOwnedDefinitionEventPayload::from_parts(&workflow, &basic_definition);
        let basic_json = basic_payload.to_json_value();
        assert_eq!(
            basic_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "source_slice": "capture-ticket",
                "definition_kind": "command",
                "definition_name": "CaptureTicket",
                "definition_stream": null,
                "source_provenance": null,
                "event_participation": null,
                "view_role": null,
            })
        );
        assert_eq!(
            WorkflowOwnedDefinitionEventPayload::from_json_value(&basic_json)?.into_parts(),
            (workflow.clone(), basic_definition)
        );

        let event_payload =
            WorkflowOwnedDefinitionEventPayload::from_parts(&workflow, &event_definition);
        let event_json = event_payload.to_json_value();
        assert_eq!(
            event_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "source_slice": "capture-ticket",
                "definition_kind": "event",
                "definition_name": "TicketCaptured",
                "definition_stream": "ticket-events",
                "source_provenance": "CaptureTicket emits TicketCaptured.",
                "event_participation": "emitted",
                "view_role": null,
            })
        );
        assert_eq!(
            WorkflowOwnedDefinitionEventPayload::from_json_value(&event_json)?.into_parts(),
            (workflow.clone(), event_definition)
        );

        let view_payload = WorkflowOwnedDefinitionEventPayload::from_parts(&workflow, &entry_view);
        let view_json = view_payload.to_json_value();
        assert_eq!(
            view_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "source_slice": "capture-ticket",
                "definition_kind": "view",
                "definition_name": "CaptureTicketScreen",
                "definition_stream": null,
                "source_provenance": null,
                "event_participation": null,
                "view_role": "entry",
            })
        );
        assert_eq!(
            WorkflowOwnedDefinitionEventPayload::from_json_value(&view_json)?.into_parts(),
            (workflow, entry_view)
        );

        let incompatible_json = serde_json::json!({
            "workflow": "open-ticket",
            "source_slice": "capture-ticket",
            "definition_kind": "event",
            "definition_name": "TicketCaptured",
            "definition_stream": "ticket-events",
            "source_provenance": null,
            "event_participation": null,
            "view_role": null,
        });
        let error = match WorkflowOwnedDefinitionEventPayload::from_json_value(&incompatible_json) {
            Ok(_) => {
                return Err(
                    "definition stream without source provenance must be rejected".to_owned(),
                );
            }
            Err(error) => error,
        };
        assert_eq!(
            error,
            "WorkflowOwnedDefinitionAdded has incompatible optional fields"
        );

        Ok(())
    }

    #[test]
    fn workflow_readiness_declared_event_payload_round_trips_review_reference() -> Result<(), String>
    {
        let workflow = workflow_slug("open-ticket")?;
        let projection_fingerprint =
            ProjectionFingerprint::new(artifact_digest_from_str("projection-fingerprint")?);
        let model_content_digest =
            ModelContentDigest::new(artifact_digest_from_str("model-content-digest")?);
        let verified_at = review_timestamp("2026-06-08T15:00:00.000Z")?;
        let verified_by = reviewer_id("auto-review")?;
        let recorded_review = ReviewEventReference::from_optional(Some(ReviewEventId::new(
            artifact_digest_from_str("review-event-digest")?,
        )));
        let unrecorded_review = ReviewEventReference::unrecorded();

        let recorded_payload = WorkflowReadinessDeclaredEventPayload::from_parts(
            &workflow,
            &projection_fingerprint,
            &model_content_digest,
            &verified_at,
            &verified_by,
            &recorded_review,
        );
        let recorded_json = recorded_payload.to_json_value();
        assert_eq!(
            recorded_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "projection_fingerprint": "projection-fingerprint",
                "model_content_digest": "model-content-digest",
                "verified_at": "2026-06-08T15:00:00.000Z",
                "verified_by": "auto-review",
                "review_event_id": "review-event-digest",
            })
        );
        assert_eq!(
            WorkflowReadinessDeclaredEventPayload::from_json_value(&recorded_json)?,
            recorded_payload
        );

        let unrecorded_payload = WorkflowReadinessDeclaredEventPayload::from_parts(
            &workflow,
            &projection_fingerprint,
            &model_content_digest,
            &verified_at,
            &verified_by,
            &unrecorded_review,
        );
        let unrecorded_json = unrecorded_payload.to_json_value();
        assert_eq!(
            unrecorded_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "projection_fingerprint": "projection-fingerprint",
                "model_content_digest": "model-content-digest",
                "verified_at": "2026-06-08T15:00:00.000Z",
                "verified_by": "auto-review",
                "review_event_id": null,
            })
        );
        assert_eq!(
            WorkflowReadinessDeclaredEventPayload::from_json_value(&unrecorded_json)?,
            unrecorded_payload
        );

        Ok(())
    }

    #[test]
    fn workflow_fact_event_payloads_round_trip_between_semantic_records_and_json_boundary()
    -> Result<(), String> {
        let workflow = workflow_slug("open-ticket")?;
        let outcome = WorkflowOutcomeRecord::new(
            workflow_transition_endpoint("capture-ticket")?,
            outcome_label_name("ticket_captured")?,
            true,
        );
        let command_error = WorkflowCommandErrorRecord::new(
            workflow_transition_endpoint("capture-ticket")?,
            command_name("CaptureTicket")?,
            command_error_name("DuplicateTicket")?,
        );
        let transition_evidence = WorkflowTransitionEvidenceRecord::new(
            workflow_transition_endpoint("capture-ticket")?,
            workflow_transition_endpoint("review-ticket")?,
            workflow_transition_kind("navigation")?,
            transition_trigger_name("review-ticket-screen")?,
            workflow_transition_source_evidence_text("capture-ticket submits valid details")?,
            workflow_transition_target_evidence_text("review-ticket receives ticket details")?,
        );
        let lifecycle_state = WorkflowEntryLifecycleStateRecord::new(
            workflow_entry_lifecycle_state_name("fresh_uninitialized")?,
            workflow_transition_endpoint("capture-ticket")?,
            workflow_entry_lifecycle_evidence_text(
                "capture-ticket distinguishes first arrival before initialization",
            )?,
        );

        let outcome_payload = WorkflowOutcomeEventPayload::from_parts(&workflow, &outcome);
        let outcome_json = outcome_payload.to_json_value();
        assert_eq!(
            outcome_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "source_slice": "capture-ticket",
                "label": "ticket_captured",
                "externally_relevant": true,
            })
        );
        assert_eq!(
            WorkflowOutcomeEventPayload::from_json_value(&outcome_json)?.into_parts(),
            (workflow.clone(), outcome)
        );

        let command_error_payload =
            WorkflowCommandErrorEventPayload::from_parts(&workflow, &command_error);
        let command_error_json = command_error_payload.to_json_value();
        assert_eq!(
            command_error_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "source_slice": "capture-ticket",
                "command": "CaptureTicket",
                "error": "DuplicateTicket",
            })
        );
        assert_eq!(
            WorkflowCommandErrorEventPayload::from_json_value(&command_error_json)?.into_parts(),
            (workflow.clone(), command_error)
        );

        let transition_evidence_payload =
            WorkflowTransitionEvidenceEventPayload::from_parts(&workflow, &transition_evidence);
        let transition_evidence_json = transition_evidence_payload.to_json_value();
        assert_eq!(
            transition_evidence_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "from": "capture-ticket",
                "to": "review-ticket",
                "via": "navigation",
                "name": "review-ticket-screen",
                "source_evidence": "capture-ticket submits valid details",
                "target_evidence": "review-ticket receives ticket details",
            })
        );
        assert_eq!(
            WorkflowTransitionEvidenceEventPayload::from_json_value(&transition_evidence_json)?
                .into_parts(),
            (workflow.clone(), transition_evidence)
        );

        let lifecycle_state_payload =
            WorkflowEntryLifecycleStateEventPayload::from_parts(&workflow, &lifecycle_state);
        let lifecycle_state_json = lifecycle_state_payload.to_json_value();
        assert_eq!(
            lifecycle_state_json,
            serde_json::json!({
                "workflow": "open-ticket",
                "state": "fresh_uninitialized",
                "step": "capture-ticket",
                "evidence": "capture-ticket distinguishes first arrival before initialization",
            })
        );
        assert_eq!(
            WorkflowEntryLifecycleStateEventPayload::from_json_value(&lifecycle_state_json)?
                .into_parts(),
            (workflow, lifecycle_state)
        );

        Ok(())
    }
}
