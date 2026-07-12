// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, write};
    use std::path::Path;
    use std::process::Command as ProcessCommand;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_rejects_mutation_without_project_root_before_writing() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(missing_project_root_request())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "add_workflow requires project_root",
            ));

        assert!(
            !temp_dir.path().join("model/lean/OpenTicket.lean").exists(),
            "a rejected mutation must not write a workflow artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_check_project_does_not_rewrite_drifted_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let root_artifact = temp_dir.path().join("model/lean/RepairDesk.lean");
        write(&root_artifact, "corrupt generated project root\n")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(check_project_request())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Lean project root drift for Repair Desk",
            ));

        assert_eq!(
            read_to_string(&root_artifact)?,
            "corrupt generated project root\n",
            "check_project must not rewrite the artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_sync_project_repairs_drift_after_root_attestation() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let root_artifact = temp_dir.path().join("model/lean/RepairDesk.lean");
        write(&root_artifact, "corrupt generated project root\n")?;
        let project_root = temp_dir.path().canonicalize()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(sync_project_request(&project_root))
            .assert()
            .success()
            .stdout(predicate::str::contains("project layout is complete"));

        assert!(
            read_to_string(&root_artifact)?.contains("namespace RepairDesk"),
            "sync_project must regenerate the artifact after a matching root attestation"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_rejects_a_mutation_attested_to_another_project_root() -> Result<(), Box<dyn Error>>
    {
        let server_project = TempDir::new()?;
        let other_project = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Server Project"])
            .current_dir(server_project.path())
            .assert()
            .success();

        let server_root = server_project.path().canonicalize()?;
        let supplied_root = other_project.path().canonicalize()?;
        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(server_project.path())
            .write_stdin(mismatched_project_root_request(&supplied_root))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "expected {}, supplied {}",
                server_root.display(),
                supplied_root.display()
            )));

        assert!(
            !server_project
                .path()
                .join("model/lean/OpenTicket.lean")
                .exists(),
            "a mismatched root attestation must not write a workflow artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_rejects_a_mutation_attested_to_a_linked_worktree() -> Result<(), Box<dyn Error>> {
        let repository = TempDir::new()?;
        let linked_worktree = TempDir::new()?;

        git(repository.path(), ["init", "--initial-branch", "main"])?;
        git(repository.path(), ["config", "user.name", "EMC test"])?;
        git(
            repository.path(),
            ["config", "user.email", "emc-test@example.test"],
        )?;
        write(repository.path().join("README.md"), "# test\n")?;
        git(repository.path(), ["add", "README.md"])?;
        git(repository.path(), ["commit", "-m", "initial"])?;
        git(
            repository.path(),
            [
                "worktree",
                "add",
                "--detach",
                linked_worktree
                    .path()
                    .to_str()
                    .ok_or("non-UTF-8 worktree path")?,
            ],
        )?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Server Project"])
            .current_dir(repository.path())
            .assert()
            .success();

        let server_root = repository.path().canonicalize()?;
        let linked_root = linked_worktree.path().canonicalize()?;
        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(repository.path())
            .write_stdin(mismatched_project_root_request(&linked_root))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "expected {}, supplied {}",
                server_root.display(),
                linked_root.display()
            )));

        assert!(
            !repository
                .path()
                .join("model/lean/OpenTicket.lean")
                .exists(),
            "a linked-worktree mismatch must not write in the server worktree"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_project_context_returns_the_canonical_server_root() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path().canonicalize()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(project_context_request())
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "\\\"project_root\\\": \\\"{}\\\"",
                project_root.display()
            )));

        Ok(())
    }

    fn missing_project_root_request() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow\",\"arguments\":{\"slug\":\"open-ticket\",\"name\":\"Open ticket\",\"description\":\"Actor opens a repair ticket.\"}}}\n",
        )
    }

    fn check_project_request() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"check_project\",\"arguments\":{}}}\n",
        )
    }

    fn sync_project_request(project_root: &Path) -> String {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"sync_project\",\"arguments\":{\"project_root\":\"__PROJECT_ROOT__\"}}}\n",
        )
        .replace("__PROJECT_ROOT__", &project_root.display().to_string())
    }

    fn mismatched_project_root_request(project_root: &Path) -> String {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow\",\"arguments\":{\"slug\":\"open-ticket\",\"name\":\"Open ticket\",\"description\":\"Actor opens a repair ticket.\",\"project_root\":\"__PROJECT_ROOT__\"}}}\n",
        )
        .replace("__PROJECT_ROOT__", &project_root.display().to_string())
    }

    fn project_context_request() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"project_context\",\"arguments\":{}}}\n",
        )
    }

    fn git<'command>(
        directory: &Path,
        arguments: impl IntoIterator<Item = &'command str>,
    ) -> Result<(), Box<dyn Error>> {
        let status = ProcessCommand::new("git")
            .args(arguments)
            .current_dir(directory)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err("git fixture setup failed".into())
        }
    }
}
