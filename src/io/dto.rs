use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::{Map, Value};

use crate::core::emc::{EMCSliceImport, EMCWorkflowImport};
use crate::core::effect::FileContents;
use crate::core::layout::ImportedWorkflowLayout;
use crate::core::project::ProjectName;
use crate::core::types::{
    LeanModuleName, ModelDigest, ModelName, QuintModuleName, SliceSlug, WorkflowSlug,
};
use crate::core::validation::{
    AutomationCommandPolicy, AutomationTrigger, BoardReadModelCommandDependency, CommandDefinition,
    CommandDefinitionParts, CommandInputSource, CommandInputSourceKind, CommandReadModelReads,
    ControlCommandErrorHandling, ControlErrorRecoveryBehavior, DefinitionKind, DefinitionName,
    EventAttribute, EventAttributeSource, EventDefinition, EventModelDocument,
    EventModelDocumentParts, EventModelFileKind, ExternalInputSchema, ExternalPayloadVariant,
    LegacyScenariosField, NamedDefinition, NavigationType, OutcomeDefinition, ReadModelDefinition,
    ReadModelField, ReadModelFieldAbsenceDefault, ReadModelFieldDerivation, ReadModelFieldSource,
    ReadModelState, ReadModelTransitiveDerivation, ScenarioSetKind, ScenarioStepField,
    SingletonBehavior, SliceDefinition, SliceDefinitionCount, SliceDefinitionParts, SliceScenario,
    SliceScenarioParts, SliceType, TopLevelKey, TranslationContract, ViewControlDefinition,
    ViewControlDefinitionParts, ViewDefinition, ViewWireframe, WorkflowComposition,
    WorkflowEntryStepCount, WorkflowInternalDefinitions, WorkflowStep, WorkflowStepExit,
    WorkflowStepLifecycleRole, WorkflowStepRelationship, WorkflowStepTrigger,
    empty_top_level_key_issue, model_must_be_object_issue,
};

#[derive(Debug)]
pub struct BoundaryParseError {
    message: String,
}

impl BoundaryParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for BoundaryParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BoundaryParseError {}

pub fn parse_model_name(raw: &str) -> Result<ModelName, BoundaryParseError> {
    ModelName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid model name: {error}")))
}

pub fn parse_event_model_document(
    raw: &str,
    file_kind: EventModelFileKind,
) -> Result<EventModelDocument, BoundaryParseError> {
    serde_json::from_str::<Value>(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid JSON: {error}")))
        .and_then(|value| event_model_document_from_json(value, file_kind))
}

pub fn parse_project_name(raw: &str) -> Result<ProjectName, BoundaryParseError> {
    ProjectName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid project name: {error}")))
}

pub fn parse_project_manifest_name(raw: &str) -> Result<ProjectName, BoundaryParseError> {
    raw.lines()
        .find_map(|line| line.trim().strip_prefix("name = "))
        .and_then(quoted_value)
        .ok_or_else(|| BoundaryParseError::new("emc.toml is missing project name"))
        .and_then(parse_project_name)
}

pub fn parse_browser_index_workflows(
    raw: &str,
) -> Result<Vec<ImportedWorkflowLayout>, BoundaryParseError> {
    let value = serde_json::from_str::<Value>(raw)
        .map_err(|error| BoundaryParseError::new(format!("invalid browser index JSON: {error}")))?;
    let workflows = value
        .get("workflows")
        .and_then(Value::as_array)
        .ok_or_else(|| BoundaryParseError::new("browser index is missing workflows"))?;

    workflows
        .iter()
        .map(|workflow| {
            let name = workflow
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("browser index workflow is missing name"))
                .and_then(parse_model_name)?;
            let path = workflow
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("browser index workflow is missing path"))?;
            let slug = path
                .strip_prefix("data/workflows/")
                .and_then(|file_name| file_name.strip_suffix(".eventmodel.json"))
                .ok_or_else(|| BoundaryParseError::new("browser index workflow path is invalid"))
                .and_then(parse_workflow_slug)?;

            Ok(ImportedWorkflowLayout::new(name, slug))
        })
        .collect()
}

pub fn parse_emc_workflow_import(
    slug: WorkflowSlug,
    raw_json: &str,
    slices: Vec<EMCSliceImport>,
) -> Result<EMCWorkflowImport, BoundaryParseError> {
    let value = serde_json::from_str::<Value>(raw_json)
        .map_err(|error| BoundaryParseError::new(format!("invalid EMC workflow JSON: {error}")))?;
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryParseError::new("EMC workflow is missing name"))
        .and_then(parse_model_name)?;
    let json = FileContents::try_new(raw_json.to_owned()).map_err(|error| {
        BoundaryParseError::new(format!("invalid EMC workflow content: {error}"))
    })?;

    Ok(EMCWorkflowImport::new(name, slug, json, slices))
}

pub fn parse_emc_slice_import(
    slug: SliceSlug,
    raw_json: &str,
) -> Result<EMCSliceImport, BoundaryParseError> {
    let value = serde_json::from_str::<Value>(raw_json)
        .map_err(|error| BoundaryParseError::new(format!("invalid EMC slice JSON: {error}")))?;
    value
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryParseError::new("EMC slice is missing name"))?;
    let json = FileContents::try_new(raw_json.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid EMC slice content: {error}")))?;

    Ok(EMCSliceImport::new(slug, json))
}

pub fn parse_workflow_slug(raw: &str) -> Result<WorkflowSlug, BoundaryParseError> {
    WorkflowSlug::try_new(slugify(raw))
        .map_err(|error| BoundaryParseError::new(format!("invalid workflow slug: {error}")))
}

pub fn parse_slice_slug(raw: &str) -> Result<SliceSlug, BoundaryParseError> {
    SliceSlug::try_new(slugify(raw))
        .map_err(|error| BoundaryParseError::new(format!("invalid slice slug: {error}")))
}

pub fn parse_lean_module_name(raw: &str) -> Result<LeanModuleName, BoundaryParseError> {
    LeanModuleName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid Lean module name: {error}")))
}

pub fn parse_quint_module_name(raw: &str) -> Result<QuintModuleName, BoundaryParseError> {
    QuintModuleName::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid Quint module name: {error}")))
}

pub fn parse_model_digest(raw: &str) -> Result<ModelDigest, BoundaryParseError> {
    ModelDigest::try_new(raw.to_owned())
        .map_err(|error| BoundaryParseError::new(format!("invalid model digest: {error}")))
}

fn slugify(raw: &str) -> String {
    raw.trim()
        .chars()
        .fold(
            (String::new(), false),
            |(mut slug, pending_dash), character| {
                if character.is_ascii_alphanumeric() {
                    if pending_dash && !slug.is_empty() {
                        slug.push('-');
                    }
                    slug.push(character.to_ascii_lowercase());
                    (slug, false)
                } else {
                    (slug, true)
                }
            },
        )
        .0
}

fn quoted_value(raw: &str) -> Option<&str> {
    raw.strip_prefix('"')?.strip_suffix('"')
}

fn event_model_document_from_json(
    value: Value,
    file_kind: EventModelFileKind,
) -> Result<EventModelDocument, BoundaryParseError> {
    let object = value
        .as_object()
        .ok_or_else(|| BoundaryParseError::new(model_must_be_object_issue().to_string()))?;
    object
        .keys()
        .map(|key| {
            TopLevelKey::try_new(key.to_owned())
                .map_err(|_| BoundaryParseError::new(empty_top_level_key_issue().to_string()))
        })
        .collect::<Result<BTreeSet<_>, _>>()
        .and_then(|top_level_keys| {
            let name = optional_definition_name_from_json_object(object, "name", "model")?;
            let slice_definitions = slice_definitions_from_json_object(object)?;
            let event_names = event_names_from_json_object(object)?;
            let view_definitions = view_definitions_from_json_object(object)?;
            let stream_names = stream_names_from_json_object(object)?;
            let event_definitions = event_definitions_from_json_object(object)?;
            let command_definitions = command_definitions_from_json_object(object)?;
            let read_model_definitions = read_model_definitions_from_json_object(object)?;
            let workflow_slice_references = workflow_slice_references_from_json_object(object)?;
            let workflow_step_slices = workflow_step_slices_from_json_object(object)?;
            let workflow_steps = workflow_steps_from_json_object(object)?;
            let duplicate_workflow_step_slice =
                duplicate_workflow_step_slice_from_json_object(object)?;
            let workflow_composition = workflow_composition_from_json_object(object);
            let workflow_entry_step_count =
                workflow_entry_step_count_from_json_object(object, workflow_composition);
            let workflow_internal_definitions =
                workflow_internal_definitions_from_json_object(object);
            let workflow_transition_errors = workflow_transition_errors_from_json_object(object)?;
            let workflow_transition_outcomes =
                workflow_transition_outcomes_from_json_object(object)?;
            let board_read_model_command_dependencies =
                board_read_model_command_dependencies_from_json_object(object)?;
            let command_produced_events = command_produced_events_from_json_object(object)?;
            let state_view_observed_events =
                state_view_observed_events_from_slices(&slice_definitions);
            named_definitions_from_json_object(object).map(|named_definitions| {
                EventModelDocument::new(
                    EventModelDocumentParts::new(file_kind)
                        .with_name(name)
                        .with_top_level_keys(top_level_keys)
                        .with_event_names(event_names)
                        .with_stream_names(stream_names)
                        .with_event_definitions(event_definitions)
                        .with_command_definitions(command_definitions)
                        .with_command_produced_events(command_produced_events)
                        .with_state_view_observed_events(state_view_observed_events)
                        .with_named_definitions(named_definitions)
                        .with_read_model_definitions(read_model_definitions)
                        .with_board_read_model_command_dependencies(
                            board_read_model_command_dependencies,
                        )
                        .with_slice_count(slice_definition_count(&slice_definitions))
                        .with_slice_definitions(slice_definitions)
                        .with_view_definitions(view_definitions)
                        .with_workflow_slice_references(workflow_slice_references)
                        .with_workflow_step_slices(workflow_step_slices)
                        .with_workflow_steps(workflow_steps)
                        .with_duplicate_workflow_step_slice(duplicate_workflow_step_slice)
                        .with_workflow_composition(workflow_composition)
                        .with_workflow_entry_step_count(workflow_entry_step_count)
                        .with_workflow_internal_definitions(workflow_internal_definitions)
                        .with_workflow_transition_errors(workflow_transition_errors)
                        .with_workflow_transition_outcomes(workflow_transition_outcomes),
                )
            })
        })
}

fn slice_definition_count(slice_definitions: &[SliceDefinition]) -> SliceDefinitionCount {
    match slice_definitions.len() {
        0 => SliceDefinitionCount::Zero,
        1 => SliceDefinitionCount::One,
        _ => SliceDefinitionCount::Multiple,
    }
}

fn workflow_composition_from_json_object(object: &Map<String, Value>) -> WorkflowComposition {
    if object.get("slice_files").is_some() {
        if object
            .get("steps")
            .and_then(Value::as_array)
            .is_some_and(|steps| !steps.is_empty())
        {
            WorkflowComposition::DeclaresSteps
        } else {
            WorkflowComposition::MissingSteps
        }
    } else {
        WorkflowComposition::NotComposition
    }
}

fn workflow_slice_references_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    object
        .get("slice_files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(workflow_slice_reference_from_path)
        .collect()
}

fn workflow_slice_reference_from_path(path: &str) -> Result<DefinitionName, BoundaryParseError> {
    let file_name = path
        .rsplit('/')
        .next()
        .and_then(|file_name| file_name.strip_suffix(".eventmodel.json"))
        .unwrap_or(path);
    DefinitionName::try_new(file_name.to_owned()).map_err(|error| {
        BoundaryParseError::new(format!("invalid workflow slice reference: {error}"))
    })
}

fn workflow_step_slices_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    workflow_step_slice_values(object)
        .map(|step_slice| {
            DefinitionName::try_new(step_slice.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid workflow step slice: {error}"))
            })
        })
        .collect()
}

fn workflow_steps_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<WorkflowStep>, BoundaryParseError> {
    object
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .filter(|step| step.get("slice").and_then(Value::as_str).is_some())
        .map(workflow_step_from_json_object)
        .collect()
}

fn workflow_step_from_json_object(
    object: &Map<String, Value>,
) -> Result<WorkflowStep, BoundaryParseError> {
    let slice = object
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryParseError::new("workflow step is missing slice"))
        .and_then(|slice| {
            DefinitionName::try_new(slice.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid workflow step slice: {error}"))
            })
        })?;
    let relationship = workflow_step_relationship_from_json_object(object);
    let trigger = workflow_step_trigger_from_json_object(object);
    let workflow_exit = workflow_step_exit_from_json_object(object);
    let transition_targets = workflow_step_transition_targets_from_json_object(object)?;
    let selected_scenario = workflow_step_selected_scenario_from_json_object(object)?;
    let lifecycle_role = workflow_step_lifecycle_role_from_json_object(object);

    Ok(WorkflowStep::new(
        slice,
        relationship,
        trigger,
        workflow_exit,
        transition_targets,
        selected_scenario,
        lifecycle_role,
    ))
}

fn workflow_step_selected_scenario_from_json_object(
    object: &Map<String, Value>,
) -> Result<Option<DefinitionName>, BoundaryParseError> {
    object
        .get("scenario")
        .and_then(Value::as_str)
        .map(|scenario| {
            DefinitionName::try_new(scenario.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid workflow step scenario: {error}"))
            })
        })
        .transpose()
}

fn workflow_step_lifecycle_role_from_json_object(
    object: &Map<String, Value>,
) -> WorkflowStepLifecycleRole {
    let label = workflow_step_label_from_json_object(object);
    if object
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|step_type| step_type == "state_change")
        && object
            .get("relationship")
            .and_then(Value::as_str)
            .is_some_and(|relationship| relationship == "entry")
        && label.contains("bootstrap")
        && label.contains("root")
    {
        WorkflowStepLifecycleRole::BootstrapRootEntryStateChange
    } else if object
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|step_type| step_type == "state_view")
        && (label.contains("application entry") || label.contains("root bootstrap"))
    {
        WorkflowStepLifecycleRole::ApplicationEntryStateView
    } else {
        WorkflowStepLifecycleRole::Other
    }
}

fn workflow_step_label_from_json_object(object: &Map<String, Value>) -> String {
    ["slice", "name", "relationship"]
        .into_iter()
        .filter_map(|key| object.get(key).and_then(Value::as_str))
        .fold(String::new(), |label, value| {
            if label.is_empty() {
                value.to_lowercase()
            } else {
                format!("{label} {}", value.to_lowercase())
            }
        })
}

fn workflow_step_relationship_from_json_object(
    object: &Map<String, Value>,
) -> WorkflowStepRelationship {
    match object.get("relationship").and_then(Value::as_str) {
        Some("alternate") => WorkflowStepRelationship::Alternate,
        Some("async_lifecycle") => WorkflowStepRelationship::AsyncLifecycle,
        Some("branch") => WorkflowStepRelationship::Branch,
        Some("entry") => WorkflowStepRelationship::Entry,
        Some("main") => WorkflowStepRelationship::Main,
        Some("supporting") => WorkflowStepRelationship::Supporting,
        Some(_) | None => WorkflowStepRelationship::Other,
    }
}

fn workflow_step_trigger_from_json_object(object: &Map<String, Value>) -> WorkflowStepTrigger {
    if object.get("trigger").and_then(Value::as_str).is_some() {
        WorkflowStepTrigger::Present
    } else {
        WorkflowStepTrigger::Absent
    }
}

fn workflow_step_exit_from_json_object(object: &Map<String, Value>) -> WorkflowStepExit {
    if object
        .get("transitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .any(|transition| {
            transition
                .get("to_workflow")
                .and_then(Value::as_str)
                .is_some()
        })
    {
        WorkflowStepExit::Present
    } else {
        WorkflowStepExit::Absent
    }
}

fn workflow_step_transition_targets_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    object
        .get("transitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .filter_map(|transition| transition.get("to").and_then(Value::as_str))
        .map(|target| {
            DefinitionName::try_new(target.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid workflow transition target: {error}"))
            })
        })
        .collect()
}

fn duplicate_workflow_step_slice_from_json_object(
    object: &Map<String, Value>,
) -> Result<Option<DefinitionName>, BoundaryParseError> {
    let mut seen = BTreeSet::new();
    workflow_step_slice_values(object)
        .find_map(|step_slice| {
            DefinitionName::try_new(step_slice.to_owned())
                .map(|step_slice| {
                    if seen.insert(step_slice.clone()) {
                        None
                    } else {
                        Some(step_slice)
                    }
                })
                .map_err(|error| {
                    BoundaryParseError::new(format!("invalid workflow step slice: {error}"))
                })
                .transpose()
        })
        .transpose()
}

fn workflow_step_slice_values(object: &Map<String, Value>) -> impl Iterator<Item = &str> {
    object
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|step| step.get("slice").and_then(Value::as_str))
}

fn workflow_entry_step_count_from_json_object(
    object: &Map<String, Value>,
    workflow_composition: WorkflowComposition,
) -> WorkflowEntryStepCount {
    if workflow_composition != WorkflowComposition::DeclaresSteps {
        WorkflowEntryStepCount::NotComposition
    } else if workflow_entry_steps_from_json_object(object) == 1 {
        WorkflowEntryStepCount::One
    } else {
        WorkflowEntryStepCount::NotOne
    }
}

fn workflow_entry_steps_from_json_object(object: &Map<String, Value>) -> usize {
    object
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|step| {
            step.get("relationship")
                .and_then(Value::as_str)
                .is_some_and(|relationship| relationship == "entry")
        })
        .count()
}

fn workflow_internal_definitions_from_json_object(
    object: &Map<String, Value>,
) -> WorkflowInternalDefinitions {
    if [
        "commands",
        "views",
        "read_models",
        "automations",
        "scenarios",
    ]
    .into_iter()
    .any(|key| {
        object
            .get(key)
            .and_then(Value::as_array)
            .is_some_and(|definitions| !definitions.is_empty())
    }) {
        WorkflowInternalDefinitions::Present
    } else {
        WorkflowInternalDefinitions::Absent
    }
}

fn workflow_transition_errors_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    workflow_transition_references_from_json_object(
        object,
        "via_error",
        "workflow transition error",
    )
}

fn workflow_transition_outcomes_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    workflow_transition_references_from_json_object(
        object,
        "via_outcome",
        "workflow transition outcome",
    )
}

fn workflow_transition_references_from_json_object(
    object: &Map<String, Value>,
    field: &str,
    label: &str,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    object
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|step| workflow_transition_references_from_json_step(step, field))
        .map(|reference| {
            DefinitionName::try_new(reference.to_owned())
                .map_err(|error| BoundaryParseError::new(format!("invalid {label}: {error}")))
        })
        .collect()
}

fn workflow_transition_references_from_json_step<'a>(step: &'a Value, field: &str) -> Vec<&'a str> {
    step.get("transitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|transition| transition.get(field).and_then(Value::as_str))
        .collect()
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BoardElementKind {
    Automation,
    Command,
    Other,
    ReadModel,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BoardElement {
    kind: BoardElementKind,
    name: DefinitionName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BoardConnection {
    from: String,
    to: String,
}

fn board_read_model_command_dependencies_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<BoardReadModelCommandDependency>, BoundaryParseError> {
    object
        .get("board")
        .and_then(|board| board.get("slices"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(board_read_model_command_dependencies_from_json_slice)
        .collect::<Result<Vec<_>, _>>()
        .map(|dependencies| dependencies.into_iter().flatten().collect())
}

fn board_read_model_command_dependencies_from_json_slice(
    board_slice: &Value,
) -> Result<Vec<BoardReadModelCommandDependency>, BoundaryParseError> {
    let elements = board_elements_from_json_slice(board_slice)?;
    let connections = board_connections_from_json_slice(board_slice);
    Ok(connections
        .iter()
        .flat_map(|connection| {
            board_read_model_command_dependency_from_connection(&elements, &connections, connection)
        })
        .collect())
}

fn board_elements_from_json_slice(
    board_slice: &Value,
) -> Result<BTreeMap<String, BoardElement>, BoundaryParseError> {
    board_slice
        .get("elements")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|element| {
            element
                .get("id")
                .and_then(Value::as_str)
                .zip(element.get("name").and_then(Value::as_str))
                .map(|(id, name)| (id, name, board_element_kind_from_json(element)))
        })
        .map(|(id, name, kind)| {
            DefinitionName::try_new(name.to_owned())
                .map(|name| (id.to_owned(), BoardElement { kind, name }))
                .map_err(|error| {
                    BoundaryParseError::new(format!("invalid board element name: {error}"))
                })
        })
        .collect()
}

fn board_element_kind_from_json(element: &Value) -> BoardElementKind {
    match element.get("kind").and_then(Value::as_str) {
        Some("automation") => BoardElementKind::Automation,
        Some("command") => BoardElementKind::Command,
        Some("read_model") => BoardElementKind::ReadModel,
        _ => BoardElementKind::Other,
    }
}

fn board_connections_from_json_slice(board_slice: &Value) -> Vec<BoardConnection> {
    board_slice
        .get("connections")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|connection| {
            connection
                .get("from")
                .and_then(Value::as_str)
                .zip(connection.get("to").and_then(Value::as_str))
                .map(|(from, to)| BoardConnection {
                    from: from.to_owned(),
                    to: to.to_owned(),
                })
        })
        .collect()
}

fn board_read_model_command_dependency_from_connection(
    elements: &BTreeMap<String, BoardElement>,
    connections: &[BoardConnection],
    connection: &BoardConnection,
) -> Vec<BoardReadModelCommandDependency> {
    let Some(read_model) = elements
        .get(&connection.from)
        .filter(|element| element.kind == BoardElementKind::ReadModel)
    else {
        return Vec::new();
    };
    let Some(intermediate) = elements
        .get(&connection.to)
        .filter(|element| element.kind == BoardElementKind::Automation)
    else {
        return Vec::new();
    };
    connections
        .iter()
        .filter(|candidate| candidate.from == connection.to)
        .filter_map(|candidate| {
            elements
                .get(&candidate.to)
                .filter(|command| command.kind == BoardElementKind::Command)
                .map(|command| {
                    BoardReadModelCommandDependency::new(
                        read_model.name.clone(),
                        command.name.clone(),
                        intermediate.name.clone(),
                    )
                })
        })
        .collect()
}

fn slice_definitions_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<SliceDefinition>, BoundaryParseError> {
    object
        .get("slices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|slice| {
            slice
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("slice is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid slice name: {error}"))
                    })
                })
                .map(|name| {
                    let issued_commands =
                        definition_names_from_json_array_field(slice, "commands", "command")?;
                    let handled_command_errors = handled_command_errors_from_json_slice(slice)?;
                    let owned_automations =
                        definition_names_from_json_array_field(slice, "automations", "automation")?;
                    let owned_read_models =
                        definition_names_from_json_array_field(slice, "read_models", "read model")?;
                    let owned_translations = definition_names_from_json_array_field(
                        slice,
                        "translations",
                        "translation",
                    )?;
                    let owned_views =
                        definition_names_from_json_array_field(slice, "views", "view")?;
                    let owned_events =
                        definition_names_from_json_array_field(slice, "events", "event")?;
                    let external_payload_variants =
                        external_payload_variants_from_json_slice(slice)?;
                    let outcome_labels = outcome_labels_from_json_slice(slice)?;
                    let outcomes = outcomes_from_json_slice(slice)?;
                    let automation_trigger = automation_trigger_from_json_slice(slice)?;
                    slice_scenarios_from_json_slice(slice).map(|scenarios| {
                        SliceDefinition::new(
                            SliceDefinitionParts::new(name, slice_type_from_json_slice(slice))
                                .with_issued_commands(issued_commands)
                                .with_handled_command_errors(handled_command_errors)
                                .with_owned_automations(owned_automations)
                                .with_owned_read_models(owned_read_models)
                                .with_owned_translations(owned_translations)
                                .with_owned_views(owned_views)
                                .with_owned_events(owned_events)
                                .with_external_payload_variants(external_payload_variants)
                                .with_outcome_labels(outcome_labels)
                                .with_outcomes(outcomes)
                                .with_legacy_scenarios(legacy_scenarios_field_from_json_slice(
                                    slice,
                                ))
                                .with_singleton_behavior(singleton_behavior_from_json_slice(slice))
                                .with_automation_trigger(automation_trigger)
                                .with_automation_command_policy(
                                    automation_command_policy_from_json_slice(slice),
                                )
                                .with_translation_contract(translation_contract_from_json_slice(
                                    slice,
                                ))
                                .with_scenarios(scenarios),
                        )
                    })
                })
                .and_then(|slice_definition| slice_definition)
        })
        .collect()
}

fn outcomes_from_json_slice(slice: &Value) -> Result<Vec<OutcomeDefinition>, BoundaryParseError> {
    slice
        .get("outcomes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|outcome| {
            outcome
                .get("label")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("outcome is missing label"))
                .and_then(|label| {
                    DefinitionName::try_new(label.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid outcome label: {error}"))
                    })
                })
                .and_then(|label| {
                    definition_names_from_json_array_field(outcome, "events", "event").map(
                        |mut events| {
                            events.sort();
                            OutcomeDefinition::new(label, events)
                        },
                    )
                })
        })
        .collect()
}

fn handled_command_errors_from_json_slice(
    slice: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    slice
        .get("error_handling")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|handling| handling.get("error").and_then(Value::as_str))
        .map(|error| {
            DefinitionName::try_new(error.to_owned()).map_err(|parse_error| {
                BoundaryParseError::new(format!("invalid handled command error: {parse_error}"))
            })
        })
        .collect()
}

fn automation_command_policy_from_json_slice(slice: &Value) -> AutomationCommandPolicy {
    if slice_type_from_json_slice(slice) == SliceType::Automation {
        if slice_command_count(slice) > 1 {
            AutomationCommandPolicy::MultipleCommands
        } else {
            AutomationCommandPolicy::SingleCommand
        }
    } else {
        AutomationCommandPolicy::NotAutomation
    }
}

fn slice_command_count(slice: &Value) -> usize {
    slice
        .get("commands")
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

fn automation_trigger_from_json_slice(
    slice: &Value,
) -> Result<AutomationTrigger, BoundaryParseError> {
    if slice_type_from_json_slice(slice) == SliceType::Automation {
        let trigger_events = automation_trigger_events_from_json_slice(slice)?;
        if trigger_events.is_empty() {
            Ok(AutomationTrigger::MissingTrigger)
        } else {
            Ok(AutomationTrigger::DeclaresTriggers(trigger_events))
        }
    } else {
        Ok(AutomationTrigger::NotAutomation)
    }
}

fn automation_trigger_events_from_json_slice(
    slice: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    optional_definition_names_from_json_fields(slice, &["trigger", "external_event"]).and_then(
        |single_triggers| {
            definition_names_from_json_array_field(slice, "triggers", "trigger").map(
                |trigger_array| {
                    single_triggers
                        .into_iter()
                        .chain(trigger_array)
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect()
                },
            )
        },
    )
}

fn translation_contract_from_json_slice(slice: &Value) -> TranslationContract {
    if slice_type_from_json_slice(slice) == SliceType::Translation {
        if slice_has_external_contract(slice) {
            TranslationContract::DeclaresExternalContract
        } else {
            TranslationContract::MissingExternalContract
        }
    } else {
        TranslationContract::NotTranslation
    }
}

fn external_payload_variants_from_json_slice(
    slice: &Value,
) -> Result<Vec<ExternalPayloadVariant>, BoundaryParseError> {
    slice
        .get("external_input_schemas")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(external_payload_variants_from_json_schema)
        .collect::<Result<Vec<_>, _>>()
        .map(|variants| variants.into_iter().flatten().collect())
}

fn external_payload_variants_from_json_schema(
    schema: &Value,
) -> Result<Vec<ExternalPayloadVariant>, BoundaryParseError> {
    schema
        .get("variants")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|variant| {
            external_payload_name_from_json_schema(schema).and_then(|payload| {
                DefinitionName::try_new(variant.to_owned())
                    .map(|variant| ExternalPayloadVariant::new(payload, variant))
                    .map_err(|error| {
                        BoundaryParseError::new(format!(
                            "invalid external payload variant: {error}"
                        ))
                    })
            })
        })
        .collect()
}

fn external_payload_name_from_json_schema(
    schema: &Value,
) -> Result<DefinitionName, BoundaryParseError> {
    schema
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryParseError::new("external input schema is missing name"))
        .and_then(|name| {
            DefinitionName::try_new(name.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid external input schema name: {error}"))
            })
        })
}

fn slice_has_external_contract(slice: &Value) -> bool {
    slice_has_non_empty_string(slice, "external_event")
        || slice
            .get("external_input_schemas")
            .and_then(Value::as_array)
            .is_some_and(|schemas| !schemas.is_empty())
}

fn slice_has_non_empty_string(slice: &Value, key: &str) -> bool {
    slice
        .get(key)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn singleton_behavior_from_json_slice(slice: &Value) -> SingletonBehavior {
    slice
        .get("singleton")
        .and_then(Value::as_bool)
        .filter(|singleton| *singleton)
        .map_or(SingletonBehavior::NotSingleton, |_| {
            if slice_declares_repeat_behavior(slice) {
                SingletonBehavior::DeclaresRepeatBehavior
            } else {
                SingletonBehavior::MissingRepeatBehavior
            }
        })
}

fn slice_declares_repeat_behavior(slice: &Value) -> bool {
    first_class_scenario_fields()
        .iter()
        .filter_map(|spec| slice.get(spec.key).and_then(Value::as_array))
        .flatten()
        .map(|scenario| scenario.to_string().to_lowercase())
        .any(|scenario_text| {
            scenario_text.contains("already-exists")
                || scenario_text.contains("already exists")
                || scenario_text.contains("idempotent")
        })
}

fn legacy_scenarios_field_from_json_slice(slice: &Value) -> LegacyScenariosField {
    if slice.get("scenarios").is_some() {
        LegacyScenariosField::Present
    } else {
        LegacyScenariosField::Absent
    }
}

fn slice_scenarios_from_json_slice(
    slice: &Value,
) -> Result<Vec<SliceScenario>, BoundaryParseError> {
    first_class_scenario_fields()
        .iter()
        .map(|spec| slice_scenarios_from_json_field(slice, spec))
        .collect::<Result<Vec<_>, _>>()
        .map(|scenarios| scenarios.into_iter().flatten().collect())
}

fn outcome_labels_from_json_slice(
    slice: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    slice
        .get("outcomes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|outcome| outcome.get("label").and_then(Value::as_str))
        .map(|label| {
            DefinitionName::try_new(label.to_owned())
                .map_err(|error| BoundaryParseError::new(format!("invalid outcome label: {error}")))
        })
        .collect()
}

fn slice_scenarios_from_json_field(
    slice: &Value,
    spec: &ScenarioFieldSpec,
) -> Result<Vec<SliceScenario>, BoundaryParseError> {
    slice
        .get(spec.key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|scenario| {
            scenario
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("scenario is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid scenario name: {error}"))
                    })
                })
                .map(|name| {
                    let read_model_states = read_model_states_from_json_scenario(scenario)?;
                    let read_model_state_values =
                        read_model_state_values_from_json_scenario(scenario)?;
                    let then_events = event_references_from_json_field(scenario, "then")?;
                    let command_errors = command_errors_from_json_scenario(scenario)?;
                    let given_streams = given_streams_from_json_scenario(scenario)?;
                    let scenario_step_references =
                        scenario_step_references_from_json_scenario(scenario)?;
                    event_references_from_json_scenario(scenario).map(|referenced_events| {
                        SliceScenario::new(
                            SliceScenarioParts::new(
                                name,
                                scenario_step_field(scenario, "when"),
                                spec.kind,
                            )
                            .with_referenced_events(referenced_events)
                            .with_scenario_step_references(scenario_step_references)
                            .with_then_events(then_events)
                            .with_command_errors(command_errors)
                            .with_given_streams(given_streams)
                            .with_read_model_states(read_model_states)
                            .with_read_model_state_values(read_model_state_values),
                        )
                    })
                })
                .and_then(|scenario| scenario)
        })
        .collect()
}
fn given_streams_from_json_scenario(
    scenario: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario
        .get("given_streams")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|given_stream| given_stream.get("stream").and_then(Value::as_str))
        .map(|stream| {
            DefinitionName::try_new(stream.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid given stream reference: {error}"))
            })
        })
        .collect()
}

fn read_model_states_from_json_scenario(
    scenario: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario
        .get("read_model_states")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(Map::keys)
        .map(|read_model| {
            DefinitionName::try_new(read_model.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid read model state name: {error}"))
            })
        })
        .collect()
}

fn read_model_state_values_from_json_scenario(
    scenario: &Value,
) -> Result<Vec<ReadModelState>, BoundaryParseError> {
    scenario
        .get("read_model_states")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(Map::iter)
        .map(|(read_model, state)| {
            let read_model = DefinitionName::try_new(read_model.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid read model state name: {error}"))
            })?;
            let state = state
                .as_str()
                .ok_or_else(|| BoundaryParseError::new("read model state value must be a string"))
                .and_then(|state| {
                    DefinitionName::try_new(state.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid read model state value: {error}"))
                    })
                })?;
            Ok(ReadModelState::new(read_model, state))
        })
        .collect()
}

fn command_errors_from_json_scenario(
    scenario: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario_reference_fields()
        .iter()
        .filter_map(|field| scenario.get(field).and_then(Value::as_array))
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(command_error_from_scenario_reference)
        .map(|error_name| {
            DefinitionName::try_new(error_name.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid scenario command error: {error}"))
            })
        })
        .collect()
}

fn command_error_from_scenario_reference(reference: &str) -> Option<&str> {
    reference
        .strip_prefix("error ")
        .and_then(|raw_error| raw_error.strip_suffix(" is returned"))
}

fn event_references_from_json_scenario(
    scenario: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario_reference_fields()
        .iter()
        .map(|field| event_references_from_json_field(scenario, field))
        .collect::<Result<Vec<_>, _>>()
        .map(|references| references.into_iter().flatten().collect())
}

fn scenario_step_references_from_json_scenario(
    scenario: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario_reference_fields()
        .iter()
        .map(|field| scenario_step_references_from_json_field(scenario, field))
        .collect::<Result<Vec<_>, _>>()
        .map(|references| references.into_iter().flatten().collect())
}

fn scenario_step_references_from_json_field(
    scenario: &Value,
    field: &str,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario
        .get(field)
        .into_iter()
        .flat_map(scenario_step_reference_values)
        .map(|reference| {
            DefinitionName::try_new(reference.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid scenario step reference: {error}"))
            })
        })
        .collect()
}

fn scenario_step_reference_values(value: &Value) -> Vec<&str> {
    value.as_str().map_or_else(
        || {
            value
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect()
        },
        |reference| vec![reference],
    )
}

fn event_references_from_json_field(
    scenario: &Value,
    field: &str,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|reference| {
            DefinitionName::try_new(reference.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid scenario reference: {error}"))
            })
        })
        .collect()
}

fn scenario_reference_fields() -> &'static [&'static str] {
    &["given", "when", "then"]
}

fn scenario_step_field(scenario: &Value, field: &str) -> ScenarioStepField {
    if scenario.get(field).is_some() {
        ScenarioStepField::Present
    } else {
        ScenarioStepField::Absent
    }
}

struct ScenarioFieldSpec {
    key: &'static str,
    kind: ScenarioSetKind,
}

fn first_class_scenario_fields() -> &'static [ScenarioFieldSpec] {
    &[
        ScenarioFieldSpec {
            key: "acceptance_scenarios",
            kind: ScenarioSetKind::Acceptance,
        },
        ScenarioFieldSpec {
            key: "contract_scenarios",
            kind: ScenarioSetKind::Contract,
        },
    ]
}

fn slice_type_from_json_slice(slice: &Value) -> SliceType {
    match slice.get("type").and_then(Value::as_str) {
        Some("automation") => SliceType::Automation,
        Some("state_change") => SliceType::StateChange,
        Some("state_view") => SliceType::StateView,
        Some("translation") => SliceType::Translation,
        _ => SliceType::Other,
    }
}

fn view_definitions_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<ViewDefinition>, BoundaryParseError> {
    object
        .get("views")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|view| {
            view.get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("view is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid view name: {error}"))
                    })
                })
                .and_then(|name| {
                    let controls = view_control_definitions_from_json_view(view)?;
                    let local_states = definition_names_from_json_array_field(
                        view,
                        "local_states",
                        "local state",
                    )?;
                    let wireframe = view_wireframe_from_json_view(view);
                    definition_names_from_json_array_field(view, "uses_read_models", "read model")
                        .map(|read_models| {
                            ViewDefinition::new(
                                name,
                                read_models,
                                controls,
                                local_states,
                                wireframe,
                            )
                        })
                })
        })
        .collect()
}

fn view_wireframe_from_json_view(view: &Value) -> ViewWireframe {
    view.get("wireframe")
        .and_then(Value::as_str)
        .filter(|wireframe| !wireframe.trim().is_empty())
        .map_or(ViewWireframe::Absent, |_| ViewWireframe::Present)
}

fn view_control_definitions_from_json_view(
    view: &Value,
) -> Result<Vec<ViewControlDefinition>, BoundaryParseError> {
    view.get("controls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|control| {
            control
                .get("label")
                .and_then(Value::as_str)
                .map(|label| (control, label))
        })
        .map(|(control, label)| {
            let command = optional_definition_name_from_json_field(control, "command", "command")?;
            let navigation_target =
                optional_definition_name_from_json_field(control, "navigation", "navigation")?;
            let workflow_target =
                optional_definition_name_from_json_field(control, "workflow_target", "workflow")?;
            let external_system = optional_definition_name_from_json_field(
                control,
                "external_system",
                "external system",
            )?;
            let payload_contract = optional_definition_name_from_json_field(
                control,
                "payload_contract",
                "payload contract",
            )?;
            let navigation_type = navigation_type_from_json_control(control);
            let command_error_handling = command_error_handling_from_json_control(control)?;
            DefinitionName::try_new(label.to_owned())
                .map(|label| {
                    ViewControlDefinition::new(
                        ViewControlDefinitionParts::new(label)
                            .with_command(command)
                            .with_command_error_handling(command_error_handling)
                            .with_navigation_target(navigation_target)
                            .with_navigation_type(navigation_type)
                            .with_workflow_target(workflow_target)
                            .with_external_system(external_system)
                            .with_payload_contract(payload_contract),
                    )
                })
                .map_err(|error| BoundaryParseError::new(format!("invalid control label: {error}")))
        })
        .collect()
}

fn navigation_type_from_json_control(control: &Value) -> NavigationType {
    match control.get("navigation_type").and_then(Value::as_str) {
        Some("modeled_view") => NavigationType::ModeledView,
        Some("local_view_state") => NavigationType::LocalViewState,
        Some("external_system") => NavigationType::ExternalSystem,
        Some("external_workflow") => NavigationType::ExternalWorkflow,
        _ => NavigationType::Absent,
    }
}

fn command_error_handling_from_json_control(
    control: &Value,
) -> Result<Vec<ControlCommandErrorHandling>, BoundaryParseError> {
    control
        .get("error_handling")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|handling| {
            handling
                .get("error")
                .and_then(Value::as_str)
                .map(|error_name| (handling, error_name))
        })
        .map(|(handling, error_name)| {
            DefinitionName::try_new(error_name.to_owned())
                .map_err(|error| {
                    BoundaryParseError::new(format!("invalid control command error: {error}"))
                })
                .map(|error_name| {
                    ControlCommandErrorHandling::new(
                        error_name,
                        control_error_recovery_behavior_from_json_handling(handling),
                    )
                })
        })
        .collect()
}

fn control_error_recovery_behavior_from_json_handling(
    handling: &Value,
) -> ControlErrorRecoveryBehavior {
    if handling_has_recovery_action(handling)
        || handling_has_navigation_target(handling)
        || handling_has_retry(handling)
        || handling_stays_on_screen(handling)
    {
        ControlErrorRecoveryBehavior::Present
    } else {
        ControlErrorRecoveryBehavior::Missing
    }
}

fn handling_has_recovery_action(handling: &Value) -> bool {
    handling
        .get("recovery_action")
        .and_then(Value::as_str)
        .is_some_and(|recovery_action| !recovery_action.trim().is_empty())
}

fn handling_has_navigation_target(handling: &Value) -> bool {
    handling
        .get("navigation")
        .and_then(Value::as_str)
        .is_some_and(|navigation| !navigation.trim().is_empty())
}

fn handling_has_retry(handling: &Value) -> bool {
    handling
        .get("retry")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn handling_stays_on_screen(handling: &Value) -> bool {
    handling
        .get("stay_on_screen")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn read_model_definitions_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<ReadModelDefinition>, BoundaryParseError> {
    object
        .get("read_models")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|read_model| {
            read_model
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("read model is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid read model name: {error}"))
                    })
                })
                .and_then(|name| {
                    read_model_fields_from_json_read_model(read_model).map(|fields| {
                        let transitive_derivation =
                            read_model_transitive_derivation_from_json(read_model, &fields);
                        ReadModelDefinition::new(name, fields, transitive_derivation)
                    })
                })
        })
        .collect()
}

fn read_model_fields_from_json_read_model(
    read_model: &Value,
) -> Result<Vec<ReadModelField>, BoundaryParseError> {
    read_model
        .get("fields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|field| {
            field
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("read model field is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid read model field name: {error}"))
                    })
                })
                .map(|name| {
                    ReadModelField::new(
                        name,
                        read_model_field_source_from_json(field),
                        read_model_field_derivation_from_json(field),
                        read_model_field_absence_default_from_json(field),
                    )
                })
        })
        .collect()
}

fn read_model_field_source_from_json(field: &Value) -> ReadModelFieldSource {
    field
        .get("source")
        .and_then(Value::as_str)
        .and_then(read_model_event_attribute_source)
        .unwrap_or(ReadModelFieldSource::Other)
}

fn read_model_event_attribute_source(source: &str) -> Option<ReadModelFieldSource> {
    if let Some(derivation_name) = source.strip_prefix("derivation.") {
        return DefinitionName::try_new(derivation_name.to_owned())
            .ok()
            .map(ReadModelFieldSource::Derivation);
    }
    let (event_name, attribute_name) = source.split_once('.')?;
    DefinitionName::try_new(event_name.to_owned())
        .ok()
        .zip(DefinitionName::try_new(attribute_name.to_owned()).ok())
        .map(|(event_name, attribute_name)| {
            ReadModelFieldSource::EventAttribute(event_name, attribute_name)
        })
}

fn read_model_field_derivation_from_json(field: &Value) -> ReadModelFieldDerivation {
    field
        .get("derived")
        .and_then(Value::as_bool)
        .filter(|derived| *derived)
        .map_or(ReadModelFieldDerivation::NotDerived, |_| {
            if !read_model_field_has_derivation_provenance(field) {
                ReadModelFieldDerivation::DerivedWithoutProvenance
            } else if !read_model_field_has_derivation_scenarios(field) {
                ReadModelFieldDerivation::DerivedWithoutScenarios
            } else {
                ReadModelFieldDerivation::DerivedComplete
            }
        })
}

fn read_model_field_has_derivation_provenance(field: &Value) -> bool {
    field
        .get("derivation_source_fields")
        .and_then(Value::as_array)
        .is_some_and(|source_fields| !source_fields.is_empty())
        && field
            .get("derivation_description")
            .and_then(Value::as_str)
            .is_some_and(|description| !description.is_empty())
}

fn read_model_field_has_derivation_scenarios(field: &Value) -> bool {
    field
        .get("derivation_scenarios")
        .and_then(Value::as_array)
        .is_some_and(|scenarios| !scenarios.is_empty())
}

fn read_model_field_absence_default_from_json(field: &Value) -> ReadModelFieldAbsenceDefault {
    field
        .get("defaulted_from_absence")
        .and_then(Value::as_bool)
        .filter(|defaulted| *defaulted)
        .map_or(ReadModelFieldAbsenceDefault::NotDefaulted, |_| {
            if !read_model_field_has_absence_event(field) {
                ReadModelFieldAbsenceDefault::DefaultedWithoutAbsenceEvent
            } else if !read_model_field_has_absence_scenarios(field) {
                ReadModelFieldAbsenceDefault::DefaultedWithoutScenarios
            } else {
                ReadModelFieldAbsenceDefault::DefaultedComplete
            }
        })
}

fn read_model_field_has_absence_event(field: &Value) -> bool {
    field
        .get("absence_event")
        .and_then(Value::as_str)
        .is_some_and(|event| !event.is_empty())
}

fn read_model_field_has_absence_scenarios(field: &Value) -> bool {
    field
        .get("absence_scenarios")
        .and_then(Value::as_array)
        .is_some_and(|scenarios| !scenarios.is_empty())
}

fn read_model_transitive_derivation_from_json(
    read_model: &Value,
    fields: &[ReadModelField],
) -> ReadModelTransitiveDerivation {
    read_model
        .get("transitive")
        .and_then(Value::as_bool)
        .filter(|transitive| *transitive)
        .map_or(ReadModelTransitiveDerivation::NotTransitive, |_| {
            if read_model_has_complete_transitive_derivation(read_model, fields) {
                ReadModelTransitiveDerivation::TransitiveComplete
            } else {
                ReadModelTransitiveDerivation::TransitiveWithoutRule
            }
        })
}

fn read_model_has_complete_transitive_derivation(
    read_model: &Value,
    fields: &[ReadModelField],
) -> bool {
    read_model
        .get("fields")
        .and_then(Value::as_array)
        .is_some_and(|field_records| {
            field_records.len() == fields.len()
                && field_records
                    .iter()
                    .all(read_model_field_has_complete_transitive_derivation)
        })
}

fn read_model_field_has_complete_transitive_derivation(field: &Value) -> bool {
    field
        .get("source_relationship_fields")
        .and_then(Value::as_array)
        .is_some_and(|source_fields| !source_fields.is_empty())
        && field
            .get("transitive_derivation_rule")
            .and_then(Value::as_str)
            .is_some_and(|rule| !rule.is_empty())
        && read_model_field_has_derivation_scenarios(field)
}

fn definition_names_from_json_array_field(
    object: &Value,
    field: &str,
    label: &str,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    object
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|value| {
            DefinitionName::try_new(value.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid {label} reference: {error}"))
            })
        })
        .collect()
}

fn named_definitions_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<NamedDefinition>, BoundaryParseError> {
    named_definition_specs()
        .iter()
        .map(|spec| named_definitions_for_spec(object, spec))
        .collect::<Result<Vec<_>, _>>()
        .map(|definitions| definitions.into_iter().flatten().collect())
}

fn event_names_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    named_definitions_for_spec(
        object,
        &NamedDefinitionSpec {
            key: "events",
            label: "event",
            kind: DefinitionKind::Event,
        },
    )
    .map(|definitions| {
        definitions
            .into_iter()
            .map(NamedDefinition::into_name)
            .collect()
    })
}

fn stream_names_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    named_definitions_for_spec(
        object,
        &NamedDefinitionSpec {
            key: "streams",
            label: "stream",
            kind: DefinitionKind::Stream,
        },
    )
    .map(|definitions| {
        definitions
            .into_iter()
            .map(NamedDefinition::into_name)
            .collect()
    })
}

fn event_definitions_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<EventDefinition>, BoundaryParseError> {
    object
        .get("events")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|event| {
            event
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("event is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid event name: {error}"))
                    })
                })
                .and_then(|name| {
                    let attributes = event_attributes_from_json_event(event)?;
                    optional_definition_name_from_json_field(event, "stream", "stream")
                        .map(|stream| EventDefinition::new(name, stream, attributes))
                })
        })
        .collect()
}

fn event_attributes_from_json_event(
    event: &Value,
) -> Result<Vec<EventAttribute>, BoundaryParseError> {
    event
        .get("attributes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|attribute| {
            attribute
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("event attribute is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid event attribute name: {error}"))
                    })
                })
                .map(|name| EventAttribute::new(name, event_attribute_source_from_json(attribute)))
        })
        .collect()
}

fn event_attribute_source_from_json(attribute: &Value) -> EventAttributeSource {
    let source = attribute.get("source").and_then(Value::as_str);

    source
        .and_then(command_attribute_source)
        .or_else(|| source.and_then(external_attribute_source))
        .or_else(|| source.and_then(generated_attribute_source))
        .or_else(|| source.and_then(read_model_attribute_source))
        .unwrap_or(EventAttributeSource::Other)
}

fn command_attribute_source(source: &str) -> Option<EventAttributeSource> {
    source
        .strip_prefix("command.")
        .and_then(|input_name| DefinitionName::try_new(input_name.to_owned()).ok())
        .map(EventAttributeSource::CommandInput)
}

fn external_attribute_source(source: &str) -> Option<EventAttributeSource> {
    let external_reference = source.strip_prefix("external.")?;
    let (payload_name, field_name) = external_reference.split_once('.')?;
    DefinitionName::try_new(payload_name.to_owned())
        .ok()
        .zip(DefinitionName::try_new(field_name.to_owned()).ok())
        .map(|(payload_name, field_name)| {
            EventAttributeSource::ExternalField(payload_name, field_name)
        })
}

fn read_model_attribute_source(source: &str) -> Option<EventAttributeSource> {
    let read_model_reference = source.strip_prefix("read_model.")?;
    let (read_model_name, field_name) = read_model_reference.split_once('.')?;
    DefinitionName::try_new(read_model_name.to_owned())
        .ok()
        .zip(DefinitionName::try_new(field_name.to_owned()).ok())
        .map(|(read_model_name, field_name)| {
            EventAttributeSource::ReadModelField(read_model_name, field_name)
        })
}

fn generated_attribute_source(source: &str) -> Option<EventAttributeSource> {
    (source == "generated.").then_some(EventAttributeSource::GeneratedEmpty)
}

fn command_definitions_from_json_object(
    object: &Map<String, Value>,
) -> Result<Vec<CommandDefinition>, BoundaryParseError> {
    object
        .get("commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|command| {
            let name = optional_definition_name_from_json_field(command, "name", "command")?;
            let inputs = definition_names_from_json_array_field(command, "inputs", "input")?;
            let input_sources = command_input_sources_from_json_command(command)?;
            let read_model_reads = command_read_model_reads_from_json_command(command);
            let external_inputs = definition_names_from_json_array_field(
                command,
                "external_inputs",
                "external input",
            )?;
            let external_input_schemas = external_input_schemas_from_json_command(command)?;
            let command_errors = command_errors_from_json_command(command)?;
            definition_names_from_json_array_field(command, "produces", "event").map(|produces| {
                CommandDefinition::new(
                    CommandDefinitionParts::new(name)
                        .with_inputs(inputs)
                        .with_input_sources(input_sources)
                        .with_read_model_reads(read_model_reads)
                        .with_external_inputs(external_inputs)
                        .with_external_input_schemas(external_input_schemas)
                        .with_produces(produces)
                        .with_errors(command_errors),
                )
            })
        })
        .collect()
}

fn command_errors_from_json_command(
    command: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    command
        .get("errors")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(command_error_name)
        .map(|error_name| {
            DefinitionName::try_new(error_name.to_owned())
                .map_err(|error| BoundaryParseError::new(format!("invalid command error: {error}")))
        })
        .collect()
}

fn command_error_name(error: &Value) -> Option<&str> {
    error
        .as_str()
        .or_else(|| error.get("name").and_then(Value::as_str))
}

fn command_read_model_reads_from_json_command(command: &Value) -> CommandReadModelReads {
    command
        .get("reads")
        .and_then(Value::as_array)
        .filter(|reads| !reads.is_empty())
        .map_or(CommandReadModelReads::Absent, |_| {
            CommandReadModelReads::Present
        })
}

fn command_input_sources_from_json_command(
    command: &Value,
) -> Result<Vec<CommandInputSource>, BoundaryParseError> {
    command
        .get("input_sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(command_input_source_from_json_source)
        .collect()
}

fn command_input_source_from_json_source(
    input_source: &Value,
) -> Result<CommandInputSource, BoundaryParseError> {
    input_source
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryParseError::new("command input source is missing name"))
        .and_then(|name| {
            DefinitionName::try_new(name.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid command input source name: {error}"))
            })
        })
        .map(|name| {
            CommandInputSource::new(name, command_input_source_kind_from_json(input_source))
        })
}

fn command_input_source_kind_from_json(input_source: &Value) -> CommandInputSourceKind {
    input_source
        .get("source")
        .and_then(Value::as_str)
        .and_then(command_external_input_source)
        .unwrap_or(CommandInputSourceKind::Other)
}

fn command_external_input_source(source: &str) -> Option<CommandInputSourceKind> {
    let external_reference = source.strip_prefix("external.")?;
    let (payload_name, field_name) = external_reference.split_once('.')?;
    DefinitionName::try_new(payload_name.to_owned())
        .ok()
        .zip(DefinitionName::try_new(field_name.to_owned()).ok())
        .map(|(payload_name, field_name)| {
            CommandInputSourceKind::ExternalField(payload_name, field_name)
        })
}

fn external_input_schemas_from_json_command(
    command: &Value,
) -> Result<Vec<ExternalInputSchema>, BoundaryParseError> {
    command
        .get("external_input_schemas")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|schema| {
            schema
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new("external input schema is missing name"))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!(
                            "invalid external input schema name: {error}"
                        ))
                    })
                })
                .and_then(|name| {
                    schema_fields_from_json_schema(schema)
                        .map(|fields| ExternalInputSchema::new(name, fields))
                })
        })
        .collect()
}

fn schema_fields_from_json_schema(
    schema: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    schema
        .get("fields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|field| {
            field
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    BoundaryParseError::new("external input schema field is missing name")
                })
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!(
                            "invalid external input schema field name: {error}"
                        ))
                    })
                })
        })
        .collect()
}

fn command_produced_events_from_json_object(
    object: &Map<String, Value>,
) -> Result<BTreeSet<DefinitionName>, BoundaryParseError> {
    object
        .get("commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|command| definition_names_from_json_array_field(command, "produces", "event"))
        .collect::<Result<Vec<_>, _>>()
        .map(|events| events.into_iter().flatten().collect())
}

fn state_view_observed_events_from_slices(
    slice_definitions: &[SliceDefinition],
) -> BTreeSet<DefinitionName> {
    slice_definitions
        .iter()
        .filter(|slice| slice.is_state_view())
        .flat_map(SliceDefinition::owned_events)
        .cloned()
        .collect()
}

fn optional_definition_name_from_json_field(
    object: &Value,
    field: &str,
    label: &str,
) -> Result<Option<DefinitionName>, BoundaryParseError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(|value| {
            DefinitionName::try_new(value.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid {label} reference: {error}"))
            })
        })
        .transpose()
}

fn optional_definition_name_from_json_object(
    object: &Map<String, Value>,
    field: &str,
    label: &str,
) -> Result<Option<DefinitionName>, BoundaryParseError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(|value| {
            DefinitionName::try_new(value.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid {label} reference: {error}"))
            })
        })
        .transpose()
}

fn optional_definition_names_from_json_fields(
    object: &Value,
    fields: &[&str],
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    fields
        .iter()
        .filter_map(|field| {
            object
                .get(field)
                .and_then(Value::as_str)
                .map(|value| (*field, value))
        })
        .filter(|(_, value)| !value.trim().is_empty())
        .map(|(field, value)| {
            DefinitionName::try_new(value.to_owned()).map_err(|error| {
                BoundaryParseError::new(format!("invalid {field} reference: {error}"))
            })
        })
        .collect()
}

fn named_definitions_for_spec(
    object: &Map<String, Value>,
    spec: &NamedDefinitionSpec,
) -> Result<Vec<NamedDefinition>, BoundaryParseError> {
    object
        .get(spec.key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|definition| {
            definition
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BoundaryParseError::new(format!("{} is missing name", spec.label)))
                .and_then(|name| {
                    DefinitionName::try_new(name.to_owned()).map_err(|error| {
                        BoundaryParseError::new(format!("invalid {} name: {error}", spec.label))
                    })
                })
                .map(|name| NamedDefinition::new(spec.kind, name))
        })
        .collect()
}

struct NamedDefinitionSpec {
    key: &'static str,
    label: &'static str,
    kind: DefinitionKind,
}

fn named_definition_specs() -> &'static [NamedDefinitionSpec] {
    &[
        NamedDefinitionSpec {
            key: "streams",
            label: "stream",
            kind: DefinitionKind::Stream,
        },
        NamedDefinitionSpec {
            key: "events",
            label: "event",
            kind: DefinitionKind::Event,
        },
        NamedDefinitionSpec {
            key: "commands",
            label: "command",
            kind: DefinitionKind::Command,
        },
        NamedDefinitionSpec {
            key: "read_models",
            label: "read model",
            kind: DefinitionKind::ReadModel,
        },
        NamedDefinitionSpec {
            key: "views",
            label: "view",
            kind: DefinitionKind::View,
        },
    ]
}
