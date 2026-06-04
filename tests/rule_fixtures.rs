#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

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
    fn runner_meta_gherkin_is_checked_in_as_emc_rule_fixtures() -> Result<(), Box<dyn Error>> {
        let path = workspace_root()
            .join("tests/features")
            .join("event_model_cucumber_execution.feature");
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("{} is missing: {error}", path.display()))?;

        assert_eq!(
            count_scenarios(&source),
            3,
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

    fn feature_sources() -> Result<Vec<(PathBuf, String)>, Box<dyn Error>> {
        let roots = [
            workspace_root().join("tests/features"),
            workspace_root().join("tests/features/event_model_review_gate"),
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

        if relative_path.starts_with("tests/features/event_model_review_gate/") {
            return Ok("review_gate");
        }

        Err(format!("no expected test target for {relative_path}"))
    }

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }
}
