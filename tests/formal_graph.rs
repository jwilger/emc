#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::digest::{WorkflowArtifactDigestInput, artifact_digest};
    use emc::core::effect::FileContents;
    use emc::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
    use emc::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
    use emc::core::formal_graph::{parse_lean_workflow_graph, parse_quint_workflow_graph};
    use emc::core::types::{
        SliceKindName, TransitionTriggerName, WorkflowCommandErrorRecords, WorkflowModuleData,
        WorkflowOutcomeRecords, WorkflowOwnedDefinitionRecords, WorkflowSliceDetail,
        WorkflowSliceDetails, WorkflowStepRelationshipName, WorkflowTransitionEndpoint,
        WorkflowTransitionKind, WorkflowTransitionRecord, WorkflowTransitionRecords,
    };
    use emc::io::dto::{
        parse_lean_module_name, parse_model_description, parse_model_name, parse_quint_module_name,
        parse_slice_slug, parse_workflow_slug,
    };

    #[test]
    fn lean_workflow_artifact_parses_to_the_semantic_workflow_graph() -> Result<(), Box<dyn Error>>
    {
        let artifact = emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_module_data(workflow_slice_details()?, workflow_transitions()?)?,
        );
        let expected = parse_quint_workflow_graph(&emit_quint_workflow_module(
            parse_quint_module_name("OpenTicket")?,
            workflow_module_data(workflow_slice_details()?, workflow_transitions()?)?,
        ))?;

        assert_eq!(parse_lean_workflow_graph(&artifact)?, expected);

        Ok(())
    }

    #[test]
    fn quint_workflow_artifact_parses_to_the_semantic_workflow_graph() -> Result<(), Box<dyn Error>>
    {
        let artifact = emit_quint_workflow_module(
            parse_quint_module_name("OpenTicket")?,
            workflow_module_data(workflow_slice_details()?, workflow_transitions()?)?,
        );
        let expected = parse_lean_workflow_graph(&emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_module_data(workflow_slice_details()?, workflow_transitions()?)?,
        ))?;

        assert_eq!(parse_quint_workflow_graph(&artifact)?, expected);

        Ok(())
    }

    #[test]
    fn parsed_formal_graph_exposes_transition_drift() -> Result<(), Box<dyn Error>> {
        let expected = parse_lean_workflow_graph(&emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_module_data(workflow_slice_details()?, workflow_transitions()?)?,
        ))?;
        let stale = emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_module_data(
                workflow_slice_details()?,
                vec![WorkflowTransitionRecord::new(
                    WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
                    WorkflowTransitionEndpoint::try_new("review-ticket".to_owned())?,
                    WorkflowTransitionKind::try_new("navigation".to_owned())?,
                    TransitionTriggerName::try_new("stale-screen".to_owned())?,
                )],
            )?,
        );

        assert_ne!(parse_lean_workflow_graph(&stale)?, expected);

        Ok(())
    }

    #[test]
    fn parsed_formal_graph_preserves_workflow_exit_rationale() -> Result<(), Box<dyn Error>> {
        let artifact_with_rationale = emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_module_data(workflow_slice_details()?, workflow_exit_transitions()?)?,
        );
        let artifact_without_rationale = emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_module_data(
                workflow_slice_details()?,
                workflow_exit_transitions_without_rationale()?,
            )?,
        );

        assert_ne!(
            parse_lean_workflow_graph(&artifact_with_rationale)?,
            parse_lean_workflow_graph(&artifact_without_rationale)?,
            "formal parser must preserve workflow-exit rationale"
        );

        Ok(())
    }

    #[test]
    fn parsed_formal_graph_rejects_malformed_transition_field_groups() -> Result<(), Box<dyn Error>>
    {
        let artifact = emit_lean_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            workflow_module_data(workflow_slice_details()?, workflow_transitions()?)?,
        );
        let malformed = artifact.as_ref().replace(
            "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", rationale := \"\", payloadContract := \"\" }]",
            "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\" }]",
        );

        assert!(
            parse_lean_workflow_graph(&FileContents::try_new(malformed)?).is_err(),
            "formal parser must reject transition declarations that do not match the current six-field transition record shape"
        );

        Ok(())
    }

    fn workflow_module_data(
        workflow_slice_details: Vec<WorkflowSliceDetail>,
        workflow_transitions: Vec<WorkflowTransitionRecord>,
    ) -> Result<WorkflowModuleData, Box<dyn Error>> {
        let workflow_name = parse_model_name("Open ticket")?;
        let workflow_slug = parse_workflow_slug("open-ticket")?;
        let workflow_description = parse_model_description("Actor opens a repair ticket.")?;
        let workflow_slice_details = WorkflowSliceDetails::from_details(workflow_slice_details);
        let workflow_transitions = WorkflowTransitionRecords::from_records(workflow_transitions);
        let workflow_outcomes = WorkflowOutcomeRecords::from_records([]);
        let workflow_command_errors = WorkflowCommandErrorRecords::from_records([]);
        let workflow_owned_definitions = WorkflowOwnedDefinitionRecords::from_records([]);
        Ok(WorkflowModuleData::new(
            workflow_name.clone(),
            workflow_description.clone(),
            workflow_slug.clone(),
            artifact_digest(WorkflowArtifactDigestInput {
                workflow_name,
                workflow_slug,
                workflow_description,
                workflow_slice_details: workflow_slice_details.clone(),
                workflow_transitions: workflow_transitions.clone(),
                workflow_outcomes: workflow_outcomes.clone(),
                workflow_command_errors: workflow_command_errors.clone(),
                workflow_owned_definitions: workflow_owned_definitions.clone(),
                workflow_transition_evidences: Default::default(),
                workflow_requires_entry_lifecycle_coverage: false,
                workflow_entry_lifecycle_states: Default::default(),
            }),
        )
        .with_slice_details(workflow_slice_details)
        .with_transitions(workflow_transitions)
        .with_outcomes(workflow_outcomes)
        .with_command_errors(workflow_command_errors)
        .with_owned_definitions(workflow_owned_definitions))
    }

    fn workflow_slice_details() -> Result<Vec<WorkflowSliceDetail>, Box<dyn Error>> {
        Ok(vec![
            WorkflowSliceDetail::new_with_relationship(
                parse_slice_slug("capture-ticket")?,
                parse_model_name("Capture ticket")?,
                SliceKindName::try_new("state_view".to_owned())?,
                parse_model_description("Actor enters repair ticket details.")?,
                WorkflowStepRelationshipName::try_new("entry".to_owned())?,
            ),
            WorkflowSliceDetail::new_with_relationship(
                parse_slice_slug("review-ticket")?,
                parse_model_name("Review ticket")?,
                SliceKindName::try_new("state_view".to_owned())?,
                parse_model_description("Actor reviews the repair ticket.")?,
                WorkflowStepRelationshipName::try_new("main".to_owned())?,
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

    fn workflow_exit_transitions() -> Result<Vec<WorkflowTransitionRecord>, Box<dyn Error>> {
        Ok(vec![WorkflowTransitionRecord::new_with_rationale(
            WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
            WorkflowTransitionEndpoint::try_new("repair-complete".to_owned())?,
            WorkflowTransitionKind::try_new("workflow_exit".to_owned())?,
            TransitionTriggerName::try_new("ticket_closed".to_owned())?,
            parse_model_description("Closed tickets continue to completion.")?,
        )])
    }

    fn workflow_exit_transitions_without_rationale()
    -> Result<Vec<WorkflowTransitionRecord>, Box<dyn Error>> {
        Ok(vec![WorkflowTransitionRecord::new(
            WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
            WorkflowTransitionEndpoint::try_new("repair-complete".to_owned())?,
            WorkflowTransitionKind::try_new("workflow_exit".to_owned())?,
            TransitionTriggerName::try_new("ticket_closed".to_owned())?,
        )])
    }
}
