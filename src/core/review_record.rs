use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{ReviewRuleName, ReviewStatus, WorkflowSlug};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReviewRecordDocument {
    value: Value,
}

impl ReviewRecordDocument {
    pub fn parse(contents: &FileContents) -> Result<Self, ReviewRecordDocumentError> {
        let value = serde_json::from_str::<Value>(contents.as_ref()).map_err(|error| {
            ReviewRecordDocumentError::new(format!("invalid review record JSON: {error}"))
        })?;
        value
            .as_object()
            .ok_or_else(|| ReviewRecordDocumentError::new("review record must be a JSON object"))?;
        Ok(Self { value })
    }

    pub fn matches_workflow(&self, workflow_slug: &WorkflowSlug) -> bool {
        self.string_field("workflow_slug")
            .is_some_and(|observed| observed == workflow_slug.as_ref())
    }

    pub fn workflow_slug(&self) -> Option<WorkflowSlug> {
        self.string_field("workflow_slug")
            .and_then(|slug| WorkflowSlug::try_new(slug.to_owned()).ok())
    }

    pub fn is_clean(&self) -> bool {
        self.status()
            .is_some_and(|status| status.as_ref() == "clean")
    }

    pub fn model_content_digest_matches(&self, digest: &ArtifactDigest) -> bool {
        self.string_field("model_content_digest")
            .is_some_and(|observed| observed == digest.as_ref())
    }

    pub fn current_mandatory_findings_include(&self, digest: &ArtifactDigest) -> bool {
        self.object()
            .and_then(|object| object.get("mandatory_findings"))
            .and_then(Value::as_array)
            .is_some_and(|findings| {
                findings.iter().any(|finding| {
                    finding.get("model_content_digest").and_then(Value::as_str)
                        == Some(digest.as_ref())
                })
            })
    }

    pub fn has_mandatory_findings(&self) -> bool {
        self.object()
            .and_then(|object| object.get("mandatory_findings"))
            .and_then(Value::as_array)
            .is_some_and(|findings| !findings.is_empty())
    }

    pub fn first_non_clean_category(
        &self,
        required_categories: &[ReviewRuleName],
    ) -> Option<ReviewCategoryFinding> {
        let category_results = self
            .object()
            .and_then(|object| object.get("category_results"))
            .and_then(Value::as_object)?;
        required_categories.iter().find_map(|category| {
            match category_results
                .get(category.as_ref())
                .and_then(Value::as_str)
            {
                Some("clean") => None,
                Some(_) => Some(ReviewCategoryFinding::NotClean(category.clone())),
                None => Some(ReviewCategoryFinding::Missing(category.clone())),
            }
        })
    }

    pub fn has_category_results(&self) -> bool {
        self.object()
            .and_then(|object| object.get("category_results"))
            .and_then(Value::as_object)
            .is_some()
    }

    fn status(&self) -> Option<ReviewStatus> {
        self.string_field("status")
            .and_then(|status| ReviewStatus::try_new(status.to_owned()).ok())
    }

    fn string_field(&self, field: &str) -> Option<&str> {
        self.object()
            .and_then(|object| object.get(field))
            .and_then(Value::as_str)
    }

    fn object(&self) -> Option<&serde_json::Map<String, Value>> {
        self.value.as_object()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReviewCategoryFinding {
    Missing(ReviewRuleName),
    NotClean(ReviewRuleName),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReviewRecordDocumentError {
    message: String,
}

impl ReviewRecordDocumentError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ReviewRecordDocumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ReviewRecordDocumentError {}
