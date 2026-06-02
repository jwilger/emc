use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::emc::{EMCSliceImport, EMCWorkflowImport};
use crate::core::effect::FileContents;
use crate::core::project::ProjectName;
use crate::core::types::{
    LeanModuleName, ModelDigest, ModelName, QuintModuleName, SliceSlug, WorkflowSlug,
};

#[derive(Debug)]
pub struct BoundaryParseError {
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

pub fn parse_model_name(raw: &str) -> Result<ModelName, BoundaryParseError> {
    ModelName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid model name: {error}")))
}

pub fn parse_project_name(raw: &str) -> Result<ProjectName, BoundaryParseError> {
    ProjectName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid project name: {error}")))
}

pub fn parse_project_manifest_name(raw: &str) -> Result<ProjectName, BoundaryParseError> {
    raw.lines()
        .find_map(|line| line.trim().strip_prefix("name = "))
        .and_then(quoted_value)
        .ok_or_else(|| BoundaryParseError::new("emc.toml is missing project name"))
        .and_then(parse_project_name)
}

pub fn parse_emc_workflow_import(
    slug: WorkflowSlug,
    raw_json: &str,
    slices: Vec<EMCSliceImport>,
) -> Result<EMCWorkflowImport, BoundaryParseError> {
    let value = serde_json::from_str::<Value>(raw_json)
        .map_err(|error| BoundaryParseError::new(format!("invalid EMC workflow JSON: {error}")))?;
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryParseError::new("EMC workflow is missing name"))
        .and_then(parse_model_name)?;
    let json = FileContents::try_new(raw_json.to_owned()).map_err(|error| {
        BoundaryParseError::new(format!("invalid EMC workflow content: {error}"))
    })?;

    Ok(EMCWorkflowImport::new(name, slug, json, slices))
}

pub fn parse_emc_slice_import(
    slug: SliceSlug,
    raw_json: &str,
) -> Result<EMCSliceImport, BoundaryParseError> {
    let value = serde_json::from_str::<Value>(raw_json)
        .map_err(|error| BoundaryParseError::new(format!("invalid EMC slice JSON: {error}")))?;
    value
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryParseError::new("EMC slice is missing name"))?;
    let json = FileContents::try_new(raw_json.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid EMC slice content: {error}")))?;

    Ok(EMCSliceImport::new(slug, json))
}

pub fn parse_workflow_slug(raw: &str) -> Result<WorkflowSlug, BoundaryParseError> {
    WorkflowSlug::try_new(slugify(raw))
        .map_err(|error| BoundaryParseError::new(format!("invalid workflow slug: {error}")))
}

pub fn parse_slice_slug(raw: &str) -> Result<SliceSlug, BoundaryParseError> {
    SliceSlug::try_new(slugify(raw))
        .map_err(|error| BoundaryParseError::new(format!("invalid slice slug: {error}")))
}

pub fn parse_lean_module_name(raw: &str) -> Result<LeanModuleName, BoundaryParseError> {
    LeanModuleName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid Lean module name: {error}")))
}

pub fn parse_quint_module_name(raw: &str) -> Result<QuintModuleName, BoundaryParseError> {
    QuintModuleName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid Quint module name: {error}")))
}

pub fn parse_model_digest(raw: &str) -> Result<ModelDigest, BoundaryParseError> {
    ModelDigest::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid model digest: {error}")))
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
