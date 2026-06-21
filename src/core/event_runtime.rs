// Copyright 2026 John Wilger

use std::fs::{self, File, OpenOptions};
use std::future::Future;
use std::path::{Path, PathBuf};

use eventcore::{RetryPolicy, execute};
use eventcore_fs::{FileEventStore, ResolutionOutcome};
use eventcore_types::{BatchSize, EventFilter, EventPage, EventReader};
use fs4::FileExt;
use tokio::runtime::Builder;

use crate::core::event_commands::{
    AddSliceCommand, AddSliceFactCommand, AddWorkflowCommand, AddWorkflowFactCommand,
    ConnectWorkflowCommand, DeclareWorkflowReadinessCommand, EmcEvent, InitializeProjectCommand,
    RecordReviewCommand, RemoveAutomationDefinitionCommand, RemoveBitLevelDataFlowCommand,
    RemoveBoardConnectionCommand, RemoveBoardElementCommand, RemoveCommandDefinitionCommand,
    RemoveEventDefinitionCommand, RemoveExternalPayloadDefinitionCommand,
    RemoveOutcomeDefinitionCommand, RemoveReadModelDefinitionCommand, RemoveSliceCommand,
    RemoveSliceScenarioCommand, RemoveTranslationDefinitionCommand, RemoveViewControlCommand,
    RemoveViewDefinitionCommand, RemoveWorkflowCommand, RemoveWorkflowCommandErrorCommand,
    RemoveWorkflowOutcomeCommand, RemoveWorkflowOwnedDefinitionCommand,
    RemoveWorkflowTransitionCommand, ResolveConflictCommand, SliceFactInput,
    UpdateAutomationDefinitionCommand, UpdateBitLevelDataFlowCommand, UpdateBoardConnectionCommand,
    UpdateBoardElementCommand, UpdateCommandDefinitionCommand, UpdateEventDefinitionCommand,
    UpdateExternalPayloadDefinitionCommand, UpdateOutcomeDefinitionCommand,
    UpdateReadModelDefinitionCommand, UpdateSliceCommand, UpdateSliceScenarioCommand,
    UpdateTranslationDefinitionCommand, UpdateViewControlCommand, UpdateViewDefinitionCommand,
    UpdateWorkflowCommand, UpdateWorkflowCommandErrorCommand, UpdateWorkflowOutcomeCommand,
    UpdateWorkflowOwnedDefinitionCommand, UpdateWorkflowTransitionCommand, WorkflowFactInput,
};
use crate::core::events::{EventDraft, ExportedEventBody};

/// Store root, relative to the project root. eventcore-fs commits only
/// `model/events/events/` (immutable JSONL transactions); it writes its own
/// `.gitignore`/`.gitattributes` excluding `tmp/`, `index/`, `.eventcore/`,
/// `locks/`, and `.lock`.
const EVENT_STORE_DIRECTORY: &str = "model/events";

#[cfg(test)]
#[path = "event_runtime_external_tests.rs"]
mod external_tests;

pub(crate) fn event_store_root(project_root: &Path) -> PathBuf {
    project_root.join(EVENT_STORE_DIRECTORY)
}

fn open_store(project_root: &Path) -> Result<FileEventStore, String> {
    FileEventStore::open(event_store_root(project_root)).map_err(|error| error.to_string())
}

fn run_async<F: Future>(future: F) -> Result<F::Output, String> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    Ok(runtime.block_on(future))
}

pub(crate) struct ProjectRuntimeLock {
    file: File,
}

impl Drop for ProjectRuntimeLock {
    fn drop(&mut self) {
        // Releasing the advisory lock can fail, but a panic in `drop` would be
        // worse than a stale advisory lock: the OS releases the lock when the
        // underlying file handle is closed regardless, so we observe (rather
        // than discard) the result and continue without panicking.
        if let Err(unlock_error) = self.file.unlock() {
            drop(unlock_error);
        }
    }
}

/// Acquire the project-wide runtime lock so a single `emc` invocation owns the
/// command + projection cycle. The lock file lives under the store's gitignored
/// `locks/` directory, separate from the store's own `.lock`.
pub(crate) fn lock_project_runtime(project_root: &Path) -> Result<ProjectRuntimeLock, String> {
    let lock_directory = event_store_root(project_root).join("locks");
    fs::create_dir_all(&lock_directory).map_err(|error| error.to_string())?;
    let path = lock_directory.join("runtime.lock");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(|error| error.to_string())?;
    FileExt::lock(&file).map_err(|error| error.to_string())?;
    Ok(ProjectRuntimeLock { file })
}

/// Open (creating if absent) the file event store so its directory layout and
/// git metadata exist. Replaces the former `SQLite` cache provisioning.
pub(crate) fn ensure_event_store(project_root: &Path) -> Result<(), String> {
    let _store = open_store(project_root)?;
    Ok(())
}

/// Read every persisted `EmcEvent` in local-ingestion order (ADR-0043), which
/// for a single-writer history is append order and for a merged history places
/// merge-introduced events after everything previously ingested.
pub(crate) fn read_all_emc_events(project_root: &Path) -> Result<Vec<EmcEvent>, String> {
    if !event_store_root(project_root).join("events").exists() {
        return Ok(Vec::new());
    }
    let store = open_store(project_root)?;
    run_async(async {
        let filter = EventFilter::all();
        let mut page = EventPage::first(BatchSize::new(1024));
        let mut events = Vec::new();
        loop {
            let batch = store
                .read_events::<EmcEvent>(filter.clone(), page)
                .await
                .map_err(|error| error.to_string())?;
            let Some(last_position) = batch.last().map(|(_, position)| *position) else {
                break;
            };
            events.extend(batch.into_iter().map(|(event, _position)| event));
            page = EventPage::after(last_position, BatchSize::new(1024));
        }
        Ok::<_, String>(events)
    })?
}

/// The unresolved forks in the (possibly git-merged) history: each is two or
/// more transactions extending one stream from the same base version. This is
/// the eventcore-fs replacement for the former bespoke semantic-conflict
/// detector. A store with no `events/` directory has no forks.
pub(crate) fn list_forks(project_root: &Path) -> Result<Vec<eventcore_fs::Fork>, String> {
    if !event_store_root(project_root).join("events").exists() {
        return Ok(Vec::new());
    }
    open_store(project_root)?
        .detect_forks()
        .map_err(|error| error.to_string())
}

pub(crate) fn execute_eventcore_command_for_exported_event(
    project_root: &Path,
    draft: &EventDraft,
) -> Result<(), String> {
    let store = open_store(project_root)?;
    run_async(async { dispatch_exported_event(&store, draft.body()).await })?
}

/// Route an `ExportedEventBody` to the eventcore command that reproduces it,
/// then execute that command. Behavior-preserving split of the former monolith:
/// each arm delegates to a small builder helper that constructs the command and
/// runs it via [`run_command`], keeping ordering and error paths identical.
async fn dispatch_exported_event(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::ProjectInitialized { .. }
        | ExportedEventBody::WorkflowAdded { .. }
        | ExportedEventBody::WorkflowUpdated { .. }
        | ExportedEventBody::WorkflowRemoved { .. }
        | ExportedEventBody::WorkflowReadinessDeclared { .. }
        | ExportedEventBody::WorkflowConnected { .. }
        | ExportedEventBody::WorkflowTransitionUpdated { .. }
        | ExportedEventBody::WorkflowTransitionRemoved { .. } => {
            dispatch_workflow_lifecycle(store, body).await
        }
        ExportedEventBody::WorkflowOutcomeAdded { .. }
        | ExportedEventBody::WorkflowOutcomeUpdated { .. }
        | ExportedEventBody::WorkflowOutcomeRemoved { .. }
        | ExportedEventBody::WorkflowCommandErrorAdded { .. }
        | ExportedEventBody::WorkflowCommandErrorUpdated { .. }
        | ExportedEventBody::WorkflowCommandErrorRemoved { .. }
        | ExportedEventBody::WorkflowOwnedDefinitionAdded { .. }
        | ExportedEventBody::WorkflowOwnedDefinitionUpdated { .. }
        | ExportedEventBody::WorkflowOwnedDefinitionRemoved { .. }
        | ExportedEventBody::WorkflowTransitionEvidenceAdded { .. }
        | ExportedEventBody::WorkflowEntryLifecycleCoverageRequired { .. }
        | ExportedEventBody::WorkflowEntryLifecycleStateAdded { .. } => {
            dispatch_workflow_fact(store, body).await
        }
        ExportedEventBody::SliceAdded { .. }
        | ExportedEventBody::SliceUpdated { .. }
        | ExportedEventBody::SliceRemoved { .. }
        | ExportedEventBody::SliceScenarioAdded { .. }
        | ExportedEventBody::SliceScenarioUpdated { .. }
        | ExportedEventBody::SliceScenarioRemoved { .. }
        | ExportedEventBody::SliceOutcomeAdded { .. }
        | ExportedEventBody::SliceOutcomeDefinitionUpdated { .. }
        | ExportedEventBody::SliceOutcomeDefinitionRemoved { .. }
        | ExportedEventBody::SliceExternalPayloadAdded { .. }
        | ExportedEventBody::SliceExternalPayloadDefinitionUpdated { .. }
        | ExportedEventBody::SliceExternalPayloadDefinitionRemoved { .. }
        | ExportedEventBody::SliceEventDefinitionAdded { .. }
        | ExportedEventBody::SliceEventDefinitionUpdated { .. }
        | ExportedEventBody::SliceEventDefinitionRemoved { .. }
        | ExportedEventBody::SliceCommandDefinitionAdded { .. }
        | ExportedEventBody::SliceCommandDefinitionUpdated { .. }
        | ExportedEventBody::SliceCommandDefinitionRemoved { .. }
        | ExportedEventBody::SliceReadModelAdded { .. }
        | ExportedEventBody::SliceReadModelDefinitionUpdated { .. }
        | ExportedEventBody::SliceReadModelDefinitionRemoved { .. }
        | ExportedEventBody::SliceViewAdded { .. }
        | ExportedEventBody::SliceViewDefinitionUpdated { .. }
        | ExportedEventBody::SliceViewDefinitionRemoved { .. }
        | ExportedEventBody::SliceViewControlUpdated { .. }
        | ExportedEventBody::SliceViewControlRemoved { .. }
        | ExportedEventBody::SliceBitLevelDataFlowAdded { .. }
        | ExportedEventBody::SliceBitLevelDataFlowUpdated { .. }
        | ExportedEventBody::SliceBitLevelDataFlowRemoved { .. }
        | ExportedEventBody::SliceTranslationAdded { .. }
        | ExportedEventBody::SliceTranslationDefinitionUpdated { .. }
        | ExportedEventBody::SliceTranslationDefinitionRemoved { .. }
        | ExportedEventBody::SliceAutomationAdded { .. }
        | ExportedEventBody::SliceAutomationDefinitionUpdated { .. }
        | ExportedEventBody::SliceAutomationDefinitionRemoved { .. }
        | ExportedEventBody::SliceBoardElementAdded { .. }
        | ExportedEventBody::SliceBoardElementUpdated { .. }
        | ExportedEventBody::SliceBoardElementRemoved { .. }
        | ExportedEventBody::SliceBoardConnectionAdded { .. }
        | ExportedEventBody::SliceBoardConnectionUpdated { .. }
        | ExportedEventBody::SliceBoardConnectionRemoved { .. } => {
            dispatch_slice(store, body).await
        }
        ExportedEventBody::ReviewRecorded { .. } | ExportedEventBody::ConflictResolved { .. } => {
            dispatch_review_or_conflict(store, body).await
        }
    }
}

/// Execute a single eventcore command against `store`, discarding the response
/// and normalizing any error to a `String`. Centralizes the retry policy and
/// error mapping shared by every dispatch arm.
async fn run_command<C>(store: &FileEventStore, command: C) -> Result<(), String>
where
    C: eventcore::CommandLogic,
{
    execute(store, command, RetryPolicy::new())
        .await
        .map(|_response| ())
        .map_err(|error| error.to_string())
}

/// Workflow-level lifecycle bodies (create/update/remove, readiness, connection,
/// transition removal). Variants outside this group cannot reach this arm
/// because [`dispatch_exported_event`] routes only the matching set here.
async fn dispatch_workflow_lifecycle(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::ProjectInitialized { name } => {
            run_command(
                store,
                InitializeProjectCommand::from_semantic(name.clone())?,
            )
            .await
        }
        ExportedEventBody::WorkflowAdded { workflow } => {
            run_command(
                store,
                AddWorkflowCommand::from_semantic(
                    workflow.slug().clone(),
                    workflow.name().clone(),
                    workflow.description().clone(),
                )?,
            )
            .await
        }
        ExportedEventBody::WorkflowUpdated { workflow } => {
            run_command(
                store,
                UpdateWorkflowCommand::from_semantic(
                    workflow.slug().clone(),
                    workflow.name().clone(),
                    workflow.description().clone(),
                )?,
            )
            .await
        }
        ExportedEventBody::WorkflowRemoved { slug } => {
            run_command(store, RemoveWorkflowCommand::from_semantic(slug.clone())?).await
        }
        ExportedEventBody::WorkflowReadinessDeclared {
            workflow,
            projection_fingerprint,
            model_content_digest,
            verified_at,
            verified_by,
            review_event,
        } => {
            run_command(
                store,
                DeclareWorkflowReadinessCommand::new(
                    workflow.clone(),
                    projection_fingerprint.clone(),
                    model_content_digest.clone(),
                    verified_at.clone(),
                    verified_by.clone(),
                    review_event.clone(),
                )?,
            )
            .await
        }
        ExportedEventBody::WorkflowConnected { connection } => {
            run_command(
                store,
                ConnectWorkflowCommand::from_connection(connection.clone())?,
            )
            .await
        }
        ExportedEventBody::WorkflowTransitionUpdated { update } => {
            run_command(
                store,
                UpdateWorkflowTransitionCommand::from_update(update.clone())?,
            )
            .await
        }
        ExportedEventBody::WorkflowTransitionRemoved { removal } => {
            run_command(
                store,
                RemoveWorkflowTransitionCommand::from_removal(removal.clone())?,
            )
            .await
        }
        _ => Err("dispatch_workflow_lifecycle received a non-lifecycle body".to_owned()),
    }
}

/// Workflow fact bodies, all reproduced through [`AddWorkflowFactCommand`] with
/// the appropriate [`WorkflowFactInput`]. Routed exclusively from
/// [`dispatch_exported_event`].
async fn dispatch_workflow_fact(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    if dispatch_workflow_fact_mutation(store, body).await? {
        return Ok(());
    }

    let input = match body {
        ExportedEventBody::WorkflowOutcomeAdded { workflow, outcome } => {
            WorkflowFactInput::OutcomeAdded {
                workflow: workflow.clone(),
                outcome: outcome.clone(),
            }
        }
        ExportedEventBody::WorkflowCommandErrorAdded { workflow, error } => {
            WorkflowFactInput::CommandErrorAdded {
                workflow: workflow.clone(),
                error: error.clone(),
            }
        }
        ExportedEventBody::WorkflowOwnedDefinitionAdded {
            workflow,
            definition,
        } => WorkflowFactInput::OwnedDefinitionAdded {
            workflow: workflow.clone(),
            definition: definition.clone(),
        },
        ExportedEventBody::WorkflowTransitionEvidenceAdded { workflow, evidence } => {
            WorkflowFactInput::TransitionEvidenceAdded {
                workflow: workflow.clone(),
                evidence: evidence.clone(),
            }
        }
        ExportedEventBody::WorkflowEntryLifecycleCoverageRequired { workflow } => {
            WorkflowFactInput::EntryLifecycleCoverageRequired {
                workflow: workflow.clone(),
            }
        }
        ExportedEventBody::WorkflowEntryLifecycleStateAdded { workflow, coverage } => {
            WorkflowFactInput::EntryLifecycleStateAdded {
                workflow: workflow.clone(),
                state: coverage.clone(),
            }
        }
        _ => return Err("dispatch_workflow_fact received a non-fact body".to_owned()),
    };
    run_command(store, AddWorkflowFactCommand::new(input)?).await
}

async fn dispatch_workflow_fact_mutation(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<bool, String> {
    match body {
        ExportedEventBody::WorkflowOutcomeUpdated {
            workflow,
            previous,
            outcome,
        } => {
            run_command(
                store,
                UpdateWorkflowOutcomeCommand::new(
                    workflow.clone(),
                    previous.clone(),
                    outcome.clone(),
                )?,
            )
            .await?;
            Ok(true)
        }
        ExportedEventBody::WorkflowOutcomeRemoved { workflow, outcome } => {
            run_command(
                store,
                RemoveWorkflowOutcomeCommand::new(workflow.clone(), outcome.clone())?,
            )
            .await?;
            Ok(true)
        }
        ExportedEventBody::WorkflowCommandErrorUpdated {
            workflow,
            previous,
            error,
        } => {
            run_command(
                store,
                UpdateWorkflowCommandErrorCommand::new(
                    workflow.clone(),
                    previous.clone(),
                    error.clone(),
                )?,
            )
            .await?;
            Ok(true)
        }
        ExportedEventBody::WorkflowCommandErrorRemoved { workflow, error } => {
            run_command(
                store,
                RemoveWorkflowCommandErrorCommand::new(workflow.clone(), error.clone())?,
            )
            .await?;
            Ok(true)
        }
        ExportedEventBody::WorkflowOwnedDefinitionUpdated {
            workflow,
            previous,
            definition,
        } => {
            run_command(
                store,
                UpdateWorkflowOwnedDefinitionCommand::new(
                    workflow.clone(),
                    previous.clone(),
                    definition.clone(),
                )?,
            )
            .await?;
            Ok(true)
        }
        ExportedEventBody::WorkflowOwnedDefinitionRemoved {
            workflow,
            definition,
        } => {
            run_command(
                store,
                RemoveWorkflowOwnedDefinitionCommand::new(workflow.clone(), definition.clone())?,
            )
            .await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Slice bodies: the three structural commands (add/update/remove) plus the
/// uniform slice-fact group reproduced through [`AddSliceFactCommand`]. Routed
/// exclusively from [`dispatch_exported_event`].
async fn dispatch_slice(store: &FileEventStore, body: &ExportedEventBody) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceAdded { slice } => {
            run_command(
                store,
                AddSliceCommand::from_semantic(
                    slice.workflow_slug().clone(),
                    slice.slug().clone(),
                    slice.name().clone(),
                    slice.kind().into(),
                    slice.description().clone(),
                )?,
            )
            .await
        }
        ExportedEventBody::SliceUpdated { slice } => {
            run_command(
                store,
                UpdateSliceCommand::from_semantic(
                    slice.slug().clone(),
                    slice.name().clone(),
                    *slice.kind(),
                    slice.description().clone(),
                )?,
            )
            .await
        }
        ExportedEventBody::SliceRemoved { slug } => {
            run_command(store, RemoveSliceCommand::from_semantic(slug.clone())?).await
        }
        ExportedEventBody::SliceScenarioUpdated { .. }
        | ExportedEventBody::SliceScenarioRemoved { .. } => {
            dispatch_slice_scenario(store, body).await
        }
        ExportedEventBody::SliceCommandDefinitionUpdated { .. }
        | ExportedEventBody::SliceCommandDefinitionRemoved { .. } => {
            dispatch_slice_command_definition(store, body).await
        }
        ExportedEventBody::SliceEventDefinitionUpdated { event } => {
            run_command(store, UpdateEventDefinitionCommand::new(event.clone())?).await
        }
        ExportedEventBody::SliceEventDefinitionRemoved { slice, name } => {
            run_command(
                store,
                RemoveEventDefinitionCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceExternalPayloadDefinitionUpdated { .. }
        | ExportedEventBody::SliceExternalPayloadDefinitionRemoved { .. } => {
            dispatch_slice_external_payload(store, body).await
        }
        ExportedEventBody::SliceAutomationDefinitionUpdated { .. }
        | ExportedEventBody::SliceAutomationDefinitionRemoved { .. } => {
            dispatch_slice_automation(store, body).await
        }
        ExportedEventBody::SliceBitLevelDataFlowUpdated { .. }
        | ExportedEventBody::SliceBitLevelDataFlowRemoved { .. } => {
            dispatch_slice_bit_level_data_flow(store, body).await
        }
        ExportedEventBody::SliceBoardElementUpdated { .. }
        | ExportedEventBody::SliceBoardElementRemoved { .. } => {
            dispatch_slice_board_element(store, body).await
        }
        ExportedEventBody::SliceBoardConnectionUpdated { .. }
        | ExportedEventBody::SliceBoardConnectionRemoved { .. } => {
            dispatch_slice_board_connection(store, body).await
        }
        ExportedEventBody::SliceTranslationDefinitionUpdated { .. }
        | ExportedEventBody::SliceTranslationDefinitionRemoved { .. } => {
            dispatch_slice_translation(store, body).await
        }
        ExportedEventBody::SliceOutcomeDefinitionUpdated { .. }
        | ExportedEventBody::SliceOutcomeDefinitionRemoved { .. } => {
            dispatch_slice_outcome(store, body).await
        }
        ExportedEventBody::SliceReadModelDefinitionUpdated { .. }
        | ExportedEventBody::SliceReadModelDefinitionRemoved { .. } => {
            dispatch_slice_read_model(store, body).await
        }
        ExportedEventBody::SliceViewDefinitionUpdated { .. }
        | ExportedEventBody::SliceViewDefinitionRemoved { .. }
        | ExportedEventBody::SliceViewControlUpdated { .. }
        | ExportedEventBody::SliceViewControlRemoved { .. } => {
            dispatch_slice_view(store, body).await
        }
        _ => {
            run_command(
                store,
                AddSliceFactCommand::new(SliceFactInput::from_event_body(body)?)?,
            )
            .await
        }
    }
}

async fn dispatch_slice_outcome(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceOutcomeDefinitionUpdated { outcome } => {
            run_command(store, UpdateOutcomeDefinitionCommand::new(outcome.clone())?).await
        }
        ExportedEventBody::SliceOutcomeDefinitionRemoved { slice, label } => {
            run_command(
                store,
                RemoveOutcomeDefinitionCommand::new(slice.clone(), label.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_outcome received a non-outcome body".to_owned()),
    }
}

async fn dispatch_slice_scenario(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceScenarioUpdated { scenario } => {
            run_command(store, UpdateSliceScenarioCommand::new(scenario.clone())?).await
        }
        ExportedEventBody::SliceScenarioRemoved { slice, name } => {
            run_command(
                store,
                RemoveSliceScenarioCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_scenario received a non-scenario body".to_owned()),
    }
}

async fn dispatch_slice_command_definition(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceCommandDefinitionUpdated { command } => {
            run_command(store, UpdateCommandDefinitionCommand::new(command.clone())?).await
        }
        ExportedEventBody::SliceCommandDefinitionRemoved { slice, name } => {
            run_command(
                store,
                RemoveCommandDefinitionCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_command_definition received a non-command body".to_owned()),
    }
}

async fn dispatch_slice_read_model(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceReadModelDefinitionUpdated { read_model } => {
            run_command(
                store,
                UpdateReadModelDefinitionCommand::new(read_model.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceReadModelDefinitionRemoved { slice, name } => {
            run_command(
                store,
                RemoveReadModelDefinitionCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_read_model received a non-read-model body".to_owned()),
    }
}

async fn dispatch_slice_external_payload(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceExternalPayloadDefinitionUpdated { external_payload } => {
            run_command(
                store,
                UpdateExternalPayloadDefinitionCommand::new(external_payload.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceExternalPayloadDefinitionRemoved { slice, name, field } => {
            run_command(
                store,
                RemoveExternalPayloadDefinitionCommand::new(
                    slice.clone(),
                    name.clone(),
                    field.clone(),
                )?,
            )
            .await
        }
        _ => Err("dispatch_slice_external_payload received a non-external-payload body".to_owned()),
    }
}

async fn dispatch_slice_automation(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceAutomationDefinitionUpdated { automation } => {
            run_command(
                store,
                UpdateAutomationDefinitionCommand::new(automation.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceAutomationDefinitionRemoved { slice, name } => {
            run_command(
                store,
                RemoveAutomationDefinitionCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_automation received a non-automation body".to_owned()),
    }
}

async fn dispatch_slice_board_element(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceBoardElementUpdated { element } => {
            run_command(store, UpdateBoardElementCommand::new(element.clone())?).await
        }
        ExportedEventBody::SliceBoardElementRemoved { slice, name } => {
            run_command(
                store,
                RemoveBoardElementCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_board_element received a non-board-element body".to_owned()),
    }
}

async fn dispatch_slice_bit_level_data_flow(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceBitLevelDataFlowUpdated {
            previous,
            data_flow,
        } => {
            run_command(
                store,
                UpdateBitLevelDataFlowCommand::new(previous.clone(), data_flow.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceBitLevelDataFlowRemoved { data_flow } => {
            run_command(
                store,
                RemoveBitLevelDataFlowCommand::new(data_flow.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_bit_level_data_flow received a non-data-flow body".to_owned()),
    }
}

async fn dispatch_slice_board_connection(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceBoardConnectionUpdated {
            previous,
            connection,
        } => {
            run_command(
                store,
                UpdateBoardConnectionCommand::new(previous.clone(), connection.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceBoardConnectionRemoved { connection } => {
            run_command(
                store,
                RemoveBoardConnectionCommand::new(connection.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_board_connection received a non-board-connection body".to_owned()),
    }
}

async fn dispatch_slice_translation(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceTranslationDefinitionUpdated { translation } => {
            run_command(
                store,
                UpdateTranslationDefinitionCommand::new(translation.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceTranslationDefinitionRemoved { slice, name } => {
            run_command(
                store,
                RemoveTranslationDefinitionCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_translation received a non-translation body".to_owned()),
    }
}

async fn dispatch_slice_view(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::SliceViewDefinitionUpdated { view } => {
            run_command(store, UpdateViewDefinitionCommand::new(view.clone())?).await
        }
        ExportedEventBody::SliceViewDefinitionRemoved { slice, name } => {
            run_command(
                store,
                RemoveViewDefinitionCommand::new(slice.clone(), name.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceViewControlUpdated {
            slice,
            view,
            control,
        } => {
            run_command(
                store,
                UpdateViewControlCommand::new(slice.clone(), view.clone(), control.clone())?,
            )
            .await
        }
        ExportedEventBody::SliceViewControlRemoved { slice, view, name } => {
            run_command(
                store,
                RemoveViewControlCommand::new(slice.clone(), view.clone(), name.clone())?,
            )
            .await
        }
        _ => Err("dispatch_slice_view received a non-view body".to_owned()),
    }
}

/// Review and conflict-resolution bodies. Routed exclusively from
/// [`dispatch_exported_event`].
async fn dispatch_review_or_conflict(
    store: &FileEventStore,
    body: &ExportedEventBody,
) -> Result<(), String> {
    match body {
        ExportedEventBody::ReviewRecorded {
            workflow_slug,
            model_content_digest,
            reviewer_id,
            reviewed_at,
            categories,
        } => {
            run_command(
                store,
                RecordReviewCommand::from_semantic(
                    workflow_slug.clone(),
                    model_content_digest.clone(),
                    reviewer_id.clone(),
                    reviewed_at.clone(),
                    categories.clone(),
                )?,
            )
            .await
        }
        ExportedEventBody::ConflictResolved {
            conflict_id,
            chosen_event_id,
        } => {
            run_command(
                store,
                ResolveConflictCommand::from_semantic(
                    conflict_id.clone(),
                    chosen_event_id.clone(),
                )?,
            )
            .await
        }
        _ => Err("dispatch_review_or_conflict received an unrelated body".to_owned()),
    }
}

/// Resolve the fork on `stream_id` by keeping the branch written by the
/// transaction whose id serializes to `transaction_id`. eventcore-fs records a
/// merge transaction containing that branch's events, collapsing the fork.
/// Returns the number of forks resolved (0 if none matched).
pub(crate) fn reconcile_choose_branch(
    project_root: &Path,
    stream_id: &str,
    transaction_id: &str,
) -> Result<usize, String> {
    if !event_store_root(project_root).join("events").exists() {
        return Ok(0);
    }
    let store = open_store(project_root)?;
    let report = run_async(async {
        store
            .reconcile::<EmcEvent, _>(|context| {
                if context.stream_id().as_ref() != stream_id {
                    return ResolutionOutcome::Unresolvable("stream not selected".to_owned());
                }
                for branch in context.branches() {
                    if transaction_id_string(branch.transaction_id()) == transaction_id {
                        return ResolutionOutcome::Resolve(branch.events().to_vec());
                    }
                }
                ResolutionOutcome::Unresolvable("no matching branch".to_owned())
            })
            .await
            .map_err(|error| error.to_string())
    })??;
    Ok(report.resolved_count())
}

fn transaction_id_string(id: eventcore_fs::TransactionId) -> String {
    serde_json::to_value(id)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}
