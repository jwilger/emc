// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::super::{
        event_store_root, execute_eventcore_command_for_exported_event, list_forks,
        read_all_emc_events, reconcile_choose_branch, transaction_id_string,
    };
    use crate::core::event_commands::EmcEvent;
    use crate::core::events::EventDraft;
    use crate::core::project::ProjectName;
    use crate::core::types::{ModelDescription, ModelName, WorkflowSlug};
    use crate::core::workflow::NewWorkflow;

    /// Copy every committed transaction file from one project's store into
    /// another's, mimicking a `git merge` of the `events/` directory. Only
    /// `events/*.jsonl` is committed, so this is the entire shared surface.
    fn union_committed_events(from: &Path, into: &Path) -> Result<(), Box<dyn Error>> {
        let source = event_store_root(from).join("events");
        let destination = event_store_root(into).join("events");
        fs::create_dir_all(&destination)?;
        for entry in fs::read_dir(source)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "jsonl")
            {
                let name = path.file_name().ok_or("transaction file has no name")?;
                fs::copy(&path, destination.join(name))?;
            }
        }
        Ok(())
    }

    fn project_name(value: &str) -> Result<ProjectName, Box<dyn Error>> {
        Ok(ProjectName::try_new(value.to_owned())?)
    }

    fn new_workflow(
        slug: &str,
        name: &str,
        description: &str,
    ) -> Result<NewWorkflow, Box<dyn Error>> {
        Ok(NewWorkflow::new(
            ModelName::try_new(name.to_owned())?,
            ModelDescription::try_new(description.to_owned())?,
            WorkflowSlug::try_new(slug.to_owned())?,
        ))
    }

    #[test]
    fn event_store_lives_inside_the_project_repository() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;

        let root = event_store_root(project.path());
        assert!(
            root.starts_with(project.path()),
            "the file event store must live in the project repository"
        );
        assert!(
            root.join("events").is_dir(),
            "the committed events/ directory must exist after the first append"
        );
        assert!(
            root.join(".gitignore").is_file(),
            "the store must write its own .gitignore"
        );
        Ok(())
    }

    #[test]
    fn executed_commands_are_appended_and_read_back_in_order() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_added(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        )?;

        let events = read_all_emc_events(project.path())?;
        assert!(
            matches!(events.first(), Some(EmcEvent::ProjectInitialized { .. })),
            "first event must be ProjectInitialized, got {events:?}"
        );
        assert!(
            matches!(
                events.get(1),
                Some(EmcEvent::WorkflowAdded { slug, .. }) if slug.as_ref() == "open-ticket"
            ),
            "second event must be the added workflow, got {events:?}"
        );
        Ok(())
    }

    #[test]
    fn reading_a_project_without_a_store_returns_no_events() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        assert!(read_all_emc_events(project.path())?.is_empty());
        Ok(())
    }

    #[test]
    fn updating_an_absent_workflow_is_rejected() -> Result<(), Box<dyn Error>> {
        let project = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;

        let result = execute_eventcore_command_for_exported_event(
            project.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        );

        assert!(
            result.is_err(),
            "updating a workflow that was never added must fail the command invariant"
        );
        Ok(())
    }

    #[test]
    fn concurrent_branches_are_reported_as_a_fork_and_reconciled_by_choosing_one()
    -> Result<(), Box<dyn Error>> {
        // Replica A builds the shared base: an initialized project with a
        // workflow.
        let replica_a = TempDir::new()?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::project_initialized(&project_name("Repairs")?),
        )?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::workflow_added(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Actor opens a repair ticket.",
            )?),
        )?;

        // Replica B starts from a clone of the committed events only (a fresh
        // replica id), then both replicas update the workflow concurrently from
        // the same base version.
        let replica_b = TempDir::new()?;
        union_committed_events(replica_a.path(), replica_b.path())?;
        execute_eventcore_command_for_exported_event(
            replica_b.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Description authored on replica B.",
            )?),
        )?;
        execute_eventcore_command_for_exported_event(
            replica_a.path(),
            &EventDraft::workflow_updated(&new_workflow(
                "open-ticket",
                "Open ticket",
                "Description authored on replica A.",
            )?),
        )?;

        // Merging replica B's events into replica A unions the two divergent
        // transactions onto one stream from the same base — a fork.
        union_committed_events(replica_b.path(), replica_a.path())?;
        let forks = list_forks(replica_a.path())?;
        assert_eq!(
            forks.len(),
            1,
            "the concurrent updates must surface one fork"
        );
        let fork = &forks[0];
        assert_eq!(fork.stream_id().as_ref(), "workflow::open-ticket");
        assert_eq!(
            fork.transactions().len(),
            2,
            "both branches must be present"
        );

        // Resolving keeps the chosen branch and collapses the fork.
        let chosen = transaction_id_string(fork.transactions()[0]);
        let resolved = reconcile_choose_branch(replica_a.path(), "workflow::open-ticket", &chosen)?;
        assert_eq!(resolved, 1, "reconcile must resolve the single fork");
        assert!(
            list_forks(replica_a.path())?.is_empty(),
            "no fork must remain after reconciliation"
        );
        Ok(())
    }
}
