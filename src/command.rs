use crate::core::connection::WorkflowConnection;
use crate::core::effect::{Effect, EffectPlan, ProjectPath};
use crate::core::gherkin::{
    GherkinSuite, list_gherkin_features, run_all_gherkin_suites, run_gherkin_suite,
};
use crate::core::project::{ProjectName, init_project};
use crate::core::review_gate::review_gate;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{ModelDescription, ModelName, SliceSlug, WorkflowSlug};
use crate::core::workflow::NewWorkflow;

pub fn add_slice(slice: NewSlice) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddSliceFromWorkflow(slice)])
}

pub fn add_workflow(workflow: NewWorkflow) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowFromIndex(workflow)])
}

pub fn check_project() -> EffectPlan {
    EffectPlan::new(vec![Effect::CheckCurrentProject])
}

pub fn connect_workflow(connection: WorkflowConnection) -> EffectPlan {
    EffectPlan::new(vec![Effect::ConnectWorkflowFromWorkflow(connection)])
}

pub fn generate_site(output: ProjectPath) -> EffectPlan {
    EffectPlan::new(vec![Effect::GenerateSiteFromManifest(output)])
}

pub fn gherkin_list(suite: GherkinSuite) -> EffectPlan {
    list_gherkin_features(suite)
}

pub fn gherkin_run(suite: GherkinSuite) -> EffectPlan {
    run_gherkin_suite(suite)
}

pub fn gherkin_run_all() -> EffectPlan {
    run_all_gherkin_suites()
}

pub fn init(name: ProjectName) -> EffectPlan {
    init_project(name)
}

pub fn list_workflows() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListWorkflowsFromIndex])
}

pub fn list_slices() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListSlicesFromIndex])
}

pub fn list_transitions() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListTransitionsFromIndex])
}

pub fn review_gate_for_workflow(slug: WorkflowSlug) -> EffectPlan {
    review_gate(slug)
}

pub fn show_workflow(slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::ShowWorkflowFromWorkflow(slug)])
}

pub fn show_slice(slug: SliceSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::ShowSliceFromSlice(slug)])
}

pub fn update_workflow_description(
    slug: WorkflowSlug,
    description: ModelDescription,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(
        slug,
        description,
    )])
}

pub fn update_workflow_name(slug: WorkflowSlug, name: ModelName) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateWorkflowNameFromIndexAndWorkflow(
        slug, name,
    )])
}

pub fn update_slice_description(slug: SliceSlug, description: ModelDescription) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceDescriptionFromWorkflow(
        slug,
        description,
    )])
}

pub fn update_slice_kind(slug: SliceSlug, kind: SliceKind) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceKindFromWorkflow(slug, kind)])
}

pub fn update_slice_name(slug: SliceSlug, name: ModelName) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceNameFromWorkflow(slug, name)])
}

pub fn validate(target: ProjectPath) -> EffectPlan {
    EffectPlan::new(vec![Effect::ValidateEventModelTarget(target)])
}

pub fn verify() -> EffectPlan {
    EffectPlan::new(vec![Effect::VerifyProjectFromIndex])
}
