#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, write};
    use std::path::Path;

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
            index_html.contains(r#"<link rel="icon" href="data:," />"#),
            "generated site must suppress missing favicon requests"
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
        assert!(
            browser_js.contains("No commands, events, or read models modeled yet."),
            "definition tab must render an empty state"
        );
        assert!(
            browser_js.contains("No views modeled yet."),
            "views tab must render an empty state"
        );
        assert!(
            browser_js.contains("acceptance_scenarios"),
            "browser validation must read EMC acceptance scenarios"
        );
        assert!(
            browser_js.contains("contract_scenarios"),
            "browser validation must read EMC contract scenarios"
        );
        let browser_css = read_to_string(temp_dir.path().join("site/assets/index-DCPB_L_9.css"))?;
        assert!(
            browser_css.contains(".overflow-x-auto{overflow-x:auto;max-width:100%;width:100%"),
            "timeline scrollers must stay viewport-constrained while allowing horizontal scrolling"
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
    fn generate_site_rejects_invalid_modeled_workflows() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_navigation_chain(temp_dir.path())?;

        remove_first_workflow_transition_fixture(temp_dir.path(), "intake-visit")?;

        Command::cargo_bin("emc")?
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow transition drift for workflow Intake visit",
            ));

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
            .stderr(predicate::str::contains(
                "expected a relative path inside the project",
            ))
            .stderr(predicate::str::contains(
                "without parent-directory traversal",
            ));

        Ok(())
    }

    fn initialize_navigation_chain(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Clinic Intake"])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "intake-visit",
                "--name",
                "Intake visit",
                "--description",
                "Actor completes intake for a clinic visit.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        [
            (
                "capture-intake",
                "Capture intake",
                "Actor captures intake details.",
            ),
            ("triage-intake", "Triage intake", "Actor triages intake."),
            (
                "schedule-visit",
                "Schedule visit",
                "Actor schedules a visit.",
            ),
        ]
        .into_iter()
        .try_for_each(|(slug, name, description)| {
            Command::cargo_bin("emc")?
                .args([
                    "add",
                    "slice",
                    "--workflow",
                    "intake-visit",
                    "--slug",
                    slug,
                    "--name",
                    name,
                    "--type",
                    "state_view",
                    "--description",
                    description,
                ])
                .current_dir(cwd)
                .assert()
                .success();
            Ok::<(), Box<dyn Error>>(())
        })?;

        [
            ("capture-intake", "triage-intake", "triage-intake-screen"),
            ("triage-intake", "schedule-visit", "schedule-visit-screen"),
        ]
        .into_iter()
        .try_for_each(|(source, target, navigation)| {
            Command::cargo_bin("emc")?
                .args([
                    "connect",
                    "workflow",
                    "--workflow",
                    "intake-visit",
                    "--from",
                    source,
                    "--to",
                    target,
                    "--via",
                    "navigation",
                    "--name",
                    navigation,
                ])
                .current_dir(cwd)
                .assert()
                .success();
            Ok::<(), Box<dyn Error>>(())
        })?;

        Ok(())
    }

    fn remove_first_workflow_transition_fixture(
        cwd: &Path,
        workflow: &str,
    ) -> Result<(), Box<dyn Error>> {
        let path = cwd.join(format!(
            "model/browser/data/workflows/{workflow}.eventmodel.json"
        ));
        let mut workflow_json: serde_json::Value = serde_json::from_str(&read_to_string(&path)?)?;
        let steps = workflow_json
            .get_mut("steps")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("workflow fixture is missing steps")?;
        let transitions = steps
            .first_mut()
            .and_then(|step| step.get_mut("transitions"))
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("workflow fixture first step is missing transitions")?;
        transitions.clear();
        write(path, serde_json::to_string_pretty(&workflow_json)?)?;
        Ok(())
    }
}
