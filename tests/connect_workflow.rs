// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;
    use std::path::Path;
    use std::path::PathBuf;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn connect_workflow_adds_navigation_transition_to_canonical_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_complete_state_view_facts(temp_dir.path(), "capture-ticket")?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;
        add_complete_state_view_facts(temp_dir.path(), "review-ticket")?;

        let initial_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let initial_digest = digest_marker(&initial_lean)
            .ok_or("generated workflow artifact is missing an initial digest")?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to review-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "Lean artifact must represent the workflow transition"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", rationale: \"\", payloadContract: \"\" }]"
            ),
            "Quint artifact must represent the workflow transition"
        );
        assert_ne!(
            initial_digest,
            digest_marker(&lean).ok_or("Lean artifact is missing an updated digest")?,
            "Lean digest must change when workflow transitions change"
        );
        assert_ne!(
            initial_digest,
            digest_marker(&quint).ok_or("Quint artifact is missing an updated digest")?,
            "Quint digest must change when workflow transitions change"
        );

        Ok(())
    }

    #[test]
    fn add_workflow_outcome_updates_canonical_workflow_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        let initial_digest = digest_marker(&read_to_string(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
        )?)
        .ok_or("Lean artifact is missing its initial digest")?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--externally-relevant",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added workflow outcome ticket_captured to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let updated_digest = digest_marker(&lean).ok_or("Lean artifact is missing digest")?;

        assert!(lean.contains(
            "def workflowOutcomes : List WorkflowOutcome := [{ sourceSlice := \"capture-ticket\", label := \"ticket_captured\", externallyRelevant := true }]"
        ));
        assert!(quint.contains(
            "val workflowOutcomes: List[WorkflowOutcome] = [{ sourceSlice: \"capture-ticket\", label: \"ticket_captured\", externallyRelevant: true }]"
        ));
        assert_ne!(
            initial_digest, updated_digest,
            "workflow digest must change when an outcome fact is authored"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_outcome_rewrites_canonical_workflow_artifacts() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        initialize_project_with_open_ticket_workflow(temp_dir.path())?;
        add_workflow_outcome(temp_dir.path(), "ticket_captured", true)?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--externally-relevant",
                "true",
                "--new-source-slice",
                "capture-ticket",
                "--new-label",
                "ticket_ready",
                "--new-externally-relevant",
                "false",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated workflow outcome ticket_captured on workflow open-ticket",
            ));

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert!(
            lean.contains(
                "def workflowOutcomes : List WorkflowOutcome := [{ sourceSlice := \"capture-ticket\", label := \"ticket_ready\", externallyRelevant := false }]"
            ),
            "updated workflow outcome must be represented in Lean workflow artifacts"
        );
        assert!(
            !lean.contains("label := \"ticket_captured\""),
            "previous workflow outcome must be absent from Lean workflow artifacts"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_outcome_removes_it_from_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_open_ticket_workflow(temp_dir.path())?;
        add_workflow_outcome(temp_dir.path(), "ticket_captured", true)?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--externally-relevant",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed workflow outcome ticket_captured from workflow open-ticket",
            ));

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(
            quint.contains("val workflowOutcomes: List[WorkflowOutcome] = []"),
            "removed workflow outcome must be absent from Quint workflow artifacts"
        );

        Ok(())
    }

    #[test]
    fn add_workflow_command_error_updates_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        let initial_digest = digest_marker(&read_to_string(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
        )?)
        .ok_or("Lean artifact is missing its initial digest")?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                "CaptureTicket",
                "--error",
                "DuplicateTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added workflow command error DuplicateTicket to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let updated_digest = digest_marker(&lean).ok_or("Lean artifact is missing digest")?;

        assert!(lean.contains(
            "def workflowCommandErrors : List WorkflowCommandError := [{ sourceSlice := \"capture-ticket\", commandName := \"CaptureTicket\", errorName := \"DuplicateTicket\" }]"
        ));
        assert!(quint.contains(
            "val workflowCommandErrors: List[WorkflowCommandError] = [{ sourceSlice: \"capture-ticket\", commandName: \"CaptureTicket\", errorName: \"DuplicateTicket\" }]"
        ));
        assert_ne!(
            initial_digest, updated_digest,
            "workflow digest must change when a command error fact is authored"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_command_error_rewrites_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_open_ticket_workflow(temp_dir.path())?;
        add_workflow_command_error(temp_dir.path(), "CaptureTicket", "DuplicateTicket")?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                "CaptureTicket",
                "--error",
                "DuplicateTicket",
                "--new-source-slice",
                "capture-ticket",
                "--new-command",
                "SubmitTicket",
                "--new-error",
                "TicketAlreadySubmitted",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated workflow command error DuplicateTicket on workflow open-ticket",
            ));

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert!(lean.contains(
            "def workflowCommandErrors : List WorkflowCommandError := [{ sourceSlice := \"capture-ticket\", commandName := \"SubmitTicket\", errorName := \"TicketAlreadySubmitted\" }]"
        ));
        assert!(
            !lean.contains("DuplicateTicket"),
            "updated workflow command error must replace the previous error"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_command_error_removes_it_from_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_open_ticket_workflow(temp_dir.path())?;
        add_workflow_command_error(temp_dir.path(), "CaptureTicket", "DuplicateTicket")?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                "CaptureTicket",
                "--error",
                "DuplicateTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed workflow command error DuplicateTicket from workflow open-ticket",
            ));

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(
            quint.contains("val workflowCommandErrors: List[WorkflowCommandError] = []"),
            "removed workflow command error must be absent from Quint workflow artifacts"
        );

        Ok(())
    }

    #[test]
    fn add_workflow_owned_definition_updates_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        let initial_digest = digest_marker(&read_to_string(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
        )?)
        .ok_or("Lean artifact is missing its initial digest")?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-owned-definition",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--definition-kind",
                "command",
                "--definition-name",
                "CaptureTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added workflow owned definition command CaptureTicket to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let updated_digest = digest_marker(&lean).ok_or("Lean artifact is missing digest")?;

        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.command, definitionName := \"CaptureTicket\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: OwnedCommand, definitionName: \"CaptureTicket\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" }]"
        ));
        assert_ne!(
            initial_digest, updated_digest,
            "workflow digest must change when an owned definition fact is authored"
        );

        Ok(())
    }

    #[test]
    fn add_workflow_owned_definition_rejects_view_role_for_non_view_definitions()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-owned-definition",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--definition-kind",
                "command",
                "--definition-name",
                "CaptureTicket",
                "--view-role",
                "entry",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "view_role requires definition_kind view",
            ));

        Ok(())
    }

    #[test]
    fn update_workflow_owned_definition_rewrites_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_open_ticket_workflow(temp_dir.path())?;
        add_workflow_owned_definition(
            temp_dir.path(),
            "open-ticket",
            "capture-ticket",
            "command",
            "CaptureTicket",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow-owned-definition",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--definition-kind",
                "command",
                "--definition-name",
                "CaptureTicket",
                "--new-source-slice",
                "capture-ticket",
                "--new-definition-kind",
                "command",
                "--new-definition-name",
                "SubmitTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated workflow owned definition command CaptureTicket on workflow open-ticket",
            ));

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.command, definitionName := \"SubmitTicket\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" }]"
        ));
        assert!(
            !lean.contains("definitionName := \"CaptureTicket\""),
            "updated workflow owned definition must replace the previous definition"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_owned_definition_removes_it_from_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_open_ticket_workflow(temp_dir.path())?;
        add_workflow_owned_definition(
            temp_dir.path(),
            "open-ticket",
            "capture-ticket",
            "command",
            "CaptureTicket",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "workflow-owned-definition",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--definition-kind",
                "command",
                "--definition-name",
                "CaptureTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed workflow owned definition command CaptureTicket from workflow open-ticket",
            ));

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(
            quint.contains("val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = []"),
            "removed workflow owned definition must be absent from Quint workflow artifacts"
        );

        Ok(())
    }

    fn setup_navigation_transition_workflow(cwd: &Path) -> Result<String, Box<dyn Error>> {
        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        add_complete_state_view_facts(cwd, "capture-ticket")?;
        add_review_ticket_slice(cwd)?;
        add_complete_state_view_facts(cwd, "review-ticket")?;
        let initial_digest = initial_lean_digest(cwd)?;
        connect_navigation_review_ticket_screen(cwd)?;
        Ok(initial_digest)
    }

    fn add_navigation_transition_evidence_and_ownership(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc_stdout(
            cwd,
            &[
                "add",
                "workflow-transition-evidence",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
            ],
            "added workflow transition evidence navigation review-ticket-screen to workflow open-ticket",
        )?;

        add_workflow_owned_definition(
            cwd,
            "open-ticket",
            "capture-ticket",
            "control",
            "review-ticket-screen",
        )?;
        add_workflow_owned_entry_view_definition(
            cwd,
            "open-ticket",
            "review-ticket",
            "review-ticket-screen",
        )
    }

    fn assert_navigation_transition_evidence_artifacts(lean: &str, quint: &str) {
        assert!(lean.contains(
            "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(quint.contains(
            "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.control, definitionName := \"review-ticket-screen\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" },{ sourceSlice := \"review-ticket\", definitionKind := WorkflowOwnedDefinitionKind.view, definitionName := \"review-ticket-screen\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"entry\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: OwnedControl, definitionName: \"review-ticket-screen\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" },{ sourceSlice: \"review-ticket\", definitionKind: OwnedView, definitionName: \"review-ticket-screen\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"entry\" }]"
        ));
        assert!(lean.contains(
            "def workflowNavigationTransitionTargetsEntryView (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.navigation || workflowOwnsEntryView transition.target (workflowNavigationTargetView transition)"
        ));
        assert!(lean.contains(
            "theorem workflowNavigationTransitionsResolveToEntryViewsIsStable : workflowNavigationTransitionsResolveToEntryViews = true := by native_decide"
        ));
        assert!(quint.contains(
            "val workflowNavigationTransitionsResolveToEntryViews = workflowTransitions.select(transition => workflowNavigationTransitionTargetsEntryView(transition)).length() == workflowTransitions.length()"
        ));
    }

    #[test]
    fn add_workflow_transition_evidence_updates_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        let initial_digest = setup_navigation_transition_workflow(cwd)?;
        add_navigation_transition_evidence_and_ownership(cwd)?;

        let lean = read_lean(cwd)?;
        let quint = read_quint(cwd)?;
        let updated_digest = digest_marker(&lean).ok_or("Lean artifact is missing digest")?;

        assert_navigation_transition_evidence_artifacts(&lean, &quint);
        assert_ne!(
            initial_digest, updated_digest,
            "workflow digest must change when transition evidence and ownership facts are authored"
        );

        run_emc(cwd, &["check"])?;
        Ok(())
    }

    #[test]
    fn update_workflow_transition_evidence_rewrites_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();
        setup_navigation_transition_workflow(cwd)?;
        add_navigation_transition_evidence_and_ownership(cwd)?;

        run_emc_stdout(
            cwd,
            &[
                "update",
                "workflow-transition-evidence",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
                "--new-from",
                "capture-ticket",
                "--new-to",
                "review-ticket",
                "--new-via",
                "navigation",
                "--new-name",
                "review-ticket-screen",
                "--new-source-control",
                "review-ticket-screen",
                "--new-target-view",
                "review-ticket-screen",
                "--new-source-evidence",
                "capture-ticket control is the modeled navigation source",
                "--new-target-evidence",
                "review-ticket entry view is the modeled navigation target",
            ],
            "updated workflow transition evidence navigation review-ticket-screen on workflow open-ticket",
        )?;

        run_emc(cwd, &["check"])?;

        let lean = read_lean(cwd)?;
        assert!(lean.contains(
            "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", sourceEvidence := \"capture-ticket control is the modeled navigation source\", targetEvidence := \"review-ticket entry view is the modeled navigation target\" }]"
        ));
        assert!(
            !lean.contains("capture-ticket view owns the review-ticket-screen navigation control"),
            "updated workflow transition evidence must replace the previous evidence text"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_transition_evidence_removes_it_from_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();
        setup_navigation_transition_workflow(cwd)?;
        add_navigation_transition_evidence_and_ownership(cwd)?;

        run_emc_stdout(
            cwd,
            &[
                "remove",
                "workflow-transition-evidence",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
            ],
            "removed workflow transition evidence navigation review-ticket-screen from workflow open-ticket",
        )?;

        run_emc(cwd, &["check"])?;

        let quint = read_quint(cwd)?;
        assert!(
            quint
                .contains("val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = []"),
            "removed workflow transition evidence must be absent from Quint workflow artifacts"
        );

        Ok(())
    }

    fn setup_application_entry_workflow(cwd: &Path) -> Result<String, Box<dyn Error>> {
        init_repair_desk(cwd)?;
        run_emc(
            cwd,
            &[
                "add",
                "workflow",
                "--slug",
                "application-entry",
                "--name",
                "Application entry",
                "--description",
                "Actor enters the application.",
            ],
        )?;
        run_emc(
            cwd,
            &[
                "add",
                "slice",
                "--workflow",
                "application-entry",
                "--slug",
                "entry-state",
                "--name",
                "Entry state",
                "--type",
                "state_view",
                "--description",
                "Actor sees the correct entry state.",
            ],
        )?;
        add_complete_state_view_facts(cwd, "entry-state")?;
        digest_marker(&read_to_string(
            cwd.join("model/lean/ApplicationEntry.lean"),
        )?)
        .ok_or_else(|| "Lean artifact is missing its initial digest".into())
    }

    fn add_all_entry_lifecycle_states(cwd: &Path) -> Result<(), Box<dyn Error>> {
        add_entry_lifecycle_state(
            cwd,
            "fresh_uninitialized",
            "entry-state",
            "entry-state view distinguishes first arrival before initialization",
        )?;
        add_entry_lifecycle_state(
            cwd,
            "initialized_unauthenticated",
            "entry-state",
            "entry-state view distinguishes initialized unauthenticated sessions",
        )?;
        add_entry_lifecycle_state(
            cwd,
            "initialized_authenticated",
            "entry-state",
            "entry-state view distinguishes initialized authenticated sessions",
        )?;
        add_entry_lifecycle_state(
            cwd,
            "partially_configured",
            "entry-state",
            "entry-state view distinguishes partially configured accounts",
        )?;
        add_entry_lifecycle_state(
            cwd,
            "fully_configured",
            "entry-state",
            "entry-state view distinguishes fully configured accounts",
        )
    }

    fn assert_entry_lifecycle_artifacts(lean: &str, quint: &str) {
        assert!(lean.contains("def workflowRequiresEntryLifecycleCoverage : Bool := true"));
        assert!(quint.contains("val workflowRequiresEntryLifecycleCoverage = true"));
        assert!(lean.contains(
            "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := [{ state := WorkflowEntryLifecycleStateName.freshUninitialized, step := \"entry-state\", evidence := \"entry-state view distinguishes first arrival before initialization\" },{ state := WorkflowEntryLifecycleStateName.initializedUnauthenticated, step := \"entry-state\", evidence := \"entry-state view distinguishes initialized unauthenticated sessions\" },{ state := WorkflowEntryLifecycleStateName.initializedAuthenticated, step := \"entry-state\", evidence := \"entry-state view distinguishes initialized authenticated sessions\" },{ state := WorkflowEntryLifecycleStateName.partiallyConfigured, step := \"entry-state\", evidence := \"entry-state view distinguishes partially configured accounts\" },{ state := WorkflowEntryLifecycleStateName.fullyConfigured, step := \"entry-state\", evidence := \"entry-state view distinguishes fully configured accounts\" }]"
        ));
        assert!(quint.contains(
            "val workflowEntryLifecycleStates: List[WorkflowEntryLifecycleState] = [{ state: FreshUninitialized, step: \"entry-state\", evidence: \"entry-state view distinguishes first arrival before initialization\" },{ state: InitializedUnauthenticated, step: \"entry-state\", evidence: \"entry-state view distinguishes initialized unauthenticated sessions\" },{ state: InitializedAuthenticated, step: \"entry-state\", evidence: \"entry-state view distinguishes initialized authenticated sessions\" },{ state: PartiallyConfigured, step: \"entry-state\", evidence: \"entry-state view distinguishes partially configured accounts\" },{ state: FullyConfigured, step: \"entry-state\", evidence: \"entry-state view distinguishes fully configured accounts\" }]"
        ));
    }

    #[test]
    fn add_workflow_entry_lifecycle_coverage_updates_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        let initial_digest = setup_application_entry_workflow(cwd)?;

        run_emc_stdout(
            cwd,
            &[
                "mark",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "application-entry",
            ],
            "marked workflow application-entry as requiring entry lifecycle coverage",
        )?;

        add_all_entry_lifecycle_states(cwd)?;

        let lean = read_to_string(cwd.join("model/lean/ApplicationEntry.lean"))?;
        let quint = read_to_string(cwd.join("model/quint/ApplicationEntry.qnt"))?;
        let updated_digest = digest_marker(&lean).ok_or("Lean artifact is missing digest")?;

        assert_entry_lifecycle_artifacts(&lean, &quint);
        assert_ne!(
            initial_digest, updated_digest,
            "workflow digest must change when entry lifecycle coverage is authored"
        );

        run_emc(cwd, &["check"])?;
        run_emc(cwd, &["verify"])?;
        Ok(())
    }

    #[test]
    fn update_workflow_entry_lifecycle_state_rewrites_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        setup_application_entry_workflow(cwd)?;
        run_emc(
            cwd,
            &[
                "mark",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "application-entry",
            ],
        )?;
        add_all_entry_lifecycle_states(cwd)?;

        run_emc_stdout(
            cwd,
            &[
                "update",
                "workflow-entry-lifecycle-state",
                "--workflow",
                "application-entry",
                "--state",
                "fully_configured",
                "--step",
                "entry-state",
                "--evidence",
                "entry-state view distinguishes fully configured accounts",
                "--new-state",
                "fully_configured",
                "--new-step",
                "entry-state",
                "--new-evidence",
                "entry-state view confirms configuration completion",
            ],
            "updated workflow entry lifecycle state fully_configured on workflow application-entry",
        )?;

        let lean = read_to_string(cwd.join("model/lean/ApplicationEntry.lean"))?;
        let quint = read_to_string(cwd.join("model/quint/ApplicationEntry.qnt"))?;

        assert!(
            lean.contains("evidence := \"entry-state view confirms configuration completion\"")
        );
        assert!(quint.contains("evidence: \"entry-state view confirms configuration completion\""));
        assert!(
            !lean.contains("entry-state view distinguishes fully configured accounts"),
            "updated workflow entry lifecycle evidence must replace the old Lean value"
        );
        assert!(
            !quint.contains("entry-state view distinguishes fully configured accounts"),
            "updated workflow entry lifecycle evidence must replace the old Quint value"
        );

        run_emc(cwd, &["check"])?;
        run_emc(cwd, &["verify"])?;
        Ok(())
    }

    #[test]
    fn remove_workflow_entry_lifecycle_coverage_and_state_rewrites_canonical_workflow_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        setup_application_entry_workflow(cwd)?;
        run_emc(
            cwd,
            &[
                "mark",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "application-entry",
            ],
        )?;
        add_entry_lifecycle_state(
            cwd,
            "fresh_uninitialized",
            "entry-state",
            "entry-state view distinguishes first arrival before initialization",
        )?;

        run_emc_stdout(
            cwd,
            &[
                "remove",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "application-entry",
            ],
            "removed workflow entry lifecycle coverage requirement from workflow application-entry",
        )?;
        run_emc_stdout(
            cwd,
            &[
                "remove",
                "workflow-entry-lifecycle-state",
                "--workflow",
                "application-entry",
                "--state",
                "fresh_uninitialized",
                "--step",
                "entry-state",
                "--evidence",
                "entry-state view distinguishes first arrival before initialization",
            ],
            "removed workflow entry lifecycle state fresh_uninitialized from workflow application-entry",
        )?;

        let lean = read_to_string(cwd.join("model/lean/ApplicationEntry.lean"))?;
        let quint = read_to_string(cwd.join("model/quint/ApplicationEntry.qnt"))?;

        assert!(lean.contains("def workflowRequiresEntryLifecycleCoverage : Bool := false"));
        assert!(quint.contains("val workflowRequiresEntryLifecycleCoverage = false"));
        assert!(
            lean.contains(
                "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := []"
            )
        );
        assert!(
            quint.contains(
                "val workflowEntryLifecycleStates: List[WorkflowEntryLifecycleState] = []"
            )
        );

        run_emc(cwd, &["check"])?;
        run_emc(cwd, &["verify"])?;
        Ok(())
    }

    #[test]
    fn check_accepts_synchronized_formal_workflow_outcome_and_error_facts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                "draft_saved",
                "--externally-relevant",
                "false",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                "CaptureTicket",
                "--error",
                "DuplicateTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("project layout is complete"));

        Ok(())
    }

    fn add_ticket_captured_outcome(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--externally-relevant",
                "true",
            ],
        )
    }

    fn add_duplicate_ticket_command_error(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                "CaptureTicket",
                "--error",
                "DuplicateTicket",
            ],
        )
    }

    fn add_navigation_review_ticket_screen_evidence(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "workflow-transition-evidence",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
            ],
        )
    }

    fn assert_preserved_workflow_facts(lean: &str, quint: &str) {
        assert!(lean.contains(
            "def workflowOutcomes : List WorkflowOutcome := [{ sourceSlice := \"capture-ticket\", label := \"ticket_captured\", externallyRelevant := true }]"
        ));
        assert!(quint.contains(
            "val workflowOutcomes: List[WorkflowOutcome] = [{ sourceSlice: \"capture-ticket\", label: \"ticket_captured\", externallyRelevant: true }]"
        ));
        assert!(lean.contains(
            "def workflowCommandErrors : List WorkflowCommandError := [{ sourceSlice := \"capture-ticket\", commandName := \"CaptureTicket\", errorName := \"DuplicateTicket\" }]"
        ));
        assert!(quint.contains(
            "val workflowCommandErrors: List[WorkflowCommandError] = [{ sourceSlice: \"capture-ticket\", commandName: \"CaptureTicket\", errorName: \"DuplicateTicket\" }]"
        ));
        assert!(lean.contains(
            "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(quint.contains(
            "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
    }

    #[test]
    fn workflow_updates_preserve_authored_workflow_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        add_review_ticket_slice(cwd)?;

        add_ticket_captured_outcome(cwd)?;
        add_duplicate_ticket_command_error(cwd)?;
        connect_navigation_review_ticket_screen(cwd)?;
        add_navigation_review_ticket_screen_evidence(cwd)?;

        run_emc(
            cwd,
            &[
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens and reviews a repair ticket.",
            ],
        )?;

        let lean = read_lean(cwd)?;
        let quint = read_quint(cwd)?;

        assert_preserved_workflow_facts(&lean, &quint);

        Ok(())
    }

    fn setup_command_event_transition_slices(cwd: &Path) -> Result<(), Box<dyn Error>> {
        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        add_complete_state_view_facts(cwd, "capture-ticket")?;
        add_slice(
            cwd,
            "submit-ticket",
            "Submit ticket",
            "Actor submits repair ticket details.",
        )?;
        add_complete_state_change_facts(cwd, "submit-ticket")?;
        run_emc(cwd, &["check"])?;
        add_review_ticket_slice(cwd)?;
        run_emc(cwd, &["check"])?;
        add_review_state_view_facts(cwd)
    }

    fn connect_command_and_event_transitions(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc_stdout(
            cwd,
            &[
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "submit-ticket",
                "--via",
                "command",
                "--name",
                "SubmitTicketForReview",
            ],
            "connected capture-ticket to submit-ticket",
        )?;
        run_emc_stdout(
            cwd,
            &[
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "submit-ticket",
                "--to",
                "review-ticket",
                "--via",
                "event",
                "--name",
                "TicketSubmittedForReview",
            ],
            "connected submit-ticket to review-ticket",
        )
    }

    fn add_command_event_transition_evidence_and_ownership(
        cwd: &Path,
    ) -> Result<(), Box<dyn Error>> {
        add_workflow_transition_evidence(
            cwd,
            &WorkflowTransitionEvidenceFixture {
                workflow: "open-ticket",
                source: "capture-ticket",
                target: "submit-ticket",
                kind: "command",
                trigger: "SubmitTicketForReview",
                source_evidence: "capture-ticket view owns the SubmitTicketForReview control",
                target_evidence: "submit-ticket owns the SubmitTicketForReview command",
            },
        )?;
        add_workflow_owned_definition(
            cwd,
            "open-ticket",
            "capture-ticket",
            "control",
            "SubmitTicketForReview",
        )?;
        add_workflow_owned_definition(
            cwd,
            "open-ticket",
            "submit-ticket",
            "command",
            "SubmitTicketForReview",
        )?;
        add_workflow_transition_evidence(
            cwd,
            &WorkflowTransitionEvidenceFixture {
                workflow: "open-ticket",
                source: "submit-ticket",
                target: "review-ticket",
                kind: "event",
                trigger: "TicketSubmittedForReview",
                source_evidence: "submit-ticket emits TicketSubmittedForReview",
                target_evidence: "review-ticket observes TicketSubmittedForReview",
            },
        )?;
        add_workflow_owned_event_definition(
            cwd,
            "open-ticket",
            "submit-ticket",
            "TicketSubmittedForReview",
            "tickets",
            "SubmitTicketForReview command output",
            "emitted",
        )?;
        add_workflow_owned_event_definition(
            cwd,
            "open-ticket",
            "review-ticket",
            "TicketSubmittedForReview",
            "tickets",
            "SubmitTicketForReview command output",
            "observed",
        )
    }

    fn assert_command_event_transition_artifacts(lean: &str, quint: &str) {
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := WorkflowTransitionKind.command, trigger := \"SubmitTicketForReview\", sourceControl := \"\", targetView := \"\", rationale := \"\", payloadContract := \"\" },{ source := \"submit-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.event, trigger := \"TicketSubmittedForReview\", sourceControl := \"\", targetView := \"\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "Lean artifact must represent command and event workflow transitions"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: Command, trigger: \"SubmitTicketForReview\", sourceControl: \"\", targetView: \"\", rationale: \"\", payloadContract: \"\" },{ source: \"submit-ticket\", target: \"review-ticket\", kind: Event, trigger: \"TicketSubmittedForReview\", sourceControl: \"\", targetView: \"\", rationale: \"\", payloadContract: \"\" }]"
            ),
            "Quint artifact must represent command and event workflow transitions"
        );
        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.control, definitionName := \"SubmitTicketForReview\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" },{ sourceSlice := \"submit-ticket\", definitionKind := WorkflowOwnedDefinitionKind.command, definitionName := \"SubmitTicketForReview\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" },{ sourceSlice := \"submit-ticket\", definitionKind := WorkflowOwnedDefinitionKind.event, definitionName := \"TicketSubmittedForReview\", definitionStream := \"tickets\", sourceProvenance := \"SubmitTicketForReview command output\", eventParticipation := \"emitted\", viewRole := \"\" },{ sourceSlice := \"review-ticket\", definitionKind := WorkflowOwnedDefinitionKind.event, definitionName := \"TicketSubmittedForReview\", definitionStream := \"tickets\", sourceProvenance := \"SubmitTicketForReview command output\", eventParticipation := \"observed\", viewRole := \"\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: OwnedControl, definitionName: \"SubmitTicketForReview\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" },{ sourceSlice: \"submit-ticket\", definitionKind: OwnedCommand, definitionName: \"SubmitTicketForReview\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" },{ sourceSlice: \"submit-ticket\", definitionKind: OwnedEvent, definitionName: \"TicketSubmittedForReview\", definitionStream: \"tickets\", sourceProvenance: \"SubmitTicketForReview command output\", eventParticipation: \"emitted\", viewRole: \"\" },{ sourceSlice: \"review-ticket\", definitionKind: OwnedEvent, definitionName: \"TicketSubmittedForReview\", definitionStream: \"tickets\", sourceProvenance: \"SubmitTicketForReview command output\", eventParticipation: \"observed\", viewRole: \"\" }]"
        ));
    }

    #[test]
    fn connect_workflow_adds_command_and_event_transitions_to_canonical_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        setup_command_event_transition_slices(cwd)?;
        connect_command_and_event_transitions(cwd)?;
        add_command_event_transition_evidence_and_ownership(cwd)?;

        let lean = read_lean(cwd)?;
        let quint = read_quint(cwd)?;

        assert_command_event_transition_artifacts(&lean, &quint);

        Ok(())
    }

    fn connect_alternate_navigation_review_ticket_screen(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "alternate-review-ticket-screen",
                "--source-control",
                "alternate-review-ticket-screen",
                "--target-view",
                "alternate-review-ticket-screen",
            ],
        )
    }

    #[test]
    fn remove_transition_removes_modeled_transition_from_canonical_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        add_review_ticket_slice(cwd)?;

        connect_navigation_review_ticket_screen(cwd)?;
        connect_alternate_navigation_review_ticket_screen(cwd)?;

        run_emc_stdout(
            cwd,
            &[
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            "removed transition capture-ticket to review-ticket",
        )?;

        run_emc(cwd, &["check"])?;

        let lean = read_lean(cwd)?;
        let quint = read_quint(cwd)?;

        assert!(
            lean.contains("alternate-review-ticket-screen"),
            "Lean artifact must preserve the remaining workflow transition"
        );
        assert!(
            quint.contains("alternate-review-ticket-screen"),
            "Quint artifact must preserve the remaining workflow transition"
        );

        Ok(())
    }

    #[test]
    fn update_transition_replaces_modeled_transition_in_canonical_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        add_review_ticket_slice(cwd)?;
        connect_navigation_review_ticket_screen(cwd)?;

        run_emc_stdout(
            cwd,
            &[
                "update",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--new-from",
                "capture-ticket",
                "--new-to",
                "review-ticket",
                "--new-via",
                "navigation",
                "--new-name",
                "alternate-review-ticket-screen",
                "--new-source-control",
                "alternate-review-ticket-screen",
                "--new-target-view",
                "alternate-review-ticket-screen",
            ],
            "updated transition capture-ticket to review-ticket",
        )?;

        run_emc(cwd, &["check"])?;

        let lean = read_lean(cwd)?;
        let quint = read_quint(cwd)?;

        assert!(
            !lean.contains("trigger := \"review-ticket-screen\""),
            "Lean artifact must remove the replaced workflow transition"
        );
        assert!(
            !quint.contains("trigger: \"review-ticket-screen\""),
            "Quint artifact must remove the replaced workflow transition"
        );
        assert!(
            lean.contains("trigger := \"alternate-review-ticket-screen\""),
            "Lean artifact must contain the replacement workflow transition"
        );
        assert!(
            quint.contains("trigger: \"alternate-review-ticket-screen\""),
            "Quint artifact must contain the replacement workflow transition"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_rejects_removing_required_incoming_transition_without_mutating_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        add_review_ticket_slice(cwd)?;
        connect_navigation_review_ticket_screen(cwd)?;

        let before = capture_open_ticket_artifacts(cwd)?;

        run_emc_failure(
            cwd,
            &[
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            "removing transition would leave workflow step 'review-ticket' without an incoming transition",
        )?;

        assert_open_ticket_artifacts_unchanged(cwd, &before)?;

        Ok(())
    }

    #[test]
    fn update_transition_rejects_unknown_transition_without_mutating_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        add_review_ticket_slice(cwd)?;
        connect_navigation_review_ticket_screen(cwd)?;

        let before = capture_open_ticket_artifacts(cwd)?;

        run_emc_failure(
            cwd,
            &[
                "update",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "missing-review-ticket-screen",
                "--new-from",
                "capture-ticket",
                "--new-to",
                "review-ticket",
                "--new-via",
                "navigation",
                "--new-name",
                "alternate-review-ticket-screen",
                "--new-source-control",
                "alternate-review-ticket-screen",
                "--new-target-view",
                "alternate-review-ticket-screen",
            ],
            "workflow transition capture-ticket->review-ticket:navigation:missing-review-ticket-screen does not exist",
        )?;

        assert_open_ticket_artifacts_unchanged(cwd, &before)?;

        Ok(())
    }

    /// Snapshot of the canonical `OpenTicket` Lean/Quint artifacts at a point in time.
    struct OpenTicketArtifacts {
        lean: String,
        quint: String,
    }

    fn capture_open_ticket_artifacts(cwd: &Path) -> Result<OpenTicketArtifacts, Box<dyn Error>> {
        Ok(OpenTicketArtifacts {
            lean: read_lean(cwd)?,
            quint: read_quint(cwd)?,
        })
    }

    fn assert_open_ticket_artifacts_unchanged(
        cwd: &Path,
        before: &OpenTicketArtifacts,
    ) -> Result<(), Box<dyn Error>> {
        assert_eq!(
            before.lean,
            read_lean(cwd)?,
            "rejected transition removal must not mutate Lean workflow data"
        );
        assert_eq!(
            before.quint,
            read_quint(cwd)?,
            "rejected transition removal must not mutate Quint workflow data"
        );
        Ok(())
    }

    #[test]
    fn remove_transition_rejects_unknown_transition_without_mutating_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow transition capture-ticket->review-ticket:navigation:review-ticket-screen does not exist",
            ));

        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "rejected transition removal must not mutate Lean workflow data"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "rejected transition removal must not mutate Quint workflow data"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_removes_workflow_exit_transition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        init_repair_desk(cwd)?;
        add_open_ticket_workflow(cwd)?;
        add_close_ticket_workflow(cwd)?;
        add_capture_ticket_slice(cwd)?;
        connect_open_ticket_exit_to_close_ticket(cwd)?;

        run_emc_stdout(
            cwd,
            &[
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            "removed transition capture-ticket to close-ticket",
        )?;

        run_emc(cwd, &["check"])?;

        let lean = read_lean(cwd)?;
        let quint = read_quint(cwd)?;

        assert!(
            lean.contains("def workflowTransitions : List WorkflowTransition := []"),
            "Lean artifact must remove the workflow-exit transition"
        );
        assert!(
            quint.contains("val workflowTransitions: List[WorkflowTransition] = []"),
            "Quint artifact must remove the workflow-exit transition"
        );

        Ok(())
    }

    fn add_close_ticket_workflow(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "workflow",
                "--slug",
                "close-ticket",
                "--name",
                "Close ticket",
                "--description",
                "Actor closes a repair ticket.",
            ],
        )
    }

    fn connect_open_ticket_exit_to_close_ticket(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
        )
    }

    fn malformed_in_workflow_remove_commands_a() -> [[&'static str; 12]; 4] {
        [
            [
                "delete",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "edge",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "transition",
                "--model",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--source",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
        ]
    }

    fn malformed_in_workflow_remove_commands_b() -> [[&'static str; 12]; 3] {
        [
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--target",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--kind",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--trigger",
                "review-ticket-screen",
            ],
        ]
    }

    #[test]
    fn remove_transition_requires_exact_in_workflow_command_shape() -> Result<(), Box<dyn Error>> {
        malformed_in_workflow_remove_commands_a()
            .into_iter()
            .chain(malformed_in_workflow_remove_commands_b())
            .try_for_each(assert_usage)?;

        Ok(())
    }

    fn malformed_workflow_exit_remove_commands_a() -> [[&'static str; 12]; 4] {
        [
            [
                "delete",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "edge",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--model",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--source",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
        ]
    }

    fn malformed_workflow_exit_remove_commands_b() -> [[&'static str; 12]; 3] {
        [
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--target-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--kind",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--trigger",
                "ticket-closed",
            ],
        ]
    }

    #[test]
    fn remove_transition_requires_exact_workflow_exit_command_shape() -> Result<(), Box<dyn Error>>
    {
        malformed_workflow_exit_remove_commands_a()
            .into_iter()
            .chain(malformed_workflow_exit_remove_commands_b())
            .try_for_each(assert_usage)?;

        Ok(())
    }

    #[test]
    fn connect_workflow_adds_external_trigger_transition_to_canonical_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "record-callback",
            "Record callback",
            "System records an external callback.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "record-callback",
                "--via",
                "external_trigger",
                "--name",
                "callback_received",
                "--payload-contract",
                "CallbackReceivedPayload",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to record-callback",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := WorkflowTransitionKind.externalTrigger, trigger := \"callback_received\", sourceControl := \"\", targetView := \"\", rationale := \"\", payloadContract := \"CallbackReceivedPayload\" }]"
            ),
            "Lean artifact must represent the external trigger workflow transition"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"record-callback\", kind: ExternalTrigger, trigger: \"callback_received\", sourceControl: \"\", targetView: \"\", rationale: \"\", payloadContract: \"CallbackReceivedPayload\" }]"
            ),
            "Quint artifact must represent the external trigger workflow transition"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_adds_workflow_exit_to_canonical_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to repair-complete",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := WorkflowTransitionKind.workflowExitOutcome, trigger := \"ticket_closed\", sourceControl := \"\", targetView := \"\", rationale := \"Closed tickets continue to completion.\", payloadContract := \"\" }]"
            ),
            "Lean artifact must represent the workflow exit"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: WorkflowExitOutcome, trigger: \"ticket_closed\", sourceControl: \"\", targetView: \"\", rationale: \"Closed tickets continue to completion.\", payloadContract: \"\" }]"
            ),
            "Quint artifact must represent the workflow exit"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_rejects_duplicate_transition_without_rewriting_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "submit-ticket",
            "Submit ticket",
            "Actor submits repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "submit-ticket",
                "--via",
                "command",
                "--name",
                "SubmitTicketForReview",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "submit-ticket",
                "--via",
                "command",
                "--name",
                "SubmitTicketForReview",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow transition capture-ticket->submit-ticket:command:SubmitTicketForReview already exists",
            ));

        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "duplicate transition rejection must leave Lean workflow data unchanged"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "duplicate transition rejection must leave Quint workflow data unchanged"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_rejects_unknown_target_slice_without_rewriting_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "missing-ticket",
                "--via",
                "navigation",
                "--name",
                "missing-ticket-screen",
                "--source-control",
                "missing-ticket-screen",
                "--target-view",
                "missing-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "unknown workflow step missing-ticket",
            ));

        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "unknown target rejection must leave Lean workflow data unchanged"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "unknown target rejection must leave Quint workflow data unchanged"
        );

        Ok(())
    }

    fn malformed_connect_exit_commands_a() -> [[&'static str; 14]; 4] {
        [
            [
                "wrong-command",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "wrong-subject",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--wrong-workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--wrong-from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
        ]
    }

    fn malformed_connect_exit_commands_b() -> [[&'static str; 14]; 4] {
        [
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--wrong-target",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--wrong-via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--wrong-name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--wrong-reason",
                "Closed tickets continue to completion.",
            ],
        ]
    }

    #[test]
    fn connect_workflow_exit_requires_exact_flag_order() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path();

        for malformed_command in malformed_connect_exit_commands_a()
            .into_iter()
            .chain(malformed_connect_exit_commands_b())
        {
            run_emc_failure(
                cwd,
                &malformed_command,
                "usage: emc <command> [arguments]; run emc --help",
            )?;
        }

        Ok(())
    }

    fn assert_usage<const N: usize>(args: [&str; N]) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));
        Ok(())
    }

    fn run_emc(cwd: &Path, args: &[&str]) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(args)
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn run_emc_stdout(
        cwd: &Path,
        args: &[&str],
        expected_stdout: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(args)
            .current_dir(cwd)
            .assert()
            .success()
            .stdout(predicate::str::contains(expected_stdout.to_owned()));
        Ok(())
    }

    fn run_emc_failure(
        cwd: &Path,
        args: &[&str],
        expected_stderr: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(args)
            .current_dir(cwd)
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected_stderr.to_owned()));
        Ok(())
    }

    fn init_repair_desk(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(cwd, &["init", "--name", "Repair Desk"])
    }

    fn add_open_ticket_workflow(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ],
        )
    }

    fn add_capture_ticket_slice(cwd: &Path) -> Result<(), Box<dyn Error>> {
        add_slice(
            cwd,
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )
    }

    fn add_review_ticket_slice(cwd: &Path) -> Result<(), Box<dyn Error>> {
        add_slice(
            cwd,
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )
    }

    fn lean_path(cwd: &Path) -> PathBuf {
        cwd.join("model/lean/OpenTicket.lean")
    }

    fn quint_path(cwd: &Path) -> PathBuf {
        cwd.join("model/quint/OpenTicket.qnt")
    }

    fn read_lean(cwd: &Path) -> Result<String, Box<dyn Error>> {
        Ok(read_to_string(lean_path(cwd))?)
    }

    fn read_quint(cwd: &Path) -> Result<String, Box<dyn Error>> {
        Ok(read_to_string(quint_path(cwd))?)
    }

    fn initial_lean_digest(cwd: &Path) -> Result<String, Box<dyn Error>> {
        digest_marker(&read_lean(cwd)?)
            .ok_or_else(|| "Lean artifact is missing its initial digest".into())
    }

    fn connect_navigation_review_ticket_screen(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
            ],
        )
    }

    fn initialize_project_with_open_ticket_workflow(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        add_slice(
            cwd,
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )
    }

    fn add_workflow_outcome(
        cwd: &Path,
        label: &str,
        externally_relevant: bool,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                label,
                "--externally-relevant",
                if externally_relevant { "true" } else { "false" },
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_workflow_command_error(
        cwd: &Path,
        command: &str,
        error: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                command,
                "--error",
                error,
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_slice(
        cwd: &Path,
        slug: &str,
        name: &str,
        description: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                slug,
                "--name",
                name,
                "--type",
                "state_view",
                "--description",
                description,
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_state_view_scenario(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "scenario",
                "--slice",
                slug,
                "--kind",
                "contract",
                "--name",
                "Ticket state projects title",
                "--given",
                "TicketCaptured carries ticket_title",
                "--when",
                "ticket_state projects TicketCaptured",
                "--then",
                "ticket_state.ticket_title equals TicketCaptured.ticket_title",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state",
            ],
        )
    }

    fn add_ticket_captured_event(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "event",
                "--slice",
                slug,
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "upstream_event_store",
                "--attribute-source-field",
                "ticket_title",
                "--generated-source-kind",
                "event_store_observation",
                "--attribute-provenance",
                "TicketCaptured.ticket_title",
                "--observed",
                "true",
            ],
        )
    }

    fn add_ticket_state_read_model(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "read-model",
                "--slice",
                slug,
                "--name",
                "ticket_state",
                "--field",
                "ticket_title",
                "--field-source",
                "event_attribute",
                "--source-event",
                "TicketCaptured",
                "--source-attribute",
                "ticket_title",
                "--field-provenance",
                "TicketCaptured.ticket_title",
            ],
        )
    }

    fn add_ticket_summary_view(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "view",
                "--slice",
                slug,
                "--name",
                "ticket_summary",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "title-label",
                "--field-provenance",
                "ticket_state.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ],
        )
    }

    fn add_complete_state_view_facts(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        add_state_view_scenario(cwd, slug)?;
        add_ticket_captured_event(cwd, slug)?;
        add_ticket_state_read_model(cwd, slug)?;
        add_ticket_summary_view(cwd, slug)?;

        add_data_flow(
            cwd,
            slug,
            "upstream event store",
            "identity",
            "TicketCaptured",
        )?;
        add_data_flow(
            cwd,
            slug,
            "TicketCaptured.ticket_title",
            "projection",
            "ticket_state",
        )?;
        add_data_flow(
            cwd,
            slug,
            "ticket_state.ticket_title",
            "projection",
            "ticket_summary",
        )
    }

    fn add_review_state_view_scenario(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "scenario",
                "--slice",
                "review-ticket",
                "--kind",
                "contract",
                "--name",
                "Review state projects title",
                "--given",
                "ReviewTicketCaptured carries ticket_title",
                "--when",
                "review_ticket_state projects ReviewTicketCaptured",
                "--then",
                "review_ticket_state.ticket_title equals ReviewTicketCaptured.ticket_title",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "review_ticket_state",
            ],
        )
    }

    fn add_review_ticket_captured_event(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "event",
                "--slice",
                "review-ticket",
                "--name",
                "ReviewTicketCaptured",
                "--stream",
                "review_tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "review_event_store",
                "--attribute-source-field",
                "ticket_title",
                "--generated-source-kind",
                "event_store_observation",
                "--attribute-provenance",
                "ReviewTicketCaptured.ticket_title",
                "--observed",
                "true",
            ],
        )
    }

    fn add_review_ticket_state_read_model(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "read-model",
                "--slice",
                "review-ticket",
                "--name",
                "review_ticket_state",
                "--field",
                "ticket_title",
                "--field-source",
                "event_attribute",
                "--source-event",
                "ReviewTicketCaptured",
                "--source-attribute",
                "ticket_title",
                "--field-provenance",
                "ReviewTicketCaptured.ticket_title",
            ],
        )
    }

    fn add_review_ticket_summary_view(cwd: &Path) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "view",
                "--slice",
                "review-ticket",
                "--name",
                "review_ticket_summary",
                "--read-model",
                "review_ticket_state",
                "--field",
                "ticket_title",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "review-title-label",
                "--field-provenance",
                "review_ticket_state.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ],
        )
    }

    fn add_review_state_view_facts(cwd: &Path) -> Result<(), Box<dyn Error>> {
        add_review_state_view_scenario(cwd)?;
        add_review_ticket_captured_event(cwd)?;
        add_review_ticket_state_read_model(cwd)?;
        add_review_ticket_summary_view(cwd)?;

        add_data_flow(
            cwd,
            "review-ticket",
            "review event store",
            "identity",
            "ReviewTicketCaptured",
        )?;
        add_data_flow(
            cwd,
            "review-ticket",
            "ReviewTicketCaptured.ticket_title",
            "projection",
            "review_ticket_state",
        )?;
        add_data_flow(
            cwd,
            "review-ticket",
            "review_ticket_state.ticket_title",
            "projection",
            "review_ticket_summary",
        )
    }

    fn add_submit_ticket_event(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "event",
                "--slice",
                slug,
                "--name",
                "TicketSubmittedForReview",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "actor_input",
                "--attribute-source-field",
                "ticket_title",
                "--generated-source-kind",
                "actor_input",
                "--attribute-provenance",
                "SubmitTicketForReview.ticket_title",
            ],
        )
    }

    fn add_submit_ticket_command(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "command",
                "--slice",
                slug,
                "--name",
                "SubmitTicketForReview",
                "--input",
                "ticket_title",
                "--input-source",
                "actor",
                "--input-description",
                "title field on the submit form",
                "--input-provenance",
                "actor confirmation -> form field",
                "--emits",
                "TicketSubmittedForReview",
            ],
        )
    }

    fn add_submit_ticket_scenario(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "scenario",
                "--slice",
                slug,
                "--kind",
                "contract",
                "--name",
                "Submit command emits review event",
                "--given",
                "The actor confirms the ticket title",
                "--when",
                "SubmitTicketForReview runs",
                "--then",
                "TicketSubmittedForReview carries the ticket title",
                "--contract-kind",
                "command",
                "--covered-definition",
                "SubmitTicketForReview",
                "--read-streams",
                "tickets",
                "--written-streams",
                "tickets",
            ],
        )
    }

    fn add_submit_ticket_outcome(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &[
                "add",
                "outcome",
                "--slice",
                slug,
                "--label",
                "ticket_submitted_for_review",
                "--events",
                "TicketSubmittedForReview",
                "--externally-relevant",
                "false",
            ],
        )
    }

    fn add_complete_state_change_facts(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        run_emc(
            cwd,
            &["update", "slice", "--slug", slug, "--type", "state_change"],
        )?;
        add_submit_ticket_event(cwd, slug)?;
        add_submit_ticket_command(cwd, slug)?;
        add_submit_ticket_scenario(cwd, slug)?;
        add_submit_ticket_outcome(cwd, slug)?;

        add_data_flow(
            cwd,
            slug,
            "actor submit form title field",
            "identity",
            "SubmitTicketForReview",
        )?;
        add_data_flow(
            cwd,
            slug,
            "SubmitTicketForReview.ticket_title",
            "projection",
            "TicketSubmittedForReview",
        )
    }

    fn add_data_flow(
        cwd: &Path,
        slug: &str,
        source: &str,
        transformation: &str,
        target: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                slug,
                "--datum",
                "ticket_title",
                "--source",
                source,
                "--source-kind",
                "original",
                "--transformation",
                transformation,
                "--target",
                target,
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_entry_lifecycle_state(
        cwd: &Path,
        state: &str,
        step: &str,
        evidence: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-entry-lifecycle-state",
                "--workflow",
                "application-entry",
                "--state",
                state,
                "--step",
                step,
                "--evidence",
                evidence,
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_workflow_owned_definition(
        cwd: &Path,
        workflow: &str,
        source_slice: &str,
        definition_kind: &str,
        definition_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-owned-definition",
                "--workflow",
                workflow,
                "--source-slice",
                source_slice,
                "--definition-kind",
                definition_kind,
                "--definition-name",
                definition_name,
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_workflow_owned_entry_view_definition(
        cwd: &Path,
        workflow: &str,
        source_slice: &str,
        definition_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-owned-definition",
                "--workflow",
                workflow,
                "--source-slice",
                source_slice,
                "--definition-kind",
                "view",
                "--definition-name",
                definition_name,
                "--view-role",
                "entry",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_workflow_owned_event_definition(
        cwd: &Path,
        workflow: &str,
        source_slice: &str,
        definition_name: &str,
        definition_stream: &str,
        source_provenance: &str,
        event_participation: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-owned-definition",
                "--workflow",
                workflow,
                "--source-slice",
                source_slice,
                "--definition-kind",
                "event",
                "--definition-name",
                definition_name,
                "--definition-stream",
                definition_stream,
                "--source-provenance",
                source_provenance,
                "--event-participation",
                event_participation,
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    struct WorkflowTransitionEvidenceFixture<'a> {
        workflow: &'a str,
        source: &'a str,
        target: &'a str,
        kind: &'a str,
        trigger: &'a str,
        source_evidence: &'a str,
        target_evidence: &'a str,
    }

    fn add_workflow_transition_evidence(
        cwd: &Path,
        evidence: &WorkflowTransitionEvidenceFixture<'_>,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-transition-evidence",
                "--workflow",
                evidence.workflow,
                "--from",
                evidence.source,
                "--to",
                evidence.target,
                "--via",
                evidence.kind,
                "--name",
                evidence.trigger,
                "--source-evidence",
                evidence.source_evidence,
                "--target-evidence",
                evidence.target_evidence,
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn digest_marker(contents: &str) -> Option<String> {
        contents.lines().find_map(|line| {
            line.trim()
                .strip_prefix("-- EMC-DIGEST: ")
                .or_else(|| line.trim().strip_prefix("// EMC-DIGEST: "))
                .map(str::to_owned)
        })
    }
}
