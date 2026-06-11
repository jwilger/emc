// Copyright 2026 John Wilger

use std::collections::{BTreeMap, BTreeSet};
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

use crate::core::connection::{connect_workflow, remove_transition};
use crate::core::effect::{
    ArtifactDigest, Effect, EffectPlan, FileContents, ModelContentDigest, ProcessInvocation,
    ProcessInvocations, ProjectPath, ProjectionFingerprint, ReviewEventReference,
};
use crate::core::event_runtime::{
    ProjectRuntimeLock, ensure_sqlite_event_store, execute_eventcore_command_for_exported_event,
    lock_project_runtime,
};
use crate::core::events::{
    EventDraft, ExportedEventType, export_event_file_contents,
    exported_events_projection_fingerprint, list_event_conflicts, list_stale_workflow_readiness,
    project_exported_events, reject_legacy_artifact_only_project, resolve_event_conflict,
    unresolved_event_conflicts_exist,
};
use crate::core::formal_graph::{
    FormalGraphError, FormalWorkflowGraph, FormalWorkflowGraphs, parse_lean_workflow_graph,
    parse_quint_workflow_graph,
};
use crate::core::formal_project_facts::{
    NewProjectAutomation, NewProjectBoardConnection, NewProjectBoardElement, NewProjectCommand,
    NewProjectDataFlow, NewProjectEvent, NewProjectExternalPayload, NewProjectOutcome,
    NewProjectReadModel, NewProjectScenario, NewProjectStream, NewProjectTranslation,
    NewProjectView, ProjectAutomation, ProjectAutomationDefinition, ProjectBoardConnection,
    ProjectBoardElement, ProjectCommand, ProjectCommandError, ProjectCommandInput, ProjectDataFlow,
    ProjectEvent, ProjectEventAttribute, ProjectExternalPayload, ProjectExternalPayloadField,
    ProjectOutcome, ProjectReadModel, ProjectReadModelDefinition, ProjectReadModelField,
    ProjectScenario, ProjectScenarioDefinition, ProjectStream, ProjectTranslation,
    ProjectTranslationDefinition, ProjectView, ProjectViewControl, ProjectViewDefinition,
    ProjectViewField, add_project_automation, add_project_board_connection,
    add_project_board_element, add_project_command, add_project_data_flow, add_project_event,
    add_project_external_payload, add_project_outcome, add_project_read_model,
    add_project_scenario, add_project_stream, add_project_translation, add_project_view,
    parse_lean_project_automation_definitions, parse_lean_project_automations,
    parse_lean_project_board_connections, parse_lean_project_board_elements,
    parse_lean_project_command_errors, parse_lean_project_command_inputs,
    parse_lean_project_commands, parse_lean_project_data_flows,
    parse_lean_project_event_attributes, parse_lean_project_events,
    parse_lean_project_external_payload_fields, parse_lean_project_external_payloads,
    parse_lean_project_outcomes, parse_lean_project_read_model_definitions,
    parse_lean_project_read_model_fields, parse_lean_project_read_models,
    parse_lean_project_scenario_definitions, parse_lean_project_scenarios,
    parse_lean_project_streams, parse_lean_project_translation_definitions,
    parse_lean_project_translations, parse_lean_project_view_controls,
    parse_lean_project_view_definitions, parse_lean_project_view_fields, parse_lean_project_views,
    parse_quint_project_automation_definitions, parse_quint_project_automations,
    parse_quint_project_board_connections, parse_quint_project_board_elements,
    parse_quint_project_command_errors, parse_quint_project_command_inputs,
    parse_quint_project_commands, parse_quint_project_data_flows,
    parse_quint_project_event_attributes, parse_quint_project_events,
    parse_quint_project_external_payload_fields, parse_quint_project_external_payloads,
    parse_quint_project_outcomes, parse_quint_project_read_model_definitions,
    parse_quint_project_read_model_fields, parse_quint_project_read_models,
    parse_quint_project_scenario_definitions, parse_quint_project_scenarios,
    parse_quint_project_streams, parse_quint_project_translation_definitions,
    parse_quint_project_translations, parse_quint_project_view_controls,
    parse_quint_project_view_definitions, parse_quint_project_view_fields,
    parse_quint_project_views,
};
use crate::core::formal_slice_facts::{
    add_automation_definition, add_bit_level_data_flow, add_board_connection, add_board_element,
    add_command_definition, add_event_definition, add_external_payload_definition,
    add_outcome_definition, add_read_model_definition, add_slice_scenario,
    add_translation_definition, add_view_definition,
};
use crate::core::formal_workflow_facts::{
    add_workflow_command_error, add_workflow_entry_lifecycle_state, add_workflow_outcome,
    add_workflow_owned_definition, add_workflow_transition_evidence,
    require_workflow_entry_lifecycle_coverage,
};
use crate::core::layout::{
    ModeledProjectRootInventories, ModeledProjectRootInventoryParts, ModeledWorkflowLayout,
    ModeledWorkflowLayouts, ModeledWorkflowSliceDetails, ModeledWorkflowTransitions, check_project,
    list_slices, list_transitions, list_workflows, show_document, show_workflow,
};
use crate::core::project::{ProjectName, ProjectSliceMembership, ProjectSliceMemberships};
use crate::core::review_record::{
    RequiredReviewCategories, ReviewCategoryFinding, ReviewRecordDocument, record_clean_review,
};
use crate::core::slice::{
    SliceProjectRootContext, add_slice, remove_slice, update_slice_description, update_slice_kind,
    update_slice_name,
};
use crate::core::types::{
    LeanModuleName, ReviewRuleName, ReviewTimestamp, ReviewerId, SliceSlug, WorkflowSliceDetail,
    WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionRecord,
};
use crate::core::verify::verify_project;
use crate::core::workflow::{
    IndexedWorkflowGraph, IndexedWorkflowGraphs, add_workflow, remove_workflow,
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

    pub(crate) fn project_path(error: impl Display) -> Self {
        Self {
            message: format!("invalid project path: {error}"),
        }
    }

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

pub(crate) fn interpret(plan: EffectPlan) -> Result<(), ShellError> {
    interpret_collect_reports_with_progress(plan, &mut |report| println!("{report}")).map(|_| ())
}

pub(crate) fn interpret_collect_reports(plan: EffectPlan) -> Result<Vec<String>, ShellError> {
    let runtime_lock = lock_project_runtime_if_needed()?;
    if runtime_lock.is_some() {
        if !project_initialization_plan(&plan) {
            reject_legacy_artifact_only_project().map_err(ShellError::message)?;
        }
        ensure_sqlite_event_store_if_needed()?;
        project_exported_events_into_worktree()?;
        reject_mutation_when_event_conflicts_exist(&plan)?;
    }
    plan.effects()
        .iter()
        .try_fold(Vec::new(), |mut reports, effect| {
            reports.extend(interpret_effect(effect)?);
            Ok(reports)
        })
}

fn interpret_collect_reports_with_progress(
    plan: EffectPlan,
    report: &mut impl FnMut(&str),
) -> Result<Vec<String>, ShellError> {
    let runtime_lock = lock_project_runtime_if_needed()?;
    if runtime_lock.is_some() {
        if !project_initialization_plan(&plan) {
            reject_legacy_artifact_only_project().map_err(ShellError::message)?;
        }
        ensure_sqlite_event_store_if_needed()?;
        project_exported_events_into_worktree()?;
        reject_mutation_when_event_conflicts_exist(&plan)?;
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
            reports.iter().for_each(|line| report(line));
            Ok(reports)
        }
        Effect::RunProcessBatch(invocations) => {
            let reports = run_process_batch_with_progress(invocations, report)?;
            Ok(reports)
        }
        Effect::VerifyProjectFromIndex => interpret_verify_project_from_index_with_progress(report),
        _ => {
            let reports = interpret_effect(effect)?;
            reports.iter().for_each(|line| report(line));
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
    let mut reports = interpret_collect_reports(verify_project(
        project_name,
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
        verify_project(
            project_name,
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
        readiness_reports.iter().for_each(|line| report(line));
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

fn ensure_sqlite_event_store_if_needed() -> Result<(), ShellError> {
    if !Path::new("model/events/v1").exists() {
        return Ok(());
    }
    ensure_sqlite_event_store(Path::new("."))
        .map(|_path| ())
        .map_err(ShellError::message)
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
            | Effect::RemoveSliceFromWorkflow(_)
            | Effect::RemoveTransitionFromWorkflow(_)
            | Effect::RemoveWorkflowFromIndex(_)
            | Effect::RequireWorkflowEntryLifecycleCoverageFromWorkflow(_)
            | Effect::UpdateSliceDescriptionFromWorkflow(_)
            | Effect::UpdateSliceKindFromWorkflow(_)
            | Effect::UpdateSliceNameFromWorkflow(_)
            | Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(_)
            | Effect::UpdateWorkflowNameFromIndexAndWorkflow(_)
    )
}

fn project_exported_events_into_worktree() -> Result<(), ShellError> {
    if !event_projection_is_needed() {
        return Ok(());
    }

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

fn event_projection_is_needed() -> bool {
    if !Path::new("emc.toml").exists()
        || !Path::new("model/lean").exists()
        || !Path::new("model/quint").exists()
        || !Path::new("reviews").exists()
    {
        return true;
    }

    let Ok(Some(current_fingerprint)) = exported_events_projection_fingerprint() else {
        return false;
    };
    fs::read_to_string("model/events/projection.fingerprint")
        .map(|fingerprint| fingerprint.trim() != current_fingerprint)
        .unwrap_or(true)
}

fn interpret_effect(effect: &Effect) -> Result<Vec<String>, ShellError> {
    match effect {
        Effect::AddAutomationDefinitionFromSlice(automation) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(automation.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) = find_formal_workflow_containing_slice_in(
                &formal_workflows,
                automation.slice_slug(),
            )?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_automation_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                automation.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_automation_plan = add_project_automation(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectAutomation::from_automation(workflow_layout.slug().clone(), automation),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_automation_plan)?);
            interpret_effect(&Effect::ExportEvent(EventDraft::slice_automation_added(
                automation,
            )))?;
            Ok(reports)
        }
        Effect::AddBitLevelDataFlowFromSlice(data_flow) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(data_flow.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) = find_formal_workflow_containing_slice_in(
                &formal_workflows,
                data_flow.slice_slug(),
            )?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_bit_level_data_flow(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                data_flow.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_data_flow_plan = add_project_data_flow(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectDataFlow::from_slice_data_flow(workflow_layout.slug().clone(), data_flow),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_data_flow_plan)?);
            interpret_effect(&Effect::ExportEvent(
                EventDraft::slice_bit_level_data_flow_added(data_flow),
            ))?;
            Ok(reports)
        }
        Effect::AddBoardConnectionFromSlice(connection) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(connection.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) = find_formal_workflow_containing_slice_in(
                &formal_workflows,
                connection.slice_slug(),
            )?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_board_connection(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                connection.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_board_plan = add_project_board_connection(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectBoardConnection::from_slice_board_connection(
                    workflow_layout.slug().clone(),
                    connection,
                ),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_board_plan)?);
            interpret_effect(&Effect::ExportEvent(
                EventDraft::slice_board_connection_added(connection),
            ))?;
            Ok(reports)
        }
        Effect::AddBoardElementFromSlice(element) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(element.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, element.slice_slug())?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_board_element(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                element.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_board_plan = add_project_board_element(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectBoardElement::from_slice_board_element(
                    workflow_layout.slug().clone(),
                    element,
                ),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_board_plan)?);
            interpret_effect(&Effect::ExportEvent(EventDraft::slice_board_element_added(
                element,
            )))?;
            Ok(reports)
        }
        Effect::AddCommandDefinitionFromSlice(command) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(command.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, command.slice_slug())?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_command_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                command.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_command_plan = add_project_command(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectCommand::from_command(workflow_layout.slug().clone(), command),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_command_plan)?);
            interpret_effect(&Effect::ExportEvent(
                EventDraft::slice_command_definition_added(command),
            ))?;
            Ok(reports)
        }
        Effect::AddEventDefinitionFromSlice(event) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(event.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, event.slice_slug())?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_event_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                event.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_stream_plan = add_project_stream(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectStream::new(
                    workflow_layout.slug().clone(),
                    event.slice_slug().clone(),
                    event.stream().clone(),
                ),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_stream_plan)?);
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let project_event_plan = add_project_event(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectEvent::from_event(workflow_layout.slug().clone(), event),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            reports.extend(interpret_collect_reports(project_event_plan)?);
            interpret_effect(&Effect::ExportEvent(
                EventDraft::slice_event_definition_added(event),
            ))?;
            Ok(reports)
        }
        Effect::AddExternalPayloadDefinitionFromSlice(external_payload) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(external_payload.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) = find_formal_workflow_containing_slice_in(
                &formal_workflows,
                external_payload.slice_slug(),
            )?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_external_payload_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                external_payload.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_external_payload_plan = add_project_external_payload(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectExternalPayload::from_external_payload(
                    workflow_layout.slug().clone(),
                    external_payload,
                ),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_external_payload_plan)?);
            interpret_effect(&Effect::ExportEvent(
                EventDraft::slice_external_payload_added(external_payload),
            ))?;
            Ok(reports)
        }
        Effect::AddOutcomeDefinitionFromSlice(outcome) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(outcome.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, outcome.slice_slug())?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_outcome_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                outcome.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_outcome_plan = add_project_outcome(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectOutcome::new(
                    workflow_layout.slug().clone(),
                    outcome.slice_slug().clone(),
                    outcome.label().clone(),
                    outcome.event_set().clone(),
                    outcome.externally_relevant(),
                ),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_outcome_plan)?);
            interpret_effect(&Effect::ExportEvent(EventDraft::slice_outcome_added(
                outcome,
            )))?;
            Ok(reports)
        }
        Effect::AddReadModelDefinitionFromSlice(read_model) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(read_model.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) = find_formal_workflow_containing_slice_in(
                &formal_workflows,
                read_model.slice_slug(),
            )?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_read_model_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                read_model.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_read_model_plan = add_project_read_model(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectReadModel::from_read_model(workflow_layout.slug().clone(), read_model),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_read_model_plan)?);
            interpret_effect(&Effect::ExportEvent(EventDraft::slice_read_model_added(
                read_model,
            )))?;
            Ok(reports)
        }
        Effect::AddViewDefinitionFromSlice(view) => {
            let slice_artifacts = read_formal_slice_artifact_paths_and_contents(view.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, view.slice_slug())?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_view_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                view.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_view_plan = add_project_view(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectView::from_view(workflow_layout.slug().clone(), view),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_view_plan)?);
            interpret_effect(&Effect::ExportEvent(EventDraft::slice_view_added(view)))?;
            Ok(reports)
        }
        Effect::AddSliceFromWorkflow(slice) => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, workflow_graph) =
                formal_workflow_layout_and_graph(&formal_workflows, slice.workflow_slug())?;
            let plan = add_slice(
                project_name,
                &formal_workflows,
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_graph,
                slice.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddSliceScenarioFromSlice(scenario) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(scenario.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, scenario.slice_slug())?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_slice_scenario(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                scenario.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_scenario_plan = add_project_scenario(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectScenario::from_slice_scenario(workflow_layout.slug().clone(), scenario),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_scenario_plan)?);
            interpret_effect(&Effect::ExportEvent(EventDraft::slice_scenario_added(
                scenario,
            )))?;
            Ok(reports)
        }
        Effect::AddTranslationDefinitionFromSlice(translation) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(translation.slice_slug())?;
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, _workflow_graph) = find_formal_workflow_containing_slice_in(
                &formal_workflows,
                translation.slice_slug(),
            )?;
            let project_artifacts = read_project_root_artifact_paths_and_contents(&project_name)?;
            let slice_plan = add_translation_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                translation.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let project_translation_plan = add_project_translation(
                project_artifacts.lean_path,
                project_artifacts.lean_contents,
                project_artifacts.quint_path,
                project_artifacts.quint_contents,
                NewProjectTranslation::from_translation(
                    workflow_layout.slug().clone(),
                    translation,
                ),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let mut reports = interpret_collect_reports(slice_plan)?;
            reports.extend(interpret_collect_reports(project_translation_plan)?);
            interpret_effect(&Effect::ExportEvent(EventDraft::slice_translation_added(
                translation,
            )))?;
            Ok(reports)
        }
        Effect::AddWorkflowFromIndex(workflow) => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?;
            let existing_slice_memberships = formal_project_slice_memberships(&formal_workflows);
            let existing_workflows = formal_workflow_layouts(formal_workflows);
            let plan = add_workflow(
                project_name,
                ModeledWorkflowLayouts::new(existing_workflows),
                existing_slice_memberships,
                workflow.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowCommandErrorFromWorkflow(effect) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(effect.workflow_slug())?;
            let plan = add_workflow_command_error(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                effect.workflow_slug().clone(),
                effect.error().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(
                EventDraft::workflow_command_error_added(effect.workflow_slug(), effect.error()),
            ))?;
            Ok(reports)
        }
        Effect::AddWorkflowOwnedDefinitionFromWorkflow(effect) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(effect.workflow_slug())?;
            let plan = add_workflow_owned_definition(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                effect.workflow_slug().clone(),
                effect.definition().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(
                EventDraft::workflow_owned_definition_added(
                    effect.workflow_slug(),
                    effect.definition(),
                ),
            ))?;
            Ok(reports)
        }
        Effect::AddWorkflowOutcomeFromWorkflow(effect) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(effect.workflow_slug())?;
            let plan = add_workflow_outcome(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                effect.workflow_slug().clone(),
                effect.outcome().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(EventDraft::workflow_outcome_added(
                effect.workflow_slug(),
                effect.outcome(),
            )))?;
            Ok(reports)
        }
        Effect::AddWorkflowTransitionEvidenceFromWorkflow(effect) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(effect.workflow_slug())?;
            let plan = add_workflow_transition_evidence(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                effect.workflow_slug().clone(),
                effect.evidence().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(
                EventDraft::workflow_transition_evidence_added(
                    effect.workflow_slug(),
                    effect.evidence(),
                ),
            ))?;
            Ok(reports)
        }
        Effect::RequireWorkflowEntryLifecycleCoverageFromWorkflow(workflow_slug) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(workflow_slug)?;
            let plan = require_workflow_entry_lifecycle_coverage(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                workflow_slug.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(
                EventDraft::workflow_entry_lifecycle_coverage_required(workflow_slug),
            ))?;
            Ok(reports)
        }
        Effect::AddWorkflowEntryLifecycleStateFromWorkflow(effect) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(effect.workflow_slug())?;
            let plan = add_workflow_entry_lifecycle_state(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                effect.workflow_slug().clone(),
                effect.coverage().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(
                EventDraft::workflow_entry_lifecycle_state_added(
                    effect.workflow_slug(),
                    effect.coverage(),
                ),
            ))?;
            Ok(reports)
        }
        Effect::CheckCurrentProject => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?;
            let project_scenarios = read_synchronized_project_scenarios(&project_name)?;
            let project_scenario_definitions =
                read_synchronized_project_scenario_definitions(&project_name)?;
            let project_data_flows = read_synchronized_project_data_flows(&project_name)?;
            let project_outcomes = read_synchronized_project_outcomes(&project_name)?;
            let project_command_errors = read_synchronized_project_command_errors(&project_name)?;
            let project_commands = read_synchronized_project_commands(&project_name)?;
            let project_command_inputs = read_synchronized_project_command_inputs(&project_name)?;
            let project_read_models = read_synchronized_project_read_models(&project_name)?;
            let project_read_model_definitions =
                read_synchronized_project_read_model_definitions(&project_name)?;
            let project_read_model_fields =
                read_synchronized_project_read_model_fields(&project_name)?;
            let project_views = read_synchronized_project_views(&project_name)?;
            let project_view_definitions =
                read_synchronized_project_view_definitions(&project_name)?;
            let project_view_controls = read_synchronized_project_view_controls(&project_name)?;
            let project_board_elements = read_synchronized_project_board_elements(&project_name)?;
            let project_board_connections =
                read_synchronized_project_board_connections(&project_name)?;
            let project_view_fields = read_synchronized_project_view_fields(&project_name)?;
            let project_automations = read_synchronized_project_automations(&project_name)?;
            let project_automation_definitions =
                read_synchronized_project_automation_definitions(&project_name)?;
            let project_translations = read_synchronized_project_translations(&project_name)?;
            let project_translation_definitions =
                read_synchronized_project_translation_definitions(&project_name)?;
            let project_external_payloads =
                read_synchronized_project_external_payloads(&project_name)?;
            let project_external_payload_fields =
                read_synchronized_project_external_payload_fields(&project_name)?;
            let project_streams = read_synchronized_project_streams(&project_name)?;
            let project_events = read_synchronized_project_events(&project_name)?;
            let project_event_attributes =
                read_synchronized_project_event_attributes(&project_name)?;
            interpret_collect_reports(check_project(
                project_name,
                formal_workflows,
                ModeledProjectRootInventories::from_parts(ModeledProjectRootInventoryParts {
                    scenarios: project_scenarios,
                    scenario_definitions: project_scenario_definitions,
                    data_flows: project_data_flows,
                    outcomes: project_outcomes,
                    command_errors: project_command_errors,
                    commands: project_commands,
                    command_inputs: project_command_inputs,
                    read_models: project_read_models,
                    read_model_definitions: project_read_model_definitions,
                    read_model_fields: project_read_model_fields,
                    views: project_views,
                    view_definitions: project_view_definitions,
                    view_controls: project_view_controls,
                    board_elements: project_board_elements,
                    board_connections: project_board_connections,
                    view_fields: project_view_fields,
                    automations: project_automations,
                    automation_definitions: project_automation_definitions,
                    translations: project_translations,
                    translation_definitions: project_translation_definitions,
                    external_payloads: project_external_payloads,
                    external_payload_fields: project_external_payload_fields,
                    streams: project_streams,
                    events: project_events,
                    event_attributes: project_event_attributes,
                }),
            ))
        }
        Effect::ConnectWorkflowFromWorkflow(connection) => {
            let (workflow_layout, workflow_graph) =
                read_formal_workflow_layout_and_graph(connection.workflow_slug())?;
            let plan = connect_workflow(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_graph,
                connection.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(EventDraft::workflow_connected(
                connection,
            )))?;
            Ok(reports)
        }
        Effect::EnsureDirectory(path) => fs::create_dir_all(Path::new(path.as_ref()))
            .map(|()| Vec::new())
            .map_err(ShellError::io),
        Effect::ExportEvent(draft) => {
            if exported_events_are_projecting()? {
                return Ok(Vec::new());
            }
            execute_eventcore_command_for_exported_event(Path::new("."), draft)
                .map_err(ShellError::message)?;
            let (path, contents) =
                export_event_file_contents(draft).map_err(ShellError::message)?;
            write_file(&path, contents.as_ref()).map(|()| Vec::new())
        }
        Effect::ListConflictsFromEvents => {
            interpret_collect_reports(list_event_conflicts().map_err(ShellError::message)?)
        }
        Effect::ListWorkflowsFromIndex => {
            let modeled_workflows =
                formal_workflow_layouts(read_synchronized_formal_workflow_graphs()?);
            let mut reports = interpret_collect_reports(list_workflows(
                ModeledWorkflowLayouts::new(modeled_workflows),
            ))?;
            reports.extend(interpret_collect_reports(
                list_stale_workflow_readiness().map_err(ShellError::message)?,
            )?);
            Ok(reports)
        }
        Effect::ListSlicesFromIndex => {
            let modeled_slices =
                formal_workflow_slice_details(read_synchronized_formal_workflow_graphs()?);
            interpret_collect_reports(list_slices(ModeledWorkflowSliceDetails::new(
                modeled_slices,
            )))
        }
        Effect::ListTransitionsFromIndex => {
            let modeled_transitions =
                formal_workflow_transitions(read_synchronized_formal_workflow_graphs()?);
            interpret_collect_reports(list_transitions(ModeledWorkflowTransitions::new(
                modeled_transitions,
            )))
        }
        Effect::RequireCanonicalDeclaration(requirement) => {
            let contents = fs::read_to_string(Path::new(requirement.path().as_ref()))
                .map_err(ShellError::io)?;
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
        Effect::RequireDigest(requirement) => {
            let contents = fs::read_to_string(Path::new(requirement.path().as_ref()))
                .map_err(ShellError::io)?;
            if artifact_contains_one_digest_marker(&contents, requirement.digest().as_ref()) {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(
                    requirement.message().as_ref().to_owned(),
                ))
            }
        }
        Effect::RequireFile(path) => {
            if Path::new(path.as_ref()).is_file() {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(format!(
                    "missing required project artifact {}",
                    path.as_ref()
                )))
            }
        }
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
        Effect::RequireReviewRecord(requirement) => {
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
        Effect::RecordCleanReviewFromWorkflow(effect) => {
            let current_digest = formal_model_content_digest(effect.workflow_slug())?;
            let required_categories = required_review_categories()?;
            let plan = record_clean_review(
                effect.workflow_slug().clone(),
                current_digest.clone(),
                effect.reviewer_id().clone(),
                effect.reviewed_at().clone(),
                RequiredReviewCategories::new(required_categories.clone()),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(EventDraft::review_recorded(
                effect.workflow_slug(),
                &current_digest,
                effect.reviewer_id(),
                effect.reviewed_at(),
                &required_categories,
            )))?;
            Ok(reports)
        }
        Effect::RemoveFile(path) => remove_file_if_present(path.as_ref()).map(|()| Vec::new()),
        Effect::RemoveSliceFromWorkflow(slug) => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, slug)?;
            let plan = remove_slice(
                project_name,
                &formal_workflows,
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                slug.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::RemoveTransitionFromWorkflow(removal) => {
            let (workflow_layout, workflow_graph) =
                read_formal_workflow_layout_and_graph(removal.workflow_slug())?;
            let plan = remove_transition(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_graph,
                removal.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            let reports = interpret_collect_reports(plan)?;
            interpret_effect(&Effect::ExportEvent(
                EventDraft::workflow_transition_removed(removal),
            ))?;
            Ok(reports)
        }
        Effect::RemoveWorkflowFromIndex(slug) => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let existing_workflows = formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(
                formal_workflows.clone(),
            ));
            let workflow_graphs = indexed_formal_workflow_graphs(&formal_workflows);
            let plan = remove_workflow(
                project_name,
                ModeledWorkflowLayouts::new(existing_workflows),
                IndexedWorkflowGraphs::new(workflow_graphs),
                slug.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::ResolveEventConflict(resolution) => {
            let plan = resolve_event_conflict(
                resolution.conflict_id().clone(),
                resolution.chosen_event_id().clone(),
            )
            .map_err(ShellError::message)?;
            plan.effects()
                .iter()
                .try_fold(Vec::new(), |mut reports, effect| {
                    reports.extend(interpret_effect(effect)?);
                    Ok(reports)
                })
        }
        Effect::ShowSliceFromSlice(slug) => {
            let slice_document = read_formal_slice_artifacts(slug)?;
            interpret_collect_reports(show_document(slice_document))
        }
        Effect::ShowWorkflowFromWorkflow(slug) => {
            let workflow_document = read_formal_workflow_artifacts(slug)?;
            interpret_collect_reports(show_workflow(workflow_document))
        }
        Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(effect) => {
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let existing_workflows = formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(
                formal_workflows.clone(),
            ));
            let workflow_graph = read_formal_workflow_graph(effect.workflow_slug())?;
            let plan = update_workflow_description(
                ModeledWorkflowLayouts::new(existing_workflows),
                workflow_graph,
                effect.workflow_slug().clone(),
                effect.description().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateWorkflowNameFromIndexAndWorkflow(effect) => {
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let existing_workflows = formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(
                formal_workflows.clone(),
            ));
            let workflow_graph = read_formal_workflow_graph(effect.workflow_slug())?;
            let plan = update_workflow_name(
                ModeledWorkflowLayouts::new(existing_workflows),
                workflow_graph,
                effect.workflow_slug().clone(),
                effect.name().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateSliceDescriptionFromWorkflow(effect) => {
            let (workflow_layout, workflow_graph) =
                find_formal_workflow_containing_slice(effect.slice_slug())?;
            let plan = update_slice_description(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                effect.slice_slug().clone(),
                effect.description().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateSliceKindFromWorkflow(effect) => {
            let (workflow_layout, workflow_graph) =
                find_formal_workflow_containing_slice(effect.slice_slug())?;
            let plan = update_slice_kind(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                effect.slice_slug().clone(),
                effect.kind(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateSliceNameFromWorkflow(effect) => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let (workflow_layout, workflow_graph) =
                find_formal_workflow_containing_slice_in(&formal_workflows, effect.slice_slug())?;
            let plan = update_slice_name(
                SliceProjectRootContext::new(project_name, &formal_workflows),
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                effect.slice_slug().clone(),
                effect.name().clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::VerifyProjectFromIndex => interpret_verify_project_from_index_collecting(),
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
        Effect::WriteFileIfMissing(write) => {
            if Path::new(write.path().as_ref()).exists() {
                Ok(Vec::new())
            } else {
                write_file(write.path().as_ref(), write.contents().as_ref()).map(|()| Vec::new())
            }
        }
        Effect::Report(line) => Ok(vec![line.as_ref().to_owned()]),
        Effect::ReportDocument(contents) => Ok(vec![contents.as_ref().to_owned()]),
    }
}

fn read_project_manifest_name() -> Result<ProjectName, ShellError> {
    fs::read_to_string("emc.toml")
        .map_err(ShellError::io)
        .and_then(|manifest| {
            parse_project_manifest_name(&manifest).map_err(ShellError::project_name)
        })
}

fn read_synchronized_formal_workflow_graphs() -> Result<FormalWorkflowGraphs, ShellError> {
    let lean_graphs = read_formal_workflow_graphs(
        Path::new("model/lean"),
        ".lean",
        "def workflowName :=",
        parse_lean_workflow_graph,
    )?;
    let quint_graphs = read_formal_workflow_graphs(
        Path::new("model/quint"),
        ".qnt",
        "val workflowName =",
        parse_quint_workflow_graph,
    )?;

    let quint_by_slug = formal_graphs_by_slug(quint_graphs, "Quint")?;
    let mut matched_slugs = BTreeSet::new();
    let synchronized_graphs = lean_graphs
        .into_iter()
        .map(|lean_graph| {
            let quint_graph = quint_by_slug
                .get(lean_graph.slug().as_ref())
                .ok_or_else(|| {
                    ShellError::message(format!(
                        "Quint workflow artifact is missing for workflow {}",
                        lean_graph.slug().as_ref()
                    ))
                })?;
            if &lean_graph == quint_graph {
                Ok(lean_graph)
            } else {
                Err(ShellError::message(format!(
                    "Lean and Quint workflow artifacts disagree for workflow {}",
                    lean_graph.slug().as_ref()
                )))
            }
        })
        .inspect(|result| {
            if let Ok(graph) = result {
                matched_slugs.insert(graph.slug().as_ref().to_owned());
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(unmatched_slug) = quint_by_slug
        .keys()
        .find(|slug| !matched_slugs.contains(*slug))
    {
        Err(ShellError::message(format!(
            "Lean workflow artifact is missing for workflow {unmatched_slug}"
        )))
    } else {
        Ok(FormalWorkflowGraphs::from_graphs(synchronized_graphs))
    }
}

fn read_synchronized_project_streams(
    project_name: &ProjectName,
) -> Result<Vec<ProjectStream>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_streams(&artifacts.lean_contents),
        parse_quint_project_streams(&artifacts.quint_contents),
    ) {
        (Ok(lean_streams), Ok(quint_streams)) if lean_streams == quint_streams => Ok(lean_streams),
        (Ok(_lean_streams), Ok(_quint_streams)) => Err(ShellError::message(
            "Lean and Quint project root stream inventories disagree",
        )),
        (_lean_streams, _quint_streams) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_commands(
    project_name: &ProjectName,
) -> Result<Vec<ProjectCommand>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_commands(&artifacts.lean_contents),
        parse_quint_project_commands(&artifacts.quint_contents),
    ) {
        (Ok(lean_commands), Ok(quint_commands)) if lean_commands == quint_commands => {
            Ok(lean_commands)
        }
        (Ok(_lean_commands), Ok(_quint_commands)) => Err(ShellError::message(
            "Lean and Quint project root command inventories disagree",
        )),
        (_lean_commands, _quint_commands) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_command_inputs(
    project_name: &ProjectName,
) -> Result<Vec<ProjectCommandInput>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_command_inputs(&artifacts.lean_contents),
        parse_quint_project_command_inputs(&artifacts.quint_contents),
    ) {
        (Ok(lean_command_inputs), Ok(quint_command_inputs))
            if lean_command_inputs == quint_command_inputs =>
        {
            Ok(lean_command_inputs)
        }
        (Ok(_lean_command_inputs), Ok(_quint_command_inputs)) => Err(ShellError::message(
            "Lean and Quint project root command input inventories disagree",
        )),
        (_lean_command_inputs, _quint_command_inputs) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_command_errors(
    project_name: &ProjectName,
) -> Result<Vec<ProjectCommandError>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_command_errors(&artifacts.lean_contents),
        parse_quint_project_command_errors(&artifacts.quint_contents),
    ) {
        (Ok(lean_command_errors), Ok(quint_command_errors))
            if lean_command_errors == quint_command_errors =>
        {
            Ok(lean_command_errors)
        }
        (Ok(_lean_command_errors), Ok(_quint_command_errors)) => Err(ShellError::message(
            "Lean and Quint project root command error inventories disagree",
        )),
        (_lean_command_errors, _quint_command_errors) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_scenarios(
    project_name: &ProjectName,
) -> Result<Vec<ProjectScenario>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_scenarios(&artifacts.lean_contents),
        parse_quint_project_scenarios(&artifacts.quint_contents),
    ) {
        (Ok(lean_scenarios), Ok(quint_scenarios)) if lean_scenarios == quint_scenarios => {
            Ok(lean_scenarios)
        }
        (Ok(_lean_scenarios), Ok(_quint_scenarios)) => Err(ShellError::message(
            "Lean and Quint project root scenario inventories disagree",
        )),
        (_lean_scenarios, _quint_scenarios) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_scenario_definitions(
    project_name: &ProjectName,
) -> Result<Vec<ProjectScenarioDefinition>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_scenario_definitions(&artifacts.lean_contents),
        parse_quint_project_scenario_definitions(&artifacts.quint_contents),
    ) {
        (Ok(lean_scenarios), Ok(quint_scenarios)) if lean_scenarios == quint_scenarios => {
            Ok(lean_scenarios)
        }
        (Ok(_lean_scenarios), Ok(_quint_scenarios)) => Err(ShellError::message(
            "Lean and Quint project root scenario definition inventories disagree",
        )),
        (_lean_scenarios, _quint_scenarios) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_data_flows(
    project_name: &ProjectName,
) -> Result<Vec<ProjectDataFlow>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_data_flows(&artifacts.lean_contents),
        parse_quint_project_data_flows(&artifacts.quint_contents),
    ) {
        (Ok(lean_data_flows), Ok(quint_data_flows)) if lean_data_flows == quint_data_flows => {
            Ok(lean_data_flows)
        }
        (Ok(_lean_data_flows), Ok(_quint_data_flows)) => Err(ShellError::message(
            "Lean and Quint project root data-flow inventories disagree",
        )),
        (_lean_data_flows, _quint_data_flows) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_outcomes(
    project_name: &ProjectName,
) -> Result<Vec<ProjectOutcome>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_outcomes(&artifacts.lean_contents),
        parse_quint_project_outcomes(&artifacts.quint_contents),
    ) {
        (Ok(lean_outcomes), Ok(quint_outcomes)) if lean_outcomes == quint_outcomes => {
            Ok(lean_outcomes)
        }
        (Ok(_lean_outcomes), Ok(_quint_outcomes)) => Err(ShellError::message(
            "Lean and Quint project root outcome inventories disagree",
        )),
        (_lean_outcomes, _quint_outcomes) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_read_models(
    project_name: &ProjectName,
) -> Result<Vec<ProjectReadModel>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_read_models(&artifacts.lean_contents),
        parse_quint_project_read_models(&artifacts.quint_contents),
    ) {
        (Ok(lean_read_models), Ok(quint_read_models)) if lean_read_models == quint_read_models => {
            Ok(lean_read_models)
        }
        (Ok(_lean_read_models), Ok(_quint_read_models)) => Err(ShellError::message(
            "Lean and Quint project root read model inventories disagree",
        )),
        (_lean_read_models, _quint_read_models) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_read_model_definitions(
    project_name: &ProjectName,
) -> Result<Vec<ProjectReadModelDefinition>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_read_model_definitions(&artifacts.lean_contents),
        parse_quint_project_read_model_definitions(&artifacts.quint_contents),
    ) {
        (Ok(lean_read_model_definitions), Ok(quint_read_model_definitions))
            if lean_read_model_definitions == quint_read_model_definitions =>
        {
            Ok(lean_read_model_definitions)
        }
        (Ok(_lean_read_model_definitions), Ok(_quint_read_model_definitions)) => {
            Err(ShellError::message(
                "Lean and Quint project root read model definition inventories disagree",
            ))
        }
        (_lean_read_model_definitions, _quint_read_model_definitions) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_read_model_fields(
    project_name: &ProjectName,
) -> Result<Vec<ProjectReadModelField>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_read_model_fields(&artifacts.lean_contents),
        parse_quint_project_read_model_fields(&artifacts.quint_contents),
    ) {
        (Ok(lean_read_model_fields), Ok(quint_read_model_fields))
            if lean_read_model_fields == quint_read_model_fields =>
        {
            Ok(lean_read_model_fields)
        }
        (Ok(_lean_read_model_fields), Ok(_quint_read_model_fields)) => Err(ShellError::message(
            "Lean and Quint project root read model field inventories disagree",
        )),
        (_lean_read_model_fields, _quint_read_model_fields) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_views(
    project_name: &ProjectName,
) -> Result<Vec<ProjectView>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_views(&artifacts.lean_contents),
        parse_quint_project_views(&artifacts.quint_contents),
    ) {
        (Ok(lean_views), Ok(quint_views)) if lean_views == quint_views => Ok(lean_views),
        (Ok(_lean_views), Ok(_quint_views)) => Err(ShellError::message(
            "Lean and Quint project root view inventories disagree",
        )),
        (_lean_views, _quint_views) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_view_definitions(
    project_name: &ProjectName,
) -> Result<Vec<ProjectViewDefinition>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_view_definitions(&artifacts.lean_contents),
        parse_quint_project_view_definitions(&artifacts.quint_contents),
    ) {
        (Ok(lean_view_definitions), Ok(quint_view_definitions))
            if lean_view_definitions == quint_view_definitions =>
        {
            Ok(lean_view_definitions)
        }
        (Ok(_lean_view_definitions), Ok(_quint_view_definitions)) => Err(ShellError::message(
            "Lean and Quint project root view definition inventories disagree",
        )),
        (_lean_view_definitions, _quint_view_definitions) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_view_controls(
    project_name: &ProjectName,
) -> Result<Vec<ProjectViewControl>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_view_controls(&artifacts.lean_contents),
        parse_quint_project_view_controls(&artifacts.quint_contents),
    ) {
        (Ok(lean_view_controls), Ok(quint_view_controls))
            if lean_view_controls == quint_view_controls =>
        {
            Ok(lean_view_controls)
        }
        (Ok(_lean_view_controls), Ok(_quint_view_controls)) => Err(ShellError::message(
            "Lean and Quint project root view control inventories disagree",
        )),
        (_lean_view_controls, _quint_view_controls) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_board_elements(
    project_name: &ProjectName,
) -> Result<Vec<ProjectBoardElement>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_board_elements(&artifacts.lean_contents),
        parse_quint_project_board_elements(&artifacts.quint_contents),
    ) {
        (Ok(lean_board_elements), Ok(quint_board_elements))
            if lean_board_elements == quint_board_elements =>
        {
            Ok(lean_board_elements)
        }
        (Ok(_lean_board_elements), Ok(_quint_board_elements)) => Err(ShellError::message(
            "Lean and Quint project root board element inventories disagree",
        )),
        (_lean_board_elements, _quint_board_elements) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_board_connections(
    project_name: &ProjectName,
) -> Result<Vec<ProjectBoardConnection>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_board_connections(&artifacts.lean_contents),
        parse_quint_project_board_connections(&artifacts.quint_contents),
    ) {
        (Ok(lean_board_connections), Ok(quint_board_connections))
            if lean_board_connections == quint_board_connections =>
        {
            Ok(lean_board_connections)
        }
        (Ok(_lean_board_connections), Ok(_quint_board_connections)) => Err(ShellError::message(
            "Lean and Quint project root board connection inventories disagree",
        )),
        (_lean_board_connections, _quint_board_connections) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_view_fields(
    project_name: &ProjectName,
) -> Result<Vec<ProjectViewField>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_view_fields(&artifacts.lean_contents),
        parse_quint_project_view_fields(&artifacts.quint_contents),
    ) {
        (Ok(lean_view_fields), Ok(quint_view_fields)) if lean_view_fields == quint_view_fields => {
            Ok(lean_view_fields)
        }
        (Ok(_lean_view_fields), Ok(_quint_view_fields)) => Err(ShellError::message(
            "Lean and Quint project root view field inventories disagree",
        )),
        (_lean_view_fields, _quint_view_fields) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_automations(
    project_name: &ProjectName,
) -> Result<Vec<ProjectAutomation>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_automations(&artifacts.lean_contents),
        parse_quint_project_automations(&artifacts.quint_contents),
    ) {
        (Ok(lean_automations), Ok(quint_automations)) if lean_automations == quint_automations => {
            Ok(lean_automations)
        }
        (Ok(_lean_automations), Ok(_quint_automations)) => Err(ShellError::message(
            "Lean and Quint project root automation inventories disagree",
        )),
        (_lean_automations, _quint_automations) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_automation_definitions(
    project_name: &ProjectName,
) -> Result<Vec<ProjectAutomationDefinition>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_automation_definitions(&artifacts.lean_contents),
        parse_quint_project_automation_definitions(&artifacts.quint_contents),
    ) {
        (Ok(lean_definitions), Ok(quint_definitions)) if lean_definitions == quint_definitions => {
            Ok(lean_definitions)
        }
        (Ok(_lean_definitions), Ok(_quint_definitions)) => Err(ShellError::message(
            "Lean and Quint project root automation definition inventories disagree",
        )),
        (_lean_definitions, _quint_definitions) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_translations(
    project_name: &ProjectName,
) -> Result<Vec<ProjectTranslation>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_translations(&artifacts.lean_contents),
        parse_quint_project_translations(&artifacts.quint_contents),
    ) {
        (Ok(lean_translations), Ok(quint_translations))
            if lean_translations == quint_translations =>
        {
            Ok(lean_translations)
        }
        (Ok(_lean_translations), Ok(_quint_translations)) => Err(ShellError::message(
            "Lean and Quint project root translation inventories disagree",
        )),
        (_lean_translations, _quint_translations) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_translation_definitions(
    project_name: &ProjectName,
) -> Result<Vec<ProjectTranslationDefinition>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_translation_definitions(&artifacts.lean_contents),
        parse_quint_project_translation_definitions(&artifacts.quint_contents),
    ) {
        (Ok(lean_definitions), Ok(quint_definitions)) if lean_definitions == quint_definitions => {
            Ok(lean_definitions)
        }
        (Ok(_lean_definitions), Ok(_quint_definitions)) => Err(ShellError::message(
            "Lean and Quint project root translation definition inventories disagree",
        )),
        (_lean_definitions, _quint_definitions) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_external_payloads(
    project_name: &ProjectName,
) -> Result<Vec<ProjectExternalPayload>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_external_payloads(&artifacts.lean_contents),
        parse_quint_project_external_payloads(&artifacts.quint_contents),
    ) {
        (Ok(lean_external_payloads), Ok(quint_external_payloads))
            if lean_external_payloads == quint_external_payloads =>
        {
            Ok(lean_external_payloads)
        }
        (Ok(_lean_external_payloads), Ok(_quint_external_payloads)) => Err(ShellError::message(
            "Lean and Quint project root external payload inventories disagree",
        )),
        (_lean_external_payloads, _quint_external_payloads) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_external_payload_fields(
    project_name: &ProjectName,
) -> Result<Vec<ProjectExternalPayloadField>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_external_payload_fields(&artifacts.lean_contents),
        parse_quint_project_external_payload_fields(&artifacts.quint_contents),
    ) {
        (Ok(lean_fields), Ok(quint_fields)) if lean_fields == quint_fields => Ok(lean_fields),
        (Ok(_lean_fields), Ok(_quint_fields)) => Err(ShellError::message(
            "Lean and Quint project root external payload field inventories disagree",
        )),
        (_lean_fields, _quint_fields) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_events(
    project_name: &ProjectName,
) -> Result<Vec<ProjectEvent>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_events(&artifacts.lean_contents),
        parse_quint_project_events(&artifacts.quint_contents),
    ) {
        (Ok(lean_events), Ok(quint_events)) if lean_events == quint_events => Ok(lean_events),
        (Ok(_lean_events), Ok(_quint_events)) => Err(ShellError::message(
            "Lean and Quint project root event inventories disagree",
        )),
        (_lean_events, _quint_events) => Ok(Vec::new()),
    }
}

fn read_synchronized_project_event_attributes(
    project_name: &ProjectName,
) -> Result<Vec<ProjectEventAttribute>, ShellError> {
    let Ok(artifacts) = read_project_root_artifact_paths_and_contents(project_name) else {
        return Ok(Vec::new());
    };
    match (
        parse_lean_project_event_attributes(&artifacts.lean_contents),
        parse_quint_project_event_attributes(&artifacts.quint_contents),
    ) {
        (Ok(lean_event_attributes), Ok(quint_event_attributes))
            if lean_event_attributes == quint_event_attributes =>
        {
            Ok(lean_event_attributes)
        }
        (Ok(_lean_event_attributes), Ok(_quint_event_attributes)) => Err(ShellError::message(
            "Lean and Quint project root event attribute inventories disagree",
        )),
        (_lean_event_attributes, _quint_event_attributes) => Ok(Vec::new()),
    }
}

fn read_formal_workflow_graphs(
    directory: &Path,
    extension: &str,
    workflow_marker: &str,
    parser: fn(&FileContents) -> Result<FormalWorkflowGraph, FormalGraphError>,
) -> Result<Vec<FormalWorkflowGraph>, ShellError> {
    let mut paths = fs::read_dir(directory)
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    paths.sort();

    paths
        .into_iter()
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == extension.strip_prefix('.')
        })
        .map(|path| {
            fs::read_to_string(&path)
                .map_err(ShellError::io)
                .and_then(|contents| {
                    let file_contents = FileContents::try_new(contents)
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    Ok((path, file_contents))
                })
        })
        .filter_map(|result| match result {
            Ok((path, contents)) if contents.as_ref().contains(workflow_marker) => {
                Some(Ok((path, contents)))
            }
            Ok((_path, _contents)) => None,
            Err(error) => Some(Err(error)),
        })
        .map(|result| {
            let (path, contents) = result?;
            parser(&contents).map_err(|error| {
                ShellError::message(format!(
                    "failed to parse formal workflow artifact {}: {error}",
                    path.display()
                ))
            })
        })
        .collect()
}

fn formal_graphs_by_slug(
    graphs: Vec<FormalWorkflowGraph>,
    artifact_family: &str,
) -> Result<BTreeMap<String, FormalWorkflowGraph>, ShellError> {
    graphs
        .into_iter()
        .try_fold(BTreeMap::new(), |mut indexed, graph| {
            let slug = graph.slug().as_ref().to_owned();
            if indexed.insert(slug.clone(), graph).is_none() {
                Ok(indexed)
            } else {
                Err(ShellError::message(format!(
                    "{artifact_family} workflow artifact slug {slug} is duplicated"
                )))
            }
        })
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
        .map(|module_name| {
            formal_artifact_bundle(&[
                format!("model/lean/slices/{module_name}.lean"),
                format!("model/quint/slices/{module_name}.qnt"),
            ])
        })
        .unwrap_or_else(|| {
            Err(ShellError::message(format!(
                "slice {} is not referenced by any modeled workflow",
                slug.as_ref()
            )))
        })
}

struct FormalSliceArtifactDocuments {
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
}

struct FormalWorkflowArtifactDocuments {
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
}

struct FormalProjectRootArtifactDocuments {
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
}

fn read_project_root_artifact_paths_and_contents(
    project_name: &ProjectName,
) -> Result<FormalProjectRootArtifactDocuments, ShellError> {
    let module_name = module_name_from_raw(project_name.as_ref());
    let lean_path = format!("model/lean/{module_name}.lean");
    let quint_path = format!("model/quint/{module_name}.qnt");
    Ok(FormalProjectRootArtifactDocuments {
        lean_path: project_path(lean_path.clone())?,
        lean_contents: read_file_contents(&lean_path)?,
        quint_path: project_path(quint_path.clone())?,
        quint_contents: read_file_contents(&quint_path)?,
    })
}

fn read_formal_workflow_artifact_paths_and_contents(
    slug: &WorkflowSlug,
) -> Result<FormalWorkflowArtifactDocuments, ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    let module_name = module_name_from_raw(graph.name().as_ref());
    let lean_path = format!("model/lean/{module_name}.lean");
    let quint_path = format!("model/quint/{module_name}.qnt");
    Ok(FormalWorkflowArtifactDocuments {
        lean_path: project_path(lean_path.clone())?,
        lean_contents: read_file_contents(&lean_path)?,
        quint_path: project_path(quint_path.clone())?,
        quint_contents: read_file_contents(&quint_path)?,
    })
}

fn read_formal_slice_artifact_paths_and_contents(
    slug: &SliceSlug,
) -> Result<FormalSliceArtifactDocuments, ShellError> {
    let module_name = find_formal_slice_module_name(slug)?;
    let lean_path = format!("model/lean/slices/{module_name}.lean");
    let quint_path = format!("model/quint/slices/{module_name}.qnt");
    Ok(FormalSliceArtifactDocuments {
        lean_path: project_path(lean_path.clone())?,
        lean_contents: read_file_contents(&lean_path)?,
        quint_path: project_path(quint_path.clone())?,
        quint_contents: read_file_contents(&quint_path)?,
    })
}

fn find_formal_slice_module_name(slug: &SliceSlug) -> Result<String, ShellError> {
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
        .ok_or_else(|| {
            ShellError::message(format!(
                "slice {} is not referenced by any modeled workflow",
                slug.as_ref()
            ))
        })
}

fn project_path(path: String) -> Result<ProjectPath, ShellError> {
    ProjectPath::try_new(path).map_err(ShellError::project_path)
}

fn read_file_contents(path: &str) -> Result<FileContents, ShellError> {
    FileContents::try_new(fs::read_to_string(Path::new(path)).map_err(ShellError::io)?)
        .map_err(|error| ShellError::message(error.to_string()))
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
            let indentation_length = line.len() - line.trim_start().len();
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
            let indentation_length = line.len() - line.trim_start().len();
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

    let required_categories = required_review_categories()?;
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

fn required_review_categories() -> Result<Vec<ReviewRuleName>, ShellError> {
    Ok(ReviewRuleName::REQUIRED.to_vec())
}

fn current_projection_fingerprint() -> Result<ProjectionFingerprint, ShellError> {
    exported_events_projection_fingerprint()
        .map_err(ShellError::message)?
        .ok_or_else(|| ShellError::message("event export is required before verification"))
        .and_then(|fingerprint| {
            ArtifactDigest::try_new(fingerprint)
                .map(ProjectionFingerprint::new)
                .map_err(|error| ShellError::message(error.to_string()))
        })
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
    reports.iter().for_each(|line| report(line));
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
            Ok(value) => Self::parse(value),
            Err(env::VarError::NotPresent) => Ok(Self {
                process_count: thread::available_parallelism()
                    .map(|parallelism| parallelism.get().min(MAX_PARALLEL_VERIFICATION_PROCESSES))
                    .unwrap_or(1),
            }),
            Err(env::VarError::NotUnicode(_value)) => Err(ShellError::message(format!(
                "{VERIFY_PARALLELISM_ENV} must be a positive integer"
            ))),
        }
    }

    fn parse(value: String) -> Result<Self, ShellError> {
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
        arguments.insert(input_position + 1, endpoint.endpoint().to_owned());
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
        let _ = FileExt::unlock(&self.file);
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
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
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
        let _ = fs::remove_file(&self.path);
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
        .map(|argument| argument.as_ref())
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
