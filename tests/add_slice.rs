#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn add_slice_updates_workflow_composition_and_slice_data() -> Result<(), Box<dyn Error>> {
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

        let initial_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let initial_digest = digest_marker(&initial_lean)
            .ok_or("generated workflow artifact is missing an initial digest")?;

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
            .success()
            .stdout(predicate::str::contains("added slice Capture ticket"));

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
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_json.contains("\"../slices/capture-ticket.eventmodel.json\""),
            "workflow must reference the new slice file"
        );
        assert!(
            workflow_json.contains("\"slice\": \"capture-ticket\""),
            "workflow must add a step for the new slice"
        );
        assert!(
            workflow_json.contains("\"relationship\": \"entry\""),
            "first slice must become the workflow entry"
        );
        assert!(
            slice_json.contains("\"name\": \"Capture ticket\""),
            "slice data must preserve the business slice name"
        );
        assert!(
            slice_json.contains("\"type\": \"state_view\""),
            "slice data must preserve the semantic slice type"
        );
        assert!(
            slice_json.contains("\"description\": \"Actor enters repair ticket details.\""),
            "slice data must preserve the slice description"
        );
        assert!(
            slice_lean.contains("namespace CaptureTicket"),
            "Lean slice artifact must use the business slice module name"
        );
        assert!(
            slice_lean.contains("def sliceName := \"Capture ticket\""),
            "Lean slice artifact must represent the business slice name"
        );
        assert!(
            slice_lean.contains("def sliceSlug := \"capture-ticket\""),
            "Lean slice artifact must represent the business slice slug"
        );
        assert!(
            slice_lean.contains("def sliceKind := \"state_view\""),
            "Lean slice artifact must represent the semantic slice kind"
        );
        assert!(
            slice_lean.contains("def sliceDescription := \"Actor enters repair ticket details.\""),
            "Lean slice artifact must represent the business slice description"
        );
        assert!(
            slice_quint.contains("module CaptureTicket"),
            "Quint slice artifact must use the business slice module name"
        );
        assert!(
            slice_quint.contains("val sliceName = \"Capture ticket\""),
            "Quint slice artifact must represent the business slice name"
        );
        assert!(
            slice_quint.contains("val sliceSlug = \"capture-ticket\""),
            "Quint slice artifact must represent the business slice slug"
        );
        assert!(
            slice_quint.contains("val sliceKind = \"state_view\""),
            "Quint slice artifact must represent the semantic slice kind"
        );
        assert!(
            slice_quint.contains("val sliceDescription = \"Actor enters repair ticket details.\""),
            "Quint slice artifact must represent the business slice description"
        );
        assert!(
            lean.contains("def workflowSlices : List String := [\"capture-ticket\"]"),
            "Lean artifact must represent the workflow's business slices"
        );
        assert!(
            lean.contains(
                "def workflowSliceDetails : List (String × String × String × String) := [(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Actor enters repair ticket details.\")]"
            ),
            "Lean artifact must represent the workflow's business slice details"
        );
        assert!(
            quint.contains("val workflowSlices = [\"capture-ticket\"]"),
            "Quint artifact must represent the workflow's business slices"
        );
        assert!(
            quint.contains(
                "val workflowSliceDetails = [{ slug: \"capture-ticket\", name: \"Capture ticket\", kind: \"state_view\", description: \"Actor enters repair ticket details.\" }]"
            ),
            "Quint artifact must represent the workflow's business slice details"
        );
        assert_ne!(
            initial_digest,
            digest_marker(&lean).ok_or("Lean artifact is missing an updated digest")?,
            "Lean digest must change when the composed workflow slice details change"
        );
        assert_ne!(
            initial_digest,
            digest_marker(&quint).ok_or("Quint artifact is missing an updated digest")?,
            "Quint digest must change when the composed workflow slice details change"
        );

        Ok(())
    }

    #[test]
    fn add_slice_preserves_existing_canonical_workflow_transitions() -> Result<(), Box<dyn Error>> {
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
        add_slice(temp_dir.path(), "submit-ticket", "Submit ticket")?;

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

        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := \"command\", trigger := \"SubmitTicketForReview\" }]"
            ),
            "Lean artifact must preserve existing command transitions when a later slice is added"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: \"command\", trigger: \"SubmitTicketForReview\" }]"
            ),
            "Quint artifact must preserve existing command transitions when a later slice is added"
        );

        Ok(())
    }

    #[test]
    fn add_slice_preserves_existing_external_trigger_transition() -> Result<(), Box<dyn Error>> {
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
        add_slice(temp_dir.path(), "record-callback", "Record callback")?;

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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := \"external_trigger\", trigger := \"callback_received\" }]"
            ),
            "Lean artifact must preserve existing external-trigger transitions when a later slice is added"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"record-callback\", kind: \"external_trigger\", trigger: \"callback_received\" }]"
            ),
            "Quint artifact must preserve existing external-trigger transitions when a later slice is added"
        );

        Ok(())
    }

    #[test]
    fn add_slice_preserves_existing_workflow_exit_transition() -> Result<(), Box<dyn Error>> {
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

        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := \"workflow_exit:outcome\", trigger := \"ticket_closed\" }]"
            ),
            "Lean artifact must preserve existing workflow exit transitions when a later slice is added"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: \"workflow_exit:outcome\", trigger: \"ticket_closed\" }]"
            ),
            "Quint artifact must preserve existing workflow exit transitions when a later slice is added"
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

    fn digest_marker(contents: &str) -> Option<String> {
        contents.lines().find_map(|line| {
            line.trim()
                .strip_prefix("-- EMC-DIGEST: ")
                .or_else(|| line.trim().strip_prefix("// EMC-DIGEST: "))
                .map(str::to_owned)
        })
    }
}
