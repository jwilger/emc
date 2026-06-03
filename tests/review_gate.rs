#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, write};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::{PredicateBooleanExt, predicate};
    use tempfile::TempDir;

    #[test]
    fn review_gate_blocks_workflow_without_structured_review_record() -> Result<(), Box<dyn Error>>
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
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("workflow review is not clean"));

        Ok(())
    }

    #[test]
    fn review_gate_blocks_clean_record_without_required_categories() -> Result<(), Box<dyn Error>> {
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
            temp_dir.path().join("reviews/open-ticket.review.json"),
            format!(
                "{{\n  \"workflow_slug\": \"open-ticket\",\n  \"model_content_digest\": \"{}\",\n  \"reviewer_id\": \"event-model-reviewer\",\n  \"status\": \"clean\",\n  \"category_results\": {{}},\n  \"mandatory_findings\": [],\n  \"reviewed_at\": \"2026-06-01T00:00:00.000Z\"\n}}\n",
                current_model_digest(temp_dir.path())?
            ),
        )?;

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "clean review is missing category 'lifecycle-entry'",
            ));

        Ok(())
    }

    #[test]
    fn review_gate_blocks_clean_record_with_stale_model_digest() -> Result<(), Box<dyn Error>> {
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
            temp_dir.path().join("reviews/open-ticket.review.json"),
            clean_review_record("stale-digest"),
        )?;

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "clean review is stale for current model digest",
            ));

        Ok(())
    }

    #[test]
    fn review_gate_blocks_current_mandatory_findings() -> Result<(), Box<dyn Error>> {
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
            temp_dir.path().join("reviews/open-ticket.review.json"),
            review_record_with_current_mandatory_finding(&current_model_digest(temp_dir.path())?),
        )?;

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "mandatory review findings remain for current model digest",
            ));

        Ok(())
    }

    #[test]
    fn review_gate_blocks_non_clean_review_without_mandatory_findings() -> Result<(), Box<dyn Error>>
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

        write(
            temp_dir.path().join("reviews/open-ticket.review.json"),
            review_record_without_mandatory_findings("stale-digest"),
        )?;

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("workflow review is not clean"))
            .stderr(
                predicate::str::contains("corrected workflow requires clean follow-up review")
                    .not(),
            );

        Ok(())
    }

    #[test]
    fn review_gate_blocks_review_record_for_different_workflow() -> Result<(), Box<dyn Error>> {
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
            temp_dir.path().join("reviews/open-ticket.review.json"),
            clean_review_record_for("other-ticket", &current_model_digest(temp_dir.path())?),
        )?;

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "review record workflow 'other-ticket' does not match 'open-ticket'",
            ));

        Ok(())
    }

    #[test]
    fn review_gate_blocks_corrected_workflow_without_follow_up_review() -> Result<(), Box<dyn Error>>
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

        let previous_digest = current_model_digest(temp_dir.path())?;
        write(
            temp_dir.path().join("reviews/open-ticket.review.json"),
            review_record_with_current_mandatory_finding(&previous_digest),
        )?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket after correction.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "corrected workflow requires clean follow-up review",
            ));

        Ok(())
    }

    #[test]
    fn review_record_command_stores_clean_review_for_current_workflow_digest()
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

        let current_digest = current_model_digest(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "review",
                "record",
                "--workflow",
                "open-ticket",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "recorded clean review for workflow open-ticket",
            ));

        let review_record =
            read_to_string(temp_dir.path().join("reviews/open-ticket.review.json"))?;
        assert!(
            review_record.contains(&format!("\"model_content_digest\": \"{current_digest}\"")),
            "review record must bind the clean review to the current workflow digest"
        );
        assert!(
            review_record.contains("\"reviewed_at\": \"2026-06-03T00:00:00.000Z\""),
            "review record must preserve the supplied review timestamp"
        );

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("workflow review is clean"));

        Ok(())
    }

    #[test]
    fn review_record_rejects_invalid_modeled_workflows() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_navigation_chain(temp_dir.path())?;

        remove_first_formal_workflow_transition_fixture(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "review",
                "record",
                "--workflow",
                "intake-visit",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow step 'triage-intake' has no incoming transition",
            ));

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "intake-visit"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow step 'triage-intake' has no incoming transition",
            ));

        Ok(())
    }

    #[test]
    fn review_record_uses_formal_artifacts_when_browser_workflow_is_stale()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_navigation_chain(temp_dir.path())?;

        remove_first_browser_workflow_transition_fixture(temp_dir.path(), "intake-visit")?;

        Command::cargo_bin("emc")?
            .args([
                "review",
                "record",
                "--workflow",
                "intake-visit",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "recorded clean review for workflow intake-visit",
            ));

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "intake-visit"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("workflow review is clean"));

        Ok(())
    }

    #[test]
    fn review_record_command_rejects_unrecognized_argument_shapes() -> Result<(), Box<dyn Error>> {
        [
            [
                "review",
                "unknown",
                "--workflow",
                "open-ticket",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ],
            [
                "review",
                "record",
                "--slug",
                "open-ticket",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ],
            [
                "review",
                "record",
                "--workflow",
                "open-ticket",
                "--actor",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ],
            [
                "review",
                "record",
                "--workflow",
                "open-ticket",
                "--reviewer",
                "event-model-reviewer",
                "--at",
                "2026-06-03T00:00:00.000Z",
            ],
        ]
        .into_iter()
        .try_for_each(|arguments| {
            Command::cargo_bin("emc")?
                .args(arguments)
                .assert()
                .failure()
                .stderr(predicate::str::contains(
                    "usage: emc <command> [arguments]; run emc --help",
                ));

            Ok::<(), Box<dyn Error>>(())
        })
    }

    #[test]
    fn review_record_command_rejects_invalid_review_timestamps() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "review",
                "record",
                "--workflow",
                "open-ticket",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "not-a-timestamp",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid review timestamp"))
            .stderr(predicate::str::contains(
                "expected UTC millisecond timestamp like 2026-06-03T12:00:00.000Z",
            ));

        Ok(())
    }

    fn clean_review_record(model_content_digest: &str) -> String {
        clean_review_record_for("open-ticket", model_content_digest)
    }

    fn clean_review_record_for(workflow_slug: &str, model_content_digest: &str) -> String {
        format!(
            "{{\n  \"workflow_slug\": \"{workflow_slug}\",\n  \"model_content_digest\": \"{model_content_digest}\",\n  \"reviewer_id\": \"event-model-reviewer\",\n  \"status\": \"clean\",\n  \"category_results\": {{\n    \"lifecycle-entry\": \"clean\",\n    \"canonical-lanes\": \"clean\",\n    \"board-connections\": \"clean\",\n    \"fake-intermediates\": \"clean\",\n    \"slice-ownership\": \"clean\",\n    \"source-chains\": \"clean\",\n    \"workflow-reachability\": \"clean\",\n    \"transition-resolution\": \"clean\",\n    \"navigation-targets\": \"clean\",\n    \"branch-shape\": \"clean\",\n    \"outcomes-and-errors\": \"clean\",\n    \"scenario-coverage\": \"clean\",\n    \"timeline-rendering\": \"clean\"\n  }},\n  \"mandatory_findings\": [],\n  \"reviewed_at\": \"2026-06-01T00:00:00.000Z\"\n}}\n"
        )
    }

    fn review_record_with_current_mandatory_finding(model_content_digest: &str) -> String {
        format!(
            "{{\n  \"workflow_slug\": \"open-ticket\",\n  \"model_content_digest\": \"{model_content_digest}\",\n  \"reviewer_id\": \"event-model-reviewer\",\n  \"status\": \"changes_requested\",\n  \"category_results\": {{}},\n  \"mandatory_findings\": [\n    {{\n      \"summary\": \"bad board lane\",\n      \"model_content_digest\": \"{model_content_digest}\"\n    }}\n  ],\n  \"reviewed_at\": \"2026-06-01T00:00:00.000Z\"\n}}\n"
        )
    }

    fn review_record_without_mandatory_findings(model_content_digest: &str) -> String {
        format!(
            "{{\n  \"workflow_slug\": \"open-ticket\",\n  \"model_content_digest\": \"{model_content_digest}\",\n  \"reviewer_id\": \"event-model-reviewer\",\n  \"status\": \"changes_requested\",\n  \"category_results\": {{}},\n  \"mandatory_findings\": [],\n  \"reviewed_at\": \"2026-06-01T00:00:00.000Z\"\n}}\n"
        )
    }

    fn current_model_digest(project_root: &Path) -> Result<String, Box<dyn Error>> {
        let workflow_path = "model/browser/data/workflows/open-ticket.eventmodel.json";
        let workflow_contents = read_to_string(project_root.join(workflow_path))?;
        let mut digest = StableDigest::new();
        digest.write(workflow_path);
        digest.write(&workflow_contents);
        Ok(digest.finish())
    }

    fn initialize_navigation_chain(cwd: &Path) -> Result<(), Box<dyn Error>> {
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

    fn remove_first_browser_workflow_transition_fixture(
        cwd: &Path,
        workflow: &str,
    ) -> Result<(), Box<dyn Error>> {
        let path = cwd.join(format!(
            "model/browser/data/workflows/{workflow}.eventmodel.json"
        ));
        let mut workflow_json: serde_json::Value = serde_json::from_str(&read_to_string(&path)?)?;
        let steps = workflow_json
            .get_mut("steps")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("workflow fixture is missing steps")?;
        let transitions = steps
            .first_mut()
            .and_then(|step| step.get_mut("transitions"))
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("workflow fixture first step is missing transitions")?;
        transitions.clear();
        write(path, serde_json::to_string_pretty(&workflow_json)?)?;
        Ok(())
    }

    fn remove_first_formal_workflow_transition_fixture(cwd: &Path) -> Result<(), Box<dyn Error>> {
        let lean_path = cwd.join("model/lean/IntakeVisit.lean");
        let lean = read_to_string(&lean_path)?;
        write(
            lean_path,
            lean.replace(
                "{ source := \"capture-intake\", target := \"triage-intake\", kind := \"navigation\", trigger := \"triage-intake-screen\", rationale := \"\" },",
                "",
            ),
        )?;

        let quint_path = cwd.join("model/quint/IntakeVisit.qnt");
        let quint = read_to_string(&quint_path)?;
        write(
            quint_path,
            quint.replace(
                "{ source: \"capture-intake\", target: \"triage-intake\", kind: \"navigation\", trigger: \"triage-intake-screen\", rationale: \"\" },",
                "",
            ),
        )?;

        Ok(())
    }

    struct StableDigest {
        value: u64,
    }

    impl StableDigest {
        fn new() -> Self {
            Self {
                value: 0xcbf2_9ce4_8422_2325,
            }
        }

        fn write(&mut self, value: &str) {
            value.as_bytes().iter().for_each(|byte| {
                self.value ^= u64::from(*byte);
                self.value = self.value.wrapping_mul(0x0000_0100_0000_01b3);
            });
        }

        fn finish(self) -> String {
            format!("emc-fnv1a64:{:016x}", self.value)
        }
    }
}
