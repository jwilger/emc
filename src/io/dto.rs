use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::{Map, Value};

use crate::core::emc::{EMCSliceImport, EMCWorkflowImport};
use crate::core::effect::FileContents;
use crate::core::layout::ImportedWorkflowLayout;
use crate::core::project::ProjectName;
use crate::core::types::{
    LeanModuleName, ModelDigest, ModelName, QuintModuleName, SliceSlug, WorkflowSlug,
};
use crate::core::validation::{
    DefinitionKind, DefinitionName, EventModelDocument, NamedDefinition, TopLevelKey,
    empty_top_level_key_issue, model_must_be_object_issue,
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

pub fn parse_event_model_document(raw: &str) -> Result<EventModelDocument, BoundaryParseError> {
    serde_json::from_str::<Value>(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid JSON: {error}")))
        .and_then(event_model_document_from_json)
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

pub fn parse_browser_index_workflows(
    raw: &str,
) -> Result<Vec<ImportedWorkflowLayout>, BoundaryParseError> {
    let value = serde_json::from_str::<Value>(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid browser index JSON: {error}")))?;
    let workflows = value
        .get("workflows")
        .and_then(Value::as_array)
        .ok_or_else(|| BoundaryParseError::new("browser index is missing workflows"))?;

    workflows
        .iter()
        .map(|workflow| {
            let name = workflow
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("browser index workflow is missing name"))
                .and_then(parse_model_name)?;
            let path = workflow
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("browser index workflow is missing path"))?;
            let slug = path
                .strip_prefix("data/workflows/")
                .and_then(|file_name| file_name.strip_suffix(".eventmodel.json"))
                .ok_or_else(|| BoundaryParseError::new("browser index workflow path is invalid"))
                .and_then(parse_workflow_slug)?;

            Ok(ImportedWorkflowLayout::new(name, slug))
        })
        .collect()
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

fn event_model_document_from_json(value: Value) -> Result<EventModelDocument, BoundaryParseError> {
    let object = value
        .as_object()
        .ok_or_else(|| BoundaryParseError::new(model_must_be_object_issue().to_string()))?;
    object
        .keys()
        .map(|key| {
            TopLevelKey::try_new(key.to_owned())
                .map_err(|_| BoundaryParseError::new(empty_top_level_key_issue().to_string()))
        })
        .collect::<Result<BTreeSet<_>, _>>()
        .and_then(|top_level_keys| {
            named_definitions_from_json_object(object)
                .map(|named_definitions| EventModelDocument::new(top_level_keys, named_definitions))
        })
}

fn named_definitions_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<NamedDefinition>, BoundaryParseError> {
    named_definition_specs()
        .iter()
        .map(|spec| named_definitions_for_spec(object, spec))
        .collect::<Result<Vec<_>, _>>()
        .map(|definitions| definitions.into_iter().flatten().collect())
}

fn named_definitions_for_spec(
    object: &Map<String, Value>,
    spec: &NamedDefinitionSpec,
) -> Result<Vec<NamedDefinition>, BoundaryParseError> {
    object
        .get(spec.key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|definition| {
            definition
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new(format!("{} is missing name", spec.label)))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid {} name: {error}", spec.label))
                    })
                })
                .map(|name| NamedDefinition::new(spec.kind, name))
        })
        .collect()
}

struct NamedDefinitionSpec {
    key: &'static str,
    label: &'static str,
    kind: DefinitionKind,
}

fn named_definition_specs() -> &'static [NamedDefinitionSpec] {
    &[
        NamedDefinitionSpec {
            key: "streams",
            label: "stream",
            kind: DefinitionKind::Stream,
        },
        NamedDefinitionSpec {
            key: "events",
            label: "event",
            kind: DefinitionKind::Event,
        },
        NamedDefinitionSpec {
            key: "commands",
            label: "command",
            kind: DefinitionKind::Command,
        },
        NamedDefinitionSpec {
            key: "read_models",
            label: "read model",
            kind: DefinitionKind::ReadModel,
        },
        NamedDefinitionSpec {
            key: "views",
            label: "view",
            kind: DefinitionKind::View,
        },
    ]
}
