use crate::core::effect::{Effect, EffectPlan, ProjectPath, ReportLine};
use crate::core::project::ProjectName;

pub fn check_project(project_name: ProjectName) -> EffectPlan {
    let module_name = module_name(&project_name);

    EffectPlan::new(vec![
        Effect::RequireFile(project_path("emc.toml")),
        Effect::RequireFile(project_path(format!("model/lean/{module_name}.lean"))),
        Effect::RequireFile(project_path(format!("model/quint/{module_name}.qnt"))),
        Effect::RequireFile(project_path("model/browser/data/index.json")),
        Effect::RequireFile(project_path("model/browser/data/workflows/.gitkeep")),
        Effect::RequireFile(project_path("model/browser/data/slices/.gitkeep")),
        Effect::RequireFile(project_path("reviews/.gitkeep")),
        Effect::Report(report_line("project layout is complete")),
    ])
}

fn module_name(project_name: &ProjectName) -> String {
    let mut capitalize_next = true;
    project_name
        .as_ref()
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                let next = if capitalize_next {
                    character.to_ascii_uppercase()
                } else {
                    character
                };
                capitalize_next = false;
                Some(next)
            } else {
                capitalize_next = true;
                None
            }
        })
        .collect()
}

fn project_path(value: impl Into<String>) -> ProjectPath {
    ProjectPath::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static project path must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static report line must be valid: {error}");
    })
}
