use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::effect::FileContents;
use crate::core::types::{
    CommandErrorName, CommandName, ModelDescription, ModelName, OutcomeLabelName,
    PayloadContractName, SliceKindName, SliceSlug, StreamName, TransitionTriggerName,
    WorkflowCommandErrorRecord, WorkflowCommandErrorRecords, WorkflowEntryLifecycleEvidenceText,
    WorkflowEntryLifecycleStateName, WorkflowEntryLifecycleStateRecord,
    WorkflowEntryLifecycleStateRecords, WorkflowOutcomeRecord, WorkflowOutcomeRecords,
    WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord,
    WorkflowOwnedDefinitionRecords, WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug,
    WorkflowStepRelationshipName, WorkflowTransitionEndpoint, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionEvidenceText, WorkflowTransitionKind,
    WorkflowTransitionRecord, WorkflowTransitionRecords,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormalWorkflowGraph {
    name: ModelName,
    slug: WorkflowSlug,
    description: ModelDescription,
    slice_details: WorkflowSliceDetails,
    transitions: WorkflowTransitionRecords,
    outcomes: WorkflowOutcomeRecords,
    command_errors: WorkflowCommandErrorRecords,
    owned_definitions: WorkflowOwnedDefinitionRecords,
    transition_evidences: WorkflowTransitionEvidenceRecords,
    entry_lifecycle_required: bool,
    entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
}

impl FormalWorkflowGraph {
    pub fn name(&self) -> &ModelName {
        &self.name
    }

    pub fn slug(&self) -> &WorkflowSlug {
        &self.slug
    }

    pub fn description(&self) -> &ModelDescription {
        &self.description
    }

    pub fn slice_details(&self) -> &WorkflowSliceDetails {
        &self.slice_details
    }

    pub fn transitions(&self) -> &WorkflowTransitionRecords {
        &self.transitions
    }

    pub fn outcomes(&self) -> &WorkflowOutcomeRecords {
        &self.outcomes
    }

    pub fn command_errors(&self) -> &WorkflowCommandErrorRecords {
        &self.command_errors
    }

    pub fn owned_definitions(&self) -> &WorkflowOwnedDefinitionRecords {
        &self.owned_definitions
    }

    pub fn transition_evidences(&self) -> &WorkflowTransitionEvidenceRecords {
        &self.transition_evidences
    }

    pub fn entry_lifecycle_required(&self) -> bool {
        self.entry_lifecycle_required
    }

    pub fn entry_lifecycle_states(&self) -> &WorkflowEntryLifecycleStateRecords {
        &self.entry_lifecycle_states
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormalWorkflowGraphs {
    graphs: Vec<FormalWorkflowGraph>,
}

impl FormalWorkflowGraphs {
    pub fn from_graphs(graphs: impl IntoIterator<Item = FormalWorkflowGraph>) -> Self {
        Self {
            graphs: graphs.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[FormalWorkflowGraph] {
        &self.graphs
    }

    pub(crate) fn into_inner(self) -> Vec<FormalWorkflowGraph> {
        self.graphs
    }
}

pub fn parse_lean_workflow_graph(
    artifact: &FileContents,
) -> Result<FormalWorkflowGraph, FormalGraphError> {
    parse_workflow_graph(
        artifact.as_ref(),
        WorkflowGraphPrefixes {
            name: "def workflowName := ",
            slug: "def workflowSlug := ",
            description: "def workflowDescription := ",
            slice_details: "def workflowSliceDetails : List (String × String × String × String) := ",
            step_relationships: "def workflowStepRelationships : List (String × String) := ",
            transitions: "def workflowTransitions : List WorkflowTransition := ",
            outcomes: "def workflowOutcomes : List WorkflowOutcome := ",
            command_errors: "def workflowCommandErrors : List WorkflowCommandError := ",
            owned_definitions: "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := ",
            transition_evidences: "def workflowTransitionEvidences : List WorkflowTransitionEvidence := ",
            entry_lifecycle_required: "def workflowRequiresEntryLifecycleCoverage : Bool := ",
            entry_lifecycle_states: "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := ",
        },
    )
}

pub fn parse_quint_workflow_graph(
    artifact: &FileContents,
) -> Result<FormalWorkflowGraph, FormalGraphError> {
    let artifact = artifact.as_ref();
    Ok(FormalWorkflowGraph {
        name: model_name(json_line_value(artifact, "val workflowName = ")?)?,
        slug: workflow_slug(json_line_value(artifact, "val workflowSlug = ")?)?,
        description: model_description(json_line_value(artifact, "val workflowDescription = ")?)?,
        slice_details: WorkflowSliceDetails::from_details(slice_details_with_relationships(
            parse_slice_details(quint_val_value(artifact, "workflowSliceDetails")?)?,
            quint_val_value_optional(artifact, "workflowStepRelationships")?
                .map(parse_step_relationships)
                .transpose()?
                .unwrap_or_default(),
        )),
        transitions: WorkflowTransitionRecords::from_records(parse_transitions(quint_val_value(
            artifact,
            "workflowTransitions",
        )?)?),
        outcomes: WorkflowOutcomeRecords::from_records(parse_workflow_outcomes(quint_val_value(
            artifact,
            "workflowOutcomes",
        )?)?),
        command_errors: WorkflowCommandErrorRecords::from_records(parse_workflow_command_errors(
            quint_val_value(artifact, "workflowCommandErrors")?,
        )?),
        owned_definitions: parse_optional_workflow_owned_definitions(quint_val_value_optional(
            artifact,
            "workflowOwnedDefinitions",
        )?)?,
        transition_evidences: parse_optional_workflow_transition_evidences(
            quint_val_value_optional(artifact, "workflowTransitionEvidences")?,
        )?,
        entry_lifecycle_required: quint_val_value_optional(
            artifact,
            "workflowRequiresEntryLifecycleCoverage",
        )?
        .map(parse_bool_value)
        .transpose()?
        .unwrap_or(false),
        entry_lifecycle_states: parse_optional_workflow_entry_lifecycle_states(
            quint_val_value_optional(artifact, "workflowEntryLifecycleStates")?,
        )?,
    })
}

fn parse_optional_workflow_owned_definitions(
    value: Option<&str>,
) -> Result<WorkflowOwnedDefinitionRecords, FormalGraphError> {
    value
        .map(parse_workflow_owned_definitions)
        .transpose()
        .map(|records| WorkflowOwnedDefinitionRecords::from_records(records.unwrap_or_default()))
}

fn parse_optional_workflow_transition_evidences(
    value: Option<&str>,
) -> Result<WorkflowTransitionEvidenceRecords, FormalGraphError> {
    value
        .map(parse_workflow_transition_evidences)
        .transpose()
        .map(|records| WorkflowTransitionEvidenceRecords::from_records(records.unwrap_or_default()))
}

fn parse_optional_workflow_entry_lifecycle_states(
    value: Option<&str>,
) -> Result<WorkflowEntryLifecycleStateRecords, FormalGraphError> {
    value
        .map(parse_workflow_entry_lifecycle_states)
        .transpose()
        .map(|records| {
            WorkflowEntryLifecycleStateRecords::from_records(records.unwrap_or_default())
        })
}

struct WorkflowGraphPrefixes {
    name: &'static str,
    slug: &'static str,
    description: &'static str,
    slice_details: &'static str,
    step_relationships: &'static str,
    transitions: &'static str,
    outcomes: &'static str,
    command_errors: &'static str,
    owned_definitions: &'static str,
    transition_evidences: &'static str,
    entry_lifecycle_required: &'static str,
    entry_lifecycle_states: &'static str,
}

fn parse_workflow_graph(
    artifact: &str,
    prefixes: WorkflowGraphPrefixes,
) -> Result<FormalWorkflowGraph, FormalGraphError> {
    Ok(FormalWorkflowGraph {
        name: model_name(json_line_value(artifact, prefixes.name)?)?,
        slug: workflow_slug(json_line_value(artifact, prefixes.slug)?)?,
        description: model_description(json_line_value(artifact, prefixes.description)?)?,
        slice_details: WorkflowSliceDetails::from_details(slice_details_with_relationships(
            parse_slice_details(line_value(artifact, prefixes.slice_details)?)?,
            line_value_optional(artifact, prefixes.step_relationships)?
                .map(parse_step_relationships)
                .transpose()?
                .unwrap_or_default(),
        )),
        transitions: WorkflowTransitionRecords::from_records(parse_transitions(line_value(
            artifact,
            prefixes.transitions,
        )?)?),
        outcomes: WorkflowOutcomeRecords::from_records(parse_workflow_outcomes(line_value(
            artifact,
            prefixes.outcomes,
        )?)?),
        command_errors: WorkflowCommandErrorRecords::from_records(parse_workflow_command_errors(
            line_value(artifact, prefixes.command_errors)?,
        )?),
        owned_definitions: parse_optional_workflow_owned_definitions(line_value_optional(
            artifact,
            prefixes.owned_definitions,
        )?)?,
        transition_evidences: parse_optional_workflow_transition_evidences(line_value_optional(
            artifact,
            prefixes.transition_evidences,
        )?)?,
        entry_lifecycle_required: line_value_optional(artifact, prefixes.entry_lifecycle_required)?
            .map(parse_bool_value)
            .transpose()?
            .unwrap_or(false),
        entry_lifecycle_states: parse_optional_workflow_entry_lifecycle_states(
            line_value_optional(artifact, prefixes.entry_lifecycle_states)?,
        )?,
    })
}

fn line_value<'a>(artifact: &'a str, prefix: &str) -> Result<&'a str, FormalGraphError> {
    line_value_optional(artifact, prefix)?.ok_or_else(|| {
        FormalGraphError::new(format!("formal artifact is missing declaration '{prefix}'"))
    })
}

fn line_value_optional<'a>(
    artifact: &'a str,
    prefix: &str,
) -> Result<Option<&'a str>, FormalGraphError> {
    let matching_lines = artifact
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(prefix))
        .collect::<Vec<_>>();

    match matching_lines.as_slice() {
        [value] => Ok(Some(value.trim())),
        [] => Ok(None),
        _ => Err(FormalGraphError::new(format!(
            "formal artifact has duplicate declaration '{prefix}'"
        ))),
    }
}
fn json_line_value(artifact: &str, prefix: &str) -> Result<String, FormalGraphError> {
    serde_json::from_str::<String>(line_value(artifact, prefix)?).map_err(|error| {
        FormalGraphError::new(format!("invalid formal string declaration: {error}"))
    })
}

fn quint_val_value<'a>(
    artifact: &'a str,
    declaration_name: &str,
) -> Result<&'a str, FormalGraphError> {
    quint_val_value_optional(artifact, declaration_name)?.ok_or_else(|| {
        FormalGraphError::new(format!(
            "formal artifact is missing declaration 'val {declaration_name}'"
        ))
    })
}

fn quint_val_value_optional<'a>(
    artifact: &'a str,
    declaration_name: &str,
) -> Result<Option<&'a str>, FormalGraphError> {
    let declaration_prefix = format!("val {declaration_name}");
    let matching_lines = artifact
        .lines()
        .filter_map(|line| {
            let remainder = line.trim_start().strip_prefix(&declaration_prefix)?;
            let remainder = remainder.trim_start();
            if let Some(value) = remainder.strip_prefix('=') {
                return Some(value.trim());
            }
            remainder
                .strip_prefix(':')
                .and_then(|typed_remainder| typed_remainder.split_once(" = "))
                .map(|(_, value)| value.trim())
        })
        .collect::<Vec<_>>();

    match matching_lines.as_slice() {
        [value] => Ok(Some(value)),
        [] => Ok(None),
        _ => Err(FormalGraphError::new(format!(
            "formal artifact has duplicate declaration 'val {declaration_name}'"
        ))),
    }
}

fn parse_slice_details(value: &str) -> Result<Vec<WorkflowSliceDetail>, FormalGraphError> {
    let strings = parse_quoted_strings(value)?;
    if strings.len() % 5 == 0 {
        strings
            .chunks_exact(5)
            .map(|chunk| {
                Ok(WorkflowSliceDetail::new_with_relationship(
                    slice_slug(&chunk[0])?,
                    model_name(chunk[1].clone())?,
                    slice_kind_name(&chunk[2])?,
                    model_description(chunk[3].clone())?,
                    workflow_step_relationship_name(&chunk[4])?,
                ))
            })
            .collect()
    } else if strings.len() % 4 == 0 {
        strings
            .chunks_exact(4)
            .map(|chunk| {
                Ok(WorkflowSliceDetail::new(
                    slice_slug(&chunk[0])?,
                    model_name(chunk[1].clone())?,
                    slice_kind_name(&chunk[2])?,
                    model_description(chunk[3].clone())?,
                ))
            })
            .collect()
    } else {
        Err(FormalGraphError::new(
            "formal workflow slice detail declarations must contain groups of four or five strings",
        ))
    }
}

fn parse_step_relationships(
    value: &str,
) -> Result<Vec<(String, WorkflowStepRelationshipName)>, FormalGraphError> {
    quoted_string_groups(value, 2)?
        .chunks_exact(2)
        .map(|chunk| {
            Ok((
                chunk[0].clone(),
                workflow_step_relationship_name(&chunk[1])?,
            ))
        })
        .collect()
}

fn slice_details_with_relationships(
    slice_details: Vec<WorkflowSliceDetail>,
    step_relationships: Vec<(String, WorkflowStepRelationshipName)>,
) -> Vec<WorkflowSliceDetail> {
    slice_details
        .into_iter()
        .map(|slice| {
            let relationship = step_relationships
                .iter()
                .find(|(step, _relationship)| step == slice.slug().as_ref())
                .map(|(_step, relationship)| relationship.clone())
                .unwrap_or_else(|| slice.relationship().clone());
            WorkflowSliceDetail::new_with_relationship(
                slice.slug().clone(),
                slice.name().clone(),
                slice.kind().clone(),
                slice.description().clone(),
                relationship,
            )
        })
        .collect()
}

fn parse_transitions(value: &str) -> Result<Vec<WorkflowTransitionRecord>, FormalGraphError> {
    quoted_string_groups(value, 6)?
        .chunks_exact(6)
        .map(|chunk| {
            transition_record_from_formal_fields(
                &chunk[0],
                &chunk[1],
                &chunk[2],
                &chunk[3],
                Some(&chunk[4]),
                Some(&chunk[5]),
            )
        })
        .collect()
}

fn parse_workflow_outcomes(value: &str) -> Result<Vec<WorkflowOutcomeRecord>, FormalGraphError> {
    let strings = quoted_string_groups(value, 2)?;
    let externally_relevant_values = parse_bool_field_values(value, "externallyRelevant")?;
    if externally_relevant_values.len() != strings.len() / 2 {
        return Err(FormalGraphError::new(
            "formal workflow outcome declarations must include externallyRelevant for every outcome",
        ));
    }

    strings
        .chunks_exact(2)
        .zip(externally_relevant_values)
        .map(|(chunk, externally_relevant)| {
            Ok(WorkflowOutcomeRecord::new(
                transition_endpoint(&chunk[0])?,
                outcome_label_name(&chunk[1])?,
                externally_relevant,
            ))
        })
        .collect()
}

fn parse_workflow_command_errors(
    value: &str,
) -> Result<Vec<WorkflowCommandErrorRecord>, FormalGraphError> {
    quoted_string_groups(value, 3)?
        .chunks_exact(3)
        .map(|chunk| {
            Ok(WorkflowCommandErrorRecord::new(
                transition_endpoint(&chunk[0])?,
                command_name(&chunk[1])?,
                command_error_name(&chunk[2])?,
            ))
        })
        .collect()
}

fn parse_workflow_owned_definitions(
    value: &str,
) -> Result<Vec<WorkflowOwnedDefinitionRecord>, FormalGraphError> {
    let strings = parse_quoted_strings(value)?;
    if value.contains("definitionStream") || value.contains("sourceProvenance") {
        if strings.len() % 5 != 0 {
            return Err(FormalGraphError::new(
                "workflow owned definition declarations must contain groups of 5 strings",
            ));
        }
        strings
            .chunks_exact(5)
            .map(|chunk| {
                if chunk[3].is_empty() && chunk[4].is_empty() {
                    Ok(WorkflowOwnedDefinitionRecord::new(
                        transition_endpoint(&chunk[0])?,
                        workflow_owned_definition_kind(&chunk[1])?,
                        workflow_owned_definition_name(&chunk[2])?,
                    ))
                } else {
                    Ok(WorkflowOwnedDefinitionRecord::new_with_event_identity(
                        transition_endpoint(&chunk[0])?,
                        workflow_owned_definition_kind(&chunk[1])?,
                        workflow_owned_definition_name(&chunk[2])?,
                        stream_name(&chunk[3])?,
                        model_description(chunk[4].clone())?,
                    ))
                }
            })
            .collect()
    } else if strings.len() % 3 == 0 {
        strings
            .chunks_exact(3)
            .map(|chunk| {
                Ok(WorkflowOwnedDefinitionRecord::new(
                    transition_endpoint(&chunk[0])?,
                    workflow_owned_definition_kind(&chunk[1])?,
                    workflow_owned_definition_name(&chunk[2])?,
                ))
            })
            .collect()
    } else {
        Err(FormalGraphError::new(
            "workflow owned definition declarations must contain groups of 3 or 5 strings",
        ))
    }
}

fn parse_workflow_transition_evidences(
    value: &str,
) -> Result<Vec<WorkflowTransitionEvidenceRecord>, FormalGraphError> {
    quoted_string_groups(value, 6)?
        .chunks_exact(6)
        .map(|chunk| {
            Ok(WorkflowTransitionEvidenceRecord::new(
                transition_endpoint(&chunk[0])?,
                transition_endpoint(&chunk[1])?,
                workflow_transition_kind(&chunk[2])?,
                transition_trigger_name(&chunk[3])?,
                workflow_transition_evidence_text(&chunk[4])?,
                workflow_transition_evidence_text(&chunk[5])?,
            ))
        })
        .collect()
}

fn parse_workflow_entry_lifecycle_states(
    value: &str,
) -> Result<Vec<WorkflowEntryLifecycleStateRecord>, FormalGraphError> {
    quoted_string_groups(value, 3)?
        .chunks_exact(3)
        .map(|chunk| {
            Ok(WorkflowEntryLifecycleStateRecord::new(
                workflow_entry_lifecycle_state_name(&chunk[0])?,
                transition_endpoint(&chunk[1])?,
                workflow_entry_lifecycle_evidence_text(&chunk[2])?,
            ))
        })
        .collect()
}

fn transition_record_from_formal_fields(
    source: &str,
    target: &str,
    kind: &str,
    trigger: &str,
    rationale: Option<&str>,
    payload_contract: Option<&str>,
) -> Result<WorkflowTransitionRecord, FormalGraphError> {
    let source = transition_endpoint(source)?;
    let target = transition_endpoint(target)?;
    let kind = workflow_transition_kind(kind)?;
    let trigger = transition_trigger_name(trigger)?;
    match (
        rationale.filter(|value| !value.is_empty()),
        payload_contract.filter(|value| !value.is_empty()),
    ) {
        (None, Some(payload_contract)) => Ok(WorkflowTransitionRecord::new_with_payload_contract(
            source,
            target,
            kind,
            trigger,
            payload_contract_name(payload_contract)?,
        )),
        (Some(rationale), _) => Ok(WorkflowTransitionRecord::new_with_rationale(
            source,
            target,
            kind,
            trigger,
            model_description(rationale.to_owned())?,
        )),
        (None, None) => Ok(WorkflowTransitionRecord::new(source, target, kind, trigger)),
    }
}

fn quoted_string_groups(value: &str, group_size: usize) -> Result<Vec<String>, FormalGraphError> {
    let strings = parse_quoted_strings(value)?;
    if strings.len() % group_size == 0 {
        Ok(strings)
    } else {
        Err(FormalGraphError::new(format!(
            "formal graph collection declarations must contain groups of {group_size} strings"
        )))
    }
}

fn parse_quoted_strings(value: &str) -> Result<Vec<String>, FormalGraphError> {
    value
        .match_indices('"')
        .scan(None, |start, (index, _)| {
            if value[..index]
                .chars()
                .rev()
                .take_while(|character| *character == '\\')
                .count()
                % 2
                == 1
            {
                return Some(None);
            }
            match start.take() {
                Some(opening) => Some(Some((opening, index))),
                None => {
                    *start = Some(index);
                    Some(None)
                }
            }
        })
        .flatten()
        .map(|(opening, closing)| {
            serde_json::from_str::<String>(&value[opening..=closing]).map_err(|error| {
                FormalGraphError::new(format!("invalid formal quoted string: {error}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn parse_bool_field_values(value: &str, field_name: &str) -> Result<Vec<bool>, FormalGraphError> {
    value
        .split(field_name)
        .skip(1)
        .map(|after_name| {
            let after_separator = after_name
                .trim_start()
                .strip_prefix(":=")
                .or_else(|| after_name.trim_start().strip_prefix(':'))
                .ok_or_else(|| {
                    FormalGraphError::new(format!(
                        "formal bool field '{field_name}' is missing a value separator"
                    ))
                })?
                .trim_start();
            if after_separator.starts_with("true") {
                Ok(true)
            } else if after_separator.starts_with("false") {
                Ok(false)
            } else {
                Err(FormalGraphError::new(format!(
                    "formal bool field '{field_name}' must be true or false"
                )))
            }
        })
        .collect()
}

fn parse_bool_value(value: &str) -> Result<bool, FormalGraphError> {
    match value.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(FormalGraphError::new(
            "formal bool declaration must be true or false",
        )),
    }
}

fn model_name(value: String) -> Result<ModelName, FormalGraphError> {
    ModelName::try_new(value).map_err(|error| FormalGraphError::new(error.to_string()))
}

fn model_description(value: String) -> Result<ModelDescription, FormalGraphError> {
    ModelDescription::try_new(value).map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_slug(value: String) -> Result<WorkflowSlug, FormalGraphError> {
    WorkflowSlug::try_new(value).map_err(|error| FormalGraphError::new(error.to_string()))
}

fn slice_slug(value: &str) -> Result<SliceSlug, FormalGraphError> {
    SliceSlug::try_new(value.to_owned()).map_err(|error| FormalGraphError::new(error.to_string()))
}

fn slice_kind_name(value: &str) -> Result<SliceKindName, FormalGraphError> {
    SliceKindName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn transition_endpoint(value: &str) -> Result<WorkflowTransitionEndpoint, FormalGraphError> {
    WorkflowTransitionEndpoint::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_transition_kind(value: &str) -> Result<WorkflowTransitionKind, FormalGraphError> {
    WorkflowTransitionKind::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_step_relationship_name(
    value: &str,
) -> Result<WorkflowStepRelationshipName, FormalGraphError> {
    WorkflowStepRelationshipName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn transition_trigger_name(value: &str) -> Result<TransitionTriggerName, FormalGraphError> {
    TransitionTriggerName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_transition_evidence_text(
    value: &str,
) -> Result<WorkflowTransitionEvidenceText, FormalGraphError> {
    WorkflowTransitionEvidenceText::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_entry_lifecycle_state_name(
    value: &str,
) -> Result<WorkflowEntryLifecycleStateName, FormalGraphError> {
    WorkflowEntryLifecycleStateName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_entry_lifecycle_evidence_text(
    value: &str,
) -> Result<WorkflowEntryLifecycleEvidenceText, FormalGraphError> {
    WorkflowEntryLifecycleEvidenceText::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn payload_contract_name(value: &str) -> Result<PayloadContractName, FormalGraphError> {
    PayloadContractName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn stream_name(value: &str) -> Result<StreamName, FormalGraphError> {
    StreamName::try_new(value.to_owned()).map_err(|error| FormalGraphError::new(error.to_string()))
}

fn outcome_label_name(value: &str) -> Result<OutcomeLabelName, FormalGraphError> {
    OutcomeLabelName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn command_name(value: &str) -> Result<CommandName, FormalGraphError> {
    CommandName::try_new(value.to_owned()).map_err(|error| FormalGraphError::new(error.to_string()))
}

fn command_error_name(value: &str) -> Result<CommandErrorName, FormalGraphError> {
    CommandErrorName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_owned_definition_kind(
    value: &str,
) -> Result<WorkflowOwnedDefinitionKind, FormalGraphError> {
    WorkflowOwnedDefinitionKind::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

fn workflow_owned_definition_name(
    value: &str,
) -> Result<WorkflowOwnedDefinitionName, FormalGraphError> {
    WorkflowOwnedDefinitionName::try_new(value.to_owned())
        .map_err(|error| FormalGraphError::new(error.to_string()))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormalGraphError {
    message: String,
}

impl FormalGraphError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for FormalGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for FormalGraphError {}
