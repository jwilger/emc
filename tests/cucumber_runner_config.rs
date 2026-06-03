#[cfg(test)]
mod tests {
    use std::error::Error;

    use assert_cmd::Command;
    use predicates::prelude::PredicateBooleanExt;
    use predicates::prelude::predicate;

    #[test]
    fn gherkin_runner_lists_browser_feature_paths_without_execution() -> Result<(), Box<dyn Error>>
    {
        Command::cargo_bin("emc")?
            .args(["gherkin", "list", "--suite", "browser"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "tests/features/event_model_browser/timeline_rendering.feature",
            ))
            .stdout(predicate::str::contains("Scenario:").not());

        Ok(())
    }

    #[test]
    fn gherkin_runner_lists_all_event_model_feature_suites() -> Result<(), Box<dyn Error>> {
        let suites = [
            (
                "meta",
                "tests/features/event_model_cucumber_execution.feature",
            ),
            (
                "validator",
                "tests/features/event_model_validator/board_timeline_and_workflow.feature",
            ),
            (
                "review-gate",
                "tests/features/event_model_review_gate/workflow_review_gate.feature",
            ),
        ];

        suites
            .iter()
            .map(|(suite, expected_path)| {
                Command::cargo_bin("emc")?
                    .args(["gherkin", "list", "--suite", suite])
                    .assert()
                    .success()
                    .stdout(predicate::str::contains(*expected_path))
                    .stdout(predicate::str::contains("Scenario:").not());

                Ok(())
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

        Ok(())
    }

    #[test]
    fn gherkin_runner_run_fails_undefined_pending_or_skipped_steps() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["gherkin", "run", "--suite", "browser"])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "undefined, pending, or skipped Gherkin steps are failures for browser",
            ));

        Ok(())
    }
}
