use nutype::nutype;

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ProjectName(String);

pub fn init_project(project_name: ProjectName) -> EffectPlan {
    let module_name = module_name(&project_name);
    let project_name_text = project_name.as_ref();

    EffectPlan::new(vec![
        Effect::WriteFileIfMissing(
            project_path("emc.toml"),
            file_contents(format!(
                "[project]\nname = \"{project_name_text}\"\nlean_module = \"{module_name}\"\nquint_module = \"{module_name}\"\n"
            )),
        ),
        Effect::EnsureDirectory(project_path("model/lean")),
        Effect::WriteFileIfMissing(
            project_path("model/lean/lean-toolchain"),
            file_contents("leanprover/lean4:4.29.1\n"),
        ),
        Effect::WriteFileIfMissing(
            project_path("model/lean/lakefile.lean"),
            file_contents("package EMCModel\n"),
        ),
        Effect::WriteFileIfMissing(
            project_path(format!("model/lean/{module_name}.lean")),
            file_contents(format!(
                "namespace {module_name}\n\n-- EMC generated Lean4 model root.\n\nend {module_name}\n"
            )),
        ),
        Effect::WriteFileIfMissing(
            project_path("model/lean/slices/.gitkeep"),
            file_contents("\n"),
        ),
        Effect::EnsureDirectory(project_path("model/quint")),
        Effect::WriteFileIfMissing(
            project_path("model/quint/quint.json"),
            file_contents(format!(
                "{{\n  \"main\": \"{module_name}.qnt\",\n  \"invariants\": [\n    \"workflowIdentityStable\",\n    \"workflowSliceDetailsComplete\",\n    \"workflowTransitionsStructured\",\n    \"workflowTransitionSourcesResolve\",\n    \"workflowTransitionTargetsResolve\",\n    \"workflowStepRelationshipsAreAllowed\",\n    \"workflowStepSlugsAreUnique\",\n    \"workflowHasExactlyOneEntryStep\",\n    \"workflowMainStepsHaveIncomingReachability\",\n    \"workflowNonSupportingStepsReachableFromEntry\",\n    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\",\n    \"workflowTransitionsHaveModeledKinds\",\n    \"workflowExitsNameTargetsAndRationale\",\n    \"workflowExternallyRelevantOutcomesHandled\",\n    \"workflowOutcomesSourceResolve\",\n    \"workflowCommandErrorsSourceResolve\",\n    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\",\n    \"workflowNonEventDefinitionsAreUniquelyOwned\",\n    \"workflowCommandTransitionsResolveControlsAndCommands\",\n    \"workflowEventTransitionsAreSharedByEndpointSlices\",\n    \"workflowNavigationTransitionsResolveControlsAndViews\",\n    \"workflowExternalTriggersDeclarePayloadContracts\",\n    \"workflowTransitionsHaveRequiredEvidence\",\n    \"workflowEntryLifecycleStatesCoverRequiredStates\"\n  ]\n}}\n"
            )),
        ),
        Effect::WriteFileIfMissing(
            project_path(format!("model/quint/{module_name}.qnt")),
            file_contents(format!("module {module_name} {{\n}}\n")),
        ),
        Effect::WriteFileIfMissing(
            project_path("model/quint/slices/.gitkeep"),
            file_contents("\n"),
        ),
        Effect::EnsureDirectory(project_path("reviews")),
        Effect::WriteFileIfMissing(project_path("reviews/.gitkeep"), file_contents("\n")),
        Effect::Report(report_line(format!(
            "EMC project {project_name} layout is present"
        ))),
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

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static file contents must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static report line must be valid: {error}");
    })
}
