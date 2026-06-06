// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;
    use std::path::Path;

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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "Lean artifact must represent the workflow transition"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", rationale: \"\", payloadContract: \"\" }]"
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
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := \"command\", definitionName := \"CaptureTicket\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: \"command\", definitionName: \"CaptureTicket\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" }]"
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
    fn add_workflow_transition_evidence_updates_canonical_workflow_artifacts()
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

        let initial_digest = digest_marker(&read_to_string(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
        )?)
        .ok_or("Lean artifact is missing its initial digest")?;

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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added workflow transition evidence navigation review-ticket-screen to workflow open-ticket",
            ));

        add_workflow_owned_definition(
            temp_dir.path(),
            "open-ticket",
            "capture-ticket",
            "control",
            "review-ticket-screen",
        )?;
        add_workflow_owned_entry_view_definition(
            temp_dir.path(),
            "open-ticket",
            "review-ticket",
            "review-ticket-screen",
        )?;

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let updated_digest = digest_marker(&lean).ok_or("Lean artifact is missing digest")?;

        assert!(lean.contains(
            "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(quint.contains(
            "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := \"control\", definitionName := \"review-ticket-screen\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" },{ sourceSlice := \"review-ticket\", definitionKind := \"view\", definitionName := \"review-ticket-screen\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"entry\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: \"control\", definitionName: \"review-ticket-screen\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" },{ sourceSlice: \"review-ticket\", definitionKind: \"view\", definitionName: \"review-ticket-screen\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"entry\" }]"
        ));
        assert!(lean.contains(
            "def workflowNavigationTransitionTargetsEntryView (transition : WorkflowTransition) : Bool := transition.kind != \"navigation\" || workflowOwnsEntryView transition.target transition.trigger"
        ));
        assert!(lean.contains(
            "theorem workflowNavigationTransitionsResolveToEntryViewsIsStable : workflowNavigationTransitionsResolveToEntryViews = true := rfl"
        ));
        assert!(quint.contains(
            "val workflowNavigationTransitionsResolveToEntryViews = workflowTransitions.select(transition => workflowNavigationTransitionTargetsEntryView(transition)).length() == workflowTransitions.length()"
        ));
        assert_ne!(
            initial_digest, updated_digest,
            "workflow digest must change when transition evidence and ownership facts are authored"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_workflow_entry_lifecycle_coverage_updates_canonical_workflow_artifacts()
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
                "application-entry",
                "--name",
                "Application entry",
                "--description",
                "Actor enters the application.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        add_complete_state_view_facts(temp_dir.path(), "entry-state")?;

        let initial_digest = digest_marker(&read_to_string(
            temp_dir.path().join("model/lean/ApplicationEntry.lean"),
        )?)
        .ok_or("Lean artifact is missing its initial digest")?;

        Command::cargo_bin("emc")?
            .args([
                "mark",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "application-entry",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "marked workflow application-entry as requiring entry lifecycle coverage",
            ));

        add_entry_lifecycle_state(
            temp_dir.path(),
            "fresh_uninitialized",
            "entry-state",
            "entry-state view distinguishes first arrival before initialization",
        )?;
        add_entry_lifecycle_state(
            temp_dir.path(),
            "initialized_unauthenticated",
            "entry-state",
            "entry-state view distinguishes initialized unauthenticated sessions",
        )?;
        add_entry_lifecycle_state(
            temp_dir.path(),
            "initialized_authenticated",
            "entry-state",
            "entry-state view distinguishes initialized authenticated sessions",
        )?;
        add_entry_lifecycle_state(
            temp_dir.path(),
            "partially_configured",
            "entry-state",
            "entry-state view distinguishes partially configured accounts",
        )?;
        add_entry_lifecycle_state(
            temp_dir.path(),
            "fully_configured",
            "entry-state",
            "entry-state view distinguishes fully configured accounts",
        )?;

        let lean = read_to_string(temp_dir.path().join("model/lean/ApplicationEntry.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/ApplicationEntry.qnt"))?;
        let updated_digest = digest_marker(&lean).ok_or("Lean artifact is missing digest")?;

        assert!(lean.contains("def workflowRequiresEntryLifecycleCoverage : Bool := true"));
        assert!(quint.contains("val workflowRequiresEntryLifecycleCoverage = true"));
        assert!(lean.contains(
            "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := [{ state := \"fresh_uninitialized\", step := \"entry-state\", evidence := \"entry-state view distinguishes first arrival before initialization\" },{ state := \"initialized_unauthenticated\", step := \"entry-state\", evidence := \"entry-state view distinguishes initialized unauthenticated sessions\" },{ state := \"initialized_authenticated\", step := \"entry-state\", evidence := \"entry-state view distinguishes initialized authenticated sessions\" },{ state := \"partially_configured\", step := \"entry-state\", evidence := \"entry-state view distinguishes partially configured accounts\" },{ state := \"fully_configured\", step := \"entry-state\", evidence := \"entry-state view distinguishes fully configured accounts\" }]"
        ));
        assert!(quint.contains(
            "val workflowEntryLifecycleStates: List[WorkflowEntryLifecycleState] = [{ state: \"fresh_uninitialized\", step: \"entry-state\", evidence: \"entry-state view distinguishes first arrival before initialization\" },{ state: \"initialized_unauthenticated\", step: \"entry-state\", evidence: \"entry-state view distinguishes initialized unauthenticated sessions\" },{ state: \"initialized_authenticated\", step: \"entry-state\", evidence: \"entry-state view distinguishes initialized authenticated sessions\" },{ state: \"partially_configured\", step: \"entry-state\", evidence: \"entry-state view distinguishes partially configured accounts\" },{ state: \"fully_configured\", step: \"entry-state\", evidence: \"entry-state view distinguishes fully configured accounts\" }]"
        ));
        assert_ne!(
            initial_digest, updated_digest,
            "workflow digest must change when entry lifecycle coverage is authored"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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

    #[test]
    fn workflow_updates_preserve_authored_workflow_facts() -> Result<(), Box<dyn Error>> {
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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens and reviews a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

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
            "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(quint.contains(
            "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));

        Ok(())
    }

    #[test]
    fn connect_workflow_adds_command_and_event_transitions_to_canonical_artifacts()
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
            "submit-ticket",
            "Submit ticket",
            "Actor submits repair ticket details.",
        )?;
        add_complete_state_change_facts(temp_dir.path(), "submit-ticket")?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;
        add_complete_state_view_facts(temp_dir.path(), "review-ticket")?;

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
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to submit-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected submit-ticket to review-ticket",
            ));

        add_workflow_transition_evidence(
            temp_dir.path(),
            WorkflowTransitionEvidenceFixture {
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
            temp_dir.path(),
            "open-ticket",
            "capture-ticket",
            "control",
            "SubmitTicketForReview",
        )?;
        add_workflow_owned_definition(
            temp_dir.path(),
            "open-ticket",
            "submit-ticket",
            "command",
            "SubmitTicketForReview",
        )?;
        add_workflow_transition_evidence(
            temp_dir.path(),
            WorkflowTransitionEvidenceFixture {
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
            temp_dir.path(),
            "open-ticket",
            "submit-ticket",
            "TicketSubmittedForReview",
            "tickets",
            "SubmitTicketForReview command output",
            "emitted",
        )?;
        add_workflow_owned_event_definition(
            temp_dir.path(),
            "open-ticket",
            "review-ticket",
            "TicketSubmittedForReview",
            "tickets",
            "SubmitTicketForReview command output",
            "observed",
        )?;

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := \"command\", trigger := \"SubmitTicketForReview\", rationale := \"\", payloadContract := \"\" },{ source := \"submit-ticket\", target := \"review-ticket\", kind := \"event\", trigger := \"TicketSubmittedForReview\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "Lean artifact must represent command and event workflow transitions"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: \"command\", trigger: \"SubmitTicketForReview\", rationale: \"\", payloadContract: \"\" },{ source: \"submit-ticket\", target: \"review-ticket\", kind: \"event\", trigger: \"TicketSubmittedForReview\", rationale: \"\", payloadContract: \"\" }]"
            ),
            "Quint artifact must represent command and event workflow transitions"
        );
        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := \"control\", definitionName := \"SubmitTicketForReview\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" },{ sourceSlice := \"submit-ticket\", definitionKind := \"command\", definitionName := \"SubmitTicketForReview\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" },{ sourceSlice := \"submit-ticket\", definitionKind := \"event\", definitionName := \"TicketSubmittedForReview\", definitionStream := \"tickets\", sourceProvenance := \"SubmitTicketForReview command output\", eventParticipation := \"emitted\", viewRole := \"\" },{ sourceSlice := \"review-ticket\", definitionKind := \"event\", definitionName := \"TicketSubmittedForReview\", definitionStream := \"tickets\", sourceProvenance := \"SubmitTicketForReview command output\", eventParticipation := \"observed\", viewRole := \"\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: \"control\", definitionName: \"SubmitTicketForReview\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" },{ sourceSlice: \"submit-ticket\", definitionKind: \"command\", definitionName: \"SubmitTicketForReview\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" },{ sourceSlice: \"submit-ticket\", definitionKind: \"event\", definitionName: \"TicketSubmittedForReview\", definitionStream: \"tickets\", sourceProvenance: \"SubmitTicketForReview command output\", eventParticipation: \"emitted\", viewRole: \"\" },{ sourceSlice: \"review-ticket\", definitionKind: \"event\", definitionName: \"TicketSubmittedForReview\", definitionStream: \"tickets\", sourceProvenance: \"SubmitTicketForReview command output\", eventParticipation: \"observed\", viewRole: \"\" }]"
        ));

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn remove_transition_removes_modeled_transition_from_canonical_artifacts()
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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "alternate-review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
            .success()
            .stdout(predicate::str::contains(
                "removed transition capture-ticket to review-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

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
    fn remove_transition_rejects_removing_required_incoming_transition_without_mutating_artifacts()
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
                "removing transition would leave workflow step 'review-ticket' without an incoming transition",
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

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "close-ticket",
                "--name",
                "Close ticket",
                "--description",
                "Actor closes a repair ticket.",
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
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
                "--reason",
                "Closed tickets continue to completion.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed transition capture-ticket to close-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

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

    #[test]
    fn remove_transition_requires_exact_in_workflow_command_shape() -> Result<(), Box<dyn Error>> {
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
        .into_iter()
        .try_for_each(assert_usage)?;

        Ok(())
    }

    #[test]
    fn remove_transition_requires_exact_workflow_exit_command_shape() -> Result<(), Box<dyn Error>>
    {
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
        .into_iter()
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := \"external_trigger\", trigger := \"callback_received\", rationale := \"\", payloadContract := \"CallbackReceivedPayload\" }]"
            ),
            "Lean artifact must represent the external trigger workflow transition"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"record-callback\", kind: \"external_trigger\", trigger: \"callback_received\", rationale: \"\", payloadContract: \"CallbackReceivedPayload\" }]"
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := \"workflow_exit:outcome\", trigger := \"ticket_closed\", rationale := \"Closed tickets continue to completion.\", payloadContract := \"\" }]"
            ),
            "Lean artifact must represent the workflow exit"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: \"workflow_exit:outcome\", trigger: \"ticket_closed\", rationale: \"Closed tickets continue to completion.\", payloadContract: \"\" }]"
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

    #[test]
    fn connect_workflow_exit_requires_exact_flag_order() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let malformed_commands = [
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
        ];

        for malformed_command in malformed_commands {
            Command::cargo_bin("emc")?
                .args(malformed_command)
                .current_dir(temp_dir.path())
                .assert()
                .failure()
                .stderr(predicate::str::contains(
                    "usage: emc <command> [arguments]; run emc --help",
                ));
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

    fn add_complete_state_view_facts(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        add_data_flow(
            cwd,
            slug,
            "upstream event store",
            "observed without transformation",
            "TicketCaptured",
        )?;
        add_data_flow(
            cwd,
            slug,
            "TicketCaptured.ticket_title",
            "projected without transformation",
            "ticket_state",
        )?;
        add_data_flow(
            cwd,
            slug,
            "ticket_state.ticket_title",
            "displayed without transformation",
            "ticket_summary",
        )
    }

    fn add_complete_state_change_facts(cwd: &Path, slug: &str) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["update", "slice", "--slug", slug, "--type", "state_change"])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
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
            ])
            .current_dir(cwd)
            .assert()
            .success();

        add_data_flow(
            cwd,
            slug,
            "actor submit form title field",
            "captured without normalization",
            "SubmitTicketForReview",
        )?;
        add_data_flow(
            cwd,
            slug,
            "SubmitTicketForReview.ticket_title",
            "copied into TicketSubmittedForReview.ticket_title",
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
        evidence: WorkflowTransitionEvidenceFixture<'_>,
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
