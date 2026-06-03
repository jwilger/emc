#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn validator_gherkin_is_checked_in_as_emc_rule_fixtures() -> Result<(), Box<dyn Error>> {
        let validator_features = [
            ("board_timeline_and_workflow.feature", 54_usize),
            ("outcomes_errors_and_review.feature", 20_usize),
            ("slice_architecture.feature", 28_usize),
            ("structure_and_sources.feature", 31_usize),
            ("views_controls_and_information.feature", 26_usize),
        ];

        let violations = validator_features
            .iter()
            .filter_map(|(file_name, expected_scenarios)| {
                let path = validator_fixture_path(file_name);
                match fs::read_to_string(&path) {
                    Ok(source) => {
                        let observed_scenarios = count_scenarios(&source);
                        (observed_scenarios != *expected_scenarios).then(|| {
                            format!(
                                "{} has {observed_scenarios} scenarios; expected {expected_scenarios}",
                                path.display()
                            )
                        })
                    }
                    Err(error) => Some(format!("{} is missing: {error}", path.display())),
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "EMC must check in the validator Gherkin as validation-rule fixtures"
        );

        Ok(())
    }

    #[test]
    fn validator_gherkin_does_not_describe_commands_as_reading_read_models()
    -> Result<(), Box<dyn Error>> {
        let violations = validator_feature_sources()?
            .into_iter()
            .flat_map(|(path, source)| {
                source
                    .lines()
                    .enumerate()
                    .filter_map(|(index, line)| {
                        line.contains("command")
                            .then_some(line)
                            .filter(|line| line.contains("reads read model"))
                            .map(|line| {
                                format!(
                                    "{}:{} describes command/read-model reads: {line}",
                                    path.display(),
                                    index + 1
                                )
                            })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "commands receive input from invocation arguments and event streams, not read models"
        );

        Ok(())
    }

    #[test]
    fn review_gate_gherkin_is_checked_in_as_emc_rule_fixtures() -> Result<(), Box<dyn Error>> {
        let path = workspace_root()
            .join("tests/features/event_model_review_gate")
            .join("workflow_review_gate.feature");
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("{} is missing: {error}", path.display()))?;

        assert_eq!(
            count_scenarios(&source),
            9,
            "EMC must check in the review-gate Gherkin as review-rule fixtures"
        );

        Ok(())
    }

    #[test]
    fn browser_gherkin_is_checked_in_as_emc_rule_fixtures() -> Result<(), Box<dyn Error>> {
        let path = workspace_root()
            .join("tests/features/event_model_browser")
            .join("timeline_rendering.feature");
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("{} is missing: {error}", path.display()))?;

        assert_eq!(
            count_scenarios(&source),
            11,
            "EMC must check in the browser Gherkin as browser parity fixtures"
        );

        Ok(())
    }

    #[test]
    fn runner_meta_gherkin_is_checked_in_as_emc_rule_fixtures() -> Result<(), Box<dyn Error>> {
        let path = workspace_root()
            .join("tests/features")
            .join("event_model_cucumber_execution.feature");
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("{} is missing: {error}", path.display()))?;

        assert_eq!(
            count_scenarios(&source),
            6,
            "EMC must check in the runner/meta Gherkin as execution parity fixtures"
        );

        Ok(())
    }

    fn count_scenarios(source: &str) -> usize {
        source
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:")
            })
            .count()
    }

    fn validator_feature_sources() -> Result<Vec<(PathBuf, String)>, Box<dyn Error>> {
        let root = workspace_root().join("tests/features/event_model_validator");
        fs::read_dir(root)?
            .map(|entry| entry.map(|directory_entry| directory_entry.path()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "feature")
            })
            .map(|path| {
                fs::read_to_string(&path)
                    .map(|source| (path, source))
                    .map_err(Into::into)
            })
            .collect()
    }

    fn validator_fixture_path(file_name: &str) -> PathBuf {
        workspace_root()
            .join("tests/features/event_model_validator")
            .join(file_name)
    }

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }
}
