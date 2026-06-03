#[cfg(test)]
mod tests {
    use std::error::Error;

    use assert_cmd::Command;
    use predicates::prelude::predicate;

    #[test]
    fn cli_help_lists_user_facing_command_families() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Event Model Compiler"))
            .stdout(predicate::str::contains("init"))
            .stdout(predicate::str::contains("add workflow"))
            .stdout(predicate::str::contains("remove workflow"))
            .stdout(predicate::str::contains("add slice"))
            .stdout(predicate::str::contains("connect workflow"))
            .stdout(predicate::str::contains("validate"))
            .stdout(predicate::str::contains("verify"))
            .stdout(predicate::str::contains("check"))
            .stdout(predicate::str::contains("generate site"))
            .stdout(predicate::str::contains("review record"))
            .stdout(predicate::str::contains("mcp stdio"))
            .stdout(predicate::str::contains("mcp http"));

        Ok(())
    }

    #[test]
    fn cli_without_arguments_prints_help() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .assert()
            .success()
            .stdout(predicate::str::contains("Event Model Compiler"))
            .stdout(predicate::str::contains("emc init --name <project-name>"));

        Ok(())
    }
}
