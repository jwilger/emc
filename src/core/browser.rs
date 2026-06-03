use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::iter;

use serde_json::Value;

use crate::core::effect::FileContents;
use crate::core::types::{BoardLaneId, WorkflowBranchLabel, WorkflowStepName};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserWorkflow {
    lane_ids: Vec<BoardLaneId>,
    main_path_names: Vec<WorkflowStepName>,
    branch_cards: Vec<BrowserBranchCard>,
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

    Ok(BrowserWorkflow {
        lane_ids,
        main_path_names,
        branch_cards,
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
