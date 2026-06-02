use std::collections::{BTreeMap, BTreeSet};

use nutype::nutype;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventModelDocument {
    file_kind: EventModelFileKind,
    top_level_keys: BTreeSet<TopLevelKey>,
    event_names: BTreeSet<DefinitionName>,
    stream_names: BTreeSet<DefinitionName>,
    event_definitions: Vec<EventDefinition>,
    command_definitions: Vec<CommandDefinition>,
    command_produced_events: BTreeSet<DefinitionName>,
    state_view_observed_events: BTreeSet<DefinitionName>,
    named_definitions: Vec<NamedDefinition>,
    read_model_definitions: Vec<ReadModelDefinition>,
    slice_count: SliceDefinitionCount,
    slice_definitions: Vec<SliceDefinition>,
    view_definitions: Vec<ViewDefinition>,
}

impl EventModelDocument {
    pub fn new(parts: EventModelDocumentParts) -> Self {
        Self {
            file_kind: parts.file_kind,
            top_level_keys: parts.top_level_keys,
            event_names: parts.event_names,
            stream_names: parts.stream_names,
            event_definitions: parts.event_definitions,
            command_definitions: parts.command_definitions,
            command_produced_events: parts.command_produced_events,
            state_view_observed_events: parts.state_view_observed_events,
            named_definitions: parts.named_definitions,
            read_model_definitions: parts.read_model_definitions,
            slice_count: parts.slice_count,
            slice_definitions: parts.slice_definitions,
            view_definitions: parts.view_definitions,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventModelDocumentParts {
    file_kind: EventModelFileKind,
    top_level_keys: BTreeSet<TopLevelKey>,
    event_names: BTreeSet<DefinitionName>,
    stream_names: BTreeSet<DefinitionName>,
    event_definitions: Vec<EventDefinition>,
    command_definitions: Vec<CommandDefinition>,
    command_produced_events: BTreeSet<DefinitionName>,
    state_view_observed_events: BTreeSet<DefinitionName>,
    named_definitions: Vec<NamedDefinition>,
    read_model_definitions: Vec<ReadModelDefinition>,
    slice_count: SliceDefinitionCount,
    slice_definitions: Vec<SliceDefinition>,
    view_definitions: Vec<ViewDefinition>,
}

impl EventModelDocumentParts {
    pub fn new(file_kind: EventModelFileKind) -> Self {
        Self {
            file_kind,
            top_level_keys: BTreeSet::new(),
            event_names: BTreeSet::new(),
            stream_names: BTreeSet::new(),
            event_definitions: Vec::new(),
            command_definitions: Vec::new(),
            command_produced_events: BTreeSet::new(),
            state_view_observed_events: BTreeSet::new(),
            named_definitions: Vec::new(),
            read_model_definitions: Vec::new(),
            slice_count: SliceDefinitionCount::Zero,
            slice_definitions: Vec::new(),
            view_definitions: Vec::new(),
        }
    }

    pub fn with_top_level_keys(mut self, top_level_keys: BTreeSet<TopLevelKey>) -> Self {
        self.top_level_keys = top_level_keys;
        self
    }

    pub fn with_event_names(mut self, event_names: BTreeSet<DefinitionName>) -> Self {
        self.event_names = event_names;
        self
    }

    pub fn with_stream_names(mut self, stream_names: BTreeSet<DefinitionName>) -> Self {
        self.stream_names = stream_names;
        self
    }

    pub fn with_event_definitions(mut self, event_definitions: Vec<EventDefinition>) -> Self {
        self.event_definitions = event_definitions;
        self
    }

    pub fn with_command_definitions(mut self, command_definitions: Vec<CommandDefinition>) -> Self {
        self.command_definitions = command_definitions;
        self
    }

    pub fn with_command_produced_events(
        mut self,
        command_produced_events: BTreeSet<DefinitionName>,
    ) -> Self {
        self.command_produced_events = command_produced_events;
        self
    }

    pub fn with_state_view_observed_events(
        mut self,
        state_view_observed_events: BTreeSet<DefinitionName>,
    ) -> Self {
        self.state_view_observed_events = state_view_observed_events;
        self
    }

    pub fn with_named_definitions(mut self, named_definitions: Vec<NamedDefinition>) -> Self {
        self.named_definitions = named_definitions;
        self
    }

    pub fn with_read_model_definitions(
        mut self,
        read_model_definitions: Vec<ReadModelDefinition>,
    ) -> Self {
        self.read_model_definitions = read_model_definitions;
        self
    }

    pub fn with_slice_count(mut self, slice_count: SliceDefinitionCount) -> Self {
        self.slice_count = slice_count;
        self
    }

    pub fn with_slice_definitions(mut self, slice_definitions: Vec<SliceDefinition>) -> Self {
        self.slice_definitions = slice_definitions;
        self
    }

    pub fn with_view_definitions(mut self, view_definitions: Vec<ViewDefinition>) -> Self {
        self.view_definitions = view_definitions;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EventModelFileKind {
    Slice,
    Workflow,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SliceDefinitionCount {
    Multiple,
    One,
    Zero,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LegacyScenariosField {
    Absent,
    Present,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScenarioStepField {
    Absent,
    Present,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScenarioSetKind {
    Acceptance,
    Contract,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SliceType {
    Other,
    StateView,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum DefinitionKind {
    Command,
    Event,
    ReadModel,
    Stream,
    View,
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, AsRef, Display)
)]
pub struct DefinitionName(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NamedDefinition {
    kind: DefinitionKind,
    name: DefinitionName,
}

impl NamedDefinition {
    pub fn new(kind: DefinitionKind, name: DefinitionName) -> Self {
        Self { kind, name }
    }

    pub fn into_name(self) -> DefinitionName {
        self.name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceDefinition {
    name: DefinitionName,
    slice_type: SliceType,
    owned_views: Vec<DefinitionName>,
    owned_events: Vec<DefinitionName>,
    outcome_labels: Vec<DefinitionName>,
    legacy_scenarios: LegacyScenariosField,
    scenarios: Vec<SliceScenario>,
}

impl SliceDefinition {
    pub fn new(
        name: DefinitionName,
        slice_type: SliceType,
        owned_views: Vec<DefinitionName>,
        owned_events: Vec<DefinitionName>,
        outcome_labels: Vec<DefinitionName>,
        legacy_scenarios: LegacyScenariosField,
        scenarios: Vec<SliceScenario>,
    ) -> Self {
        Self {
            name,
            slice_type,
            owned_views,
            owned_events,
            outcome_labels,
            legacy_scenarios,
            scenarios,
        }
    }

    pub fn is_state_view(&self) -> bool {
        self.slice_type == SliceType::StateView
    }

    pub fn owned_events(&self) -> &[DefinitionName] {
        &self.owned_events
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceScenario {
    name: DefinitionName,
    when_field: ScenarioStepField,
    scenario_set: ScenarioSetKind,
    referenced_events: Vec<DefinitionName>,
    read_model_states: Vec<DefinitionName>,
}

impl SliceScenario {
    pub fn new(
        name: DefinitionName,
        when_field: ScenarioStepField,
        scenario_set: ScenarioSetKind,
        referenced_events: Vec<DefinitionName>,
        read_model_states: Vec<DefinitionName>,
    ) -> Self {
        Self {
            name,
            when_field,
            scenario_set,
            referenced_events,
            read_model_states,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewDefinition {
    name: DefinitionName,
    read_models: Vec<DefinitionName>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventDefinition {
    name: DefinitionName,
    stream: Option<DefinitionName>,
    attributes: Vec<EventAttribute>,
}

impl EventDefinition {
    pub fn new(
        name: DefinitionName,
        stream: Option<DefinitionName>,
        attributes: Vec<EventAttribute>,
    ) -> Self {
        Self {
            name,
            stream,
            attributes,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventAttribute {
    name: DefinitionName,
    source: EventAttributeSource,
}

impl EventAttribute {
    pub fn new(name: DefinitionName, source: EventAttributeSource) -> Self {
        Self { name, source }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EventAttributeSource {
    CommandInput(DefinitionName),
    ExternalField(DefinitionName, DefinitionName),
    GeneratedEmpty,
    ReadModelField(DefinitionName, DefinitionName),
    Other,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelDefinition {
    name: DefinitionName,
    fields: Vec<ReadModelField>,
}

impl ReadModelDefinition {
    pub fn new(name: DefinitionName, fields: Vec<ReadModelField>) -> Self {
        Self { name, fields }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelField {
    name: DefinitionName,
    source: ReadModelFieldSource,
    derivation: ReadModelFieldDerivation,
}

impl ReadModelField {
    pub fn new(
        name: DefinitionName,
        source: ReadModelFieldSource,
        derivation: ReadModelFieldDerivation,
    ) -> Self {
        Self {
            name,
            source,
            derivation,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadModelFieldSource {
    EventAttribute(DefinitionName, DefinitionName),
    Derivation(DefinitionName),
    Other,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReadModelFieldDerivation {
    NotDerived,
    DerivedWithoutProvenance,
    DerivedWithProvenance,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandDefinition {
    inputs: Vec<DefinitionName>,
    external_inputs: Vec<DefinitionName>,
    external_input_schemas: Vec<ExternalInputSchema>,
    produces: Vec<DefinitionName>,
}

impl CommandDefinition {
    pub fn new(
        inputs: Vec<DefinitionName>,
        external_inputs: Vec<DefinitionName>,
        external_input_schemas: Vec<ExternalInputSchema>,
        produces: Vec<DefinitionName>,
    ) -> Self {
        Self {
            inputs,
            external_inputs,
            external_input_schemas,
            produces,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExternalInputSchema {
    name: DefinitionName,
    fields: Vec<DefinitionName>,
}

impl ExternalInputSchema {
    pub fn new(name: DefinitionName, fields: Vec<DefinitionName>) -> Self {
        Self { name, fields }
    }
}

impl ViewDefinition {
    pub fn new(name: DefinitionName, read_models: Vec<DefinitionName>) -> Self {
        Self { name, read_models }
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, AsRef, Display)
)]
pub struct TopLevelKey(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ValidationIssue(String);

pub fn validate_event_model(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    required_top_level_keys()
        .iter()
        .find(|key| !document.top_level_keys.contains(*key))
        .map_or(Ok(()), |key| {
            Err(validation_issue(format!("missing top-level key '{key}'")))
        })?;

    document
        .top_level_keys
        .contains(&explicit_board_key())
        .then_some(())
        .ok_or_else(|| validation_issue("missing explicit board"))?;

    duplicate_named_definition(document).map_or(Ok(()), |definition| {
        Err(validation_issue(format!(
            "duplicate {} name '{}'",
            definition_kind_label(definition.kind),
            definition.name
        )))
    })?;

    validate_slice_file_count(document)?;

    validate_no_legacy_slice_scenarios(document)?;

    validate_scenario_when_fields(document)?;

    validate_duplicate_scenario_names(document)?;

    validate_acceptance_scenario_boundaries(document)?;

    validate_state_view_projector_contract_scenarios(document)?;

    validate_duplicate_outcome_labels(document)?;

    validate_event_stream_references(document)?;

    validate_event_producers(document)?;

    validate_command_sourced_event_attributes(document)?;

    validate_external_sourced_event_attributes(document)
        .and_then(|()| validate_read_model_sourced_event_attributes(document))
        .and_then(|()| validate_generated_event_attribute_sources(document))
        .and_then(|()| validate_derived_read_model_field_provenance(document))
        .and_then(|()| validate_read_model_field_event_sources(document))
}

pub fn model_must_be_object_issue() -> ValidationIssue {
    validation_issue("model must be a JSON object")
}

pub fn empty_top_level_key_issue() -> ValidationIssue {
    validation_issue("top-level key must not be empty")
}

fn top_level_key(raw: &str) -> TopLevelKey {
    TopLevelKey::try_new(raw.to_owned()).unwrap_or_else(|error| {
        unreachable!("EMC required top-level key must be valid: {error}");
    })
}

fn validation_issue(value: impl Into<String>) -> ValidationIssue {
    ValidationIssue::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC validation issue must be non-empty: {error}");
    })
}

fn required_top_level_keys() -> Vec<TopLevelKey> {
    [
        "name",
        "version",
        "streams",
        "events",
        "commands",
        "read_models",
        "slices",
    ]
    .iter()
    .map(|key| top_level_key(key))
    .collect()
}

fn explicit_board_key() -> TopLevelKey {
    top_level_key("board")
}

fn duplicate_named_definition(document: &EventModelDocument) -> Option<NamedDefinition> {
    let mut seen = BTreeSet::new();
    document.named_definitions.iter().find_map(|definition| {
        let key = (definition.kind, definition.name.clone());
        if seen.insert(key) {
            None
        } else {
            Some(definition.clone())
        }
    })
}

fn definition_kind_label(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Command => "command",
        DefinitionKind::Event => "event",
        DefinitionKind::ReadModel => "read model",
        DefinitionKind::Stream => "stream",
        DefinitionKind::View => "view",
    }
}

fn validate_slice_file_count(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    match (document.file_kind, document.slice_count) {
        (EventModelFileKind::Slice, SliceDefinitionCount::One)
        | (EventModelFileKind::Workflow, _) => Ok(()),
        (
            EventModelFileKind::Slice,
            SliceDefinitionCount::Multiple | SliceDefinitionCount::Zero,
        ) => Err(validation_issue(
            "slice file must contain exactly one slice",
        )),
    }
}

fn validate_no_legacy_slice_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find(|slice| slice.legacy_scenarios == LegacyScenariosField::Present)
        .map_or(Ok(()), |slice| {
            Err(validation_issue(format!(
                "slice '{}' uses legacy 'scenarios'; use 'acceptance_scenarios' and 'contract_scenarios'",
                slice.name
            )))
        })
}

fn validate_scenario_when_fields(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .flat_map(|slice| {
            slice
                .scenarios
                .iter()
                .map(move |scenario| (slice, scenario))
        })
        .find(|(_, scenario)| scenario.when_field == ScenarioStepField::Absent)
        .map_or(Ok(()), |(slice, scenario)| {
            Err(validation_issue(format!(
                "slice '{}' scenario '{}' is missing 'when'",
                slice.name, scenario.name
            )))
        })
}

fn validate_duplicate_scenario_names(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find_map(duplicate_scenario_name)
        .map_or(Ok(()), |duplicate| {
            Err(validation_issue(format!(
                "slice '{}' has duplicate scenario name '{}'{}",
                duplicate.slice_name,
                duplicate.scenario_name,
                duplicate_scenario_suffix(duplicate.duplicate_kind)
            )))
        })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DuplicateScenarioKind {
    AcrossFirstClassFields,
    WithinField,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct DuplicateScenario {
    slice_name: DefinitionName,
    scenario_name: DefinitionName,
    duplicate_kind: DuplicateScenarioKind,
}

fn duplicate_scenario_name(slice: &SliceDefinition) -> Option<DuplicateScenario> {
    let mut seen = BTreeMap::new();
    slice.scenarios.iter().find_map(|scenario| {
        seen.insert(scenario.name.clone(), scenario.scenario_set)
            .map(|existing_scenario_set| DuplicateScenario {
                slice_name: slice.name.clone(),
                scenario_name: scenario.name.clone(),
                duplicate_kind: if existing_scenario_set == scenario.scenario_set {
                    DuplicateScenarioKind::WithinField
                } else {
                    DuplicateScenarioKind::AcrossFirstClassFields
                },
            })
    })
}

fn duplicate_scenario_suffix(duplicate_kind: DuplicateScenarioKind) -> &'static str {
    match duplicate_kind {
        DuplicateScenarioKind::AcrossFirstClassFields => {
            " across acceptance_scenarios and contract_scenarios"
        }
        DuplicateScenarioKind::WithinField => "",
    }
}

fn validate_acceptance_scenario_boundaries(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .flat_map(|slice| {
            slice
                .scenarios
                .iter()
                .filter(|scenario| scenario.scenario_set == ScenarioSetKind::Acceptance)
                .flat_map(move |scenario| {
                    scenario
                        .referenced_events
                        .iter()
                        .filter(|event_name| document.event_names.contains(*event_name))
                        .map(move |event_name| (slice, scenario, event_name))
                })
        })
        .next()
        .map_or(Ok(()), |(slice, scenario, event_name)| {
            Err(validation_issue(format!(
                "slice '{}' acceptance scenario '{}' references event '{}'; acceptance_scenarios must describe user-facing behavior only",
                slice.name, scenario.name, event_name
            )))
        })
}

fn validate_state_view_projector_contract_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateView)
        .flat_map(|slice| {
            read_models_for_slice_views(document, slice)
                .into_iter()
                .filter(|read_model| !slice_has_contract_state(slice, read_model))
                .map(move |read_model| (slice, read_model))
        })
        .next()
        .map_or(Ok(()), |(slice, read_model)| {
            Err(validation_issue(format!(
                "state_view slice '{}' read model '{}' requires a contract_scenarios GWT for the projector",
                slice.name, read_model
            )))
        })
}

fn read_models_for_slice_views(
    document: &EventModelDocument,
    slice: &SliceDefinition,
) -> Vec<DefinitionName> {
    slice
        .owned_views
        .iter()
        .filter_map(|view_name| {
            document
                .view_definitions
                .iter()
                .find(|view| view.name == *view_name)
        })
        .flat_map(|view| view.read_models.clone())
        .collect()
}

fn slice_has_contract_state(slice: &SliceDefinition, read_model: &DefinitionName) -> bool {
    slice.scenarios.iter().any(|scenario| {
        scenario.scenario_set == ScenarioSetKind::Contract
            && scenario.read_model_states.contains(read_model)
    })
}

fn validate_duplicate_outcome_labels(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find_map(duplicate_outcome_label)
        .map_or(Ok(()), |(slice, outcome_label)| {
            Err(validation_issue(format!(
                "slice '{}' has duplicate outcome label '{}'",
                slice.name, outcome_label
            )))
        })
}

fn duplicate_outcome_label(slice: &SliceDefinition) -> Option<(&SliceDefinition, DefinitionName)> {
    let mut seen = BTreeSet::new();
    slice.outcome_labels.iter().find_map(|outcome_label| {
        if seen.insert(outcome_label.clone()) {
            None
        } else {
            Some((slice, outcome_label.clone()))
        }
    })
}

fn validate_event_stream_references(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    document
        .event_definitions
        .iter()
        .filter_map(|event| event.stream.as_ref().map(|stream| (event, stream)))
        .find(|(_, stream)| !document.stream_names.contains(*stream))
        .map_or(Ok(()), |(event, stream)| {
            Err(validation_issue(format!(
                "event '{}' references unknown stream '{}'",
                event.name, stream
            )))
        })
}

fn validate_event_producers(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    document
        .event_definitions
        .iter()
        .find(|event| {
            !document.command_produced_events.contains(&event.name)
                && !document.state_view_observed_events.contains(&event.name)
        })
        .map_or(Ok(()), |event| {
            Err(validation_issue(format!(
                "event '{}' is not produced by any command",
                event.name
            )))
        })
}

fn validate_command_sourced_event_attributes(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .event_definitions
        .iter()
        .flat_map(|event| {
            event.attributes.iter().filter_map(move |attribute| {
                if let EventAttributeSource::CommandInput(input_name) = &attribute.source {
                    Some((event, attribute, input_name))
                } else {
                    None
                }
            })
        })
        .find(|(event, _, input_name)| !event_has_producer_input(document, event, input_name))
        .map_or(Ok(()), |(event, attribute, input_name)| {
            Err(validation_issue(format!(
                "event '{}' attribute '{}' has invalid source 'command.{}'",
                event.name, attribute.name, input_name
            )))
        })
}

fn event_has_producer_input(
    document: &EventModelDocument,
    event: &EventDefinition,
    input_name: &DefinitionName,
) -> bool {
    document.command_definitions.iter().any(|command| {
        command.produces.contains(&event.name) && command.inputs.contains(input_name)
    })
}

fn validate_external_sourced_event_attributes(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .event_definitions
        .iter()
        .flat_map(|event| {
            event.attributes.iter().filter_map(move |attribute| {
                if let EventAttributeSource::ExternalField(payload_name, field_name) =
                    &attribute.source
                {
                    Some((event, attribute, payload_name, field_name))
                } else {
                    None
                }
            })
        })
        .find(|(event, _, payload_name, _)| {
            !event_has_producer_external_input(document, event, payload_name)
        })
        .map_or_else(
            || validate_external_sourced_event_attribute_fields(document),
            |(event, attribute, payload_name, field_name)| {
                Err(validation_issue(format!(
                    "event '{}' attribute '{}' has invalid source 'external.{}.{}'",
                    event.name, attribute.name, payload_name, field_name
                )))
            },
        )
}

fn event_has_producer_external_input(
    document: &EventModelDocument,
    event: &EventDefinition,
    payload_name: &DefinitionName,
) -> bool {
    document.command_definitions.iter().any(|command| {
        command.produces.contains(&event.name) && command.external_inputs.contains(payload_name)
    })
}

fn validate_external_sourced_event_attribute_fields(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .event_definitions
        .iter()
        .flat_map(|event| {
            event.attributes.iter().filter_map(move |attribute| {
                if let EventAttributeSource::ExternalField(payload_name, field_name) =
                    &attribute.source
                {
                    Some((event, attribute, payload_name, field_name))
                } else {
                    None
                }
            })
        })
        .find(|(event, _, payload_name, field_name)| {
            !event_has_producer_external_field(document, event, payload_name, field_name)
        })
        .map_or(Ok(()), |(event, attribute, _, field_name)| {
            Err(validation_issue(format!(
                "event '{}' attribute '{}' references undeclared external input field '{}'",
                event.name, attribute.name, field_name
            )))
        })
}

fn event_has_producer_external_field(
    document: &EventModelDocument,
    event: &EventDefinition,
    payload_name: &DefinitionName,
    field_name: &DefinitionName,
) -> bool {
    document.command_definitions.iter().any(|command| {
        command.produces.contains(&event.name)
            && external_field_is_declared(command, payload_name, field_name)
    })
}

fn external_field_is_declared(
    command: &CommandDefinition,
    payload_name: &DefinitionName,
    field_name: &DefinitionName,
) -> bool {
    command
        .external_input_schemas
        .iter()
        .find(|schema| schema.name == *payload_name)
        .is_some_and(|schema| schema.fields.contains(field_name))
}

fn validate_read_model_sourced_event_attributes(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .event_definitions
        .iter()
        .flat_map(|event| {
            event.attributes.iter().filter_map(move |attribute| {
                if let EventAttributeSource::ReadModelField(read_model_name, field_name) =
                    &attribute.source
                {
                    Some((event, attribute, read_model_name, field_name))
                } else {
                    None
                }
            })
        })
        .next()
        .map_or(Ok(()), |(event, attribute, read_model_name, field_name)| {
            Err(validation_issue(format!(
                "event '{}' attribute '{}' has invalid source 'read_model.{}.{}'",
                event.name, attribute.name, read_model_name, field_name
            )))
        })
}

fn validate_generated_event_attribute_sources(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .event_definitions
        .iter()
        .flat_map(|event| {
            event.attributes.iter().filter_map(move |attribute| {
                (attribute.source == EventAttributeSource::GeneratedEmpty)
                    .then_some((event, attribute))
            })
        })
        .next()
        .map_or(Ok(()), |(event, attribute)| {
            Err(validation_issue(format!(
                "event '{}' attribute '{}' has invalid source 'generated.'",
                event.name, attribute.name
            )))
        })
}

fn validate_read_model_field_event_sources(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .read_model_definitions
        .iter()
        .flat_map(|read_model| {
            read_model.fields.iter().filter_map(move |field| {
                if let ReadModelFieldSource::EventAttribute(event_name, attribute_name) =
                    &field.source
                {
                    Some((read_model, field, event_name, attribute_name))
                } else {
                    None
                }
            })
        })
        .find(|(_, _, event_name, attribute_name)| {
            !event_attribute_exists(document, event_name, attribute_name)
        })
        .map_or(Ok(()), |(read_model, field, event_name, attribute_name)| {
            Err(validation_issue(format!(
                "read model '{}' field '{}' references unknown event attribute '{}.{}'",
                read_model.name, field.name, event_name, attribute_name
            )))
        })
}

fn validate_derived_read_model_field_provenance(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .read_model_definitions
        .iter()
        .flat_map(|read_model| {
            read_model.fields.iter().filter(|field| {
                field.derivation == ReadModelFieldDerivation::DerivedWithoutProvenance
            })
        })
        .next()
        .map_or(Ok(()), |field| {
            Err(validation_issue(format!(
                "derived read model field '{}' must declare source fields and derivation",
                field.name
            )))
        })
}

fn event_attribute_exists(
    document: &EventModelDocument,
    event_name: &DefinitionName,
    attribute_name: &DefinitionName,
) -> bool {
    document
        .event_definitions
        .iter()
        .find(|event| event.name == *event_name)
        .is_some_and(|event| {
            event
                .attributes
                .iter()
                .any(|attribute| attribute.name == *attribute_name)
        })
}
