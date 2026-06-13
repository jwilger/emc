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
    RecordReviewCommand, RemoveSliceCommand, RemoveWorkflowCommand,
    RemoveWorkflowTransitionCommand, ResolveConflictCommand, SliceFactInput, UpdateSliceCommand,
    UpdateWorkflowCommand, WorkflowFactInput,
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
        let _ = self.file.unlock();
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
/// git metadata exist. Replaces the former SQLite cache provisioning.
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
    run_async(async {
        match draft.body() {
            ExportedEventBody::ProjectInitialized { name } => {
                execute(
                    &store,
                    InitializeProjectCommand::from_semantic(name.clone())?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowAdded { workflow } => {
                execute(
                    &store,
                    AddWorkflowCommand::from_semantic(
                        workflow.slug().clone(),
                        workflow.name().clone(),
                        workflow.description().clone(),
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowUpdated { workflow } => {
                execute(
                    &store,
                    UpdateWorkflowCommand::from_semantic(
                        workflow.slug().clone(),
                        workflow.name().clone(),
                        workflow.description().clone(),
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowRemoved { slug } => {
                execute(
                    &store,
                    RemoveWorkflowCommand::from_semantic(slug.clone())?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowOutcomeAdded { workflow, outcome } => {
                execute(
                    &store,
                    AddWorkflowFactCommand::new(WorkflowFactInput::OutcomeAdded {
                        workflow: workflow.clone(),
                        outcome: outcome.clone(),
                    })?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowCommandErrorAdded { workflow, error } => {
                execute(
                    &store,
                    AddWorkflowFactCommand::new(WorkflowFactInput::CommandErrorAdded {
                        workflow: workflow.clone(),
                        error: error.clone(),
                    })?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowOwnedDefinitionAdded {
                workflow,
                definition,
            } => {
                execute(
                    &store,
                    AddWorkflowFactCommand::new(WorkflowFactInput::OwnedDefinitionAdded {
                        workflow: workflow.clone(),
                        definition: definition.clone(),
                    })?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowTransitionEvidenceAdded { workflow, evidence } => {
                execute(
                    &store,
                    AddWorkflowFactCommand::new(WorkflowFactInput::TransitionEvidenceAdded {
                        workflow: workflow.clone(),
                        evidence: evidence.clone(),
                    })?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowEntryLifecycleCoverageRequired { workflow } => {
                execute(
                    &store,
                    AddWorkflowFactCommand::new(
                        WorkflowFactInput::EntryLifecycleCoverageRequired {
                            workflow: workflow.clone(),
                        },
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowEntryLifecycleStateAdded { workflow, coverage } => {
                execute(
                    &store,
                    AddWorkflowFactCommand::new(WorkflowFactInput::EntryLifecycleStateAdded {
                        workflow: workflow.clone(),
                        state: coverage.clone(),
                    })?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowReadinessDeclared {
                workflow,
                projection_fingerprint,
                model_content_digest,
                verified_at,
                verified_by,
                review_event,
            } => {
                execute(
                    &store,
                    DeclareWorkflowReadinessCommand::new(
                        workflow.clone(),
                        projection_fingerprint.clone(),
                        model_content_digest.clone(),
                        verified_at.clone(),
                        verified_by.clone(),
                        review_event.clone(),
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowConnected { connection } => {
                execute(
                    &store,
                    ConnectWorkflowCommand::from_connection(connection.clone())?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::WorkflowTransitionRemoved { removal } => {
                execute(
                    &store,
                    RemoveWorkflowTransitionCommand::from_removal(removal.clone())?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::SliceAdded { slice } => {
                execute(
                    &store,
                    AddSliceCommand::from_semantic(
                        slice.workflow_slug().clone(),
                        slice.slug().clone(),
                        slice.name().clone(),
                        slice.kind().into(),
                        slice.description().clone(),
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::SliceUpdated { slice } => {
                execute(
                    &store,
                    UpdateSliceCommand::from_semantic(
                        slice.slug().clone(),
                        slice.name().clone(),
                        *slice.kind(),
                        slice.description().clone(),
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::SliceRemoved { slug } => {
                execute(
                    &store,
                    RemoveSliceCommand::from_semantic(slug.clone())?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::SliceScenarioAdded { .. }
            | ExportedEventBody::SliceOutcomeAdded { .. }
            | ExportedEventBody::SliceExternalPayloadAdded { .. }
            | ExportedEventBody::SliceEventDefinitionAdded { .. }
            | ExportedEventBody::SliceCommandDefinitionAdded { .. }
            | ExportedEventBody::SliceReadModelAdded { .. }
            | ExportedEventBody::SliceViewAdded { .. }
            | ExportedEventBody::SliceBitLevelDataFlowAdded { .. }
            | ExportedEventBody::SliceTranslationAdded { .. }
            | ExportedEventBody::SliceAutomationAdded { .. }
            | ExportedEventBody::SliceBoardElementAdded { .. }
            | ExportedEventBody::SliceBoardConnectionAdded { .. } => {
                execute(
                    &store,
                    AddSliceFactCommand::new(SliceFactInput::from_event_body(draft.body())?)?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::ReviewRecorded {
                workflow_slug,
                model_content_digest,
                reviewer_id,
                reviewed_at,
                categories,
            } => {
                execute(
                    &store,
                    RecordReviewCommand::from_semantic(
                        workflow_slug.clone(),
                        model_content_digest.clone(),
                        reviewer_id.clone(),
                        reviewed_at.clone(),
                        categories.clone(),
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            ExportedEventBody::ConflictResolved {
                conflict_id,
                chosen_event_id,
            } => {
                execute(
                    &store,
                    ResolveConflictCommand::from_semantic(
                        conflict_id.clone(),
                        chosen_event_id.clone(),
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
        }
        Ok::<(), String>(())
    })?
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
