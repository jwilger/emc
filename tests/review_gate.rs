#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::write;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
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
            "{\n  \"workflow_slug\": \"open-ticket\",\n  \"model_content_digest\": \"abc123\",\n  \"reviewer_id\": \"event-model-reviewer\",\n  \"status\": \"clean\",\n  \"category_results\": {},\n  \"mandatory_findings\": [],\n  \"reviewed_at\": \"2026-06-01T00:00:00.000Z\"\n}\n",
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
}
