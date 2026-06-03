#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, write};

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn generate_site_emits_browsable_site_files() -> Result<(), Box<dyn Error>> {
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
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("generated site at site"));

        assert!(temp_dir.path().join("site/index.html").is_file());
        assert!(
            temp_dir
                .path()
                .join("site/assets/index-CTzj-YfP.js")
                .is_file()
        );
        assert!(
            temp_dir
                .path()
                .join("site/assets/index-DCPB_L_9.css")
                .is_file()
        );
        assert!(temp_dir.path().join("site/data/index.json").is_file());
        assert!(
            temp_dir
                .path()
                .join("site/data/workflows/open-ticket.eventmodel.json")
                .is_file()
        );

        let index_html = read_to_string(temp_dir.path().join("site/index.html"))?;
        assert!(
            index_html.contains("<title>Repair Desk Event Model Browser</title>"),
            "generated site must use project branding in the document title"
        );
        assert!(
            index_html.contains("window.EMC_PROJECT_NAME = \"Repair Desk Event Model Browser\""),
            "generated site must provide project branding to the browser runtime"
        );
        assert!(
            !index_html.contains("<title>Event Model Browser</title>"),
            "generated site must not drop project branding"
        );
        assert!(
            index_html.contains("./assets/index-CTzj-YfP.js"),
            "generated site must load the bundled browser JavaScript"
        );
        assert!(
            index_html.contains("./assets/index-DCPB_L_9.css"),
            "generated site must load the bundled browser CSS"
        );
        assert!(
            !index_html.contains("Open data/index.json"),
            "generated site must not use the placeholder browser shell"
        );
        let browser_js = read_to_string(temp_dir.path().join("site/assets/index-CTzj-YfP.js"))?;
        assert!(
            browser_js.contains("window.EMC_PROJECT_NAME"),
            "browser runtime must read the generated project branding"
        );
        assert!(
            !browser_js.contains(&format!("{}{}", "Ed", "dy")),
            "generated browser assets must not mention unrelated product labels"
        );

        Ok(())
    }

    #[test]
    fn generate_site_removes_stale_browser_data_files() -> Result<(), Box<dyn Error>> {
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
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let stale_workflow = temp_dir
            .path()
            .join("site/data/workflows/stale-ticket.eventmodel.json");
        write(&stale_workflow, "{\"name\":\"Stale ticket\"}")?;
        assert!(stale_workflow.is_file());

        Command::cargo_bin("emc")?
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("generated site at site"));

        assert!(
            !stale_workflow.exists(),
            "regenerating the browser site must remove stale data files"
        );
        assert!(
            temp_dir
                .path()
                .join("site/data/workflows/open-ticket.eventmodel.json")
                .is_file(),
            "regenerating the browser site must keep current workflow data"
        );

        Ok(())
    }

    #[test]
    fn generate_site_rejects_absolute_output_paths_at_the_boundary() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let absolute_output = temp_dir.path().join("outside-site");

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "generate",
                "site",
                "--output",
                absolute_output
                    .to_str()
                    .ok_or("temporary path is not valid UTF-8")?,
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid project path"));

        Ok(())
    }
}
