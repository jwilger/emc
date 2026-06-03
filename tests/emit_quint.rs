#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::digest::artifact_digest;
    use emc::core::emit::quint::emit_workflow_module;
    use emc::core::types::{SliceKindName, WorkflowSliceDetail, WorkflowTransitionLabel};
    use emc::io::dto::{
        parse_model_description, parse_model_name, parse_quint_module_name, parse_slice_slug,
        parse_workflow_slug,
    };

    #[test]
    fn quint_workflow_module_represents_business_workflow_fields() -> Result<(), Box<dyn Error>> {
        let workflow_name = parse_model_name("Open ticket")?;
        let workflow_description = parse_model_description("Actor opens a repair ticket.")?;
        let workflow_slug = parse_workflow_slug("open-ticket")?;
        let workflow_slice_details = vec![WorkflowSliceDetail::new(
            parse_slice_slug("capture-ticket")?,
            parse_model_name("Capture ticket")?,
            SliceKindName::try_new("state_view".to_owned())?,
            parse_model_description("Actor enters repair ticket details.")?,
        )];
        let workflow_transitions = vec![WorkflowTransitionLabel::try_new(
            "capture-ticket->review-ticket:navigation:review-ticket-screen".to_owned(),
        )?];
        let module = emit_workflow_module(
            parse_quint_module_name("OpenTicket")?,
            workflow_name.clone(),
            workflow_description.clone(),
            workflow_slug.clone(),
            workflow_slice_details.clone(),
            workflow_transitions.clone(),
            artifact_digest(
                workflow_name,
                workflow_slug,
                workflow_description,
                workflow_slice_details,
                workflow_transitions,
            ),
        );
        let quint = module.as_ref();

        assert!(quint.contains("module OpenTicket"));
        assert!(
            quint.contains(
                "// EMC-DIGEST: workflow:name=Open ticket;slug=open-ticket;description=Actor opens a repair ticket.;slices=capture-ticket|Capture ticket|state_view|Actor enters repair ticket details.;transitions=capture-ticket->review-ticket:navigation:review-ticket-screen"
            )
        );
        assert!(quint.contains("val workflowName = \"Open ticket\""));
        assert!(quint.contains("val workflowSlug = \"open-ticket\""));
        assert!(quint.contains("val workflowDescription = \"Actor opens a repair ticket.\""));
        assert!(quint.contains("val workflowSlices = [\"capture-ticket\"]"));
        assert!(
            quint.contains(
                "val workflowSliceDetails = [{ slug: \"capture-ticket\", name: \"Capture ticket\", kind: \"state_view\", description: \"Actor enters repair ticket details.\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\" }]"
            )
        );
        assert!(quint.contains("val workflowIdentityStable"));
        assert!(quint.contains("val workflowSlicesHaveDetails ="));
        assert!(quint.contains("val workflowSliceDetailsComplete = workflowSlicesHaveDetails"));
        assert!(quint.contains("var modelState: int"));
        assert!(quint.contains("action init = modelState' = 0"));
        assert!(quint.contains("action step = modelState' = modelState"));

        Ok(())
    }
}
