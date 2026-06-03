use crate::core::effect::FileContents;
use crate::core::formal_graph::FormalWorkflowGraph;
use crate::core::types::{WorkflowSliceDetail, WorkflowTransitionKind, WorkflowTransitionRecord};
use serde_json::{Value, json};

pub(crate) fn project_browser_index_document(workflows: &[FormalWorkflowGraph]) -> FileContents {
    let mut workflows = workflows.to_owned();
    workflows.sort_by(|left, right| left.slug().as_ref().cmp(right.slug().as_ref()));
    file_contents(browser_index_json(&workflows))
}

pub(crate) fn project_workflow_browser_document(workflow: &FormalWorkflowGraph) -> FileContents {
    file_contents(workflow_projection_json(workflow))
}

pub(crate) fn project_slice_browser_document(
    workflow: &FormalWorkflowGraph,
    slice: &WorkflowSliceDetail,
) -> FileContents {
    file_contents(slice_projection_json(workflow, slice))
}

fn workflow_projection_json(workflow: &FormalWorkflowGraph) -> String {
    if workflow.slice_details().as_slice().is_empty()
        && workflow.transitions().as_slice().is_empty()
    {
        return compact_empty_workflow_json(workflow);
    }

    let slice_files = workflow
        .slice_details()
        .as_slice()
        .iter()
        .map(|slice| {
            Value::String(format!(
                "../slices/{}.eventmodel.json",
                slice.slug().as_ref()
            ))
        })
        .collect::<Vec<_>>();
    let steps = workflow
        .slice_details()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, slice)| {
            workflow_step_json(
                index,
                slice,
                workflow.transitions().as_slice(),
                !workflow.transitions().as_slice().is_empty(),
            )
        })
        .collect::<Vec<_>>();
    pretty_json(json!({
        "name": workflow.name().as_ref(),
        "version": "0.1.0",
        "description": workflow.description().as_ref(),
        "board": {},
        "streams": [],
        "events": [],
        "commands": [],
        "read_models": [],
        "slices": [],
        "slice_files": slice_files,
        "steps": steps,
    }))
}

fn compact_empty_workflow_json(workflow: &FormalWorkflowGraph) -> String {
    format!(
        "{{\n  \"name\": {},\n  \"version\": \"0.1.0\",\n  \"description\": {},\n  \"board\": {{}},\n  \"streams\": [],\n  \"events\": [],\n  \"commands\": [],\n  \"read_models\": [],\n  \"slices\": [],\n  \"slice_files\": [],\n  \"steps\": []\n}}\n",
        json_string(workflow.name().as_ref()),
        json_string(workflow.description().as_ref()),
    )
}

fn browser_index_json(workflows: &[FormalWorkflowGraph]) -> String {
    let entries = workflows
        .iter()
        .map(|workflow| {
            format!(
                "    {{\n      \"name\": {},\n      \"path\": \"data/workflows/{}.eventmodel.json\",\n      \"description\": {}\n    }}",
                json_string(workflow.name().as_ref()),
                workflow.slug().as_ref(),
                json_string(workflow.description().as_ref())
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    if entries.is_empty() {
        "{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": []\n}\n".to_owned()
    } else {
        format!(
            "{{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": [\n{entries}\n  ]\n}}\n"
        )
    }
}

fn workflow_step_json(
    index: usize,
    slice: &WorkflowSliceDetail,
    transitions: &[WorkflowTransitionRecord],
    include_empty_transitions: bool,
) -> Value {
    let relationship = if index == 0 { "entry" } else { "main" };
    let slice_transitions = transitions
        .iter()
        .filter(|transition| transition.source().as_ref() == slice.slug().as_ref())
        .map(transition_projection_json)
        .collect::<Vec<_>>();
    if slice_transitions.is_empty() && !include_empty_transitions {
        return json!({
            "slice": slice.slug().as_ref(),
            "name": slice.name().as_ref(),
            "type": slice.kind().as_ref(),
            "description": slice.description().as_ref(),
            "relationship": relationship,
        });
    }
    json!({
        "slice": slice.slug().as_ref(),
        "name": slice.name().as_ref(),
        "type": slice.kind().as_ref(),
        "description": slice.description().as_ref(),
        "relationship": relationship,
        "transitions": slice_transitions,
    })
}

fn transition_projection_json(transition: &WorkflowTransitionRecord) -> Value {
    if let Some(kind) = workflow_exit_kind(transition.kind()) {
        let mut transition_json = serde_json::Map::from_iter([
            (
                "to_workflow".to_owned(),
                Value::String(transition.target().as_ref().to_owned()),
            ),
            (
                transition_field(kind).to_owned(),
                Value::String(transition.trigger().as_ref().to_owned()),
            ),
        ]);
        if let Some(rationale) = transition.rationale() {
            transition_json.insert(
                "exit_reason".to_owned(),
                Value::String(rationale.as_ref().to_owned()),
            );
        }
        Value::Object(transition_json)
    } else {
        json!({
            "to": transition.target().as_ref(),
            transition_field(transition.kind().as_ref()): transition.trigger().as_ref(),
        })
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

fn slice_projection_json(workflow: &FormalWorkflowGraph, slice: &WorkflowSliceDetail) -> String {
    if slice.kind().as_ref() == "state_view" {
        state_view_slice_json(workflow, slice)
    } else {
        draft_slice_json(slice)
    }
}

fn state_view_slice_json(workflow: &FormalWorkflowGraph, slice: &WorkflowSliceDetail) -> String {
    if modeled_navigation_targets(workflow, slice).is_empty() {
        return compact_state_view_slice_json(slice);
    }

    let source_view_name = slice.slug().as_ref().to_owned();
    let navigation_view_name = format!("{}-screen", slice.slug().as_ref().replace('_', "-"));
    let source_element_id = format!("view-{source_view_name}");
    let scenario_name = slice.name().as_ref().to_ascii_lowercase();
    let mut views = vec![
        json!({
            "name": source_view_name,
            "description": slice.description().as_ref(),
            "wireframe": wireframe(workflow, slice),
            "uses_read_models": [],
            "controls": navigation_controls(workflow, slice),
        }),
        json!({
            "name": navigation_view_name,
            "description": slice.description().as_ref(),
            "wireframe": "<section></section>",
            "uses_read_models": [],
            "controls": [],
        }),
    ];
    let navigation_targets = modeled_navigation_targets(workflow, slice);
    for navigation in navigation_targets {
        if !views.iter().any(|view| {
            view.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == navigation)
        }) {
            views.push(json!({
                "name": navigation,
                "wireframe": "<section></section>",
                "uses_read_models": [],
                "controls": [],
            }));
        }
    }

    pretty_json(json!({
        "name": slice.name().as_ref(),
        "version": "0.1.0",
        "description": slice.description().as_ref(),
        "type": slice.kind().as_ref(),
        "board": {
            "lanes": [
                {"id": "ux", "name": "People, Views, and Translations"},
                {"id": "actions", "name": "Commands and Projections"},
                {"id": "events", "name": "Stored Facts"},
            ],
            "slices": [{
                "name": slice.name().as_ref(),
                "elements": [{"id": source_element_id, "kind": "view", "lane": "ux", "name": source_view_name}],
                "connections": [],
            }],
        },
        "streams": [],
        "events": [],
        "commands": [],
        "read_models": [],
        "views": views,
        "slices": [{
            "name": slice.name().as_ref(),
            "slug": slice.slug().as_ref(),
            "type": slice.kind().as_ref(),
            "events": [],
            "views": [slice.slug().as_ref(), navigation_view_name],
            "acceptance_scenarios": [{"name": scenario_name, "given": [], "when": {}, "then": []}],
            "contract_scenarios": [],
        }],
    }))
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

fn compact_state_view_slice_json(slice: &WorkflowSliceDetail) -> String {
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

fn wireframe(workflow: &FormalWorkflowGraph, slice: &WorkflowSliceDetail) -> String {
    let buttons = modeled_navigation_targets(workflow, slice)
        .into_iter()
        .map(|navigation| format!("<button data-ref=\"{navigation}\"></button>"))
        .collect::<String>();
    format!("<section>{buttons}</section>")
}

fn navigation_controls(workflow: &FormalWorkflowGraph, slice: &WorkflowSliceDetail) -> Vec<Value> {
    modeled_navigation_targets(workflow, slice)
        .into_iter()
        .map(|navigation| {
            json!({
                "label": navigation,
                "navigation": navigation,
                "navigation_type": "modeled_view",
            })
        })
        .collect()
}

fn modeled_navigation_targets(
    workflow: &FormalWorkflowGraph,
    slice: &WorkflowSliceDetail,
) -> Vec<String> {
    workflow
        .transitions()
        .as_slice()
        .iter()
        .filter(|transition| transition.source().as_ref() == slice.slug().as_ref())
        .filter(|transition| transition.kind().as_ref() == "navigation")
        .map(|transition| transition.trigger().as_ref().to_owned())
        .collect()
}

fn pretty_json(value: Value) -> String {
    let json = serde_json::to_string_pretty(&value).unwrap_or_else(|error| {
        unreachable!("EMC generated browser projection JSON must be valid: {error}");
    });
    format!("{json}\n")
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
