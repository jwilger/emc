use std::collections::BTreeSet;
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
    CommandDefinition, DefinitionKind, DefinitionName, EventAttribute, EventAttributeSource,
    EventDefinition, EventModelDocument, EventModelDocumentParts, EventModelFileKind,
    ExternalInputSchema, LegacyScenariosField, NamedDefinition, ReadModelDefinition,
    ReadModelField, ReadModelFieldSource, ScenarioSetKind, ScenarioStepField, SliceDefinition,
    SliceDefinitionCount, SliceScenario, SliceType, TopLevelKey, ViewDefinition,
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
            let slice_definitions = slice_definitions_from_json_object(object)?;
            let event_names = event_names_from_json_object(object)?;
            let view_definitions = view_definitions_from_json_object(object)?;
            let stream_names = stream_names_from_json_object(object)?;
            let event_definitions = event_definitions_from_json_object(object)?;
            let command_definitions = command_definitions_from_json_object(object)?;
            let read_model_definitions = read_model_definitions_from_json_object(object)?;
            let command_produced_events = command_produced_events_from_json_object(object)?;
            let state_view_observed_events =
                state_view_observed_events_from_slices(&slice_definitions);
            named_definitions_from_json_object(object).map(|named_definitions| {
                EventModelDocument::new(
                    EventModelDocumentParts::new(file_kind)
                        .with_top_level_keys(top_level_keys)
                        .with_event_names(event_names)
                        .with_stream_names(stream_names)
                        .with_event_definitions(event_definitions)
                        .with_command_definitions(command_definitions)
                        .with_command_produced_events(command_produced_events)
                        .with_state_view_observed_events(state_view_observed_events)
                        .with_named_definitions(named_definitions)
                        .with_read_model_definitions(read_model_definitions)
                        .with_slice_count(slice_definition_count(&slice_definitions))
                        .with_slice_definitions(slice_definitions)
                        .with_view_definitions(view_definitions),
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
                    let owned_views =
                        definition_names_from_json_array_field(slice, "views", "view")?;
                    let owned_events =
                        definition_names_from_json_array_field(slice, "events", "event")?;
                    let outcome_labels = outcome_labels_from_json_slice(slice)?;
                    slice_scenarios_from_json_slice(slice).map(|scenarios| {
                        SliceDefinition::new(
                            name,
                            slice_type_from_json_slice(slice),
                            owned_views,
                            owned_events,
                            outcome_labels,
                            legacy_scenarios_field_from_json_slice(slice),
                            scenarios,
                        )
                    })
                })
                .and_then(|slice_definition| slice_definition)
        })
        .collect()
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
                    event_references_from_json_scenario(scenario).map(|referenced_events| {
                        SliceScenario::new(
                            name,
                            scenario_step_field(scenario, "when"),
                            spec.kind,
                            referenced_events,
                            read_model_states,
                        )
                    })
                })
                .and_then(|scenario| scenario)
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

fn event_references_from_json_scenario(
    scenario: &Value,
) -> Result<Vec<DefinitionName>, BoundaryParseError> {
    scenario_reference_fields()
        .iter()
        .flat_map(|field| scenario.get(field).and_then(Value::as_array))
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
    slice
        .get("type")
        .and_then(Value::as_str)
        .filter(|slice_type| *slice_type == "state_view")
        .map_or(SliceType::Other, |_| SliceType::StateView)
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
                    definition_names_from_json_array_field(view, "uses_read_models", "read model")
                        .map(|read_models| ViewDefinition::new(name, read_models))
                })
        })
        .collect()
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
                    read_model_fields_from_json_read_model(read_model)
                        .map(|fields| ReadModelDefinition::new(name, fields))
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
                .map(|name| ReadModelField::new(name, read_model_field_source_from_json(field)))
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
    let (event_name, attribute_name) = source.split_once('.')?;
    DefinitionName::try_new(event_name.to_owned())
        .ok()
        .zip(DefinitionName::try_new(attribute_name.to_owned()).ok())
        .map(|(event_name, attribute_name)| {
            ReadModelFieldSource::EventAttribute(event_name, attribute_name)
        })
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
            let inputs = definition_names_from_json_array_field(command, "inputs", "input")?;
            let external_inputs = definition_names_from_json_array_field(
                command,
                "external_inputs",
                "external input",
            )?;
            let external_input_schemas = external_input_schemas_from_json_command(command)?;
            definition_names_from_json_array_field(command, "produces", "event").map(|produces| {
                CommandDefinition::new(inputs, external_inputs, external_input_schemas, produces)
            })
        })
        .collect()
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
