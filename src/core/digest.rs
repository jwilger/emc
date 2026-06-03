use crate::core::effect::ArtifactDigest;
use crate::core::types::{ModelDescription, ModelName, WorkflowSlug};

pub fn artifact_digest(
    workflow_name: ModelName,
    workflow_slug: WorkflowSlug,
    workflow_description: ModelDescription,
) -> ArtifactDigest {
    ArtifactDigest::try_new(format!(
        "workflow:name={workflow_name};slug={workflow_slug};description={workflow_description}"
    ))
    .unwrap_or_else(|error| {
        unreachable!("EMC generated artifact digest must be valid: {error}");
    })
}
