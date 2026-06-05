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

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            slice_lean.contains("namespace CaptureTicket"),
            "Lean slice artifact must use the business slice module name"
        );
        assert!(
            slice_lean.contains(
                "-- EMC-DIGEST: slice:name=Capture ticket;slug=capture-ticket;kind=state_view;description=Actor enters repair ticket details."
            ),
            "Lean slice artifact must carry a deterministic business slice digest"
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
            slice_quint.contains(
                "// EMC-DIGEST: slice:name=Capture ticket;slug=capture-ticket;kind=state_view;description=Actor enters repair ticket details."
            ),
            "Quint slice artifact must carry a deterministic business slice digest"
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
            lean.contains(
                "def workflowSliceModules : List (String × String) := [(\"capture-ticket\", \"CaptureTicket\")]"
            ),
            "Lean artifact must represent the formal module composed for each workflow slice"
        );
        assert!(
            lean.contains(
                "theorem workflowSlicesHaveModuleReferences : workflowSlices.length = workflowSliceModules.length := rfl"
            ),
            "Lean artifact must prove every workflow slice has a composed formal module reference"
        );
        assert!(
            quint.contains("val workflowSlices: List[str] = [\"capture-ticket\"]"),
            "Quint artifact must represent the workflow's business slices"
        );
        assert!(
            quint.contains(
                "val workflowSliceDetails: List[WorkflowSliceDetail] = [{ slug: \"capture-ticket\", name: \"Capture ticket\", kind: \"state_view\", description: \"Actor enters repair ticket details.\" }]"
            ),
            "Quint artifact must represent the workflow's business slice details"
        );
        assert!(
            quint.contains("type WorkflowSliceModule = { slice: str, formalModule: str }"),
            "Quint artifact must type the formal module composed for each workflow slice"
        );
        assert!(
            quint.contains(
                "val workflowSliceModules: List[WorkflowSliceModule] = [{ slice: \"capture-ticket\", formalModule: \"CaptureTicket\" }]"
            ),
            "Quint artifact must represent the formal module composed for each workflow slice"
        );
        assert!(
            quint.contains(
                "val workflowSliceModulesComplete = workflowSlices.length() == workflowSliceModules.length()"
            ),
            "Quint artifact must verify every workflow slice has a composed formal module reference"
        );
        assert!(
            lean_root.contains("def modelSlices : List (String × String) := [(\"open-ticket\", \"capture-ticket\")]"),
            "Lean project root must represent composed workflow-to-slice membership"
        );
        assert!(
            lean_root.contains(
                "def modelSliceModules : List (String × String × String) := [(\"open-ticket\", \"capture-ticket\", \"CaptureTicket\")]"
            ),
            "Lean project root must represent the formal module composed for each workflow slice"
        );
        assert!(
            lean_root.contains(
                "def modelDigest := \"project:name=Repair Desk;version=0.1.0;workflows=open-ticket;slices=open-ticket/capture-ticket@CaptureTicket;commands=;read-models=;streams=;events=\""
            ),
            "Lean project root digest must include composed workflow, slice, and module membership"
        );
        assert!(
            lean_root.contains(
                "theorem modelDigestIsStable : modelDigest = \"project:name=Repair Desk;version=0.1.0;workflows=open-ticket;slices=open-ticket/capture-ticket@CaptureTicket;commands=;read-models=;streams=;events=\" := rfl"
            ),
            "Lean project root must prove composed model digest stability"
        );
        assert!(
            lean_root.contains("theorem modelSlicesAreDeclared : modelSlices.length = 1 := rfl"),
            "Lean project root must prove composed slice membership is declared"
        );
        assert!(
            lean_root.contains(
                "theorem modelSliceModulesAreDeclared : modelSliceModules.length = 1 := rfl"
            ),
            "Lean project root must prove composed slice modules are declared"
        );
        assert!(
            quint_root.contains("val modelSlices: List[ModelSlice] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\" }]"),
            "Quint project root must represent composed workflow-to-slice membership"
        );
        assert!(
            quint_root.contains(
                "type ModelSliceModule = { workflow: str, slice: str, formalModule: str }"
            ),
            "Quint project root must type the formal module composed for each workflow slice"
        );
        assert!(
            quint_root.contains("val modelSliceModules: List[ModelSliceModule] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", formalModule: \"CaptureTicket\" }]"),
            "Quint project root must represent the formal module composed for each workflow slice"
        );
        assert!(
            quint_root.contains(
                "val modelDigest = \"project:name=Repair Desk;version=0.1.0;workflows=open-ticket;slices=open-ticket/capture-ticket@CaptureTicket;commands=;read-models=;streams=;events=\""
            ),
            "Quint project root digest must include composed workflow, slice, and module membership"
        );
        assert!(
            quint_root.contains(
                "val modelDigestStable = modelDigest == \"project:name=Repair Desk;version=0.1.0;workflows=open-ticket;slices=open-ticket/capture-ticket@CaptureTicket;commands=;read-models=;streams=;events=\""
            ),
            "Quint project root must verify composed model digest stability"
        );
        assert!(
            quint_root.contains("val modelSlicesAreDeclared = modelSlices.length() == 1"),
            "Quint project root must verify composed slice membership is declared"
        );
        assert!(
            quint_root
                .contains("val modelSliceModulesAreDeclared = modelSliceModules.length() == 1"),
            "Quint project root must verify composed slice modules are declared"
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
    fn add_slice_rejects_duplicate_formal_slice_module_names() -> Result<(), Box<dyn Error>> {
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
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket-copy",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Alternate capture flow.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice module CaptureTicket already exists",
            ));

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            slice_lean.contains("def sliceSlug := \"capture-ticket\""),
            "rejected duplicate module names must not overwrite the existing formal slice artifact"
        );

        Ok(())
    }

    #[test]
    fn add_slice_rejects_duplicate_slice_slugs() -> Result<(), Box<dyn Error>> {
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
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket copy",
                "--type",
                "state_view",
                "--description",
                "Alternate capture flow.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice capture-ticket already exists",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert_eq!(
            lean.matches(
                "(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Slice description.\")"
            )
            .count(),
            1,
            "rejected duplicate slice slugs must not add another workflow step"
        );

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            slice_lean.contains("def sliceName := \"Capture ticket\""),
            "rejected duplicate slice slugs must not overwrite existing formal slice data"
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := \"command\", trigger := \"SubmitTicketForReview\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "Lean artifact must preserve existing command transitions when a later slice is added"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: \"command\", trigger: \"SubmitTicketForReview\", rationale: \"\", payloadContract: \"\" }]"
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := \"external_trigger\", trigger := \"callback_received\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "Lean artifact must preserve existing external-trigger transitions when a later slice is added"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"record-callback\", kind: \"external_trigger\", trigger: \"callback_received\", rationale: \"\", payloadContract: \"\" }]"
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := \"workflow_exit:outcome\", trigger := \"ticket_closed\", rationale := \"Closed tickets continue to completion.\", payloadContract := \"\" }]"
            ),
            "Lean artifact must preserve existing workflow exit transitions when a later slice is added"
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: \"workflow_exit:outcome\", trigger: \"ticket_closed\", rationale: \"Closed tickets continue to completion.\", payloadContract: \"\" }]"
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
