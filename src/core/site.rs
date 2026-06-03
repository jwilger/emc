use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::project::ProjectName;

pub fn generate_site(project_name: ProjectName, output: ProjectPath) -> EffectPlan {
    let output_path = output.as_ref();
    EffectPlan::new(vec![
        Effect::EnsureDirectory(output.clone()),
        Effect::CopyDirectory(
            project_path("model/browser/data"),
            project_path(format!("{output_path}/data")),
        ),
        Effect::WriteFile(
            project_path(format!("{output_path}/index.html")),
            file_contents(index_html(&project_name)),
        ),
        Effect::WriteFile(
            project_path(format!("{output_path}/assets/index-CTzj-YfP.js")),
            file_contents(BROWSER_JAVASCRIPT),
        ),
        Effect::WriteFile(
            project_path(format!("{output_path}/assets/index-DCPB_L_9.css")),
            file_contents(BROWSER_STYLESHEET),
        ),
        Effect::Report(report_line(format!("generated site at {output_path}"))),
    ])
}

fn index_html(project_name: &ProjectName) -> String {
    let title = project_name.as_ref();
    let runtime_project_name = json_string(format!("{title} Event Model Browser"));
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title} Event Model Browser</title>
    <script>window.EMC_PROJECT_NAME = {runtime_project_name};</script>
    <script type="module" crossorigin src="./assets/index-CTzj-YfP.js"></script>
    <link rel="stylesheet" crossorigin href="./assets/index-DCPB_L_9.css">
  </head>
  <body>
    <div id="root"></div>
  </body>
</html>
"#
    )
}

const BROWSER_JAVASCRIPT: &str = include_str!("../../browser/assets/index-CTzj-YfP.js");
const BROWSER_STYLESHEET: &str = include_str!("../../browser/assets/index-DCPB_L_9.css");

fn json_string(value: impl Into<String>) -> String {
    serde_json::to_string(&value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated browser runtime config must be valid JSON: {error}");
    })
}

fn project_path(value: impl Into<String>) -> ProjectPath {
    ProjectPath::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated project path must be valid: {error}");
    })
}

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated file contents must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated report line must be valid: {error}");
    })
}
