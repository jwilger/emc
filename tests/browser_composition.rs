#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::browser::compose_browser_workflow;
    use emc::core::effect::FileContents;

    #[test]
    fn composed_browser_workflow_deduplicates_canonical_board_lanes() -> Result<(), Box<dyn Error>>
    {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": [\n    {\"id\": \"ux\", \"name\": \"People, Views, and Translations\"},\n    {\"id\": \"actions\", \"name\": \"Commands and Projections\"},\n    {\"id\": \"events\", \"name\": \"Stored Facts\"}\n  ]},\n  \"slice_files\": [\n    \"../slices/lesson-entry.eventmodel.json\",\n    \"../slices/lesson-show.eventmodel.json\"\n  ],\n  \"steps\": []\n}\n",
        );
        let entry_slice = file_contents(slice_with_canonical_lanes("Entry"));
        let show_slice = file_contents(slice_with_canonical_lanes("Show lesson"));

        let composed = compose_browser_workflow(workflow, vec![entry_slice, show_slice])?;

        assert_eq!(
            composed
                .lane_ids()
                .iter()
                .map(|lane| lane.as_ref())
                .collect::<Vec<_>>(),
            ["ux", "actions", "events"]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_uses_workflow_step_order_for_main_path()
    -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\"},\n    {\"slice\": \"show-lesson\", \"name\": \"show lesson\", \"relationship\": \"main\"},\n    {\"slice\": \"checkpoint\", \"name\": \"checkpoint\", \"relationship\": \"alternate\"},\n    {\"slice\": \"submit\", \"name\": \"submit\", \"relationship\": \"main\"},\n    {\"slice\": \"review\", \"name\": \"review\", \"relationship\": \"main\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, Vec::new())?;

        assert_eq!(
            composed
                .main_path_names()
                .iter()
                .map(|name| name.as_ref())
                .collect::<Vec<_>>(),
            ["entry", "show lesson", "submit", "review"]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_renders_async_lifecycle_as_branch_card()
    -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Organization access\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Member access lifecycle.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\"},\n    {\"slice\": \"activate-member\", \"name\": \"activate-member\", \"relationship\": \"main\"},\n    {\"slice\": \"record-member-suspension\", \"name\": \"record-member-suspension\", \"relationship\": \"async_lifecycle\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, Vec::new())?;

        assert_eq!(
            composed
                .branch_cards()
                .iter()
                .map(|card| (card.name().as_ref(), card.label().as_ref()))
                .collect::<Vec<_>>(),
            [("record-member-suspension", "async lifecycle")]
        );
        assert!(
            composed
                .main_path_names()
                .iter()
                .all(|name| name.as_ref() != "record-member-suspension"),
            "async lifecycle step must not be part of the main path"
        );

        Ok(())
    }

    fn slice_with_canonical_lanes(name: &str) -> String {
        format!(
            "{{\n  \"name\": \"{name}\",\n  \"version\": \"0.1.0\",\n  \"board\": {{\"lanes\": [\n    {{\"id\": \"ux\", \"name\": \"People, Views, and Translations\"}},\n    {{\"id\": \"actions\", \"name\": \"Commands and Projections\"}},\n    {{\"id\": \"events\", \"name\": \"Stored Facts\"}}\n  ]}},\n  \"slices\": [{{\"name\": \"{name}\", \"type\": \"state_view\", \"views\": [], \"acceptance_scenarios\": [], \"contract_scenarios\": []}}]\n}}\n"
        )
    }

    fn file_contents(value: impl Into<String>) -> FileContents {
        FileContents::try_new(value.into()).unwrap_or_else(|error| {
            unreachable!("test fixture contents must be valid: {error}");
        })
    }
}
