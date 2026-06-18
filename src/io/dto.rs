// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::connection::ConnectionKind;
use crate::core::formal_slice_facts::ScenarioKind;
use crate::core::gherkin::GherkinSuite;
use crate::core::modeling_enums::{WORKFLOW_ENTRY_LIFECYCLE_STATES, accepted_values};
use crate::core::project::ProjectName;
use crate::core::slice::SliceKind;
use crate::core::types::{
    AutomationName, AutomationReactionDescription, AutomationTriggerName, BitEncodingSemantics,
    BoardConnectionEndpoint, BoardConnectionEndpointKind, BoardElementDeclaredName,
    BoardElementKind, BoardElementName, BoardLaneId, CommandErrorName, CommandErrorRecoveryKind,
    CommandInputSourceDescription, CommandInputSourceKind, CommandName, ContractKindName,
    ControlName, ControlRecoveryBehavior, CoveredDefinitionName, DataFlowSource,
    DataFlowSourceKind, DataFlowTarget, DatumName, EventAttributeName, EventAttributeSourceField,
    EventAttributeSourceKind, EventAttributeSourceName, EventName,
    GeneratedEventAttributeSourceKind, ModelDescription, ModelName, NavigationTargetName,
    NavigationTargetType, OutcomeLabelName, PayloadContractName, ProvenanceDescription,
    ReadModelDerivationRule, ReadModelFieldSourceKind, ReadModelName, ReadModelTransitiveRule,
    ReviewTimestamp, ReviewerId, ScenarioName, ScenarioStepText, SingletonRepeatBehavior,
    SketchToken, SliceSlug, SourceChainHop, StreamName, TransformationSemantics,
    TransitionTriggerName, TranslationExternalEventName, TranslationName, ViewFieldName,
    ViewFieldSourceKind, ViewName, WorkflowEntryLifecycleEvidenceText,
    WorkflowEntryLifecycleStateName, WorkflowEventParticipation, WorkflowOwnedDefinitionKind,
    WorkflowOwnedDefinitionName, WorkflowSlug, WorkflowTransitionEndpoint, WorkflowTransitionKind,
    WorkflowTransitionSourceEvidenceText, WorkflowTransitionTargetEvidenceText, WorkflowViewRole,
};

#[cfg(test)]
use crate::core::types::{LeanModuleName, ModelDigest, QuintModuleName};

#[derive(Debug)]
pub(crate) struct BoundaryParseError {
    message: String,
}

impl BoundaryParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for BoundaryParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BoundaryParseError {}

pub(crate) fn parse_model_name(raw: &str) -> Result<ModelName, BoundaryParseError> {
    ModelName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid model name: {error}")))
}

pub(crate) fn parse_model_description(raw: &str) -> Result<ModelDescription, BoundaryParseError> {
    ModelDescription::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid model description: {error}")))
}

pub(crate) fn parse_gherkin_suite(raw: &str) -> Result<GherkinSuite, BoundaryParseError> {
    match raw {
        "meta" => Ok(GherkinSuite::Meta),
        "review-gate" => Ok(GherkinSuite::ReviewGate),
        _ => Err(BoundaryParseError::new(format!(
            "invalid Gherkin suite '{raw}'"
        ))),
    }
}

pub(crate) fn parse_project_name(raw: &str) -> Result<ProjectName, BoundaryParseError> {
    ProjectName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid project name: {error}")))
}

pub(crate) fn parse_project_manifest_name(raw: &str) -> Result<ProjectName, BoundaryParseError> {
    raw.lines()
        .find_map(|line| line.trim().strip_prefix("name = "))
        .and_then(quoted_value)
        .ok_or_else(|| BoundaryParseError::new("emc.toml is missing project name"))
        .and_then(parse_project_name)
}

pub(crate) fn parse_workflow_slug(raw: &str) -> Result<WorkflowSlug, BoundaryParseError> {
    WorkflowSlug::try_new(slugify(raw))
        .map_err(|error| BoundaryParseError::new(format!("invalid workflow slug: {error}")))
}

pub(crate) fn parse_slice_slug(raw: &str) -> Result<SliceSlug, BoundaryParseError> {
    SliceSlug::try_new(slugify(raw))
        .map_err(|error| BoundaryParseError::new(format!("invalid slice slug: {error}")))
}

pub(crate) fn parse_slice_kind(raw: &str) -> Result<SliceKind, BoundaryParseError> {
    match raw.trim() {
        "state_view" => Ok(SliceKind::state_view()),
        "state_change" => Ok(SliceKind::state_change()),
        "translation" => Ok(SliceKind::translation()),
        "automation" => Ok(SliceKind::automation()),
        _ => Err(BoundaryParseError::new(format!(
            "invalid slice type: {raw}"
        ))),
    }
}

pub(crate) fn parse_connection_kind(raw: &str) -> Result<ConnectionKind, BoundaryParseError> {
    match raw.trim() {
        "command" => Ok(ConnectionKind::command()),
        "event" => Ok(ConnectionKind::event()),
        "navigation" => Ok(ConnectionKind::navigation()),
        "external_trigger" => Ok(ConnectionKind::external_trigger()),
        "outcome" => Ok(ConnectionKind::outcome()),
        _ => Err(BoundaryParseError::new(format!(
            "invalid connection kind: {raw}"
        ))),
    }
}

pub(crate) fn parse_transition_trigger_name(
    raw: &str,
) -> Result<TransitionTriggerName, BoundaryParseError> {
    TransitionTriggerName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid transition trigger name: {error}"))
    })
}

pub(crate) fn parse_payload_contract_name(
    raw: &str,
) -> Result<PayloadContractName, BoundaryParseError> {
    PayloadContractName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid payload contract name: {error}")))
}

pub(crate) fn parse_translation_name(raw: &str) -> Result<TranslationName, BoundaryParseError> {
    TranslationName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid translation name: {error}")))
}

pub(crate) fn parse_translation_external_event_name(
    raw: &str,
) -> Result<TranslationExternalEventName, BoundaryParseError> {
    TranslationExternalEventName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid translation external event name: {error}"))
    })
}

pub(crate) fn parse_outcome_label_name(raw: &str) -> Result<OutcomeLabelName, BoundaryParseError> {
    OutcomeLabelName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid outcome label: {error}")))
}

pub(crate) fn parse_scenario_name(raw: &str) -> Result<ScenarioName, BoundaryParseError> {
    ScenarioName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid scenario name: {error}")))
}

pub(crate) fn parse_scenario_step_text(raw: &str) -> Result<ScenarioStepText, BoundaryParseError> {
    ScenarioStepText::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid scenario step text: {error}")))
}

pub(crate) fn parse_scenario_kind(raw: &str) -> Result<ScenarioKind, BoundaryParseError> {
    ScenarioKind::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid scenario kind: {error}")))
}

pub(crate) fn parse_contract_kind_name(raw: &str) -> Result<ContractKindName, BoundaryParseError> {
    ContractKindName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid contract kind: {error}")))
}

pub(crate) fn parse_covered_definition_name(
    raw: &str,
) -> Result<CoveredDefinitionName, BoundaryParseError> {
    CoveredDefinitionName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid covered definition: {error}")))
}

pub(crate) fn parse_datum_name(raw: &str) -> Result<DatumName, BoundaryParseError> {
    DatumName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid datum name: {error}")))
}

pub(crate) fn parse_datum_names(raw: &str) -> Result<Vec<DatumName>, BoundaryParseError> {
    parse_comma_separated(raw, "datum names")?
        .into_iter()
        .map(|name| {
            DatumName::try_new(name)
                .map_err(|error| BoundaryParseError::new(format!("invalid datum name: {error}")))
        })
        .collect()
}

pub(crate) fn parse_data_flow_source(raw: &str) -> Result<DataFlowSource, BoundaryParseError> {
    DataFlowSource::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid data-flow source: {error}")))
}

pub(crate) fn parse_data_flow_source_kind(
    raw: &str,
) -> Result<DataFlowSourceKind, BoundaryParseError> {
    DataFlowSourceKind::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid data-flow source kind: {error}")))
}

pub(crate) fn parse_transformation_semantics(
    raw: &str,
) -> Result<TransformationSemantics, BoundaryParseError> {
    TransformationSemantics::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid transformation semantics: {error}"))
    })
}

pub(crate) fn parse_data_flow_target(raw: &str) -> Result<DataFlowTarget, BoundaryParseError> {
    DataFlowTarget::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid data-flow target: {error}")))
}

pub(crate) fn parse_bit_encoding_semantics(
    raw: &str,
) -> Result<BitEncodingSemantics, BoundaryParseError> {
    BitEncodingSemantics::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid bit encoding: {error}")))
}

pub(crate) fn parse_board_element_name(raw: &str) -> Result<BoardElementName, BoundaryParseError> {
    BoardElementName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid board element name: {error}")))
}

pub(crate) fn parse_board_element_kind(raw: &str) -> Result<BoardElementKind, BoundaryParseError> {
    BoardElementKind::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid board element kind: {error}")))
}

pub(crate) fn parse_board_lane_id(raw: &str) -> Result<BoardLaneId, BoundaryParseError> {
    BoardLaneId::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid board lane id: {error}")))
}

pub(crate) fn parse_board_element_declared_name(
    raw: &str,
) -> Result<BoardElementDeclaredName, BoundaryParseError> {
    BoardElementDeclaredName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid board element declared name: {error}"))
    })
}

pub(crate) fn parse_board_connection_endpoint(
    raw: &str,
) -> Result<BoardConnectionEndpoint, BoundaryParseError> {
    BoardConnectionEndpoint::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid board connection endpoint: {error}"))
    })
}

pub(crate) fn parse_board_connection_endpoint_kind(
    raw: &str,
) -> Result<BoardConnectionEndpointKind, BoundaryParseError> {
    BoardConnectionEndpointKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid board connection endpoint kind: {error}"))
    })
}

pub(crate) fn parse_command_name(raw: &str) -> Result<CommandName, BoundaryParseError> {
    CommandName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid command name: {error}")))
}

pub(crate) fn parse_command_error_name(raw: &str) -> Result<CommandErrorName, BoundaryParseError> {
    CommandErrorName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid command error: {error}")))
}

pub(crate) fn parse_command_error_names(
    raw: &str,
) -> Result<Vec<CommandErrorName>, BoundaryParseError> {
    parse_comma_separated(raw, "command error names")?
        .into_iter()
        .map(|name| {
            CommandErrorName::try_new(name)
                .map_err(|error| BoundaryParseError::new(format!("invalid command error: {error}")))
        })
        .collect()
}

pub(crate) fn parse_command_error_recovery_kind(
    raw: &str,
) -> Result<CommandErrorRecoveryKind, BoundaryParseError> {
    CommandErrorRecoveryKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid command error recovery kind: {error}"))
    })
}

pub(crate) fn parse_singleton_repeat_behavior(
    raw: &str,
) -> Result<SingletonRepeatBehavior, BoundaryParseError> {
    SingletonRepeatBehavior::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid singleton repeat behavior: {error}"))
    })
}

pub(crate) fn parse_automation_name(raw: &str) -> Result<AutomationName, BoundaryParseError> {
    AutomationName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid automation name: {error}")))
}

pub(crate) fn parse_automation_trigger_name(
    raw: &str,
) -> Result<AutomationTriggerName, BoundaryParseError> {
    AutomationTriggerName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid automation trigger name: {error}"))
    })
}

pub(crate) fn parse_automation_reaction_description(
    raw: &str,
) -> Result<AutomationReactionDescription, BoundaryParseError> {
    AutomationReactionDescription::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid automation reaction description: {error}"))
    })
}

pub(crate) fn parse_control_name(raw: &str) -> Result<ControlName, BoundaryParseError> {
    ControlName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid control name: {error}")))
}

pub(crate) fn parse_control_recovery_behavior(
    raw: &str,
) -> Result<ControlRecoveryBehavior, BoundaryParseError> {
    ControlRecoveryBehavior::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid control recovery behavior: {error}"))
    })
}

pub(crate) fn parse_navigation_target_type(
    raw: &str,
) -> Result<NavigationTargetType, BoundaryParseError> {
    NavigationTargetType::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid navigation target type: {error}"))
    })
}

pub(crate) fn parse_navigation_target_name(
    raw: &str,
) -> Result<NavigationTargetName, BoundaryParseError> {
    NavigationTargetName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid navigation target name: {error}"))
    })
}

pub(crate) fn parse_navigation_target_names(
    raw: &str,
) -> Result<Vec<NavigationTargetName>, BoundaryParseError> {
    parse_comma_separated(raw, "navigation target names")?
        .into_iter()
        .map(|name| {
            NavigationTargetName::try_new(name).map_err(|error| {
                BoundaryParseError::new(format!("invalid navigation target name: {error}"))
            })
        })
        .collect()
}

pub(crate) fn parse_command_input_source_kind(
    raw: &str,
) -> Result<CommandInputSourceKind, BoundaryParseError> {
    CommandInputSourceKind::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid command input source: {error}")))
}

pub(crate) fn parse_command_input_source_description(
    raw: &str,
) -> Result<CommandInputSourceDescription, BoundaryParseError> {
    CommandInputSourceDescription::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid command input source description: {error}"))
    })
}

pub(crate) fn parse_event_names(raw: &str) -> Result<Vec<EventName>, BoundaryParseError> {
    parse_comma_separated(raw, "event names")?
        .into_iter()
        .map(|name| {
            EventName::try_new(name)
                .map_err(|error| BoundaryParseError::new(format!("invalid event name: {error}")))
        })
        .collect()
}

pub(crate) fn parse_event_name(raw: &str) -> Result<EventName, BoundaryParseError> {
    EventName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid event name: {error}")))
}

pub(crate) fn parse_stream_name(raw: &str) -> Result<StreamName, BoundaryParseError> {
    StreamName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid stream name: {error}")))
}

pub(crate) fn parse_stream_names(raw: &str) -> Result<Vec<StreamName>, BoundaryParseError> {
    parse_comma_separated(raw, "stream names")?
        .into_iter()
        .map(|name| {
            StreamName::try_new(name)
                .map_err(|error| BoundaryParseError::new(format!("invalid stream name: {error}")))
        })
        .collect()
}

pub(crate) fn parse_event_attribute_name(
    raw: &str,
) -> Result<EventAttributeName, BoundaryParseError> {
    EventAttributeName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid event attribute: {error}")))
}

pub(crate) fn parse_event_attribute_source_kind(
    raw: &str,
) -> Result<EventAttributeSourceKind, BoundaryParseError> {
    EventAttributeSourceKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid event attribute source kind: {error}"))
    })
}

pub(crate) fn parse_generated_event_attribute_source_kind(
    raw: &str,
) -> Result<GeneratedEventAttributeSourceKind, BoundaryParseError> {
    GeneratedEventAttributeSourceKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!(
            "invalid generated event attribute source kind: {error}"
        ))
    })
}

pub(crate) fn parse_event_attribute_source_name(
    raw: &str,
) -> Result<EventAttributeSourceName, BoundaryParseError> {
    EventAttributeSourceName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid event attribute source name: {error}"))
    })
}

pub(crate) fn parse_event_attribute_source_field(
    raw: &str,
) -> Result<EventAttributeSourceField, BoundaryParseError> {
    EventAttributeSourceField::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid event attribute source field: {error}"))
    })
}

pub(crate) fn parse_provenance_description(
    raw: &str,
) -> Result<ProvenanceDescription, BoundaryParseError> {
    ProvenanceDescription::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid provenance: {error}")))
}

pub(crate) fn parse_read_model_name(raw: &str) -> Result<ReadModelName, BoundaryParseError> {
    ReadModelName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid read model name: {error}")))
}

pub(crate) fn parse_read_model_field_source_kind(
    raw: &str,
) -> Result<ReadModelFieldSourceKind, BoundaryParseError> {
    ReadModelFieldSourceKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid read model field source kind: {error}"))
    })
}

pub(crate) fn parse_read_model_derivation_rule(
    raw: &str,
) -> Result<ReadModelDerivationRule, BoundaryParseError> {
    ReadModelDerivationRule::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid read model derivation rule: {error}"))
    })
}

pub(crate) fn parse_read_model_transitive_rule(
    raw: &str,
) -> Result<ReadModelTransitiveRule, BoundaryParseError> {
    ReadModelTransitiveRule::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid read model transitive rule: {error}"))
    })
}

pub(crate) fn parse_view_name(raw: &str) -> Result<ViewName, BoundaryParseError> {
    ViewName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid view name: {error}")))
}

pub(crate) fn parse_view_field_name(raw: &str) -> Result<ViewFieldName, BoundaryParseError> {
    ViewFieldName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid view field name: {error}")))
}

pub(crate) fn parse_view_field_source_kind(
    raw: &str,
) -> Result<ViewFieldSourceKind, BoundaryParseError> {
    ViewFieldSourceKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid view field source kind: {error}"))
    })
}

pub(crate) fn parse_sketch_token(raw: &str) -> Result<SketchToken, BoundaryParseError> {
    SketchToken::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid sketch token: {error}")))
}

pub(crate) fn parse_source_chain_hops(
    raw: &str,
) -> Result<Vec<SourceChainHop>, BoundaryParseError> {
    parse_comma_separated(raw, "source chain hops")?
        .into_iter()
        .map(|hop| {
            SourceChainHop::try_new(hop).map_err(|error| {
                BoundaryParseError::new(format!("invalid source chain hop: {error}"))
            })
        })
        .collect()
}

pub(crate) fn parse_workflow_transition_endpoint(
    raw: &str,
) -> Result<WorkflowTransitionEndpoint, BoundaryParseError> {
    WorkflowTransitionEndpoint::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid workflow transition endpoint: {error}"))
    })
}

pub(crate) fn parse_workflow_transition_kind(
    raw: &str,
) -> Result<WorkflowTransitionKind, BoundaryParseError> {
    WorkflowTransitionKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid workflow transition kind: {error}"))
    })
}

pub(crate) fn parse_workflow_transition_source_evidence_text(
    raw: &str,
) -> Result<WorkflowTransitionSourceEvidenceText, BoundaryParseError> {
    WorkflowTransitionSourceEvidenceText::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!(
            "invalid workflow transition source evidence: {error}"
        ))
    })
}

pub(crate) fn parse_workflow_transition_target_evidence_text(
    raw: &str,
) -> Result<WorkflowTransitionTargetEvidenceText, BoundaryParseError> {
    WorkflowTransitionTargetEvidenceText::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!(
            "invalid workflow transition target evidence: {error}"
        ))
    })
}

pub(crate) fn parse_workflow_entry_lifecycle_state_name(
    raw: &str,
) -> Result<WorkflowEntryLifecycleStateName, BoundaryParseError> {
    WorkflowEntryLifecycleStateName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!(
            "invalid workflow entry lifecycle state name '{raw}': {error}; expected one of: {}",
            accepted_values(WORKFLOW_ENTRY_LIFECYCLE_STATES)
        ))
    })
}

pub(crate) fn parse_workflow_entry_lifecycle_evidence_text(
    raw: &str,
) -> Result<WorkflowEntryLifecycleEvidenceText, BoundaryParseError> {
    WorkflowEntryLifecycleEvidenceText::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!(
            "invalid workflow entry lifecycle evidence text: {error}"
        ))
    })
}

pub(crate) fn parse_workflow_owned_definition_kind(
    raw: &str,
) -> Result<WorkflowOwnedDefinitionKind, BoundaryParseError> {
    WorkflowOwnedDefinitionKind::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid workflow owned definition kind: {error}"))
    })
}

pub(crate) fn parse_workflow_owned_definition_name(
    raw: &str,
) -> Result<WorkflowOwnedDefinitionName, BoundaryParseError> {
    WorkflowOwnedDefinitionName::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid workflow owned definition name: {error}"))
    })
}

pub(crate) fn parse_workflow_event_participation(
    raw: &str,
) -> Result<WorkflowEventParticipation, BoundaryParseError> {
    WorkflowEventParticipation::try_new(raw).map_err(|error| {
        BoundaryParseError::new(format!("invalid workflow event participation: {error}"))
    })
}

pub(crate) fn parse_workflow_view_role(raw: &str) -> Result<WorkflowViewRole, BoundaryParseError> {
    WorkflowViewRole::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid workflow view role: {error}")))
}

#[cfg(test)]
pub(crate) fn parse_lean_module_name(raw: &str) -> Result<LeanModuleName, BoundaryParseError> {
    LeanModuleName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid Lean module name: {error}")))
}

#[cfg(test)]
pub(crate) fn parse_quint_module_name(raw: &str) -> Result<QuintModuleName, BoundaryParseError> {
    QuintModuleName::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid Quint module name: {error}")))
}

#[cfg(test)]
pub(crate) fn parse_model_digest(raw: &str) -> Result<ModelDigest, BoundaryParseError> {
    ModelDigest::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid model digest: {error}")))
}

pub(crate) fn parse_reviewer_id(raw: &str) -> Result<ReviewerId, BoundaryParseError> {
    ReviewerId::try_new(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid reviewer id: {error}")))
}

pub(crate) fn parse_review_timestamp(raw: &str) -> Result<ReviewTimestamp, BoundaryParseError> {
    ReviewTimestamp::try_new(raw).map_err(|_error| {
        BoundaryParseError::new(
            "invalid review timestamp: expected UTC millisecond timestamp like 2026-06-03T12:00:00.000Z",
        )
    })
}

fn slugify(raw: &str) -> String {
    raw.trim()
        .chars()
        .fold(
            (String::new(), false),
            |(mut slug, pending_dash), character| {
                if character.is_ascii_alphanumeric() {
                    if pending_dash && !slug.is_empty() {
                        slug.push('-');
                    }
                    slug.push(character.to_ascii_lowercase());
                    (slug, false)
                } else {
                    (slug, true)
                }
            },
        )
        .0
}

fn quoted_value(raw: &str) -> Option<&str> {
    raw.strip_prefix('"')?.strip_suffix('"')
}

fn parse_comma_separated(raw: &str, label: &str) -> Result<Vec<String>, BoundaryParseError> {
    let values = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if values.is_empty() {
        Err(BoundaryParseError::new(format!(
            "invalid {label}: expected at least one value"
        )))
    } else {
        Ok(values)
    }
}
