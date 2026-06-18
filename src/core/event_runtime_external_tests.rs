// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::super::{
        event_store_root, execute_eventcore_command_for_exported_event, list_forks,
        read_all_emc_events, reconcile_choose_branch, transaction_id_string,
    };
    use crate::core::connection::{ConnectionKind, WorkflowConnection, WorkflowTransitionRemoval};
    use crate::core::effect::{
        ArtifactDigest, ChosenEventId, EventConflictId, ModelContentDigest, ProjectionFingerprint,
        ReviewEventReference,
    };
    use crate::core::event_commands::EmcEvent;
    use crate::core::events::{EventDraft, resolve_event_conflict};
    use crate::core::formal_slice_facts::{
        CommandErrorNames, CommandInputProvenanceChain, CommandInputSource, EmittedEventNames,
        NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
        NewCommandDefinition, NewCommandInput, NewEventAttribute, NewEventDefinition,
        NewExternalPayloadDefinition, NewOutcomeDefinition, NewReadModelDefinition,
        NewReadModelField, NewSliceScenario, NewTranslationDefinition, NewViewDefinition,
        NewViewField, OutcomeEventNames, ReadModelFieldSource, ScenarioKind,
    };
    use crate::core::project::ProjectName;
    use crate::core::slice::{NewSlice, SliceKind};
    use crate::core::types::{
        AutomationName, AutomationReactionDescription, AutomationTriggerName, BitEncodingSemantics,
        BoardConnectionEndpoint, BoardConnectionEndpointKind, BoardElementDeclaredName,
        BoardElementKind, BoardElementName, BoardLaneId, CommandErrorName,
        CommandInputSourceDescription, CommandName, DataFlowSource, DataFlowSourceKind,
        DataFlowTarget, DatumName, EventAttributeName, EventAttributeSourceField,
        EventAttributeSourceKind, EventAttributeSourceName, EventName, ModelDescription, ModelName,
        OutcomeLabelName, PayloadContractName, ProvenanceDescription, ReadModelName,
        ReviewRuleName, ReviewTimestamp, ReviewerId, ScenarioName, ScenarioStepText, SketchToken,
        SliceKindName, SliceSlug, StreamName, TransformationSemantics, TransitionTriggerName,
        TranslationExternalEventName, TranslationName, ViewFieldName, ViewFieldSourceKind,
        ViewName, WorkflowCommandErrorRecord, WorkflowEntryLifecycleEvidenceText,
        WorkflowEntryLifecycleStateName, WorkflowEntryLifecycleStateRecord, WorkflowOutcomeRecord,
        WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord,
        WorkflowSliceDetail, WorkflowSlug, WorkflowTransitionEndpoint,
        WorkflowTransitionEvidenceRecord, WorkflowTransitionKind,
        WorkflowTransitionSourceEvidenceText, WorkflowTransitionTargetEvidenceText,
    };
    use crate::core::workflow::NewWorkflow;

    /// Copy every committed transaction file from one project's store into
    /// another's, mimicking a `git merge` of the `events/` directory. Only
    /// `events/*.jsonl` is committed, so this is the entire shared surface.
    fn union_committed_events(from: &Path, into: &Path) -> Result<(), Box<dyn Error>> {
        let source = event_store_root(from).join("events");
        let destination = event_store_root(into).join("events");
        fs::create_dir_all(&destination)?;
        for entry in fs::read_dir(source)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "jsonl")
            {
                let name = path.file_name().ok_or("transaction file has no name")?;
                fs::copy(&path, destination.join(name))?;
            }
        }
        Ok(())
    }

    fn project_name(value: &str) -> Result<ProjectName, Box<dyn Error>> {
        Ok(ProjectName::try_new(value.to_owned())?)
    }

    fn new_workflow(
        slug: &str,
        name: &str,
        description: &str,
    ) -> Result<NewWorkflow, Box<dyn Error>> {
        Ok(NewWorkflow::new(
            ModelName::try_new(name.to_owned())?,
            ModelDescription::try_new(description.to_owned())?,
            WorkflowSlug::try_new(slug.to_owned())?,
        ))
    }

    #[test]
    fn event_store_lives_inside_the_project_repository() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;

        let root = event_store_root(project.path());
        assert!(
            root.starts_with(project.path()),
            "the file event store must live in the project repository"
        );
        assert!(
            root.join("events").is_dir(),
            "the committed events/ directory must exist after the first append"
        );
        assert!(
            root.join(".gitignore").is_file(),
            "the store must write its own .gitignore"
        );
        Ok(())
    }

    #[test]
    fn store_gitignore_excludes_operational_files_but_commits_the_event_log()
    -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;

        let gitignore = fs::read_to_string(event_store_root(project.path()).join(".gitignore"))?;
        let ignored: Vec<&str> = gitignore
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        // Every operational artifact the runtime relies on staying out of git
        // must be ignored, so the committed history is exactly the event log.
        for operational in [
            "/tmp/",
            "/checkpoints/",
            "/locks/",
            "/index/",
            "/.eventcore/",
            "/.lock",
        ] {
            assert!(
                ignored.contains(&operational),
                "the store .gitignore must exclude {operational} but lists only {ignored:?}"
            );
        }

        // The committed events/ directory is the single source of truth and
        // must never be ignored.
        for committed in ["events/", "/events/", "events"] {
            assert!(
                !ignored.contains(&committed),
                "the store .gitignore must not ignore the committed event log ({committed})"
            );
        }
        Ok(())
    }

    #[test]
    fn executed_commands_are_appended_and_read_back_in_order() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_added(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            matches!(events.first(), Some(EmcEvent::ProjectInitialized { .. })),
            "first event must be ProjectInitialized, got {events:?}"
        );
        assert!(
            matches!(
                events.get(1),
                Some(EmcEvent::WorkflowAdded { slug, .. }) if slug.as_ref() == "open-ticket"
            ),
            "second event must be the added workflow, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn reading_a_project_without_a_store_returns_no_events() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        assert!(read_all_emc_events(project.path())?.is_empty());
        Ok(())
    }

    #[test]
    fn updating_an_absent_workflow_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;

        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        );

        assert!(
            result.is_err(),
            "updating a workflow that was never added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn concurrent_branches_are_reported_as_a_fork_and_reconciled_by_choosing_one()
    -> Result<(), Box<dyn Error>> {
        // Replica A builds the shared base: an initialized project with a
        // workflow.
        let replica_a = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::workflow_added(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        )?;

        // Replica B starts from a clone of the committed events only (a fresh
        // replica id), then both replicas update the workflow concurrently from
        // the same base version.
        let replica_b = TempDir::new()?;
        union_committed_events(replica_a.path(), replica_b.path())?;
        execute_eventcore_command_for_exported_event(
            replica_b.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Description authored on replica B.",
            )?),
        )?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Description authored on replica A.",
            )?),
        )?;

        // Merging replica B's events into replica A unions the two divergent
        // transactions onto one stream from the same base — a fork.
        union_committed_events(replica_b.path(), replica_a.path())?;
        let forks = list_forks(replica_a.path())?;
        assert_eq!(
            forks.len(),
            1,
            "the concurrent updates must surface one fork"
        );
        let fork = forks.first().ok_or("the single fork must be present")?;
        assert_eq!(fork.stream_id().as_ref(), "workflow::open-ticket");
        assert_eq!(
            fork.transactions().len(),
            2,
            "both branches must be present"
        );

        // Resolving keeps the chosen branch and collapses the fork.
        let chosen = transaction_id_string(
            *fork
                .transactions()
                .first()
                .ok_or("the fork must have at least one branch")?,
        );
        let resolved = reconcile_choose_branch(replica_a.path(), "workflow::open-ticket", &chosen)?;
        assert_eq!(resolved, 1, "reconcile must resolve the single fork");
        assert!(
            list_forks(replica_a.path())?.is_empty(),
            "no fork must remain after reconciliation"
        );
        Ok(())
    }

    #[test]
    fn resolving_a_conflict_records_a_replayable_conflict_resolved_event()
    -> Result<(), Box<dyn Error>> {
        // Build the same concurrent-update fork as the reconcile test.
        let replica_a = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::workflow_added(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        )?;

        let replica_b = TempDir::new()?;
        union_committed_events(replica_a.path(), replica_b.path())?;
        execute_eventcore_command_for_exported_event(
            replica_b.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Description authored on replica B.",
            )?),
        )?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Description authored on replica A.",
            )?),
        )?;
        union_committed_events(replica_b.path(), replica_a.path())?;

        let fork = list_forks(replica_a.path())?
            .into_iter()
            .next()
            .ok_or("the concurrent updates must surface one fork")?;
        let chosen = transaction_id_string(
            *fork
                .transactions()
                .first()
                .ok_or("the fork must have at least one branch")?,
        );

        // The conflict id is the forked stream; the chosen event id is the kept
        // branch's transaction id (how the CLI/MCP boundary models them).
        let conflict_id =
            EventConflictId::new(ArtifactDigest::try_new("workflow::open-ticket".to_owned())?);
        let chosen_event_id = ChosenEventId::new(ArtifactDigest::try_new(chosen)?);

        let plan = resolve_event_conflict(replica_a.path(), &conflict_id, &chosen_event_id)
            .map_err(|error| -> Box<dyn Error> { error.into() })?;
        assert_eq!(
            plan.effects().iter().count(),
            1,
            "resolution must report exactly once"
        );

        // The decision is now a replayable domain event in the log, not just an
        // eventcore-fs merge transaction.
        let events = read_all_emc_events(replica_a.path())?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::ConflictResolved { .. })),
            "resolving a conflict must record a replayable ConflictResolved event, got {events:?}"
        );
        assert!(
            list_forks(replica_a.path())?.is_empty(),
            "no fork must remain after resolution"
        );
        Ok(())
    }

    // ---------------------------------------------------------------------
    // Per-variant dispatch-boundary coverage.
    //
    // Each happy-path test seeds the preconditions its draft requires
    // (`project_initialized`, then `workflow_added`, then `slice_added`),
    // executes the draft through the same dispatch boundary, and asserts the
    // resulting `EmcEvent` variant appears in the appended log. Each rejection
    // test executes a draft without its precondition and asserts the command
    // invariant fails the write.
    // ---------------------------------------------------------------------

    const WORKFLOW_SLUG: &str = "open-ticket";
    const SLICE_SLUG: &str = "capture-ticket";

    /// Initialize a project so workflow streams can be added.
    fn seed_project(project: &Path) -> Result<(), Box<dyn Error>> {
        execute_eventcore_command_for_exported_event(
            project,
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;
        Ok(())
    }

    /// Initialize a project and add the `open-ticket` workflow.
    fn seed_workflow(project: &Path) -> Result<(), Box<dyn Error>> {
        seed_project(project)?;
        execute_eventcore_command_for_exported_event(
            project,
            &EventDraft::workflow_added(&new_workflow(
                WORKFLOW_SLUG,
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        )?;
        Ok(())
    }

    /// Initialize a project, add the workflow, and add the `capture-ticket`
    /// slice so slice facts have an existing stream to extend.
    fn seed_slice(project: &Path) -> Result<(), Box<dyn Error>> {
        seed_workflow(project)?;
        execute_eventcore_command_for_exported_event(
            project,
            &EventDraft::slice_added(&new_slice()?),
        )?;
        Ok(())
    }

    fn workflow_slug() -> Result<WorkflowSlug, Box<dyn Error>> {
        Ok(WorkflowSlug::try_new(WORKFLOW_SLUG.to_owned())?)
    }

    fn slice_slug() -> Result<SliceSlug, Box<dyn Error>> {
        Ok(SliceSlug::try_new(SLICE_SLUG.to_owned())?)
    }

    fn new_slice() -> Result<NewSlice, Box<dyn Error>> {
        Ok(NewSlice::new(
            workflow_slug()?,
            slice_slug()?,
            ModelName::try_new("Capture ticket".to_owned())?,
            ModelDescription::try_new("Capture the repair ticket details.".to_owned())?,
            SliceKind::state_change(),
        ))
    }

    /// A `WorkflowSliceDetail` for the seeded slice, used to drive slice
    /// update/remove drafts that take the projected detail rather than a
    /// `NewSlice`.
    fn slice_detail() -> Result<WorkflowSliceDetail, Box<dyn Error>> {
        Ok(WorkflowSliceDetail::new(
            slice_slug()?,
            ModelName::try_new("Capture ticket".to_owned())?,
            SliceKindName::try_new("state_change")?,
            ModelDescription::try_new("Capture the repair ticket details.".to_owned())?,
        ))
    }

    fn endpoint(value: &str) -> Result<WorkflowTransitionEndpoint, Box<dyn Error>> {
        Ok(WorkflowTransitionEndpoint::try_new(value.to_owned())?)
    }

    // --- Workflow lifecycle happy paths ---------------------------------

    #[test]
    fn removing_a_present_workflow_appends_workflow_removed() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_removed(&workflow_slug()?),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::WorkflowRemoved { slug, .. } if slug.as_ref() == WORKFLOW_SLUG
            )),
            "removing a workflow must append WorkflowRemoved, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn connecting_a_present_workflow_appends_workflow_connected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let connection = WorkflowConnection::new(
            workflow_slug()?,
            slice_slug()?,
            slice_slug()?,
            ConnectionKind::command(),
            TransitionTriggerName::try_new("CaptureTicket".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_connected(&connection),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::WorkflowConnected { .. })),
            "connecting a workflow must append WorkflowConnected, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn removing_a_workflow_transition_appends_workflow_transition_removed()
    -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let removal = WorkflowTransitionRemoval::new(
            workflow_slug()?,
            slice_slug()?,
            slice_slug()?,
            ConnectionKind::command(),
            TransitionTriggerName::try_new("CaptureTicket".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_transition_removed(&removal),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::WorkflowTransitionRemoved { .. })),
            "removing a transition must append WorkflowTransitionRemoved, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn declaring_readiness_for_a_present_workflow_appends_readiness_declared()
    -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let projection_fingerprint =
            ProjectionFingerprint::new(ArtifactDigest::try_new("fingerprint".to_owned())?);
        let model_content_digest =
            ModelContentDigest::new(ArtifactDigest::try_new("digest".to_owned())?);
        let verified_at = ReviewTimestamp::try_new("2026-06-18T00:00:00.000Z".to_owned())?;
        let verified_by = ReviewerId::try_new("reviewer".to_owned())?;
        let review_event = ReviewEventReference::unrecorded();

        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_readiness_declared(
                &workflow_slug()?,
                &projection_fingerprint,
                &model_content_digest,
                &verified_at,
                &verified_by,
                &review_event,
            ),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::WorkflowReadinessDeclared { .. })),
            "declaring readiness must append WorkflowReadinessDeclared, got {events:?}"
        );
        Ok(())
    }

    // --- Workflow fact happy paths --------------------------------------

    #[test]
    fn adding_a_workflow_outcome_appends_workflow_outcome_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let outcome = WorkflowOutcomeRecord::new(
            endpoint(SLICE_SLUG)?,
            OutcomeLabelName::try_new("TicketCaptured".to_owned())?,
            true,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_outcome_added(&workflow_slug()?, &outcome),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::WorkflowOutcomeAdded { label, .. } if label.as_ref() == "TicketCaptured"
            )),
            "adding a workflow outcome must append WorkflowOutcomeAdded, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn adding_a_workflow_command_error_appends_workflow_command_error_added()
    -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let error = WorkflowCommandErrorRecord::new(
            endpoint(SLICE_SLUG)?,
            CommandName::try_new("CaptureTicket".to_owned())?,
            CommandErrorName::try_new("DuplicateTicket".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_command_error_added(&workflow_slug()?, &error),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::WorkflowCommandErrorAdded { error, .. } if error.as_ref() == "DuplicateTicket"
            )),
            "adding a command error must append WorkflowCommandErrorAdded, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn adding_a_workflow_owned_definition_appends_workflow_owned_definition_added()
    -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let definition = WorkflowOwnedDefinitionRecord::new(
            endpoint(SLICE_SLUG)?,
            WorkflowOwnedDefinitionKind::try_new("command")?,
            WorkflowOwnedDefinitionName::try_new("CaptureTicket".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_owned_definition_added(&workflow_slug()?, &definition),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::WorkflowOwnedDefinitionAdded { definition_name, .. }
                    if definition_name.as_ref() == "CaptureTicket"
            )),
            "adding an owned definition must append WorkflowOwnedDefinitionAdded, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn adding_workflow_transition_evidence_appends_transition_evidence_added()
    -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let evidence = WorkflowTransitionEvidenceRecord::new(
            endpoint(SLICE_SLUG)?,
            endpoint(SLICE_SLUG)?,
            WorkflowTransitionKind::try_new("command")?,
            TransitionTriggerName::try_new("CaptureTicket".to_owned())?,
            WorkflowTransitionSourceEvidenceText::try_new("source evidence".to_owned())?,
            WorkflowTransitionTargetEvidenceText::try_new("target evidence".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_transition_evidence_added(&workflow_slug()?, &evidence),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::WorkflowTransitionEvidenceAdded { .. })),
            "adding transition evidence must append WorkflowTransitionEvidenceAdded, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn requiring_entry_lifecycle_coverage_appends_coverage_required() -> Result<(), Box<dyn Error>>
    {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_entry_lifecycle_coverage_required(&workflow_slug()?),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::WorkflowEntryLifecycleCoverageRequired { .. }
            )),
            "requiring coverage must append WorkflowEntryLifecycleCoverageRequired, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn adding_entry_lifecycle_state_appends_lifecycle_state_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let coverage = WorkflowEntryLifecycleStateRecord::new(
            WorkflowEntryLifecycleStateName::try_new("fresh_uninitialized")?,
            endpoint(SLICE_SLUG)?,
            WorkflowEntryLifecycleEvidenceText::try_new("lifecycle evidence".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_entry_lifecycle_state_added(&workflow_slug()?, &coverage),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::WorkflowEntryLifecycleStateAdded { .. })),
            "adding a lifecycle state must append WorkflowEntryLifecycleStateAdded, got {events:?}"
        );
        Ok(())
    }

    // --- Slice structural happy paths -----------------------------------

    #[test]
    fn adding_a_slice_appends_slice_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_added(&new_slice()?),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::SliceAdded { slug, .. } if slug.as_ref() == SLICE_SLUG
            )),
            "adding a slice must append SliceAdded, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn updating_a_present_slice_appends_slice_updated() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_updated(&slice_detail()?),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::SliceUpdated { slug, .. } if slug.as_ref() == SLICE_SLUG
            )),
            "updating a slice must append SliceUpdated, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn removing_a_present_slice_appends_slice_removed() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_removed(&slice_detail()?),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events.iter().any(|event| matches!(
                event,
                EmcEvent::SliceRemoved { slug, .. } if slug.as_ref() == SLICE_SLUG
            )),
            "removing a slice must append SliceRemoved, got {events:?}"
        );
        Ok(())
    }

    // --- Slice fact happy paths (all surface as SliceFactAdded) ----------

    #[test]
    fn adding_a_slice_scenario_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let scenario = NewSliceScenario::new(
            slice_slug()?,
            ScenarioKind::try_new("acceptance")?,
            ScenarioName::try_new("Ticket is captured".to_owned())?,
            ScenarioStepText::try_new("an actor with ticket details".to_owned())?,
            ScenarioStepText::try_new("the actor submits the ticket".to_owned())?,
            ScenarioStepText::try_new("the ticket is captured".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_scenario_added(&scenario),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_outcome_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let outcome = NewOutcomeDefinition::new(
            slice_slug()?,
            OutcomeLabelName::try_new("TicketCaptured".to_owned())?,
            OutcomeEventNames::from_events([EventName::try_new("TicketCaptured".to_owned())?]),
            true,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_outcome_added(&outcome),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_external_payload_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let external_payload = NewExternalPayloadDefinition::new(
            slice_slug()?,
            EventAttributeSourceName::try_new("TicketForm".to_owned())?,
            EventAttributeSourceField::try_new("title".to_owned())?,
            ProvenanceDescription::try_new("the submitted form title".to_owned())?,
            BitEncodingSemantics::try_new("UTF-8 string".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_external_payload_added(&external_payload),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_event_definition_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let attribute = NewEventAttribute::new(
            EventAttributeName::try_new("ticket_title".to_owned())?,
            EventAttributeSourceKind::try_new("command_input")?,
            EventAttributeSourceName::try_new("CaptureTicket".to_owned())?,
            EventAttributeSourceField::try_new("title".to_owned())?,
            ProvenanceDescription::try_new("captured from the command input".to_owned())?,
        );
        let event = NewEventDefinition::new(
            slice_slug()?,
            EventName::try_new("TicketCaptured".to_owned())?,
            StreamName::try_new("ticket".to_owned())?,
            attribute,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_event_definition_added(&event),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_command_definition_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let input = NewCommandInput::new(
            DatumName::try_new("ticket_title".to_owned())?,
            CommandInputSource::actor(),
            CommandInputSourceDescription::try_new("actor-provided ticket title".to_owned())?,
            CommandInputProvenanceChain::from_hops([]),
        );
        let command = NewCommandDefinition::new(
            slice_slug()?,
            CommandName::try_new("CaptureTicket".to_owned())?,
            input,
            EmittedEventNames::from_events([EventName::try_new("TicketCaptured".to_owned())?]),
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_command_definition_added(&command),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_read_model_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let field = NewReadModelField::new(
            DatumName::try_new("ticket_title".to_owned())?,
            ReadModelFieldSource::event_attribute(
                EventName::try_new("TicketCaptured".to_owned())?,
                EventAttributeName::try_new("ticket_title".to_owned())?,
            ),
            ProvenanceDescription::try_new("projected from the captured event".to_owned())?,
        );
        let read_model = NewReadModelDefinition::new(
            slice_slug()?,
            ReadModelName::try_new("TicketSummary".to_owned())?,
            field,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_read_model_added(&read_model),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_view_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let field = NewViewField::new(
            ViewFieldName::try_new("ticket_title".to_owned())?,
            ViewFieldSourceKind::try_new("read_model")?,
            ReadModelName::try_new("TicketSummary".to_owned())?,
            ViewFieldName::try_new("ticket_title".to_owned())?,
            SketchToken::try_new("title-field".to_owned())?,
            ProvenanceDescription::try_new("rendered from the read model".to_owned())?,
            BitEncodingSemantics::try_new("UTF-8 string".to_owned())?,
        );
        let view = NewViewDefinition::new(
            slice_slug()?,
            ViewName::try_new("TicketView".to_owned())?,
            field,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_view_added(&view),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_bit_level_data_flow_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let data_flow = NewBitLevelDataFlow::new(
            slice_slug()?,
            DatumName::try_new("ticket_title".to_owned())?,
            DataFlowSourceKind::try_new("original")?,
            DataFlowSource::try_new("actor input title field".to_owned())?,
            TransformationSemantics::try_new("identity")?,
            DataFlowTarget::try_new("Capture ticket.ticket_title".to_owned())?,
            BitEncodingSemantics::try_new("UTF-8 string".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_bit_level_data_flow_added(&data_flow),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_translation_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let translation = NewTranslationDefinition::new(
            slice_slug()?,
            TranslationName::try_new("TicketTranslation".to_owned())?,
            TranslationExternalEventName::try_new("ExternalTicketReceived".to_owned())?,
            PayloadContractName::try_new("TicketPayload".to_owned())?,
            CommandName::try_new("CaptureTicket".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_translation_added(&translation),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_automation_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let automation = NewAutomationDefinition::new(
            slice_slug()?,
            AutomationName::try_new("TicketAutomation".to_owned())?,
            AutomationTriggerName::try_new("TicketCaptured".to_owned())?,
            CommandName::try_new("NotifyDesk".to_owned())?,
            CommandErrorNames::empty(),
            AutomationReactionDescription::try_new("notify the repair desk".to_owned())?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_automation_added(&automation),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_board_element_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let element = NewBoardElement::new(
            slice_slug()?,
            BoardElementName::try_new("CaptureTicket".to_owned())?,
            BoardElementKind::try_new("command")?,
            BoardLaneId::try_new("actions")?,
            BoardElementDeclaredName::try_new("Capture ticket".to_owned())?,
            true,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_board_element_added(&element),
        )?;

        assert_slice_fact_appended(project.path())
    }

    #[test]
    fn adding_a_slice_board_connection_appends_slice_fact_added() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let connection = NewBoardConnection::new(
            slice_slug()?,
            BoardConnectionEndpoint::try_new("CaptureTicket".to_owned())?,
            BoardConnectionEndpointKind::try_new("command")?,
            BoardConnectionEndpoint::try_new("TicketCaptured".to_owned())?,
            BoardConnectionEndpointKind::try_new("event")?,
        );
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_board_connection_added(&connection),
        )?;

        assert_slice_fact_appended(project.path())
    }

    /// Every slice fact dispatches through `AddSliceFactCommand` and surfaces
    /// as a single `EmcEvent::SliceFactAdded` variant.
    fn assert_slice_fact_appended(project: &Path) -> Result<(), Box<dyn Error>> {
        let events = read_all_emc_events(project)?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::SliceFactAdded { .. })),
            "adding a slice fact must append SliceFactAdded, got {events:?}"
        );
        Ok(())
    }

    // --- Review happy path ----------------------------------------------

    #[test]
    fn recording_a_review_appends_review_recorded() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let model_content_digest =
            ModelContentDigest::new(ArtifactDigest::try_new("digest".to_owned())?);
        let reviewer_id = ReviewerId::try_new("reviewer".to_owned())?;
        let reviewed_at = ReviewTimestamp::try_new("2026-06-18T00:00:00.000Z".to_owned())?;
        let categories = [ReviewRuleName::try_new("scenario-coverage")?];

        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::review_recorded(
                &workflow_slug()?,
                &model_content_digest,
                &reviewer_id,
                &reviewed_at,
                &categories,
            ),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EmcEvent::ReviewRecorded { .. })),
            "recording a review must append ReviewRecorded, got {events:?}"
        );
        Ok(())
    }

    // --- Command-invariant rejections -----------------------------------

    #[test]
    fn removing_an_absent_workflow_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_project(project.path())?;

        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_removed(&workflow_slug()?),
        );

        assert!(
            result.is_err(),
            "removing a workflow that was never added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn connecting_an_absent_workflow_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_project(project.path())?;

        let connection = WorkflowConnection::new(
            workflow_slug()?,
            slice_slug()?,
            slice_slug()?,
            ConnectionKind::command(),
            TransitionTriggerName::try_new("CaptureTicket".to_owned())?,
        );
        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_connected(&connection),
        );

        assert!(
            result.is_err(),
            "connecting a workflow that was never added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn removing_a_transition_for_an_absent_workflow_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_project(project.path())?;

        let removal = WorkflowTransitionRemoval::new(
            workflow_slug()?,
            slice_slug()?,
            slice_slug()?,
            ConnectionKind::command(),
            TransitionTriggerName::try_new("CaptureTicket".to_owned())?,
        );
        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_transition_removed(&removal),
        );

        assert!(
            result.is_err(),
            "removing a transition for a workflow that was never added must fail the invariant"
        );
        Ok(())
    }

    #[test]
    fn adding_a_fact_to_an_absent_workflow_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_project(project.path())?;

        let outcome = WorkflowOutcomeRecord::new(
            endpoint(SLICE_SLUG)?,
            OutcomeLabelName::try_new("TicketCaptured".to_owned())?,
            true,
        );
        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_outcome_added(&workflow_slug()?, &outcome),
        );

        assert!(
            result.is_err(),
            "adding a fact to a workflow that was never added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn declaring_readiness_for_an_absent_workflow_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_project(project.path())?;

        let projection_fingerprint =
            ProjectionFingerprint::new(ArtifactDigest::try_new("fingerprint".to_owned())?);
        let model_content_digest =
            ModelContentDigest::new(ArtifactDigest::try_new("digest".to_owned())?);
        let verified_at = ReviewTimestamp::try_new("2026-06-18T00:00:00.000Z".to_owned())?;
        let verified_by = ReviewerId::try_new("reviewer".to_owned())?;
        let review_event = ReviewEventReference::unrecorded();

        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_readiness_declared(
                &workflow_slug()?,
                &projection_fingerprint,
                &model_content_digest,
                &verified_at,
                &verified_by,
                &review_event,
            ),
        );

        assert!(
            result.is_err(),
            "declaring readiness for a workflow that was never added must fail the invariant"
        );
        Ok(())
    }

    #[test]
    fn adding_a_slice_that_already_exists_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_slice(project.path())?;

        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_added(&new_slice()?),
        );

        assert!(
            result.is_err(),
            "adding a slice that was already added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn updating_an_absent_slice_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_updated(&slice_detail()?),
        );

        assert!(
            result.is_err(),
            "updating a slice that was never added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn removing_an_absent_slice_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_removed(&slice_detail()?),
        );

        assert!(
            result.is_err(),
            "removing a slice that was never added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn adding_a_fact_to_an_absent_slice_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        seed_workflow(project.path())?;

        let outcome = NewOutcomeDefinition::new(
            slice_slug()?,
            OutcomeLabelName::try_new("TicketCaptured".to_owned())?,
            OutcomeEventNames::from_events([EventName::try_new("TicketCaptured".to_owned())?]),
            true,
        );
        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::slice_outcome_added(&outcome),
        );

        assert!(
            result.is_err(),
            "adding a fact to a slice that was never added must fail the command invariant"
        );
        Ok(())
    }

    /// Every persisted transaction is a complete, self-describing envelope: the
    /// header carries the schema (format) version, the transaction id (one emc
    /// command is one transaction, so the transaction id is the command/group
    /// identity), and the parent transaction ids; each event carries an event
    /// id, stream id, stream version (ordinal), event type, and the typed
    /// payload. eventcore-fs owns this envelope; emc recovers the per-operation
    /// event type as the typed payload via `read_all_emc_events`.
    #[test]
    fn committed_transactions_carry_a_complete_self_describing_envelope()
    -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        // Span the project, workflow, and slice streams.
        seed_slice(project.path())?;

        let events_dir = event_store_root(project.path()).join("events");
        let mut headers = 0_usize;
        let mut envelopes = 0_usize;
        for entry in fs::read_dir(&events_dir)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "jsonl")
            {
                let contents = fs::read_to_string(&path)?;
                for line in contents.lines().filter(|line| !line.trim().is_empty()) {
                    let record: serde_json::Value = serde_json::from_str(line)?;
                    match record.get("record").and_then(serde_json::Value::as_str) {
                        Some("header") => {
                            for field in
                                ["format_version", "transaction_id", "parent_transaction_ids"]
                            {
                                assert!(
                                    record.get(field).is_some(),
                                    "transaction header must carry {field}: {line}"
                                );
                            }
                            headers += 1;
                        }
                        Some("event") => {
                            for field in [
                                "event_id",
                                "stream_id",
                                "stream_version",
                                "event_type",
                                "event_data",
                            ] {
                                assert!(
                                    record.get(field).is_some(),
                                    "event envelope must carry {field}: {line}"
                                );
                            }
                            envelopes += 1;
                        }
                        other => {
                            return Err(
                                format!("unexpected transaction record kind: {other:?}").into()
                            );
                        }
                    }
                }
            }
        }
        assert!(
            headers >= 3 && envelopes >= 3,
            "project, workflow, and slice operations must each persist an enveloped transaction \
             (headers={headers}, envelopes={envelopes})"
        );

        // emc recovers each per-operation event type as the typed payload.
        let typed = read_all_emc_events(project.path())?;
        assert!(
            typed
                .iter()
                .any(|event| matches!(event, EmcEvent::ProjectInitialized { .. }))
                && typed
                    .iter()
                    .any(|event| matches!(event, EmcEvent::WorkflowAdded { .. }))
                && typed
                    .iter()
                    .any(|event| matches!(event, EmcEvent::SliceAdded { .. })),
            "typed read-back must recover each per-operation event type, got {typed:?}"
        );
        Ok(())
    }
}
