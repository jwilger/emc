use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{ModelDescription, ModelName, QuintModuleName, WorkflowSlug};

pub fn emit_workflow_module(
    module_name: QuintModuleName,
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    digest: ArtifactDigest,
) -> FileContents {
    file_contents(format!(
        "module {module_name} {{\n  // EMC-DIGEST: {digest}\n  const workflowName = {workflow_name_json}\n  const workflowSlug = {workflow_slug_json}\n  const workflowDescription = {workflow_description_json}\n  val workflowIdentityStable = workflowName == {workflow_name_json}\n}}\n",
        module_name = module_name.as_ref(),
        digest = digest.as_ref(),
        workflow_name_json = quoted(workflow_name.as_ref()),
        workflow_slug_json = quoted(workflow_slug.as_ref()),
        workflow_description_json = quoted(workflow_description.as_ref()),
    ))
}

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Quint file contents must be valid: {error}");
    })
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated Quint string literal must be valid: {error}");
    })
}
