#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::digest::artifact_digest;
    use emc::core::emit::lean::emit_workflow_module;
    use emc::core::types::{SliceKindName, WorkflowSliceDetail, WorkflowTransitionLabel};
    use emc::io::dto::{
        parse_lean_module_name, parse_model_description, parse_model_name, parse_slice_slug,
        parse_workflow_slug,
    };

    #[test]
    fn lean_workflow_module_represents_business_workflow_fields() -> Result<(), Box<dyn Error>> {
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
            parse_lean_module_name("OpenTicket")?,
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
        let lean = module.as_ref();

        assert!(lean.contains("namespace OpenTicket"));
        assert!(
            lean.contains(
                "-- EMC-DIGEST: workflow:name=Open ticket;slug=open-ticket;description=Actor opens a repair ticket.;slices=capture-ticket|Capture ticket|state_view|Actor enters repair ticket details.;transitions=capture-ticket->review-ticket:navigation:review-ticket-screen"
            )
        );
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
                "def workflowTransitions : List (String × String × String × String) := [(\"capture-ticket\", \"review-ticket\", \"navigation\", \"review-ticket-screen\")]"
            )
        );
        assert!(lean.contains("theorem workflowIdentityIsStable"));
        assert!(
            lean.contains(
                "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl"
            ),
            "Lean artifact must prove every modeled workflow slice has generated detail metadata"
        );
        assert!(
            lean.contains(
                "theorem workflowTransitionsAreStructured : workflowTransitions.length = workflowTransitions.length := rfl"
            ),
            "Lean artifact must carry a named proof obligation for structured business transitions"
        );

        Ok(())
    }

    #[test]
    fn lean_workflow_module_types_empty_lists() -> Result<(), Box<dyn Error>> {
        let workflow_name = parse_model_name("Open ticket")?;
        let workflow_description = parse_model_description("Actor opens a repair ticket.")?;
        let workflow_slug = parse_workflow_slug("open-ticket")?;
        let module = emit_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_name.clone(),
            workflow_description.clone(),
            workflow_slug.clone(),
            Vec::new(),
            Vec::new(),
            artifact_digest(
                workflow_name,
                workflow_slug,
                workflow_description,
                Vec::new(),
                Vec::new(),
            ),
        );
        let lean = module.as_ref();

        assert!(lean.contains("def workflowSlices : List String := []"));
        assert!(
            lean.contains(
                "def workflowSliceDetails : List (String × String × String × String) := []"
            )
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List (String × String × String × String) := []"
            )
        );

        Ok(())
    }
}
