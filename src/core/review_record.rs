// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::{Map, Value, json};

use crate::core::effect::{
    ArtifactDigest, Effect, EffectPlan, FileContents, ModelContentDigest, ProjectPath, ReportLine,
};
use crate::core::types::{ReviewRuleName, ReviewStatus, ReviewTimestamp, ReviewerId, WorkflowSlug};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct RequiredReviewCategories {
    categories: Vec<ReviewRuleName>,
}

impl RequiredReviewCategories {
    pub(crate) fn new(categories: Vec<ReviewRuleName>) -> Self {
        Self { categories }
    }

    fn as_slice(&self) -> &[ReviewRuleName] {
        &self.categories
    }
}

pub(crate) fn record_clean_review(
    workflow_slug: WorkflowSlug,
    model_content_digest: ModelContentDigest,
    reviewer_id: ReviewerId,
    reviewed_at: ReviewTimestamp,
    required_categories: RequiredReviewCategories,
) -> Result<EffectPlan, ReviewRecordDocumentError> {
    Ok(EffectPlan::new(vec![
        Effect::write_file(
            review_record_path(&workflow_slug)?,
            clean_review_record_contents(
                &workflow_slug,
                &model_content_digest,
                &reviewer_id,
                &reviewed_at,
                required_categories.as_slice(),
            )?,
        ),
        Effect::Report(recorded_clean_review_report(&workflow_slug)?),
    ]))
}

fn review_record_path(
    workflow_slug: &WorkflowSlug,
) -> Result<ProjectPath, ReviewRecordDocumentError> {
    ProjectPath::try_new(format!("reviews/{}.review.json", workflow_slug.as_ref()))
        .map_err(|error| ReviewRecordDocumentError::new(error.to_string()))
}

fn clean_review_record_contents(
    workflow_slug: &WorkflowSlug,
    model_content_digest: &ModelContentDigest,
    reviewer_id: &ReviewerId,
    reviewed_at: &ReviewTimestamp,
    required_categories: &[ReviewRuleName],
) -> Result<FileContents, ReviewRecordDocumentError> {
    let category_results = required_categories
        .iter()
        .fold(Map::new(), |mut results, category| {
            results.insert(
                category.as_ref().to_owned(),
                json!(ReviewStatus::Clean.as_ref()),
            );
            results
        });
    let document = json!({
        "workflow_slug": workflow_slug.as_ref(),
        "model_content_digest": model_content_digest.as_ref(),
        "reviewer_id": reviewer_id.as_ref(),
        "status": ReviewStatus::Clean.as_ref(),
        "category_results": category_results,
        "mandatory_findings": [],
        "reviewed_at": reviewed_at.as_ref()
    });
    FileContents::try_new(format!(
        "{}\n",
        serde_json::to_string_pretty(&document)
            .map_err(|error| ReviewRecordDocumentError::new(error.to_string()))?
    ))
    .map_err(|error| ReviewRecordDocumentError::new(error.to_string()))
}

fn recorded_clean_review_report(
    workflow_slug: &WorkflowSlug,
) -> Result<ReportLine, ReviewRecordDocumentError> {
    ReportLine::try_new(format!(
        "recorded clean review for workflow {}",
        workflow_slug.as_ref()
    ))
    .map_err(|error| ReviewRecordDocumentError::new(error.to_string()))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ReviewRecordDocument {
    workflow_slug: Option<WorkflowSlug>,
    model_content_digest: Option<ModelContentDigest>,
    status: Option<ReviewStatus>,
    category_results: Option<Vec<ReviewCategoryResult>>,
    mandatory_findings: Option<Vec<MandatoryReviewFinding>>,
}

impl ReviewRecordDocument {
    pub(crate) fn parse(contents: &FileContents) -> Result<Self, ReviewRecordDocumentError> {
        let value = serde_json::from_str::<Value>(contents.as_ref()).map_err(|error| {
            ReviewRecordDocumentError::new(format!("invalid review record JSON: {error}"))
        })?;
        let object = value
            .as_object()
            .ok_or_else(|| ReviewRecordDocumentError::new("review record must be a JSON object"))?;
        Ok(Self {
            workflow_slug: optional_string_field(object, "workflow_slug")?
                .map(|slug| {
                    WorkflowSlug::try_new(slug.to_owned()).map_err(|error| {
                        ReviewRecordDocumentError::new(format!(
                            "invalid review record workflow slug: {error}"
                        ))
                    })
                })
                .transpose()?,
            model_content_digest: optional_string_field(object, "model_content_digest")?
                .map(|digest| {
                    ArtifactDigest::try_new(digest.to_owned())
                        .map(ModelContentDigest::new)
                        .map_err(|error| {
                            ReviewRecordDocumentError::new(format!(
                                "invalid review record model content digest: {error}"
                            ))
                        })
                })
                .transpose()?,
            status: optional_string_field(object, "status")?
                .map(|status| {
                    ReviewStatus::try_new(status.to_owned()).map_err(|error| {
                        ReviewRecordDocumentError::new(format!(
                            "invalid review record status: {error}"
                        ))
                    })
                })
                .transpose()?,
            category_results: parse_category_results(object)?,
            mandatory_findings: parse_mandatory_findings(object)?,
        })
    }

    pub(crate) fn matches_workflow(&self, workflow_slug: &WorkflowSlug) -> bool {
        self.workflow_slug.as_ref() == Some(workflow_slug)
    }

    pub(crate) fn workflow_slug(&self) -> Option<WorkflowSlug> {
        self.workflow_slug.clone()
    }

    pub(crate) fn is_clean(&self) -> bool {
        self.status == Some(ReviewStatus::Clean)
    }

    pub(crate) fn model_content_digest_matches(&self, digest: &ModelContentDigest) -> bool {
        self.model_content_digest.as_ref() == Some(digest)
    }

    pub(crate) fn current_mandatory_findings_include(&self, digest: &ModelContentDigest) -> bool {
        self.mandatory_findings.as_deref().is_some_and(|findings| {
            findings
                .iter()
                .any(|finding| finding.model_content_digest.as_ref() == Some(digest))
        })
    }

    pub(crate) fn has_mandatory_findings(&self) -> bool {
        self.mandatory_findings
            .as_deref()
            .is_some_and(|findings| !findings.is_empty())
    }

    pub(crate) fn first_non_clean_category(
        &self,
        required_categories: &[ReviewRuleName],
    ) -> Option<ReviewCategoryFinding> {
        let category_results = self.category_results.as_deref()?;
        required_categories.iter().find_map(|category| {
            match category_results
                .iter()
                .find(|result| result.category == *category)
                .map(|result| result.status)
            {
                Some(ReviewStatus::Clean) => None,
                Some(_status) => Some(ReviewCategoryFinding::NotClean(*category)),
                None => Some(ReviewCategoryFinding::Missing(*category)),
            }
        })
    }

    pub(crate) fn has_category_results(&self) -> bool {
        self.category_results.is_some()
    }
}

fn optional_string_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, ReviewRecordDocumentError> {
    match object.get(field) {
        Some(Value::String(value)) => Ok(Some(value.as_str())),
        Some(_value) => Err(ReviewRecordDocumentError::new(format!(
            "review record field '{field}' must be a string"
        ))),
        None => Ok(None),
    }
}

fn parse_category_results(
    object: &Map<String, Value>,
) -> Result<Option<Vec<ReviewCategoryResult>>, ReviewRecordDocumentError> {
    object
        .get("category_results")
        .map(|value| {
            let results = value.as_object().ok_or_else(|| {
                ReviewRecordDocumentError::new("review record category_results must be an object")
            })?;
            results
                .iter()
                .map(|(category, status)| {
                    let status = status.as_str().ok_or_else(|| {
                        ReviewRecordDocumentError::new(format!(
                            "review record category '{category}' status must be a string"
                        ))
                    })?;
                    Ok(ReviewCategoryResult {
                        category: ReviewRuleName::try_new(category.to_owned()).map_err(
                            |error| {
                                ReviewRecordDocumentError::new(format!(
                                    "invalid review record category: {error}"
                                ))
                            },
                        )?,
                        status: ReviewStatus::try_new(status.to_owned()).map_err(|error| {
                            ReviewRecordDocumentError::new(format!(
                                "invalid review record category status: {error}"
                            ))
                        })?,
                    })
                })
                .collect()
        })
        .transpose()
}

fn parse_mandatory_findings(
    object: &Map<String, Value>,
) -> Result<Option<Vec<MandatoryReviewFinding>>, ReviewRecordDocumentError> {
    object
        .get("mandatory_findings")
        .map(|value| {
            let findings = value.as_array().ok_or_else(|| {
                ReviewRecordDocumentError::new("review record mandatory_findings must be an array")
            })?;
            findings
                .iter()
                .map(|finding| {
                    let model_content_digest = finding
                        .as_object()
                        .and_then(|finding_object| finding_object.get("model_content_digest"))
                        .map(|digest| {
                            digest.as_str().ok_or_else(|| {
                                ReviewRecordDocumentError::new(
                                    "review record mandatory finding model_content_digest must be a string",
                                )
                            })
                        })
                        .transpose()?
                        .map(|digest| {
                            ArtifactDigest::try_new(digest.to_owned())
                                .map(ModelContentDigest::new)
                                .map_err(|error| {
                                    ReviewRecordDocumentError::new(format!(
                                        "invalid review record mandatory finding digest: {error}"
                                    ))
                                })
                        })
                        .transpose()?;
                    Ok(MandatoryReviewFinding {
                        model_content_digest,
                    })
                })
                .collect()
        })
        .transpose()
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ReviewCategoryResult {
    category: ReviewRuleName,
    status: ReviewStatus,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MandatoryReviewFinding {
    model_content_digest: Option<ModelContentDigest>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ReviewCategoryFinding {
    Missing(ReviewRuleName),
    NotClean(ReviewRuleName),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ReviewRecordDocumentError {
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
