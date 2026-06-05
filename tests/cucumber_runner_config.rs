// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::env;
    use std::error::Error;
    use std::fs::{Permissions, create_dir_all, read_to_string, set_permissions, write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use assert_cmd::Command;
    use emc::core::effect::Effect;
    use emc::core::gherkin::run_all_gherkin_suites;
    use predicates::prelude::PredicateBooleanExt;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn gherkin_runner_lists_all_event_model_feature_suites() -> Result<(), Box<dyn Error>> {
        let suites = [
            (
                "meta",
                "tests/features/event_model_cucumber_execution.feature",
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
    fn gherkin_runner_run_executes_configured_review_gate_suite() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");
        let cargo_log = temp_dir.path().join("cargo.log");
        create_fake_cargo(&tool_dir, &cargo_log)?;

        Command::cargo_bin("emc")?
            .args(["gherkin", "run", "--suite", "review-gate"])
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "review-gate Gherkin suite passed; attempted 9 configured review-gate scenarios",
            ))
            .stderr(predicate::str::is_empty());

        assert_eq!(read_to_string(cargo_log)?, "test --test review_gate\n");

        Ok(())
    }

    #[test]
    fn gherkin_runner_run_all_executes_every_configured_suite() -> Result<(), Box<dyn Error>> {
        let plan = run_all_gherkin_suites();
        let commands = plan
            .effects()
            .iter()
            .map(run_process_command)
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(
            commands,
            vec![
                "cargo test --test review_gate",
                "cargo test --test cucumber_runner_config",
            ]
        );

        Ok(())
    }

    fn run_process_command(effect: &Effect) -> Result<String, Box<dyn Error>> {
        match effect {
            Effect::RunProcess(invocation) => Ok(format!(
                "{} {}",
                invocation.program().as_ref(),
                invocation
                    .arguments()
                    .iter()
                    .map(|argument| argument.as_ref())
                    .collect::<Vec<_>>()
                    .join(" ")
            )),
            _ => Err("expected a run-process effect".into()),
        }
    }

    fn create_fake_cargo(tool_dir: &Path, log_path: &Path) -> Result<(), Box<dyn Error>> {
        create_dir_all(tool_dir)?;
        let tool_path = tool_dir.join("cargo");
        write(
            &tool_path,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n",
                log_path.display()
            ),
        )?;
        set_permissions(&tool_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn path_with_fake_tools(tool_dir: &Path) -> Result<String, Box<dyn Error>> {
        let mut paths = vec![tool_dir.to_path_buf()];
        paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
        Ok(env::join_paths(paths)?.to_string_lossy().into_owned())
    }
}
