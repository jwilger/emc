use nutype::nutype;

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::types::{SliceSlug, WorkflowSlug};

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ProjectName(String);

const FORMAL_MODEL_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectSliceMembership {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
}

impl ProjectSliceMembership {
    pub fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug) -> Self {
        Self {
            workflow_slug,
            slice_slug,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectSliceMemberships {
    memberships: Vec<ProjectSliceMembership>,
}

impl ProjectSliceMemberships {
    pub fn from_memberships(memberships: impl IntoIterator<Item = ProjectSliceMembership>) -> Self {
        Self {
            memberships: memberships.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[ProjectSliceMembership] {
        &self.memberships
    }
}

pub fn init_project(project_name: ProjectName) -> EffectPlan {
    let module_name = module_name(&project_name);
    let project_name_text = project_name.as_ref();

    EffectPlan::new(vec![
        Effect::WriteFileIfMissing(
            project_path("emc.toml"),
            file_contents(format!(
                "[project]\nname = \"{project_name_text}\"\nversion = \"{FORMAL_MODEL_VERSION}\"\nlean_module = \"{module_name}\"\nquint_module = \"{module_name}\"\n"
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
            emit_lean_project_root(&project_name, &[], &[]),
        ),
        Effect::WriteFileIfMissing(
            project_path("model/lean/slices/.gitkeep"),
            file_contents("\n"),
        ),
        Effect::EnsureDirectory(project_path("model/quint")),
        Effect::WriteFileIfMissing(
            project_path("model/quint/quint.json"),
            file_contents(format!(
                "{{\n  \"main\": \"{module_name}.qnt\",\n  \"invariants\": [\n    \"workflowIdentityStable\",\n    \"workflowSliceDetailsComplete\",\n    \"workflowTransitionsStructured\",\n    \"workflowTransitionSourcesResolve\",\n    \"workflowTransitionTargetsResolve\",\n    \"workflowStepRelationshipsAreAllowed\",\n    \"workflowStepSlugsAreUnique\",\n    \"workflowHasExactlyOneEntryStep\",\n    \"workflowMainStepsHaveIncomingReachability\",\n    \"workflowNonSupportingStepsReachableFromEntry\",\n    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\",\n    \"workflowTransitionsHaveModeledKinds\",\n    \"workflowExitsNameTargetsAndRationale\",\n    \"workflowExternallyRelevantOutcomesHandled\",\n    \"workflowOutcomesSourceResolve\",\n    \"workflowCommandErrorsSourceResolve\",\n    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\",\n    \"workflowNonEventDefinitionsAreUniquelyOwned\",\n    \"workflowSharedEventDefinitionsHaveIdenticalIdentity\",\n    \"workflowCommandTransitionsResolveControlsAndCommands\",\n    \"workflowEventTransitionsAreSharedByEndpointSlices\",\n    \"workflowNavigationTransitionsResolveControlsAndViews\",\n    \"workflowExternalTriggersDeclarePayloadContracts\",\n    \"workflowTransitionsHaveRequiredEvidence\",\n    \"workflowEntryLifecycleStatesCoverRequiredStates\"\n  ]\n}}\n"
            )),
        ),
        Effect::WriteFileIfMissing(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_project_root(&project_name, &[], &[]),
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

pub fn project_root_effects(
    project_name: ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> [Effect; 2] {
    let module_name = module_name(&project_name);
    [
        Effect::WriteFile(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_project_root(&project_name, workflow_slugs, slice_memberships),
        ),
        Effect::WriteFile(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_project_root(&project_name, workflow_slugs, slice_memberships),
        ),
    ]
}

fn emit_lean_project_root(
    project_name: &ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> FileContents {
    let module_name = module_name(project_name);
    let workflow_list = lean_workflow_slug_list(workflow_slugs);
    let workflow_count = workflow_slugs.len();
    let slice_list = lean_slice_membership_list(slice_memberships);
    let slice_count = slice_memberships.len();
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC generated Lean4 model root.\n\ndef modelVersion := \"{FORMAL_MODEL_VERSION}\"\n\ndef modelWorkflows : List String := {workflow_list}\n\ndef modelSlices : List (String × String) := {slice_list}\n\ntheorem modelVersionIsStable : modelVersion = \"{FORMAL_MODEL_VERSION}\" := rfl\n\ntheorem modelWorkflowsAreDeclared : modelWorkflows.length = {workflow_count} := rfl\n\ntheorem modelSlicesAreDeclared : modelSlices.length = {slice_count} := rfl\n\nend {module_name}\n"
    ))
}

fn emit_quint_project_root(
    project_name: &ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> FileContents {
    let module_name = module_name(project_name);
    let workflow_list = quint_workflow_slug_list(workflow_slugs);
    let workflow_count = workflow_slugs.len();
    let slice_list = quint_slice_membership_list(slice_memberships);
    let slice_count = slice_memberships.len();
    file_contents(format!(
        "module {module_name} {{\n  type ModelSlice = {{ workflow: str, slice: str }}\n  val modelVersion = \"{FORMAL_MODEL_VERSION}\"\n  val modelWorkflows: List[str] = {workflow_list}\n  val modelSlices: List[ModelSlice] = {slice_list}\n  val modelVersionStable = modelVersion == \"{FORMAL_MODEL_VERSION}\"\n  val modelWorkflowsAreDeclared = modelWorkflows.length() == {workflow_count}\n  val modelSlicesAreDeclared = modelSlices.length() == {slice_count}\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}}\n"
    ))
}

fn lean_workflow_slug_list(workflow_slugs: &[WorkflowSlug]) -> String {
    let mut workflow_slugs = workflow_slugs
        .iter()
        .map(|slug| slug.as_ref())
        .collect::<Vec<_>>();
    workflow_slugs.sort_unstable();
    format!(
        "[{}]",
        workflow_slugs
            .into_iter()
            .map(|slug| format!("{slug:?}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_workflow_slug_list(workflow_slugs: &[WorkflowSlug]) -> String {
    let mut workflow_slugs = workflow_slugs
        .iter()
        .map(|slug| slug.as_ref())
        .collect::<Vec<_>>();
    workflow_slugs.sort_unstable();
    format!(
        "[{}]",
        workflow_slugs
            .into_iter()
            .map(|slug| format!("{slug:?}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_slice_membership_list(slice_memberships: &[ProjectSliceMembership]) -> String {
    let mut slice_memberships = slice_memberships
        .iter()
        .map(|membership| {
            (
                membership.workflow_slug.as_ref(),
                membership.slice_slug.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    slice_memberships.sort_unstable();
    format!(
        "[{}]",
        slice_memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug)| { format!("({workflow_slug:?}, {slice_slug:?})") })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_slice_membership_list(slice_memberships: &[ProjectSliceMembership]) -> String {
    let mut slice_memberships = slice_memberships
        .iter()
        .map(|membership| {
            (
                membership.workflow_slug.as_ref(),
                membership.slice_slug.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    slice_memberships.sort_unstable();
    format!(
        "[{}]",
        slice_memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug)| {
                format!("{{ workflow: {workflow_slug:?}, slice: {slice_slug:?} }}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
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
