// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{exists, read_to_string};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn update_slice_description_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--description",
                "Actor enters repair ticket details and priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("updated slice Capture ticket"));

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"capture-ticket\", name := \"Capture ticket\", kind := \"state_view\", description := \"Actor enters repair ticket details and priority.\" }"
            ),
            "workflow Lean artifact must represent the updated slice detail"
        );
        assert!(
            workflow_quint
                .contains("description: \"Actor enters repair ticket details and priority.\""),
            "workflow Quint artifact must represent the updated slice detail"
        );
        assert!(
            slice_lean.contains(
                "def sliceDescription := \"Actor enters repair ticket details and priority.\""
            ),
            "slice Lean artifact must represent the updated slice description"
        );
        assert!(
            slice_quint.contains(
                "val sliceDescription = \"Actor enters repair ticket details and priority.\""
            ),
            "slice Quint artifact must represent the updated slice description"
        );

        Ok(())
    }

    #[test]
    fn update_slice_kind_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "automation",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("updated slice Capture ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"capture-ticket\", name := \"Capture ticket\", kind := \"automation\", description := \"Actor enters repair ticket details.\" }"
            ),
            "workflow Lean artifact must represent the updated slice kind"
        );
        assert!(
            workflow_quint.contains("kind: \"automation\""),
            "workflow Quint artifact must represent the updated slice kind"
        );
        assert!(
            slice_lean.contains("def sliceKind := \"automation\""),
            "slice Lean artifact must represent the updated slice kind"
        );
        assert!(
            slice_quint.contains("val sliceKind = \"automation\""),
            "slice Quint artifact must represent the updated slice kind"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture repair ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated slice Capture repair ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean = read_to_string(
            temp_dir
                .path()
                .join("model/lean/slices/CaptureRepairTicket.lean"),
        )?;
        let slice_quint = read_to_string(
            temp_dir
                .path()
                .join("model/quint/slices/CaptureRepairTicket.qnt"),
        )?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"capture-ticket\", name := \"Capture repair ticket\", kind := \"state_view\", description := \"Actor enters repair ticket details.\" }"
            ),
            "workflow Lean artifact must represent the updated slice name"
        );
        assert!(
            workflow_quint.contains("name: \"Capture repair ticket\""),
            "workflow Quint artifact must represent the updated slice name"
        );
        assert!(
            slice_lean.contains("namespace CaptureRepairTicket"),
            "slice Lean artifact must move to the updated module"
        );
        assert!(
            slice_lean.contains("def sliceName := \"Capture repair ticket\""),
            "slice Lean artifact must represent the updated slice name"
        );
        assert!(
            slice_quint.contains("module CaptureRepairTicket {"),
            "slice Quint artifact must move to the updated module"
        );
        assert!(
            slice_quint.contains("val sliceName = \"Capture repair ticket\""),
            "slice Quint artifact must represent the updated slice name"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?,
            "slice name update must remove the old Lean slice module"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?,
            "slice name update must remove the old Quint slice module"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_preserves_outgoing_workflow_transitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_navigation_chain(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "triage-intake",
                "--name",
                "Clinical triage",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("updated slice Clinical triage"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/IntakeVisit.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/IntakeVisit.qnt"))?;
        assert!(
            workflow_lean.contains(
                "{ source := \"triage-intake\", target := \"schedule-visit\", kind := \"navigation\", trigger := \"schedule-visit-screen\", rationale := \"\", payloadContract := \"\" }"
            ),
            "slice rename must preserve outgoing workflow transitions"
        );
        assert!(
            workflow_quint.contains(
                "{ source: \"triage-intake\", target: \"schedule-visit\", kind: \"navigation\", trigger: \"schedule-visit-screen\", rationale: \"\", payloadContract: \"\" }"
            ),
            "slice rename must preserve outgoing workflow transitions"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_rejects_formal_module_name_collisions() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "review-ticket",
                "--name",
                "Review ticket",
                "--type",
                "state_view",
                "--description",
                "Actor reviews repair ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "review-ticket",
                "--name",
                "Capture-ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice module CaptureTicket already exists",
            ));

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/ReviewTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/ReviewTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"review-ticket\", name := \"Review ticket\", kind := \"state_view\", description := \"Actor reviews repair ticket details.\" }"
            ),
            "rejected slice rename must not mutate the workflow Lean artifact"
        );
        assert!(
            workflow_quint.contains("name: \"Review ticket\""),
            "rejected slice rename must not mutate the workflow Quint artifact"
        );
        assert!(
            slice_lean.contains("def sliceName := \"Review ticket\""),
            "rejected slice rename must not mutate the slice Lean artifact"
        );
        assert!(
            slice_quint.contains("val sliceName = \"Review ticket\""),
            "rejected slice rename must not mutate the slice Quint artifact"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_rejects_non_slug_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slice",
                "capture-ticket",
                "--name",
                "Capture repair ticket",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn update_slice_name_rejects_non_name_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--title",
                "Capture repair ticket",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn remove_slice_removes_synchronized_slice_artifacts_and_connected_transitions()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "review-ticket",
                "--name",
                "Review ticket",
                "--type",
                "state_view",
                "--description",
                "Actor reviews repair ticket details.",
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
            .args(["remove", "slice", "--slug", "review-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("removed slice Review ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "def workflowSlices : List WorkflowSlice := [{ slug := \"capture-ticket\" }]"
            ),
            "workflow Lean artifact must keep only remaining slices"
        );
        assert!(
            workflow_lean.contains("def workflowTransitions : List WorkflowTransition := []"),
            "workflow Lean artifact must remove transitions involving the slice"
        );
        assert!(
            workflow_quint.contains(
                "val workflowSlices: List[WorkflowSlice] = [{ slug: \"capture-ticket\" }]"
            ),
            "workflow Quint artifact must keep only remaining slices"
        );
        assert!(
            workflow_quint.contains("val workflowTransitions: List[WorkflowTransition] = []"),
            "workflow Quint artifact must remove transitions involving the slice"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/slices/ReviewTicket.lean"))?,
            "removed slice Lean artifact must be deleted"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/slices/ReviewTicket.qnt"))?,
            "removed slice Quint artifact must be deleted"
        );

        Ok(())
    }

    #[test]
    fn remove_middle_slice_rejects_outgoing_navigation_dependencies() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        initialize_project_with_navigation_chain(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slug", "triage-intake"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice triage-intake has outgoing workflow transitions",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/IntakeVisit.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/IntakeVisit.qnt"))?;
        assert!(
            workflow_lean.contains("triage-intake-screen"),
            "rejected middle slice removal must preserve incoming navigation transitions"
        );
        assert!(
            workflow_quint.contains("schedule-visit-screen"),
            "rejected middle slice removal must preserve outgoing navigation transitions"
        );

        Ok(())
    }

    #[test]
    fn remove_slice_requires_exact_slug_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slice", "capture-ticket"])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn update_slice_kind_rejects_non_slug_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slice",
                "capture-ticket",
                "--type",
                "automation",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn update_slice_kind_rejects_non_type_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--kind",
                "automation",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    fn initialize_project_with_slice(cwd: &Path) -> Result<(), Box<dyn Error>> {
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

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Actor enters repair ticket details.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn initialize_project_with_navigation_chain(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Clinic Intake"])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "intake-visit",
                "--name",
                "Intake visit",
                "--description",
                "Actor completes intake for a clinic visit.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        [
            (
                "capture-intake",
                "Capture intake",
                "Actor captures intake details.",
            ),
            ("triage-intake", "Triage intake", "Actor triages intake."),
            (
                "schedule-visit",
                "Schedule visit",
                "Actor schedules a visit.",
            ),
        ]
        .into_iter()
        .try_for_each(|(slug, name, description)| {
            Command::cargo_bin("emc")?
                .args([
                    "add",
                    "slice",
                    "--workflow",
                    "intake-visit",
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
            Ok::<(), Box<dyn Error>>(())
        })?;

        [
            ("capture-intake", "triage-intake", "triage-intake-screen"),
            ("triage-intake", "schedule-visit", "schedule-visit-screen"),
        ]
        .into_iter()
        .try_for_each(|(source, target, navigation)| {
            Command::cargo_bin("emc")?
                .args([
                    "connect",
                    "workflow",
                    "--workflow",
                    "intake-visit",
                    "--from",
                    source,
                    "--to",
                    target,
                    "--via",
                    "navigation",
                    "--name",
                    navigation,
                ])
                .current_dir(cwd)
                .assert()
                .success();
            Ok::<(), Box<dyn Error>>(())
        })?;

        Ok(())
    }
}
