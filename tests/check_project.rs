#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, remove_file, write};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn check_reports_initialized_project_artifacts_are_present() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
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
    fn check_uses_the_initialized_project_name() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Inventory Intake"])
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
    fn check_reports_missing_project_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        remove_file(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/quint/RepairDesk.qnt",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_verification_entrypoint_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        remove_file(temp_dir.path().join("model/lean/lakefile.lean"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/lean/lakefile.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_modeled_workflow_artifacts() -> Result<(), Box<dyn Error>> {
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

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/lean/OpenTicket.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unmodeled_lean_workflow_artifacts() -> Result<(), Box<dyn Error>> {
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

        write(
            temp_dir.path().join("model/lean/OrphanWorkflow.lean"),
            "namespace OrphanWorkflow\n\nend OrphanWorkflow\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean model artifact drift for OrphanWorkflow.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unmodeled_quint_workflow_artifacts() -> Result<(), Box<dyn Error>> {
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

        write(
            temp_dir.path().join("model/quint/OrphanWorkflow.qnt"),
            "module OrphanWorkflow\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint model artifact drift for OrphanWorkflow.qnt",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_modeled_workflow_artifact_drift() -> Result<(), Box<dyn Error>> {
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

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\ndef workflowName := \"Changed\"\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "artifact digest mismatch for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_drift() -> Result<(), Box<dyn Error>> {
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

        write(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
            "{\n  \"name\": \"Changed\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Actor opens a repair ticket.\",\n  \"slice_files\": [],\n  \"steps\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unindexed_browser_workflow_files() -> Result<(), Box<dyn Error>> {
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

        write(
            temp_dir.path().join("model/browser/data/index.json"),
            "{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow index drift for open-ticket.eventmodel.json",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_field_drift() -> Result<(), Box<dyn Error>> {
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

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Changed\"\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow field drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_name_declaration_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str("\ndef workflowName := \"Stale ticket\"\n");
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow field drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_description_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str("  val workflowDescription = \"Stale description.\"\n");
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow field drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_transition_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices : List String := [\"capture-ticket\",\"review-ticket\"]\n\ndef workflowSliceDetails : List (String × String × String × String) := [(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Slice description.\"),(\"review-ticket\", \"Review ticket\", \"state_view\", \"Slice description.\")]\n\ndef workflowTransitions : List String := []\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_transition_declaration_drift() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str(
            "\ndef workflowTransitions : List String := [\"capture-ticket->review-ticket:navigation:stale-screen\"]\n",
        );
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_slice_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices : List String := []\n\ndef workflowTransitions : List String := [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_slice_declaration_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str("\ndef workflowSlices : List String := [\"stale-slice\"]\n");
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_slice_detail_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices : List String := [\"capture-ticket\",\"review-ticket\"]\n\ndef workflowSliceDetails : List (String × String × String × String) := []\n\ndef workflowTransitions : List String := [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_slice_detail_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str(
            "\ndef workflowSliceDetails : List (String × String × String × String) := [(\"stale-slice\", \"Stale slice\", \"state_view\", \"Stale description.\")]\n",
        );
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_transition_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket {\n  // EMC-DIGEST: workflow:Open ticket\n  val workflowName = \"Open ticket\"\n  val workflowSlug = \"open-ticket\"\n  val workflowDescription = \"Actor opens a repair ticket.\"\n  val workflowSlices = [\"capture-ticket\",\"review-ticket\"]\n  val workflowSliceDetails = [{ slug: \"capture-ticket\", name: \"Capture ticket\", kind: \"state_view\", description: \"Slice description.\" },{ slug: \"review-ticket\", name: \"Review ticket\", kind: \"state_view\", description: \"Slice description.\" }]\n  val workflowTransitions = []\n  val workflowIdentityStable = workflowName == \"Open ticket\"\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_transition_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str(
            "  val workflowTransitions = [\"capture-ticket->review-ticket:navigation:stale-screen\"]\n",
        );
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_slice_detail_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket {\n  // EMC-DIGEST: workflow:Open ticket\n  val workflowName = \"Open ticket\"\n  val workflowSlug = \"open-ticket\"\n  val workflowDescription = \"Actor opens a repair ticket.\"\n  val workflowSlices = [\"capture-ticket\",\"review-ticket\"]\n  val workflowSliceDetails = []\n  val workflowTransitions = [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]\n  val workflowIdentityStable = workflowName == \"Open ticket\"\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_slice_detail_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str(
            "  val workflowSliceDetails = [{ slug: \"stale-slice\", name: \"Stale slice\", kind: \"state_view\", description: \"Stale description.\" }]\n",
        );
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_slice_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket\n\n// EMC-DIGEST: workflow:Open ticket\nval workflowName = \"Open ticket\"\n\nval workflowSlug = \"open-ticket\"\n\nval workflowDescription = \"Actor opens a repair ticket.\"\n\nval workflowSlices = []\n\nval workflowTransitions = [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_slice_declaration_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str("  val workflowSlices = [\"stale-slice\"]\n");
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_referenced_browser_slice_file() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        remove_file(
            temp_dir
                .path()
                .join("model/browser/data/slices/capture-ticket.eventmodel.json"),
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow Open ticket references missing slice artifact",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unreferenced_browser_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        write(
            temp_dir
                .path()
                .join("model/browser/data/slices/orphan-slice.eventmodel.json"),
            "{\"name\":\"Orphan slice\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[],\"slices\":[{\"name\":\"Orphan slice\",\"type\":\"state_view\",\"events\":[],\"views\":[],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser slice reference drift for orphan-slice.eventmodel.json",
            ));

        Ok(())
    }

    fn create_connected_workflow(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
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
        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

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
}
