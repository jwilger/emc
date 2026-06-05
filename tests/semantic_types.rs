// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::io::dto::{
        parse_lean_module_name, parse_model_description, parse_model_digest, parse_model_name,
        parse_project_name, parse_project_path, parse_quint_module_name, parse_review_timestamp,
        parse_reviewer_id, parse_slice_slug, parse_transition_trigger_name, parse_workflow_slug,
        parse_workflow_transition_endpoint, parse_workflow_transition_kind,
    };

    #[test]
    fn boundary_parsers_convert_raw_strings_to_semantic_types() -> Result<(), Box<dyn Error>> {
        let project_name = parse_project_name(" Repair Desk ")?;
        let model_name = parse_model_name(" Repair Desk ")?;
        let model_description = parse_model_description(" Repairs organization access ")?;
        let workflow_slug = parse_workflow_slug(" Organization Access ")?;
        let slice_slug = parse_slice_slug(" Resolve Application Entry ")?;
        let lean_module = parse_lean_module_name(" RepairDesk ")?;
        let quint_module = parse_quint_module_name(" RepairDesk ")?;
        let digest = parse_model_digest(" abc123 ")?;
        let transition_trigger = parse_transition_trigger_name(" Open request ")?;
        let transition_endpoint = parse_workflow_transition_endpoint(" capture-ticket ")?;
        let transition_kind = parse_workflow_transition_kind(" navigation ")?;
        let reviewer = parse_reviewer_id(" event-model-reviewer ")?;
        let reviewed_at = parse_review_timestamp(" 2026-06-03T00:00:00.000Z ")?;

        assert_eq!(
            project_name.as_ref(),
            "Repair Desk",
            "project name is trimmed"
        );
        assert_eq!(model_name.as_ref(), "Repair Desk", "model name is trimmed");
        assert_eq!(
            model_description.as_ref(),
            "Repairs organization access",
            "model description is trimmed"
        );
        assert_eq!(
            workflow_slug.as_ref(),
            "organization-access",
            "workflow slug is normalized"
        );
        assert_eq!(
            slice_slug.as_ref(),
            "resolve-application-entry",
            "slice slug is normalized"
        );
        assert_eq!(
            lean_module.as_ref(),
            "RepairDesk",
            "Lean module name is trimmed"
        );
        assert_eq!(
            quint_module.as_ref(),
            "RepairDesk",
            "Quint module name is trimmed"
        );
        assert_eq!(digest.as_ref(), "abc123", "digest is trimmed");
        assert_eq!(
            transition_trigger.as_ref(),
            "Open request",
            "transition trigger is trimmed"
        );
        assert_eq!(
            transition_endpoint.as_ref(),
            "capture-ticket",
            "workflow transition endpoint is trimmed"
        );
        assert_eq!(
            transition_kind.as_ref(),
            "navigation",
            "workflow transition kind is trimmed"
        );
        assert_eq!(
            reviewer.as_ref(),
            "event-model-reviewer",
            "reviewer id is trimmed"
        );
        assert_eq!(
            reviewed_at.as_ref(),
            "2026-06-03T00:00:00.000Z",
            "review timestamp is trimmed"
        );

        Ok(())
    }

    #[test]
    fn boundary_parsers_reject_empty_semantic_values() {
        assert!(
            parse_project_name("   ").is_err(),
            "blank project names must not enter the core"
        );
        assert!(
            parse_model_name("   ").is_err(),
            "blank model names must not enter the core"
        );
        assert!(
            parse_model_description("   ").is_err(),
            "blank model descriptions must not enter the core"
        );
        assert!(
            parse_workflow_slug("   ").is_err(),
            "blank workflow slugs must not enter the core"
        );
        assert!(
            parse_slice_slug("   ").is_err(),
            "blank slice slugs must not enter the core"
        );
        assert!(
            parse_lean_module_name("   ").is_err(),
            "blank Lean module names must not enter the core"
        );
        assert!(
            parse_quint_module_name("   ").is_err(),
            "blank Quint module names must not enter the core"
        );
        assert!(
            parse_model_digest("   ").is_err(),
            "blank model digests must not enter the core"
        );
        assert!(
            parse_transition_trigger_name("   ").is_err(),
            "blank transition triggers must not enter the core"
        );
        assert!(
            parse_workflow_transition_endpoint("   ").is_err(),
            "blank workflow transition endpoints must not enter the core"
        );
        assert!(
            parse_workflow_transition_kind("   ").is_err(),
            "blank workflow transition kinds must not enter the core"
        );
        assert!(
            parse_reviewer_id("   ").is_err(),
            "blank reviewer ids must not enter the core"
        );
        assert!(
            parse_review_timestamp("   ").is_err(),
            "blank review timestamps must not enter the core"
        );
    }

    #[test]
    fn review_timestamps_must_use_utc_millisecond_instants() {
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00.000Z").is_ok(),
            "review timestamps use UTC instants with millisecond precision"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00Z").is_err(),
            "review timestamps must include millisecond precision"
        );
        assert!(
            parse_review_timestamp("2026-06-03 00:00:00.000Z").is_err(),
            "review timestamps must use the RFC3339 separator"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00.000-07:00").is_err(),
            "review timestamps must be normalized to UTC"
        );
        assert!(
            parse_review_timestamp("202A-06-03T00:00:00.000Z").is_err(),
            "review timestamp years must be numeric"
        );
        assert!(
            parse_review_timestamp("2026-0:-03T00:00:00.000Z").is_err(),
            "review timestamp months must be numeric"
        );
        assert!(
            parse_review_timestamp("2026-19-03T00:00:00.000Z").is_err(),
            "review timestamp months must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T24:00:00.000Z").is_err(),
            "review timestamp hours must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:60:00.000Z").is_err(),
            "review timestamp minutes must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:60.000Z").is_err(),
            "review timestamp seconds must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00.A00Z").is_err(),
            "review timestamp milliseconds must be numeric"
        );
        assert!(
            parse_review_timestamp("not-a-timestamp").is_err(),
            "review timestamps must not accept arbitrary text"
        );
    }

    #[test]
    fn project_paths_are_relative_to_the_current_project() -> Result<(), Box<dyn Error>> {
        let path = parse_project_path("model/lean/OpenTicket.lean")?;

        assert_eq!(path.as_ref(), "model/lean/OpenTicket.lean");
        assert!(
            parse_project_path("/tmp/site").is_err(),
            "absolute paths must not enter project-local effects"
        );
        assert!(
            parse_project_path("../outside-model").is_err(),
            "parent traversal must not enter project-local effects"
        );

        Ok(())
    }
}
