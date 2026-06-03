#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::digest::artifact_digest;
    use emc::core::effect::{ArtifactDigest, FileContents};
    use emc::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
    use emc::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
    use emc::core::formal_graph::{
        parse_lean_workflow_graph, parse_quint_workflow_graph, workflow_graph_from_document,
    };
    use emc::core::types::{
        SliceKindName, TransitionTriggerName, WorkflowSliceDetail, WorkflowSliceDetails,
        WorkflowTransitionEndpoint, WorkflowTransitionKind, WorkflowTransitionRecord,
        WorkflowTransitionRecords,
    };
    use emc::io::dto::{
        parse_lean_module_name, parse_model_description, parse_model_name, parse_quint_module_name,
        parse_slice_slug, parse_workflow_slug,
    };

    #[test]
    fn lean_workflow_artifact_parses_to_the_semantic_workflow_graph() -> Result<(), Box<dyn Error>>
    {
        let workflow_document = workflow_document()?;
        let artifact = emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            parse_model_name("Open ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            parse_workflow_slug("open-ticket")?,
            WorkflowSliceDetails::from_details(workflow_slice_details()?),
            WorkflowTransitionRecords::from_records(workflow_transitions()?),
            workflow_digest()?,
        );

        assert_eq!(
            parse_lean_workflow_graph(&artifact)?,
            workflow_graph_from_document(parse_workflow_slug("open-ticket")?, workflow_document)?,
        );

        Ok(())
    }

    #[test]
    fn quint_workflow_artifact_parses_to_the_semantic_workflow_graph() -> Result<(), Box<dyn Error>>
    {
        let workflow_document = workflow_document()?;
        let artifact = emit_quint_workflow_module(
            parse_quint_module_name("OpenTicket")?,
            parse_model_name("Open ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            parse_workflow_slug("open-ticket")?,
            WorkflowSliceDetails::from_details(workflow_slice_details()?),
            WorkflowTransitionRecords::from_records(workflow_transitions()?),
            workflow_digest()?,
        );

        assert_eq!(
            parse_quint_workflow_graph(&artifact)?,
            workflow_graph_from_document(parse_workflow_slug("open-ticket")?, workflow_document)?,
        );

        Ok(())
    }

    #[test]
    fn parsed_formal_graph_exposes_transition_drift() -> Result<(), Box<dyn Error>> {
        let artifact = emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            parse_model_name("Open ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            parse_workflow_slug("open-ticket")?,
            WorkflowSliceDetails::from_details(workflow_slice_details()?),
            WorkflowTransitionRecords::from_records([WorkflowTransitionRecord::new(
                WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
                WorkflowTransitionEndpoint::try_new("review-ticket".to_owned())?,
                WorkflowTransitionKind::try_new("navigation".to_owned())?,
                TransitionTriggerName::try_new("stale-screen".to_owned())?,
            )]),
            workflow_digest()?,
        );

        assert_ne!(
            parse_lean_workflow_graph(&artifact)?,
            workflow_graph_from_document(
                parse_workflow_slug("open-ticket")?,
                workflow_document()?
            )?,
        );

        Ok(())
    }

    fn workflow_digest() -> Result<ArtifactDigest, Box<dyn Error>> {
        Ok(artifact_digest(
            parse_model_name("Open ticket")?,
            parse_workflow_slug("open-ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            WorkflowSliceDetails::from_details(workflow_slice_details()?),
            WorkflowTransitionRecords::from_records(workflow_transitions()?),
        ))
    }

    fn workflow_slice_details() -> Result<Vec<WorkflowSliceDetail>, Box<dyn Error>> {
        Ok(vec![
            WorkflowSliceDetail::new(
                parse_slice_slug("capture-ticket")?,
                parse_model_name("Capture ticket")?,
                SliceKindName::try_new("state_view".to_owned())?,
                parse_model_description("Actor enters repair ticket details.")?,
            ),
            WorkflowSliceDetail::new(
                parse_slice_slug("review-ticket")?,
                parse_model_name("Review ticket")?,
                SliceKindName::try_new("state_view".to_owned())?,
                parse_model_description("Actor reviews the repair ticket.")?,
            ),
        ])
    }

    fn workflow_transitions() -> Result<Vec<WorkflowTransitionRecord>, Box<dyn Error>> {
        Ok(vec![WorkflowTransitionRecord::new(
            WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
            WorkflowTransitionEndpoint::try_new("review-ticket".to_owned())?,
            WorkflowTransitionKind::try_new("navigation".to_owned())?,
            TransitionTriggerName::try_new("review-ticket-screen".to_owned())?,
        )])
    }

    fn workflow_document() -> Result<FileContents, Box<dyn Error>> {
        Ok(FileContents::try_new(
            "{\n  \"name\": \"Open ticket\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Actor opens a repair ticket.\",\n  \"board\": {},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"capture-ticket\", \"name\": \"Capture ticket\", \"type\": \"state_view\", \"description\": \"Actor enters repair ticket details.\", \"relationship\": \"entry\", \"transitions\": [{\"to\": \"review-ticket\", \"via_navigation\": \"review-ticket-screen\"}]},\n    {\"slice\": \"review-ticket\", \"name\": \"Review ticket\", \"type\": \"state_view\", \"description\": \"Actor reviews the repair ticket.\", \"relationship\": \"main\"}\n  ]\n}\n".to_owned(),
        )?)
    }
}
