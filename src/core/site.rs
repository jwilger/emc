use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};

pub fn generate_site(output: ProjectPath) -> EffectPlan {
    let output_path = output.as_ref();
    EffectPlan::new(vec![
        Effect::EnsureDirectory(output.clone()),
        Effect::CopyDirectory(
            project_path("model/browser/data"),
            project_path(format!("{output_path}/data")),
        ),
        Effect::WriteFile(
            project_path(format!("{output_path}/index.html")),
            file_contents(INDEX_HTML),
        ),
        Effect::Report(report_line(format!("generated site at {output_path}"))),
    ])
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>EMC Event Model Browser</title>
  </head>
  <body>
    <main id="root">
      <h1>EMC Event Model Browser</h1>
      <p>Open data/index.json to inspect the generated event model data.</p>
    </main>
  </body>
</html>
"#;

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
