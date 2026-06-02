use std::collections::{BTreeMap, BTreeSet};

use nutype::nutype;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventModelDocument {
    file_kind: EventModelFileKind,
    top_level_keys: BTreeSet<TopLevelKey>,
    named_definitions: Vec<NamedDefinition>,
    slice_count: SliceDefinitionCount,
    slice_definitions: Vec<SliceDefinition>,
}

impl EventModelDocument {
    pub fn new(
        file_kind: EventModelFileKind,
        top_level_keys: BTreeSet<TopLevelKey>,
        named_definitions: Vec<NamedDefinition>,
        slice_count: SliceDefinitionCount,
        slice_definitions: Vec<SliceDefinition>,
    ) -> Self {
        Self {
            file_kind,
            top_level_keys,
            named_definitions,
            slice_count,
            slice_definitions,
        }
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceDefinition {
    name: DefinitionName,
    legacy_scenarios: LegacyScenariosField,
    scenarios: Vec<SliceScenario>,
}

impl SliceDefinition {
    pub fn new(
        name: DefinitionName,
        legacy_scenarios: LegacyScenariosField,
        scenarios: Vec<SliceScenario>,
    ) -> Self {
        Self {
            name,
            legacy_scenarios,
            scenarios,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceScenario {
    name: DefinitionName,
    when_field: ScenarioStepField,
    scenario_set: ScenarioSetKind,
}

impl SliceScenario {
    pub fn new(
        name: DefinitionName,
        when_field: ScenarioStepField,
        scenario_set: ScenarioSetKind,
    ) -> Self {
        Self {
            name,
            when_field,
            scenario_set,
        }
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

    validate_duplicate_scenario_names(document)
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
