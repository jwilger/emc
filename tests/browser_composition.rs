#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::Error as IoError;

    use emc::core::browser::{BrowserSliceDocuments, compose_browser_workflow};
    use emc::core::effect::FileContents;

    #[test]
    fn composed_browser_workflow_deduplicates_canonical_board_lanes() -> Result<(), Box<dyn Error>>
    {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": [\n    {\"id\": \"ux\", \"name\": \"People, Views, and Translations\"},\n    {\"id\": \"actions\", \"name\": \"Commands and Projections\"},\n    {\"id\": \"events\", \"name\": \"Stored Facts\"}\n  ]},\n  \"slice_files\": [\n    \"../slices/lesson-entry.eventmodel.json\",\n    \"../slices/lesson-show.eventmodel.json\"\n  ],\n  \"steps\": []\n}\n",
        );
        let entry_slice = file_contents(slice_with_canonical_lanes("Entry"));
        let show_slice = file_contents(slice_with_canonical_lanes("Show lesson"));

        let composed = compose_browser_workflow(
            workflow,
            BrowserSliceDocuments::from_documents([entry_slice, show_slice]),
        )?;

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

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

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

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

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

    #[test]
    fn composed_browser_workflow_labels_transition_cards() -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\", \"transitions\": [{\"to\": \"show-lesson\", \"via_navigation\": \"lesson_screen\"}]},\n    {\"slice\": \"show-lesson\", \"name\": \"show lesson\", \"relationship\": \"main\", \"transitions\": [\n      {\"to\": \"submit\", \"via_command\": \"SubmitLessonForReview\"},\n      {\"to\": \"checkpoint\", \"via_external_trigger\": \"lesson_checkpoint_result\"}\n    ]},\n    {\"slice\": \"submit\", \"name\": \"submit\", \"relationship\": \"main\", \"transitions\": [{\"to\": \"review\", \"via_event\": \"LessonSubmittedForReview\"}]},\n    {\"slice\": \"review\", \"name\": \"review\", \"relationship\": \"main\", \"transitions\": [{\"to_workflow\": \"course-lesson-02\", \"target_name\": \"next lesson\", \"via_outcome\": \"LessonAccepted\"}]},\n    {\"slice\": \"checkpoint\", \"name\": \"checkpoint\", \"relationship\": \"alternate\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

        assert_eq!(
            composed
                .transition_cards()
                .iter()
                .map(|card| {
                    (
                        card.source().as_ref(),
                        card.target().as_ref(),
                        card.kind().as_ref(),
                        card.label().as_ref(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                ("entry", "show lesson", "navigation", "lesson_screen"),
                ("show lesson", "submit", "command", "SubmitLessonForReview",),
                (
                    "show lesson",
                    "checkpoint",
                    "external trigger",
                    "lesson_checkpoint_result",
                ),
                ("submit", "review", "event", "LessonSubmittedForReview"),
                ("review", "next lesson", "workflow exit", "LessonAccepted"),
            ]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_labels_alternate_outcome_branches() -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\"},\n    {\"slice\": \"review\", \"name\": \"review\", \"relationship\": \"main\", \"transitions\": [\n      {\"to_workflow\": \"course-lesson-02\", \"target_name\": \"next lesson\", \"via_outcome\": \"LessonAccepted\"},\n      {\"to\": \"revision\", \"via_outcome\": \"LessonRevisionRequested\"}\n    ]},\n    {\"slice\": \"revision\", \"name\": \"LessonRevisionRequested\", \"relationship\": \"alternate\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

        assert_eq!(
            composed
                .branch_cards()
                .iter()
                .map(|card| (card.name().as_ref(), card.label().as_ref()))
                .collect::<Vec<_>>(),
            [("LessonRevisionRequested", "alternate outcome")]
        );
        assert!(
            composed
                .main_path_names()
                .iter()
                .all(|name| name.as_ref() != "LessonRevisionRequested"),
            "alternate outcome must not be part of the main path"
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_keeps_plain_alternate_label_without_incoming_outcome()
    -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\"},\n    {\"slice\": \"review\", \"name\": \"review\", \"relationship\": \"main\", \"transitions\": [\n      {\"to_workflow\": \"course-lesson-02\", \"target_name\": \"next lesson\", \"via_outcome\": \"LessonAccepted\"}\n    ]},\n    {\"slice\": \"checkpoint\", \"name\": \"checkpoint\", \"relationship\": \"alternate\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

        assert_eq!(
            composed
                .branch_cards()
                .iter()
                .map(|card| (card.name().as_ref(), card.label().as_ref()))
                .collect::<Vec<_>>(),
            [("checkpoint", "alternate")]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_does_not_mark_non_alternate_branches_as_outcomes()
    -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Organization access\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Member access lifecycle.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\"},\n    {\"slice\": \"activate-member\", \"name\": \"activate-member\", \"relationship\": \"main\", \"transitions\": [\n      {\"to\": \"record-member-suspension\", \"via_outcome\": \"MemberSuspended\"}\n    ]},\n    {\"slice\": \"record-member-suspension\", \"name\": \"record-member-suspension\", \"relationship\": \"async_lifecycle\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

        assert_eq!(
            composed
                .branch_cards()
                .iter()
                .map(|card| (card.name().as_ref(), card.label().as_ref()))
                .collect::<Vec<_>>(),
            [("record-member-suspension", "async lifecycle")]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_requires_same_transition_to_target_and_outcome()
    -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\", \"transitions\": [\n      {\"to\": \"checkpoint\", \"via_navigation\": \"checkpoint_screen\"},\n      {\"to\": \"revision\", \"via_outcome\": \"LessonRevisionRequested\"}\n    ]},\n    {\"slice\": \"checkpoint\", \"name\": \"checkpoint\", \"relationship\": \"alternate\"},\n    {\"slice\": \"revision\", \"name\": \"revision\", \"relationship\": \"alternate\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

        assert_eq!(
            composed
                .branch_cards()
                .iter()
                .map(|card| (card.name().as_ref(), card.label().as_ref()))
                .collect::<Vec<_>>(),
            [
                ("checkpoint", "alternate"),
                ("revision", "alternate outcome")
            ]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_labels_retry_transition() -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"review\", \"name\": \"review\", \"relationship\": \"entry\", \"transitions\": [\n      {\"name\": \"RegenerateTeacherReview\", \"to\": \"review\", \"retry\": true}\n    ]}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

        assert_eq!(
            composed
                .transition_cards()
                .iter()
                .map(|card| {
                    (
                        card.name().as_ref(),
                        card.kind().as_ref(),
                        card.label().as_ref(),
                        card.target().as_ref(),
                    )
                })
                .collect::<Vec<_>>(),
            [("RegenerateTeacherReview", "retry", "retry", "review")]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_renders_error_recovery_cards() -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [\"../slices/submit.eventmodel.json\"],\n  \"steps\": []\n}\n",
        );
        let submit_slice = file_contents(
            "{\n  \"name\": \"Submit lesson\",\n  \"version\": \"0.1.0\",\n  \"board\": {\"lanes\": [], \"slices\": [{\"name\": \"Submit lesson\", \"elements\": [\n    {\"id\": \"lesson_screen\", \"kind\": \"view\", \"lane\": \"ux\", \"name\": \"lesson_screen\"},\n    {\"id\": \"lesson_submitted\", \"kind\": \"event\", \"lane\": \"events\", \"name\": \"LessonSubmittedForReview\"}\n  ], \"connections\": []}]},\n  \"views\": [{\"name\": \"lesson_screen\", \"uses_read_models\": [], \"controls\": [\n    {\"label\": \"Submit for review\", \"command\": \"SubmitLesson\", \"error_handling\": [{\"error\": \"evidence_required\", \"stay_on_screen\": true}]}\n  ]}],\n  \"slices\": [{\"name\": \"Submit lesson\", \"type\": \"state_view\", \"views\": [\"lesson_screen\"], \"acceptance_scenarios\": [], \"contract_scenarios\": []}]\n}\n",
        );

        let composed = compose_browser_workflow(
            workflow,
            BrowserSliceDocuments::from_documents([submit_slice]),
        )?;

        assert_eq!(
            composed
                .error_recovery_cards()
                .iter()
                .map(|card| (card.name().as_ref(), card.source_screen().as_ref()))
                .collect::<Vec<_>>(),
            [("evidence_required", "lesson_screen")]
        );
        assert!(
            composed
                .event_element_names()
                .iter()
                .all(|name| name.as_ref() != "evidence_required"),
            "command errors must not render as event elements"
        );
        assert_eq!(
            composed
                .event_element_names()
                .iter()
                .map(|name| name.as_ref())
                .collect::<Vec<_>>(),
            ["LessonSubmittedForReview"]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_projects_review_diagnostics() -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [],\n  \"steps\": [\n    {\"slice\": \"entry\", \"name\": \"entry\", \"relationship\": \"entry\"},\n    {\"slice\": \"review\", \"name\": \"review\", \"relationship\": \"main\"}\n  ],\n  \"review_diagnostics\": [\n    {\"step\": \"review\", \"status\": \"unreachable\", \"missing_rule\": \"entry reachability\"}\n  ]\n}\n",
        );

        let composed = compose_browser_workflow(workflow, BrowserSliceDocuments::empty())?;

        assert_eq!(
            composed
                .review_overlays()
                .iter()
                .map(|overlay| {
                    (
                        overlay.step().as_ref(),
                        overlay.status().as_ref(),
                        overlay.missing_rule().as_ref(),
                    )
                })
                .collect::<Vec<_>>(),
            [("review", "unreachable", "entry reachability")]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_projects_command_definition_back_references()
    -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [\"../slices/submit.eventmodel.json\"],\n  \"steps\": [\n    {\"slice\": \"show-lesson\", \"name\": \"show lesson\", \"relationship\": \"entry\", \"transitions\": [{\"to\": \"submit\", \"via_command\": \"SubmitLessonForReview\"}]},\n    {\"slice\": \"submit\", \"name\": \"submit\", \"relationship\": \"main\"}\n  ]\n}\n",
        );
        let submit_slice = file_contents(
            "{\n  \"name\": \"Submit lesson for review\",\n  \"version\": \"0.1.0\",\n  \"board\": {\"lanes\": []},\n  \"commands\": [{\"name\": \"SubmitLessonForReview\", \"inputs\": [], \"produces\": [\"LessonSubmittedForReview\"], \"errors\": [\"evidence_required\"]}],\n  \"read_models\": [{\"name\": \"lesson_submission_context\", \"fields\": []}],\n  \"views\": [{\"name\": \"lesson_screen\", \"uses_read_models\": [], \"controls\": [\n    {\"label\": \"Submit for review\", \"command\": \"SubmitLessonForReview\"}\n  ]}],\n  \"slices\": [{\"name\": \"Submit lesson for review\", \"type\": \"state_view\", \"commands\": [\"SubmitLessonForReview\"], \"views\": [\"lesson_screen\"], \"read_models\": [\"lesson_submission_context\"], \"acceptance_scenarios\": [], \"contract_scenarios\": []}]\n}\n",
        );

        let composed = compose_browser_workflow(
            workflow,
            BrowserSliceDocuments::from_documents([submit_slice]),
        )?;
        let definition = composed
            .command_definitions()
            .iter()
            .find(|definition| definition.name().as_ref() == "SubmitLessonForReview")
            .ok_or_else(|| IoError::other("command definition must be projected"))?;

        assert_eq!(
            definition.owning_slice().as_ref(),
            "Submit lesson for review"
        );
        assert_eq!(
            definition
                .source_controls()
                .iter()
                .map(|source| source.as_ref())
                .collect::<Vec<_>>(),
            ["lesson_screen / Submit for review"]
        );
        assert_eq!(
            definition
                .section_labels()
                .iter()
                .map(|section| section.as_ref())
                .collect::<Vec<_>>(),
            [
                "Produced events",
                "Read models",
                "Returned errors",
                "Workflow transitions",
            ]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_projects_view_field_source_chains() -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [\"../slices/show-lesson.eventmodel.json\"],\n  \"steps\": []\n}\n",
        );
        let show_slice = file_contents(
            "{\n  \"name\": \"Show lesson\",\n  \"version\": \"0.1.0\",\n  \"board\": {\"lanes\": []},\n  \"events\": [{\"name\": \"CourseLessonCatalogPublished\", \"stream\": \"course_lesson_catalog\", \"attributes\": [\n    {\"name\": \"lesson_title\", \"source\": \"external.course_lesson_catalog_manifest.lesson_title\"}\n  ]}],\n  \"read_models\": [{\"name\": \"lesson_state\", \"fields\": [\n    {\"name\": \"lesson_title\", \"source\": \"CourseLessonCatalogPublished.lesson_title\"}\n  ]}],\n  \"views\": [{\"name\": \"lesson_screen\", \"uses_read_models\": [\"lesson_state\"], \"fields\": [\n    {\"name\": \"lesson_title\", \"source\": \"read_model.lesson_state.lesson_title\"}\n  ]}],\n  \"slices\": [{\"name\": \"Show lesson\", \"type\": \"state_view\", \"views\": [\"lesson_screen\"], \"read_models\": [\"lesson_state\"], \"events\": [\"CourseLessonCatalogPublished\"], \"acceptance_scenarios\": [], \"contract_scenarios\": []}]\n}\n",
        );

        let composed = compose_browser_workflow(
            workflow,
            BrowserSliceDocuments::from_documents([show_slice]),
        )?;
        let definition = composed
            .view_definitions()
            .iter()
            .find(|definition| definition.name().as_ref() == "lesson_screen")
            .ok_or_else(|| IoError::other("view definition must be projected"))?;

        assert_eq!(
            definition
                .field_source_chains()
                .iter()
                .map(|chain| {
                    (
                        chain.field().as_ref(),
                        chain
                            .hops()
                            .iter()
                            .map(|hop| hop.as_ref())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
            [(
                "lesson_title",
                vec![
                    "read_model.lesson_state.lesson_title",
                    "CourseLessonCatalogPublished.lesson_title",
                    "external.course_lesson_catalog_manifest.lesson_title",
                ],
            )]
        );

        Ok(())
    }

    #[test]
    fn composed_browser_workflow_projects_view_control_effects() -> Result<(), Box<dyn Error>> {
        let workflow = file_contents(
            "{\n  \"name\": \"Lesson 01\",\n  \"version\": \"0.1.0\",\n  \"description\": \"A composed lesson workflow.\",\n  \"board\": {\"lanes\": []},\n  \"slice_files\": [\"../slices/show-lesson.eventmodel.json\"],\n  \"steps\": []\n}\n",
        );
        let show_slice = file_contents(
            "{\n  \"name\": \"Show lesson\",\n  \"version\": \"0.1.0\",\n  \"board\": {\"lanes\": []},\n  \"commands\": [{\"name\": \"SubmitLessonForReview\", \"inputs\": [], \"produces\": []}],\n  \"views\": [{\"name\": \"lesson_screen\", \"uses_read_models\": [], \"controls\": [\n    {\"label\": \"Submit for review\", \"command\": \"SubmitLessonForReview\"},\n    {\"label\": \"Open next lesson\", \"navigation\": \"course-lesson-02\", \"navigation_type\": \"external_workflow\"},\n    {\"label\": \"Open help\", \"navigation\": \"https://docs.example.test/help\", \"navigation_type\": \"external_system\"},\n    {\"label\": \"Show filters\", \"navigation\": \"filters_open\", \"navigation_type\": \"local_view_state\"},\n    {\"label\": \"Open rubric\", \"navigation\": \"rubric_screen\", \"navigation_type\": \"modeled_view\"}\n  ]}],\n  \"slices\": [{\"name\": \"Show lesson\", \"type\": \"state_view\", \"commands\": [\"SubmitLessonForReview\"], \"views\": [\"lesson_screen\"], \"acceptance_scenarios\": [], \"contract_scenarios\": []}]\n}\n",
        );

        let composed = compose_browser_workflow(
            workflow,
            BrowserSliceDocuments::from_documents([show_slice]),
        )?;
        let definition = composed
            .view_definitions()
            .iter()
            .find(|definition| definition.name().as_ref() == "lesson_screen")
            .ok_or_else(|| IoError::other("view definition must be projected"))?;

        assert_eq!(
            definition
                .control_effects()
                .iter()
                .map(|effect| {
                    (
                        effect.label().as_ref(),
                        effect.kind().as_ref(),
                        effect.target().as_ref(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                ("Submit for review", "command", "SubmitLessonForReview"),
                (
                    "Open next lesson",
                    "workflow navigation",
                    "course-lesson-02",
                ),
                (
                    "Open help",
                    "external navigation",
                    "https://docs.example.test/help",
                ),
                ("Show filters", "local navigation", "filters_open"),
                ("Open rubric", "view navigation", "rubric_screen"),
            ]
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
