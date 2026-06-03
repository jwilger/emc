use crate::core::effect::ArtifactDigest;
use crate::core::types::ModelName;

pub fn artifact_digest(workflow_name: ModelName) -> ArtifactDigest {
    ArtifactDigest::try_new(format!("workflow:{workflow_name}")).unwrap_or_else(|error| {
        unreachable!("EMC generated artifact digest must be valid: {error}");
    })
}
