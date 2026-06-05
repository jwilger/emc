// Copyright 2026 John Wilger

use crate::core::effect::{Effect, EffectPlan, ProjectPath, ReportLine};
use crate::core::types::WorkflowSlug;

pub fn review_gate(workflow_slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![
        Effect::RequireReviewRecord(
            project_path(format!("reviews/{}.review.json", workflow_slug.as_ref())),
            workflow_slug,
            report_line("workflow review is not clean"),
        ),
        Effect::Report(report_line("workflow review is clean")),
    ])
}

fn project_path(value: impl Into<String>) -> ProjectPath {
    ProjectPath::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated project path must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated report line must be valid: {error}");
    })
}
