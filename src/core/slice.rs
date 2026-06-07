// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
use crate::core::effect::{Effect, EffectPlan, ProjectPath, ReportLine};
use crate::core::emit::lean::{
    emit_slice_module as emit_lean_slice_module, emit_workflow_module as emit_lean_workflow_module,
};
use crate::core::emit::quint::{
    emit_slice_module as emit_quint_slice_module,
    emit_workflow_module as emit_quint_workflow_module,
};
use crate::core::events::EventDraft;
use crate::core::formal_graph::FormalWorkflowGraph;
use crate::core::project::{ProjectName, ProjectSliceMembership, project_root_effects};
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, QuintModuleName, SliceKindName, SliceSlug,
    WorkflowCommandErrorRecords, WorkflowEntryLifecycleStateRecords, WorkflowModuleData,
    WorkflowOutcomeRecords, WorkflowOwnedDefinitionRecords, WorkflowSliceDetail,
    WorkflowSliceDetails, WorkflowSlug, WorkflowStepRelationshipName,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionRecord, WorkflowTransitionRecords,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SliceKind {
    StateView,
    StateChange,
    Translation,
    Automation,
}

impl SliceKind {
    pub fn state_view() -> Self {
        Self::StateView
    }

    pub fn state_change() -> Self {
        Self::StateChange
    }

    pub fn translation() -> Self {
        Self::Translation
    }

    pub fn automation() -> Self {
        Self::Automation
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::StateView => "state_view",
            Self::StateChange => "state_change",
            Self::Translation => "translation",
            Self::Automation => "automation",
        }
    }
}

impl From<SliceKindName> for SliceKind {
    fn from(kind: SliceKindName) -> Self {
        match kind {
            SliceKindName::StateView => Self::StateView,
            SliceKindName::StateChange => Self::StateChange,
            SliceKindName::Translation => Self::Translation,
            SliceKindName::Automation => Self::Automation,
        }
    }
}

impl From<SliceKind> for SliceKindName {
    fn from(kind: SliceKind) -> Self {
        match kind {
            SliceKind::StateView => Self::StateView,
            SliceKind::StateChange => Self::StateChange,
            SliceKind::Translation => Self::Translation,
            SliceKind::Automation => Self::Automation,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewSlice {
    workflow_slug: WorkflowSlug,
    slug: SliceSlug,
    name: ModelName,
    description: ModelDescription,
    kind: SliceKind,
}

impl NewSlice {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slug: SliceSlug,
        name: ModelName,
        description: ModelDescription,
        kind: SliceKind,
    ) -> Self {
        Self {
            workflow_slug,
            slug,
            name,
            description,
            kind,
        }
    }

    pub fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub fn slug(&self) -> &SliceSlug {
        &self.slug
    }

    pub fn name(&self) -> &ModelName {
        &self.name
    }

    pub fn description(&self) -> &ModelDescription {
        &self.description
    }

    pub fn kind(&self) -> SliceKind {
        self.kind
    }
}

pub(crate) fn add_slice(
    project_name: ProjectName,
    formal_workflows: &[FormalWorkflowGraph],
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_graph: FormalWorkflowGraph,
    new_slice: NewSlice,
) -> Result<EffectPlan, SliceMutationError> {
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != indexed_workflow_name {
        return Err(SliceMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != indexed_workflow_description {
        return Err(SliceMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }
    let existing_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    existing_slice_details
        .iter()
        .find(|slice| slice.slug() == &new_slice.slug)
        .map_or(Ok(()), |_slice| {
            Err(SliceMutationError::new(format!(
                "slice {} already exists",
                new_slice.slug.as_ref()
            )))
        })?;
    let slice_module_name = module_name(new_slice.name.as_ref());
    existing_slice_details
        .iter()
        .find(|slice| module_name(slice.name().as_ref()) == slice_module_name)
        .map_or(Ok(()), |_slice| {
            Err(SliceMutationError::new(format!(
                "slice module {slice_module_name} already exists"
            )))
        })?;
    let workflow_module_name = module_name(workflow_name.as_ref());
    let mut workflow_slice_details = existing_slice_details;
    let new_slice_is_entry = workflow_slice_details.is_empty();
    workflow_slice_details.push(workflow_slice_detail(&new_slice, new_slice_is_entry));
    let workflow_transitions = workflow_graph.transitions().as_slice().to_owned();
    let workflow_outcomes = workflow_graph.outcomes().clone();
    let workflow_command_errors = workflow_graph.command_errors().clone();
    let workflow_owned_definitions = workflow_graph.owned_definitions().clone();
    let workflow_transition_evidences = workflow_graph.transition_evidences().clone();
    let workflow_entry_lifecycle_required = workflow_graph.entry_lifecycle_required();
    let workflow_entry_lifecycle_states = workflow_graph.entry_lifecycle_states().clone();
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow_name.clone(),
        workflow_slug: new_slice.workflow_slug.clone(),
        workflow_description: workflow_description.clone(),
        workflow_slice_details: WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
        workflow_transitions: WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
        workflow_outcomes: workflow_outcomes.clone(),
        workflow_command_errors: workflow_command_errors.clone(),
        workflow_owned_definitions: workflow_owned_definitions.clone(),
        workflow_transition_evidences: workflow_transition_evidences.clone(),
        workflow_requires_entry_lifecycle_coverage: workflow_entry_lifecycle_required,
        workflow_entry_lifecycle_states: workflow_entry_lifecycle_states.clone(),
    });
    let slice_name = new_slice.name.as_ref();
    let slice_kind = slice_kind_name(new_slice.kind);
    let slice_digest = slice_artifact_digest(
        new_slice.name.clone(),
        new_slice.slug.clone(),
        slice_kind,
        new_slice.description.clone(),
    );
    let workflow_slugs = formal_workflows
        .iter()
        .map(|workflow| workflow.slug().clone())
        .collect::<Vec<_>>();
    let slice_memberships = project_slice_memberships(
        formal_workflows,
        &new_slice.workflow_slug,
        &workflow_slice_details,
    );

    Ok(EffectPlan::new(
        project_root_effects(project_name, &workflow_slugs, &slice_memberships)
            .into_iter()
            .chain([
                Effect::write_file(
                    project_path(format!("model/lean/slices/{slice_module_name}.lean")),
                    emit_lean_slice_module(
                        lean_module_name(slice_module_name.clone()),
                        new_slice.name.clone(),
                        new_slice.description.clone(),
                        new_slice.slug.clone(),
                        slice_kind,
                        slice_digest.clone(),
                    ),
                ),
                Effect::write_file(
                    project_path(format!("model/quint/slices/{slice_module_name}.qnt")),
                    emit_quint_slice_module(
                        quint_module_name(slice_module_name),
                        new_slice.name.clone(),
                        new_slice.description.clone(),
                        new_slice.slug.clone(),
                        slice_kind,
                        slice_digest,
                    ),
                ),
                Effect::write_file(
                    project_path(format!("model/lean/{workflow_module_name}.lean")),
                    emit_lean_workflow_module(
                        lean_module_name(workflow_module_name.clone()),
                        WorkflowModuleData::new(
                            workflow_name.clone(),
                            workflow_description.clone(),
                            new_slice.workflow_slug.clone(),
                            digest.clone(),
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            workflow_slice_details.clone(),
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            workflow_transitions.clone(),
                        ))
                        .with_outcomes(workflow_outcomes.clone())
                        .with_command_errors(workflow_command_errors.clone())
                        .with_owned_definitions(workflow_owned_definitions.clone())
                        .with_transition_evidences(workflow_transition_evidences.clone())
                        .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                        .with_entry_lifecycle_states(workflow_entry_lifecycle_states.clone()),
                    ),
                ),
                Effect::write_file(
                    project_path(format!("model/quint/{workflow_module_name}.qnt")),
                    emit_quint_workflow_module(
                        quint_module_name(workflow_module_name),
                        WorkflowModuleData::new(
                            workflow_name,
                            workflow_description,
                            new_slice.workflow_slug.clone(),
                            digest,
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            workflow_slice_details,
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            workflow_transitions,
                        ))
                        .with_outcomes(workflow_outcomes)
                        .with_command_errors(workflow_command_errors)
                        .with_owned_definitions(workflow_owned_definitions)
                        .with_transition_evidences(workflow_transition_evidences)
                        .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                        .with_entry_lifecycle_states(workflow_entry_lifecycle_states),
                    ),
                ),
                Effect::ExportEvent(EventDraft::slice_added(&new_slice)),
                Effect::Report(report_line(format!("added slice {slice_name}"))),
            ])
            .collect(),
    ))
}

fn project_slice_memberships(
    formal_workflows: &[FormalWorkflowGraph],
    updated_workflow_slug: &WorkflowSlug,
    updated_slice_details: &[WorkflowSliceDetail],
) -> Vec<ProjectSliceMembership> {
    formal_workflows
        .iter()
        .flat_map(|workflow| {
            if workflow.slug() == updated_workflow_slug {
                updated_slice_details
                    .iter()
                    .map(|slice| {
                        ProjectSliceMembership::new(
                            workflow.slug().clone(),
                            slice.slug().clone(),
                            lean_module_name(module_name(slice.name().as_ref())),
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                workflow
                    .slice_details()
                    .as_slice()
                    .iter()
                    .map(|slice| {
                        ProjectSliceMembership::new(
                            workflow.slug().clone(),
                            slice.slug().clone(),
                            lean_module_name(module_name(slice.name().as_ref())),
                        )
                    })
                    .collect::<Vec<_>>()
            }
        })
        .collect()
}

pub(crate) fn update_slice_description(
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_graph: FormalWorkflowGraph,
    slice_slug: SliceSlug,
    description: ModelDescription,
) -> Result<EffectPlan, SliceMutationError> {
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != indexed_workflow_name {
        return Err(SliceMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != indexed_workflow_description {
        return Err(SliceMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }

    let workflow_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    let existing_slice = workflow_slice_details
        .iter()
        .into_iter()
        .find(|slice| slice.slug() == &slice_slug)
        .ok_or_else(|| {
            SliceMutationError::new(format!("slice {} is not in workflow", slice_slug.as_ref()))
        })?;
    let updated_slice = WorkflowSliceDetail::new_with_relationship(
        slice_slug.clone(),
        existing_slice.name().clone(),
        *existing_slice.kind(),
        description,
        *existing_slice.relationship(),
    );
    updated_slice_plan(
        UpdatedSliceWorkflow {
            workflow_name,
            workflow_description,
            workflow_slug,
            workflow_slice_details: replace_slice_detail(
                workflow_slice_details,
                updated_slice.clone(),
            ),
            workflow_transitions: workflow_graph.transitions().as_slice().to_owned(),
            workflow_outcomes: workflow_graph.outcomes().clone(),
            workflow_command_errors: workflow_graph.command_errors().clone(),
            workflow_owned_definitions: workflow_graph.owned_definitions().clone(),
            workflow_transition_evidences: workflow_graph.transition_evidences().clone(),
            workflow_entry_lifecycle_required: workflow_graph.entry_lifecycle_required(),
            workflow_entry_lifecycle_states: workflow_graph.entry_lifecycle_states().clone(),
        },
        slice_slug,
        updated_slice,
        None,
        Vec::new(),
    )
}

pub(crate) fn update_slice_kind(
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_graph: FormalWorkflowGraph,
    slice_slug: SliceSlug,
    kind: SliceKind,
) -> Result<EffectPlan, SliceMutationError> {
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != indexed_workflow_name {
        return Err(SliceMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != indexed_workflow_description {
        return Err(SliceMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }

    let workflow_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    let existing_slice = workflow_slice_details
        .iter()
        .into_iter()
        .find(|slice| slice.slug() == &slice_slug)
        .ok_or_else(|| {
            SliceMutationError::new(format!("slice {} is not in workflow", slice_slug.as_ref()))
        })?;
    let updated_slice = WorkflowSliceDetail::new_with_relationship(
        slice_slug.clone(),
        existing_slice.name().clone(),
        slice_kind_name(kind),
        existing_slice.description().clone(),
        *existing_slice.relationship(),
    );
    updated_slice_plan(
        UpdatedSliceWorkflow {
            workflow_name,
            workflow_description,
            workflow_slug,
            workflow_slice_details: replace_slice_detail(
                workflow_slice_details,
                updated_slice.clone(),
            ),
            workflow_transitions: workflow_graph.transitions().as_slice().to_owned(),
            workflow_outcomes: workflow_graph.outcomes().clone(),
            workflow_command_errors: workflow_graph.command_errors().clone(),
            workflow_owned_definitions: workflow_graph.owned_definitions().clone(),
            workflow_transition_evidences: workflow_graph.transition_evidences().clone(),
            workflow_entry_lifecycle_required: workflow_graph.entry_lifecycle_required(),
            workflow_entry_lifecycle_states: workflow_graph.entry_lifecycle_states().clone(),
        },
        slice_slug,
        updated_slice,
        None,
        Vec::new(),
    )
}

pub(crate) fn update_slice_name(
    project_root: SliceProjectRootContext<'_>,
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_graph: FormalWorkflowGraph,
    slice_slug: SliceSlug,
    name: ModelName,
) -> Result<EffectPlan, SliceMutationError> {
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != indexed_workflow_name {
        return Err(SliceMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != indexed_workflow_description {
        return Err(SliceMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }

    let existing_slices = workflow_graph.slice_details().as_slice().to_owned();
    let existing_slice = existing_slices
        .iter()
        .find(|slice| slice.slug() == &slice_slug)
        .cloned()
        .ok_or_else(|| {
            SliceMutationError::new(format!("slice {} is not in workflow", slice_slug.as_ref()))
        })?;
    reject_updated_slice_module_collision(&existing_slices, &slice_slug, &name)?;

    let updated_slice = WorkflowSliceDetail::new_with_relationship(
        slice_slug.clone(),
        name,
        *existing_slice.kind(),
        existing_slice.description().clone(),
        *existing_slice.relationship(),
    );
    let workflow_slice_details = replace_slice_detail(existing_slices, updated_slice.clone());
    let formal_workflows = project_root.formal_workflows();
    let workflow_slugs = formal_workflows
        .iter()
        .map(|workflow| workflow.slug().clone())
        .collect::<Vec<_>>();
    let slice_memberships =
        project_slice_memberships(formal_workflows, &workflow_slug, &workflow_slice_details);
    let root_effects = project_root_effects(
        project_root.project_name,
        &workflow_slugs,
        &slice_memberships,
    );
    updated_slice_plan(
        UpdatedSliceWorkflow {
            workflow_name,
            workflow_description,
            workflow_slug,
            workflow_slice_details,
            workflow_transitions: workflow_graph.transitions().as_slice().to_owned(),
            workflow_outcomes: workflow_graph.outcomes().clone(),
            workflow_command_errors: workflow_graph.command_errors().clone(),
            workflow_owned_definitions: workflow_graph.owned_definitions().clone(),
            workflow_transition_evidences: workflow_graph.transition_evidences().clone(),
            workflow_entry_lifecycle_required: workflow_graph.entry_lifecycle_required(),
            workflow_entry_lifecycle_states: workflow_graph.entry_lifecycle_states().clone(),
        },
        slice_slug,
        updated_slice,
        Some(module_name(existing_slice.name().as_ref())),
        root_effects.to_vec(),
    )
}

pub(crate) struct SliceProjectRootContext<'a> {
    project_name: ProjectName,
    formal_workflows: &'a [FormalWorkflowGraph],
}

impl<'a> SliceProjectRootContext<'a> {
    pub(crate) fn new(
        project_name: ProjectName,
        formal_workflows: &'a [FormalWorkflowGraph],
    ) -> Self {
        Self {
            project_name,
            formal_workflows,
        }
    }

    fn formal_workflows(&self) -> &[FormalWorkflowGraph] {
        self.formal_workflows
    }
}

pub(crate) fn remove_slice(
    project_name: ProjectName,
    formal_workflows: &[FormalWorkflowGraph],
    indexed_workflow_name: ModelName,
    indexed_workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_graph: FormalWorkflowGraph,
    slice_slug: SliceSlug,
) -> Result<EffectPlan, SliceMutationError> {
    let workflow_name = workflow_graph.name().clone();
    if workflow_name != indexed_workflow_name {
        return Err(SliceMutationError::new(format!(
            "workflow graph name '{}' does not match index name '{}'",
            workflow_name.as_ref(),
            indexed_workflow_name.as_ref()
        )));
    }
    let workflow_description = workflow_graph.description().clone();
    if workflow_description != indexed_workflow_description {
        return Err(SliceMutationError::new(format!(
            "workflow graph description '{}' does not match index description '{}'",
            workflow_description.as_ref(),
            indexed_workflow_description.as_ref()
        )));
    }

    let existing_slice_details = workflow_graph.slice_details().as_slice().to_owned();
    let removed_slice = existing_slice_details
        .iter()
        .find(|slice| slice.slug() == &slice_slug)
        .cloned()
        .ok_or_else(|| {
            SliceMutationError::new(format!("slice {} is not in workflow", slice_slug.as_ref()))
        })?;
    let existing_transitions = workflow_graph.transitions().as_slice().to_owned();
    reject_removing_slice_with_outgoing_transitions(&slice_slug, &existing_transitions)?;
    let workflow_module_name = module_name(workflow_name.as_ref());
    let removed_slice_module_name = module_name(removed_slice.name().as_ref());
    let workflow_slice_details = existing_slice_details
        .into_iter()
        .filter(|slice| slice.slug() != &slice_slug)
        .collect::<Vec<_>>();
    let workflow_transitions = existing_transitions
        .into_iter()
        .filter(|transition| transition.target().as_ref() != slice_slug.as_ref())
        .collect::<Vec<_>>();
    let workflow_outcomes = workflow_graph.outcomes().clone();
    let workflow_command_errors = workflow_graph.command_errors().clone();
    let workflow_owned_definitions = workflow_graph.owned_definitions().clone();
    let workflow_transition_evidences = workflow_graph.transition_evidences().clone();
    let workflow_entry_lifecycle_required = workflow_graph.entry_lifecycle_required();
    let workflow_entry_lifecycle_states = workflow_graph.entry_lifecycle_states().clone();
    let workflow_digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow_name.clone(),
        workflow_slug: workflow_slug.clone(),
        workflow_description: workflow_description.clone(),
        workflow_slice_details: WorkflowSliceDetails::from_details(workflow_slice_details.clone()),
        workflow_transitions: WorkflowTransitionRecords::from_records(workflow_transitions.clone()),
        workflow_outcomes: workflow_outcomes.clone(),
        workflow_command_errors: workflow_command_errors.clone(),
        workflow_owned_definitions: workflow_owned_definitions.clone(),
        workflow_transition_evidences: workflow_transition_evidences.clone(),
        workflow_requires_entry_lifecycle_coverage: workflow_entry_lifecycle_required,
        workflow_entry_lifecycle_states: workflow_entry_lifecycle_states.clone(),
    });
    let workflow_slugs = formal_workflows
        .iter()
        .map(|workflow| workflow.slug().clone())
        .collect::<Vec<_>>();
    let slice_memberships =
        project_slice_memberships(formal_workflows, &workflow_slug, &workflow_slice_details);
    Ok(EffectPlan::new(
        [
            project_root_effects(project_name, &workflow_slugs, &slice_memberships).to_vec(),
            vec![
                Effect::RemoveFile(project_path(format!(
                    "model/lean/slices/{removed_slice_module_name}.lean"
                ))),
                Effect::RemoveFile(project_path(format!(
                    "model/quint/slices/{removed_slice_module_name}.qnt"
                ))),
                Effect::write_file(
                    project_path(format!("model/lean/{workflow_module_name}.lean")),
                    emit_lean_workflow_module(
                        lean_module_name(workflow_module_name.clone()),
                        WorkflowModuleData::new(
                            workflow_name.clone(),
                            workflow_description.clone(),
                            workflow_slug.clone(),
                            workflow_digest.clone(),
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            workflow_slice_details.clone(),
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            workflow_transitions.clone(),
                        ))
                        .with_outcomes(workflow_outcomes.clone())
                        .with_command_errors(workflow_command_errors.clone())
                        .with_owned_definitions(workflow_owned_definitions.clone())
                        .with_transition_evidences(workflow_transition_evidences.clone())
                        .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                        .with_entry_lifecycle_states(workflow_entry_lifecycle_states.clone()),
                    ),
                ),
                Effect::write_file(
                    project_path(format!("model/quint/{workflow_module_name}.qnt")),
                    emit_quint_workflow_module(
                        quint_module_name(workflow_module_name),
                        WorkflowModuleData::new(
                            workflow_name,
                            workflow_description,
                            workflow_slug,
                            workflow_digest,
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            workflow_slice_details,
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            workflow_transitions,
                        ))
                        .with_outcomes(workflow_outcomes)
                        .with_command_errors(workflow_command_errors)
                        .with_owned_definitions(workflow_owned_definitions)
                        .with_transition_evidences(workflow_transition_evidences)
                        .with_entry_lifecycle_required(workflow_entry_lifecycle_required)
                        .with_entry_lifecycle_states(workflow_entry_lifecycle_states),
                    ),
                ),
                Effect::ExportEvent(EventDraft::slice_removed(&removed_slice)),
            ],
            vec![Effect::Report(report_line(format!(
                "removed slice {}",
                removed_slice.name().as_ref()
            )))],
        ]
        .concat(),
    ))
}

struct UpdatedSliceWorkflow {
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_slice_details: Vec<WorkflowSliceDetail>,
    workflow_transitions: Vec<WorkflowTransitionRecord>,
    workflow_outcomes: WorkflowOutcomeRecords,
    workflow_command_errors: WorkflowCommandErrorRecords,
    workflow_owned_definitions: WorkflowOwnedDefinitionRecords,
    workflow_transition_evidences: WorkflowTransitionEvidenceRecords,
    workflow_entry_lifecycle_required: bool,
    workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
}

fn updated_slice_plan(
    workflow: UpdatedSliceWorkflow,
    slice_slug: SliceSlug,
    updated_slice: WorkflowSliceDetail,
    previous_slice_module_name: Option<String>,
    root_effects: Vec<Effect>,
) -> Result<EffectPlan, SliceMutationError> {
    let workflow_module_name = module_name(workflow.workflow_name.as_ref());
    let slice_module_name = module_name(updated_slice.name().as_ref());
    let workflow_digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: workflow.workflow_name.clone(),
        workflow_slug: workflow.workflow_slug.clone(),
        workflow_description: workflow.workflow_description.clone(),
        workflow_slice_details: WorkflowSliceDetails::from_details(
            workflow.workflow_slice_details.clone(),
        ),
        workflow_transitions: WorkflowTransitionRecords::from_records(
            workflow.workflow_transitions.clone(),
        ),
        workflow_outcomes: workflow.workflow_outcomes.clone(),
        workflow_command_errors: workflow.workflow_command_errors.clone(),
        workflow_owned_definitions: workflow.workflow_owned_definitions.clone(),
        workflow_transition_evidences: workflow.workflow_transition_evidences.clone(),
        workflow_requires_entry_lifecycle_coverage: workflow.workflow_entry_lifecycle_required,
        workflow_entry_lifecycle_states: workflow.workflow_entry_lifecycle_states.clone(),
    });
    let slice_digest = slice_artifact_digest(
        updated_slice.name().clone(),
        slice_slug.clone(),
        *updated_slice.kind(),
        updated_slice.description().clone(),
    );
    let source_slice_module_name = previous_slice_module_name
        .clone()
        .unwrap_or_else(|| slice_module_name.clone());
    let cleanup_effects = previous_slice_module_name
        .filter(|previous_module_name| previous_module_name != &slice_module_name)
        .into_iter()
        .flat_map(|previous_module_name| {
            [
                Effect::RemoveFile(project_path(format!(
                    "model/lean/slices/{previous_module_name}.lean"
                ))),
                Effect::RemoveFile(project_path(format!(
                    "model/quint/slices/{previous_module_name}.qnt"
                ))),
            ]
        });

    Ok(EffectPlan::new(
        root_effects
            .into_iter()
            .chain([
                Effect::write_formal_slice_artifact_preserving_authored_facts(
                    project_path(format!("model/lean/slices/{source_slice_module_name}.lean")),
                    project_path(format!("model/lean/slices/{slice_module_name}.lean")),
                    emit_lean_slice_module(
                        lean_module_name(slice_module_name.clone()),
                        updated_slice.name().clone(),
                        updated_slice.description().clone(),
                        slice_slug.clone(),
                        *updated_slice.kind(),
                        slice_digest.clone(),
                    ),
                ),
                Effect::write_formal_slice_artifact_preserving_authored_facts(
                    project_path(format!("model/quint/slices/{source_slice_module_name}.qnt")),
                    project_path(format!("model/quint/slices/{slice_module_name}.qnt")),
                    emit_quint_slice_module(
                        quint_module_name(slice_module_name),
                        updated_slice.name().clone(),
                        updated_slice.description().clone(),
                        slice_slug.clone(),
                        *updated_slice.kind(),
                        slice_digest,
                    ),
                ),
                Effect::write_file(
                    project_path(format!("model/lean/{workflow_module_name}.lean")),
                    emit_lean_workflow_module(
                        lean_module_name(workflow_module_name.clone()),
                        WorkflowModuleData::new(
                            workflow.workflow_name.clone(),
                            workflow.workflow_description.clone(),
                            workflow.workflow_slug.clone(),
                            workflow_digest.clone(),
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            workflow.workflow_slice_details.clone(),
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            workflow.workflow_transitions.clone(),
                        ))
                        .with_outcomes(workflow.workflow_outcomes.clone())
                        .with_command_errors(workflow.workflow_command_errors.clone())
                        .with_owned_definitions(workflow.workflow_owned_definitions.clone())
                        .with_transition_evidences(workflow.workflow_transition_evidences.clone())
                        .with_entry_lifecycle_required(workflow.workflow_entry_lifecycle_required)
                        .with_entry_lifecycle_states(
                            workflow.workflow_entry_lifecycle_states.clone(),
                        ),
                    ),
                ),
                Effect::write_file(
                    project_path(format!("model/quint/{workflow_module_name}.qnt")),
                    emit_quint_workflow_module(
                        quint_module_name(workflow_module_name),
                        WorkflowModuleData::new(
                            workflow.workflow_name,
                            workflow.workflow_description,
                            workflow.workflow_slug,
                            workflow_digest,
                        )
                        .with_slice_details(WorkflowSliceDetails::from_details(
                            workflow.workflow_slice_details,
                        ))
                        .with_transitions(WorkflowTransitionRecords::from_records(
                            workflow.workflow_transitions,
                        ))
                        .with_outcomes(workflow.workflow_outcomes)
                        .with_command_errors(workflow.workflow_command_errors)
                        .with_owned_definitions(workflow.workflow_owned_definitions)
                        .with_transition_evidences(workflow.workflow_transition_evidences)
                        .with_entry_lifecycle_required(workflow.workflow_entry_lifecycle_required)
                        .with_entry_lifecycle_states(workflow.workflow_entry_lifecycle_states),
                    ),
                ),
                Effect::ExportEvent(EventDraft::slice_updated(&updated_slice)),
                Effect::Report(report_line(format!(
                    "updated slice {}",
                    updated_slice.name().as_ref()
                ))),
            ])
            .chain(cleanup_effects)
            .collect(),
    ))
}

fn reject_removing_slice_with_outgoing_transitions(
    slice_slug: &SliceSlug,
    workflow_transitions: &[WorkflowTransitionRecord],
) -> Result<(), SliceMutationError> {
    workflow_transitions
        .iter()
        .find(|transition| transition.source().as_ref() == slice_slug.as_ref())
        .map_or(Ok(()), |_transition| {
            Err(SliceMutationError::new(format!(
                "slice {} has outgoing workflow transitions; remove those transitions before removing the slice",
                slice_slug.as_ref()
            )))
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceMutationError {
    message: String,
}

impl SliceMutationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for SliceMutationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for SliceMutationError {}

fn workflow_slice_detail(new_slice: &NewSlice, entry: bool) -> WorkflowSliceDetail {
    WorkflowSliceDetail::new_with_relationship(
        new_slice.slug.clone(),
        new_slice.name.clone(),
        slice_kind_name(new_slice.kind),
        new_slice.description.clone(),
        if entry {
            WorkflowStepRelationshipName::Entry
        } else {
            WorkflowStepRelationshipName::Main
        },
    )
}

fn replace_slice_detail(
    existing_slices: Vec<WorkflowSliceDetail>,
    updated_slice: WorkflowSliceDetail,
) -> Vec<WorkflowSliceDetail> {
    existing_slices
        .into_iter()
        .map(|slice| {
            if slice.slug() == updated_slice.slug() {
                updated_slice.clone()
            } else {
                slice
            }
        })
        .collect()
}

fn reject_updated_slice_module_collision(
    existing_slices: &[WorkflowSliceDetail],
    slice_slug: &SliceSlug,
    name: &ModelName,
) -> Result<(), SliceMutationError> {
    let generated_module_name = module_name(name.as_ref());
    existing_slices
        .iter()
        .filter(|slice| slice.slug() != slice_slug)
        .find(|slice| module_name(slice.name().as_ref()) == generated_module_name)
        .map_or(Ok(()), |_slice| {
            Err(SliceMutationError::new(format!(
                "slice module {generated_module_name} already exists"
            )))
        })
}

fn slice_kind_name(kind: SliceKind) -> SliceKindName {
    SliceKindName::try_new(kind.as_str().to_owned())
        .unwrap_or_else(|error| unreachable!("EMC generated slice kind must be valid: {error}"))
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
