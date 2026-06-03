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

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let slice_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/slices/capture-ticket.eventmodel.json"),
        )?;
        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            workflow_json
                .contains("\"description\": \"Actor enters repair ticket details and priority.\""),
            "workflow composition must preserve the updated slice description"
        );
        assert!(
            slice_json
                .contains("\"description\": \"Actor enters repair ticket details and priority.\""),
            "slice browser data must preserve the updated slice description"
        );
        assert!(
            workflow_lean.contains(
                "(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Actor enters repair ticket details and priority.\")"
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

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let slice_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/slices/capture-ticket.eventmodel.json"),
        )?;
        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            workflow_json.contains("\"type\": \"automation\""),
            "workflow composition must preserve the updated slice kind"
        );
        assert!(
            slice_json.contains("\"type\": \"automation\""),
            "slice browser data must preserve the updated slice kind"
        );
        assert!(
            workflow_lean.contains(
                "(\"capture-ticket\", \"Capture ticket\", \"automation\", \"Actor enters repair ticket details.\")"
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

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let slice_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/slices/capture-ticket.eventmodel.json"),
        )?;
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
            workflow_json.contains("\"name\": \"Capture repair ticket\""),
            "workflow composition must preserve the updated slice name"
        );
        assert!(
            slice_json.contains("\"name\": \"Capture repair ticket\""),
            "slice browser data must preserve the updated slice name"
        );
        assert!(
            workflow_lean.contains(
                "(\"capture-ticket\", \"Capture repair ticket\", \"state_view\", \"Actor enters repair ticket details.\")"
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

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let slice_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/slices/review-ticket.eventmodel.json"),
        )?;

        assert!(
            workflow_json.contains("\"name\": \"Review ticket\""),
            "rejected slice rename must not mutate the workflow document"
        );
        assert!(
            slice_json.contains("\"name\": \"Review ticket\""),
            "rejected slice rename must not mutate the slice document"
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
                "usage: emc init --name <project-name>",
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
                "usage: emc init --name <project-name>",
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
                "usage: emc init --name <project-name>",
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
                "usage: emc init --name <project-name>",
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
}
