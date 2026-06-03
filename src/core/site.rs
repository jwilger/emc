use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::formal_graph::{FormalWorkflowGraph, FormalWorkflowGraphs};
use crate::core::formal_projection::{
    project_slice_browser_document, project_workflow_browser_document,
};
use crate::core::project::ProjectName;

pub fn generate_site(
    project_name: ProjectName,
    output: ProjectPath,
    formal_workflows: FormalWorkflowGraphs,
) -> EffectPlan {
    let output_path = output.as_ref();
    let data_path = project_path(format!("{output_path}/data"));
    EffectPlan::new(
        [
            vec![
                Effect::EnsureDirectory(output.clone()),
                Effect::RemoveDirectory(data_path.clone()),
                Effect::EnsureDirectory(data_path),
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
            ],
            browser_projection_effects(output_path, formal_workflows),
            vec![Effect::Report(report_line(format!(
                "generated site at {output_path}"
            )))],
        ]
        .into_iter()
        .flatten()
        .collect(),
    )
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
    <link rel="icon" href="data:," />
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

fn browser_projection_effects(
    output_path: &str,
    formal_workflows: FormalWorkflowGraphs,
) -> Vec<Effect> {
    let mut formal_workflows = formal_workflows.into_inner();
    formal_workflows.sort_by(|left, right| left.slug().as_ref().cmp(right.slug().as_ref()));
    let index = browser_index(&formal_workflows);
    let workflow_effects = formal_workflows.iter().map(|workflow| {
        Effect::WriteFile(
            project_path(format!(
                "{output_path}/data/workflows/{}.eventmodel.json",
                workflow.slug().as_ref()
            )),
            project_workflow_browser_document(workflow),
        )
    });
    let slice_effects = formal_workflows.iter().flat_map(|workflow| {
        workflow.slice_details().as_slice().iter().map(|slice| {
            Effect::WriteFile(
                project_path(format!(
                    "{output_path}/data/slices/{}.eventmodel.json",
                    slice.slug().as_ref()
                )),
                project_slice_browser_document(slice),
            )
        })
    });

    [Effect::WriteFile(
        project_path(format!("{output_path}/data/index.json")),
        file_contents(index),
    )]
    .into_iter()
    .chain(workflow_effects)
    .chain(slice_effects)
    .collect()
}

fn browser_index(workflows: &[FormalWorkflowGraph]) -> String {
    let entries = workflows
        .iter()
        .map(|workflow| {
            format!(
                "    {{\n      \"name\": {},\n      \"path\": \"data/workflows/{}.eventmodel.json\",\n      \"description\": {}\n    }}",
                json_string(workflow.name().as_ref()),
                workflow.slug().as_ref(),
                json_string(workflow.description().as_ref())
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    if entries.is_empty() {
        "{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": []\n}\n".to_owned()
    } else {
        format!(
            "{{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": [\n{entries}\n  ]\n}}\n"
        )
    }
}

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
