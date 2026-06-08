// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::ffi::OsString;
    use std::path::Path;

    use super::super::{
        execute_eventcore_command_for_exported_event_at_path_for_test,
        sqlite_event_store_path_with_env,
    };
    use crate::core::connection::{ConnectionKind, WorkflowConnection, WorkflowTransitionRemoval};
    use crate::core::effect::{
        ArtifactDigest, ModelContentDigest, ProjectionFingerprint, ReviewEventReference,
    };
    use crate::core::event_commands::{
        AddSliceCommand, AddWorkflowCommand, ConnectWorkflowCommand,
        DeclareWorkflowReadinessCommand, InitializeProjectCommand, RemoveSliceCommand,
        RemoveWorkflowCommand, RemoveWorkflowTransitionCommand, UpdateSliceCommand,
        UpdateWorkflowCommand,
    };
    use crate::core::events::EventDraft;
    use crate::core::project::ProjectName;
    use crate::core::types::{
        ModelDescription, ModelName, ReviewTimestamp, ReviewerId, SliceKindName, SliceSlug,
        TransitionTriggerName, WorkflowSlug,
    };
    use crate::core::workflow::NewWorkflow;
    use eventcore::{RetryPolicy, execute};
    use eventcore_sqlite::{SqliteConfig, SqliteEventStore, rusqlite};
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;
    use tokio::runtime::Builder;

    #[test]
    fn sqlite_store_defaults_under_xdg_state_home_by_project_realpath_hash()
    -> Result<(), Box<dyn Error>> {
        let state_home = TempDir::new()?;
        let project = TempDir::new()?;
        let canonical_project = project.path().canonicalize()?;
        let project_hash = hex::encode(Sha256::digest(
            canonical_project.to_string_lossy().as_bytes(),
        ));

        let path = sqlite_event_store_path_with_env(project.path(), |name| {
            (name == "XDG_STATE_HOME").then(|| state_home.path().as_os_str().to_os_string())
        })?;

        assert_eq!(
            path,
            state_home
                .path()
                .join("emc")
                .join("projects")
                .join(project_hash)
                .join("events.sqlite3")
        );
        assert!(
            !path.starts_with(project.path()),
            "default SQLite event store must live outside the project repository"
        );

        Ok(())
    }

    #[test]
    fn sqlite_store_path_honors_env_override() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        let override_path = Path::new("/tmp/custom-emc-events.sqlite3");

        let path = sqlite_event_store_path_with_env(project.path(), |name| {
            (name == "EMC_EVENT_STORE_PATH").then(|| OsString::from(override_path))
        })?;

        assert_eq!(path, override_path);

        Ok(())
    }

    #[test]
    fn initialize_project_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                InitializeProjectCommand::from_semantic(model_project_name("Repair Desk")?)?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "project");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"ProjectInitialized\""),
            "eventcore command must append a ProjectInitialized event"
        );
        assert!(
            event_data.contains("\"Repair Desk\""),
            "eventcore command must persist the project name"
        );

        Ok(())
    }

    #[test]
    fn add_workflow_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_open_ticket_workflow_command()?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "workflow::open-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"WorkflowAdded\""),
            "eventcore command must append a WorkflowAdded event"
        );
        assert!(
            event_data.contains("\"Open ticket\""),
            "eventcore command must persist the workflow name"
        );
        assert!(
            event_data.contains("\"Actor opens a repair ticket.\""),
            "eventcore command must persist the workflow description"
        );

        Ok(())
    }

    #[test]
    fn exported_workflow_added_event_replay_uses_semantic_draft_body() -> Result<(), Box<dyn Error>>
    {
        let project = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let workflow = NewWorkflow::new(
            ModelName::try_new("Open ticket".to_owned())?,
            ModelDescription::try_new("Actor opens a repair ticket.".to_owned())?,
            WorkflowSlug::try_new("open-ticket".to_owned())?,
        );
        let draft = EventDraft::workflow_added(&workflow);

        execute_eventcore_command_for_exported_event_at_path_for_test(
            project.path(),
            &sqlite_path,
            &draft,
        )?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "workflow::open-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"WorkflowAdded\""),
            "replaying an exported event draft must append the workflow-added event"
        );
        assert!(
            event_data.contains("\"Open ticket\""),
            "replay must preserve the semantic workflow name"
        );
        assert!(
            event_data.contains("\"Actor opens a repair ticket.\""),
            "replay must preserve the semantic workflow description"
        );

        Ok(())
    }

    #[test]
    fn update_workflow_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_open_ticket_workflow_command()?,
                RetryPolicy::new(),
            )
            .await?;
            execute(
                &store,
                UpdateWorkflowCommand::from_semantic(
                    workflow_slug("open-ticket")?,
                    model_name("Open repair ticket")?,
                    model_description("Actor opens a repair ticket with priority.")?,
                )?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events
             WHERE stream_version = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "workflow::open-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"WorkflowUpdated\""),
            "eventcore command must append a WorkflowUpdated event"
        );
        assert!(
            event_data.contains("\"Open repair ticket\""),
            "eventcore command must persist the updated workflow name"
        );
        assert!(
            event_data.contains("\"Actor opens a repair ticket with priority.\""),
            "eventcore command must persist the updated workflow description"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_open_ticket_workflow_command()?,
                RetryPolicy::new(),
            )
            .await?;
            execute(
                &store,
                RemoveWorkflowCommand::from_semantic(workflow_slug("open-ticket")?)?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events
             WHERE stream_version = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "workflow::open-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"WorkflowRemoved\""),
            "eventcore command must append a WorkflowRemoved event"
        );
        assert!(
            event_data.contains("\"open-ticket\""),
            "eventcore command must persist the removed workflow slug"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_open_ticket_workflow_command()?,
                RetryPolicy::new(),
            )
            .await?;
            execute(
                &store,
                ConnectWorkflowCommand::from_connection(WorkflowConnection::new(
                    WorkflowSlug::try_new("open-ticket".to_owned())?,
                    SliceSlug::try_new("capture-ticket".to_owned())?,
                    SliceSlug::try_new("review-ticket".to_owned())?,
                    ConnectionKind::Navigation,
                    TransitionTriggerName::try_new("review-ticket-screen".to_owned())?,
                ))?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events
             WHERE stream_version = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "workflow::open-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"WorkflowConnected\""),
            "eventcore command must append a WorkflowConnected event"
        );
        assert!(
            event_data.contains("\"review-ticket-screen\""),
            "eventcore command must persist the connection trigger"
        );

        Ok(())
    }

    #[test]
    fn remove_workflow_transition_command_appends_eventcore_sqlite_event()
    -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_open_ticket_workflow_command()?,
                RetryPolicy::new(),
            )
            .await?;
            execute(
                &store,
                RemoveWorkflowTransitionCommand::from_removal(WorkflowTransitionRemoval::new(
                    WorkflowSlug::try_new("open-ticket".to_owned())?,
                    SliceSlug::try_new("capture-ticket".to_owned())?,
                    SliceSlug::try_new("review-ticket".to_owned())?,
                    ConnectionKind::Navigation,
                    TransitionTriggerName::try_new("review-ticket-screen".to_owned())?,
                ))?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events
             WHERE stream_version = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "workflow::open-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"WorkflowTransitionRemoved\""),
            "eventcore command must append a WorkflowTransitionRemoved event"
        );
        assert!(
            event_data.contains("\"review-ticket-screen\""),
            "eventcore command must persist the removed transition trigger"
        );

        Ok(())
    }

    #[test]
    fn declare_workflow_readiness_command_appends_eventcore_sqlite_event()
    -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_open_ticket_workflow_command()?,
                RetryPolicy::new(),
            )
            .await?;
            execute(
                &store,
                DeclareWorkflowReadinessCommand::new(
                    WorkflowSlug::try_new("open-ticket".to_owned())?,
                    ProjectionFingerprint::new(ArtifactDigest::try_new(
                        "verified-frontier".to_owned(),
                    )?),
                    ModelContentDigest::new(ArtifactDigest::try_new("model-content".to_owned())?),
                    ReviewTimestamp::try_new("2026-06-07T00:00:00.000Z".to_owned())?,
                    ReviewerId::try_new("emc verify".to_owned())?,
                    ReviewEventReference::unrecorded(),
                )?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events
             WHERE stream_version = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "workflow::open-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"WorkflowReadinessDeclared\""),
            "eventcore command must append a WorkflowReadinessDeclared event"
        );
        assert!(
            event_data.contains("\"verified-frontier\""),
            "eventcore command must persist the verified event frontier"
        );

        Ok(())
    }

    #[test]
    fn add_slice_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_capture_ticket_slice_command()?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "slice::capture-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"SliceAdded\""),
            "eventcore command must append a SliceAdded event"
        );
        assert!(
            event_data.contains("\"open-ticket\""),
            "eventcore command must persist the owning workflow"
        );
        assert!(
            event_data.contains("\"state_view\""),
            "eventcore command must persist the slice type"
        );
        assert!(
            event_data.contains("\"Actor enters ticket details.\""),
            "eventcore command must persist the slice description"
        );

        Ok(())
    }

    #[test]
    fn update_slice_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_capture_ticket_slice_command()?,
                RetryPolicy::new(),
            )
            .await?;
            execute(
                &store,
                UpdateSliceCommand::from_semantic(
                    slice_slug("capture-ticket")?,
                    model_name("Capture repair ticket")?,
                    slice_kind_name("state_change")?,
                    model_description("Actor enters prioritized ticket details.")?,
                )?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events
             WHERE stream_version = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "slice::capture-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"SliceUpdated\""),
            "eventcore command must append a SliceUpdated event"
        );
        assert!(
            event_data.contains("\"state_change\""),
            "eventcore command must persist the updated slice type"
        );
        assert!(
            event_data.contains("\"Actor enters prioritized ticket details.\""),
            "eventcore command must persist the updated slice description"
        );

        Ok(())
    }

    #[test]
    fn remove_slice_command_appends_eventcore_sqlite_event() -> Result<(), Box<dyn Error>> {
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let store = SqliteEventStore::new(SqliteConfig {
            path: sqlite_path.clone(),
            encryption_key: None,
        })?;

        Builder::new_current_thread().build()?.block_on(async {
            store.migrate().await?;
            execute(
                &store,
                add_capture_ticket_slice_command()?,
                RetryPolicy::new(),
            )
            .await?;
            execute(
                &store,
                RemoveSliceCommand::from_semantic(slice_slug("capture-ticket")?)?,
                RetryPolicy::new(),
            )
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })?;

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let (stream_id, event_type, event_data): (String, String, String) = conn.query_row(
            "SELECT stream_id, event_type, event_data FROM eventcore_events
             WHERE stream_version = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(stream_id, "slice::capture-ticket");
        assert_eq!(event_type, "EmcEvent");
        assert!(
            event_data.contains("\"SliceRemoved\""),
            "eventcore command must append a SliceRemoved event"
        );
        assert!(
            event_data.contains("\"capture-ticket\""),
            "eventcore command must persist the removed slice slug"
        );

        Ok(())
    }

    fn add_open_ticket_workflow_command() -> Result<AddWorkflowCommand, String> {
        AddWorkflowCommand::from_semantic(
            workflow_slug("open-ticket")?,
            model_name("Open ticket")?,
            model_description("Actor opens a repair ticket.")?,
        )
    }

    fn add_capture_ticket_slice_command() -> Result<AddSliceCommand, String> {
        AddSliceCommand::from_semantic(
            workflow_slug("open-ticket")?,
            slice_slug("capture-ticket")?,
            model_name("Capture ticket")?,
            slice_kind_name("state_view")?,
            model_description("Actor enters ticket details.")?,
        )
    }

    fn model_project_name(value: &str) -> Result<ProjectName, String> {
        ProjectName::try_new(value.to_owned()).map_err(|error| error.to_string())
    }

    fn model_name(value: &str) -> Result<ModelName, String> {
        ModelName::try_new(value.to_owned()).map_err(|error| error.to_string())
    }

    fn model_description(value: &str) -> Result<ModelDescription, String> {
        ModelDescription::try_new(value.to_owned()).map_err(|error| error.to_string())
    }

    fn workflow_slug(value: &str) -> Result<WorkflowSlug, String> {
        WorkflowSlug::try_new(value.to_owned()).map_err(|error| error.to_string())
    }

    fn slice_slug(value: &str) -> Result<SliceSlug, String> {
        SliceSlug::try_new(value.to_owned()).map_err(|error| error.to_string())
    }

    fn slice_kind_name(value: &str) -> Result<SliceKindName, String> {
        SliceKindName::try_new(value.to_owned()).map_err(|error| error.to_string())
    }
}
