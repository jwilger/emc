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
    fn generate_site_projects_browser_data_from_formal_artifacts() -> Result<(), Box<dyn Error>> {
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

        let browser_workflow = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        write(
            &browser_workflow,
            "{\n  \"name\": \"Stale ticket\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Stale browser projection.\",\n  \"steps\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("generated site at site"));

        let projected_workflow = read_to_string(
            temp_dir
                .path()
                .join("site/data/workflows/open-ticket.eventmodel.json"),
        )?;
        assert!(
            projected_workflow.contains("\"name\": \"Open ticket\""),
            "site data must use the formal workflow name, not stale browser JSON"
        );
        assert!(
            projected_workflow.contains("\"description\": \"Actor opens a repair ticket.\""),
            "site data must use the formal workflow description, not stale browser JSON"
        );
        assert!(
            !projected_workflow.contains("Stale"),
            "site data must not copy stale browser JSON"
        );

        Ok(())
    }

    #[test]
    fn generate_site_projects_workflow_exit_rationale_from_formal_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_workflow_exit(temp_dir.path())?;

        let browser_workflow = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let stale_workflow = read_to_string(&browser_workflow)?.replace(
            "Closed tickets continue to completion.",
            "Stale browser-only exit reason.",
        );
        write(&browser_workflow, stale_workflow)?;

        Command::cargo_bin("emc")?
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("generated site at site"));

        let projected_workflow = read_to_string(
            temp_dir
                .path()
                .join("site/data/workflows/open-ticket.eventmodel.json"),
        )?;
        assert!(
            projected_workflow
                .contains("\"exit_reason\": \"Closed tickets continue to completion.\""),
            "site data must project workflow-exit rationale from formal artifacts"
        );
        assert!(
            !projected_workflow.contains("Stale browser-only exit reason."),
            "site data must not copy stale browser-only workflow-exit rationale"
        );

        Ok(())
    }

    #[test]
    fn generate_site_projects_single_slice_workflow_shape_from_formal_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Clinic Intake"])
            .current_dir(temp_dir.path())
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
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "intake-visit",
                "--slug",
                "capture-intake",
                "--name",
                "Capture intake",
                "--type",
                "state_view",
                "--description",
                "Actor captures intake details.",
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

        let workflow = read_projected_workflow(temp_dir.path(), "intake-visit")?;
        assert_eq!(
            workflow
                .get("slice_files")
                .and_then(serde_json::Value::as_array)
                .ok_or("projected workflow must include slice files")?
                .as_slice(),
            [serde_json::Value::String(
                "../slices/capture-intake.eventmodel.json".to_owned()
            )],
            "workflow projection must retain formal slice references"
        );
        let steps = workflow
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .ok_or("projected workflow must include steps")?;
        assert_eq!(steps.len(), 1, "workflow projection must include its slice");
        assert_eq!(
            steps[0].get("slice").and_then(serde_json::Value::as_str),
            Some("capture-intake"),
            "workflow step must identify the formal slice"
        );
        assert!(
            steps[0].get("transitions").is_none(),
            "single-slice workflow with no formal transitions must omit transitions"
        );

        Ok(())
    }

    #[test]
    fn generate_site_projects_transition_workflow_shape_from_formal_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_navigation_chain(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("generated site at site"));

        let workflow = read_projected_workflow(temp_dir.path(), "intake-visit")?;
        let steps = workflow
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .ok_or("projected workflow must include steps")?;
        assert_eq!(
            steps.len(),
            3,
            "workflow projection must include all slices"
        );
        assert_transition(
            &steps[0],
            "capture-intake",
            "triage-intake",
            "triage-intake-screen",
        )?;
        assert_transition(
            &steps[1],
            "triage-intake",
            "schedule-visit",
            "schedule-visit-screen",
        )?;
        assert_eq!(
            steps[2].get("slice").and_then(serde_json::Value::as_str),
            Some("schedule-visit"),
            "final workflow step must identify the formal slice"
        );
        assert_eq!(
            steps[2]
                .get("transitions")
                .and_then(serde_json::Value::as_array)
                .ok_or("final step must include an empty transitions projection")?
                .len(),
            0,
            "when any formal transitions exist, steps with no outgoing edge must project an empty transitions array"
        );

        Ok(())
    }

    #[test]
    fn generate_site_rejects_unsynchronized_formal_workflows() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_navigation_chain(temp_dir.path())?;

        stale_first_lean_workflow_transition_fixture(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean and Quint workflow artifacts disagree for workflow intake-visit",
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

    fn read_projected_workflow(
        cwd: &Path,
        workflow: &str,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        serde_json::from_str(&read_to_string(
            cwd.join(format!("site/data/workflows/{workflow}.eventmodel.json")),
        )?)
        .map_err(Into::into)
    }

    fn assert_transition(
        step: &serde_json::Value,
        source: &str,
        target: &str,
        navigation: &str,
    ) -> Result<(), Box<dyn Error>> {
        assert_eq!(
            step.get("slice").and_then(serde_json::Value::as_str),
            Some(source),
            "workflow step must identify the formal transition source"
        );
        let transitions = step
            .get("transitions")
            .and_then(serde_json::Value::as_array)
            .ok_or("workflow step must include projected transitions")?;
        assert_eq!(
            transitions.as_slice(),
            [serde_json::json!({
                "to": target,
                "via_navigation": navigation,
            })],
            "workflow step must project the formal transition target and trigger"
        );
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

    fn initialize_workflow_exit(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(cwd)
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
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Actor enters repair ticket details.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn stale_first_lean_workflow_transition_fixture(cwd: &Path) -> Result<(), Box<dyn Error>> {
        let path = cwd.join("model/lean/IntakeVisit.lean");
        let artifact = read_to_string(&path)?;
        write(
            path,
            artifact.replace("triage-intake-screen", "stale-intake-screen"),
        )?;
        Ok(())
    }
}
