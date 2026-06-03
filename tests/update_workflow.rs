#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{create_dir, exists, read_to_string, remove_file};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn update_workflow_description_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
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
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket with priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("updated workflow Open ticket"));

        let index_json = read_to_string(temp_dir.path().join("model/browser/data/index.json"))?;
        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            index_json.contains("\"description\": \"Actor opens a repair ticket with priority.\""),
            "browser index must preserve the updated workflow description"
        );
        assert!(
            workflow_json
                .contains("\"description\": \"Actor opens a repair ticket with priority.\""),
            "workflow browser data must preserve the updated workflow description"
        );
        assert!(
            lean.contains(
                "def workflowDescription := \"Actor opens a repair ticket with priority.\""
            ),
            "Lean artifact must represent the updated workflow description"
        );
        assert!(
            quint.contains(
                "val workflowDescription = \"Actor opens a repair ticket with priority.\""
            ),
            "Quint artifact must represent the updated workflow description"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_name_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
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
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open repair ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated workflow Open repair ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let index_json = read_to_string(temp_dir.path().join("model/browser/data/index.json"))?;
        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenRepairTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenRepairTicket.qnt"))?;

        assert!(
            index_json.contains("\"name\": \"Open repair ticket\""),
            "browser index must preserve the updated workflow name"
        );
        assert!(
            workflow_json.contains("\"name\": \"Open repair ticket\""),
            "workflow browser data must preserve the updated workflow name"
        );
        assert!(
            lean.contains("namespace OpenRepairTicket"),
            "Lean artifact must move to the updated workflow module"
        );
        assert!(
            lean.contains("def workflowName := \"Open repair ticket\""),
            "Lean artifact must represent the updated workflow name"
        );
        assert!(
            quint.contains("module OpenRepairTicket {"),
            "Quint artifact must move to the updated workflow module"
        );
        assert!(
            quint.contains("val workflowName = \"Open repair ticket\""),
            "Quint artifact must represent the updated workflow name"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/OpenTicket.lean"))?,
            "workflow name update must remove the old Lean module"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/OpenTicket.qnt"))?,
            "workflow name update must remove the old Quint module"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_name_rejects_formal_module_name_collisions() -> Result<(), Box<dyn Error>> {
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
                "open-bug",
                "--name",
                "Open bug",
                "--description",
                "Actor opens a bug report.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-bug",
                "--name",
                "Open-ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow module OpenTicket already exists",
            ));

        let index_json = read_to_string(temp_dir.path().join("model/browser/data/index.json"))?;
        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-bug.eventmodel.json"),
        )?;

        assert!(
            index_json.contains("\"name\": \"Open bug\""),
            "rejected workflow rename must not mutate the browser index"
        );
        assert!(
            workflow_json.contains("\"name\": \"Open bug\""),
            "rejected workflow rename must not mutate the workflow document"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_name_rejects_missing_formal_source_modules() -> Result<(), Box<dyn Error>> {
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

        remove_file(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        remove_file(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open repair ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow open-ticket is not modeled",
            ));

        Ok(())
    }

    #[test]
    fn update_workflow_name_rejects_stale_formal_module_cleanup_errors()
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

        let stale_lean_module = temp_dir.path().join("model/lean/OpenTicket.lean");
        remove_file(&stale_lean_module)?;
        create_dir(&stale_lean_module)?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open repair ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure();

        assert!(
            !exists(temp_dir.path().join("model/lean/OpenRepairTicket.lean"))?,
            "failed stale-module cleanup must not write the new Lean module"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_name_rejects_non_slug_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--workflow",
                "open-ticket",
                "--name",
                "Open repair ticket",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc init --name <project-name>",
            ));

        Ok(())
    }

    #[test]
    fn update_workflow_name_rejects_non_name_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--title",
                "Open repair ticket",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc init --name <project-name>",
            ));

        Ok(())
    }

    #[test]
    fn update_workflow_description_preserves_composed_slice_artifacts() -> Result<(), Box<dyn Error>>
    {
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
                "Actor opens a repair ticket with priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("updated workflow Open ticket"));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_json.contains("\"../slices/capture-ticket.eventmodel.json\""),
            "workflow update must preserve composed slice file references"
        );
        assert!(
            workflow_json.contains("\"slice\": \"capture-ticket\""),
            "workflow update must preserve composed workflow steps"
        );
        assert!(
            lean.contains(
                "def workflowSliceDetails : List (String × String × String × String) := [(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Actor enters repair ticket details.\")]"
            ),
            "Lean update must preserve composed slice details"
        );
        assert!(
            quint.contains(
                "val workflowSliceDetails = [{ slug: \"capture-ticket\", name: \"Capture ticket\", kind: \"state_view\", description: \"Actor enters repair ticket details.\" }]"
            ),
            "Quint update must preserve composed slice details"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_description_preserves_existing_canonical_transitions()
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

        add_slice(temp_dir.path(), "submit-ticket", "Submit ticket")?;
        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

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
            .success();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket with priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"submit-ticket\", target := \"review-ticket\", kind := \"event\", trigger := \"TicketSubmittedForReview\" }]"
            ),
            "Lean update must preserve existing event transitions"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"submit-ticket\", target: \"review-ticket\", kind: \"event\", trigger: \"TicketSubmittedForReview\" }]"
            ),
            "Quint update must preserve existing event transitions"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_description_preserves_existing_workflow_exit_transition()
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

        add_slice(temp_dir.path(), "capture-ticket", "Capture ticket")?;

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
            .success();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket with priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := \"workflow_exit:outcome\", trigger := \"ticket_closed\" }]"
            ),
            "Lean update must preserve existing workflow exit transitions"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: \"workflow_exit:outcome\", trigger: \"ticket_closed\" }]"
            ),
            "Quint update must preserve existing workflow exit transitions"
        );

        Ok(())
    }

    fn add_slice(cwd: &Path, slug: &str, name: &str) -> Result<(), Box<dyn Error>> {
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
                "Slice description.",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    #[test]
    fn remove_workflow_deletes_index_entry_and_owned_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_workflow(temp_dir.path(), "open-ticket", "Open ticket")?;
        add_workflow(temp_dir.path(), "close-ticket", "Close ticket")?;
        add_slice_to_workflow(
            temp_dir.path(),
            "open-ticket",
            "capture-request",
            "Capture request",
        )?;

        Command::cargo_bin("emc")?
            .args(["remove", "workflow", "--slug", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("removed workflow Open ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let index_json = read_to_string(temp_dir.path().join("model/browser/data/index.json"))?;
        assert!(
            !index_json.contains("open-ticket"),
            "removed workflow must be removed from the browser index"
        );
        assert!(
            index_json.contains("close-ticket"),
            "unrelated workflow must remain indexed"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/browser/data/workflows/open-ticket.eventmodel.json")
            )?,
            "removed workflow browser JSON must be deleted"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/browser/data/slices/capture-request.eventmodel.json")
            )?,
            "owned slice browser JSON must be deleted"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/OpenTicket.lean"))?,
            "removed workflow Lean module must be deleted"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/OpenTicket.qnt"))?,
            "removed workflow Quint module must be deleted"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/lean/slices/CaptureRequest.lean")
            )?,
            "owned slice Lean module must be deleted"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/quint/slices/CaptureRequest.qnt")
            )?,
            "owned slice Quint module must be deleted"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_rejects_incoming_workflow_exit_references() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_workflow(temp_dir.path(), "triage-ticket", "Triage ticket")?;
        add_workflow(temp_dir.path(), "close-ticket", "Close ticket")?;
        add_slice_to_workflow(
            temp_dir.path(),
            "triage-ticket",
            "assess-request",
            "Assess request",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "triage-ticket",
                "--from",
                "assess-request",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "Close accepted request",
                "--reason",
                "Accepted request can be closed.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["remove", "workflow", "--slug", "close-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow close-ticket is referenced by workflow triage-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            exists(
                temp_dir
                    .path()
                    .join("model/browser/data/workflows/close-ticket.eventmodel.json")
            )?,
            "rejected workflow removal must leave workflow artifacts in place"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_requires_exact_command_shape() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_workflow(temp_dir.path(), "open-ticket", "Open ticket")?;

        Command::cargo_bin("emc")?
            .args(["remove", "model", "--slug", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc init --name <project-name>",
            ));

        assert!(
            exists(
                temp_dir
                    .path()
                    .join("model/browser/data/workflows/open-ticket.eventmodel.json")
            )?,
            "malformed workflow removal command must not delete workflow artifacts"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_allows_unrelated_workflow_exit_references() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_workflow(temp_dir.path(), "triage-ticket", "Triage ticket")?;
        add_workflow(temp_dir.path(), "close-ticket", "Close ticket")?;
        add_workflow(temp_dir.path(), "archive-ticket", "Archive ticket")?;
        add_slice_to_workflow(
            temp_dir.path(),
            "triage-ticket",
            "assess-request",
            "Assess request",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "triage-ticket",
                "--from",
                "assess-request",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "Close accepted request",
                "--reason",
                "Accepted request can be closed.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["remove", "workflow", "--slug", "archive-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("removed workflow Archive ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            exists(
                temp_dir
                    .path()
                    .join("model/browser/data/workflows/close-ticket.eventmodel.json")
            )?,
            "unrelated workflow exit target must remain modeled"
        );

        Ok(())
    }

    fn add_workflow(path: &Path, slug: &str, name: &str) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                slug,
                "--name",
                name,
                "--description",
                "Actor completes the workflow.",
            ])
            .current_dir(path)
            .assert()
            .success();
        Ok(())
    }

    fn add_slice_to_workflow(
        path: &Path,
        workflow: &str,
        slug: &str,
        name: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                workflow,
                "--slug",
                slug,
                "--name",
                name,
                "--type",
                "state_change",
                "--description",
                "Actor captures the request.",
            ])
            .current_dir(path)
            .assert()
            .success();
        Ok(())
    }
}
