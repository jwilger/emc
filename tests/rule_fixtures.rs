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

    #[test]
    fn every_gherkin_scenario_has_checked_in_executable_traceability() -> Result<(), Box<dyn Error>>
    {
        let traceability_path = workspace_root().join("docs/gherkin-traceability.md");
        let traceability = fs::read_to_string(&traceability_path)
            .map_err(|error| format!("{} is missing: {error}", traceability_path.display()))?;

        let missing = feature_sources()?
            .into_iter()
            .flat_map(|(path, source)| {
                let target = expected_test_target(&path)
                    .map(|target| target.to_owned())
                    .unwrap_or_else(|error| error);
                scenario_titles(&source).into_iter().map(move |title| {
                    let relative_path = path
                        .strip_prefix(workspace_root())
                        .unwrap_or(path.as_path())
                        .display()
                        .to_string();
                    format!("| `{relative_path}` | {title} | `cargo test --test {target}` |")
                })
            })
            .filter(|expected_prefix| !traceability.contains(expected_prefix))
            .collect::<Vec<_>>();

        assert_eq!(
            missing,
            Vec::<String>::new(),
            "every checked-in Gherkin scenario must be mapped to executable Rust coverage"
        );

        Ok(())
    }

    fn count_scenarios(source: &str) -> usize {
        scenario_titles(source).len()
    }

    fn scenario_titles(source: &str) -> Vec<String> {
        source
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                trimmed
                    .strip_prefix("Scenario:")
                    .or_else(|| trimmed.strip_prefix("Scenario Outline:"))
                    .map(str::trim)
                    .map(ToOwned::to_owned)
            })
            .collect()
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

    fn feature_sources() -> Result<Vec<(PathBuf, String)>, Box<dyn Error>> {
        let roots = [
            workspace_root().join("tests/features"),
            workspace_root().join("tests/features/event_model_browser"),
            workspace_root().join("tests/features/event_model_review_gate"),
            workspace_root().join("tests/features/event_model_validator"),
        ];

        roots
            .into_iter()
            .map(|root| {
                fs::read_dir(root)?
                    .map(|entry| entry.map(|directory_entry| directory_entry.path()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Into::into)
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?
            .into_iter()
            .flatten()
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

    fn expected_test_target(path: &Path) -> Result<&'static str, String> {
        let relative_path = path
            .strip_prefix(workspace_root())
            .unwrap_or(path)
            .display()
            .to_string();

        if relative_path == "tests/features/event_model_cucumber_execution.feature" {
            return Ok("cucumber_runner_config");
        }

        if relative_path.starts_with("tests/features/event_model_browser/") {
            return Ok("browser_composition");
        }

        if relative_path.starts_with("tests/features/event_model_review_gate/") {
            return Ok("review_gate");
        }

        if relative_path.starts_with("tests/features/event_model_validator/") {
            return Ok("validate_event_model");
        }

        Err(format!("no expected test target for {relative_path}"))
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
