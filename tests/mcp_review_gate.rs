#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, write};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_evaluates_workflow_review_gate() -> Result<(), Box<dyn Error>> {
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
            clean_review_record(&current_model_digest(temp_dir.path())?),
        )?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"review_gate\""))
            .stdout(predicate::str::contains("workflow review is clean"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_records_clean_review_for_current_workflow_digest() -> Result<(), Box<dyn Error>> {
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
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_record_review_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"record_clean_review\""))
            .stdout(predicate::str::contains(
                "recorded clean review for workflow open-ticket",
            ));

        let review_record =
            read_to_string(temp_dir.path().join("reviews/open-ticket.review.json"))?;
        assert!(
            review_record.contains(&format!("\"model_content_digest\": \"{current_digest}\"")),
            "review record must bind the clean review to the current workflow digest"
        );

        Command::cargo_bin("emc")?
            .args(["review", "gate", "--workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("workflow review is clean"));

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"review_gate\",\"arguments\":{\"workflow\":\"open-ticket\"}}}\n",
        )
    }

    fn mcp_record_review_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"record_clean_review\",\"arguments\":{\"workflow\":\"open-ticket\",\"reviewer\":\"event-model-reviewer\",\"reviewed_at\":\"2026-06-03T00:00:00.000Z\"}}}\n",
        )
    }

    fn clean_review_record(model_content_digest: &str) -> String {
        format!(
            "{{\n  \"workflow_slug\": \"open-ticket\",\n  \"model_content_digest\": \"{model_content_digest}\",\n  \"reviewer_id\": \"event-model-reviewer\",\n  \"status\": \"clean\",\n  \"category_results\": {{\n    \"lifecycle-entry\": \"clean\",\n    \"canonical-lanes\": \"clean\",\n    \"board-connections\": \"clean\",\n    \"fake-intermediates\": \"clean\",\n    \"slice-ownership\": \"clean\",\n    \"source-chains\": \"clean\",\n    \"workflow-reachability\": \"clean\",\n    \"transition-resolution\": \"clean\",\n    \"navigation-targets\": \"clean\",\n    \"branch-shape\": \"clean\",\n    \"outcomes-and-errors\": \"clean\",\n    \"scenario-coverage\": \"clean\",\n    \"timeline-rendering\": \"clean\"\n  }},\n  \"mandatory_findings\": [],\n  \"reviewed_at\": \"2026-06-01T00:00:00.000Z\"\n}}\n"
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
