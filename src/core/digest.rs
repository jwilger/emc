use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{
    ModelDescription, ModelName, WorkflowSliceDetail, WorkflowSlug, WorkflowTransitionLabel,
};
use crate::core::workflow_document::WorkflowDocument;

pub fn artifact_digest(
    workflow_name: ModelName,
    workflow_slug: WorkflowSlug,
    workflow_description: ModelDescription,
    workflow_slice_details: Vec<WorkflowSliceDetail>,
    workflow_transitions: Vec<WorkflowTransitionLabel>,
) -> ArtifactDigest {
    ArtifactDigest::try_new(format!(
        "workflow:name={workflow_name};slug={workflow_slug};description={workflow_description};slices={};transitions={}",
        slice_details_digest(workflow_slice_details.as_slice()),
        transitions_digest(workflow_transitions.as_slice())
    ))
    .unwrap_or_else(|error| {
        unreachable!("EMC generated artifact digest must be valid: {error}");
    })
}

pub fn artifact_digest_from_workflow_document(
    workflow_slug: WorkflowSlug,
    workflow_document: FileContents,
) -> Result<ArtifactDigest, ArtifactDigestError> {
    let workflow = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| ArtifactDigestError::new(error.to_string()))?;

    Ok(artifact_digest(
        workflow
            .name()
            .map_err(|error| ArtifactDigestError::new(error.to_string()))?,
        workflow_slug,
        workflow
            .description()
            .map_err(|error| ArtifactDigestError::new(error.to_string()))?,
        workflow
            .slice_details()
            .map_err(|error| ArtifactDigestError::new(error.to_string()))?,
        workflow
            .transitions()
            .map_err(|error| ArtifactDigestError::new(error.to_string()))?,
    ))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ArtifactDigestError {
    message: String,
}

impl ArtifactDigestError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ArtifactDigestError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ArtifactDigestError {}

fn slice_details_digest(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    workflow_slice_details
        .iter()
        .map(|slice| {
            [
                slice.slug().as_ref(),
                slice.name().as_ref(),
                slice.kind().as_ref(),
                slice.description().as_ref(),
            ]
            .join("|")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn transitions_digest(workflow_transitions: &[WorkflowTransitionLabel]) -> String {
    workflow_transitions
        .iter()
        .map(WorkflowTransitionLabel::as_ref)
        .collect::<Vec<_>>()
        .join(",")
}
