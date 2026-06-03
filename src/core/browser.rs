use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::iter;

use serde_json::Value;

use crate::core::effect::FileContents;
use crate::core::types::{
    BoardLaneId, WorkflowBranchLabel, WorkflowStepName, WorkflowTransitionKind,
    WorkflowTransitionLabel,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserWorkflow {
    lane_ids: Vec<BoardLaneId>,
    main_path_names: Vec<WorkflowStepName>,
    branch_cards: Vec<BrowserBranchCard>,
    transition_cards: Vec<BrowserTransitionCard>,
}

impl BrowserWorkflow {
    pub fn lane_ids(&self) -> &[BoardLaneId] {
        &self.lane_ids
    }

    pub fn main_path_names(&self) -> &[WorkflowStepName] {
        &self.main_path_names
    }

    pub fn branch_cards(&self) -> &[BrowserBranchCard] {
        &self.branch_cards
    }

    pub fn transition_cards(&self) -> &[BrowserTransitionCard] {
        &self.transition_cards
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserBranchCard {
    name: WorkflowStepName,
    label: WorkflowBranchLabel,
}

impl BrowserBranchCard {
    pub fn name(&self) -> &WorkflowStepName {
        &self.name
    }

    pub fn label(&self) -> &WorkflowBranchLabel {
        &self.label
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserTransitionCard {
    source: WorkflowStepName,
    target: WorkflowStepName,
    kind: WorkflowTransitionKind,
    label: WorkflowTransitionLabel,
}

impl BrowserTransitionCard {
    pub fn source(&self) -> &WorkflowStepName {
        &self.source
    }

    pub fn target(&self) -> &WorkflowStepName {
        &self.target
    }

    pub fn kind(&self) -> &WorkflowTransitionKind {
        &self.kind
    }

    pub fn label(&self) -> &WorkflowTransitionLabel {
        &self.label
    }
}

pub fn compose_browser_workflow(
    workflow_document: FileContents,
    slice_documents: Vec<FileContents>,
) -> Result<BrowserWorkflow, BrowserCompositionError> {
    let workflow_value = parse_json(workflow_document.as_ref())?;
    let slice_values = slice_documents
        .iter()
        .map(|document| parse_json(document.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;
    let lane_ids = iter::once(&workflow_value)
        .chain(slice_values.iter())
        .flat_map(board_lane_ids)
        .try_fold(Vec::<BoardLaneId>::new(), |mut lanes, lane| {
            let parsed = BoardLaneId::try_new(lane.to_owned()).map_err(|error| {
                BrowserCompositionError::new(format!("invalid board lane id: {error}"))
            })?;
            if !lanes.iter().any(|existing| existing == &parsed) {
                lanes.push(parsed);
            }
            Ok(lanes)
        })?;
    let main_path_names = workflow_main_path_names(&workflow_value)?;
    let branch_cards = workflow_branch_cards(&workflow_value)?;
    let transition_cards = workflow_transition_cards(&workflow_value)?;

    Ok(BrowserWorkflow {
        lane_ids,
        main_path_names,
        branch_cards,
        transition_cards,
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserCompositionError {
    message: String,
}

impl BrowserCompositionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for BrowserCompositionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BrowserCompositionError {}

fn parse_json(source: &str) -> Result<Value, BrowserCompositionError> {
    serde_json::from_str::<Value>(source)
        .map_err(|error| BrowserCompositionError::new(format!("invalid browser JSON: {error}")))
}

fn board_lane_ids(value: &Value) -> impl Iterator<Item = &str> {
    value
        .get("board")
        .and_then(|board| board.get("lanes"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|lane| lane.get("id").and_then(Value::as_str))
}

fn workflow_main_path_names(
    value: &Value,
) -> Result<Vec<WorkflowStepName>, BrowserCompositionError> {
    value
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|step| {
            step.get("relationship")
                .and_then(Value::as_str)
                .is_some_and(|relationship| relationship == "entry" || relationship == "main")
        })
        .filter_map(|step| step.get("name").and_then(Value::as_str))
        .map(|name| {
            WorkflowStepName::try_new(name.to_owned()).map_err(|error| {
                BrowserCompositionError::new(format!("invalid workflow step name: {error}"))
            })
        })
        .collect()
}

fn workflow_branch_cards(value: &Value) -> Result<Vec<BrowserBranchCard>, BrowserCompositionError> {
    value
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|step| {
            step.get("relationship")
                .and_then(Value::as_str)
                .is_some_and(|relationship| relationship != "entry" && relationship != "main")
        })
        .filter_map(|step| {
            step.get("name")
                .and_then(Value::as_str)
                .zip(step.get("relationship").and_then(Value::as_str))
        })
        .map(|(name, relationship)| {
            Ok(BrowserBranchCard {
                name: WorkflowStepName::try_new(name.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid workflow step name: {error}"))
                })?,
                label: WorkflowBranchLabel::try_new(branch_label(relationship)).map_err(
                    |error| {
                        BrowserCompositionError::new(format!(
                            "invalid workflow branch label: {error}"
                        ))
                    },
                )?,
            })
        })
        .collect()
}

fn branch_label(relationship: &str) -> String {
    relationship.replace('_', " ")
}

fn workflow_transition_cards(
    value: &Value,
) -> Result<Vec<BrowserTransitionCard>, BrowserCompositionError> {
    value
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|step| step_transition_cards(value, step))
        .collect::<Result<Vec<_>, _>>()
        .map(|cards| cards.into_iter().flatten().collect())
}

fn step_transition_cards(
    workflow_value: &Value,
    step: &Value,
) -> Result<Vec<BrowserTransitionCard>, BrowserCompositionError> {
    let source = step.get("name").and_then(Value::as_str);

    step.get("transitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|transition| {
            Some((
                source?,
                transition_target_name(workflow_value, transition)?,
                transition_kind_and_label(transition)?,
            ))
        })
        .map(|(source, target, (kind, label))| {
            Ok(BrowserTransitionCard {
                source: parse_workflow_step_name(source)?,
                target: parse_workflow_step_name(target)?,
                kind: WorkflowTransitionKind::try_new(kind.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!(
                        "invalid workflow transition kind: {error}"
                    ))
                })?,
                label: WorkflowTransitionLabel::try_new(label.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!(
                        "invalid workflow transition label: {error}"
                    ))
                })?,
            })
        })
        .collect()
}

fn transition_target_name<'a>(workflow_value: &'a Value, transition: &'a Value) -> Option<&'a str> {
    transition
        .get("to")
        .and_then(Value::as_str)
        .map(|target_slice| {
            workflow_step_name_for_slice(workflow_value, target_slice).unwrap_or(target_slice)
        })
        .or_else(|| transition.get("target_name").and_then(Value::as_str))
        .or_else(|| transition.get("to_workflow").and_then(Value::as_str))
}

fn workflow_step_name_for_slice<'a>(workflow_value: &'a Value, slice: &str) -> Option<&'a str> {
    workflow_value
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|step| {
            step.get("slice")
                .and_then(Value::as_str)
                .is_some_and(|step_slice| step_slice == slice)
        })
        .and_then(|step| step.get("name").and_then(Value::as_str))
}

fn transition_kind_and_label(transition: &Value) -> Option<(&'static str, &str)> {
    transition
        .get("via_navigation")
        .and_then(Value::as_str)
        .map(|label| ("navigation", label))
        .or_else(|| {
            transition
                .get("via_command")
                .and_then(Value::as_str)
                .map(|label| ("command", label))
        })
        .or_else(|| {
            transition
                .get("via_event")
                .and_then(Value::as_str)
                .map(|label| ("event", label))
        })
        .or_else(|| {
            transition
                .get("via_external_trigger")
                .and_then(Value::as_str)
                .map(|label| ("external trigger", label))
        })
        .or_else(|| {
            transition
                .get("via_outcome")
                .and_then(Value::as_str)
                .map(|label| ("workflow exit", label))
        })
}

fn parse_workflow_step_name(value: &str) -> Result<WorkflowStepName, BrowserCompositionError> {
    WorkflowStepName::try_new(value.to_owned()).map_err(|error| {
        BrowserCompositionError::new(format!("invalid workflow step name: {error}"))
    })
}
