use std::collections::BTreeSet;

use nutype::nutype;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventModelDocument {
    top_level_keys: BTreeSet<TopLevelKey>,
}

impl EventModelDocument {
    pub fn new(top_level_keys: BTreeSet<TopLevelKey>) -> Self {
        Self { top_level_keys }
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, AsRef, Display)
)]
pub struct TopLevelKey(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ValidationIssue(String);

pub fn validate_event_model(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    required_top_level_keys()
        .iter()
        .find(|key| !document.top_level_keys.contains(*key))
        .map_or(Ok(()), |key| {
            Err(validation_issue(format!("missing top-level key '{key}'")))
        })
}

pub fn model_must_be_object_issue() -> ValidationIssue {
    validation_issue("model must be a JSON object")
}

pub fn empty_top_level_key_issue() -> ValidationIssue {
    validation_issue("top-level key must not be empty")
}

fn top_level_key(raw: &str) -> TopLevelKey {
    TopLevelKey::try_new(raw.to_owned()).unwrap_or_else(|error| {
        unreachable!("EMC required top-level key must be valid: {error}");
    })
}

fn validation_issue(value: impl Into<String>) -> ValidationIssue {
    ValidationIssue::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC validation issue must be non-empty: {error}");
    })
}

fn required_top_level_keys() -> Vec<TopLevelKey> {
    [
        "name",
        "version",
        "streams",
        "events",
        "commands",
        "read_models",
        "slices",
    ]
    .iter()
    .map(|key| top_level_key(key))
    .collect()
}
