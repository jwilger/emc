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
    board_read_model_command_dependencies: Vec<BoardReadModelCommandDependency>,
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
            board_read_model_command_dependencies: parts.board_read_model_command_dependencies,
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
    board_read_model_command_dependencies: Vec<BoardReadModelCommandDependency>,
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
            board_read_model_command_dependencies: Vec::new(),
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

    pub fn with_board_read_model_command_dependencies(
        mut self,
        board_read_model_command_dependencies: Vec<BoardReadModelCommandDependency>,
    ) -> Self {
        self.board_read_model_command_dependencies = board_read_model_command_dependencies;
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
    Automation,
    Other,
    StateChange,
    StateView,
    Translation,
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
pub struct BoardReadModelCommandDependency {
    read_model: DefinitionName,
    command: DefinitionName,
    intermediate_automation: DefinitionName,
}

impl BoardReadModelCommandDependency {
    pub fn new(
        read_model: DefinitionName,
        command: DefinitionName,
        intermediate_automation: DefinitionName,
    ) -> Self {
        Self {
            read_model,
            command,
            intermediate_automation,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceDefinition {
    name: DefinitionName,
    slice_type: SliceType,
    issued_commands: Vec<DefinitionName>,
    handled_command_errors: Vec<DefinitionName>,
    owned_automations: Vec<DefinitionName>,
    owned_read_models: Vec<DefinitionName>,
    owned_translations: Vec<DefinitionName>,
    owned_views: Vec<DefinitionName>,
    owned_events: Vec<DefinitionName>,
    outcome_labels: Vec<DefinitionName>,
    outcomes: Vec<OutcomeDefinition>,
    legacy_scenarios: LegacyScenariosField,
    singleton_behavior: SingletonBehavior,
    automation_trigger: AutomationTrigger,
    automation_command_policy: AutomationCommandPolicy,
    translation_contract: TranslationContract,
    scenarios: Vec<SliceScenario>,
}

impl SliceDefinition {
    pub fn new(parts: SliceDefinitionParts) -> Self {
        Self {
            name: parts.name,
            slice_type: parts.slice_type,
            issued_commands: parts.issued_commands,
            handled_command_errors: parts.handled_command_errors,
            owned_automations: parts.owned_automations,
            owned_read_models: parts.owned_read_models,
            owned_translations: parts.owned_translations,
            owned_views: parts.owned_views,
            owned_events: parts.owned_events,
            outcome_labels: parts.outcome_labels,
            outcomes: parts.outcomes,
            legacy_scenarios: parts.legacy_scenarios,
            singleton_behavior: parts.singleton_behavior,
            automation_trigger: parts.automation_trigger,
            automation_command_policy: parts.automation_command_policy,
            translation_contract: parts.translation_contract,
            scenarios: parts.scenarios,
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
pub struct SliceDefinitionParts {
    name: DefinitionName,
    slice_type: SliceType,
    issued_commands: Vec<DefinitionName>,
    handled_command_errors: Vec<DefinitionName>,
    owned_automations: Vec<DefinitionName>,
    owned_read_models: Vec<DefinitionName>,
    owned_translations: Vec<DefinitionName>,
    owned_views: Vec<DefinitionName>,
    owned_events: Vec<DefinitionName>,
    outcome_labels: Vec<DefinitionName>,
    outcomes: Vec<OutcomeDefinition>,
    legacy_scenarios: LegacyScenariosField,
    singleton_behavior: SingletonBehavior,
    automation_trigger: AutomationTrigger,
    automation_command_policy: AutomationCommandPolicy,
    translation_contract: TranslationContract,
    scenarios: Vec<SliceScenario>,
}

impl SliceDefinitionParts {
    pub fn new(name: DefinitionName, slice_type: SliceType) -> Self {
        Self {
            name,
            slice_type,
            issued_commands: Vec::new(),
            handled_command_errors: Vec::new(),
            owned_automations: Vec::new(),
            owned_read_models: Vec::new(),
            owned_translations: Vec::new(),
            owned_views: Vec::new(),
            owned_events: Vec::new(),
            outcome_labels: Vec::new(),
            outcomes: Vec::new(),
            legacy_scenarios: LegacyScenariosField::Absent,
            singleton_behavior: SingletonBehavior::NotSingleton,
            automation_trigger: AutomationTrigger::NotAutomation,
            automation_command_policy: AutomationCommandPolicy::NotAutomation,
            translation_contract: TranslationContract::NotTranslation,
            scenarios: Vec::new(),
        }
    }

    pub fn with_issued_commands(mut self, issued_commands: Vec<DefinitionName>) -> Self {
        self.issued_commands = issued_commands;
        self
    }

    pub fn with_handled_command_errors(
        mut self,
        handled_command_errors: Vec<DefinitionName>,
    ) -> Self {
        self.handled_command_errors = handled_command_errors;
        self
    }

    pub fn with_owned_automations(mut self, owned_automations: Vec<DefinitionName>) -> Self {
        self.owned_automations = owned_automations;
        self
    }

    pub fn with_owned_read_models(mut self, owned_read_models: Vec<DefinitionName>) -> Self {
        self.owned_read_models = owned_read_models;
        self
    }

    pub fn with_owned_translations(mut self, owned_translations: Vec<DefinitionName>) -> Self {
        self.owned_translations = owned_translations;
        self
    }

    pub fn with_owned_views(mut self, owned_views: Vec<DefinitionName>) -> Self {
        self.owned_views = owned_views;
        self
    }

    pub fn with_owned_events(mut self, owned_events: Vec<DefinitionName>) -> Self {
        self.owned_events = owned_events;
        self
    }

    pub fn with_outcome_labels(mut self, outcome_labels: Vec<DefinitionName>) -> Self {
        self.outcome_labels = outcome_labels;
        self
    }

    pub fn with_outcomes(mut self, outcomes: Vec<OutcomeDefinition>) -> Self {
        self.outcomes = outcomes;
        self
    }

    pub fn with_legacy_scenarios(mut self, legacy_scenarios: LegacyScenariosField) -> Self {
        self.legacy_scenarios = legacy_scenarios;
        self
    }

    pub fn with_singleton_behavior(mut self, singleton_behavior: SingletonBehavior) -> Self {
        self.singleton_behavior = singleton_behavior;
        self
    }

    pub fn with_automation_trigger(mut self, automation_trigger: AutomationTrigger) -> Self {
        self.automation_trigger = automation_trigger;
        self
    }

    pub fn with_automation_command_policy(
        mut self,
        automation_command_policy: AutomationCommandPolicy,
    ) -> Self {
        self.automation_command_policy = automation_command_policy;
        self
    }

    pub fn with_translation_contract(mut self, translation_contract: TranslationContract) -> Self {
        self.translation_contract = translation_contract;
        self
    }

    pub fn with_scenarios(mut self, scenarios: Vec<SliceScenario>) -> Self {
        self.scenarios = scenarios;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SingletonBehavior {
    NotSingleton,
    MissingRepeatBehavior,
    DeclaresRepeatBehavior,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AutomationTrigger {
    NotAutomation,
    MissingTrigger,
    DeclaresTrigger,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AutomationCommandPolicy {
    NotAutomation,
    SingleCommand,
    MultipleCommands,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TranslationContract {
    NotTranslation,
    MissingExternalContract,
    DeclaresExternalContract,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OutcomeDefinition {
    label: DefinitionName,
    events: Vec<DefinitionName>,
}

impl OutcomeDefinition {
    pub fn new(label: DefinitionName, events: Vec<DefinitionName>) -> Self {
        Self { label, events }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceScenario {
    name: DefinitionName,
    when_field: ScenarioStepField,
    scenario_set: ScenarioSetKind,
    referenced_events: Vec<DefinitionName>,
    then_events: Vec<DefinitionName>,
    command_errors: Vec<DefinitionName>,
    given_streams: Vec<DefinitionName>,
    read_model_states: Vec<DefinitionName>,
}

impl SliceScenario {
    pub fn new(parts: SliceScenarioParts) -> Self {
        Self {
            name: parts.name,
            when_field: parts.when_field,
            scenario_set: parts.scenario_set,
            referenced_events: parts.referenced_events,
            then_events: parts.then_events,
            command_errors: parts.command_errors,
            given_streams: parts.given_streams,
            read_model_states: parts.read_model_states,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceScenarioParts {
    name: DefinitionName,
    when_field: ScenarioStepField,
    scenario_set: ScenarioSetKind,
    referenced_events: Vec<DefinitionName>,
    then_events: Vec<DefinitionName>,
    command_errors: Vec<DefinitionName>,
    given_streams: Vec<DefinitionName>,
    read_model_states: Vec<DefinitionName>,
}

impl SliceScenarioParts {
    pub fn new(
        name: DefinitionName,
        when_field: ScenarioStepField,
        scenario_set: ScenarioSetKind,
    ) -> Self {
        Self {
            name,
            when_field,
            scenario_set,
            referenced_events: Vec::new(),
            then_events: Vec::new(),
            command_errors: Vec::new(),
            given_streams: Vec::new(),
            read_model_states: Vec::new(),
        }
    }

    pub fn with_referenced_events(mut self, referenced_events: Vec<DefinitionName>) -> Self {
        self.referenced_events = referenced_events;
        self
    }

    pub fn with_then_events(mut self, then_events: Vec<DefinitionName>) -> Self {
        self.then_events = then_events;
        self
    }

    pub fn with_command_errors(mut self, command_errors: Vec<DefinitionName>) -> Self {
        self.command_errors = command_errors;
        self
    }

    pub fn with_given_streams(mut self, given_streams: Vec<DefinitionName>) -> Self {
        self.given_streams = given_streams;
        self
    }

    pub fn with_read_model_states(mut self, read_model_states: Vec<DefinitionName>) -> Self {
        self.read_model_states = read_model_states;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewDefinition {
    name: DefinitionName,
    read_models: Vec<DefinitionName>,
    controls: Vec<ViewControlDefinition>,
    local_states: Vec<DefinitionName>,
    wireframe: ViewWireframe,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewControlDefinition {
    label: DefinitionName,
    command: Option<DefinitionName>,
    command_error_handling: Vec<ControlCommandErrorHandling>,
    navigation_target: Option<DefinitionName>,
    navigation_type: NavigationType,
    workflow_target: Option<DefinitionName>,
}

impl ViewControlDefinition {
    pub fn new(parts: ViewControlDefinitionParts) -> Self {
        Self {
            label: parts.label,
            command: parts.command,
            command_error_handling: parts.command_error_handling,
            navigation_target: parts.navigation_target,
            navigation_type: parts.navigation_type,
            workflow_target: parts.workflow_target,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewControlDefinitionParts {
    label: DefinitionName,
    command: Option<DefinitionName>,
    command_error_handling: Vec<ControlCommandErrorHandling>,
    navigation_target: Option<DefinitionName>,
    navigation_type: NavigationType,
    workflow_target: Option<DefinitionName>,
}

impl ViewControlDefinitionParts {
    pub fn new(label: DefinitionName) -> Self {
        Self {
            label,
            command: None,
            command_error_handling: Vec::new(),
            navigation_target: None,
            navigation_type: NavigationType::Absent,
            workflow_target: None,
        }
    }

    pub fn with_command(mut self, command: Option<DefinitionName>) -> Self {
        self.command = command;
        self
    }

    pub fn with_command_error_handling(
        mut self,
        command_error_handling: Vec<ControlCommandErrorHandling>,
    ) -> Self {
        self.command_error_handling = command_error_handling;
        self
    }

    pub fn with_navigation_target(mut self, navigation_target: Option<DefinitionName>) -> Self {
        self.navigation_target = navigation_target;
        self
    }

    pub fn with_navigation_type(mut self, navigation_type: NavigationType) -> Self {
        self.navigation_type = navigation_type;
        self
    }

    pub fn with_workflow_target(mut self, workflow_target: Option<DefinitionName>) -> Self {
        self.workflow_target = workflow_target;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NavigationType {
    Absent,
    ModeledView,
    LocalViewState,
    ExternalSystem,
    ExternalWorkflow,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ControlCommandErrorHandling {
    error_name: DefinitionName,
    recovery_behavior: ControlErrorRecoveryBehavior,
}

impl ControlCommandErrorHandling {
    pub fn new(
        error_name: DefinitionName,
        recovery_behavior: ControlErrorRecoveryBehavior,
    ) -> Self {
        Self {
            error_name,
            recovery_behavior,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ControlErrorRecoveryBehavior {
    Missing,
    Present,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ViewWireframe {
    Absent,
    Present,
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
    transitive_derivation: ReadModelTransitiveDerivation,
}

impl ReadModelDefinition {
    pub fn new(
        name: DefinitionName,
        fields: Vec<ReadModelField>,
        transitive_derivation: ReadModelTransitiveDerivation,
    ) -> Self {
        Self {
            name,
            fields,
            transitive_derivation,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReadModelTransitiveDerivation {
    NotTransitive,
    TransitiveWithoutRule,
    TransitiveComplete,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelField {
    name: DefinitionName,
    source: ReadModelFieldSource,
    derivation: ReadModelFieldDerivation,
    absence_default: ReadModelFieldAbsenceDefault,
}

impl ReadModelField {
    pub fn new(
        name: DefinitionName,
        source: ReadModelFieldSource,
        derivation: ReadModelFieldDerivation,
        absence_default: ReadModelFieldAbsenceDefault,
    ) -> Self {
        Self {
            name,
            source,
            derivation,
            absence_default,
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
    DerivedWithoutScenarios,
    DerivedComplete,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReadModelFieldAbsenceDefault {
    NotDefaulted,
    DefaultedWithoutAbsenceEvent,
    DefaultedWithoutScenarios,
    DefaultedComplete,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandDefinition {
    name: Option<DefinitionName>,
    inputs: Vec<DefinitionName>,
    input_sources: Vec<CommandInputSource>,
    read_model_reads: CommandReadModelReads,
    external_inputs: Vec<DefinitionName>,
    external_input_schemas: Vec<ExternalInputSchema>,
    produces: Vec<DefinitionName>,
    errors: Vec<DefinitionName>,
}

impl CommandDefinition {
    pub fn new(parts: CommandDefinitionParts) -> Self {
        Self {
            name: parts.name,
            inputs: parts.inputs,
            input_sources: parts.input_sources,
            read_model_reads: parts.read_model_reads,
            external_inputs: parts.external_inputs,
            external_input_schemas: parts.external_input_schemas,
            produces: parts.produces,
            errors: parts.errors,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandDefinitionParts {
    name: Option<DefinitionName>,
    inputs: Vec<DefinitionName>,
    input_sources: Vec<CommandInputSource>,
    read_model_reads: CommandReadModelReads,
    external_inputs: Vec<DefinitionName>,
    external_input_schemas: Vec<ExternalInputSchema>,
    produces: Vec<DefinitionName>,
    errors: Vec<DefinitionName>,
}

impl CommandDefinitionParts {
    pub fn new(name: Option<DefinitionName>) -> Self {
        Self {
            name,
            inputs: Vec::new(),
            input_sources: Vec::new(),
            read_model_reads: CommandReadModelReads::Absent,
            external_inputs: Vec::new(),
            external_input_schemas: Vec::new(),
            produces: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn with_inputs(mut self, inputs: Vec<DefinitionName>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_input_sources(mut self, input_sources: Vec<CommandInputSource>) -> Self {
        self.input_sources = input_sources;
        self
    }

    pub fn with_read_model_reads(mut self, read_model_reads: CommandReadModelReads) -> Self {
        self.read_model_reads = read_model_reads;
        self
    }

    pub fn with_external_inputs(mut self, external_inputs: Vec<DefinitionName>) -> Self {
        self.external_inputs = external_inputs;
        self
    }

    pub fn with_external_input_schemas(
        mut self,
        external_input_schemas: Vec<ExternalInputSchema>,
    ) -> Self {
        self.external_input_schemas = external_input_schemas;
        self
    }

    pub fn with_produces(mut self, produces: Vec<DefinitionName>) -> Self {
        self.produces = produces;
        self
    }

    pub fn with_errors(mut self, errors: Vec<DefinitionName>) -> Self {
        self.errors = errors;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandInputSource {
    name: DefinitionName,
    source: CommandInputSourceKind,
}

impl CommandInputSource {
    pub fn new(name: DefinitionName, source: CommandInputSourceKind) -> Self {
        Self { name, source }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CommandInputSourceKind {
    ExternalField(DefinitionName, DefinitionName),
    Other,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommandReadModelReads {
    Absent,
    Present,
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
    pub fn new(
        name: DefinitionName,
        read_models: Vec<DefinitionName>,
        controls: Vec<ViewControlDefinition>,
        local_states: Vec<DefinitionName>,
        wireframe: ViewWireframe,
    ) -> Self {
        Self {
            name,
            read_models,
            controls,
            local_states,
            wireframe,
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

    validate_duplicate_scenario_names(document)?;

    validate_acceptance_scenario_boundaries(document)?;

    validate_state_view_projector_contract_scenarios(document)?;

    validate_duplicate_outcome_labels(document)?;

    validate_outcome_event_sets_not_empty(document)?;

    validate_outcome_events_reference_known_events(document)?;

    validate_outcome_events_belong_to_slice(document)?;

    validate_duplicate_outcome_event_sets(document)?;

    validate_event_stream_references(document)?;

    validate_event_producers(document)?;

    validate_state_change_scenario_given_streams(document)?;

    validate_singleton_state_change_repeat_behavior(document)?;

    validate_translation_slice_external_contracts(document)?;

    validate_translation_slice_view_ownership(document)?;

    validate_automation_slice_triggers(document)?;

    validate_automation_slice_command_policy(document)?;

    validate_automation_slice_command_error_handling(document)?;

    validate_scenario_command_errors_are_declared(document)?;

    validate_state_change_command_error_scenarios(document)?;

    validate_control_command_error_handling(document)?;

    validate_control_error_handling_recovery(document)?;

    validate_navigation_controls_declare_type(document)?;

    validate_modeled_view_navigation_targets(document)?;

    validate_local_view_state_navigation_targets(document)?;

    validate_external_workflow_navigation_targets(document)?;

    validate_board_read_model_to_command_intermediates(document)?;

    validate_command_sourced_event_attributes(document)?;

    validate_command_legacy_read_model_reads(document)?;

    validate_external_sourced_event_attributes(document)
        .and_then(|()| validate_read_model_sourced_event_attributes(document))
        .and_then(|()| validate_generated_event_attribute_sources(document))
        .and_then(|()| validate_command_input_external_source_fields(document))
        .and_then(|()| validate_derived_read_model_field_provenance(document))
        .and_then(|()| validate_derived_read_model_field_scenarios(document))
        .and_then(|()| validate_absence_default_read_model_field_events(document))
        .and_then(|()| validate_absence_default_read_model_field_scenarios(document))
        .and_then(|()| validate_transitive_read_model_derivation(document))
        .and_then(|()| validate_read_model_field_event_sources(document))
}

pub fn validate_event_model_corpus(
    documents: &[EventModelDocument],
) -> Result<(), ValidationIssue> {
    duplicate_slice_command_definition(documents).map_or(Ok(()), |command_name| {
        Err(validation_issue(format!(
            "command '{command_name}' is defined by more than one slice"
        )))
    })?;

    duplicate_slice_read_model_definition(documents).map_or(Ok(()), |read_model_name| {
        Err(validation_issue(format!(
            "read model '{read_model_name}' is defined by more than one slice"
        )))
    })?;

    duplicate_slice_control_definition(documents).map_or(Ok(()), |duplicate| {
        Err(validation_issue(format!(
            "control '{}' on view '{}' is defined by more than one slice",
            duplicate.control_name, duplicate.view_name
        )))
    })?;

    duplicate_slice_automation_definition(documents).map_or(Ok(()), |automation_name| {
        Err(validation_issue(format!(
            "automation '{automation_name}' is defined by more than one slice"
        )))
    })?;

    duplicate_slice_translation_definition(documents).map_or(Ok(()), |translation_name| {
        Err(validation_issue(format!(
            "translation '{translation_name}' is defined by more than one slice"
        )))
    })?;

    duplicate_cross_slice_scenario_definition(documents).map_or(Ok(()), |scenario_name| {
        Err(validation_issue(format!(
            "scenario '{scenario_name}' is ambiguously defined across slices"
        )))
    })?;

    duplicate_slice_wireframe_definition(documents).map_or(Ok(()), |view_name| {
        Err(validation_issue(format!(
            "wireframe for view '{view_name}' is defined by more than one slice"
        )))
    })?;

    conflicting_event_definition(documents).map_or(Ok(()), |event_name| {
        Err(validation_issue(format!(
            "event '{event_name}' has conflicting definitions across slices"
        )))
    })?;

    duplicate_slice_view_definition(documents).map_or(Ok(()), |view_name| {
        Err(validation_issue(format!(
            "view '{view_name}' is defined by more than one slice"
        )))
    })
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

fn duplicate_slice_command_definition(documents: &[EventModelDocument]) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| document.slice_definitions.iter())
        .flat_map(|slice| {
            slice
                .issued_commands
                .iter()
                .map(move |command_name| (slice, command_name))
        })
        .find_map(|(slice, command_name)| {
            seen.insert(command_name.clone(), slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| command_name.clone())
        })
}

fn duplicate_slice_read_model_definition(
    documents: &[EventModelDocument],
) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| document.slice_definitions.iter())
        .flat_map(|slice| {
            slice
                .owned_read_models
                .iter()
                .map(move |read_model_name| (slice, read_model_name))
        })
        .find_map(|(slice, read_model_name)| {
            seen.insert(read_model_name.clone(), slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| read_model_name.clone())
        })
}

fn conflicting_event_definition(documents: &[EventModelDocument]) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| document.event_definitions.iter())
        .find_map(|event| {
            seen.insert(event.name.clone(), event.clone())
                .filter(|previous_event| previous_event != event)
                .map(|_| event.name.clone())
        })
}

fn duplicate_slice_wireframe_definition(
    documents: &[EventModelDocument],
) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| {
            document.slice_definitions.iter().flat_map(move |slice| {
                slice.owned_views.iter().filter_map(move |view_name| {
                    view_definition(document, view_name)
                        .filter(|view| view.wireframe == ViewWireframe::Present)
                        .map(|view| (slice, view))
                })
            })
        })
        .find_map(|(slice, view)| {
            seen.insert(view.name.clone(), slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| view.name.clone())
        })
}

fn duplicate_cross_slice_scenario_definition(
    documents: &[EventModelDocument],
) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| document.slice_definitions.iter())
        .flat_map(|slice| {
            slice
                .scenarios
                .iter()
                .filter(|scenario| !is_generated_state_scenario(&scenario.name))
                .map(move |scenario| (slice, scenario))
        })
        .find_map(|(slice, scenario)| {
            seen.insert(scenario.name.clone(), slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| scenario.name.clone())
        })
}

fn is_generated_state_scenario(scenario_name: &DefinitionName) -> bool {
    scenario_name.as_ref().ends_with(" empty state")
        || scenario_name.as_ref().ends_with(" partial state")
}

fn duplicate_slice_translation_definition(
    documents: &[EventModelDocument],
) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| document.slice_definitions.iter())
        .flat_map(|slice| {
            slice
                .owned_translations
                .iter()
                .map(move |translation_name| (slice, translation_name))
        })
        .find_map(|(slice, translation_name)| {
            seen.insert(translation_name.clone(), slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| translation_name.clone())
        })
}

fn duplicate_slice_automation_definition(
    documents: &[EventModelDocument],
) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| document.slice_definitions.iter())
        .flat_map(|slice| {
            slice
                .owned_automations
                .iter()
                .map(move |automation_name| (slice, automation_name))
        })
        .find_map(|(slice, automation_name)| {
            seen.insert(automation_name.clone(), slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| automation_name.clone())
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct DuplicateControlDefinition {
    view_name: DefinitionName,
    control_name: DefinitionName,
}

fn duplicate_slice_control_definition(
    documents: &[EventModelDocument],
) -> Option<DuplicateControlDefinition> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| {
            document.slice_definitions.iter().flat_map(move |slice| {
                slice.owned_views.iter().flat_map(move |view_name| {
                    view_definition(document, view_name)
                        .into_iter()
                        .flat_map(move |view| {
                            view.controls
                                .iter()
                                .map(move |control| (slice, view, control))
                        })
                })
            })
        })
        .find_map(|(slice, view, control)| {
            let key = (view.name.clone(), control.label.clone());
            seen.insert(key, slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| DuplicateControlDefinition {
                    view_name: view.name.clone(),
                    control_name: control.label.clone(),
                })
        })
}

fn view_definition<'a>(
    document: &'a EventModelDocument,
    view_name: &DefinitionName,
) -> Option<&'a ViewDefinition> {
    document
        .view_definitions
        .iter()
        .find(|view| view.name == *view_name)
}

fn duplicate_slice_view_definition(documents: &[EventModelDocument]) -> Option<DefinitionName> {
    let mut seen = BTreeMap::new();
    documents
        .iter()
        .filter(|document| document.file_kind == EventModelFileKind::Slice)
        .flat_map(|document| document.slice_definitions.iter())
        .flat_map(|slice| {
            slice
                .owned_views
                .iter()
                .map(move |view_name| (slice, view_name))
        })
        .find_map(|(slice, view_name)| {
            seen.insert(view_name.clone(), slice.name.clone())
                .filter(|previous_slice_name| *previous_slice_name != slice.name)
                .map(|_| view_name.clone())
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

fn validate_outcome_event_sets_not_empty(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .flat_map(|slice| slice.outcomes.iter())
        .find(|outcome| outcome.events.is_empty())
        .map_or(Ok(()), |outcome| {
            Err(validation_issue(format!(
                "outcome '{}' must declare at least one event",
                outcome.label
            )))
        })
}

fn validate_outcome_events_reference_known_events(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find_map(|slice| outcome_unknown_event(document, slice))
        .map_or(Ok(()), |reference| {
            Err(validation_issue(format!(
                "outcome '{}' references unknown event '{}'",
                reference.outcome_label, reference.event_name
            )))
        })
}

fn outcome_unknown_event(
    document: &EventModelDocument,
    slice: &SliceDefinition,
) -> Option<OutcomeEventReference> {
    slice.outcomes.iter().find_map(|outcome| {
        outcome
            .events
            .iter()
            .find(|event_name| !document.event_names.contains(event_name))
            .map(|event_name| OutcomeEventReference {
                slice_name: slice.name.clone(),
                outcome_label: outcome.label.clone(),
                event_name: event_name.clone(),
            })
    })
}

fn validate_outcome_events_belong_to_slice(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find_map(outcome_event_outside_slice)
        .map_or(Ok(()), |reference| {
            Err(validation_issue(format!(
                "outcome '{}' references event '{}' that is not emitted or observed by slice '{}'",
                reference.outcome_label, reference.event_name, reference.slice_name
            )))
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct OutcomeEventReference {
    slice_name: DefinitionName,
    outcome_label: DefinitionName,
    event_name: DefinitionName,
}

fn outcome_event_outside_slice(slice: &SliceDefinition) -> Option<OutcomeEventReference> {
    slice.outcomes.iter().find_map(|outcome| {
        outcome
            .events
            .iter()
            .find(|event_name| !slice.owned_events.contains(event_name))
            .map(|event_name| OutcomeEventReference {
                slice_name: slice.name.clone(),
                outcome_label: outcome.label.clone(),
                event_name: event_name.clone(),
            })
    })
}

fn validate_duplicate_outcome_event_sets(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find_map(duplicate_outcome_event_set)
        .map_or(Ok(()), |duplicate| {
            Err(validation_issue(format!(
                "outcomes '{}' and '{}' use the same event set",
                duplicate.first_label, duplicate.second_label
            )))
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct DuplicateOutcomeEventSet {
    first_label: DefinitionName,
    second_label: DefinitionName,
}

fn duplicate_outcome_event_set(slice: &SliceDefinition) -> Option<DuplicateOutcomeEventSet> {
    slice
        .outcomes
        .iter()
        .enumerate()
        .find_map(|(index, outcome)| {
            slice
                .outcomes
                .iter()
                .skip(index + 1)
                .find(|candidate| outcome.events == candidate.events)
                .map(|candidate| DuplicateOutcomeEventSet {
                    first_label: outcome.label.clone(),
                    second_label: candidate.label.clone(),
                })
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

fn validate_state_change_scenario_given_streams(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateChange)
        .flat_map(|slice| {
            slice.scenarios.iter().filter_map(move |scenario| {
                scenario
                    .then_events
                    .iter()
                    .filter_map(|event_name| event_stream(document, event_name))
                    .find(|stream_name| !scenario.given_streams.contains(stream_name))
                    .map(|stream_name| (scenario, stream_name))
            })
        })
        .next()
        .map_or(Ok(()), |(scenario, stream_name)| {
            Err(validation_issue(format!(
                "state-change scenario '{}' writes stream '{}' but does not name it in given_streams",
                scenario.name, stream_name
            )))
        })
}

fn event_stream(
    document: &EventModelDocument,
    event_name: &DefinitionName,
) -> Option<DefinitionName> {
    document
        .event_definitions
        .iter()
        .find(|event| event.name == *event_name)
        .and_then(|event| event.stream.clone())
}

fn validate_singleton_state_change_repeat_behavior(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateChange)
        .find(|slice| slice.singleton_behavior == SingletonBehavior::MissingRepeatBehavior)
        .map_or(Ok(()), |slice| {
            Err(validation_issue(format!(
                "singleton state_change slice '{}' must declare already-exists or idempotent behavior",
                slice.name
            )))
        })
}

fn validate_translation_slice_external_contracts(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::Translation)
        .find(|slice| slice.translation_contract == TranslationContract::MissingExternalContract)
        .map_or(Ok(()), |slice| {
            Err(validation_issue(format!(
                "translation slice '{}' must declare an external event or payload contract",
                slice.name
            )))
        })
}

fn validate_translation_slice_view_ownership(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::Translation)
        .flat_map(|slice| {
            slice
                .owned_views
                .iter()
                .map(move |view_name| (slice, view_name))
        })
        .next()
        .map_or(Ok(()), |(slice, view_name)| {
            Err(validation_issue(format!(
                "translation slice '{}' must not own view '{}'",
                slice.name, view_name
            )))
        })
}

fn validate_automation_slice_triggers(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::Automation)
        .find(|slice| slice.automation_trigger == AutomationTrigger::MissingTrigger)
        .map_or(Ok(()), |slice| {
            Err(validation_issue(format!(
                "automation slice '{}' must declare a trigger",
                slice.name
            )))
        })
}

fn validate_automation_slice_command_policy(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::Automation)
        .find(|slice| slice.automation_command_policy == AutomationCommandPolicy::MultipleCommands)
        .map_or(Ok(()), |slice| {
            Err(validation_issue(format!(
                "automation slice '{}' must issue one command per operation",
                slice.name
            )))
        })
}

fn validate_automation_slice_command_error_handling(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::Automation)
        .find_map(|slice| {
            unhandled_automation_command_error(document, slice)
                .map(|error_name| (slice, error_name))
        })
        .map_or(Ok(()), |(slice, error_name)| {
            Err(validation_issue(format!(
                "automation slice '{}' does not handle command error '{}'",
                slice.name, error_name
            )))
        })
}

fn unhandled_automation_command_error(
    document: &EventModelDocument,
    slice: &SliceDefinition,
) -> Option<DefinitionName> {
    slice
        .issued_commands
        .iter()
        .filter_map(|command_name| command_definition(document, command_name))
        .flat_map(|command| command.errors.iter())
        .find(|error_name| !slice.handled_command_errors.contains(error_name))
        .cloned()
}

fn command_definition<'a>(
    document: &'a EventModelDocument,
    command_name: &DefinitionName,
) -> Option<&'a CommandDefinition> {
    document
        .command_definitions
        .iter()
        .find(|command| command.name.as_ref() == Some(command_name))
}

fn validate_scenario_command_errors_are_declared(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find_map(|slice| undeclared_scenario_command_error(document, slice))
        .map_or(Ok(()), |error_name| {
            Err(validation_issue(format!(
                "scenario references undeclared command error '{error_name}'"
            )))
        })
}

fn undeclared_scenario_command_error(
    document: &EventModelDocument,
    slice: &SliceDefinition,
) -> Option<DefinitionName> {
    slice
        .scenarios
        .iter()
        .flat_map(|scenario| scenario.command_errors.iter())
        .find(|error_name| !slice_command_errors(document, slice).contains(error_name))
        .cloned()
}

fn slice_command_errors(
    document: &EventModelDocument,
    slice: &SliceDefinition,
) -> Vec<DefinitionName> {
    slice
        .issued_commands
        .iter()
        .filter_map(|command_name| command_definition(document, command_name))
        .flat_map(|command| command.errors.iter())
        .cloned()
        .collect()
}

fn validate_state_change_command_error_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateChange)
        .find_map(|slice| uncovered_state_change_command_error(document, slice))
        .map_or(Ok(()), |error_name| {
            Err(validation_issue(format!(
                "command error '{error_name}' must be covered by a state-change scenario"
            )))
        })
}

fn uncovered_state_change_command_error(
    document: &EventModelDocument,
    slice: &SliceDefinition,
) -> Option<DefinitionName> {
    slice_command_errors(document, slice)
        .into_iter()
        .find(|error_name| !state_change_slice_covers_command_error(slice, error_name))
}

fn state_change_slice_covers_command_error(
    slice: &SliceDefinition,
    error_name: &DefinitionName,
) -> bool {
    slice
        .scenarios
        .iter()
        .any(|scenario| scenario.command_errors.contains(error_name))
}

fn validate_control_command_error_handling(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .view_definitions
        .iter()
        .find_map(|view| unhandled_control_command_error(document, view))
        .map_or(Ok(()), |unhandled| {
            Err(validation_issue(format!(
                "control '{}' does not handle command error '{}'",
                unhandled.control_label, unhandled.error_name
            )))
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct UnhandledControlCommandError {
    control_label: DefinitionName,
    error_name: DefinitionName,
}

fn unhandled_control_command_error(
    document: &EventModelDocument,
    view: &ViewDefinition,
) -> Option<UnhandledControlCommandError> {
    view.controls.iter().find_map(|control| {
        control.command.as_ref().and_then(|command_name| {
            command_definition(document, command_name).and_then(|command| {
                command
                    .errors
                    .iter()
                    .find(|error_name| !control_handles_command_error(control, error_name))
                    .map(|error_name| UnhandledControlCommandError {
                        control_label: control.label.clone(),
                        error_name: error_name.clone(),
                    })
            })
        })
    })
}

fn control_handles_command_error(
    control: &ViewControlDefinition,
    error_name: &DefinitionName,
) -> bool {
    control
        .command_error_handling
        .iter()
        .any(|handling| handling.error_name == *error_name)
}

fn validate_control_error_handling_recovery(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .view_definitions
        .iter()
        .flat_map(|view| view.controls.iter())
        .flat_map(|control| control.command_error_handling.iter())
        .find(|handling| handling.recovery_behavior == ControlErrorRecoveryBehavior::Missing)
        .map_or(Ok(()), |handling| {
            Err(validation_issue(format!(
                "error handling for '{}' must describe recovery behavior",
                handling.error_name
            )))
        })
}

fn validate_navigation_controls_declare_type(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .view_definitions
        .iter()
        .flat_map(|view| view.controls.iter())
        .find(|control| {
            control.navigation_target.is_some() && control.navigation_type == NavigationType::Absent
        })
        .map_or(Ok(()), |_| {
            Err(validation_issue(
                "navigation target must be classified as modeled_view, local_view_state, external_system, or external_workflow",
            ))
        })
}

fn validate_modeled_view_navigation_targets(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .view_definitions
        .iter()
        .flat_map(|view| view.controls.iter())
        .filter(|control| control.navigation_type == NavigationType::ModeledView)
        .filter_map(|control| control.navigation_target.as_ref())
        .find(|navigation_target| !view_exists(document, navigation_target))
        .map_or(Ok(()), |navigation_target| {
            Err(validation_issue(format!(
                "references unknown modeled view navigation target '{navigation_target}'"
            )))
        })
}

fn view_exists(document: &EventModelDocument, view_name: &DefinitionName) -> bool {
    document
        .view_definitions
        .iter()
        .any(|view| view.name == *view_name)
}

fn validate_local_view_state_navigation_targets(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .view_definitions
        .iter()
        .find_map(undeclared_local_view_state_navigation_target)
        .map_or(Ok(()), |target| {
            Err(validation_issue(format!(
                "local view state navigation target '{}' is not declared by view '{}'",
                target.navigation_target, target.view_name
            )))
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct UndeclaredLocalViewStateNavigationTarget {
    view_name: DefinitionName,
    navigation_target: DefinitionName,
}

fn undeclared_local_view_state_navigation_target(
    view: &ViewDefinition,
) -> Option<UndeclaredLocalViewStateNavigationTarget> {
    view.controls
        .iter()
        .filter(|control| control.navigation_type == NavigationType::LocalViewState)
        .filter_map(|control| control.navigation_target.as_ref())
        .find(|navigation_target| !view.local_states.contains(navigation_target))
        .map(
            |navigation_target| UndeclaredLocalViewStateNavigationTarget {
                view_name: view.name.clone(),
                navigation_target: navigation_target.clone(),
            },
        )
}

fn validate_external_workflow_navigation_targets(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .view_definitions
        .iter()
        .flat_map(|view| view.controls.iter())
        .find(|control| {
            control.navigation_type == NavigationType::ExternalWorkflow
                && control.workflow_target.is_none()
        })
        .map_or(Ok(()), |_| {
            Err(validation_issue(
                "external workflow navigation must name the target workflow",
            ))
        })
}

fn validate_board_read_model_to_command_intermediates(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .board_read_model_command_dependencies
        .iter()
        .find(|dependency| !automation_slice_exists(document, &dependency.intermediate_automation))
        .map_or(Ok(()), |dependency| {
            Err(validation_issue(format!(
                "board element between read model '{}' and command '{}' is not a declared automation",
                dependency.read_model, dependency.command
            )))
        })
}

fn automation_slice_exists(
    document: &EventModelDocument,
    automation_name: &DefinitionName,
) -> bool {
    document
        .slice_definitions
        .iter()
        .any(|slice| slice.slice_type == SliceType::Automation && slice.name == *automation_name)
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

fn validate_command_legacy_read_model_reads(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .command_definitions
        .iter()
        .find(|command| command.read_model_reads == CommandReadModelReads::Present)
        .and_then(|command| command.name.as_ref())
        .map_or(Ok(()), |command_name| {
            Err(validation_issue(format!(
                "command '{command_name}' uses legacy read-model reads"
            )))
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

fn validate_command_input_external_source_fields(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .command_definitions
        .iter()
        .flat_map(|command| {
            command
                .input_sources
                .iter()
                .filter_map(move |input_source| {
                    if let CommandInputSourceKind::ExternalField(payload_name, field_name) =
                        &input_source.source
                    {
                        Some((command, input_source, payload_name, field_name))
                    } else {
                        None
                    }
                })
        })
        .find(|(command, _, payload_name, field_name)| {
            !external_field_is_declared(command, payload_name, field_name)
        })
        .map_or(Ok(()), |(_, input_source, _, field_name)| {
            Err(validation_issue(format!(
                "command input '{}' references undeclared external input field '{}'",
                input_source.name, field_name
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

fn validate_derived_read_model_field_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .read_model_definitions
        .iter()
        .flat_map(|read_model| {
            read_model.fields.iter().filter(|field| {
                field.derivation == ReadModelFieldDerivation::DerivedWithoutScenarios
            })
        })
        .next()
        .map_or(Ok(()), |field| {
            Err(validation_issue(format!(
                "derived read model field '{}' must have a derivation scenario",
                field.name
            )))
        })
}

fn validate_absence_default_read_model_field_events(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .read_model_definitions
        .iter()
        .flat_map(|read_model| {
            read_model.fields.iter().filter(|field| {
                field.absence_default == ReadModelFieldAbsenceDefault::DefaultedWithoutAbsenceEvent
            })
        })
        .next()
        .map_or(Ok(()), |field| {
            Err(validation_issue(format!(
                "absence/default field '{}' must declare the event absence it derives from",
                field.name
            )))
        })
}

fn validate_absence_default_read_model_field_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .read_model_definitions
        .iter()
        .flat_map(|read_model| {
            read_model.fields.iter().filter(|field| {
                field.absence_default == ReadModelFieldAbsenceDefault::DefaultedWithoutScenarios
            })
        })
        .next()
        .map_or(Ok(()), |field| {
            Err(validation_issue(format!(
                "absence/default field '{}' must have an absence scenario",
                field.name
            )))
        })
}

fn validate_transitive_read_model_derivation(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .read_model_definitions
        .iter()
        .find(|read_model| {
            read_model.transitive_derivation
                == ReadModelTransitiveDerivation::TransitiveWithoutRule
        })
        .map_or(Ok(()), |read_model| {
            Err(validation_issue(format!(
                "transitive read model '{}' must declare source fields, derivation rule, and scenarios",
                read_model.name
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
