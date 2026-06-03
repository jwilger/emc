use crate::core::effect::FileContents;
use crate::core::formal_graph::FormalWorkflowGraph;
use crate::core::types::{WorkflowSliceDetail, WorkflowTransitionKind, WorkflowTransitionRecord};

pub fn project_workflow_browser_document(workflow: &FormalWorkflowGraph) -> FileContents {
    file_contents(workflow_projection_json(workflow))
}

pub fn project_slice_browser_document(slice: &WorkflowSliceDetail) -> FileContents {
    file_contents(slice_projection_json(slice))
}

fn workflow_projection_json(workflow: &FormalWorkflowGraph) -> String {
    let slice_files = workflow
        .slice_details()
        .as_slice()
        .iter()
        .map(|slice| {
            format!(
                "    \"../slices/{}.eventmodel.json\"",
                slice.slug().as_ref()
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let steps = workflow
        .slice_details()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, slice)| workflow_step_json(index, slice, workflow.transitions().as_slice()))
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "{{\n  \"name\": {},\n  \"version\": \"0.1.0\",\n  \"description\": {},\n  \"board\": {{}},\n  \"streams\": [],\n  \"events\": [],\n  \"commands\": [],\n  \"read_models\": [],\n  \"slices\": [],\n  \"slice_files\": [\n{slice_files}\n  ],\n  \"steps\": [\n{steps}\n  ]\n}}\n",
        json_string(workflow.name().as_ref()),
        json_string(workflow.description().as_ref()),
    )
}

fn workflow_step_json(
    index: usize,
    slice: &WorkflowSliceDetail,
    transitions: &[WorkflowTransitionRecord],
) -> String {
    let relationship = if index == 0 { "entry" } else { "main" };
    let slice_transitions = transitions
        .iter()
        .filter(|transition| transition.source().as_ref() == slice.slug().as_ref())
        .map(transition_projection_json)
        .collect::<Vec<_>>();
    let transition_field = if slice_transitions.is_empty() {
        String::new()
    } else {
        format!(
            ",\n      \"transitions\": [\n{}\n      ]",
            slice_transitions.join(",\n")
        )
    };
    format!(
        "    {{\n      \"slice\": {},\n      \"name\": {},\n      \"type\": {},\n      \"description\": {},\n      \"relationship\": {}{transition_field}\n    }}",
        json_string(slice.slug().as_ref()),
        json_string(slice.name().as_ref()),
        json_string(slice.kind().as_ref()),
        json_string(slice.description().as_ref()),
        json_string(relationship),
    )
}

fn transition_projection_json(transition: &WorkflowTransitionRecord) -> String {
    if let Some(kind) = workflow_exit_kind(transition.kind()) {
        format!(
            "        {{\"to_workflow\": {}, \"{}\": {}}}",
            json_string(transition.target().as_ref()),
            transition_field(kind),
            json_string(transition.trigger().as_ref())
        )
    } else {
        format!(
            "        {{\"to\": {}, \"{}\": {}}}",
            json_string(transition.target().as_ref()),
            transition_field(transition.kind().as_ref()),
            json_string(transition.trigger().as_ref())
        )
    }
}

fn workflow_exit_kind(kind: &WorkflowTransitionKind) -> Option<&str> {
    kind.as_ref().strip_prefix("workflow_exit:")
}

fn transition_field(kind: &str) -> &'static str {
    match kind {
        "command" => "via_command",
        "event" => "via_event",
        "external_trigger" => "via_external_trigger",
        "outcome" => "via_outcome",
        _ => "via_navigation",
    }
}

fn slice_projection_json(slice: &WorkflowSliceDetail) -> String {
    if slice.kind().as_ref() == "state_view" {
        state_view_slice_json(slice)
    } else {
        draft_slice_json(slice)
    }
}

fn state_view_slice_json(slice: &WorkflowSliceDetail) -> String {
    let source_view_name = slice.slug().as_ref().to_owned();
    let navigation_view_name = format!("{}-screen", slice.slug().as_ref().replace('_', "-"));
    let source_element_id = format!("view-{source_view_name}");
    let scenario_name = slice.name().as_ref().to_ascii_lowercase();
    format!(
        "{{\n  \"name\": {},\n  \"version\": \"0.1.0\",\n  \"description\": {},\n  \"type\": {},\n  \"board\": {{\n    \"lanes\": [\n      {{\"id\": \"ux\", \"name\": \"People, Views, and Translations\"}},\n      {{\"id\": \"actions\", \"name\": \"Commands and Projections\"}},\n      {{\"id\": \"events\", \"name\": \"Stored Facts\"}}\n    ],\n    \"slices\": [\n      {{\n        \"name\": {},\n        \"elements\": [{{\"id\": {}, \"kind\": \"view\", \"lane\": \"ux\", \"name\": {}}}],\n        \"connections\": []\n      }}\n    ]\n  }},\n  \"streams\": [],\n  \"events\": [],\n  \"commands\": [],\n  \"read_models\": [],\n  \"views\": [\n    {{\n      \"name\": {},\n      \"description\": {},\n      \"wireframe\": \"<section></section>\",\n      \"uses_read_models\": [],\n      \"controls\": []\n    }},\n    {{\n      \"name\": {},\n      \"description\": {},\n      \"wireframe\": \"<section></section>\",\n      \"uses_read_models\": [],\n      \"controls\": []\n    }}\n  ],\n  \"slices\": [\n    {{\n      \"name\": {},\n      \"slug\": {},\n      \"type\": {},\n      \"events\": [],\n      \"views\": [{}, {}],\n      \"acceptance_scenarios\": [{{\"name\": {}, \"given\": [], \"when\": {{}}, \"then\": []}}],\n      \"contract_scenarios\": []\n    }}\n  ]\n}}\n",
        json_string(slice.name().as_ref()),
        json_string(slice.description().as_ref()),
        json_string(slice.kind().as_ref()),
        json_string(slice.name().as_ref()),
        json_string(&source_element_id),
        json_string(&source_view_name),
        json_string(&source_view_name),
        json_string(slice.description().as_ref()),
        json_string(&navigation_view_name),
        json_string(slice.description().as_ref()),
        json_string(slice.name().as_ref()),
        json_string(slice.slug().as_ref()),
        json_string(slice.kind().as_ref()),
        json_string(&source_view_name),
        json_string(&navigation_view_name),
        json_string(&scenario_name),
    )
}

fn draft_slice_json(slice: &WorkflowSliceDetail) -> String {
    format!(
        "{{\n  \"name\": {},\n  \"version\": \"0.1.0\",\n  \"description\": {},\n  \"type\": {},\n  \"board\": {{}},\n  \"streams\": [],\n  \"events\": [],\n  \"commands\": [],\n  \"read_models\": [],\n  \"views\": [],\n  \"slices\": [\n    {{\n      \"name\": {},\n      \"type\": {},\n      \"events\": [],\n      \"views\": [],\n      \"acceptance_scenarios\": [],\n      \"contract_scenarios\": []\n    }}\n  ]\n}}\n",
        json_string(slice.name().as_ref()),
        json_string(slice.description().as_ref()),
        json_string(slice.kind().as_ref()),
        json_string(slice.name().as_ref()),
        json_string(slice.kind().as_ref()),
    )
}

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated browser projection contents must be valid: {error}");
    })
}

fn json_string(value: impl Into<String>) -> String {
    serde_json::to_string(&value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated browser projection JSON must be valid: {error}");
    })
}
