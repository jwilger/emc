// Copyright 2026 John Wilger

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
use crate::core::effect::{
    ArtifactDigest, Effect, EffectPlan, FileContents, ProjectPath, ReportLine,
};
use crate::core::emit::lean::{
    emit_slice_module as emit_lean_slice_module, emit_workflow_module as emit_lean_workflow_module,
};
use crate::core::emit::quint::{
    emit_slice_module as emit_quint_slice_module,
    emit_workflow_module as emit_quint_workflow_module,
};
use crate::core::formal_slice_facts::{
    CommandErrorDefinitions, CommandErrorNames, CommandInputProvenanceChain,
    CommandObservedStreams, EmittedEventNames, NewAutomationDefinition, NewBitLevelDataFlow,
    NewBoardConnection, NewBoardElement, NewCommandDefinition, NewCommandErrorDefinition,
    NewCommandInput, NewControlDefinition, NewControlInputProvision, NewEventAttribute,
    NewEventDefinition, NewExternalPayloadDefinition, NewNavigationTarget, NewOutcomeDefinition,
    NewReadModelDefinition, NewReadModelField, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition, NewViewField, OutcomeEventNames, ReadModelDerivationSourceFields,
    ReadModelRelationshipFields, ScenarioKind, ScenarioStreamNames, ViewControls, ViewFilters,
    ViewLocalStates,
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
    EventAttributeSourceKind, EventAttributeSourceName, EventName, LeanModuleName,
    ModelDescription, ModelName, NavigationTargetName, NavigationTargetType, OutcomeLabelName,
    PayloadContractName, ProvenanceDescription, QuintModuleName, ReadModelDerivationRule,
    ReadModelFieldSourceKind, ReadModelName, ReadModelTransitiveRule, ReviewRuleName,
    ReviewTimestamp, ReviewerId, ScenarioName, ScenarioStepText, SingletonRepeatBehavior,
    SketchToken, SliceKindName, SliceSlug, SourceChainHop, StreamName, TransformationSemantics,
    TransitionTriggerName, TranslationExternalEventName, TranslationName, ViewFieldName,
    ViewFieldSourceKind, ViewName, WorkflowCommandErrorRecord, WorkflowCommandErrorRecords,
    WorkflowEntryLifecycleEvidenceText, WorkflowEntryLifecycleStateName,
    WorkflowEntryLifecycleStateRecord, WorkflowEntryLifecycleStateRecords,
    WorkflowEventParticipation, WorkflowModuleData, WorkflowOutcomeRecord, WorkflowOutcomeRecords,
    WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord,
    WorkflowOwnedDefinitionRecords, WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug,
    WorkflowStepRelationshipName, WorkflowTransitionEndpoint, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionEvidenceText, WorkflowTransitionKind,
    WorkflowTransitionRecord, WorkflowTransitionRecords, WorkflowViewRole,
};
use crate::core::workflow::NewWorkflow;

const SCHEMA_VERSION: &str = "emc.events.v1";
const EVENT_EXPORT_DIRECTORY: &str = "model/events/v1";
const PROJECTION_FINGERPRINT_PATH: &str = "model/events/projection.fingerprint";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventDraft {
    stream_id: String,
    event_type: String,
    payload: Value,
}

impl EventDraft {
    pub fn project_initialized(name: &ProjectName) -> Self {
        Self {
            stream_id: "project".to_owned(),
            event_type: "ProjectInitialized".to_owned(),
            payload: json!({ "name": name.as_ref() }),
        }
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn stream_id(&self) -> &str {
        &self.stream_id
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }

    pub fn workflow_added(workflow: &NewWorkflow) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.slug().as_ref()),
            event_type: "WorkflowAdded".to_owned(),
            payload: json!({
                "slug": workflow.slug().as_ref(),
                "name": workflow.name().as_ref(),
                "description": workflow.description().as_ref(),
            }),
        }
    }

    pub fn workflow_updated(workflow: &NewWorkflow) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.slug().as_ref()),
            event_type: "WorkflowUpdated".to_owned(),
            payload: json!({
                "slug": workflow.slug().as_ref(),
                "name": workflow.name().as_ref(),
                "description": workflow.description().as_ref(),
            }),
        }
    }

    pub fn workflow_removed(workflow: &WorkflowSlug) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowRemoved".to_owned(),
            payload: json!({
                "slug": workflow.as_ref(),
            }),
        }
    }

    pub fn workflow_outcome_added(
        workflow: &WorkflowSlug,
        outcome: &WorkflowOutcomeRecord,
    ) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowOutcomeAdded".to_owned(),
            payload: json!({
                "workflow": workflow.as_ref(),
                "source_slice": outcome.source_slice().as_ref(),
                "label": outcome.label().as_ref(),
                "externally_relevant": outcome.externally_relevant(),
            }),
        }
    }

    pub fn workflow_command_error_added(
        workflow: &WorkflowSlug,
        error: &WorkflowCommandErrorRecord,
    ) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowCommandErrorAdded".to_owned(),
            payload: json!({
                "workflow": workflow.as_ref(),
                "source_slice": error.source_slice().as_ref(),
                "command": error.command_name().as_ref(),
                "error": error.error_name().as_ref(),
            }),
        }
    }

    pub fn workflow_owned_definition_added(
        workflow: &WorkflowSlug,
        definition: &WorkflowOwnedDefinitionRecord,
    ) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowOwnedDefinitionAdded".to_owned(),
            payload: json!({
                "workflow": workflow.as_ref(),
                "source_slice": definition.source_slice().as_ref(),
                "definition_kind": definition.definition_kind().as_ref(),
                "definition_name": definition.definition_name().as_ref(),
                "definition_stream": definition
                    .definition_stream()
                    .map(|stream| stream.as_ref()),
                "source_provenance": definition
                    .source_provenance()
                    .map(|provenance| provenance.as_ref()),
                "event_participation": definition
                    .event_participation()
                    .map(|participation| participation.as_ref()),
                "view_role": definition.view_role().map(|role| role.as_ref()),
            }),
        }
    }

    pub fn workflow_transition_evidence_added(
        workflow: &WorkflowSlug,
        evidence: &WorkflowTransitionEvidenceRecord,
    ) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowTransitionEvidenceAdded".to_owned(),
            payload: json!({
                "workflow": workflow.as_ref(),
                "from": evidence.source().as_ref(),
                "to": evidence.target().as_ref(),
                "via": evidence.kind().as_ref(),
                "name": evidence.trigger().as_ref(),
                "source_evidence": evidence.source_evidence().as_ref(),
                "target_evidence": evidence.target_evidence().as_ref(),
            }),
        }
    }

    pub fn workflow_entry_lifecycle_coverage_required(workflow: &WorkflowSlug) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowEntryLifecycleCoverageRequired".to_owned(),
            payload: json!({
                "workflow": workflow.as_ref(),
            }),
        }
    }

    pub fn workflow_entry_lifecycle_state_added(
        workflow: &WorkflowSlug,
        coverage: &WorkflowEntryLifecycleStateRecord,
    ) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowEntryLifecycleStateAdded".to_owned(),
            payload: json!({
                "workflow": workflow.as_ref(),
                "state": coverage.state().as_ref(),
                "step": coverage.step().as_ref(),
                "evidence": coverage.evidence().as_ref(),
            }),
        }
    }

    pub fn workflow_readiness_declared(
        workflow: &WorkflowSlug,
        projection_fingerprint: &ArtifactDigest,
        model_content_digest: &ArtifactDigest,
        verified_at: &ReviewTimestamp,
        verified_by: &ReviewerId,
        review_event_id: Option<&ArtifactDigest>,
    ) -> Self {
        Self {
            stream_id: format!("workflow::{}", workflow.as_ref()),
            event_type: "WorkflowReadinessDeclared".to_owned(),
            payload: json!({
                "workflow": workflow.as_ref(),
                "projection_fingerprint": projection_fingerprint.as_ref(),
                "model_content_digest": model_content_digest.as_ref(),
                "verified_at": verified_at.as_ref(),
                "verified_by": verified_by.as_ref(),
                "review_event_id": review_event_id.map(ArtifactDigest::as_ref),
            }),
        }
    }

    pub fn workflow_connected(connection: &WorkflowConnection) -> Self {
        Self {
            stream_id: format!("workflow::{}", connection.workflow_slug().as_ref()),
            event_type: "WorkflowConnected".to_owned(),
            payload: json!({
                "workflow": connection.workflow_slug().as_ref(),
                "from": connection.source().as_ref(),
                "to": connection
                    .target()
                    .slice_slug()
                    .map(|target| target.as_ref()),
                "to_workflow": connection
                    .target()
                    .workflow_slug()
                    .map(|target| target.as_ref()),
                "via": connection.kind().trigger_kind(),
                "name": connection.trigger().as_ref(),
                "payload_contract": connection
                    .payload_contract()
                    .map(|contract| contract.as_ref()),
                "reason": connection.target().reason().map(|reason| reason.as_ref()),
            }),
        }
    }

    pub fn workflow_transition_removed(removal: &WorkflowTransitionRemoval) -> Self {
        Self {
            stream_id: format!("workflow::{}", removal.workflow_slug().as_ref()),
            event_type: "WorkflowTransitionRemoved".to_owned(),
            payload: json!({
                "workflow": removal.workflow_slug().as_ref(),
                "from": removal.source().as_ref(),
                "to": removal.target().slice_slug().map(|target| target.as_ref()),
                "to_workflow": removal
                    .target()
                    .workflow_slug()
                    .map(|target| target.as_ref()),
                "via": removal.kind().trigger_kind(),
                "name": removal.trigger().as_ref(),
            }),
        }
    }

    pub fn slice_added(slice: &NewSlice) -> Self {
        Self {
            stream_id: format!("slice::{}", slice.slug().as_ref()),
            event_type: "SliceAdded".to_owned(),
            payload: json!({
                "workflow": slice.workflow_slug().as_ref(),
                "slug": slice.slug().as_ref(),
                "name": slice.name().as_ref(),
                "kind": slice.kind().as_str(),
                "description": slice.description().as_ref(),
            }),
        }
    }

    pub fn slice_updated(slice: &WorkflowSliceDetail) -> Self {
        Self {
            stream_id: format!("slice::{}", slice.slug().as_ref()),
            event_type: "SliceUpdated".to_owned(),
            payload: json!({
                "slug": slice.slug().as_ref(),
                "name": slice.name().as_ref(),
                "kind": slice.kind().as_ref(),
                "description": slice.description().as_ref(),
            }),
        }
    }

    pub fn slice_removed(slice: &WorkflowSliceDetail) -> Self {
        Self {
            stream_id: format!("slice::{}", slice.slug().as_ref()),
            event_type: "SliceRemoved".to_owned(),
            payload: json!({
                "slug": slice.slug().as_ref(),
            }),
        }
    }

    pub fn slice_scenario_added(scenario: &NewSliceScenario) -> Self {
        Self {
            stream_id: format!("slice::{}", scenario.slice_slug().as_ref()),
            event_type: "SliceScenarioAdded".to_owned(),
            payload: json!({
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
            }),
        }
    }

    pub fn slice_outcome_added(outcome: &NewOutcomeDefinition) -> Self {
        Self {
            stream_id: format!("slice::{}", outcome.slice_slug().as_ref()),
            event_type: "SliceOutcomeAdded".to_owned(),
            payload: json!({
                "slice": outcome.slice_slug().as_ref(),
                "label": outcome.label().as_ref(),
                "events": outcome
                    .event_set()
                    .as_slice()
                    .iter()
                    .map(|event| event.as_ref())
                    .collect::<Vec<_>>(),
                "externally_relevant": outcome.externally_relevant(),
            }),
        }
    }

    pub fn slice_external_payload_added(external_payload: &NewExternalPayloadDefinition) -> Self {
        Self {
            stream_id: format!("slice::{}", external_payload.slice_slug().as_ref()),
            event_type: "SliceExternalPayloadAdded".to_owned(),
            payload: json!({
                "slice": external_payload.slice_slug().as_ref(),
                "name": external_payload.name().as_ref(),
                "field": external_payload.field().as_ref(),
                "field_provenance": external_payload.field_provenance().as_ref(),
                "bit_encoding": external_payload.bit_encoding().as_ref(),
            }),
        }
    }

    pub fn slice_event_definition_added(event: &NewEventDefinition) -> Self {
        let attribute = event.attribute();
        Self {
            stream_id: format!("slice::{}", event.slice_slug().as_ref()),
            event_type: "SliceEventDefinitionAdded".to_owned(),
            payload: json!({
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
            }),
        }
    }

    pub fn slice_command_definition_added(command: &NewCommandDefinition) -> Self {
        Self {
            stream_id: format!("slice::{}", command.slice_slug().as_ref()),
            event_type: "SliceCommandDefinitionAdded".to_owned(),
            payload: json!({
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
            }),
        }
    }

    pub fn slice_read_model_added(read_model: &NewReadModelDefinition) -> Self {
        Self {
            stream_id: format!("slice::{}", read_model.slice_slug().as_ref()),
            event_type: "SliceReadModelAdded".to_owned(),
            payload: json!({
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
            }),
        }
    }

    pub fn slice_view_added(view: &NewViewDefinition) -> Self {
        Self {
            stream_id: format!("slice::{}", view.slice_slug().as_ref()),
            event_type: "SliceViewAdded".to_owned(),
            payload: json!({
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
            }),
        }
    }

    pub fn slice_bit_level_data_flow_added(data_flow: &NewBitLevelDataFlow) -> Self {
        Self {
            stream_id: format!("slice::{}", data_flow.slice_slug().as_ref()),
            event_type: "SliceBitLevelDataFlowAdded".to_owned(),
            payload: json!({
                "slice": data_flow.slice_slug().as_ref(),
                "datum": data_flow.datum().as_ref(),
                "source": data_flow.source().as_ref(),
                "source_kind": data_flow.source_kind().as_ref(),
                "transformation": data_flow.transformation().as_ref(),
                "target": data_flow.target().as_ref(),
                "bit_encoding": data_flow.bit_encoding().as_ref(),
            }),
        }
    }

    pub fn slice_translation_added(translation: &NewTranslationDefinition) -> Self {
        Self {
            stream_id: format!("slice::{}", translation.slice_slug().as_ref()),
            event_type: "SliceTranslationAdded".to_owned(),
            payload: json!({
                "slice": translation.slice_slug().as_ref(),
                "name": translation.name().as_ref(),
                "external_event": translation.external_event_name().as_ref(),
                "payload_contract": translation.payload_contract_name().as_ref(),
                "command": translation.command_name().as_ref(),
            }),
        }
    }

    pub fn slice_automation_added(automation: &NewAutomationDefinition) -> Self {
        Self {
            stream_id: format!("slice::{}", automation.slice_slug().as_ref()),
            event_type: "SliceAutomationAdded".to_owned(),
            payload: json!({
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
            }),
        }
    }

    pub fn slice_board_element_added(element: &NewBoardElement) -> Self {
        Self {
            stream_id: format!("slice::{}", element.slice_slug().as_ref()),
            event_type: "SliceBoardElementAdded".to_owned(),
            payload: json!({
                "slice": element.slice_slug().as_ref(),
                "name": element.name().as_ref(),
                "kind": element.kind().as_ref(),
                "lane": element.lane().as_ref(),
                "declared_name": element.declared_name().as_ref(),
                "main_path": element.main_path(),
            }),
        }
    }

    pub fn slice_board_connection_added(connection: &NewBoardConnection) -> Self {
        Self {
            stream_id: format!("slice::{}", connection.slice_slug().as_ref()),
            event_type: "SliceBoardConnectionAdded".to_owned(),
            payload: json!({
                "slice": connection.slice_slug().as_ref(),
                "source": connection.source().as_ref(),
                "source_kind": connection.source_kind().as_ref(),
                "target": connection.target().as_ref(),
                "target_kind": connection.target_kind().as_ref(),
            }),
        }
    }

    pub fn conflict_resolved(
        conflict_id: &ArtifactDigest,
        chosen_event_id: &ArtifactDigest,
    ) -> Self {
        Self {
            stream_id: "project".to_owned(),
            event_type: "ConflictResolved".to_owned(),
            payload: json!({
                "conflict_id": conflict_id.as_ref(),
                "chosen_event_id": chosen_event_id.as_ref(),
            }),
        }
    }

    pub fn review_recorded(
        workflow_slug: &WorkflowSlug,
        model_content_digest: &ArtifactDigest,
        reviewer_id: &ReviewerId,
        reviewed_at: &ReviewTimestamp,
        categories: &[ReviewRuleName],
    ) -> Self {
        Self {
            stream_id: format!("review::{}", workflow_slug.as_ref()),
            event_type: "ReviewRecorded".to_owned(),
            payload: json!({
                "workflow": workflow_slug.as_ref(),
                "model_content_digest": model_content_digest.as_ref(),
                "reviewer_id": reviewer_id.as_ref(),
                "reviewed_at": reviewed_at.as_ref(),
                "categories": categories
                    .iter()
                    .map(ReviewRuleName::as_ref)
                    .collect::<Vec<_>>(),
            }),
        }
    }
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

pub fn export_event_file_contents(draft: &EventDraft) -> Result<(String, FileContents), String> {
    let parents = exported_event_ids()?;
    let command_ordinal = command_ordinal_for_stream(&draft.stream_id)?;
    let event_id = event_id(draft, &parents, command_ordinal)?;
    let command_id = command_id(&event_id);
    let exported = json!({
        "schema_version": SCHEMA_VERSION,
        "event_id": event_id.clone(),
        "command_id": command_id,
        "command_ordinal": command_ordinal,
        "stream_id": draft.stream_id.clone(),
        "parents": parents,
        "type": draft.event_type.clone(),
        "payload": draft.payload.clone(),
    });
    let json = serde_json::to_string_pretty(&exported).map_err(|error| error.to_string())?;
    let contents = FileContents::try_new(format!("{json}\n")).map_err(|error| error.to_string())?;
    Ok((
        format!("{EVENT_EXPORT_DIRECTORY}/{event_id}.json"),
        contents,
    ))
}

pub fn project_exported_events() -> Result<Option<EffectPlan>, String> {
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

pub fn exported_events_projection_fingerprint() -> Result<Option<String>, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(None);
    }

    let events = exported_events_in_topological_order(path)?;
    if events.is_empty() {
        return Ok(None);
    }

    projection_fingerprint(&events).map(Some)
}

pub fn list_event_conflicts() -> Result<EffectPlan, String> {
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
            let event_ids = conflict.event_ids.into_iter().collect::<Vec<_>>().join(",");
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

pub fn list_stale_workflow_readiness() -> Result<EffectPlan, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(EffectPlan::new(Vec::new()));
    }

    let events = exported_events_in_topological_order(path)?;
    let current_fingerprint = projection_fingerprint_digest(&events)?;
    let latest_readiness = events.into_iter().try_fold(
        BTreeMap::<WorkflowSlug, WorkflowReadinessDeclaration>::new(),
        |mut declarations, event| -> Result<_, String> {
            if event.get("type").and_then(Value::as_str) == Some("WorkflowReadinessDeclared") {
                let payload = required_object(&event, "payload")?;
                let declaration = workflow_readiness_declaration_from_payload(payload)?;
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

pub fn resolve_event_conflict(
    conflict_id: String,
    chosen_event_id: String,
) -> Result<EffectPlan, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Err(format!("unknown event conflict {conflict_id}"));
    }

    let conflict = event_conflicts(path)?
        .into_iter()
        .find(|conflict| conflict.id == conflict_id)
        .ok_or_else(|| format!("unknown event conflict {conflict_id}"))?;
    if !conflict.event_ids.contains(&chosen_event_id) {
        return Err(format!(
            "event {chosen_event_id} is not part of conflict {conflict_id}"
        ));
    }

    Ok(EffectPlan::new(vec![
        Effect::ExportEvent(EventDraft::conflict_resolved(
            &artifact_digest_from_str(&conflict_id)?,
            &artifact_digest_from_str(&chosen_event_id)?,
        )),
        Effect::Report(report_line(format!("resolved conflict {conflict_id}"))?),
    ]))
}

pub fn unresolved_event_conflicts_exist() -> Result<bool, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(false);
    }

    event_conflicts(path).map(|conflicts| !conflicts.is_empty())
}

pub fn reject_legacy_artifact_only_project() -> Result<(), String> {
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

fn exported_event_ids() -> Result<Vec<String>, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut event_ids = fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .map(|entry| {
            entry.map_err(|error| error.to_string()).and_then(|entry| {
                let contents =
                    fs::read_to_string(entry.path()).map_err(|error| error.to_string())?;
                let event =
                    serde_json::from_str::<Value>(&contents).map_err(|error| error.to_string())?;
                event
                    .get("event_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or_else(|| "exported event is missing event_id".to_owned())
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    event_ids.sort();
    Ok(event_ids)
}

#[derive(Debug)]
struct EventConflict {
    id: String,
    stream_id: String,
    event_type: String,
    semantic_key: String,
    event_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ConflictKey {
    stream_id: String,
    event_type: String,
    semantic_key: String,
    parents: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ConflictPayload {
    event_id: String,
    payload: String,
}

fn event_conflicts(path: &Path) -> Result<Vec<EventConflict>, String> {
    let events = exported_events_in_topological_order(path)?;
    let resolved_conflicts = resolved_conflict_ids(&events)?;
    let mut grouped = BTreeMap::<ConflictKey, BTreeSet<ConflictPayload>>::new();

    for event in events {
        if let Some(key) = conflict_key(&event)? {
            grouped.entry(key).or_default().insert(ConflictPayload {
                event_id: required_str(&event, "event_id")?.to_owned(),
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

fn resolved_conflict_ids(events: &[Value]) -> Result<BTreeSet<String>, String> {
    events
        .iter()
        .filter(|event| event.get("type").and_then(Value::as_str) == Some("ConflictResolved"))
        .map(|event| {
            let payload = required_object(event, "payload")?;
            required_str(payload, "conflict_id").map(str::to_owned)
        })
        .collect()
}

fn conflict_id(key: &ConflictKey) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(&json!({
            "stream_id": key.stream_id,
            "type": key.event_type,
            "semantic_key": key.semantic_key,
            "parents": key.parents,
        }))
        .unwrap_or_default(),
    ))
}

fn conflict_key(event: &Value) -> Result<Option<ConflictKey>, String> {
    let event_type = required_str(event, "type")?;
    if !matches!(event_type, "WorkflowUpdated" | "SliceUpdated") {
        return Ok(None);
    }

    let payload = required_object(event, "payload")?;
    let semantic_key = required_str(payload, "slug")?.to_owned();
    Ok(Some(ConflictKey {
        stream_id: required_str(event, "stream_id")?.to_owned(),
        event_type: event_type.to_owned(),
        semantic_key,
        parents: parents(event)?,
    }))
}

fn parents(event: &Value) -> Result<Vec<String>, String> {
    let mut parents = event
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| "exported event is missing parents".to_owned())?
        .iter()
        .map(|parent| {
            parent
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "exported event parent must be a string".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    parents.sort();
    Ok(parents)
}

fn canonical_payload(event: &Value) -> Result<String, String> {
    serde_json::to_string(required_object(event, "payload")?).map_err(|error| error.to_string())
}

fn exported_events_in_topological_order(path: &Path) -> Result<Vec<Value>, String> {
    let mut events = fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .map(|entry| {
            entry
                .map_err(|error| error.to_string())
                .and_then(|entry| read_event_file(&entry.path()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    events.sort_by(|left, right| {
        parent_count(left)
            .cmp(&parent_count(right))
            .then(event_id_value(left).cmp(event_id_value(right)))
    });
    Ok(events)
}

fn read_event_file(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<Value>(&contents).map_err(|error| error.to_string())
}

fn parent_count(event: &Value) -> usize {
    event
        .get("parents")
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

fn event_id_value(event: &Value) -> &str {
    event.get("event_id").and_then(Value::as_str).unwrap_or("")
}

fn projection_fingerprint(events: &[Value]) -> Result<String, String> {
    projection_fingerprint_digest(events).map(|digest| digest.as_ref().to_owned())
}

fn projection_fingerprint_digest(events: &[Value]) -> Result<ArtifactDigest, String> {
    let event_ids = events
        .iter()
        .filter(|event| {
            event.get("type").and_then(Value::as_str) != Some("WorkflowReadinessDeclared")
        })
        .map(|event| required_str(event, "event_id"))
        .collect::<Result<Vec<_>, _>>()?;
    ArtifactDigest::try_new(hex::encode(Sha256::digest(
        serde_json::to_vec(&event_ids).map_err(|error| error.to_string())?,
    )))
    .map_err(|error| error.to_string())
}

fn command_ordinal_for_stream(stream_id: &str) -> Result<usize, String> {
    let path = Path::new(EVENT_EXPORT_DIRECTORY);
    if !path.exists() {
        return Ok(0);
    }

    fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .map(|entry| {
            entry.map_err(|error| error.to_string()).and_then(|entry| {
                let contents =
                    fs::read_to_string(entry.path()).map_err(|error| error.to_string())?;
                serde_json::from_str::<Value>(&contents).map_err(|error| error.to_string())
            })
        })
        .try_fold(0_usize, |count, event| {
            let event = event?;
            let increments_count =
                event.get("stream_id").and_then(Value::as_str) == Some(stream_id);
            Ok(count + usize::from(increments_count))
        })
}

fn event_id(
    draft: &EventDraft,
    parents: &[String],
    command_ordinal: usize,
) -> Result<String, String> {
    let canonical = serde_json::to_vec(&json!({
        "schema_version": SCHEMA_VERSION,
        "command_ordinal": command_ordinal,
        "stream_id": draft.stream_id,
        "parents": parents,
        "type": draft.event_type,
        "payload": draft.payload,
    }))
    .map_err(|error| error.to_string())?;
    Ok(hex::encode(Sha256::digest(canonical)))
}

fn command_id(event_id: &str) -> String {
    event_id.to_owned()
}

#[derive(Debug)]
struct ProjectedModel {
    project_name: ProjectName,
    workflows: Vec<ProjectedWorkflow>,
    reviews: Vec<ProjectedReview>,
}

impl ProjectedModel {
    fn from_events(events: Vec<Value>) -> Result<Self, String> {
        events
            .into_iter()
            .try_fold(
                None::<Self>,
                |model, event| -> Result<Option<Self>, String> {
                    let event_type = required_str(&event, "type")?;
                    match event_type {
                        "ProjectInitialized" => {
                            let payload = required_object(&event, "payload")?;
                            let project_name = project_name(required_str(payload, "name")?)?;
                            let (workflows, reviews) = model
                                .map(|model| (model.workflows, model.reviews))
                                .unwrap_or_default();
                            Ok(Some(Self {
                                project_name,
                                workflows,
                                reviews,
                            }))
                        }
                        "WorkflowAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowAdded appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            model.workflows.push(ProjectedWorkflow {
                                slug: workflow_slug(required_str(payload, "slug")?)?,
                                name: model_name(required_str(payload, "name")?)?,
                                description: model_description(required_str(
                                    payload,
                                    "description",
                                )?)?,
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
                        "WorkflowUpdated" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowUpdated appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "slug")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowUpdated references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow.name = model_name(required_str(payload, "name")?)?;
                            workflow.description =
                                model_description(required_str(payload, "description")?)?;
                            Ok(Some(model))
                        }
                        "WorkflowRemoved" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowRemoved appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "slug")?;
                            let before = model.workflows.len();
                            model
                                .workflows
                                .retain(|workflow| workflow.slug.as_ref() != workflow_slug_text);
                            if model.workflows.len() == before {
                                return Err(format!(
                                    "WorkflowRemoved references unknown workflow {workflow_slug_text}"
                                ));
                            }
                            Ok(Some(model))
                        }
                        "SliceAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceAdded appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                        .workflows
                        .iter_mut()
                        .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                        .ok_or_else(|| {
                            format!("SliceAdded references unknown workflow {workflow_slug_text}")
                        })?;
                            let entry = workflow.slices.is_empty();
                            workflow.slices.push(ProjectedSlice {
                                slug: slice_slug(required_str(payload, "slug")?)?,
                                name: model_name(required_str(payload, "name")?)?,
                                kind: slice_kind_name(required_str(payload, "kind")?)?,
                                description: model_description(required_str(
                                    payload,
                                    "description",
                                )?)?,
                                relationship: workflow_step_relationship_name(if entry {
                                    "entry"
                                } else {
                                    "main"
                                })?,
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
                        "SliceUpdated" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceUpdated appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slug")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceUpdated references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice.name = model_name(required_str(payload, "name")?)?;
                            slice.kind = slice_kind_name(required_str(payload, "kind")?)?;
                            slice.description =
                                model_description(required_str(payload, "description")?)?;
                            Ok(Some(model))
                        }
                        "SliceRemoved" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceRemoved appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slug")?;
                            let removed_count = model
                                .workflows
                                .iter_mut()
                                .map(|workflow| {
                                    let before = workflow.slices.len();
                                    workflow
                                        .slices
                                        .retain(|slice| slice.slug.as_ref() != slice_slug_text);
                                    workflow.transitions.retain(|transition| {
                                        transition.source().as_ref() != slice_slug_text
                                            && transition.target().as_ref() != slice_slug_text
                                    });
                                    before - workflow.slices.len()
                                })
                                .sum::<usize>();
                            if removed_count == 0 {
                                return Err(format!(
                                    "SliceRemoved references unknown slice {slice_slug_text}"
                                ));
                            }
                            Ok(Some(model))
                        }
                        "SliceOutcomeAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceOutcomeAdded appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceOutcomeAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice.outcomes.push(slice_outcome_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceScenarioAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceScenarioAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceScenarioAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice.scenarios.push(slice_scenario_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceExternalPayloadAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceExternalPayloadAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceExternalPayloadAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .external_payloads
                                .push(slice_external_payload_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceEventDefinitionAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceEventDefinitionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceEventDefinitionAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .event_definitions
                                .push(slice_event_definition_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceCommandDefinitionAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceCommandDefinitionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceCommandDefinitionAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .command_definitions
                                .push(slice_command_definition_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceReadModelAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceReadModelAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceReadModelAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .read_models
                                .push(slice_read_model_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceBitLevelDataFlowAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceBitLevelDataFlowAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceBitLevelDataFlowAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .bit_level_data_flows
                                .push(slice_bit_level_data_flow_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceViewAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceViewAdded appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!("SliceViewAdded references unknown slice {slice_slug_text}")
                                })?;
                            slice.views.push(slice_view_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceTranslationAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceTranslationAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceTranslationAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .translations
                                .push(slice_translation_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceAutomationAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceAutomationAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceAutomationAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .automations
                                .push(slice_automation_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceBoardElementAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceBoardElementAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceBoardElementAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .board_elements
                                .push(slice_board_element_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "SliceBoardConnectionAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "SliceBoardConnectionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let slice_slug_text = required_str(payload, "slice")?;
                            let slice = model
                                .workflows
                                .iter_mut()
                                .flat_map(|workflow| workflow.slices.iter_mut())
                                .find(|slice| slice.slug.as_ref() == slice_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "SliceBoardConnectionAdded references unknown slice {slice_slug_text}"
                                    )
                                })?;
                            slice
                                .board_connections
                                .push(slice_board_connection_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "WorkflowReadinessDeclared" => {
                            let model = model.ok_or_else(|| {
                                "WorkflowReadinessDeclared appeared before project initialization"
                                    .to_owned()
                            })?;
                            Ok(Some(model))
                        }
                        "WorkflowConnected" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowConnected appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowConnected references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow.transitions.push(workflow_transition_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "WorkflowOutcomeAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowOutcomeAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowOutcomeAdded references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow.outcomes.push(workflow_outcome_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "WorkflowCommandErrorAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowCommandErrorAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowCommandErrorAdded references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow
                                .command_errors
                                .push(workflow_command_error_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "WorkflowOwnedDefinitionAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowOwnedDefinitionAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowOwnedDefinitionAdded references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow
                                .owned_definitions
                                .push(workflow_owned_definition_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "WorkflowTransitionEvidenceAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowTransitionEvidenceAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowTransitionEvidenceAdded references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow
                                .transition_evidences
                                .push(workflow_transition_evidence_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "WorkflowEntryLifecycleCoverageRequired" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowEntryLifecycleCoverageRequired appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowEntryLifecycleCoverageRequired references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow.requires_entry_lifecycle_coverage = true;
                            Ok(Some(model))
                        }
                        "WorkflowEntryLifecycleStateAdded" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowEntryLifecycleStateAdded appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowEntryLifecycleStateAdded references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            workflow
                                .entry_lifecycle_states
                                .push(workflow_entry_lifecycle_state_from_payload(payload)?);
                            Ok(Some(model))
                        }
                        "WorkflowTransitionRemoved" => {
                            let mut model = model.ok_or_else(|| {
                                "WorkflowTransitionRemoved appeared before project initialization"
                                    .to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            let workflow = model
                                .workflows
                                .iter_mut()
                                .find(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                                .ok_or_else(|| {
                                    format!(
                                        "WorkflowTransitionRemoved references unknown workflow {workflow_slug_text}"
                                    )
                                })?;
                            let removed_transition = workflow_transition_from_payload(payload)?;
                            workflow
                                .transitions
                                .retain(|transition| !same_transition(transition, &removed_transition));
                            Ok(Some(model))
                        }
                        "ReviewRecorded" => {
                            let mut model = model.ok_or_else(|| {
                                "ReviewRecorded appeared before project initialization".to_owned()
                            })?;
                            let payload = required_object(&event, "payload")?;
                            let workflow_slug_text = required_str(payload, "workflow")?;
                            if !model
                                .workflows
                                .iter()
                                .any(|workflow| workflow.slug.as_ref() == workflow_slug_text)
                            {
                                return Err(format!(
                                    "ReviewRecorded references unknown workflow {workflow_slug_text}"
                                ));
                            }
                            let review = ProjectedReview {
                                workflow_slug: workflow_slug(workflow_slug_text)?,
                                model_content_digest: required_str(payload, "model_content_digest")?
                                    .to_owned(),
                                reviewer_id: required_str(payload, "reviewer_id")?.to_owned(),
                                reviewed_at: required_str(payload, "reviewed_at")?.to_owned(),
                                categories: required_string_array(payload, "categories")?,
                            };
                            model.reviews.retain(|existing| {
                                existing.workflow_slug.as_ref() != review.workflow_slug.as_ref()
                            });
                            model.reviews.push(review);
                            Ok(Some(model))
                        }
                        _ => Ok(model),
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
            Effect::WriteFile(
                project_path(PROJECTION_FINGERPRINT_PATH)?,
                file_contents(format!("{projection_fingerprint}\n"))?,
            ),
            Effect::WriteFile(
                project_path("emc.toml")?,
                file_contents(format!(
                    "[project]\nname = \"{}\"\nversion = \"0.1.0\"\nlean_module = \"{project_module_name}\"\nquint_module = \"{project_module_name}\"\n",
                    self.project_name.as_ref()
                ))?,
            ),
            Effect::EnsureDirectory(project_path("model/lean")?),
            Effect::WriteFile(
                project_path("model/lean/lean-toolchain")?,
                file_contents("leanprover/lean4:4.29.1\n")?,
            ),
            Effect::WriteFile(
                project_path("model/lean/lakefile.lean")?,
                file_contents("import Lake\nopen Lake DSL\npackage EMCModel where\n")?,
            ),
            Effect::EnsureDirectory(project_path("model/lean/slices")?),
            Effect::WriteFile(
                project_path("model/lean/slices/.gitkeep")?,
                file_contents("\n")?,
            ),
            Effect::EnsureDirectory(project_path("model/quint")?),
            Effect::EnsureDirectory(project_path("model/quint/slices")?),
            Effect::WriteFile(
                project_path("model/quint/slices/.gitkeep")?,
                file_contents("\n")?,
            ),
            Effect::EnsureDirectory(project_path("reviews")?),
            Effect::WriteFile(project_path("reviews/.gitkeep")?, file_contents("\n")?),
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
    model_content_digest: String,
    reviewer_id: String,
    reviewed_at: String,
    categories: Vec<String>,
}

#[derive(Debug)]
struct WorkflowReadinessDeclaration {
    workflow: WorkflowSlug,
    projection_fingerprint: ArtifactDigest,
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
            "model_content_digest": self.model_content_digest,
            "reviewer_id": self.reviewer_id,
            "status": "clean",
            "category_results": category_results,
            "mandatory_findings": [],
            "reviewed_at": self.reviewed_at
        });
        let contents =
            serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?;
        Ok(Effect::WriteFile(
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
            Effect::WriteFile(
                project_path(format!("model/lean/{module_name}.lean"))?,
                emit_lean_workflow_module(
                    lean_module_name(module_name.clone()),
                    workflow_data.clone(),
                ),
            ),
            Effect::WriteFile(
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
            self.kind.clone(),
            self.description.clone(),
            self.relationship.clone(),
        )
    }

    fn effects(self) -> Result<Vec<Effect>, String> {
        let module_name = module_name(self.name.as_ref());
        let digest = slice_artifact_digest(
            self.name.clone(),
            self.slug.clone(),
            self.kind.clone(),
            self.description.clone(),
        );
        let mut effects = vec![
            Effect::WriteFile(
                project_path(format!("model/lean/slices/{module_name}.lean"))?,
                emit_lean_slice_module(
                    lean_module_name(module_name.clone()),
                    self.name.clone(),
                    self.description.clone(),
                    self.slug.clone(),
                    self.kind.clone(),
                    digest.clone(),
                ),
            ),
            Effect::WriteFile(
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
        .map(event_attribute_source_kind)
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
    let mut input = NewCommandInput::new(
        datum_name(required_str(input_payload, "name")?)?,
        command_input_source_kind(required_str(input_payload, "source_kind")?)?,
        command_input_source_description(required_str(input_payload, "source_description")?)?,
        CommandInputProvenanceChain::from_hops(
            required_string_array(input_payload, "provenance_chain")?
                .into_iter()
                .map(|hop| source_chain_hop(&hop))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    if let Some((event, attribute)) = optional_event_stream_source(input_payload)? {
        input = input.with_event_stream_source(event, attribute);
    }
    if let Some((source, field)) = optional_source_field_pair(
        input_payload,
        "external_payload_source_name",
        "external_payload_source_field",
    )? {
        input = input.with_external_payload_source(source, field);
    }
    if let Some((source, field)) = optional_source_field_pair(
        input_payload,
        "generated_source_name",
        "generated_source_field",
    )? {
        input = input.with_generated_source(source, field);
    }
    if let Some((source, field)) =
        optional_source_field_pair(input_payload, "session_source_name", "session_source_field")?
    {
        input = input.with_session_source(source, field);
    }
    if let Some((source, field)) = optional_source_field_pair(
        input_payload,
        "invocation_argument_source_name",
        "invocation_argument_source_field",
    )? {
        input = input.with_invocation_argument_source(source, field);
    }

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
    match (
        optional_str(payload, "source_event")?,
        optional_str(payload, "source_attribute")?,
        optional_str(payload, "derivation_rule")?,
        required_string_array(payload, "derivation_source_fields")?,
        optional_str(payload, "absence_event")?,
        optional_str(payload, "derivation_scenario")?,
        optional_str(payload, "absence_scenario")?,
    ) {
        (
            Some(source_event),
            Some(source_attribute),
            None,
            derivation_source_fields,
            None,
            None,
            None,
        ) if derivation_source_fields.is_empty() => Ok(NewReadModelField::new(
            name,
            source_kind,
            event_name(source_event)?,
            event_attribute_name(source_attribute)?,
            provenance,
        )),
        (
            None,
            None,
            Some(derivation_rule),
            derivation_source_fields,
            None,
            Some(derivation_scenario),
            None,
        ) => Ok(NewReadModelField::new_derivation(
            name,
            source_kind,
            read_model_derivation_rule(derivation_rule)?,
            ReadModelDerivationSourceFields::from_fields(
                derivation_source_fields
                    .into_iter()
                    .map(|field| datum_name(&field))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            scenario_name(derivation_scenario)?,
            provenance,
        )),
        (
            None,
            None,
            None,
            derivation_source_fields,
            Some(absence_event),
            None,
            Some(absence_scenario),
        ) if derivation_source_fields.is_empty() => Ok(NewReadModelField::new_absence_default(
            name,
            source_kind,
            event_name(absence_event)?,
            scenario_name(absence_scenario)?,
            provenance,
        )),
        _ => Err("SliceReadModelAdded has incompatible field source fields".to_owned()),
    }
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
        workflow_transition_evidence_text(required_str(payload, "source_evidence")?)?,
        workflow_transition_evidence_text(required_str(payload, "target_evidence")?)?,
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

fn workflow_transition_from_payload(payload: &Value) -> Result<WorkflowTransitionRecord, String> {
    let source = workflow_transition_endpoint(required_str(payload, "from")?)?;
    let target_workflow = optional_str(payload, "to_workflow")?;
    let target = workflow_transition_endpoint(
        optional_str(payload, "to")?
            .or(target_workflow)
            .ok_or_else(|| "exported workflow transition is missing target".to_owned())?,
    )?;
    let raw_kind = required_str(payload, "via")?;
    let kind_text = target_workflow
        .map(|_| format!("workflow_exit:{raw_kind}"))
        .unwrap_or_else(|| raw_kind.to_owned());
    let kind = workflow_transition_kind(&kind_text)?;
    let trigger = transition_trigger_name(required_str(payload, "name")?)?;
    let rationale = optional_str(payload, "reason")?
        .map(model_description)
        .transpose()?;
    let payload_contract = optional_str(payload, "payload_contract")?
        .map(payload_contract_name)
        .transpose()?;

    match (rationale, payload_contract) {
        (Some(rationale), None) => Ok(WorkflowTransitionRecord::new_with_rationale(
            source, target, kind, trigger, rationale,
        )),
        (None, Some(payload_contract)) => Ok(WorkflowTransitionRecord::new_with_payload_contract(
            source,
            target,
            kind,
            trigger,
            payload_contract,
        )),
        (None, None) => Ok(WorkflowTransitionRecord::new(source, target, kind, trigger)),
        (Some(_), Some(_)) => {
            Err("WorkflowConnected cannot project both rationale and payload contract".to_owned())
        }
    }
}

fn workflow_readiness_declaration_from_payload(
    payload: &Value,
) -> Result<WorkflowReadinessDeclaration, String> {
    Ok(WorkflowReadinessDeclaration {
        workflow: workflow_slug(required_str(payload, "workflow")?)?,
        projection_fingerprint: artifact_digest_from_str(required_str(
            payload,
            "projection_fingerprint",
        )?)?,
    })
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

fn slice_slug(value: &str) -> Result<SliceSlug, String> {
    SliceSlug::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn slice_kind_name(value: &str) -> Result<SliceKindName, String> {
    SliceKindName::try_new(value.to_owned()).map_err(|error| error.to_string())
}

fn workflow_step_relationship_name(value: &str) -> Result<WorkflowStepRelationshipName, String> {
    WorkflowStepRelationshipName::try_new(value.to_owned()).map_err(|error| error.to_string())
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
    match value {
        "acceptance" => Ok(ScenarioKind::acceptance()),
        "contract" => Ok(ScenarioKind::contract()),
        _ => Err(format!("unknown scenario kind {value}")),
    }
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

fn workflow_transition_evidence_text(
    value: &str,
) -> Result<WorkflowTransitionEvidenceText, String> {
    WorkflowTransitionEvidenceText::try_new(value.to_owned()).map_err(|error| error.to_string())
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
