use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::effect::FileContents;
use crate::core::types::{
    ModelDescription, ModelName, SliceKindName, SliceSlug, TransitionTriggerName,
    WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionEndpoint,
    WorkflowTransitionKind, WorkflowTransitionRecord, WorkflowTransitionRecords,
};
use crate::core::workflow_document::WorkflowDocument;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormalWorkflowGraph {
    name: ModelName,
    slug: WorkflowSlug,
    description: ModelDescription,
    slice_details: WorkflowSliceDetails,
    transitions: WorkflowTransitionRecords,
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

    pub(crate) fn into_inner(self) -> Vec<FormalWorkflowGraph> {
        self.graphs
    }
}

pub fn workflow_graph_from_document(
    workflow_slug: WorkflowSlug,
    workflow_document: FileContents,
) -> Result<FormalWorkflowGraph, FormalGraphError> {
    let workflow = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| FormalGraphError::new(error.to_string()))?;

    Ok(FormalWorkflowGraph {
        name: workflow
            .name()
            .map_err(|error| FormalGraphError::new(error.to_string()))?,
        slug: workflow_slug,
        description: workflow
            .description()
            .map_err(|error| FormalGraphError::new(error.to_string()))?,
        slice_details: WorkflowSliceDetails::from_details(
            workflow
                .slice_details()
                .map_err(|error| FormalGraphError::new(error.to_string()))?,
        ),
        transitions: WorkflowTransitionRecords::from_records(
            workflow
                .transitions()
                .map_err(|error| FormalGraphError::new(error.to_string()))?,
        ),
    })
}

pub fn parse_lean_workflow_graph(
    artifact: &FileContents,
) -> Result<FormalWorkflowGraph, FormalGraphError> {
    parse_workflow_graph(
        artifact.as_ref(),
        "def workflowName := ",
        "def workflowSlug := ",
        "def workflowDescription := ",
        "def workflowSliceDetails : List (String × String × String × String) := ",
        "def workflowTransitions : List WorkflowTransition := ",
    )
}

pub fn parse_quint_workflow_graph(
    artifact: &FileContents,
) -> Result<FormalWorkflowGraph, FormalGraphError> {
    parse_workflow_graph(
        artifact.as_ref(),
        "val workflowName = ",
        "val workflowSlug = ",
        "val workflowDescription = ",
        "val workflowSliceDetails = ",
        "val workflowTransitions = ",
    )
}

fn parse_workflow_graph(
    artifact: &str,
    name_prefix: &str,
    slug_prefix: &str,
    description_prefix: &str,
    slice_details_prefix: &str,
    transitions_prefix: &str,
) -> Result<FormalWorkflowGraph, FormalGraphError> {
    Ok(FormalWorkflowGraph {
        name: model_name(json_line_value(artifact, name_prefix)?)?,
        slug: workflow_slug(json_line_value(artifact, slug_prefix)?)?,
        description: model_description(json_line_value(artifact, description_prefix)?)?,
        slice_details: WorkflowSliceDetails::from_details(parse_slice_details(line_value(
            artifact,
            slice_details_prefix,
        )?)?),
        transitions: WorkflowTransitionRecords::from_records(parse_transitions(line_value(
            artifact,
            transitions_prefix,
        )?)?),
    })
}

fn line_value<'a>(artifact: &'a str, prefix: &str) -> Result<&'a str, FormalGraphError> {
    let matching_lines = artifact
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(prefix))
        .collect::<Vec<_>>();

    match matching_lines.as_slice() {
        [value] => Ok(value.trim()),
        [] => Err(FormalGraphError::new(format!(
            "formal artifact is missing declaration '{prefix}'"
        ))),
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

fn parse_slice_details(value: &str) -> Result<Vec<WorkflowSliceDetail>, FormalGraphError> {
    quoted_string_groups(value, 4)?
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
}

fn parse_transitions(value: &str) -> Result<Vec<WorkflowTransitionRecord>, FormalGraphError> {
    let strings = parse_quoted_strings(value)?;
    if strings.len() % 5 == 0 {
        strings
            .chunks_exact(5)
            .map(|chunk| {
                transition_record_from_formal_fields(
                    &chunk[0],
                    &chunk[1],
                    &chunk[2],
                    &chunk[3],
                    Some(&chunk[4]),
                )
            })
            .collect()
    } else if strings.len() % 4 == 0 {
        strings
            .chunks_exact(4)
            .map(|chunk| {
                transition_record_from_formal_fields(
                    &chunk[0], &chunk[1], &chunk[2], &chunk[3], None,
                )
            })
            .collect()
    } else {
        Err(FormalGraphError::new(
            "formal workflow transition declarations must contain groups of four or five strings",
        ))
    }
}

fn transition_record_from_formal_fields(
    source: &str,
    target: &str,
    kind: &str,
    trigger: &str,
    rationale: Option<&str>,
) -> Result<WorkflowTransitionRecord, FormalGraphError> {
    let source = transition_endpoint(source)?;
    let target = transition_endpoint(target)?;
    let kind = workflow_transition_kind(kind)?;
    let trigger = transition_trigger_name(trigger)?;
    match rationale.filter(|value| !value.is_empty()) {
        Some(rationale) => Ok(WorkflowTransitionRecord::new_with_rationale(
            source,
            target,
            kind,
            trigger,
            model_description(rationale.to_owned())?,
        )),
        None => Ok(WorkflowTransitionRecord::new(source, target, kind, trigger)),
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

fn transition_trigger_name(value: &str) -> Result<TransitionTriggerName, FormalGraphError> {
    TransitionTriggerName::try_new(value.to_owned())
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
