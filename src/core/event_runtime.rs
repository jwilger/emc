// Copyright 2026 John Wilger

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

use eventcore::{RetryPolicy, execute};
use eventcore_sqlite::rusqlite::params;
use eventcore_sqlite::{SqliteConfig, SqliteEventStore, rusqlite};
use fs4::FileExt;
use sha2::{Digest, Sha256};
use tokio::runtime::Builder;

use crate::core::event_commands::{
    AddSliceCommand, AddSliceFactCommand, AddWorkflowCommand, AddWorkflowFactCommand,
    ConnectWorkflowCommand, DeclareWorkflowReadinessCommand, InitializeProjectCommand,
    RecordReviewCommand, RemoveSliceCommand, RemoveWorkflowCommand,
    RemoveWorkflowTransitionCommand, ResolveConflictCommand, SliceFactInput, UpdateSliceCommand,
    UpdateWorkflowCommand, WorkflowFactInput,
};
use crate::core::events::{EventDraft, ExportedEvent, ExportedEventBody, ExportedEventType};
use crate::core::types::{ModelDescription, ModelName, SliceKindName, SliceSlug, WorkflowSlug};

const EVENT_EXPORT_DIRECTORY: &str = "model/events/v1";

const EVENT_STORE_PATH_ENV: &str = "EMC_EVENT_STORE_PATH";
const XDG_STATE_HOME_ENV: &str = "XDG_STATE_HOME";
const HOME_ENV: &str = "HOME";

#[cfg(test)]
#[path = "event_runtime_external_tests.rs"]
mod external_tests;

pub(crate) fn sqlite_event_store_path(project_root: &Path) -> Result<PathBuf, String> {
    sqlite_event_store_path_with_env(project_root, |name| env::var_os(name))
}

fn sqlite_event_store_path_with_env(
    project_root: &Path,
    env_var: impl Fn(&str) -> Option<OsString>,
) -> Result<PathBuf, String> {
    if let Some(path) = env_var(EVENT_STORE_PATH_ENV).filter(|path| !path.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    let state_home = env_var(XDG_STATE_HOME_ENV)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env_var(HOME_ENV)
                .filter(|path| !path.is_empty())
                .map(PathBuf::from)
                .map(|home| home.join(".local").join("state"))
        })
        .ok_or_else(|| {
            format!("{XDG_STATE_HOME_ENV} or {HOME_ENV} is required to resolve event store path")
        })?;
    let project_hash = project_realpath_hash(project_root)?;

    Ok(state_home
        .join("emc")
        .join("projects")
        .join(project_hash)
        .join("events.sqlite3"))
}

pub(crate) struct ProjectRuntimeLock {
    file: File,
}

impl Drop for ProjectRuntimeLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub(crate) fn lock_project_runtime(project_root: &Path) -> Result<ProjectRuntimeLock, String> {
    let lock_directory = project_root.join("model/events");
    fs::create_dir_all(&lock_directory).map_err(|error| error.to_string())?;
    let path = lock_directory.join(".lock");
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

pub(crate) fn ensure_sqlite_event_store(project_root: &Path) -> Result<PathBuf, String> {
    let path = sqlite_event_store_path(project_root)?;
    migrate_eventcore_sqlite_store(&path)?;
    sync_exported_events_into_sqlite(project_root, &path)?;
    Ok(path)
}

pub(crate) fn execute_eventcore_command_for_exported_event(
    project_root: &Path,
    draft: &EventDraft,
) -> Result<(), String> {
    let path = sqlite_event_store_path(project_root)?;
    execute_eventcore_command_for_exported_event_at_path(project_root, &path, draft)
}

#[cfg(test)]
pub(crate) fn execute_eventcore_command_for_exported_event_at_path_for_test(
    project_root: &Path,
    sqlite_path: &Path,
    draft: &EventDraft,
) -> Result<(), String> {
    execute_eventcore_command_for_exported_event_at_path(project_root, sqlite_path, draft)
}

fn execute_eventcore_command_for_exported_event_at_path(
    project_root: &Path,
    path: &Path,
    draft: &EventDraft,
) -> Result<(), String> {
    repair_project_stream_if_needed(path, draft)?;
    let workflow_added_prerequisite =
        workflow_added_prerequisite_if_stream_needs_repair(project_root, path, draft)?;
    let slice_added_prerequisite =
        slice_added_prerequisite_if_stream_needs_repair(project_root, path, draft)?;
    let store = migrate_eventcore_sqlite_store(path)?;
    Builder::new_current_thread()
        .build()
        .map_err(|error| error.to_string())?
        .block_on(async {
            if let Some(prerequisite) = workflow_added_prerequisite {
                execute(
                    &store,
                    AddWorkflowCommand::from_semantic(
                        prerequisite.slug,
                        prerequisite.name,
                        prerequisite.description,
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            if let Some(prerequisite) = slice_added_prerequisite {
                execute(
                    &store,
                    AddSliceCommand::from_semantic(
                        prerequisite.workflow,
                        prerequisite.slug,
                        prerequisite.name,
                        prerequisite.kind,
                        prerequisite.description,
                    )?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
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
        })
}

fn repair_project_stream_if_needed(sqlite_path: &Path, draft: &EventDraft) -> Result<(), String> {
    if draft.event_type() != ExportedEventType::ConflictResolved {
        return Ok(());
    }

    let conn = rusqlite::Connection::open(sqlite_path).map_err(|error| error.to_string())?;
    if non_emc_event_rows_for_stream(&conn, "project").map_err(|error| error.to_string())? == 0 {
        return Ok(());
    }

    conn.execute(
        "DELETE FROM eventcore_events WHERE stream_id = 'project'",
        [],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn workflow_added_prerequisite_if_stream_needs_repair(
    project_root: &Path,
    sqlite_path: &Path,
    draft: &EventDraft,
) -> Result<Option<WorkflowAddedPrerequisite>, String> {
    if !matches!(
        draft.event_type(),
        ExportedEventType::WorkflowUpdated
            | ExportedEventType::WorkflowRemoved
            | ExportedEventType::WorkflowConnected
            | ExportedEventType::WorkflowTransitionRemoved
            | ExportedEventType::WorkflowOutcomeAdded
            | ExportedEventType::WorkflowCommandErrorAdded
            | ExportedEventType::WorkflowOwnedDefinitionAdded
            | ExportedEventType::WorkflowTransitionEvidenceAdded
            | ExportedEventType::WorkflowEntryLifecycleCoverageRequired
            | ExportedEventType::WorkflowEntryLifecycleStateAdded
            | ExportedEventType::WorkflowReadinessDeclared
    ) {
        return Ok(None);
    }

    let conn = rusqlite::Connection::open(sqlite_path).map_err(|error| error.to_string())?;
    if non_emc_event_rows_for_stream(&conn, draft.stream_id()).map_err(|error| error.to_string())?
        == 0
        && emc_event_rows_for_stream(&conn, draft.stream_id()).map_err(|error| error.to_string())?
            > 0
    {
        return Ok(None);
    }

    conn.execute(
        "DELETE FROM eventcore_events WHERE stream_id = ?1",
        params![draft.stream_id()],
    )
    .map_err(|error| error.to_string())?;
    workflow_added_payload(project_root, draft.stream_id()).map(Some)
}

fn slice_added_prerequisite_if_stream_needs_repair(
    project_root: &Path,
    sqlite_path: &Path,
    draft: &EventDraft,
) -> Result<Option<SliceAddedPrerequisite>, String> {
    if !matches!(
        draft.event_type(),
        ExportedEventType::SliceUpdated | ExportedEventType::SliceRemoved
    ) && !draft.event_type().is_slice_fact()
    {
        return Ok(None);
    }

    let conn = rusqlite::Connection::open(sqlite_path).map_err(|error| error.to_string())?;
    if non_emc_event_rows_for_stream(&conn, draft.stream_id()).map_err(|error| error.to_string())?
        == 0
    {
        return Ok(None);
    }

    conn.execute(
        "DELETE FROM eventcore_events WHERE stream_id = ?1",
        params![draft.stream_id()],
    )
    .map_err(|error| error.to_string())?;
    slice_added_payload(project_root, draft.stream_id()).map(Some)
}

fn emc_event_rows_for_stream(
    conn: &rusqlite::Connection,
    stream_id: &str,
) -> rusqlite::Result<usize> {
    conn.query_row(
        "SELECT count(*) FROM eventcore_events WHERE stream_id = ?1 AND event_type = 'EmcEvent'",
        params![stream_id],
        |row| row.get(0),
    )
}

fn non_emc_event_rows_for_stream(
    conn: &rusqlite::Connection,
    stream_id: &str,
) -> rusqlite::Result<usize> {
    conn.query_row(
        "SELECT count(*) FROM eventcore_events WHERE stream_id = ?1 AND event_type != 'EmcEvent'",
        params![stream_id],
        |row| row.get(0),
    )
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowAddedPrerequisite {
    slug: WorkflowSlug,
    name: ModelName,
    description: ModelDescription,
}

impl WorkflowAddedPrerequisite {
    fn from_exported_event(event: &ExportedEvent) -> Result<Self, String> {
        match event.body() {
            ExportedEventBody::WorkflowAdded { workflow } => Ok(Self {
                slug: workflow.slug().clone(),
                name: workflow.name().clone(),
                description: workflow.description().clone(),
            }),
            _ => Err("expected exported WorkflowAdded event".to_owned()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SliceAddedPrerequisite {
    workflow: WorkflowSlug,
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKindName,
    description: ModelDescription,
}

impl SliceAddedPrerequisite {
    fn from_exported_event(event: &ExportedEvent) -> Result<Self, String> {
        match event.body() {
            ExportedEventBody::SliceAdded { slice } => Ok(Self {
                workflow: slice.workflow_slug().clone(),
                slug: slice.slug().clone(),
                name: slice.name().clone(),
                kind: SliceKindName::from(slice.kind()),
                description: slice.description().clone(),
            }),
            _ => Err("expected exported SliceAdded event".to_owned()),
        }
    }
}

fn workflow_added_payload(
    project_root: &Path,
    stream_id: &str,
) -> Result<WorkflowAddedPrerequisite, String> {
    let event_directory = project_root.join(EVENT_EXPORT_DIRECTORY);
    for entry in fs::read_dir(event_directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let event = ExportedEvent::from_json_str(&contents)?;
        if event.stream_id().as_ref() == stream_id
            && matches!(event.body(), ExportedEventBody::WorkflowAdded { .. })
        {
            return WorkflowAddedPrerequisite::from_exported_event(&event);
        }
    }
    Err(format!("{stream_id} requires exported WorkflowAdded event"))
}

fn slice_added_payload(
    project_root: &Path,
    stream_id: &str,
) -> Result<SliceAddedPrerequisite, String> {
    let event_directory = project_root.join(EVENT_EXPORT_DIRECTORY);
    for entry in fs::read_dir(event_directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let event = ExportedEvent::from_json_str(&contents)?;
        if event.stream_id().as_ref() == stream_id
            && matches!(event.body(), ExportedEventBody::SliceAdded { .. })
        {
            return SliceAddedPrerequisite::from_exported_event(&event);
        }
    }
    Err(format!("{stream_id} requires exported SliceAdded event"))
}

fn migrate_eventcore_sqlite_store(sqlite_path: &Path) -> Result<SqliteEventStore, String> {
    if let Some(parent) = sqlite_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let store = SqliteEventStore::new(SqliteConfig {
        path: sqlite_path.to_path_buf(),
        encryption_key: None,
    })
    .map_err(|error| error.to_string())?;
    Builder::new_current_thread()
        .build()
        .map_err(|error| error.to_string())?
        .block_on(store.migrate())
        .map_err(|error| error.to_string())?;
    Ok(store)
}

fn sync_exported_events_into_sqlite(project_root: &Path, sqlite_path: &Path) -> Result<(), String> {
    let event_directory = project_root.join(EVENT_EXPORT_DIRECTORY);
    if !event_directory.exists() {
        return Ok(());
    }

    let mut events = fs::read_dir(&event_directory)
        .map_err(|error| error.to_string())?
        .map(|entry| {
            let path = entry.map_err(|error| error.to_string())?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                return Ok(None);
            }
            let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
            let event = ExportedEvent::from_json_str(&contents)?;
            Ok(Some(event))
        })
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, String>>()?;
    events.sort_by(|left, right| left.event_id().cmp(right.event_id()));

    let mut conn = rusqlite::Connection::open(sqlite_path).map_err(|error| error.to_string())?;
    let eventcore_streams = existing_eventcore_streams(&conn).map_err(|error| error.to_string())?;
    let mut stream_versions = existing_stream_versions(&conn).map_err(|error| error.to_string())?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    for event in events {
        let stream_id = event.stream_id().as_ref().to_owned();
        if eventcore_streams.contains_key(&stream_id) {
            continue;
        }
        let version = stream_versions.entry(stream_id.clone()).or_insert(0);
        *version += 1;
        tx.execute(
            "INSERT OR IGNORE INTO eventcore_events
             (event_id, stream_id, stream_version, event_type, event_data, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, '{}')",
            params![
                event.event_id().as_ref(),
                stream_id,
                *version,
                event.event_type().as_ref(),
                event.to_json_string()?,
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())
}

fn existing_eventcore_streams(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<BTreeMap<String, usize>> {
    let mut statement = conn.prepare(
        "SELECT stream_id, count(*) FROM eventcore_events WHERE event_type = 'EmcEvent' GROUP BY stream_id",
    )?;
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

fn existing_stream_versions(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<BTreeMap<String, usize>> {
    let mut statement = conn.prepare(
        "SELECT stream_id, max(stream_version) FROM eventcore_events GROUP BY stream_id",
    )?;
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

fn project_realpath_hash(project_root: &Path) -> Result<String, String> {
    let canonical = fs::canonicalize(project_root).map_err(|error| error.to_string())?;
    Ok(hex::encode(Sha256::digest(
        canonical.to_string_lossy().as_bytes(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_added_prerequisite_parses_exported_payload_into_semantic_fields()
    -> Result<(), String> {
        let event = ExportedEvent::from_json_str(
            r#"{
                "schema_version": "emc.events.v1",
                "event_id": "workflow-added-1",
                "command_id": "workflow-added-1",
                "command_ordinal": 0,
                "stream_id": "workflow::open-ticket",
                "parents": [],
                "type": "WorkflowAdded",
                "payload": {
                    "slug": "open-ticket",
                    "name": "Open ticket",
                    "description": "Actor opens a repair ticket."
                }
            }
        "#,
        )?;

        let prerequisite = WorkflowAddedPrerequisite::from_exported_event(&event)?;

        assert_eq!(prerequisite.slug.as_ref(), "open-ticket");
        assert_eq!(prerequisite.name.as_ref(), "Open ticket");
        assert_eq!(
            prerequisite.description.as_ref(),
            "Actor opens a repair ticket."
        );

        Ok(())
    }

    #[test]
    fn slice_added_prerequisite_parses_exported_payload_into_semantic_fields() -> Result<(), String>
    {
        let event = ExportedEvent::from_json_str(
            r#"{
                "schema_version": "emc.events.v1",
                "event_id": "slice-added-1",
                "command_id": "slice-added-1",
                "command_ordinal": 0,
                "stream_id": "slice::capture-ticket",
                "parents": [],
                "type": "SliceAdded",
                "payload": {
                    "workflow": "open-ticket",
                    "slug": "capture-ticket",
                    "name": "Capture ticket",
                    "kind": "state_view",
                    "description": "Actor captures repair ticket details."
                }
            }
        "#,
        )?;

        let prerequisite = SliceAddedPrerequisite::from_exported_event(&event)?;

        assert_eq!(prerequisite.workflow.as_ref(), "open-ticket");
        assert_eq!(prerequisite.slug.as_ref(), "capture-ticket");
        assert_eq!(prerequisite.name.as_ref(), "Capture ticket");
        assert_eq!(prerequisite.kind, SliceKindName::StateView);
        assert_eq!(
            prerequisite.description.as_ref(),
            "Actor captures repair ticket details."
        );

        Ok(())
    }

    #[test]
    fn slice_added_prerequisite_rejects_unmodeled_slice_kinds() {
        let event = ExportedEvent::from_json_str(
            r#"{
                "schema_version": "emc.events.v1",
                "event_id": "slice-added-1",
                "command_id": "slice-added-1",
                "command_ordinal": 0,
                "stream_id": "slice::capture-ticket",
                "parents": [],
                "type": "SliceAdded",
                "payload": {
                    "workflow": "open-ticket",
                    "slug": "capture-ticket",
                    "name": "Capture ticket",
                    "kind": "screen",
                    "description": "Actor captures repair ticket details."
                }
            }
        }"#,
        );

        assert!(
            event.is_err(),
            "unmodeled exported slice kinds must not enter runtime repair events"
        );
    }
}
