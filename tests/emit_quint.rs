#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::effect::ArtifactDigest;
    use emc::core::emit::quint::emit_workflow_module;
    use emc::core::types::WorkflowTransitionLabel;
    use emc::io::dto::{
        parse_model_description, parse_model_name, parse_quint_module_name, parse_slice_slug,
        parse_workflow_slug,
    };

    #[test]
    fn quint_workflow_module_represents_business_workflow_fields() -> Result<(), Box<dyn Error>> {
        let module = emit_workflow_module(
            parse_quint_module_name("OpenTicket")?,
            parse_model_name("Open ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            parse_workflow_slug("open-ticket")?,
            vec![parse_slice_slug("capture-ticket")?],
            vec![WorkflowTransitionLabel::try_new(
                "capture-ticket->review-ticket:navigation:review-ticket-screen".to_owned(),
            )?],
            ArtifactDigest::try_new("workflow:Open ticket".to_owned())?,
        );
        let quint = module.as_ref();

        assert!(quint.contains("module OpenTicket"));
        assert!(quint.contains("// EMC-DIGEST: workflow:Open ticket"));
        assert!(quint.contains("const workflowName = \"Open ticket\""));
        assert!(quint.contains("const workflowSlug = \"open-ticket\""));
        assert!(quint.contains("const workflowDescription = \"Actor opens a repair ticket.\""));
        assert!(quint.contains("const workflowSlices = [\"capture-ticket\"]"));
        assert!(
            quint.contains(
                "const workflowTransitions = [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            )
        );
        assert!(quint.contains("val workflowIdentityStable"));

        Ok(())
    }
}
