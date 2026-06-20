// Copyright 2026 John Wilger

use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io;
use std::net::TcpListener;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use fs4::{FileExt, TryLockError};

use crate::core::connection::{
    WorkflowConnection, WorkflowTransitionRemoval, connect_workflow, remove_transition,
};
use crate::core::effect::{
    ArtifactDigest, ArtifactDigestRequirement, CanonicalDeclarationRequirement, CleanReviewEffect,
    Effect, EffectPlan, EventConflictResolution, FileContents, FileWriteEffect, ModelContentDigest,
    ProcessInvocation, ProcessInvocations, ProjectPath, ProjectionFingerprint, ReportLine,
    ReviewEventReference, ReviewRecordRequirement, SliceCommandDefinitionRemovalEffect,
    SliceDescriptionUpdateEffect, SliceKindUpdateEffect, SliceNameUpdateEffect,
    SliceScenarioRemovalEffect, WorkflowCommandErrorEffect, WorkflowDescriptionUpdateEffect,
    WorkflowEntryLifecycleStateEffect, WorkflowNameUpdateEffect, WorkflowOutcomeEffect,
    WorkflowOwnedDefinitionEffect, WorkflowTransitionEvidenceEffect,
};
use crate::core::event_runtime::{
    ProjectRuntimeLock, ensure_event_store, execute_eventcore_command_for_exported_event,
    lock_project_runtime,
};
use crate::core::events::{
    EventDraft, ExportedEventType, exported_events_projection_fingerprint, list_event_conflicts,
    list_stale_workflow_readiness, project_exported_events, projected_formal_workflow_graphs,
    projected_project_root_inventories, projected_slice_command_definitions,
    reject_legacy_artifact_only_project, resolve_event_conflict, unresolved_event_conflicts_exist,
};
use crate::core::formal_graph::{FormalWorkflowGraph, FormalWorkflowGraphs};
use crate::core::formal_slice_facts::{
    NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
    NewCommandDefinition, NewEventDefinition, NewExternalPayloadDefinition, NewOutcomeDefinition,
    NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition, NewViewDefinition,
    validate_event_attribute_source,
};
use crate::core::layout::{
    ModeledWorkflowLayout, ModeledWorkflowLayouts, ModeledWorkflowSliceDetails,
    ModeledWorkflowTransitions, check_project, list_slices, list_transitions, list_workflows,
    show_document, show_workflow,
};
use crate::core::project::{ProjectName, ProjectSliceMembership, ProjectSliceMemberships};
use crate::core::review_record::{
    RequiredReviewCategories, ReviewCategoryFinding, ReviewRecordDocument, record_clean_review,
};
use crate::core::slice::{
    NewSlice, SliceProjectRootContext, add_slice, remove_slice, update_slice_description,
    update_slice_kind, update_slice_name,
};
use crate::core::types::{
    LeanModuleName, ReviewRuleName, ReviewTimestamp, ReviewerId, SliceSlug, WorkflowSliceDetail,
    WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionRecord,
};
use crate::core::verify::verify_project;
use crate::core::workflow::{
    IndexedWorkflowGraph, IndexedWorkflowGraphs, NewWorkflow, add_workflow, remove_workflow,
    update_workflow_description, update_workflow_name,
};
use crate::io::dto::parse_project_manifest_name;

const MAX_PARALLEL_VERIFICATION_PROCESSES: usize = 4;
const VERIFY_PARALLELISM_ENV: &str = "EMC_VERIFY_PARALLELISM";

#[derive(Debug)]
pub(crate) struct ShellError {
    message: String,
}

impl ShellError {
    pub(crate) fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(crate) fn project_name(error: impl Display) -> Self {
        Self {
            message: format!("invalid project name: {error}"),
        }
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "used as a `.map_err(ShellError::io)` function-pointer adaptor at ~20 call sites, which requires taking io::Error by value"
    )]
    fn io(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl Display for ShellError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ShellError {}

pub(crate) fn interpret(plan: &EffectPlan) -> Result<(), ShellError> {
    interpret_collect_reports_with_progress(plan, &mut |report| println!("{report}")).map(|_| ())
}

pub(crate) fn interpret_collect_reports(plan: &EffectPlan) -> Result<Vec<String>, ShellError> {
    let runtime_lock = lock_project_runtime_if_needed()?;
    if runtime_lock.is_some() {
        if !project_initialization_plan(plan) {
            reject_legacy_artifact_only_project().map_err(ShellError::message)?;
        }
        ensure_event_store_if_needed()?;
        project_exported_events_into_worktree()?;
        reject_mutation_when_event_conflicts_exist(plan)?;
    }
    plan.effects()
        .iter()
        .try_fold(Vec::new(), |mut reports, effect| {
            reports.extend(interpret_effect(effect)?);
            Ok(reports)
        })
}

fn interpret_collect_reports_with_progress(
    plan: &EffectPlan,
    report: &mut impl FnMut(&str),
) -> Result<Vec<String>, ShellError> {
    let runtime_lock = lock_project_runtime_if_needed()?;
    if runtime_lock.is_some() {
        if !project_initialization_plan(plan) {
            reject_legacy_artifact_only_project().map_err(ShellError::message)?;
        }
        ensure_event_store_if_needed()?;
        project_exported_events_into_worktree()?;
        reject_mutation_when_event_conflicts_exist(plan)?;
    }
    plan.effects()
        .iter()
        .try_fold(Vec::new(), |mut reports, effect| {
            reports.extend(interpret_effect_with_progress(effect, report)?);
            Ok(reports)
        })
}

fn interpret_effect_with_progress(
    effect: &Effect,
    report: &mut impl FnMut(&str),
) -> Result<Vec<String>, ShellError> {
    match effect {
        Effect::RunProcess(invocation) => {
            let progress = process_progress_report(invocation);
            report(&progress);
            let reports = run_process(invocation)?;
            for line in &reports {
                report(line);
            }
            Ok(reports)
        }
        Effect::RunProcessBatch(invocations) => {
            let reports = run_process_batch_with_progress(invocations, report)?;
            Ok(reports)
        }
        Effect::VerifyProjectFromIndex => interpret_verify_project_from_index_with_progress(report),
        _ => {
            let reports = interpret_effect(effect)?;
            for line in &reports {
                report(line);
            }
            Ok(reports)
        }
    }
}

fn interpret_verify_project_from_index_collecting() -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?;
    let readiness_workflows = formal_workflows
        .as_slice()
        .iter()
        .map(FormalWorkflowGraph::slug)
        .cloned()
        .collect::<Vec<_>>();
    let verified_frontier = current_projection_fingerprint()?;
    let modeled_slices = formal_workflow_slice_details(formal_workflows.clone());
    let modeled_workflows = formal_workflow_layouts(formal_workflows);
    let mut reports = interpret_collect_reports(&verify_project(
        &project_name,
        ModeledWorkflowLayouts::new(modeled_workflows),
        WorkflowSliceDetails::from_details(modeled_slices),
    ))?;
    let current_frontier = current_projection_fingerprint()?;
    if current_frontier != verified_frontier {
        return Err(ShellError::message(
            "event frontier changed during verification; retry `emc verify`",
        ));
    }
    for workflow in readiness_workflows {
        let model_content_digest = formal_model_content_digest(&workflow)?;
        reports.extend(interpret_effect(&Effect::declare_workflow_readiness(
            workflow,
            verified_frontier.clone(),
            model_content_digest,
            readiness_verified_at()?,
            readiness_verified_by()?,
            ReviewEventReference::unrecorded(),
        ))?);
    }
    Ok(reports)
}

fn interpret_verify_project_from_index_with_progress(
    report: &mut impl FnMut(&str),
) -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?;
    let readiness_workflows = formal_workflows
        .as_slice()
        .iter()
        .map(FormalWorkflowGraph::slug)
        .cloned()
        .collect::<Vec<_>>();
    let verified_frontier = current_projection_fingerprint()?;
    let modeled_slices = formal_workflow_slice_details(formal_workflows.clone());
    let modeled_workflows = formal_workflow_layouts(formal_workflows);
    let mut reports = interpret_collect_reports_with_progress(
        &verify_project(
            &project_name,
            ModeledWorkflowLayouts::new(modeled_workflows),
            WorkflowSliceDetails::from_details(modeled_slices),
        ),
        report,
    )?;
    let current_frontier = current_projection_fingerprint()?;
    if current_frontier != verified_frontier {
        return Err(ShellError::message(
            "event frontier changed during verification; retry `emc verify`",
        ));
    }
    for workflow in readiness_workflows {
        let model_content_digest = formal_model_content_digest(&workflow)?;
        let readiness_reports = interpret_effect(&Effect::declare_workflow_readiness(
            workflow,
            verified_frontier.clone(),
            model_content_digest,
            readiness_verified_at()?,
            readiness_verified_by()?,
            ReviewEventReference::unrecorded(),
        ))?;
        for line in &readiness_reports {
            report(line);
        }
        reports.extend(readiness_reports);
    }
    Ok(reports)
}

fn project_initialization_plan(plan: &EffectPlan) -> bool {
    plan.effects().iter().any(|effect| {
        matches!(
            effect,
            Effect::ExportEvent(draft)
                if draft.event_type() == ExportedEventType::ProjectInitialized
        )
    })
}

struct ShellProjectRuntimeLock {
    _lock: ProjectRuntimeLock,
}

impl Drop for ShellProjectRuntimeLock {
    fn drop(&mut self) {
        if let Ok(mut is_locked) = project_runtime_lock_state().lock() {
            *is_locked = false;
        }
    }
}

fn lock_project_runtime_if_needed() -> Result<Option<ShellProjectRuntimeLock>, ShellError> {
    let mut is_locked = project_runtime_lock_state()
        .lock()
        .map_err(|error| ShellError::message(error.to_string()))?;
    if *is_locked {
        return Ok(None);
    }

    let runtime_lock = lock_project_runtime(Path::new(".")).map_err(ShellError::message)?;
    *is_locked = true;
    Ok(Some(ShellProjectRuntimeLock {
        _lock: runtime_lock,
    }))
}

fn project_runtime_lock_state() -> &'static Mutex<bool> {
    static LOCKING_PROJECT_RUNTIME: OnceLock<Mutex<bool>> = OnceLock::new();
    LOCKING_PROJECT_RUNTIME.get_or_init(|| Mutex::new(false))
}

fn ensure_event_store_if_needed() -> Result<(), ShellError> {
    ensure_event_store(Path::new(".")).map_err(ShellError::message)
}

fn reject_mutation_when_event_conflicts_exist(plan: &EffectPlan) -> Result<(), ShellError> {
    if plan.effects().iter().any(effect_is_mutation)
        && unresolved_event_conflicts_exist().map_err(ShellError::message)?
    {
        return Err(ShellError::message(
            "unresolved event conflicts; run `emc list conflicts` and `emc resolve conflict`",
        ));
    }
    Ok(())
}

fn effect_is_mutation(effect: &Effect) -> bool {
    matches!(
        effect,
        Effect::AddAutomationDefinitionFromSlice(_)
            | Effect::AddBitLevelDataFlowFromSlice(_)
            | Effect::AddBoardConnectionFromSlice(_)
            | Effect::AddBoardElementFromSlice(_)
            | Effect::AddCommandDefinitionFromSlice(_)
            | Effect::AddEventDefinitionFromSlice(_)
            | Effect::AddExternalPayloadDefinitionFromSlice(_)
            | Effect::AddOutcomeDefinitionFromSlice(_)
            | Effect::AddReadModelDefinitionFromSlice(_)
            | Effect::AddSliceFromWorkflow(_)
            | Effect::AddSliceScenarioFromSlice(_)
            | Effect::AddTranslationDefinitionFromSlice(_)
            | Effect::AddViewDefinitionFromSlice(_)
            | Effect::AddWorkflowCommandErrorFromWorkflow(_)
            | Effect::AddWorkflowEntryLifecycleStateFromWorkflow(_)
            | Effect::AddWorkflowFromIndex(_)
            | Effect::AddWorkflowOutcomeFromWorkflow(_)
            | Effect::AddWorkflowOwnedDefinitionFromWorkflow(_)
            | Effect::AddWorkflowTransitionEvidenceFromWorkflow(_)
            | Effect::ConnectWorkflowFromWorkflow(_)
            | Effect::DeclareWorkflowReadinessFromWorkflow(_)
            | Effect::RecordCleanReviewFromWorkflow(_)
            | Effect::RemoveCommandDefinitionFromSlice(_)
            | Effect::RemoveSliceScenarioFromSlice(_)
            | Effect::RemoveSliceFromWorkflow(_)
            | Effect::RemoveTransitionFromWorkflow(_)
            | Effect::RemoveWorkflowFromIndex(_)
            | Effect::RequireWorkflowEntryLifecycleCoverageFromWorkflow(_)
            | Effect::UpdateCommandDefinitionFromSlice(_)
            | Effect::UpdateSliceScenarioFromSlice(_)
            | Effect::UpdateSliceDescriptionFromWorkflow(_)
            | Effect::UpdateSliceKindFromWorkflow(_)
            | Effect::UpdateSliceNameFromWorkflow(_)
            | Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(_)
            | Effect::UpdateWorkflowNameFromIndexAndWorkflow(_)
    )
}

fn project_exported_events_into_worktree() -> Result<(), ShellError> {
    // Regenerate the Lean/Quint artifacts from the authoritative event log on
    // every command. The artifacts are write-only projections of the log: this
    // self-heals any on-disk drift (a hand edit, a bad merge, a partial write)
    // by restoring them to exactly what the log projects. `write_file` is
    // idempotent, so when the artifacts already match the log nothing is
    // rewritten.
    let projecting = projecting_events_state();
    {
        let mut is_projecting = projecting
            .lock()
            .map_err(|error| ShellError::message(error.to_string()))?;
        if *is_projecting {
            return Ok(());
        }
        *is_projecting = true;
    }

    let result = project_exported_events()
        .map_err(ShellError::message)
        .and_then(|projection| {
            projection.map_or(Ok(()), |plan| {
                plan.effects()
                    .iter()
                    .try_for_each(|effect| interpret_effect(effect).map(|_reports| ()))
            })
        });

    let mut is_projecting = projecting
        .lock()
        .map_err(|error| ShellError::message(error.to_string()))?;
    *is_projecting = false;
    result
}

fn projecting_events_state() -> &'static Mutex<bool> {
    static PROJECTING_EVENTS: OnceLock<Mutex<bool>> = OnceLock::new();
    PROJECTING_EVENTS.get_or_init(|| Mutex::new(false))
}

fn exported_events_are_projecting() -> Result<bool, ShellError> {
    projecting_events_state()
        .lock()
        .map(|is_projecting| *is_projecting)
        .map_err(|error| ShellError::message(error.to_string()))
}

/// Append a fact event to the authoritative log, then regenerate the Lean/Quint
/// artifacts as a pure projection of the log. Authoring commands validate against
/// the log and call this — they never read, parse, or splice the generated
/// artifacts; the artifacts are write-only projections.
fn export_and_regenerate(draft: EventDraft) -> Result<(), ShellError> {
    interpret_effect(&Effect::ExportEvent(draft))?;
    project_exported_events_into_worktree()
}

fn projected_report(value: impl Into<String>) -> Result<String, ShellError> {
    ReportLine::try_new(value)
        .map(|line| line.as_ref().to_owned())
        .map_err(|error| ShellError::message(error.to_string()))
}

fn interpret_effect(effect: &Effect) -> Result<Vec<String>, ShellError> {
    if let Some(result) = interpret_slice_fact_effect(effect) {
        return result;
    }
    if let Some(result) = interpret_workflow_fact_effect(effect) {
        return result;
    }
    if let Some(result) = interpret_workflow_structure_effect(effect) {
        return result;
    }
    if let Some(result) = interpret_listing_effect(effect) {
        return result;
    }
    if let Some(result) = interpret_requirement_effect(effect) {
        return result;
    }
    interpret_io_effect(effect)
}

/// Shared tail for the uniform "add definition to a slice" effects: confirm the
/// slice is referenced by an indexed workflow, then export the fact and
/// regenerate the projection. Returns the precomputed slice/project-root
/// reports unchanged so ordering is preserved.
fn interpret_slice_addition(
    slice_slug: &SliceSlug,
    reports: Vec<String>,
    draft: EventDraft,
) -> Result<Vec<String>, ShellError> {
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    find_formal_workflow_containing_slice_in(&formal_workflows, slice_slug)?;
    export_and_regenerate(draft)?;
    Ok(reports)
}

/// Per-slice fact additions. Each arm builds its slice and project-root report
/// lines, then delegates the validate/export/regenerate tail to
/// `interpret_slice_addition`. Returns `None` for non-slice effects.
fn interpret_slice_fact_effect(effect: &Effect) -> Option<Result<Vec<String>, ShellError>> {
    Some(match effect {
        Effect::AddAutomationDefinitionFromSlice(automation) => {
            interpret_automation_added(automation)
        }
        Effect::AddBitLevelDataFlowFromSlice(data_flow) => {
            interpret_bit_level_data_flow_added(data_flow)
        }
        Effect::AddBoardConnectionFromSlice(connection) => {
            interpret_board_connection_added(connection)
        }
        Effect::AddBoardElementFromSlice(element) => interpret_board_element_added(element),
        Effect::AddCommandDefinitionFromSlice(command) => {
            interpret_command_definition_added(command)
        }
        Effect::RemoveCommandDefinitionFromSlice(removal) => {
            interpret_command_definition_removed(removal)
        }
        Effect::UpdateCommandDefinitionFromSlice(command) => {
            interpret_command_definition_updated(command)
        }
        Effect::AddEventDefinitionFromSlice(event) => interpret_event_definition_added(event),
        Effect::AddExternalPayloadDefinitionFromSlice(external_payload) => {
            interpret_external_payload_added(external_payload)
        }
        Effect::AddOutcomeDefinitionFromSlice(outcome) => interpret_outcome_added(outcome),
        Effect::AddReadModelDefinitionFromSlice(read_model) => {
            interpret_read_model_added(read_model)
        }
        Effect::AddViewDefinitionFromSlice(view) => interpret_view_added(view),
        Effect::AddSliceScenarioFromSlice(scenario) => interpret_slice_scenario_added(scenario),
        Effect::RemoveSliceScenarioFromSlice(removal) => interpret_slice_scenario_removed(removal),
        Effect::UpdateSliceScenarioFromSlice(scenario) => {
            interpret_slice_scenario_updated(scenario)
        }
        Effect::AddTranslationDefinitionFromSlice(translation) => {
            interpret_translation_added(translation)
        }
        _ => return None,
    })
}

fn interpret_automation_added(
    automation: &NewAutomationDefinition,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added automation {} to slice {}",
            automation.name().as_ref(),
            automation.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added automation {} to project root",
            automation.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        automation.slice_slug(),
        reports,
        EventDraft::slice_automation_added(automation),
    )
}

fn interpret_bit_level_data_flow_added(
    data_flow: &NewBitLevelDataFlow,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added bit-level data flow {} to slice {}",
            data_flow.datum().as_ref(),
            data_flow.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added bit-level data flow {} to project root",
            data_flow.datum().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        data_flow.slice_slug(),
        reports,
        EventDraft::slice_bit_level_data_flow_added(data_flow),
    )
}

fn interpret_board_connection_added(
    connection: &NewBoardConnection,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added board connection {} -> {} to slice {}",
            connection.source().as_ref(),
            connection.target().as_ref(),
            connection.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added board connection {} -> {} to project root",
            connection.source().as_ref(),
            connection.target().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        connection.slice_slug(),
        reports,
        EventDraft::slice_board_connection_added(connection),
    )
}

fn interpret_board_element_added(element: &NewBoardElement) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added board element {} to slice {}",
            element.name().as_ref(),
            element.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added board element {} to project root",
            element.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        element.slice_slug(),
        reports,
        EventDraft::slice_board_element_added(element),
    )
}

fn interpret_command_definition_added(
    command: &NewCommandDefinition,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added command {} to slice {}",
            command.name().as_ref(),
            command.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added command {} to project root",
            command.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        command.slice_slug(),
        reports,
        EventDraft::slice_command_definition_added(command),
    )
}

fn interpret_command_definition_updated(
    command: &NewCommandDefinition,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "updated command {} on slice {}",
            command.name().as_ref(),
            command.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "updated command {} in project root",
            command.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        command.slice_slug(),
        reports,
        EventDraft::slice_command_definition_updated(command),
    )
}

fn interpret_command_definition_removed(
    removal: &SliceCommandDefinitionRemovalEffect,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "removed command {} from slice {}",
            removal.command_name().as_ref(),
            removal.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "removed command {} from project root",
            removal.command_name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        removal.slice_slug(),
        reports,
        EventDraft::slice_command_definition_removed(removal.slice_slug(), removal.command_name()),
    )
}

fn interpret_event_definition_added(event: &NewEventDefinition) -> Result<Vec<String>, ShellError> {
    let slice_commands =
        projected_slice_command_definitions(event.slice_slug()).map_err(ShellError::message)?;
    validate_event_attribute_source(event, &slice_commands)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let reports = vec![
        projected_report(format!(
            "added event {} to slice {}",
            event.name().as_ref(),
            event.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added stream {} to project root",
            event.stream().as_ref()
        ))?,
        projected_report(format!(
            "added event {} to project root",
            event.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        event.slice_slug(),
        reports,
        EventDraft::slice_event_definition_added(event),
    )
}

fn interpret_external_payload_added(
    external_payload: &NewExternalPayloadDefinition,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added external payload {} to slice {}",
            external_payload.name().as_ref(),
            external_payload.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added external payload {} to project root",
            external_payload.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        external_payload.slice_slug(),
        reports,
        EventDraft::slice_external_payload_added(external_payload),
    )
}

fn interpret_outcome_added(outcome: &NewOutcomeDefinition) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added outcome {} to slice {}",
            outcome.label().as_ref(),
            outcome.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added outcome {} to project root",
            outcome.label().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        outcome.slice_slug(),
        reports,
        EventDraft::slice_outcome_added(outcome),
    )
}

fn interpret_read_model_added(
    read_model: &NewReadModelDefinition,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added read model {} to slice {}",
            read_model.name().as_ref(),
            read_model.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added read model {} to project root",
            read_model.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        read_model.slice_slug(),
        reports,
        EventDraft::slice_read_model_added(read_model),
    )
}

fn interpret_view_added(view: &NewViewDefinition) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added view {} to slice {}",
            view.name().as_ref(),
            view.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added view {} to project root",
            view.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        view.slice_slug(),
        reports,
        EventDraft::slice_view_added(view),
    )
}

fn interpret_slice_scenario_added(scenario: &NewSliceScenario) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added {} scenario {} to slice {}",
            scenario.kind().as_str(),
            scenario.name().as_ref(),
            scenario.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added {} scenario {} to project root",
            scenario.kind().as_str(),
            scenario.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        scenario.slice_slug(),
        reports,
        EventDraft::slice_scenario_added(scenario),
    )
}

fn interpret_slice_scenario_updated(
    scenario: &NewSliceScenario,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "updated scenario {} on slice {}",
            scenario.name().as_ref(),
            scenario.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "updated scenario {} in project root",
            scenario.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        scenario.slice_slug(),
        reports,
        EventDraft::slice_scenario_updated(scenario),
    )
}

fn interpret_slice_scenario_removed(
    removal: &SliceScenarioRemovalEffect,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "removed scenario {} from slice {}",
            removal.scenario_name().as_ref(),
            removal.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "removed scenario {} from project root",
            removal.scenario_name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        removal.slice_slug(),
        reports,
        EventDraft::slice_scenario_removed(removal.slice_slug(), removal.scenario_name()),
    )
}

fn interpret_translation_added(
    translation: &NewTranslationDefinition,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![
        projected_report(format!(
            "added translation {} to slice {}",
            translation.name().as_ref(),
            translation.slice_slug().as_ref()
        ))?,
        projected_report(format!(
            "added translation {} to project root",
            translation.name().as_ref()
        ))?,
    ];
    interpret_slice_addition(
        translation.slice_slug(),
        reports,
        EventDraft::slice_translation_added(translation),
    )
}

/// Shared tail for the uniform "add fact to an existing workflow" effects:
/// confirm the workflow is modeled, then export the fact and regenerate the
/// projection. Returns the precomputed report unchanged.
fn interpret_workflow_addition(
    workflow_slug: &WorkflowSlug,
    reports: Vec<String>,
    draft: EventDraft,
) -> Result<Vec<String>, ShellError> {
    read_formal_workflow_graph(workflow_slug)?;
    export_and_regenerate(draft)?;
    Ok(reports)
}

/// Per-workflow fact additions that only need the workflow to exist. Returns
/// `None` for effects handled by other clusters.
fn interpret_workflow_fact_effect(effect: &Effect) -> Option<Result<Vec<String>, ShellError>> {
    Some(match effect {
        Effect::AddWorkflowCommandErrorFromWorkflow(effect) => {
            interpret_workflow_command_error_added(effect)
        }
        Effect::AddWorkflowOwnedDefinitionFromWorkflow(effect) => {
            interpret_workflow_owned_definition_added(effect)
        }
        Effect::AddWorkflowOutcomeFromWorkflow(effect) => interpret_workflow_outcome_added(effect),
        Effect::AddWorkflowTransitionEvidenceFromWorkflow(effect) => {
            interpret_workflow_transition_evidence_added(effect)
        }
        Effect::RequireWorkflowEntryLifecycleCoverageFromWorkflow(workflow_slug) => {
            interpret_workflow_entry_lifecycle_coverage_required(workflow_slug)
        }
        Effect::AddWorkflowEntryLifecycleStateFromWorkflow(effect) => {
            interpret_workflow_entry_lifecycle_state_added(effect)
        }
        _ => return None,
    })
}

fn interpret_workflow_command_error_added(
    effect: &WorkflowCommandErrorEffect,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![projected_report(format!(
        "added workflow command error {} to workflow {}",
        effect.error().error_name().as_ref(),
        effect.workflow_slug().as_ref()
    ))?];
    interpret_workflow_addition(
        effect.workflow_slug(),
        reports,
        EventDraft::workflow_command_error_added(effect.workflow_slug(), effect.error()),
    )
}

fn interpret_workflow_owned_definition_added(
    effect: &WorkflowOwnedDefinitionEffect,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![projected_report(format!(
        "added workflow owned definition {} {} to workflow {}",
        effect.definition().definition_kind().as_ref(),
        effect.definition().definition_name().as_ref(),
        effect.workflow_slug().as_ref()
    ))?];
    interpret_workflow_addition(
        effect.workflow_slug(),
        reports,
        EventDraft::workflow_owned_definition_added(effect.workflow_slug(), effect.definition()),
    )
}

fn interpret_workflow_outcome_added(
    effect: &WorkflowOutcomeEffect,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![projected_report(format!(
        "added workflow outcome {} to workflow {}",
        effect.outcome().label().as_ref(),
        effect.workflow_slug().as_ref()
    ))?];
    interpret_workflow_addition(
        effect.workflow_slug(),
        reports,
        EventDraft::workflow_outcome_added(effect.workflow_slug(), effect.outcome()),
    )
}

fn interpret_workflow_transition_evidence_added(
    effect: &WorkflowTransitionEvidenceEffect,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![projected_report(format!(
        "added workflow transition evidence {} {} to workflow {}",
        effect.evidence().kind().as_ref(),
        effect.evidence().trigger().as_ref(),
        effect.workflow_slug().as_ref()
    ))?];
    interpret_workflow_addition(
        effect.workflow_slug(),
        reports,
        EventDraft::workflow_transition_evidence_added(effect.workflow_slug(), effect.evidence()),
    )
}

fn interpret_workflow_entry_lifecycle_coverage_required(
    workflow_slug: &WorkflowSlug,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![projected_report(format!(
        "marked workflow {} as requiring entry lifecycle coverage",
        workflow_slug.as_ref()
    ))?];
    interpret_workflow_addition(
        workflow_slug,
        reports,
        EventDraft::workflow_entry_lifecycle_coverage_required(workflow_slug),
    )
}

fn interpret_workflow_entry_lifecycle_state_added(
    effect: &WorkflowEntryLifecycleStateEffect,
) -> Result<Vec<String>, ShellError> {
    let reports = vec![projected_report(format!(
        "added workflow entry lifecycle state {} to workflow {}",
        effect.coverage().state().as_ref(),
        effect.workflow_slug().as_ref()
    ))?];
    interpret_workflow_addition(
        effect.workflow_slug(),
        reports,
        EventDraft::workflow_entry_lifecycle_state_added(effect.workflow_slug(), effect.coverage()),
    )
}

/// Effects that mutate workflow/slice structure by building an `EffectPlan` from
/// the modeled workflow graphs. Returns `None` for effects handled elsewhere.
fn interpret_workflow_structure_effect(effect: &Effect) -> Option<Result<Vec<String>, ShellError>> {
    Some(match effect {
        Effect::AddSliceFromWorkflow(slice) => interpret_add_slice(slice),
        Effect::AddWorkflowFromIndex(workflow) => interpret_add_workflow(workflow),
        Effect::ConnectWorkflowFromWorkflow(connection) => interpret_connect_workflow(connection),
        Effect::RemoveSliceFromWorkflow(slug) => interpret_remove_slice(slug),
        Effect::RemoveTransitionFromWorkflow(removal) => interpret_remove_transition(removal),
        Effect::RemoveWorkflowFromIndex(slug) => interpret_remove_workflow(slug),
        Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(effect) => {
            interpret_update_workflow_description(effect)
        }
        Effect::UpdateWorkflowNameFromIndexAndWorkflow(effect) => {
            interpret_update_workflow_name(effect)
        }
        Effect::UpdateSliceDescriptionFromWorkflow(effect) => {
            interpret_update_slice_description(effect)
        }
        Effect::UpdateSliceKindFromWorkflow(effect) => interpret_update_slice_kind(effect),
        Effect::UpdateSliceNameFromWorkflow(effect) => interpret_update_slice_name(effect),
        _ => return None,
    })
}

fn interpret_add_slice(slice: &NewSlice) -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    let (workflow_layout, workflow_graph) =
        formal_workflow_layout_and_graph(&formal_workflows, slice.workflow_slug())?;
    let plan = add_slice(
        &project_name,
        &formal_workflows,
        workflow_layout.name(),
        workflow_layout.description(),
        &workflow_graph,
        slice,
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_add_workflow(workflow: &NewWorkflow) -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?;
    let existing_slice_memberships = formal_project_slice_memberships(&formal_workflows);
    let existing_workflows = formal_workflow_layouts(formal_workflows);
    let plan = add_workflow(
        &project_name,
        &ModeledWorkflowLayouts::new(existing_workflows),
        &existing_slice_memberships,
        workflow,
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_connect_workflow(connection: &WorkflowConnection) -> Result<Vec<String>, ShellError> {
    let (workflow_layout, workflow_graph) =
        read_formal_workflow_layout_and_graph(connection.workflow_slug())?;
    let plan = connect_workflow(
        workflow_layout.name(),
        workflow_layout.description(),
        &workflow_graph,
        connection,
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    let reports = interpret_collect_reports(&plan)?;
    interpret_effect(&Effect::ExportEvent(EventDraft::workflow_connected(
        connection,
    )))?;
    Ok(reports)
}

fn interpret_remove_slice(slug: &SliceSlug) -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    let (workflow_layout, workflow_graph) =
        find_formal_workflow_containing_slice_in(&formal_workflows, slug)?;
    let plan = remove_slice(
        &project_name,
        &formal_workflows,
        workflow_layout.name(),
        workflow_layout.description(),
        workflow_layout.slug(),
        &workflow_graph,
        slug,
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_remove_transition(
    removal: &WorkflowTransitionRemoval,
) -> Result<Vec<String>, ShellError> {
    let (workflow_layout, workflow_graph) =
        read_formal_workflow_layout_and_graph(removal.workflow_slug())?;
    let plan = remove_transition(
        workflow_layout.name(),
        workflow_layout.description(),
        &workflow_graph,
        removal,
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    let reports = interpret_collect_reports(&plan)?;
    interpret_effect(&Effect::ExportEvent(
        EventDraft::workflow_transition_removed(removal),
    ))?;
    Ok(reports)
}

fn interpret_remove_workflow(slug: &WorkflowSlug) -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    let existing_workflows =
        formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(formal_workflows.clone()));
    let workflow_graphs = indexed_formal_workflow_graphs(&formal_workflows);
    let plan = remove_workflow(
        &project_name,
        &ModeledWorkflowLayouts::new(existing_workflows),
        &IndexedWorkflowGraphs::new(workflow_graphs),
        slug,
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_update_workflow_description(
    effect: &WorkflowDescriptionUpdateEffect,
) -> Result<Vec<String>, ShellError> {
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    let existing_workflows =
        formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(formal_workflows.clone()));
    let workflow_graph = read_formal_workflow_graph(effect.workflow_slug())?;
    let plan = update_workflow_description(
        ModeledWorkflowLayouts::new(existing_workflows),
        &workflow_graph,
        effect.workflow_slug().clone(),
        effect.description().clone(),
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_update_workflow_name(
    effect: &WorkflowNameUpdateEffect,
) -> Result<Vec<String>, ShellError> {
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    let existing_workflows =
        formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(formal_workflows.clone()));
    let workflow_graph = read_formal_workflow_graph(effect.workflow_slug())?;
    let plan = update_workflow_name(
        ModeledWorkflowLayouts::new(existing_workflows),
        &workflow_graph,
        effect.workflow_slug(),
        effect.name(),
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_update_slice_description(
    effect: &SliceDescriptionUpdateEffect,
) -> Result<Vec<String>, ShellError> {
    let (workflow_layout, workflow_graph) =
        find_formal_workflow_containing_slice(effect.slice_slug())?;
    let plan = update_slice_description(
        workflow_layout.name(),
        workflow_layout.description(),
        workflow_layout.slug().clone(),
        &workflow_graph,
        effect.slice_slug(),
        effect.description().clone(),
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_update_slice_kind(effect: &SliceKindUpdateEffect) -> Result<Vec<String>, ShellError> {
    let (workflow_layout, workflow_graph) =
        find_formal_workflow_containing_slice(effect.slice_slug())?;
    let plan = update_slice_kind(
        workflow_layout.name(),
        workflow_layout.description(),
        workflow_layout.slug().clone(),
        &workflow_graph,
        effect.slice_slug(),
        effect.kind(),
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

fn interpret_update_slice_name(effect: &SliceNameUpdateEffect) -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    let (workflow_layout, workflow_graph) =
        find_formal_workflow_containing_slice_in(&formal_workflows, effect.slice_slug())?;
    let plan = update_slice_name(
        &SliceProjectRootContext::new(project_name, &formal_workflows),
        workflow_layout.name(),
        workflow_layout.description(),
        workflow_layout.slug().clone(),
        &workflow_graph,
        effect.slice_slug(),
        effect.name().clone(),
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(&plan)
}

/// Read-only listing / inspection / verification effects. Returns `None` for
/// effects handled elsewhere.
fn interpret_listing_effect(effect: &Effect) -> Option<Result<Vec<String>, ShellError>> {
    Some(match effect {
        Effect::CheckCurrentProject => interpret_check_current_project(),
        Effect::ListConflictsFromEvents => {
            interpret_collect_reports(&match list_event_conflicts().map_err(ShellError::message) {
                Ok(plan) => plan,
                Err(error) => return Some(Err(error)),
            })
        }
        Effect::ListWorkflowsFromIndex => interpret_list_workflows(),
        Effect::ListSlicesFromIndex => {
            let modeled_slices =
                formal_workflow_slice_details(match read_synchronized_formal_workflow_graphs() {
                    Ok(graphs) => graphs,
                    Err(error) => return Some(Err(error)),
                });
            interpret_collect_reports(&list_slices(ModeledWorkflowSliceDetails::new(
                modeled_slices,
            )))
        }
        Effect::ListTransitionsFromIndex => {
            let modeled_transitions =
                formal_workflow_transitions(match read_synchronized_formal_workflow_graphs() {
                    Ok(graphs) => graphs,
                    Err(error) => return Some(Err(error)),
                });
            interpret_collect_reports(&list_transitions(ModeledWorkflowTransitions::new(
                modeled_transitions,
            )))
        }
        Effect::ShowSliceFromSlice(slug) => interpret_show_slice(slug),
        Effect::ShowWorkflowFromWorkflow(slug) => interpret_show_workflow(slug),
        Effect::VerifyProjectFromIndex => interpret_verify_project_from_index_collecting(),
        _ => return None,
    })
}

fn interpret_check_current_project() -> Result<Vec<String>, ShellError> {
    let project_name = read_project_manifest_name()?;
    let formal_workflows = read_synchronized_formal_workflow_graphs()?;
    // Project-root inventories are sourced from the authoritative event
    // log, never parsed back out of the generated Lean/Quint root
    // artifact. The artifact is a write-only projection; `check_project`
    // verifies the on-disk artifact matches these log-sourced rows.
    let project_inventories = projected_project_root_inventories().map_err(ShellError::message)?;
    interpret_collect_reports(&check_project(
        &project_name,
        formal_workflows,
        &project_inventories,
    ))
}

fn interpret_list_workflows() -> Result<Vec<String>, ShellError> {
    let modeled_workflows = formal_workflow_layouts(read_synchronized_formal_workflow_graphs()?);
    let mut reports = interpret_collect_reports(&list_workflows(ModeledWorkflowLayouts::new(
        modeled_workflows,
    )))?;
    reports.extend(interpret_collect_reports(
        &list_stale_workflow_readiness().map_err(ShellError::message)?,
    )?);
    Ok(reports)
}

fn interpret_show_slice(slug: &SliceSlug) -> Result<Vec<String>, ShellError> {
    let slice_document = read_formal_slice_artifacts(slug)?;
    interpret_collect_reports(&show_document(slice_document))
}

fn interpret_show_workflow(slug: &WorkflowSlug) -> Result<Vec<String>, ShellError> {
    let workflow_document = read_formal_workflow_artifacts(slug)?;
    interpret_collect_reports(&show_workflow(workflow_document))
}

/// Artifact-precondition requirement effects. Returns `None` for effects
/// handled elsewhere.
fn interpret_requirement_effect(effect: &Effect) -> Option<Result<Vec<String>, ShellError>> {
    Some(match effect {
        Effect::RequireCanonicalDeclaration(requirement) => {
            interpret_require_canonical_declaration(requirement)
        }
        Effect::RequireDigest(requirement) => interpret_require_digest(requirement),
        Effect::RequireFile(path) => interpret_require_file(path),
        Effect::RequireFileContentsWithAuthoredFormalFacts(requirement) => {
            require_file_contents_with_authored_formal_facts(
                requirement.path().as_ref(),
                requirement.expected().as_ref(),
                requirement.message().as_ref(),
            )
            .map(|()| Vec::new())
        }
        Effect::RequireOnlyModeledArtifacts(requirement) => require_only_modeled_artifacts(
            requirement.path().as_ref(),
            requirement.extension().as_ref(),
            requirement.allowed_paths().as_slice(),
            requirement.message().as_ref(),
        )
        .map(|()| Vec::new()),
        Effect::RequireReviewRecord(requirement) => interpret_require_review_record(requirement),
        _ => return None,
    })
}

fn interpret_require_canonical_declaration(
    requirement: &CanonicalDeclarationRequirement,
) -> Result<Vec<String>, ShellError> {
    let contents =
        fs::read_to_string(Path::new(requirement.path().as_ref())).map_err(ShellError::io)?;
    if artifact_contains_one_canonical_declaration(
        &contents,
        requirement.prefix().as_ref(),
        requirement.marker().as_ref(),
    ) {
        Ok(Vec::new())
    } else {
        Err(ShellError::message(
            requirement.message().as_ref().to_owned(),
        ))
    }
}

fn interpret_require_digest(
    requirement: &ArtifactDigestRequirement,
) -> Result<Vec<String>, ShellError> {
    let contents =
        fs::read_to_string(Path::new(requirement.path().as_ref())).map_err(ShellError::io)?;
    if artifact_contains_one_digest_marker(&contents, requirement.digest().as_ref()) {
        Ok(Vec::new())
    } else {
        Err(ShellError::message(
            requirement.message().as_ref().to_owned(),
        ))
    }
}

fn interpret_require_file(path: &ProjectPath) -> Result<Vec<String>, ShellError> {
    if Path::new(path.as_ref()).is_file() {
        Ok(Vec::new())
    } else {
        Err(ShellError::message(format!(
            "missing required project artifact {}",
            path.as_ref()
        )))
    }
}

fn interpret_require_review_record(
    requirement: &ReviewRecordRequirement,
) -> Result<Vec<String>, ShellError> {
    if Path::new(requirement.path().as_ref()).is_file() {
        require_clean_review_record(
            requirement.path().as_ref(),
            requirement.workflow_slug(),
            requirement.message().as_ref(),
        )
        .map(|()| Vec::new())
    } else {
        Err(ShellError::message(
            requirement.message().as_ref().to_owned(),
        ))
    }
}

/// Terminal cluster covering filesystem, process, and event-store effects. This
/// match is total over the remaining `Effect` variants so the dispatcher needs
/// no fallback.
fn interpret_io_effect(effect: &Effect) -> Result<Vec<String>, ShellError> {
    match effect {
        Effect::EnsureDirectory(path) => fs::create_dir_all(Path::new(path.as_ref()))
            .map(|()| Vec::new())
            .map_err(ShellError::io),
        Effect::ExportEvent(draft) => interpret_export_event(draft),
        Effect::RunProcess(invocation) => run_process(invocation),
        Effect::RunProcessBatch(invocations) => run_process_batch(invocations),
        Effect::DeclareWorkflowReadinessFromWorkflow(readiness) => interpret_effect(
            &Effect::ExportEvent(EventDraft::workflow_readiness_declared(
                readiness.workflow_slug(),
                readiness.projection_fingerprint(),
                readiness.model_content_digest(),
                readiness.verified_at(),
                readiness.verified_by(),
                readiness.review_event(),
            )),
        ),
        Effect::RecordCleanReviewFromWorkflow(effect) => interpret_record_clean_review(effect),
        Effect::RemoveFile(path) => remove_file_if_present(path.as_ref()).map(|()| Vec::new()),
        Effect::ResolveEventConflict(resolution) => interpret_resolve_event_conflict(resolution),
        Effect::WriteFile(write) => {
            write_file(write.path().as_ref(), write.contents().as_ref()).map(|()| Vec::new())
        }
        Effect::WriteFormalSliceArtifactPreservingAuthoredFacts(write) => {
            write_formal_slice_artifact_preserving_authored_facts(
                write.source().as_ref(),
                write.target().as_ref(),
                write.generated().as_ref(),
            )
            .map(|()| Vec::new())
        }
        Effect::WriteFileIfMissing(write) => interpret_write_file_if_missing(write),
        Effect::Report(line) => Ok(vec![line.as_ref().to_owned()]),
        Effect::ReportDocument(contents) => Ok(vec![contents.as_ref().to_owned()]),
        // Every other `Effect` variant is dispatched by an earlier cluster in
        // `interpret_effect`, so this arm is never reached; surface a clear
        // error rather than silently succeeding if routing ever regresses.
        _ => Err(ShellError::message(
            "internal error: effect was not routed to an interpreter cluster",
        )),
    }
}

fn interpret_export_event(draft: &EventDraft) -> Result<Vec<String>, ShellError> {
    if exported_events_are_projecting()? {
        return Ok(Vec::new());
    }
    execute_eventcore_command_for_exported_event(Path::new("."), draft)
        .map_err(ShellError::message)?;
    Ok(Vec::new())
}

fn interpret_record_clean_review(effect: &CleanReviewEffect) -> Result<Vec<String>, ShellError> {
    let current_digest = formal_model_content_digest(effect.workflow_slug())?;
    let required_categories = required_review_categories();
    let plan = record_clean_review(
        effect.workflow_slug(),
        &current_digest,
        effect.reviewer_id(),
        effect.reviewed_at(),
        &RequiredReviewCategories::new(required_categories.clone()),
    )
    .map_err(|error| ShellError::message(error.to_string()))?;
    let reports = interpret_collect_reports(&plan)?;
    interpret_effect(&Effect::ExportEvent(EventDraft::review_recorded(
        effect.workflow_slug(),
        &current_digest,
        effect.reviewer_id(),
        effect.reviewed_at(),
        &required_categories,
    )))?;
    Ok(reports)
}

fn interpret_resolve_event_conflict(
    resolution: &EventConflictResolution,
) -> Result<Vec<String>, ShellError> {
    let plan = resolve_event_conflict(
        Path::new("."),
        resolution.conflict_id(),
        resolution.chosen_event_id(),
    )
    .map_err(ShellError::message)?;
    plan.effects()
        .iter()
        .try_fold(Vec::new(), |mut reports, effect| {
            reports.extend(interpret_effect(effect)?);
            Ok(reports)
        })
}

fn interpret_write_file_if_missing(write: &FileWriteEffect) -> Result<Vec<String>, ShellError> {
    if Path::new(write.path().as_ref()).exists() {
        Ok(Vec::new())
    } else {
        write_file(write.path().as_ref(), write.contents().as_ref()).map(|()| Vec::new())
    }
}

fn read_project_manifest_name() -> Result<ProjectName, ShellError> {
    let manifest = fs::read_to_string("emc.toml").map_err(ShellError::io)?;
    parse_project_manifest_name(&manifest).map_err(ShellError::project_name)
}

/// The workflow graphs that command decisions consume, sourced from the
/// authoritative event log (`ProjectedModel`) rather than parsed from the
/// generated Lean/Quint artifacts. The event log is the single source of truth;
/// the Lean/Quint artifacts are write-only projections of it (regenerated by
/// `project_exported_events`) and are never parsed back to drive a decision.
fn read_synchronized_formal_workflow_graphs() -> Result<FormalWorkflowGraphs, ShellError> {
    projected_formal_workflow_graphs()
        .map(FormalWorkflowGraphs::from_graphs)
        .map_err(ShellError::message)
}

fn formal_workflow_layouts(graphs: FormalWorkflowGraphs) -> Vec<ModeledWorkflowLayout> {
    graphs
        .into_inner()
        .into_iter()
        .map(|graph| formal_workflow_layout(&graph))
        .collect()
}

fn formal_project_slice_memberships(graphs: &FormalWorkflowGraphs) -> ProjectSliceMemberships {
    ProjectSliceMemberships::from_memberships(graphs.as_slice().iter().flat_map(|workflow| {
        workflow.slice_details().as_slice().iter().map(|slice| {
            let module_name = module_name_from_raw(slice.name().as_ref());
            ProjectSliceMembership::new(
                workflow.slug().clone(),
                slice.slug().clone(),
                lean_module_name(module_name),
            )
        })
    }))
}

fn formal_workflow_layout(graph: &FormalWorkflowGraph) -> ModeledWorkflowLayout {
    ModeledWorkflowLayout::new(
        graph.name().clone(),
        graph.description().clone(),
        graph.slug().clone(),
    )
}

fn formal_workflow_layout_and_graph(
    graphs: &[FormalWorkflowGraph],
    slug: &WorkflowSlug,
) -> Result<(ModeledWorkflowLayout, FormalWorkflowGraph), ShellError> {
    graphs
        .iter()
        .find(|graph| graph.slug() == slug)
        .cloned()
        .map(|graph| (formal_workflow_layout(&graph), graph))
        .ok_or_else(|| ShellError::message(format!("unknown workflow {}", slug.as_ref())))
}

fn formal_workflow_slice_details(graphs: FormalWorkflowGraphs) -> Vec<WorkflowSliceDetail> {
    graphs
        .into_inner()
        .into_iter()
        .flat_map(|graph| graph.slice_details().as_slice().to_owned())
        .collect()
}

fn formal_workflow_transitions(graphs: FormalWorkflowGraphs) -> Vec<WorkflowTransitionRecord> {
    graphs
        .into_inner()
        .into_iter()
        .flat_map(|graph| graph.transitions().as_slice().to_owned())
        .collect()
}

fn read_formal_workflow_graph(slug: &WorkflowSlug) -> Result<FormalWorkflowGraph, ShellError> {
    read_synchronized_formal_workflow_graphs()?
        .into_inner()
        .into_iter()
        .find(|graph| graph.slug() == slug)
        .ok_or_else(|| ShellError::message(format!("workflow {} is not modeled", slug.as_ref())))
}

fn read_formal_workflow_artifacts(slug: &WorkflowSlug) -> Result<FileContents, ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    let module_name = module_name_from_raw(graph.name().as_ref());
    formal_artifact_bundle(&[
        format!("model/lean/{module_name}.lean"),
        format!("model/quint/{module_name}.qnt"),
    ])
}

fn read_formal_slice_artifacts(slug: &SliceSlug) -> Result<FileContents, ShellError> {
    read_synchronized_formal_workflow_graphs()?
        .into_inner()
        .into_iter()
        .find_map(|graph| {
            graph
                .slice_details()
                .as_slice()
                .iter()
                .find(|slice| slice.slug() == slug)
                .map(|slice| module_name_from_raw(slice.name().as_ref()))
        })
        .map_or_else(
            || {
                Err(ShellError::message(format!(
                    "slice {} is not referenced by any modeled workflow",
                    slug.as_ref()
                )))
            },
            |module_name| {
                formal_artifact_bundle(&[
                    format!("model/lean/slices/{module_name}.lean"),
                    format!("model/quint/slices/{module_name}.qnt"),
                ])
            },
        )
}

fn formal_artifact_bundle(paths: &[String]) -> Result<FileContents, ShellError> {
    let mut bundle = String::new();
    for path in paths {
        let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
        bundle.push_str("# ");
        bundle.push_str(path);
        bundle.push('\n');
        bundle.push_str(&contents);
        if !contents.ends_with('\n') {
            bundle.push('\n');
        }
        bundle.push('\n');
    }
    FileContents::try_new(bundle).map_err(|error| ShellError::message(error.to_string()))
}

fn read_formal_workflow_layout_and_graph(
    slug: &WorkflowSlug,
) -> Result<(ModeledWorkflowLayout, FormalWorkflowGraph), ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    Ok((formal_workflow_layout(&graph), graph))
}

fn indexed_formal_workflow_graphs(graphs: &[FormalWorkflowGraph]) -> Vec<IndexedWorkflowGraph> {
    graphs
        .iter()
        .map(|graph| IndexedWorkflowGraph::new(graph.slug().clone(), graph.clone()))
        .collect()
}

fn find_formal_workflow_containing_slice(
    slug: &SliceSlug,
) -> Result<(ModeledWorkflowLayout, FormalWorkflowGraph), ShellError> {
    let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
    find_formal_workflow_containing_slice_in(&formal_workflows, slug)
}

fn find_formal_workflow_containing_slice_in(
    formal_workflows: &[FormalWorkflowGraph],
    slug: &SliceSlug,
) -> Result<(ModeledWorkflowLayout, FormalWorkflowGraph), ShellError> {
    formal_workflows
        .iter()
        .find(|graph| {
            graph
                .slice_details()
                .as_slice()
                .iter()
                .any(|slice| slice.slug() == slug)
        })
        .cloned()
        .map(|graph| (formal_workflow_layout(&graph), graph))
        .ok_or_else(|| {
            ShellError::message(format!(
                "slice {} is not referenced by any indexed workflow",
                slug.as_ref()
            ))
        })
}

fn require_file_contents_with_authored_formal_facts(
    path: &str,
    expected: &str,
    message: &str,
) -> Result<(), ShellError> {
    let actual = fs::read_to_string(Path::new(path))
        .map_err(|_error| ShellError::message(message.to_owned()))?;
    if normalize_authored_formal_fact_lists(&actual)
        == normalize_authored_formal_fact_lists(expected)
    {
        Ok(())
    } else {
        Err(ShellError::message(message.to_owned()))
    }
}

fn normalize_authored_formal_fact_lists(contents: &str) -> String {
    const MARKERS: &[&str] = &[
        "def sliceCommands : List SliceCommandReference := ",
        "def sliceCommandDefinitions : List CommandDefinition := ",
        "def sliceReferencedCommands : List SliceCommandReference := ",
        "def sliceAutomations : List AutomationDefinition := ",
        "def sliceTranslations : List TranslationDefinition := ",
        "def sliceBoardElements : List BoardElement := ",
        "def sliceBoardConnections : List BoardConnection := ",
        "def sliceOutcomeDefinitions : List OutcomeDefinition := ",
        "def sliceEvents : List SliceEventReference := ",
        "def sliceStreams : List StreamDefinition := ",
        "def sliceExternalPayloads : List ExternalPayloadDefinition := ",
        "def sliceEventDefinitions : List EventDefinition := ",
        "def sliceReadModels : List SliceReadModelReference := ",
        "def sliceReadModelDefinitions : List ReadModelDefinition := ",
        "def sliceViews : List SliceViewReference := ",
        "def sliceViewDefinitions : List ViewDefinition := ",
        "def sliceAcceptanceScenarios : List EventModelScenario := ",
        "def sliceContractScenarios : List EventModelScenario := ",
        "def sliceBitLevelDataFlows : List BitLevelDataFlow := ",
        "val sliceCommands: List[SliceCommandReference] = ",
        "val sliceCommandDefinitions: List[CommandDefinition] = ",
        "val sliceReferencedCommands: List[SliceCommandReference] = ",
        "val sliceAutomations: List[AutomationDefinition] = ",
        "val sliceTranslations: List[TranslationDefinition] = ",
        "val sliceBoardElements: List[BoardElement] = ",
        "val sliceBoardConnections: List[BoardConnection] = ",
        "val sliceOutcomeDefinitions: List[OutcomeDefinition] = ",
        "val sliceEvents: List[SliceEventReference] = ",
        "val sliceStreams: List[StreamDefinition] = ",
        "val sliceExternalPayloads: List[ExternalPayloadDefinition] = ",
        "val sliceEventDefinitions: List[EventDefinition] = ",
        "val sliceReadModels: List[SliceReadModelReference] = ",
        "val sliceReadModelDefinitions: List[ReadModelDefinition] = ",
        "val sliceViews: List[SliceViewReference] = ",
        "val sliceViewDefinitions: List[ViewDefinition] = ",
        "val sliceAcceptanceScenarios: List[EventModelScenario] = ",
        "val sliceContractScenarios: List[EventModelScenario] = ",
        "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = ",
    ];
    let mut normalized = contents
        .lines()
        .map(|line| {
            // `trim_start` yields a suffix of `line`, so its length never
            // exceeds `line.len()`; saturation is exact, not a clamp.
            let indentation_length = line.len().saturating_sub(line.trim_start().len());
            let (indentation, declaration) = line.split_at(indentation_length);
            MARKERS
                .iter()
                .find_map(|marker| {
                    declaration
                        .starts_with(marker)
                        .then(|| format!("{indentation}{marker}[]"))
                })
                .unwrap_or_else(|| line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n");
    if contents.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn write_formal_slice_artifact_preserving_authored_facts(
    source: &str,
    target: &str,
    generated: &str,
) -> Result<(), ShellError> {
    let contents = match fs::read_to_string(Path::new(source)) {
        Ok(existing) => preserve_authored_formal_fact_lists(&existing, generated),
        Err(error) if error.kind() == io::ErrorKind::NotFound => generated.to_owned(),
        Err(error) => return Err(ShellError::io(error)),
    };
    write_file(target, &contents)
}

fn preserve_authored_formal_fact_lists(existing: &str, generated: &str) -> String {
    const MARKERS: &[&str] = &[
        "def sliceCommands : List SliceCommandReference := ",
        "def sliceCommandDefinitions : List CommandDefinition := ",
        "def sliceReferencedCommands : List SliceCommandReference := ",
        "def sliceAutomations : List AutomationDefinition := ",
        "def sliceTranslations : List TranslationDefinition := ",
        "def sliceBoardElements : List BoardElement := ",
        "def sliceBoardConnections : List BoardConnection := ",
        "def sliceOutcomeDefinitions : List OutcomeDefinition := ",
        "def sliceEvents : List SliceEventReference := ",
        "def sliceStreams : List StreamDefinition := ",
        "def sliceExternalPayloads : List ExternalPayloadDefinition := ",
        "def sliceEventDefinitions : List EventDefinition := ",
        "def sliceReadModels : List SliceReadModelReference := ",
        "def sliceReadModelDefinitions : List ReadModelDefinition := ",
        "def sliceViews : List SliceViewReference := ",
        "def sliceViewDefinitions : List ViewDefinition := ",
        "def sliceAcceptanceScenarios : List EventModelScenario := ",
        "def sliceContractScenarios : List EventModelScenario := ",
        "def sliceBitLevelDataFlows : List BitLevelDataFlow := ",
        "val sliceCommands: List[SliceCommandReference] = ",
        "val sliceCommandDefinitions: List[CommandDefinition] = ",
        "val sliceReferencedCommands: List[SliceCommandReference] = ",
        "val sliceAutomations: List[AutomationDefinition] = ",
        "val sliceTranslations: List[TranslationDefinition] = ",
        "val sliceBoardElements: List[BoardElement] = ",
        "val sliceBoardConnections: List[BoardConnection] = ",
        "val sliceOutcomeDefinitions: List[OutcomeDefinition] = ",
        "val sliceEvents: List[SliceEventReference] = ",
        "val sliceStreams: List[StreamDefinition] = ",
        "val sliceExternalPayloads: List[ExternalPayloadDefinition] = ",
        "val sliceEventDefinitions: List[EventDefinition] = ",
        "val sliceReadModels: List[SliceReadModelReference] = ",
        "val sliceReadModelDefinitions: List[ReadModelDefinition] = ",
        "val sliceViews: List[SliceViewReference] = ",
        "val sliceViewDefinitions: List[ViewDefinition] = ",
        "val sliceAcceptanceScenarios: List[EventModelScenario] = ",
        "val sliceContractScenarios: List[EventModelScenario] = ",
        "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = ",
    ];
    let existing_declarations = MARKERS
        .iter()
        .filter_map(|marker| {
            authored_formal_fact_declaration(existing, marker)
                .map(|declaration| (*marker, declaration))
        })
        .collect::<Vec<_>>();
    let mut preserved = generated
        .lines()
        .map(|line| {
            // `trim_start` yields a suffix of `line`, so its length never
            // exceeds `line.len()`; saturation is exact, not a clamp.
            let indentation_length = line.len().saturating_sub(line.trim_start().len());
            let (indentation, declaration) = line.split_at(indentation_length);
            existing_declarations
                .iter()
                .find_map(|(marker, authored_declaration)| {
                    declaration
                        .starts_with(marker)
                        .then(|| format!("{indentation}{authored_declaration}"))
                })
                .unwrap_or_else(|| line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n");
    if generated.ends_with('\n') {
        preserved.push('\n');
    }
    preserved
}

fn authored_formal_fact_declaration(contents: &str, marker: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let declaration = line.trim_start();
        declaration
            .starts_with(marker)
            .then(|| declaration.to_owned())
    })
}

fn require_only_modeled_artifacts(
    path: &str,
    extension: &str,
    allowed_paths: &[ProjectPath],
    message: &str,
) -> Result<(), ShellError> {
    let allowed_file_names = allowed_artifact_file_names(allowed_paths);
    let mut artifact_files = fs::read_dir(Path::new(path))
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    artifact_files.sort();

    artifact_files
        .into_iter()
        .filter_map(|artifact_path| {
            artifact_path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .filter(|file_name| file_name.ends_with(extension))
                .map(str::to_owned)
        })
        .find(|file_name| !allowed_file_names.contains(file_name))
        .map_or(Ok(()), |file_name| {
            Err(ShellError::message(format!("{message} for {file_name}")))
        })
}

fn allowed_artifact_file_names(allowed_paths: &[ProjectPath]) -> BTreeSet<String> {
    allowed_paths
        .iter()
        .filter_map(|path| {
            Path::new(path.as_ref())
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(str::to_owned)
        })
        .collect()
}

fn require_clean_review_record(
    path: &str,
    workflow_slug: &WorkflowSlug,
    fallback_message: &str,
) -> Result<(), ShellError> {
    let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
    let record_contents = FileContents::try_new(contents)
        .map_err(|_error| ShellError::message(fallback_message.to_owned()))?;
    let record = ReviewRecordDocument::parse(&record_contents)
        .map_err(|_error| ShellError::message(fallback_message.to_owned()))?;
    let expected_workflow_slug = review_record_workflow_slug(path)?;
    if !record.matches_workflow(&expected_workflow_slug) {
        let observed = record
            .workflow_slug()
            .map_or_else(String::new, |workflow_slug| {
                workflow_slug.as_ref().to_owned()
            });
        return Err(ShellError::message(format!(
            "review record workflow '{observed}' does not match '{expected_workflow_slug}'"
        )));
    }
    let current_digest = formal_model_content_digest(workflow_slug)?;
    if !record.is_clean() {
        if record.current_mandatory_findings_include(&current_digest) {
            return Err(ShellError::message(
                "mandatory review findings remain for current model digest",
            ));
        }
        if record.has_mandatory_findings() && !record.model_content_digest_matches(&current_digest)
        {
            return Err(ShellError::message(
                "corrected workflow requires clean follow-up review",
            ));
        }
        return Err(ShellError::message(fallback_message.to_owned()));
    }
    if !record.model_content_digest_matches(&current_digest) {
        return Err(ShellError::message(
            "clean review is stale for current model digest",
        ));
    }
    if !record.has_category_results() {
        return Err(ShellError::message(fallback_message.to_owned()));
    }

    let required_categories = required_review_categories();
    match record.first_non_clean_category(&required_categories) {
        Some(ReviewCategoryFinding::NotClean(category)) => Err(ShellError::message(format!(
            "review category '{category}' is not clean"
        ))),
        Some(ReviewCategoryFinding::Missing(category)) => Err(ShellError::message(format!(
            "clean review is missing category '{category}'"
        ))),
        None => Ok(()),
    }
}

fn review_record_workflow_slug(path: &str) -> Result<WorkflowSlug, ShellError> {
    Path::new(path)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .and_then(|file_name| file_name.strip_suffix(".review.json"))
        .ok_or_else(|| ShellError::message("review record path is invalid"))
        .and_then(|slug| {
            WorkflowSlug::try_new(slug.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })
}

fn required_review_categories() -> Vec<ReviewRuleName> {
    ReviewRuleName::REQUIRED.to_vec()
}

fn current_projection_fingerprint() -> Result<ProjectionFingerprint, ShellError> {
    let fingerprint = exported_events_projection_fingerprint()
        .map_err(ShellError::message)?
        .ok_or_else(|| ShellError::message("event export is required before verification"))?;
    ArtifactDigest::try_new(fingerprint)
        .map(ProjectionFingerprint::new)
        .map_err(|error| ShellError::message(error.to_string()))
}

fn readiness_verified_at() -> Result<ReviewTimestamp, ShellError> {
    ReviewTimestamp::try_new("1970-01-01T00:00:00.000Z".to_owned())
        .map_err(|error| ShellError::message(error.to_string()))
}

fn readiness_verified_by() -> Result<ReviewerId, ShellError> {
    ReviewerId::try_new("emc verify".to_owned())
        .map_err(|error| ShellError::message(error.to_string()))
}

fn formal_model_content_digest(slug: &WorkflowSlug) -> Result<ModelContentDigest, ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    let mut digest = StableDigest::new();
    write_formal_artifact_digest(
        &mut digest,
        &format!(
            "model/lean/{}.lean",
            module_name_from_raw(graph.name().as_ref())
        ),
    )?;
    write_formal_artifact_digest(
        &mut digest,
        &format!(
            "model/quint/{}.qnt",
            module_name_from_raw(graph.name().as_ref())
        ),
    )?;
    graph
        .slice_details()
        .as_slice()
        .iter()
        .try_for_each(|slice| {
            let module_name = module_name_from_raw(slice.name().as_ref());
            write_formal_artifact_digest(
                &mut digest,
                &format!("model/lean/slices/{module_name}.lean"),
            )?;
            write_formal_artifact_digest(
                &mut digest,
                &format!("model/quint/slices/{module_name}.qnt"),
            )
        })?;
    ArtifactDigest::try_new(digest.finish())
        .map(ModelContentDigest::new)
        .map_err(|error| ShellError::message(error.to_string()))
}

fn write_formal_artifact_digest(digest: &mut StableDigest, path: &str) -> Result<(), ShellError> {
    let contents = fs::read_to_string(path).map_err(ShellError::io)?;
    digest.write(path);
    digest.write(&contents);
    Ok(())
}

struct StableDigest {
    value: u64,
}

impl StableDigest {
    fn new() -> Self {
        Self {
            value: 0xcbf2_9ce4_8422_2325,
        }
    }

    fn write(&mut self, value: &str) {
        value.as_bytes().iter().for_each(|byte| {
            self.value ^= u64::from(*byte);
            self.value = self.value.wrapping_mul(0x0000_0100_0000_01b3);
        });
    }

    fn finish(self) -> String {
        format!("emc-fnv1a64:{:016x}", self.value)
    }
}

fn artifact_contains_one_canonical_declaration(
    artifact_contents: &str,
    prefix: &str,
    marker: &str,
) -> bool {
    let mut declarations = artifact_contents
        .lines()
        .filter_map(|line| canonical_declaration_line(line, prefix));

    matches!(
        (declarations.next(), declarations.next()),
        (Some(declaration), None) if declaration == marker
    )
}

fn artifact_contains_one_digest_marker(artifact_contents: &str, digest: &str) -> bool {
    let mut declarations = artifact_contents
        .lines()
        .filter_map(canonical_digest_marker_line);

    matches!(
        (declarations.next(), declarations.next()),
        (Some(declaration), None) if declaration == digest
    )
}

fn canonical_digest_marker_line(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("-- EMC-DIGEST: ")
        .or_else(|| trimmed.strip_prefix("// EMC-DIGEST: "))
}

fn canonical_declaration_line<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.starts_with(' ') && line.starts_with(prefix) {
        Some(line)
    } else {
        let trimmed = line.trim_start();
        trimmed.starts_with(prefix).then_some(trimmed)
    }
}

fn module_name_from_raw(raw: &str) -> String {
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

fn lean_module_name(value: impl Into<String>) -> LeanModuleName {
    LeanModuleName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("generated Lean module names must be valid: {error}");
    })
}

fn write_file(path: &str, contents: &str) -> Result<(), ShellError> {
    // Idempotent: skip the write when the file already holds the intended
    // contents. This keeps regenerating artifacts from the event log on every
    // command (the self-healing projection) from churning unchanged files.
    if fs::read_to_string(Path::new(path)).is_ok_and(|existing| existing == contents) {
        return Ok(());
    }
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(ShellError::io)?;
    }
    fs::write(Path::new(path), contents).map_err(ShellError::io)
}

fn remove_file_if_present(path: &str) -> Result<(), ShellError> {
    match fs::remove_file(Path::new(path)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ShellError::io(error)),
    }
}

fn run_process(invocation: &ProcessInvocation) -> Result<Vec<String>, ShellError> {
    let _quint_verification_slot = quint_verification_process_slot(invocation)?;
    let ProcessCommand {
        arguments,
        _endpoint_guard,
    } = process_arguments(invocation)?;
    let output = Command::new(invocation.program().as_ref())
        .args(arguments)
        .output()
        .map_err(|error| {
            ShellError::message(format!(
                "failed to run {}: {}. Use `nix run . -- verify` from this repository or install the pinned EMC tooling from the Nix package",
                invocation.program().as_ref(),
                error
            ))
        })?;

    if output.status.success() {
        Ok(vec![invocation.success().as_ref().to_owned()])
    } else {
        Err(ShellError::message(format!(
            "{} failed with {}{}. Run `emc check` to confirm generated artifacts are synchronized, then run `emc verify` again",
            verification_label(invocation),
            output.status,
            process_diagnostics(&output.stdout, &output.stderr)
        )))
    }
}

fn run_process_batch(invocations: &ProcessInvocations) -> Result<Vec<String>, ShellError> {
    run_process_batch_reporting(invocations, &mut |_invocation| {})
}

fn run_process_batch_with_progress(
    invocations: &ProcessInvocations,
    report: &mut impl FnMut(&str),
) -> Result<Vec<String>, ShellError> {
    let reports = run_process_batch_reporting(invocations, &mut |invocation| {
        let progress = process_progress_report(invocation);
        report(&progress);
    })?;
    for line in &reports {
        report(line);
    }
    Ok(reports)
}

fn run_process_batch_reporting(
    invocations: &ProcessInvocations,
    started: &mut impl FnMut(&ProcessInvocation),
) -> Result<Vec<String>, ShellError> {
    let mut reports = Vec::new();
    let mut first_error = None;
    let parallelism = verification_parallelism()?;

    for chunk in invocations
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .chunks(parallelism.process_count())
    {
        chunk.iter().for_each(&mut *started);
        let handles = chunk
            .iter()
            .cloned()
            .map(|invocation| thread::spawn(move || run_process(&invocation)))
            .collect::<Vec<_>>();

        for handle in handles {
            match handle.join() {
                Ok(Ok(process_reports)) => {
                    reports.extend(process_reports);
                }
                Ok(Err(error)) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
                Err(_panic) => {
                    if first_error.is_none() {
                        first_error = Some(ShellError::message("verification worker panicked"));
                    }
                }
            }
        }
    }

    if let Some(error) = first_error {
        Err(error)
    } else {
        Ok(reports)
    }
}

#[derive(Clone, Copy)]
struct VerificationParallelism {
    process_count: usize,
}

impl VerificationParallelism {
    fn detect() -> Result<Self, ShellError> {
        match env::var(VERIFY_PARALLELISM_ENV) {
            Ok(value) => Self::parse(&value),
            Err(env::VarError::NotPresent) => Ok(Self {
                process_count: thread::available_parallelism().map_or(1, |parallelism| {
                    parallelism.get().min(MAX_PARALLEL_VERIFICATION_PROCESSES)
                }),
            }),
            Err(env::VarError::NotUnicode(_value)) => Err(ShellError::message(format!(
                "{VERIFY_PARALLELISM_ENV} must be a positive integer"
            ))),
        }
    }

    fn parse(value: &str) -> Result<Self, ShellError> {
        let process_count = value.parse::<usize>().map_err(|_error| {
            ShellError::message(format!(
                "{VERIFY_PARALLELISM_ENV} must be a positive integer"
            ))
        })?;
        if process_count == 0 {
            return Err(ShellError::message(format!(
                "{VERIFY_PARALLELISM_ENV} must be a positive integer"
            )));
        }
        Ok(Self { process_count })
    }

    fn process_count(self) -> usize {
        self.process_count
    }
}

fn verification_parallelism() -> Result<VerificationParallelism, ShellError> {
    VerificationParallelism::detect()
}

struct ProcessCommand {
    arguments: Vec<String>,
    _endpoint_guard: Option<ReservedQuintEndpoint>,
}

fn process_arguments(invocation: &ProcessInvocation) -> Result<ProcessCommand, ShellError> {
    let mut arguments = invocation
        .arguments()
        .iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<_>>();

    let mut endpoint_guard = None;
    if invocation.program().as_ref() == "quint"
        && arguments.first().map(String::as_str) == Some("verify")
        && !arguments
            .iter()
            .any(|argument| argument == "--server-endpoint")
    {
        let input_position = arguments.len().saturating_sub(1);
        let endpoint = quint_server_endpoint()?;
        arguments.insert(input_position, "--server-endpoint".to_owned());
        // The first insert grew `arguments` by one, so the endpoint value goes
        // immediately after the flag; `input_position` is a small list index, so
        // saturation is exact, not a clamp.
        arguments.insert(
            input_position.saturating_add(1),
            endpoint.endpoint().to_owned(),
        );
        endpoint_guard = Some(endpoint);
    }

    Ok(ProcessCommand {
        arguments,
        _endpoint_guard: endpoint_guard,
    })
}

struct QuintVerificationProcessSlot {
    file: fs::File,
}

impl Drop for QuintVerificationProcessSlot {
    fn drop(&mut self) {
        drop(FileExt::unlock(&self.file));
    }
}

#[derive(Clone, Copy)]
struct QuintVerificationSlotIndex {
    value: usize,
}

impl QuintVerificationSlotIndex {
    fn new(value: usize) -> Self {
        Self { value }
    }

    fn as_usize(self) -> usize {
        self.value
    }
}

fn quint_verification_process_slot(
    invocation: &ProcessInvocation,
) -> Result<Option<QuintVerificationProcessSlot>, ShellError> {
    if !quint_verify_invocation(invocation) {
        return Ok(None);
    }
    acquire_quint_verification_process_slot().map(Some)
}

fn acquire_quint_verification_process_slot() -> Result<QuintVerificationProcessSlot, ShellError> {
    let parallelism = verification_parallelism()?;
    loop {
        for index in 0..parallelism.process_count() {
            if let Some(slot) =
                try_acquire_quint_verification_slot(QuintVerificationSlotIndex::new(index))?
            {
                return Ok(slot);
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn try_acquire_quint_verification_slot(
    index: QuintVerificationSlotIndex,
) -> Result<Option<QuintVerificationProcessSlot>, ShellError> {
    let lock_directory = quint_verification_lock_directory();
    fs::create_dir_all(&lock_directory).map_err(ShellError::io)?;
    let path = lock_directory.join(format!("quint-verification-{}.lock", index.as_usize()));
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(ShellError::io)?;
    match FileExt::try_lock(&file) {
        Ok(()) => Ok(Some(QuintVerificationProcessSlot { file })),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(ShellError::io(error)),
    }
}

fn quint_verify_invocation(invocation: &ProcessInvocation) -> bool {
    invocation.program().as_ref() == "quint"
        && invocation
            .arguments()
            .iter()
            .next()
            .is_some_and(|argument| argument.as_ref() == "verify")
}

fn quint_verification_lock_directory() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .filter(|path| !path.is_empty())
        .map_or_else(env::temp_dir, PathBuf::from)
        .join("emc")
        .join("verification")
}

struct ReservedQuintEndpoint {
    endpoint: String,
    _lock: Option<QuintEndpointLock>,
}

impl ReservedQuintEndpoint {
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

struct QuintEndpointLock {
    path: PathBuf,
    _file: fs::File,
}

impl Drop for QuintEndpointLock {
    fn drop(&mut self) {
        drop(fs::remove_file(&self.path));
    }
}

fn quint_server_endpoint() -> Result<ReservedQuintEndpoint, ShellError> {
    if let Ok(endpoint) = env::var("EMC_QUINT_SERVER_ENDPOINT") {
        return Ok(ReservedQuintEndpoint {
            endpoint,
            _lock: None,
        });
    }

    (0..128)
        .find_map(|_| reserve_quint_endpoint().transpose())
        .transpose()?
        .ok_or_else(|| {
            ShellError::message("could not reserve a unique local Quint Apalache server endpoint")
        })
}

fn reserve_quint_endpoint() -> Result<Option<ReservedQuintEndpoint>, ShellError> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(ShellError::io)?;
    let port = listener.local_addr().map_err(ShellError::io)?.port();
    if quint_endpoint_port_was_used(port)? {
        return Ok(None);
    }
    let lock_path = env::temp_dir().join(format!("emc-quint-apalache-{port}.lock"));
    let lock_file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(None),
        Err(error) => return Err(ShellError::io(error)),
    };
    drop(listener);
    remember_quint_endpoint_port(port)?;

    Ok(Some(ReservedQuintEndpoint {
        endpoint: format!("127.0.0.1:{port}"),
        _lock: Some(QuintEndpointLock {
            path: lock_path,
            _file: lock_file,
        }),
    }))
}

fn quint_endpoint_port_was_used(port: u16) -> Result<bool, ShellError> {
    let used_ports = used_quint_endpoint_ports()
        .lock()
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(used_ports.contains(&port))
}

fn remember_quint_endpoint_port(port: u16) -> Result<(), ShellError> {
    let mut used_ports = used_quint_endpoint_ports()
        .lock()
        .map_err(|error| ShellError::message(error.to_string()))?;
    used_ports.insert(port);
    Ok(())
}

fn used_quint_endpoint_ports() -> &'static Mutex<BTreeSet<u16>> {
    static USED_PORTS: OnceLock<Mutex<BTreeSet<u16>>> = OnceLock::new();
    USED_PORTS.get_or_init(|| Mutex::new(BTreeSet::new()))
}

fn process_diagnostics(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    let diagnostics = [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if diagnostics.is_empty() {
        String::new()
    } else {
        format!(":\n{diagnostics}")
    }
}

fn verification_label(invocation: &ProcessInvocation) -> &str {
    if invocation.success().as_ref().starts_with("Lean4") {
        "Lean4 verification"
    } else if invocation.success().as_ref().starts_with("Quint") {
        "Quint verification"
    } else {
        "verification command"
    }
}

fn process_progress_report(invocation: &ProcessInvocation) -> String {
    let program = invocation.program().as_ref();
    let arguments = invocation
        .arguments()
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();

    match (program, arguments.as_slice()) {
        ("lake", ["env", "lean", artifact]) => {
            format!("Running Lean4 verification for {artifact}")
        }
        ("quint", ["typecheck", artifact]) => {
            format!("Running Quint typecheck for {artifact}")
        }
        ("quint", arguments) if arguments.first().copied() == Some("verify") => {
            let artifact = arguments.last().copied().unwrap_or("unknown artifact");
            format!("Running Quint verification for {artifact}")
        }
        (_, []) => format!("Running {program}"),
        (_, arguments) => format!("Running {program} {}", arguments.join(" ")),
    }
}
