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
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::runtime::Builder;

use crate::core::effect::ArtifactDigest;
use crate::core::event_commands::{
    AddSliceCommand, AddSliceFactCommand, AddWorkflowCommand, AddWorkflowFactCommand,
    ConnectWorkflowCommand, ConnectWorkflowInput, DeclareWorkflowReadinessCommand,
    InitializeProjectCommand, RecordReviewCommand, RemoveSliceCommand, RemoveWorkflowCommand,
    RemoveWorkflowTransitionCommand, RemoveWorkflowTransitionInput, ResolveConflictCommand,
    SliceFactInput, UpdateSliceCommand, UpdateWorkflowCommand, WorkflowFactInput,
};
use crate::core::events::{
    EventDraft, slice_automation_from_payload, slice_bit_level_data_flow_from_payload,
    slice_board_connection_from_payload, slice_board_element_from_payload,
    slice_command_definition_from_payload, slice_event_definition_from_payload,
    slice_external_payload_from_payload, slice_outcome_from_payload, slice_read_model_from_payload,
    slice_scenario_from_payload, slice_translation_from_payload, slice_view_from_payload,
};
use crate::core::types::{
    CommandErrorName, CommandName, ModelDescription, OutcomeLabelName, ReviewTimestamp, ReviewerId,
    StreamName, TransitionTriggerName, WorkflowCommandErrorRecord,
    WorkflowEntryLifecycleEvidenceText, WorkflowEntryLifecycleStateName,
    WorkflowEntryLifecycleStateRecord, WorkflowEventParticipation, WorkflowOutcomeRecord,
    WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord,
    WorkflowSlug, WorkflowTransitionEndpoint, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionEvidenceText, WorkflowTransitionKind, WorkflowViewRole,
};

const EVENT_EXPORT_DIRECTORY: &str = "model/events/v1";

const EVENT_STORE_PATH_ENV: &str = "EMC_EVENT_STORE_PATH";
const XDG_STATE_HOME_ENV: &str = "XDG_STATE_HOME";

pub fn sqlite_event_store_path(project_root: &Path) -> Result<PathBuf, String> {
    sqlite_event_store_path_with_env(project_root, |name| env::var_os(name))
}

pub fn sqlite_event_store_path_with_env(
    project_root: &Path,
    env_var: impl Fn(&str) -> Option<OsString>,
) -> Result<PathBuf, String> {
    if let Some(path) = env_var(EVENT_STORE_PATH_ENV).filter(|path| !path.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    let state_home = env_var(XDG_STATE_HOME_ENV)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| format!("{XDG_STATE_HOME_ENV} is required to resolve event store path"))?;
    let project_hash = project_realpath_hash(project_root)?;

    Ok(state_home
        .join("emc")
        .join("projects")
        .join(project_hash)
        .join("events.sqlite3"))
}

pub struct ProjectRuntimeLock {
    file: File,
}

impl Drop for ProjectRuntimeLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub fn lock_project_runtime(project_root: &Path) -> Result<ProjectRuntimeLock, String> {
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

pub fn ensure_sqlite_event_store(project_root: &Path) -> Result<PathBuf, String> {
    let path = sqlite_event_store_path(project_root)?;
    migrate_eventcore_sqlite_store(&path)?;
    sync_exported_events_into_sqlite(project_root, &path)?;
    Ok(path)
}

pub fn execute_eventcore_command_for_exported_event(
    project_root: &Path,
    draft: &EventDraft,
) -> Result<(), String> {
    let path = sqlite_event_store_path(project_root)?;
    repair_project_stream_if_needed(&path, draft)?;
    let workflow_added_prerequisite =
        workflow_added_prerequisite_if_stream_needs_repair(project_root, &path, draft)?;
    let slice_added_prerequisite =
        slice_added_prerequisite_if_stream_needs_repair(project_root, &path, draft)?;
    let store = migrate_eventcore_sqlite_store(&path)?;
    Builder::new_current_thread()
        .build()
        .map_err(|error| error.to_string())?
        .block_on(async {
            if let Some((slug, name, description)) = workflow_added_prerequisite {
                execute(
                    &store,
                    AddWorkflowCommand::new(slug, name, description)?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            if let Some((workflow, slug, name, kind, description)) = slice_added_prerequisite {
                execute(
                    &store,
                    AddSliceCommand::new(workflow, slug, name, kind, description)?,
                    RetryPolicy::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
            }
            match draft.event_type() {
                "ProjectInitialized" => {
                    execute(
                        &store,
                        InitializeProjectCommand::new(required_payload_str(draft, "name")?)?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowAdded" => {
                    execute(
                        &store,
                        AddWorkflowCommand::new(
                            required_payload_str(draft, "slug")?,
                            required_payload_str(draft, "name")?,
                            required_payload_str(draft, "description")?,
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowUpdated" => {
                    execute(
                        &store,
                        UpdateWorkflowCommand::new(
                            required_payload_str(draft, "slug")?,
                            required_payload_str(draft, "name")?,
                            required_payload_str(draft, "description")?,
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowRemoved" => {
                    execute(
                        &store,
                        RemoveWorkflowCommand::new(required_payload_str(draft, "slug")?)?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowOutcomeAdded" => {
                    execute(
                        &store,
                        AddWorkflowFactCommand::new(WorkflowFactInput::OutcomeAdded {
                            workflow: workflow_slug_payload(draft, "workflow")?,
                            outcome: workflow_outcome_payload(draft)?,
                        })?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowCommandErrorAdded" => {
                    execute(
                        &store,
                        AddWorkflowFactCommand::new(WorkflowFactInput::CommandErrorAdded {
                            workflow: workflow_slug_payload(draft, "workflow")?,
                            error: workflow_command_error_payload(draft)?,
                        })?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowOwnedDefinitionAdded" => {
                    execute(
                        &store,
                        AddWorkflowFactCommand::new(WorkflowFactInput::OwnedDefinitionAdded {
                            workflow: workflow_slug_payload(draft, "workflow")?,
                            definition: workflow_owned_definition_payload(draft)?,
                        })?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowTransitionEvidenceAdded" => {
                    execute(
                        &store,
                        AddWorkflowFactCommand::new(WorkflowFactInput::TransitionEvidenceAdded {
                            workflow: workflow_slug_payload(draft, "workflow")?,
                            evidence: workflow_transition_evidence_payload(draft)?,
                        })?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowEntryLifecycleCoverageRequired" => {
                    execute(
                        &store,
                        AddWorkflowFactCommand::new(
                            WorkflowFactInput::EntryLifecycleCoverageRequired {
                                workflow: workflow_slug_payload(draft, "workflow")?,
                            },
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowEntryLifecycleStateAdded" => {
                    execute(
                        &store,
                        AddWorkflowFactCommand::new(WorkflowFactInput::EntryLifecycleStateAdded {
                            workflow: workflow_slug_payload(draft, "workflow")?,
                            state: workflow_entry_lifecycle_state_payload(draft)?,
                        })?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowReadinessDeclared" => {
                    execute(
                        &store,
                        DeclareWorkflowReadinessCommand::new(
                            workflow_slug_payload(draft, "workflow")?,
                            artifact_digest_payload(draft, "projection_fingerprint")?,
                            artifact_digest_payload(draft, "model_content_digest")?,
                            review_timestamp_payload(draft, "verified_at")?,
                            reviewer_id_payload(draft, "verified_by")?,
                            optional_artifact_digest_payload(draft, "review_event_id")?,
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowConnected" => {
                    execute(
                        &store,
                        ConnectWorkflowCommand::new(ConnectWorkflowInput {
                            workflow: required_payload_str(draft, "workflow")?,
                            source: required_payload_str(draft, "from")?,
                            target_slice: optional_payload_str(draft, "to"),
                            target_workflow: optional_payload_str(draft, "to_workflow"),
                            via: required_payload_str(draft, "via")?,
                            name: required_payload_str(draft, "name")?,
                            payload_contract: optional_payload_str(draft, "payload_contract"),
                            reason: optional_payload_str(draft, "reason"),
                        })?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "WorkflowTransitionRemoved" => {
                    execute(
                        &store,
                        RemoveWorkflowTransitionCommand::new(RemoveWorkflowTransitionInput {
                            workflow: required_payload_str(draft, "workflow")?,
                            source: required_payload_str(draft, "from")?,
                            target_slice: optional_payload_str(draft, "to"),
                            target_workflow: optional_payload_str(draft, "to_workflow"),
                            via: required_payload_str(draft, "via")?,
                            name: required_payload_str(draft, "name")?,
                        })?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "SliceAdded" => {
                    execute(
                        &store,
                        AddSliceCommand::new(
                            required_payload_str(draft, "workflow")?,
                            required_payload_str(draft, "slug")?,
                            required_payload_str(draft, "name")?,
                            required_payload_str(draft, "kind")?,
                            required_payload_str(draft, "description")?,
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "SliceUpdated" => {
                    execute(
                        &store,
                        UpdateSliceCommand::new(
                            required_payload_str(draft, "slug")?,
                            required_payload_str(draft, "name")?,
                            required_payload_str(draft, "kind")?,
                            required_payload_str(draft, "description")?,
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "SliceRemoved" => {
                    execute(
                        &store,
                        RemoveSliceCommand::new(required_payload_str(draft, "slug")?)?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                event_type if event_type_is_slice_fact(event_type) => {
                    execute(
                        &store,
                        AddSliceFactCommand::new(slice_fact_input(draft)?)?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "ReviewRecorded" => {
                    execute(
                        &store,
                        RecordReviewCommand::new(
                            required_payload_str(draft, "workflow")?,
                            required_payload_str(draft, "model_content_digest")?,
                            required_payload_str(draft, "reviewer_id")?,
                            required_payload_str(draft, "reviewed_at")?,
                            required_payload_string_array(draft, "categories")?,
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                "ConflictResolved" => {
                    execute(
                        &store,
                        ResolveConflictCommand::new(
                            required_payload_str(draft, "conflict_id")?,
                            required_payload_str(draft, "chosen_event_id")?,
                        )?,
                        RetryPolicy::new(),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                _ => {}
            }
            Ok::<(), String>(())
        })
}

fn repair_project_stream_if_needed(sqlite_path: &Path, draft: &EventDraft) -> Result<(), String> {
    if draft.event_type() != "ConflictResolved" {
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
) -> Result<Option<(String, String, String)>, String> {
    if !matches!(
        draft.event_type(),
        "WorkflowUpdated"
            | "WorkflowRemoved"
            | "WorkflowConnected"
            | "WorkflowTransitionRemoved"
            | "WorkflowOutcomeAdded"
            | "WorkflowCommandErrorAdded"
            | "WorkflowOwnedDefinitionAdded"
            | "WorkflowTransitionEvidenceAdded"
            | "WorkflowEntryLifecycleCoverageRequired"
            | "WorkflowEntryLifecycleStateAdded"
            | "WorkflowReadinessDeclared"
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

type SliceAddedPayload = (String, String, String, String, String);

fn slice_added_prerequisite_if_stream_needs_repair(
    project_root: &Path,
    sqlite_path: &Path,
    draft: &EventDraft,
) -> Result<Option<SliceAddedPayload>, String> {
    if !matches!(draft.event_type(), "SliceUpdated" | "SliceRemoved")
        && !event_type_is_slice_fact(draft.event_type())
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

fn workflow_added_payload(
    project_root: &Path,
    stream_id: &str,
) -> Result<(String, String, String), String> {
    let event_directory = project_root.join(EVENT_EXPORT_DIRECTORY);
    for entry in fs::read_dir(event_directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let event: Value = serde_json::from_str(&contents).map_err(|error| error.to_string())?;
        if required_event_str(&event, "stream_id") == stream_id
            && required_event_str(&event, "type") == "WorkflowAdded"
        {
            return Ok((
                required_json_payload_str(&event, "slug")?,
                required_json_payload_str(&event, "name")?,
                required_json_payload_str(&event, "description")?,
            ));
        }
    }
    Err(format!("{stream_id} requires exported WorkflowAdded event"))
}

fn slice_added_payload(project_root: &Path, stream_id: &str) -> Result<SliceAddedPayload, String> {
    let event_directory = project_root.join(EVENT_EXPORT_DIRECTORY);
    for entry in fs::read_dir(event_directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let event: Value = serde_json::from_str(&contents).map_err(|error| error.to_string())?;
        if required_event_str(&event, "stream_id") == stream_id
            && required_event_str(&event, "type") == "SliceAdded"
        {
            return Ok((
                required_json_payload_str(&event, "workflow")?,
                required_json_payload_str(&event, "slug")?,
                required_json_payload_str(&event, "name")?,
                required_json_payload_str(&event, "kind")?,
                required_json_payload_str(&event, "description")?,
            ));
        }
    }
    Err(format!("{stream_id} requires exported SliceAdded event"))
}

fn required_json_payload_str(event: &Value, field: &str) -> Result<String, String> {
    event
        .get("payload")
        .and_then(|payload| payload.get(field))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("exported event payload requires {field}"))
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

fn required_payload_str(draft: &EventDraft, field: &str) -> Result<String, String> {
    draft
        .payload()
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("{} payload requires {field}", draft.event_type()))
}

fn required_payload_bool(draft: &EventDraft, field: &str) -> Result<bool, String> {
    draft
        .payload()
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{} payload requires {field}", draft.event_type()))
}

fn required_payload_string_array(draft: &EventDraft, field: &str) -> Result<Vec<String>, String> {
    draft
        .payload()
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} payload requires {field}", draft.event_type()))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{} payload requires string {field}", draft.event_type()))
        })
        .collect()
}

fn optional_payload_str(draft: &EventDraft, field: &str) -> Option<String> {
    draft
        .payload()
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn workflow_slug_payload(draft: &EventDraft, field: &str) -> Result<WorkflowSlug, String> {
    WorkflowSlug::try_new(required_payload_str(draft, field)?).map_err(|error| error.to_string())
}

fn artifact_digest_payload(draft: &EventDraft, field: &str) -> Result<ArtifactDigest, String> {
    ArtifactDigest::try_new(required_payload_str(draft, field)?).map_err(|error| error.to_string())
}

fn optional_artifact_digest_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<Option<ArtifactDigest>, String> {
    optional_payload_str(draft, field)
        .map(ArtifactDigest::try_new)
        .transpose()
        .map_err(|error| error.to_string())
}

fn review_timestamp_payload(draft: &EventDraft, field: &str) -> Result<ReviewTimestamp, String> {
    ReviewTimestamp::try_new(required_payload_str(draft, field)?).map_err(|error| error.to_string())
}

fn reviewer_id_payload(draft: &EventDraft, field: &str) -> Result<ReviewerId, String> {
    ReviewerId::try_new(required_payload_str(draft, field)?).map_err(|error| error.to_string())
}

fn slice_fact_input(draft: &EventDraft) -> Result<SliceFactInput, String> {
    match draft.event_type() {
        "SliceScenarioAdded" => {
            slice_scenario_from_payload(draft.payload()).map(SliceFactInput::Scenario)
        }
        "SliceOutcomeAdded" => {
            slice_outcome_from_payload(draft.payload()).map(SliceFactInput::Outcome)
        }
        "SliceExternalPayloadAdded" => slice_external_payload_from_payload(draft.payload())
            .map(SliceFactInput::ExternalPayload),
        "SliceEventDefinitionAdded" => slice_event_definition_from_payload(draft.payload())
            .map(SliceFactInput::EventDefinition),
        "SliceCommandDefinitionAdded" => slice_command_definition_from_payload(draft.payload())
            .map(SliceFactInput::CommandDefinition),
        "SliceReadModelAdded" => {
            slice_read_model_from_payload(draft.payload()).map(SliceFactInput::ReadModel)
        }
        "SliceViewAdded" => slice_view_from_payload(draft.payload()).map(SliceFactInput::View),
        "SliceBitLevelDataFlowAdded" => slice_bit_level_data_flow_from_payload(draft.payload())
            .map(SliceFactInput::BitLevelDataFlow),
        "SliceTranslationAdded" => {
            slice_translation_from_payload(draft.payload()).map(SliceFactInput::Translation)
        }
        "SliceAutomationAdded" => {
            slice_automation_from_payload(draft.payload()).map(SliceFactInput::Automation)
        }
        "SliceBoardElementAdded" => {
            slice_board_element_from_payload(draft.payload()).map(SliceFactInput::BoardElement)
        }
        "SliceBoardConnectionAdded" => slice_board_connection_from_payload(draft.payload())
            .map(SliceFactInput::BoardConnection),
        event_type => Err(format!("unsupported slice fact event type {event_type}")),
    }
}

fn workflow_outcome_payload(draft: &EventDraft) -> Result<WorkflowOutcomeRecord, String> {
    Ok(WorkflowOutcomeRecord::new(
        workflow_transition_endpoint_payload(draft, "source_slice")?,
        outcome_label_payload(draft, "label")?,
        required_payload_bool(draft, "externally_relevant")?,
    ))
}

fn workflow_command_error_payload(
    draft: &EventDraft,
) -> Result<WorkflowCommandErrorRecord, String> {
    Ok(WorkflowCommandErrorRecord::new(
        workflow_transition_endpoint_payload(draft, "source_slice")?,
        command_name_payload(draft, "command")?,
        command_error_name_payload(draft, "error")?,
    ))
}

fn workflow_owned_definition_payload(
    draft: &EventDraft,
) -> Result<WorkflowOwnedDefinitionRecord, String> {
    let source_slice = workflow_transition_endpoint_payload(draft, "source_slice")?;
    let definition_kind = workflow_owned_definition_kind_payload(draft, "definition_kind")?;
    let definition_name = workflow_owned_definition_name_payload(draft, "definition_name")?;
    let definition_stream = optional_payload_str(draft, "definition_stream")
        .map(StreamName::try_new)
        .transpose()
        .map_err(|error| error.to_string())?;
    let source_provenance = optional_payload_str(draft, "source_provenance")
        .map(ModelDescription::try_new)
        .transpose()
        .map_err(|error| error.to_string())?;
    let event_participation = optional_payload_str(draft, "event_participation")
        .map(WorkflowEventParticipation::try_new)
        .transpose()
        .map_err(|error| error.to_string())?;
    let view_role = optional_payload_str(draft, "view_role")
        .map(WorkflowViewRole::try_new)
        .transpose()
        .map_err(|error| error.to_string())?;

    match (
        definition_stream,
        source_provenance,
        event_participation,
        view_role,
    ) {
        (None, None, None, None) => Ok(WorkflowOwnedDefinitionRecord::new(
            source_slice,
            definition_kind,
            definition_name,
        )),
        (None, None, None, Some(view_role)) => WorkflowOwnedDefinitionRecord::new_with_view_role(
            source_slice,
            definition_kind,
            definition_name,
            view_role,
        )
        .ok_or_else(|| "view role requires a view owned-definition kind".to_owned()),
        (Some(definition_stream), Some(source_provenance), None, None) => {
            Ok(WorkflowOwnedDefinitionRecord::new_with_event_identity(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
            ))
        }
        (Some(definition_stream), Some(source_provenance), Some(event_participation), None) => Ok(
            WorkflowOwnedDefinitionRecord::new_with_event_identity_and_participation(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
                event_participation,
            ),
        ),
        _ => Err("WorkflowOwnedDefinitionAdded has incompatible optional fields".to_owned()),
    }
}

fn workflow_transition_evidence_payload(
    draft: &EventDraft,
) -> Result<WorkflowTransitionEvidenceRecord, String> {
    Ok(WorkflowTransitionEvidenceRecord::new(
        workflow_transition_endpoint_payload(draft, "from")?,
        workflow_transition_endpoint_payload(draft, "to")?,
        workflow_transition_kind_payload(draft, "via")?,
        transition_trigger_payload(draft, "name")?,
        workflow_transition_evidence_text_payload(draft, "source_evidence")?,
        workflow_transition_evidence_text_payload(draft, "target_evidence")?,
    ))
}

fn workflow_entry_lifecycle_state_payload(
    draft: &EventDraft,
) -> Result<WorkflowEntryLifecycleStateRecord, String> {
    Ok(WorkflowEntryLifecycleStateRecord::new(
        workflow_entry_lifecycle_state_name_payload(draft, "state")?,
        workflow_transition_endpoint_payload(draft, "step")?,
        workflow_entry_lifecycle_evidence_text_payload(draft, "evidence")?,
    ))
}

fn workflow_transition_endpoint_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<WorkflowTransitionEndpoint, String> {
    WorkflowTransitionEndpoint::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn workflow_transition_kind_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<WorkflowTransitionKind, String> {
    WorkflowTransitionKind::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn transition_trigger_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<TransitionTriggerName, String> {
    TransitionTriggerName::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn outcome_label_payload(draft: &EventDraft, field: &str) -> Result<OutcomeLabelName, String> {
    OutcomeLabelName::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn command_name_payload(draft: &EventDraft, field: &str) -> Result<CommandName, String> {
    CommandName::try_new(required_payload_str(draft, field)?).map_err(|error| error.to_string())
}

fn command_error_name_payload(draft: &EventDraft, field: &str) -> Result<CommandErrorName, String> {
    CommandErrorName::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn workflow_owned_definition_kind_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<WorkflowOwnedDefinitionKind, String> {
    WorkflowOwnedDefinitionKind::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn workflow_owned_definition_name_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<WorkflowOwnedDefinitionName, String> {
    WorkflowOwnedDefinitionName::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn workflow_transition_evidence_text_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<WorkflowTransitionEvidenceText, String> {
    WorkflowTransitionEvidenceText::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn workflow_entry_lifecycle_state_name_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<WorkflowEntryLifecycleStateName, String> {
    WorkflowEntryLifecycleStateName::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
}

fn workflow_entry_lifecycle_evidence_text_payload(
    draft: &EventDraft,
    field: &str,
) -> Result<WorkflowEntryLifecycleEvidenceText, String> {
    WorkflowEntryLifecycleEvidenceText::try_new(required_payload_str(draft, field)?)
        .map_err(|error| error.to_string())
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
            let event: Value =
                serde_json::from_str(&contents).map_err(|error| error.to_string())?;
            Ok(Some(event))
        })
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, String>>()?;
    events.sort_by(|left, right| {
        required_event_str(left, "event_id").cmp(&required_event_str(right, "event_id"))
    });

    let mut conn = rusqlite::Connection::open(sqlite_path).map_err(|error| error.to_string())?;
    let eventcore_streams = existing_eventcore_streams(&conn).map_err(|error| error.to_string())?;
    let mut stream_versions = existing_stream_versions(&conn).map_err(|error| error.to_string())?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    for event in events {
        let stream_id = required_event_str(&event, "stream_id");
        if eventcore_streams.contains_key(&stream_id)
            && event_type_is_backed_by_eventcore(&required_event_str(&event, "type"))
        {
            continue;
        }
        let version = stream_versions.entry(stream_id.clone()).or_insert(0);
        *version += 1;
        tx.execute(
            "INSERT OR IGNORE INTO eventcore_events
             (event_id, stream_id, stream_version, event_type, event_data, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, '{}')",
            params![
                required_event_str(&event, "event_id"),
                stream_id,
                *version,
                required_event_str(&event, "type"),
                event.to_string(),
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())
}

fn event_type_is_backed_by_eventcore(event_type: &str) -> bool {
    matches!(
        event_type,
        "ProjectInitialized"
            | "WorkflowAdded"
            | "WorkflowUpdated"
            | "WorkflowRemoved"
            | "WorkflowOutcomeAdded"
            | "WorkflowCommandErrorAdded"
            | "WorkflowOwnedDefinitionAdded"
            | "WorkflowTransitionEvidenceAdded"
            | "WorkflowEntryLifecycleCoverageRequired"
            | "WorkflowEntryLifecycleStateAdded"
            | "WorkflowReadinessDeclared"
            | "WorkflowConnected"
            | "WorkflowTransitionRemoved"
            | "SliceAdded"
            | "SliceUpdated"
            | "SliceRemoved"
            | "ReviewRecorded"
            | "ConflictResolved"
    ) || event_type_is_slice_fact(event_type)
}

fn event_type_is_slice_fact(event_type: &str) -> bool {
    matches!(
        event_type,
        "SliceScenarioAdded"
            | "SliceOutcomeAdded"
            | "SliceExternalPayloadAdded"
            | "SliceEventDefinitionAdded"
            | "SliceCommandDefinitionAdded"
            | "SliceReadModelAdded"
            | "SliceViewAdded"
            | "SliceBitLevelDataFlowAdded"
            | "SliceTranslationAdded"
            | "SliceAutomationAdded"
            | "SliceBoardElementAdded"
            | "SliceBoardConnectionAdded"
    )
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

fn required_event_str(event: &Value, field: &str) -> String {
    event
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn project_realpath_hash(project_root: &Path) -> Result<String, String> {
    let canonical = fs::canonicalize(project_root).map_err(|error| error.to_string())?;
    Ok(hex::encode(Sha256::digest(
        canonical.to_string_lossy().as_bytes(),
    )))
}
