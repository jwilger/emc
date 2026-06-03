#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::effect::ArtifactDigest;
    use emc::core::emit::lean::emit_workflow_module;
    use emc::core::types::{SliceKindName, WorkflowSliceDetail, WorkflowTransitionLabel};
    use emc::io::dto::{
        parse_lean_module_name, parse_model_description, parse_model_name, parse_slice_slug,
        parse_workflow_slug,
    };

    #[test]
    fn lean_workflow_module_represents_business_workflow_fields() -> Result<(), Box<dyn Error>> {
        let module = emit_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            parse_model_name("Open ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            parse_workflow_slug("open-ticket")?,
            vec![WorkflowSliceDetail::new(
                parse_slice_slug("capture-ticket")?,
                parse_model_name("Capture ticket")?,
                SliceKindName::try_new("state_view".to_owned())?,
                parse_model_description("Actor enters repair ticket details.")?,
            )],
            vec![WorkflowTransitionLabel::try_new(
                "capture-ticket->review-ticket:navigation:review-ticket-screen".to_owned(),
            )?],
            ArtifactDigest::try_new("workflow:Open ticket".to_owned())?,
        );
        let lean = module.as_ref();

        assert!(lean.contains("namespace OpenTicket"));
        assert!(lean.contains("-- EMC-DIGEST: workflow:Open ticket"));
        assert!(lean.contains("def workflowName := \"Open ticket\""));
        assert!(lean.contains("def workflowSlug := \"open-ticket\""));
        assert!(lean.contains("def workflowDescription := \"Actor opens a repair ticket.\""));
        assert!(lean.contains("def workflowSlices : List String := [\"capture-ticket\"]"));
        assert!(
            lean.contains(
                "def workflowSliceDetails : List (String × String × String × String) := [(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Actor enters repair ticket details.\")]"
            )
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List String := [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            )
        );
        assert!(lean.contains("theorem workflowIdentityIsStable"));
        assert!(
            lean.contains(
                "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl"
            ),
            "Lean artifact must prove every modeled workflow slice has generated detail metadata"
        );

        Ok(())
    }

    #[test]
    fn lean_workflow_module_types_empty_lists() -> Result<(), Box<dyn Error>> {
        let module = emit_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            parse_model_name("Open ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            parse_workflow_slug("open-ticket")?,
            Vec::new(),
            Vec::new(),
            ArtifactDigest::try_new("workflow:Open ticket".to_owned())?,
        );
        let lean = module.as_ref();

        assert!(lean.contains("def workflowSlices : List String := []"));
        assert!(
            lean.contains(
                "def workflowSliceDetails : List (String × String × String × String) := []"
            )
        );
        assert!(lean.contains("def workflowTransitions : List String := []"));

        Ok(())
    }
}
