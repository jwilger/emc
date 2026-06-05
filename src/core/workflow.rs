use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest};
use crate::core::effect::{Effect, EffectPlan, ProjectPath, ReportLine};
use crate::core::emit::lean::emit_workflow_module as emit_lean_workflow_module;
use crate::core::emit::quint::emit_workflow_module as emit_quint_workflow_module;
use crate::core::formal_graph::FormalWorkflowGraph;
use crate::core::layout::{ModeledWorkflowLayout, ModeledWorkflowLayouts};
use crate::core::project::{ProjectName, project_root_effects};
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, WorkflowCommandErrorRecord,
    WorkflowCommandErrorRecords, WorkflowEntryLifecycleStateRecord,
    WorkflowEntryLifecycleStateRecords, WorkflowModuleData, WorkflowOutcomeRecord,
    WorkflowOutcomeRecords, WorkflowOwnedDefinitionRecord, WorkflowOwnedDefinitionRecords,
    WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionRecord, WorkflowTransitionRecords,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewWorkflow {
    name: ModelName,
    description: ModelDescription,
    slug: WorkflowSlug,
}

impl NewWorkflow {
    pub fn new(name: ModelName, description: ModelDescription, slug: WorkflowSlug) -> Self {
        Self {
            name,
            description,
            slug,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexedWorkflowGraph {
    slug: WorkflowSlug,
    graph: FormalWorkflowGraph,
}

impl IndexedWorkflowGraph {
    pub fn new(slug: WorkflowSlug, graph: FormalWorkflowGraph) -> Self {
        Self { slug, graph }
    }

    pub fn slug(&self) -> &WorkflowSlug {
        &self.slug
    }

    pub fn graph(&self) -> &FormalWorkflowGraph {
        &self.graph
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexedWorkflowGraphs {
    graphs: Vec<IndexedWorkflowGraph>,
}

impl IndexedWorkflowGraphs {
    pub(crate) fn new(graphs: Vec<IndexedWorkflowGraph>) -> Self {
        Self { graphs }
    }

    fn as_slice(&self) -> &[IndexedWorkflowGraph] {
        &self.graphs
    }
}

pub fn add_workflow(
    project_name: ProjectName,
    existing_workflows: ModeledWorkflowLayouts,
    workflow: NewWorkflow,
) -> Result<EffectPlan, WorkflowMutationError> {
    reject_workflow_slug_collision(existing_workflows.as_slice(), &workflow)?;
    reject_workflow_module_collision(existing_workflows.as_slice(), &workflow)?;
    Ok(workflow_effect_plan(
        project_name,
        existing_workflows.into_inner(),
        workflow,
    ))
}

pub fn update_workflow_description(
    existing_workflows: ModeledWorkflowLayouts,
    workflow_graph: FormalWorkflowGraph,
    slug: WorkflowSlug,
    description: ModelDescription,
) -> Result<EffectPlan, WorkflowMutationError> {
    let existing_workflow = existing_workflows
        .as_slice()
        .iter()
        .find(|existing| existing.slug() == &slug)
        .cloned()
        .ok_or_else(|| WorkflowMutationError::new(format!("unknown workflow {}", slug.as_ref())))?;
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != *existing_workflow.name() {
        return Err(WorkflowMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            existing_workflow.name().as_ref()
        )));
    }
    let workflow_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    let workflow_transitions = workflow_graph.transitions().as_slice().to_owned();
    let workflow_outcomes = workflow_graph.outcomes().as_slice().to_owned();
    let workflow_command_errors = workflow_graph.command_errors().as_slice().to_owned();
    let workflow_owned_definitions = workflow_graph.owned_definitions().as_slice().to_owned();
    let workflow_transition_evidences = workflow_graph.transition_evidences().as_slice().to_owned();
    let workflow_entry_lifecycle_states = workflow_graph
        .entry_lifecycle_states()
        .as_slice()
        .to_owned();

    Ok(update_workflow_effect_plan(
        existing_workflows.into_inner(),
        NewWorkflow::new(workflow_name, description, slug),
        WorkflowUpdateArtifacts {
            slice_details: workflow_slice_details,
            transitions: workflow_transitions,
            outcomes: workflow_outcomes,
            command_errors: workflow_command_errors,
            owned_definitions: workflow_owned_definitions,
            transition_evidences: workflow_transition_evidences,
            entry_lifecycle_required: workflow_graph.entry_lifecycle_required(),
            entry_lifecycle_states: workflow_entry_lifecycle_states,
        },
        None,
    ))
}

pub fn update_workflow_name(
    existing_workflows: ModeledWorkflowLayouts,
    workflow_graph: FormalWorkflowGraph,
    slug: WorkflowSlug,
    name: ModelName,
) -> Result<EffectPlan, WorkflowMutationError> {
    let existing_workflow = existing_workflows
        .as_slice()
        .iter()
        .find(|existing| existing.slug() == &slug)
        .cloned()
        .ok_or_else(|| WorkflowMutationError::new(format!("unknown workflow {}", slug.as_ref())))?;
    let updated_workflow = NewWorkflow::new(
        name.clone(),
        existing_workflow.description().clone(),
        slug.clone(),
    );
    reject_workflow_module_collision(existing_workflows.as_slice(), &updated_workflow)?;

    let workflow_name = workflow_graph.name().clone();
    if workflow_name != *existing_workflow.name() {
        return Err(WorkflowMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            existing_workflow.name().as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != *existing_workflow.description() {
        return Err(WorkflowMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            existing_workflow.description().as_ref()
        )));
    }

    let workflow_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    let workflow_transitions = workflow_graph.transitions().as_slice().to_owned();
    let workflow_outcomes = workflow_graph.outcomes().as_slice().to_owned();
    let workflow_command_errors = workflow_graph.command_errors().as_slice().to_owned();
    let workflow_owned_definitions = workflow_graph.owned_definitions().as_slice().to_owned();
    let workflow_transition_evidences = workflow_graph.transition_evidences().as_slice().to_owned();
    let workflow_entry_lifecycle_states = workflow_graph
        .entry_lifecycle_states()
        .as_slice()
        .to_owned();

    Ok(update_workflow_effect_plan(
        existing_workflows.into_inner(),
        updated_workflow,
        WorkflowUpdateArtifacts {
            slice_details: workflow_slice_details,
            transitions: workflow_transitions,
            outcomes: workflow_outcomes,
            command_errors: workflow_command_errors,
            owned_definitions: workflow_owned_definitions,
            transition_evidences: workflow_transition_evidences,
            entry_lifecycle_required: workflow_graph.entry_lifecycle_required(),
            entry_lifecycle_states: workflow_entry_lifecycle_states,
        },
        Some(module_name(existing_workflow.name().as_ref())),
    ))
}

pub fn remove_workflow(
    project_name: ProjectName,
    existing_workflows: ModeledWorkflowLayouts,
    workflow_graphs: IndexedWorkflowGraphs,
    slug: WorkflowSlug,
) -> Result<EffectPlan, WorkflowMutationError> {
    let removed_workflow = existing_workflows
        .as_slice()
        .iter()
        .find(|existing| existing.slug() == &slug)
        .cloned()
        .ok_or_else(|| WorkflowMutationError::new(format!("unknown workflow {}", slug.as_ref())))?;
    reject_incoming_workflow_references(workflow_graphs.as_slice(), &slug)?;
    let workflow_graph = workflow_graphs
        .as_slice()
        .iter()
        .find(|graph| graph.slug() == &slug)
        .ok_or_else(|| {
            WorkflowMutationError::new(format!("workflow {} graph is missing", slug.as_ref()))
        })?;
    let workflow_name = workflow_graph.graph().name().clone();
    if workflow_name != *removed_workflow.name() {
        return Err(WorkflowMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            removed_workflow.name().as_ref()
        )));
    }

    let removed_slice_details = workflow_graph.graph().slice_details().as_slice().to_owned();
    let workflow_module_name = module_name(removed_workflow.name().as_ref());
    let workflow_name = removed_workflow.name().as_ref().to_owned();
    let remaining_workflow_slugs = existing_workflows
        .as_slice()
        .iter()
        .filter(|existing| existing.slug() != &slug)
        .map(|existing| existing.slug().clone())
        .collect::<Vec<_>>();

    let remove_slice_effects = removed_slice_details
        .into_iter()
        .flat_map(remove_slice_artifact_effects);
    let effects = project_root_effects(project_name, &remaining_workflow_slugs)
        .into_iter()
        .chain([
            Effect::RemoveFile(project_path(format!(
                "model/lean/{workflow_module_name}.lean"
            ))),
            Effect::RemoveFile(project_path(format!(
                "model/quint/{workflow_module_name}.qnt"
            ))),
        ])
        .chain(remove_slice_effects)
        .chain([Effect::Report(report_line(format!(
            "removed workflow {workflow_name}"
        )))])
        .collect::<Vec<_>>();

    Ok(EffectPlan::new(effects))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowMutationError {
    message: String,
}

impl WorkflowMutationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for WorkflowMutationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowMutationError {}

fn workflow_effect_plan(
    project_name: ProjectName,
    existing_workflows: Vec<ModeledWorkflowLayout>,
    workflow: NewWorkflow,
) -> EffectPlan {
    let workflow_name = workflow.name.as_ref();
    let module_name = module_name(workflow.name.as_ref());
    let lean_module_name = lean_module_name(module_name.clone());
    let quint_module_name = quint_module_name(module_name.clone());
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow.name.clone(),
        workflow_slug: workflow.slug.clone(),
        workflow_description: workflow.description.clone(),
        workflow_slice_details: WorkflowSliceDetails::from_details([]),
        workflow_transitions: WorkflowTransitionRecords::from_records([]),
        workflow_outcomes: WorkflowOutcomeRecords::from_records([]),
        workflow_command_errors: WorkflowCommandErrorRecords::from_records([]),
        workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records([]),
        workflow_transition_evidences: Default::default(),
        workflow_requires_entry_lifecycle_coverage: false,
        workflow_entry_lifecycle_states: Default::default(),
    });
    let workflow_slugs = existing_workflows
        .iter()
        .map(|existing| existing.slug().clone())
        .chain([workflow.slug.clone()])
        .collect::<Vec<_>>();
    EffectPlan::new(
        project_root_effects(project_name, &workflow_slugs)
            .into_iter()
            .chain([
                Effect::WriteFile(
                    project_path(format!("model/lean/{module_name}.lean")),
                    emit_lean_workflow_module(
                        lean_module_name,
                        WorkflowModuleData::new(
                            workflow.name.clone(),
                            workflow.description.clone(),
                            workflow.slug.clone(),
                            digest.clone(),
                        ),
                    ),
                ),
                Effect::WriteFile(
                    project_path(format!("model/quint/{module_name}.qnt")),
                    emit_quint_workflow_module(
                        quint_module_name,
                        WorkflowModuleData::new(
                            workflow.name.clone(),
                            workflow.description.clone(),
                            workflow.slug.clone(),
                            digest,
                        ),
                    ),
                ),
                Effect::Report(report_line(format!("added workflow {workflow_name}"))),
            ])
            .collect(),
    )
}

fn reject_incoming_workflow_references(
    workflow_graphs: &[IndexedWorkflowGraph],
    removed_slug: &WorkflowSlug,
) -> Result<(), WorkflowMutationError> {
    workflow_graphs
        .iter()
        .filter(|graph| graph.slug() != removed_slug)
        .find_map(|graph| incoming_workflow_reference(graph, removed_slug))
        .transpose()
        .map(|reference| {
            reference.map_or(Ok(()), |referencing_slug| {
                Err(WorkflowMutationError::new(format!(
                    "workflow {} is referenced by workflow {}",
                    removed_slug.as_ref(),
                    referencing_slug.as_ref()
                )))
            })
        })?
}

fn incoming_workflow_reference(
    graph: &IndexedWorkflowGraph,
    removed_slug: &WorkflowSlug,
) -> Option<Result<WorkflowSlug, WorkflowMutationError>> {
    graph
        .graph()
        .transitions()
        .as_slice()
        .iter()
        .any(|transition| {
            transition.kind().as_ref().starts_with("workflow_exit:")
                && transition.target().as_ref() == removed_slug.as_ref()
        })
        .then(|| Ok(graph.slug().clone()))
}

fn remove_slice_artifact_effects(slice: WorkflowSliceDetail) -> [Effect; 2] {
    let module_name = module_name(slice.name().as_ref());
    [
        Effect::RemoveFile(project_path(format!(
            "model/lean/slices/{module_name}.lean"
        ))),
        Effect::RemoveFile(project_path(format!(
            "model/quint/slices/{module_name}.qnt"
        ))),
    ]
}

struct WorkflowUpdateArtifacts {
    slice_details: Vec<WorkflowSliceDetail>,
    transitions: Vec<WorkflowTransitionRecord>,
    outcomes: Vec<WorkflowOutcomeRecord>,
    command_errors: Vec<WorkflowCommandErrorRecord>,
    owned_definitions: Vec<WorkflowOwnedDefinitionRecord>,
    transition_evidences: Vec<WorkflowTransitionEvidenceRecord>,
    entry_lifecycle_required: bool,
    entry_lifecycle_states: Vec<WorkflowEntryLifecycleStateRecord>,
}

fn update_workflow_effect_plan(
    _existing_workflows: Vec<ModeledWorkflowLayout>,
    workflow: NewWorkflow,
    artifacts: WorkflowUpdateArtifacts,
    previous_module_name: Option<String>,
) -> EffectPlan {
    let workflow_name = workflow.name.as_ref();
    let module_name = module_name(workflow.name.as_ref());
    let lean_module_name = lean_module_name(module_name.clone());
    let quint_module_name = quint_module_name(module_name.clone());
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow.name.clone(),
        workflow_slug: workflow.slug.clone(),
        workflow_description: workflow.description.clone(),
        workflow_slice_details: WorkflowSliceDetails::from_details(artifacts.slice_details.clone()),
        workflow_transitions: WorkflowTransitionRecords::from_records(
            artifacts.transitions.clone(),
        ),
        workflow_outcomes: WorkflowOutcomeRecords::from_records(artifacts.outcomes.clone()),
        workflow_command_errors: WorkflowCommandErrorRecords::from_records(
            artifacts.command_errors.clone(),
        ),
        workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records(
            artifacts.owned_definitions.clone(),
        ),
        workflow_transition_evidences: WorkflowTransitionEvidenceRecords::from_records(
            artifacts.transition_evidences.clone(),
        ),
        workflow_requires_entry_lifecycle_coverage: artifacts.entry_lifecycle_required,
        workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords::from_records(
            artifacts.entry_lifecycle_states.clone(),
        ),
    });
    let cleanup_effects = previous_module_name
        .filter(|previous_module_name| previous_module_name != &module_name)
        .into_iter()
        .flat_map(|previous_module_name| {
            [
                Effect::RemoveFile(project_path(format!(
                    "model/lean/{previous_module_name}.lean"
                ))),
                Effect::RemoveFile(project_path(format!(
                    "model/quint/{previous_module_name}.qnt"
                ))),
            ]
        });

    EffectPlan::new(
        cleanup_effects
            .chain([
                Effect::WriteFile(
                    project_path(format!("model/lean/{module_name}.lean")),
                    emit_lean_workflow_module(
                        lean_module_name,
                        WorkflowModuleData::new(
                            workflow.name.clone(),
                            workflow.description.clone(),
                            workflow.slug.clone(),
                            digest.clone(),
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            artifacts.slice_details.clone(),
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            artifacts.transitions.clone(),
                        ))
                        .with_outcomes(WorkflowOutcomeRecords::from_records(
                            artifacts.outcomes.clone(),
                        ))
                        .with_command_errors(WorkflowCommandErrorRecords::from_records(
                            artifacts.command_errors.clone(),
                        ))
                        .with_owned_definitions(WorkflowOwnedDefinitionRecords::from_records(
                            artifacts.owned_definitions.clone(),
                        ))
                        .with_transition_evidences(WorkflowTransitionEvidenceRecords::from_records(
                            artifacts.transition_evidences.clone(),
                        ))
                        .with_entry_lifecycle_required(artifacts.entry_lifecycle_required)
                        .with_entry_lifecycle_states(
                            WorkflowEntryLifecycleStateRecords::from_records(
                                artifacts.entry_lifecycle_states.clone(),
                            ),
                        ),
                    ),
                ),
                Effect::WriteFile(
                    project_path(format!("model/quint/{module_name}.qnt")),
                    emit_quint_workflow_module(
                        quint_module_name,
                        WorkflowModuleData::new(
                            workflow.name.clone(),
                            workflow.description.clone(),
                            workflow.slug.clone(),
                            digest,
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            artifacts.slice_details,
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            artifacts.transitions,
                        ))
                        .with_outcomes(WorkflowOutcomeRecords::from_records(artifacts.outcomes))
                        .with_command_errors(WorkflowCommandErrorRecords::from_records(
                            artifacts.command_errors,
                        ))
                        .with_owned_definitions(WorkflowOwnedDefinitionRecords::from_records(
                            artifacts.owned_definitions,
                        ))
                        .with_transition_evidences(WorkflowTransitionEvidenceRecords::from_records(
                            artifacts.transition_evidences,
                        ))
                        .with_entry_lifecycle_required(artifacts.entry_lifecycle_required)
                        .with_entry_lifecycle_states(
                            WorkflowEntryLifecycleStateRecords::from_records(
                                artifacts.entry_lifecycle_states,
                            ),
                        ),
                    ),
                ),
                Effect::Report(report_line(format!("updated workflow {workflow_name}"))),
            ])
            .collect(),
    )
}

fn reject_workflow_module_collision(
    existing_workflows: &[ModeledWorkflowLayout],
    workflow: &NewWorkflow,
) -> Result<(), WorkflowMutationError> {
    let generated_module_name = module_name(workflow.name.as_ref());
    existing_workflows
        .iter()
        .filter(|existing| existing.slug() != &workflow.slug)
        .find(|existing| module_name(existing.name().as_ref()) == generated_module_name)
        .map_or(Ok(()), |_existing| {
            Err(WorkflowMutationError::new(format!(
                "workflow module {generated_module_name} already exists"
            )))
        })
}

fn reject_workflow_slug_collision(
    existing_workflows: &[ModeledWorkflowLayout],
    workflow: &NewWorkflow,
) -> Result<(), WorkflowMutationError> {
    existing_workflows
        .iter()
        .find(|existing| existing.slug() == &workflow.slug)
        .map_or(Ok(()), |_existing| {
            Err(WorkflowMutationError::new(format!(
                "workflow {} already exists",
                workflow.slug.as_ref()
            )))
        })
}

fn module_name(raw: &str) -> String {
    let mut capitalize_next = true;
    raw.chars()
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
        unreachable!("EMC generated project path must be valid: {error}");
    })
}

fn lean_module_name(value: impl Into<String>) -> LeanModuleName {
    LeanModuleName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Lean4 module name must be valid: {error}");
    })
}

fn quint_module_name(value: impl Into<String>) -> QuintModuleName {
    QuintModuleName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Quint module name must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated report line must be valid: {error}");
    })
}
