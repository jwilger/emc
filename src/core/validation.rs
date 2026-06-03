use std::collections::{BTreeMap, BTreeSet, VecDeque};

use nutype::nutype;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventModelDocument {
    name: Option<DefinitionName>,
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
    board_lanes: BoardLanes,
    board_slices: Vec<BoardSliceGraph>,
    slice_count: SliceDefinitionCount,
    slice_definitions: Vec<SliceDefinition>,
    view_definitions: Vec<ViewDefinition>,
    workflow_slice_references: BTreeSet<DefinitionName>,
    workflow_step_slices: BTreeSet<DefinitionName>,
    workflow_steps: Vec<WorkflowStep>,
    workflow_event_transitions: Vec<WorkflowEventTransition>,
    workflow_command_transitions: Vec<WorkflowCommandTransition>,
    workflow_navigation_transitions: Vec<WorkflowNavigationTransition>,
    workflow_external_trigger_transitions: Vec<WorkflowExternalTriggerTransition>,
    workflow_exit_transitions: Vec<WorkflowExitTransition>,
    duplicate_workflow_step_slice: Option<DefinitionName>,
    workflow_composition: WorkflowComposition,
    workflow_entry_step_count: WorkflowEntryStepCount,
    workflow_internal_definitions: WorkflowInternalDefinitions,
    workflow_transition_errors: BTreeSet<DefinitionName>,
    workflow_transition_outcomes: BTreeSet<DefinitionName>,
}

impl EventModelDocument {
    pub fn new(parts: EventModelDocumentParts) -> Self {
        Self {
            name: parts.name,
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
            board_lanes: parts.board_lanes,
            board_slices: parts.board_slices,
            slice_count: parts.slice_count,
            slice_definitions: parts.slice_definitions,
            view_definitions: parts.view_definitions,
            workflow_slice_references: parts.workflow_slice_references,
            workflow_step_slices: parts.workflow_step_slices,
            workflow_steps: parts.workflow_steps,
            workflow_event_transitions: parts.workflow_event_transitions,
            workflow_command_transitions: parts.workflow_command_transitions,
            workflow_navigation_transitions: parts.workflow_navigation_transitions,
            workflow_external_trigger_transitions: parts.workflow_external_trigger_transitions,
            workflow_exit_transitions: parts.workflow_exit_transitions,
            duplicate_workflow_step_slice: parts.duplicate_workflow_step_slice,
            workflow_composition: parts.workflow_composition,
            workflow_entry_step_count: parts.workflow_entry_step_count,
            workflow_internal_definitions: parts.workflow_internal_definitions,
            workflow_transition_errors: parts.workflow_transition_errors,
            workflow_transition_outcomes: parts.workflow_transition_outcomes,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventModelDocumentParts {
    name: Option<DefinitionName>,
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
    board_lanes: BoardLanes,
    board_slices: Vec<BoardSliceGraph>,
    slice_count: SliceDefinitionCount,
    slice_definitions: Vec<SliceDefinition>,
    view_definitions: Vec<ViewDefinition>,
    workflow_slice_references: BTreeSet<DefinitionName>,
    workflow_step_slices: BTreeSet<DefinitionName>,
    workflow_steps: Vec<WorkflowStep>,
    workflow_event_transitions: Vec<WorkflowEventTransition>,
    workflow_command_transitions: Vec<WorkflowCommandTransition>,
    workflow_navigation_transitions: Vec<WorkflowNavigationTransition>,
    workflow_external_trigger_transitions: Vec<WorkflowExternalTriggerTransition>,
    workflow_exit_transitions: Vec<WorkflowExitTransition>,
    duplicate_workflow_step_slice: Option<DefinitionName>,
    workflow_composition: WorkflowComposition,
    workflow_entry_step_count: WorkflowEntryStepCount,
    workflow_internal_definitions: WorkflowInternalDefinitions,
    workflow_transition_errors: BTreeSet<DefinitionName>,
    workflow_transition_outcomes: BTreeSet<DefinitionName>,
}

impl EventModelDocumentParts {
    pub fn new(file_kind: EventModelFileKind) -> Self {
        Self {
            name: None,
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
            board_lanes: BoardLanes::Absent,
            board_slices: Vec::new(),
            slice_count: SliceDefinitionCount::Zero,
            slice_definitions: Vec::new(),
            view_definitions: Vec::new(),
            workflow_slice_references: BTreeSet::new(),
            workflow_step_slices: BTreeSet::new(),
            workflow_steps: Vec::new(),
            workflow_event_transitions: Vec::new(),
            workflow_command_transitions: Vec::new(),
            workflow_navigation_transitions: Vec::new(),
            workflow_external_trigger_transitions: Vec::new(),
            workflow_exit_transitions: Vec::new(),
            duplicate_workflow_step_slice: None,
            workflow_composition: WorkflowComposition::NotComposition,
            workflow_entry_step_count: WorkflowEntryStepCount::NotComposition,
            workflow_internal_definitions: WorkflowInternalDefinitions::Absent,
            workflow_transition_errors: BTreeSet::new(),
            workflow_transition_outcomes: BTreeSet::new(),
        }
    }

    pub fn with_name(mut self, name: Option<DefinitionName>) -> Self {
        self.name = name;
        self
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

    pub fn with_board_lanes(mut self, board_lanes: BoardLanes) -> Self {
        self.board_lanes = board_lanes;
        self
    }

    pub fn with_board_slices(mut self, board_slices: Vec<BoardSliceGraph>) -> Self {
        self.board_slices = board_slices;
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

    pub fn with_workflow_slice_references(
        mut self,
        workflow_slice_references: BTreeSet<DefinitionName>,
    ) -> Self {
        self.workflow_slice_references = workflow_slice_references;
        self
    }

    pub fn with_workflow_step_slices(
        mut self,
        workflow_step_slices: BTreeSet<DefinitionName>,
    ) -> Self {
        self.workflow_step_slices = workflow_step_slices;
        self
    }

    pub fn with_workflow_steps(mut self, workflow_steps: Vec<WorkflowStep>) -> Self {
        self.workflow_steps = workflow_steps;
        self
    }

    pub fn with_workflow_event_transitions(
        mut self,
        workflow_event_transitions: Vec<WorkflowEventTransition>,
    ) -> Self {
        self.workflow_event_transitions = workflow_event_transitions;
        self
    }

    pub fn with_workflow_command_transitions(
        mut self,
        workflow_command_transitions: Vec<WorkflowCommandTransition>,
    ) -> Self {
        self.workflow_command_transitions = workflow_command_transitions;
        self
    }

    pub fn with_workflow_navigation_transitions(
        mut self,
        workflow_navigation_transitions: Vec<WorkflowNavigationTransition>,
    ) -> Self {
        self.workflow_navigation_transitions = workflow_navigation_transitions;
        self
    }

    pub fn with_workflow_external_trigger_transitions(
        mut self,
        workflow_external_trigger_transitions: Vec<WorkflowExternalTriggerTransition>,
    ) -> Self {
        self.workflow_external_trigger_transitions = workflow_external_trigger_transitions;
        self
    }

    pub fn with_workflow_exit_transitions(
        mut self,
        workflow_exit_transitions: Vec<WorkflowExitTransition>,
    ) -> Self {
        self.workflow_exit_transitions = workflow_exit_transitions;
        self
    }

    pub fn with_duplicate_workflow_step_slice(
        mut self,
        duplicate_workflow_step_slice: Option<DefinitionName>,
    ) -> Self {
        self.duplicate_workflow_step_slice = duplicate_workflow_step_slice;
        self
    }

    pub fn with_workflow_composition(mut self, workflow_composition: WorkflowComposition) -> Self {
        self.workflow_composition = workflow_composition;
        self
    }

    pub fn with_workflow_entry_step_count(
        mut self,
        workflow_entry_step_count: WorkflowEntryStepCount,
    ) -> Self {
        self.workflow_entry_step_count = workflow_entry_step_count;
        self
    }

    pub fn with_workflow_internal_definitions(
        mut self,
        workflow_internal_definitions: WorkflowInternalDefinitions,
    ) -> Self {
        self.workflow_internal_definitions = workflow_internal_definitions;
        self
    }

    pub fn with_workflow_transition_errors(
        mut self,
        workflow_transition_errors: BTreeSet<DefinitionName>,
    ) -> Self {
        self.workflow_transition_errors = workflow_transition_errors;
        self
    }

    pub fn with_workflow_transition_outcomes(
        mut self,
        workflow_transition_outcomes: BTreeSet<DefinitionName>,
    ) -> Self {
        self.workflow_transition_outcomes = workflow_transition_outcomes;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EventModelFileKind {
    Slice,
    Workflow,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowComposition {
    NotComposition,
    MissingSteps,
    DeclaresSteps,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowEntryStepCount {
    NotComposition,
    NotOne,
    One,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowInternalDefinitions {
    Absent,
    Present,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowStepRelationship {
    Alternate,
    AsyncLifecycle,
    Branch,
    Entry,
    Main,
    Other,
    Supporting,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowStepTrigger {
    Absent,
    Present,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowStepExit {
    Absent,
    Present,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowStepLifecycleRole {
    ApplicationEntryStateView,
    BootstrapRootEntryStateChange,
    Other,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ApplicationEntryState {
    AlreadyInitializedAndAuthenticated,
    AlreadyInitializedAndUnauthenticated,
    FreshAndUninitialized,
    FullyConfigured,
    PartiallyConfigured,
}

impl ApplicationEntryState {
    fn required_states() -> [Self; 5] {
        [
            Self::FreshAndUninitialized,
            Self::AlreadyInitializedAndUnauthenticated,
            Self::AlreadyInitializedAndAuthenticated,
            Self::PartiallyConfigured,
            Self::FullyConfigured,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Self::AlreadyInitializedAndAuthenticated => "already initialized and authenticated",
            Self::AlreadyInitializedAndUnauthenticated => "already initialized and unauthenticated",
            Self::FreshAndUninitialized => "fresh and uninitialized",
            Self::FullyConfigured => "fully configured",
            Self::PartiallyConfigured => "partially configured",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowStep {
    slice: DefinitionName,
    relationship: WorkflowStepRelationship,
    trigger: WorkflowStepTrigger,
    workflow_exit: WorkflowStepExit,
    transition_targets: BTreeSet<DefinitionName>,
    selected_scenario: Option<DefinitionName>,
    lifecycle_role: WorkflowStepLifecycleRole,
}

impl WorkflowStep {
    pub fn new(
        slice: DefinitionName,
        relationship: WorkflowStepRelationship,
        trigger: WorkflowStepTrigger,
        workflow_exit: WorkflowStepExit,
        transition_targets: BTreeSet<DefinitionName>,
        selected_scenario: Option<DefinitionName>,
        lifecycle_role: WorkflowStepLifecycleRole,
    ) -> Self {
        Self {
            slice,
            relationship,
            trigger,
            workflow_exit,
            transition_targets,
            selected_scenario,
            lifecycle_role,
        }
    }

    fn slice(&self) -> &DefinitionName {
        &self.slice
    }

    fn transition_targets(&self) -> &BTreeSet<DefinitionName> {
        &self.transition_targets
    }

    fn selected_scenario(&self) -> Option<&DefinitionName> {
        self.selected_scenario.as_ref()
    }

    fn is_application_entry_state_view(&self) -> bool {
        self.lifecycle_role == WorkflowStepLifecycleRole::ApplicationEntryStateView
    }

    fn is_bootstrap_root_entry_state_change(&self) -> bool {
        self.lifecycle_role == WorkflowStepLifecycleRole::BootstrapRootEntryStateChange
    }

    fn application_entry_slice(&self) -> Option<&DefinitionName> {
        self.is_application_entry_state_view()
            .then_some(&self.slice)
    }

    fn requires_incoming_transition(&self) -> bool {
        !matches!(
            (self.relationship, self.trigger, self.workflow_exit),
            (
                WorkflowStepRelationship::Entry
                    | WorkflowStepRelationship::Branch
                    | WorkflowStepRelationship::Supporting,
                _,
                _,
            ) | (_, WorkflowStepTrigger::Present, _)
                | (_, _, WorkflowStepExit::Present)
        )
    }

    fn is_entry(&self) -> bool {
        self.relationship == WorkflowStepRelationship::Entry
    }

    fn is_alternate(&self) -> bool {
        self.relationship == WorkflowStepRelationship::Alternate
    }

    fn is_main(&self) -> bool {
        self.relationship == WorkflowStepRelationship::Main
    }

    fn has_trigger(&self) -> bool {
        self.trigger == WorkflowStepTrigger::Present
    }

    fn is_exempt_from_entry_reachability(&self) -> bool {
        matches!(
            self.relationship,
            WorkflowStepRelationship::AsyncLifecycle | WorkflowStepRelationship::Supporting
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowEventTransition {
    source_slice: DefinitionName,
    target_slice: DefinitionName,
    event: DefinitionName,
}

impl WorkflowEventTransition {
    pub fn new(
        source_slice: DefinitionName,
        target_slice: DefinitionName,
        event: DefinitionName,
    ) -> Self {
        Self {
            source_slice,
            target_slice,
            event,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowCommandTransition {
    source_slice: DefinitionName,
    target_slice: DefinitionName,
    command: DefinitionName,
}

impl WorkflowCommandTransition {
    pub fn new(
        source_slice: DefinitionName,
        target_slice: DefinitionName,
        command: DefinitionName,
    ) -> Self {
        Self {
            source_slice,
            target_slice,
            command,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowNavigationTransition {
    source_slice: DefinitionName,
    target_slice: DefinitionName,
    navigation_target: DefinitionName,
}

impl WorkflowNavigationTransition {
    pub fn new(
        source_slice: DefinitionName,
        target_slice: DefinitionName,
        navigation_target: DefinitionName,
    ) -> Self {
        Self {
            source_slice,
            target_slice,
            navigation_target,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowExternalTriggerTransition {
    source_slice: DefinitionName,
    target_slice: DefinitionName,
    external_trigger: DefinitionName,
}

impl WorkflowExternalTriggerTransition {
    pub fn new(
        source_slice: DefinitionName,
        target_slice: DefinitionName,
        external_trigger: DefinitionName,
    ) -> Self {
        Self {
            source_slice,
            target_slice,
            external_trigger,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowExitTransition {
    workflow: DefinitionName,
    rationale: WorkflowExitRationale,
}

impl WorkflowExitTransition {
    pub fn new(workflow: DefinitionName, rationale: WorkflowExitRationale) -> Self {
        Self {
            workflow,
            rationale,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowExitRationale {
    Missing,
    Present,
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
pub struct BoardLane {
    id: DefinitionName,
    name: Option<DefinitionName>,
}

impl BoardLane {
    pub fn new(id: DefinitionName, name: Option<DefinitionName>) -> Self {
        Self { id, name }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BoardLanes {
    Absent,
    Present(Vec<BoardLane>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BoardElementKind {
    Automation,
    Command,
    Event,
    ExternalEvent,
    Other,
    ReadModel,
    View,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoardElement {
    id: DefinitionName,
    kind: BoardElementKind,
    lane: Option<DefinitionName>,
    name: Option<DefinitionName>,
}

impl BoardElement {
    pub fn new(
        id: DefinitionName,
        kind: BoardElementKind,
        lane: Option<DefinitionName>,
        name: Option<DefinitionName>,
    ) -> Self {
        Self {
            id,
            kind,
            lane,
            name,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoardSliceGraph {
    name: DefinitionName,
    element_ids: BTreeSet<DefinitionName>,
    elements: Vec<BoardElement>,
    connections: Vec<BoardGraphConnection>,
}

impl BoardSliceGraph {
    pub fn new(
        name: DefinitionName,
        elements: Vec<BoardElement>,
        connections: Vec<BoardGraphConnection>,
    ) -> Self {
        let element_ids = elements.iter().map(|element| element.id.clone()).collect();
        Self {
            name,
            element_ids,
            elements,
            connections,
        }
    }

    fn has_slug(&self, slug: &DefinitionName) -> bool {
        slugified_definition_name(&self.name)
            .as_ref()
            .is_some_and(|slice_slug| slice_slug == slug)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoardGraphConnection {
    from: DefinitionName,
    to: DefinitionName,
}

impl BoardGraphConnection {
    pub fn new(from: DefinitionName, to: DefinitionName) -> Self {
        Self { from, to }
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
    external_triggers: Vec<DefinitionName>,
    external_payload_variants: Vec<ExternalPayloadVariant>,
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
            external_triggers: parts.external_triggers,
            external_payload_variants: parts.external_payload_variants,
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

    fn has_slug(&self, slug: &DefinitionName) -> bool {
        slugified_definition_name(&self.name)
            .as_ref()
            .is_some_and(|slice_slug| slice_slug == slug)
    }

    fn covers_application_entry_state(&self, state: ApplicationEntryState) -> bool {
        self.scenarios
            .iter()
            .any(|scenario| scenario.references_text(state.label()))
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
    external_triggers: Vec<DefinitionName>,
    external_payload_variants: Vec<ExternalPayloadVariant>,
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
            external_triggers: Vec::new(),
            external_payload_variants: Vec::new(),
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

    pub fn with_external_triggers(mut self, external_triggers: Vec<DefinitionName>) -> Self {
        self.external_triggers = external_triggers;
        self
    }

    pub fn with_external_payload_variants(
        mut self,
        external_payload_variants: Vec<ExternalPayloadVariant>,
    ) -> Self {
        self.external_payload_variants = external_payload_variants;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AutomationTrigger {
    NotAutomation,
    MissingTrigger,
    DeclaresTriggers(Vec<DefinitionName>),
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
pub struct ExternalPayloadVariant {
    payload: DefinitionName,
    variant: DefinitionName,
}

impl ExternalPayloadVariant {
    pub fn new(payload: DefinitionName, variant: DefinitionName) -> Self {
        Self { payload, variant }
    }
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
    scenario_step_references: Vec<DefinitionName>,
    then_events: Vec<DefinitionName>,
    command_errors: Vec<DefinitionName>,
    given_streams: Vec<DefinitionName>,
    read_model_states: Vec<DefinitionName>,
    read_model_state_values: Vec<ReadModelState>,
}

impl SliceScenario {
    pub fn new(parts: SliceScenarioParts) -> Self {
        Self {
            name: parts.name,
            when_field: parts.when_field,
            scenario_set: parts.scenario_set,
            referenced_events: parts.referenced_events,
            scenario_step_references: parts.scenario_step_references,
            then_events: parts.then_events,
            command_errors: parts.command_errors,
            given_streams: parts.given_streams,
            read_model_states: parts.read_model_states,
            read_model_state_values: parts.read_model_state_values,
        }
    }

    fn references_text(&self, expected_text: &str) -> bool {
        definition_name_contains(&self.name, expected_text)
            || self
                .scenario_step_references
                .iter()
                .any(|reference| definition_name_contains(reference, expected_text))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceScenarioParts {
    name: DefinitionName,
    when_field: ScenarioStepField,
    scenario_set: ScenarioSetKind,
    referenced_events: Vec<DefinitionName>,
    scenario_step_references: Vec<DefinitionName>,
    then_events: Vec<DefinitionName>,
    command_errors: Vec<DefinitionName>,
    given_streams: Vec<DefinitionName>,
    read_model_states: Vec<DefinitionName>,
    read_model_state_values: Vec<ReadModelState>,
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
            scenario_step_references: Vec::new(),
            then_events: Vec::new(),
            command_errors: Vec::new(),
            given_streams: Vec::new(),
            read_model_states: Vec::new(),
            read_model_state_values: Vec::new(),
        }
    }

    pub fn with_referenced_events(mut self, referenced_events: Vec<DefinitionName>) -> Self {
        self.referenced_events = referenced_events;
        self
    }

    pub fn with_scenario_step_references(
        mut self,
        scenario_step_references: Vec<DefinitionName>,
    ) -> Self {
        self.scenario_step_references = scenario_step_references;
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

    pub fn with_read_model_state_values(
        mut self,
        read_model_state_values: Vec<ReadModelState>,
    ) -> Self {
        self.read_model_state_values = read_model_state_values;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelState {
    read_model: DefinitionName,
    state: DefinitionName,
}

impl ReadModelState {
    pub fn new(read_model: DefinitionName, state: DefinitionName) -> Self {
        Self { read_model, state }
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
    external_system: Option<DefinitionName>,
    payload_contract: Option<DefinitionName>,
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
            external_system: parts.external_system,
            payload_contract: parts.payload_contract,
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
    external_system: Option<DefinitionName>,
    payload_contract: Option<DefinitionName>,
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
            external_system: None,
            payload_contract: None,
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

    pub fn with_external_system(mut self, external_system: Option<DefinitionName>) -> Self {
        self.external_system = external_system;
        self
    }

    pub fn with_payload_contract(mut self, payload_contract: Option<DefinitionName>) -> Self {
        self.payload_contract = payload_contract;
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

    validate_workflow_composition_steps(document)?;

    validate_workflow_entry_step_count(document)?;

    validate_workflow_internal_definitions(document)?;

    validate_workflow_steps_compose_whole_slices(document)?;

    validate_workflow_steps_reference_composed_slices(document)?;

    validate_workflow_referenced_slices_are_used(document)?;

    validate_duplicate_workflow_step_slices(document)?;

    validate_workflow_transition_targets_known_steps(document)?;

    validate_alternate_workflow_step_rationale(document)?;

    validate_async_lifecycle_steps_not_main(document)?;

    validate_application_entry_before_bootstrap(document)?;

    validate_workflow_step_incoming_reachability(document)?;

    validate_workflow_steps_reachable_from_entry(document)?;

    validate_no_legacy_slice_scenarios(document)?;

    validate_scenario_when_fields(document)?;

    validate_duplicate_scenario_names(document)?;

    validate_acceptance_scenario_boundaries(document)?;

    validate_state_view_projector_contract_scenarios(document)?;

    validate_state_view_empty_read_model_state_scenarios(document)?;

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

    validate_translation_slice_payload_variant_scenarios(document)?;

    validate_automation_slice_triggers(document)?;

    validate_automation_slice_trigger_scenarios(document)?;

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

    validate_external_system_navigation_targets(document)?;

    validate_board_lane_ids(document)?;

    validate_board_element_lanes(document)?;

    validate_board_element_references(document)?;

    validate_board_read_model_to_command_intermediates(document)?;

    validate_external_events_not_modeled_as_automation(document)?;

    validate_board_automation_references(document)?;

    validate_unnamed_board_automation_references(document)?;

    validate_board_external_event_references(document)?;

    validate_undeclared_external_event_bridges(document)?;

    validate_board_connection_kinds(document)?;

    validate_command_board_triggers(document)?;

    validate_command_event_board_connections(document)?;

    validate_event_read_model_board_connections(document)?;

    validate_view_command_board_connections(document)?;

    validate_read_model_view_board_connections(document)?;

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
        .and_then(|()| validate_state_view_slices_own_views(document))
        .and_then(|()| validate_state_view_slices_do_not_own_commands(document))
        .and_then(|()| validate_state_change_slices_emit_events(document))
        .and_then(|()| validate_state_change_slices_do_not_own_views(document))
        .and_then(|()| validate_state_change_slices_do_not_own_read_models(document))
        .and_then(|()| validate_state_change_slices_do_not_own_automations(document))
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

    application_entry_state_coverage_issue(documents).map_or(Ok(()), |issue| {
        Err(validation_issue(format!(
            "application entry slice '{}' must cover {} state",
            issue.slice_slug,
            issue.missing_state.label()
        )))
    })?;

    unshared_workflow_event_transition(documents).map_or(Ok(()), |transition| {
        Err(validation_issue(format!(
            "transition event '{}' is not shared by source and target slices",
            transition.event
        )))
    })?;

    workflow_event_transition_unavailable_from_source(documents).map_or(Ok(()), |transition| {
        Err(validation_issue(format!(
            "transition event '{}' is not available from source slice '{}'",
            transition.event, transition.source_slice
        )))
    })?;

    workflow_event_transition_not_accepted_by_target(documents).map_or(Ok(()), |transition| {
        Err(validation_issue(format!(
            "transition event '{}' is not accepted by target slice '{}'",
            transition.event, transition.target_slice
        )))
    })?;

    workflow_command_transition_not_invoked_by_source_view(documents).map_or(Ok(()), |issue| {
        Err(validation_issue(format!(
            "transition command '{}' is not invoked by source view '{}'",
            issue.transition.command, issue.source_view
        )))
    })?;

    workflow_command_transition_not_owned_by_target_slice(documents).map_or(
        Ok(()),
        |transition| {
            Err(validation_issue(format!(
                "transition command '{}' is not owned by target slice '{}'",
                transition.command, transition.target_slice
            )))
        },
    )?;

    workflow_navigation_transition_not_owned_by_source_view(documents).map_or(Ok(()), |issue| {
        Err(validation_issue(format!(
            "navigation transition to '{}' is not owned by source view '{}'",
            issue.transition.navigation_target, issue.source_view
        )))
    })?;

    workflow_navigation_transition_not_resolved_by_target_step(documents).map_or(
        Ok(()),
        |transition| {
            Err(validation_issue(format!(
                "navigation target '{}' does not resolve to target step '{}'",
                transition.navigation_target, transition.target_slice
            )))
        },
    )?;

    workflow_external_trigger_not_declared_by_target_slice(documents).map_or(
        Ok(()),
        |transition| {
            Err(validation_issue(format!(
                "external trigger '{}' is not declared by target slice '{}'",
                transition.external_trigger, transition.target_slice
            )))
        },
    )?;

    workflow_exit_without_rationale(documents).map_or(Ok(()), |transition| {
        Err(validation_issue(format!(
            "workflow exit to '{}' must declare why the exit is reached",
            transition.workflow
        )))
    })?;

    disconnected_main_workflow_board_slice(documents).map_or(Ok(()), |board_slice| {
        Err(validation_issue(format!(
            "board slice '{}' has disconnected main-path elements",
            board_slice.name
        )))
    })?;

    unhandled_workflow_slice_outcome(documents).map_or(Ok(()), |unhandled| {
        Err(validation_issue(format!(
            "workflow '{}' does not handle outcome '{}' from slice '{}'",
            unhandled.workflow_name, unhandled.outcome_label, unhandled.slice_name
        )))
    })?;

    workflow_transition_command_error(documents).map_or(Ok(()), |error_name| {
        Err(validation_issue(format!(
            "workflow transition cannot use command-local error '{error_name}' as a business outcome"
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

fn slugified_definition_name(name: &DefinitionName) -> Option<DefinitionName> {
    DefinitionName::try_new(
        name.as_ref()
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
            .0,
    )
    .ok()
}

fn definition_name_contains(name: &DefinitionName, expected_text: &str) -> bool {
    name.as_ref().to_ascii_lowercase().contains(expected_text)
}

fn workflow_transition_command_error(documents: &[EventModelDocument]) -> Option<DefinitionName> {
    let command_errors = documents
        .iter()
        .flat_map(|document| document.command_definitions.iter())
        .flat_map(|command| command.errors.iter())
        .collect::<BTreeSet<_>>();

    documents
        .iter()
        .flat_map(|document| document.workflow_transition_errors.iter())
        .find(|error_name| command_errors.contains(error_name))
        .cloned()
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ApplicationEntryStateCoverageIssue {
    slice_slug: DefinitionName,
    missing_state: ApplicationEntryState,
}

fn application_entry_state_coverage_issue(
    documents: &[EventModelDocument],
) -> Option<ApplicationEntryStateCoverageIssue> {
    documents
        .iter()
        .flat_map(application_entry_step_slices)
        .find_map(|slice_slug| {
            application_entry_slice_by_slug(documents, slice_slug).and_then(|slice| {
                missing_application_entry_state(slice).map(|missing_state| {
                    ApplicationEntryStateCoverageIssue {
                        slice_slug: slice_slug.clone(),
                        missing_state,
                    }
                })
            })
        })
}

fn application_entry_step_slices(document: &EventModelDocument) -> Vec<&DefinitionName> {
    document
        .workflow_steps
        .iter()
        .filter_map(WorkflowStep::application_entry_slice)
        .collect()
}

fn application_entry_slice_by_slug<'a>(
    documents: &'a [EventModelDocument],
    slice_slug: &DefinitionName,
) -> Option<&'a SliceDefinition> {
    documents
        .iter()
        .flat_map(|document| document.slice_definitions.iter())
        .find(|slice| slice.has_slug(slice_slug))
}

fn missing_application_entry_state(slice: &SliceDefinition) -> Option<ApplicationEntryState> {
    ApplicationEntryState::required_states()
        .into_iter()
        .find(|state| !slice.covers_application_entry_state(*state))
}

fn unshared_workflow_event_transition(
    documents: &[EventModelDocument],
) -> Option<WorkflowEventTransition> {
    documents
        .iter()
        .flat_map(|document| document.workflow_event_transitions.iter())
        .find(|transition| {
            !workflow_transition_slice_has_event(
                documents,
                &transition.source_slice,
                &transition.event,
            ) && !workflow_transition_slice_has_event(
                documents,
                &transition.target_slice,
                &transition.event,
            )
        })
        .cloned()
}

fn workflow_event_transition_unavailable_from_source(
    documents: &[EventModelDocument],
) -> Option<WorkflowEventTransition> {
    documents
        .iter()
        .flat_map(|document| document.workflow_event_transitions.iter())
        .find(|transition| {
            !workflow_transition_slice_has_event(
                documents,
                &transition.source_slice,
                &transition.event,
            )
        })
        .cloned()
}

fn workflow_event_transition_not_accepted_by_target(
    documents: &[EventModelDocument],
) -> Option<WorkflowEventTransition> {
    documents
        .iter()
        .flat_map(|document| document.workflow_event_transitions.iter())
        .find(|transition| {
            !workflow_transition_slice_has_event(
                documents,
                &transition.target_slice,
                &transition.event,
            )
        })
        .cloned()
}

fn workflow_transition_slice_has_event(
    documents: &[EventModelDocument],
    slice_slug: &DefinitionName,
    event: &DefinitionName,
) -> bool {
    application_entry_slice_by_slug(documents, slice_slug)
        .is_some_and(|slice| slice.owned_events.contains(event))
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowCommandTransitionInvocationIssue {
    transition: WorkflowCommandTransition,
    source_view: DefinitionName,
}

fn workflow_command_transition_not_invoked_by_source_view(
    documents: &[EventModelDocument],
) -> Option<WorkflowCommandTransitionInvocationIssue> {
    documents
        .iter()
        .flat_map(|document| document.workflow_command_transitions.iter())
        .find_map(|transition| {
            if workflow_command_transition_has_source_control(documents, transition) {
                None
            } else {
                Some(WorkflowCommandTransitionInvocationIssue {
                    transition: transition.clone(),
                    source_view: workflow_command_transition_source_view(documents, transition),
                })
            }
        })
}

fn workflow_command_transition_not_owned_by_target_slice(
    documents: &[EventModelDocument],
) -> Option<WorkflowCommandTransition> {
    documents
        .iter()
        .flat_map(|document| document.workflow_command_transitions.iter())
        .find(|transition| {
            !workflow_transition_slice_owns_command(
                documents,
                &transition.target_slice,
                &transition.command,
            )
        })
        .cloned()
}

fn workflow_transition_slice_owns_command(
    documents: &[EventModelDocument],
    slice_slug: &DefinitionName,
    command: &DefinitionName,
) -> bool {
    application_entry_slice_by_slug(documents, slice_slug)
        .is_some_and(|slice| slice.issued_commands.contains(command))
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WorkflowNavigationTransitionSourceIssue {
    transition: WorkflowNavigationTransition,
    source_view: DefinitionName,
}

fn workflow_navigation_transition_not_owned_by_source_view(
    documents: &[EventModelDocument],
) -> Option<WorkflowNavigationTransitionSourceIssue> {
    documents
        .iter()
        .flat_map(|document| document.workflow_navigation_transitions.iter())
        .find_map(|transition| {
            if workflow_navigation_transition_has_source_control(documents, transition) {
                None
            } else {
                Some(WorkflowNavigationTransitionSourceIssue {
                    transition: transition.clone(),
                    source_view: workflow_navigation_transition_source_view(documents, transition),
                })
            }
        })
}

fn workflow_navigation_transition_not_resolved_by_target_step(
    documents: &[EventModelDocument],
) -> Option<WorkflowNavigationTransition> {
    documents
        .iter()
        .flat_map(|document| document.workflow_navigation_transitions.iter())
        .find(|transition| !workflow_navigation_transition_resolves_target(documents, transition))
        .cloned()
}

fn workflow_navigation_transition_resolves_target(
    documents: &[EventModelDocument],
    transition: &WorkflowNavigationTransition,
) -> bool {
    target_slice_owns_navigation_view(documents, transition)
        || target_state_change_exposes_navigation_view(documents, transition)
}

fn target_slice_owns_navigation_view(
    documents: &[EventModelDocument],
    transition: &WorkflowNavigationTransition,
) -> bool {
    application_entry_slice_by_slug(documents, &transition.target_slice).is_some_and(|slice| {
        slice
            .owned_views
            .iter()
            .any(|view| view == &transition.navigation_target)
    })
}

fn target_state_change_exposes_navigation_view(
    documents: &[EventModelDocument],
    transition: &WorkflowNavigationTransition,
) -> bool {
    documents.iter().any(|document| {
        document.slice_definitions.iter().any(|slice| {
            slice.has_slug(&transition.target_slice) && {
                slice.slice_type == SliceType::StateChange
                    && view_exists(document, &transition.navigation_target)
            }
        })
    })
}

fn workflow_external_trigger_not_declared_by_target_slice(
    documents: &[EventModelDocument],
) -> Option<WorkflowExternalTriggerTransition> {
    documents
        .iter()
        .flat_map(|document| document.workflow_external_trigger_transitions.iter())
        .find(|transition| {
            !workflow_transition_slice_declares_external_trigger(
                documents,
                &transition.target_slice,
                &transition.external_trigger,
            )
        })
        .cloned()
}

fn workflow_transition_slice_declares_external_trigger(
    documents: &[EventModelDocument],
    slice_slug: &DefinitionName,
    external_trigger: &DefinitionName,
) -> bool {
    application_entry_slice_by_slug(documents, slice_slug)
        .is_some_and(|slice| slice.external_triggers.contains(external_trigger))
}

fn workflow_exit_without_rationale(
    documents: &[EventModelDocument],
) -> Option<WorkflowExitTransition> {
    documents
        .iter()
        .flat_map(|document| document.workflow_exit_transitions.iter())
        .find(|transition| transition.rationale == WorkflowExitRationale::Missing)
        .cloned()
}

fn disconnected_main_workflow_board_slice(
    documents: &[EventModelDocument],
) -> Option<BoardSliceGraph> {
    documents
        .iter()
        .flat_map(|document| document.workflow_steps.iter())
        .filter(|step| step.is_main())
        .filter_map(|step| board_slice_by_slug(documents, step.slice()))
        .find(|board_slice| board_slice_has_disconnected_elements(board_slice))
        .cloned()
}

fn board_slice_by_slug<'a>(
    documents: &'a [EventModelDocument],
    slice_slug: &DefinitionName,
) -> Option<&'a BoardSliceGraph> {
    documents
        .iter()
        .flat_map(|document| document.board_slices.iter())
        .find(|board_slice| board_slice.has_slug(slice_slug))
}

fn board_slice_has_disconnected_elements(board_slice: &BoardSliceGraph) -> bool {
    let Some(first) = board_slice.element_ids.iter().next() else {
        return false;
    };
    if board_slice.element_ids.len() < 2 {
        return false;
    }

    let adjacency = board_slice_adjacency(board_slice);
    let mut seen = BTreeSet::from([first.clone()]);
    let mut pending = VecDeque::from([first.clone()]);
    while let Some(current) = pending.pop_front() {
        for next in adjacency.get(&current).into_iter().flatten() {
            if seen.insert(next.clone()) {
                pending.push_back(next.clone());
            }
        }
    }
    seen.len() != board_slice.element_ids.len()
}

fn board_slice_adjacency(
    board_slice: &BoardSliceGraph,
) -> BTreeMap<DefinitionName, BTreeSet<DefinitionName>> {
    board_slice
        .connections
        .iter()
        .filter(|connection| {
            board_slice.element_ids.contains(&connection.from)
                && board_slice.element_ids.contains(&connection.to)
        })
        .fold(
            board_slice
                .element_ids
                .iter()
                .cloned()
                .map(|element_id| (element_id, BTreeSet::new()))
                .collect::<BTreeMap<_, _>>(),
            |mut adjacency, connection| {
                adjacency
                    .entry(connection.from.clone())
                    .or_default()
                    .insert(connection.to.clone());
                adjacency
                    .entry(connection.to.clone())
                    .or_default()
                    .insert(connection.from.clone());
                adjacency
            },
        )
}

fn workflow_navigation_transition_has_source_control(
    documents: &[EventModelDocument],
    transition: &WorkflowNavigationTransition,
) -> bool {
    workflow_transition_source_views(documents, &transition.source_slice)
        .iter()
        .any(|view| view_navigates_to(view, &transition.navigation_target))
}

fn view_navigates_to(view: &ViewDefinition, navigation_target: &DefinitionName) -> bool {
    view.controls
        .iter()
        .any(|control| control.navigation_target.as_ref() == Some(navigation_target))
}

fn workflow_navigation_transition_source_view(
    documents: &[EventModelDocument],
    transition: &WorkflowNavigationTransition,
) -> DefinitionName {
    workflow_transition_source_views(documents, &transition.source_slice)
        .first()
        .map(|view| view.name.clone())
        .unwrap_or_else(|| transition.source_slice.clone())
}

fn workflow_command_transition_has_source_control(
    documents: &[EventModelDocument],
    transition: &WorkflowCommandTransition,
) -> bool {
    workflow_transition_source_views(documents, &transition.source_slice)
        .iter()
        .any(|view| view_invokes_command(view, &transition.command))
}

fn workflow_transition_source_views<'a>(
    documents: &'a [EventModelDocument],
    source_slice: &DefinitionName,
) -> Vec<&'a ViewDefinition> {
    application_entry_slice_by_slug(documents, source_slice)
        .into_iter()
        .flat_map(|slice| slice.owned_views.iter())
        .filter_map(|view_name| view_definition_by_name(documents, view_name))
        .collect()
}

fn view_definition_by_name<'a>(
    documents: &'a [EventModelDocument],
    view_name: &DefinitionName,
) -> Option<&'a ViewDefinition> {
    documents
        .iter()
        .flat_map(|document| document.view_definitions.iter())
        .find(|view| &view.name == view_name)
}

fn view_invokes_command(view: &ViewDefinition, command: &DefinitionName) -> bool {
    view.controls
        .iter()
        .any(|control| control.command.as_ref() == Some(command))
}

fn workflow_command_transition_source_view(
    documents: &[EventModelDocument],
    transition: &WorkflowCommandTransition,
) -> DefinitionName {
    workflow_transition_source_views(documents, &transition.source_slice)
        .first()
        .map(|view| view.name.clone())
        .unwrap_or_else(|| transition.source_slice.clone())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct UnhandledWorkflowSliceOutcome {
    workflow_name: DefinitionName,
    slice_name: DefinitionName,
    outcome_label: DefinitionName,
}

fn unhandled_workflow_slice_outcome(
    documents: &[EventModelDocument],
) -> Option<UnhandledWorkflowSliceOutcome> {
    let workflow = documents
        .iter()
        .find(|document| !document.workflow_transition_outcomes.is_empty())?;
    let workflow_name = workflow.name.clone()?;

    documents
        .iter()
        .flat_map(|document| document.slice_definitions.iter())
        .find_map(|slice| {
            slice
                .outcomes
                .iter()
                .find(|outcome| {
                    !workflow
                        .workflow_transition_outcomes
                        .contains(&outcome.label)
                })
                .map(|outcome| UnhandledWorkflowSliceOutcome {
                    workflow_name: workflow_name.clone(),
                    slice_name: slice.name.clone(),
                    outcome_label: outcome.label.clone(),
                })
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

fn validate_workflow_composition_steps(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition == WorkflowComposition::MissingSteps {
        Err(validation_issue("workflow composition must declare steps"))
    } else {
        Ok(())
    }
}

fn validate_workflow_entry_step_count(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_entry_step_count == WorkflowEntryStepCount::NotOne {
        Err(validation_issue(
            "workflow must declare exactly one entry step",
        ))
    } else {
        Ok(())
    }
}

fn validate_workflow_internal_definitions(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.file_kind == EventModelFileKind::Workflow
        && document.workflow_composition != WorkflowComposition::NotComposition
        && document.workflow_internal_definitions == WorkflowInternalDefinitions::Present
    {
        Err(validation_issue(
            "workflow files must not define commands, views, read models, automations, or scenarios",
        ))
    } else {
        Ok(())
    }
}

fn validate_workflow_steps_compose_whole_slices(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition != WorkflowComposition::DeclaresSteps {
        return Ok(());
    }

    document
        .workflow_steps
        .iter()
        .find_map(|step| {
            step.selected_scenario()
                .map(|scenario| (step.slice(), scenario))
        })
        .map_or(Ok(()), |(step_slice, scenario)| {
            Err(validation_issue(format!(
                "workflow step '{step_slice}' must compose the whole slice, not scenario '{scenario}'"
            )))
        })
}

fn validate_workflow_steps_reference_composed_slices(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition == WorkflowComposition::DeclaresSteps {
        document
            .workflow_step_slices
            .iter()
            .find(|step_slice| !document.workflow_slice_references.contains(step_slice))
            .map_or(Ok(()), |step_slice| {
                Err(validation_issue(format!(
                    "workflow step '{step_slice}' does not reference a composed slice"
                )))
            })
    } else {
        Ok(())
    }
}

fn validate_workflow_referenced_slices_are_used(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition == WorkflowComposition::DeclaresSteps {
        document
            .workflow_slice_references
            .iter()
            .find(|slice_reference| !document.workflow_step_slices.contains(slice_reference))
            .map_or(Ok(()), |slice_reference| {
                Err(validation_issue(format!(
                    "referenced slice '{slice_reference}' is not used by workflow steps"
                )))
            })
    } else {
        Ok(())
    }
}

fn validate_duplicate_workflow_step_slices(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .duplicate_workflow_step_slice
        .as_ref()
        .map_or(Ok(()), |step_slice| {
            Err(validation_issue(format!(
                "workflow step slice '{step_slice}' is duplicated"
            )))
        })
}

fn validate_workflow_transition_targets_known_steps(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition != WorkflowComposition::DeclaresSteps {
        return Ok(());
    }

    unknown_workflow_transition_target(&document.workflow_steps, &document.workflow_step_slices)
        .map_or(Ok(()), |target| {
            Err(validation_issue(format!(
                "transition targets unknown workflow step '{target}'"
            )))
        })
}

fn unknown_workflow_transition_target<'a>(
    workflow_steps: &'a [WorkflowStep],
    workflow_step_slices: &BTreeSet<DefinitionName>,
) -> Option<&'a DefinitionName> {
    workflow_steps
        .iter()
        .flat_map(|step| step.transition_targets().iter())
        .find(|target| !workflow_step_slices.contains(*target))
}

fn validate_alternate_workflow_step_rationale(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition != WorkflowComposition::DeclaresSteps {
        return Ok(());
    }

    let incoming_step_slices = workflow_incoming_step_slices(&document.workflow_steps);
    document
        .workflow_steps
        .iter()
        .find(|step| {
            step.is_alternate()
                && !step.has_trigger()
                && !incoming_step_slices.contains(step.slice())
        })
        .map_or(Ok(()), |step| {
            Err(validation_issue(format!(
                "alternate workflow step '{}' must declare a trigger or incoming transition",
                step.slice()
            )))
        })
}

fn validate_async_lifecycle_steps_not_main(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition != WorkflowComposition::DeclaresSteps {
        return Ok(());
    }

    document
        .workflow_steps
        .iter()
        .find(|step| step.is_main() && step.has_trigger())
        .map_or(Ok(()), |step| {
            Err(validation_issue(format!(
                "async lifecycle step '{}' must be alternate or async_lifecycle",
                step.slice()
            )))
        })
}

fn validate_application_entry_before_bootstrap(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition != WorkflowComposition::DeclaresSteps {
        return Ok(());
    }

    if workflow_bootstraps_root_before_application_entry(&document.workflow_steps) {
        Err(validation_issue(
            "should model the application entry root bootstrap state view before bootstrap",
        ))
    } else {
        Ok(())
    }
}

fn workflow_bootstraps_root_before_application_entry(workflow_steps: &[WorkflowStep]) -> bool {
    workflow_steps
        .iter()
        .position(WorkflowStep::is_bootstrap_root_entry_state_change)
        .is_some_and(|bootstrap_index| {
            !workflow_steps
                .iter()
                .take(bootstrap_index)
                .any(WorkflowStep::is_application_entry_state_view)
        })
}

fn validate_workflow_step_incoming_reachability(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition != WorkflowComposition::DeclaresSteps {
        return Ok(());
    }

    let incoming_step_slices = workflow_incoming_step_slices(&document.workflow_steps);
    document
        .workflow_steps
        .iter()
        .find(|step| {
            step.requires_incoming_transition() && !incoming_step_slices.contains(step.slice())
        })
        .map_or(Ok(()), |step| {
            Err(validation_issue(format!(
                "workflow step '{}' has no incoming transition",
                step.slice()
            )))
        })
}

fn workflow_incoming_step_slices(workflow_steps: &[WorkflowStep]) -> BTreeSet<DefinitionName> {
    workflow_steps
        .iter()
        .flat_map(|step| step.transition_targets().iter().cloned())
        .collect()
}

fn validate_workflow_steps_reachable_from_entry(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    if document.workflow_composition != WorkflowComposition::DeclaresSteps {
        return Ok(());
    }

    workflow_entry_step_slice(&document.workflow_steps).map_or(Ok(()), |entry_slice| {
        let reachable_step_slices =
            reachable_workflow_step_slices_from_entry(entry_slice, &document.workflow_steps);
        document
            .workflow_steps
            .iter()
            .find(|step| {
                !step.is_exempt_from_entry_reachability()
                    && !reachable_step_slices.contains(step.slice())
            })
            .map_or(Ok(()), |step| {
                Err(validation_issue(format!(
                    "workflow step '{}' is not reachable from entry step '{entry_slice}'",
                    step.slice()
                )))
            })
    })
}

fn workflow_entry_step_slice(workflow_steps: &[WorkflowStep]) -> Option<&DefinitionName> {
    workflow_steps
        .iter()
        .find(|step| step.is_entry())
        .map(WorkflowStep::slice)
}

fn reachable_workflow_step_slices_from_entry(
    entry_slice: &DefinitionName,
    workflow_steps: &[WorkflowStep],
) -> BTreeSet<DefinitionName> {
    let transitions_by_slice = workflow_transitions_by_slice(workflow_steps);
    let mut reachable_step_slices = BTreeSet::from([entry_slice.clone()]);
    let mut pending_step_slices = VecDeque::from([entry_slice.clone()]);

    while let Some(current_step_slice) = pending_step_slices.pop_front() {
        for target_step_slice in transitions_by_slice
            .get(&current_step_slice)
            .into_iter()
            .flatten()
        {
            if reachable_step_slices.insert(target_step_slice.clone()) {
                pending_step_slices.push_back(target_step_slice.clone());
            }
        }
    }

    reachable_step_slices
}

fn workflow_transitions_by_slice(
    workflow_steps: &[WorkflowStep],
) -> BTreeMap<DefinitionName, BTreeSet<DefinitionName>> {
    workflow_steps
        .iter()
        .map(|step| (step.slice().clone(), step.transition_targets().clone()))
        .collect()
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

fn validate_state_view_empty_read_model_state_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateView)
        .find_map(|slice| missing_state_view_empty_state(document, slice))
        .map_or(Ok(()), |missing| {
            Err(validation_issue(format!(
                "state-view slice '{}' must include a scenario for empty state of read model '{}'",
                missing.slice_name, missing.read_model
            )))
        })
}

fn validate_state_view_slices_own_views(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find(|slice| slice.is_state_view() && slice.owned_views.is_empty())
        .map_or(Ok(()), |slice| {
            Err(validation_issue(format!(
                "state_view slice '{}' must own at least one view",
                slice.name
            )))
        })
}

fn validate_state_view_slices_do_not_own_commands(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.is_state_view())
        .flat_map(|slice| {
            slice
                .issued_commands
                .iter()
                .map(move |command| (slice, command))
        })
        .next()
        .map_or(Ok(()), |(slice, command)| {
            Err(validation_issue(format!(
                "state_view slice '{}' must not own command '{}'",
                slice.name, command
            )))
        })
}

fn validate_state_change_slices_emit_events(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .find(|slice| slice.slice_type == SliceType::StateChange && slice.owned_events.is_empty())
        .map_or(Ok(()), |slice| {
            Err(validation_issue(format!(
                "state_change slice '{}' must emit at least one event",
                slice.name
            )))
        })
}

fn validate_state_change_slices_do_not_own_views(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateChange)
        .flat_map(|slice| slice.owned_views.iter().map(move |view| (slice, view)))
        .next()
        .map_or(Ok(()), |(slice, view)| {
            Err(validation_issue(format!(
                "state_change slice '{}' must not own view '{}'",
                slice.name, view
            )))
        })
}

fn validate_state_change_slices_do_not_own_read_models(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateChange)
        .flat_map(|slice| {
            slice
                .owned_read_models
                .iter()
                .map(move |read_model| (slice, read_model))
        })
        .next()
        .map_or(Ok(()), |(slice, read_model)| {
            Err(validation_issue(format!(
                "state_change slice '{}' must not own read model '{}'",
                slice.name, read_model
            )))
        })
}

fn validate_state_change_slices_do_not_own_automations(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::StateChange)
        .flat_map(|slice| {
            slice
                .owned_automations
                .iter()
                .map(move |automation| (slice, automation))
        })
        .next()
        .map_or(Ok(()), |(slice, automation)| {
            Err(validation_issue(format!(
                "state_change slice '{}' must not own automation '{}'",
                slice.name, automation
            )))
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MissingStateViewEmptyState {
    slice_name: DefinitionName,
    read_model: DefinitionName,
}

fn missing_state_view_empty_state(
    document: &EventModelDocument,
    slice: &SliceDefinition,
) -> Option<MissingStateViewEmptyState> {
    read_models_for_slice_views(document, slice)
        .into_iter()
        .find(|read_model| !slice_has_read_model_state(slice, read_model, "empty"))
        .map(|read_model| MissingStateViewEmptyState {
            slice_name: slice.name.clone(),
            read_model,
        })
}

fn slice_has_read_model_state(
    slice: &SliceDefinition,
    read_model: &DefinitionName,
    expected_state: &str,
) -> bool {
    slice.scenarios.iter().any(|scenario| {
        scenario
            .read_model_state_values
            .iter()
            .any(|state| state.read_model == *read_model && state.state.as_ref() == expected_state)
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

fn validate_translation_slice_payload_variant_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::Translation)
        .find_map(missing_translation_payload_variant_scenario)
        .map_or(Ok(()), |missing| {
            Err(validation_issue(format!(
                "translation slice '{}' must include a scenario for external payload variant '{}'",
                missing.slice_name, missing.variant
            )))
        })
}

fn validate_board_lane_ids(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    let BoardLanes::Present(board_lanes) = &document.board_lanes else {
        return Ok(());
    };

    if board_lanes
        .iter()
        .any(|lane| !is_canonical_board_lane_id(&lane.id))
    {
        return Err(validation_issue(
            "board lanes must be exactly ux, actions, and events",
        ));
    }

    if let Some(lane_id) = duplicated_board_lane_id(board_lanes) {
        return Err(validation_issue(format!(
            "board lane id '{lane_id}' is duplicated"
        )));
    }

    if let Some(lane_id) = required_board_lane_ids()
        .into_iter()
        .find(|lane_id| !board_lanes.iter().any(|lane| lane.id.as_ref() == *lane_id))
    {
        return Err(validation_issue(format!(
            "board lanes must include canonical lane '{lane_id}'"
        )));
    }

    board_lane_name_mismatch(board_lanes).map_or(Ok(()), |mismatch| {
        Err(validation_issue(format!(
            "board lane '{}' must be named '{}'",
            mismatch.lane_id, mismatch.canonical_name
        )))
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BoardLaneNameMismatch {
    lane_id: DefinitionName,
    canonical_name: &'static str,
}

fn board_lane_name_mismatch(board_lanes: &[BoardLane]) -> Option<BoardLaneNameMismatch> {
    board_lanes.iter().find_map(|lane| {
        canonical_board_lane_name(&lane.id).and_then(|canonical_name| {
            lane.name
                .as_ref()
                .filter(|name| name.as_ref() != canonical_name)
                .map(|_name| BoardLaneNameMismatch {
                    lane_id: lane.id.clone(),
                    canonical_name,
                })
        })
    })
}

fn is_canonical_board_lane_id(lane_id: &DefinitionName) -> bool {
    required_board_lane_ids()
        .into_iter()
        .any(|canonical| lane_id.as_ref() == canonical)
}

fn duplicated_board_lane_id(board_lanes: &[BoardLane]) -> Option<DefinitionName> {
    let mut seen = BTreeSet::new();
    board_lanes.iter().find_map(|lane| {
        if seen.insert(lane.id.clone()) {
            None
        } else {
            Some(lane.id.clone())
        }
    })
}

fn required_board_lane_ids() -> [&'static str; 3] {
    ["ux", "actions", "events"]
}

fn canonical_board_lane_name(lane_id: &DefinitionName) -> Option<&'static str> {
    match lane_id.as_ref() {
        "ux" => Some("People, Views, and Translations"),
        "actions" => Some("Commands and Projections"),
        "events" => Some("Stored Facts"),
        _ => None,
    }
}

fn validate_board_element_lanes(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    board_element_lane_mismatch(document).map_or(Ok(()), |mismatch| {
        Err(validation_issue(format!(
            "board element '{}' of kind '{}' must be on lane '{}'",
            mismatch.element_id,
            board_element_kind_label(mismatch.kind),
            mismatch.expected_lane
        )))
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BoardElementLaneMismatch {
    element_id: DefinitionName,
    kind: BoardElementKind,
    expected_lane: &'static str,
}

fn board_element_lane_mismatch(document: &EventModelDocument) -> Option<BoardElementLaneMismatch> {
    document
        .board_slices
        .iter()
        .flat_map(|board_slice| board_slice.elements.iter())
        .find_map(|element| {
            expected_board_element_lane(element.kind).and_then(|expected_lane| {
                element
                    .lane
                    .as_ref()
                    .filter(|actual_lane| actual_lane.as_ref() != expected_lane)
                    .map(|_actual_lane| BoardElementLaneMismatch {
                        element_id: element.id.clone(),
                        kind: element.kind,
                        expected_lane,
                    })
            })
        })
}

fn expected_board_element_lane(kind: BoardElementKind) -> Option<&'static str> {
    match kind {
        BoardElementKind::Automation | BoardElementKind::ExternalEvent | BoardElementKind::View => {
            Some("ux")
        }
        BoardElementKind::Command | BoardElementKind::ReadModel => Some("actions"),
        BoardElementKind::Event => Some("events"),
        BoardElementKind::Other => None,
    }
}

fn board_element_kind_label(kind: BoardElementKind) -> &'static str {
    match kind {
        BoardElementKind::Automation => "automation",
        BoardElementKind::Command => "command",
        BoardElementKind::Event => "event",
        BoardElementKind::ExternalEvent => "external_event",
        BoardElementKind::Other => "other",
        BoardElementKind::ReadModel => "read_model",
        BoardElementKind::View => "view",
    }
}

fn validate_board_element_references(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    unknown_board_element_reference(document).map_or(Ok(()), |reference| {
        Err(validation_issue(format!(
            "board element '{}' references unknown {} '{}'",
            reference.element_id,
            board_element_kind_label(reference.kind),
            reference.name
        )))
    })
}

fn validate_board_automation_references(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    unknown_board_automation_reference(document).map_or(Ok(()), |reference| {
        Err(validation_issue(format!(
            "board element '{}' references unknown automation '{}'",
            reference.element_id, reference.name
        )))
    })
}

fn validate_external_events_not_modeled_as_automation(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    external_event_modeled_as_automation(document).map_or(Ok(()), |external_event| {
        Err(validation_issue(format!(
            "external event '{external_event}' must not be modeled as automation"
        )))
    })
}

fn validate_unnamed_board_automation_references(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    undeclared_unnamed_board_automation(document).map_or(Ok(()), |element_id| {
        Err(validation_issue(format!(
            "automation board element '{element_id}' is not declared by an automation slice"
        )))
    })
}

fn validate_board_external_event_references(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    unknown_board_external_event_reference(document).map_or(Ok(()), |reference| {
        Err(validation_issue(format!(
            "board element '{}' references unknown external_event '{}'",
            reference.element_id, reference.name
        )))
    })
}

fn validate_undeclared_external_event_bridges(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    undeclared_external_event_bridge(document).map_or(Ok(()), |element_id| {
        Err(validation_issue(format!(
            "board element '{element_id}' is not declared"
        )))
    })
}

fn validate_board_connection_kinds(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    invalid_board_connection(document).map_or(Ok(()), |connection| {
        Err(validation_issue(format!(
            "invalid board connection '{}' ({}) -> '{}' ({})",
            connection.from,
            board_element_kind_label(connection.from_kind),
            connection.to,
            board_element_kind_label(connection.to_kind)
        )))
    })
}

fn validate_command_board_triggers(document: &EventModelDocument) -> Result<(), ValidationIssue> {
    command_board_element_without_incoming_trigger(document).map_or(Ok(()), |element_id| {
        Err(validation_issue(format!(
            "command board element '{element_id}' has no incoming trigger"
        )))
    })
}

fn validate_command_event_board_connections(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    command_event_board_connection_without_producer(document).map_or(Ok(()), |issue| {
        Err(validation_issue(format!(
            "board connects command '{}' to event '{}' that it does not produce",
            issue.command, issue.event
        )))
    })
}

fn validate_event_read_model_board_connections(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    event_read_model_board_connection_without_projection(document).map_or(Ok(()), |issue| {
        Err(validation_issue(format!(
            "board connects event '{}' to read model '{}' but the read model does not project that event",
            issue.event, issue.read_model
        )))
    })
}

fn validate_view_command_board_connections(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    view_command_board_connection_without_control(document).map_or(Ok(()), |issue| {
        Err(validation_issue(format!(
            "board connects view '{}' to command '{}' without an owned control",
            issue.view, issue.command
        )))
    })
}

fn validate_read_model_view_board_connections(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    read_model_board_element_feeding_view_without_update(document).map_or(Ok(()), |element_id| {
        Err(validation_issue(format!(
            "read_model board element '{element_id}' has no incoming event/update"
        )))
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct InvalidBoardConnection {
    from: DefinitionName,
    from_kind: BoardElementKind,
    to: DefinitionName,
    to_kind: BoardElementKind,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct CommandEventBoardConnectionIssue {
    command: DefinitionName,
    event: DefinitionName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct EventReadModelBoardConnectionIssue {
    event: DefinitionName,
    read_model: DefinitionName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ViewCommandBoardConnectionIssue {
    view: DefinitionName,
    command: DefinitionName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct UnknownBoardElementReference {
    element_id: DefinitionName,
    kind: BoardElementKind,
    name: DefinitionName,
}

fn unknown_board_element_reference(
    document: &EventModelDocument,
) -> Option<UnknownBoardElementReference> {
    document
        .board_slices
        .iter()
        .flat_map(|board_slice| board_slice.elements.iter())
        .filter_map(|element| {
            board_element_reference_definition_kind(element.kind).and_then(|definition_kind| {
                element
                    .name
                    .as_ref()
                    .map(|name| (element, name, definition_kind))
            })
        })
        .find(|(_element, name, definition_kind)| {
            !document
                .named_definitions
                .iter()
                .any(|definition| &definition.kind == definition_kind && &definition.name == *name)
        })
        .map(
            |(element, name, _definition_kind)| UnknownBoardElementReference {
                element_id: element.id.clone(),
                kind: element.kind,
                name: name.clone(),
            },
        )
}

fn board_element_reference_definition_kind(kind: BoardElementKind) -> Option<DefinitionKind> {
    match kind {
        BoardElementKind::Command => Some(DefinitionKind::Command),
        BoardElementKind::Event => Some(DefinitionKind::Event),
        BoardElementKind::ReadModel => Some(DefinitionKind::ReadModel),
        BoardElementKind::View => Some(DefinitionKind::View),
        BoardElementKind::Automation
        | BoardElementKind::ExternalEvent
        | BoardElementKind::Other => None,
    }
}

fn unknown_board_automation_reference(
    document: &EventModelDocument,
) -> Option<UnknownBoardElementReference> {
    document
        .board_slices
        .iter()
        .flat_map(|board_slice| board_slice.elements.iter())
        .filter(|element| element.kind == BoardElementKind::Automation)
        .filter_map(|element| element.name.as_ref().map(|name| (element, name)))
        .find(|(_element, name)| {
            !document
                .slice_definitions
                .iter()
                .any(|slice| slice.slice_type == SliceType::Automation && &slice.name == *name)
        })
        .map(|(element, name)| UnknownBoardElementReference {
            element_id: element.id.clone(),
            kind: element.kind,
            name: name.clone(),
        })
}

fn external_event_modeled_as_automation(document: &EventModelDocument) -> Option<DefinitionName> {
    document
        .board_slices
        .iter()
        .flat_map(|board_slice| board_slice.elements.iter())
        .filter(|element| element.kind == BoardElementKind::Automation)
        .map(|element| element.name.as_ref().unwrap_or(&element.id))
        .find(|candidate| declared_external_event_name(document, candidate))
        .cloned()
}

fn undeclared_unnamed_board_automation(document: &EventModelDocument) -> Option<DefinitionName> {
    document
        .board_slices
        .iter()
        .flat_map(|board_slice| board_slice.elements.iter())
        .filter(|element| element.kind == BoardElementKind::Automation && element.name.is_none())
        .find(|element| {
            !document
                .slice_definitions
                .iter()
                .any(|slice| slice.slice_type == SliceType::Automation && slice.name == element.id)
        })
        .map(|element| element.id.clone())
}

fn unknown_board_external_event_reference(
    document: &EventModelDocument,
) -> Option<UnknownBoardElementReference> {
    document
        .board_slices
        .iter()
        .flat_map(|board_slice| board_slice.elements.iter())
        .filter(|element| element.kind == BoardElementKind::ExternalEvent)
        .filter_map(|element| element.name.as_ref().map(|name| (element, name)))
        .find(|(_element, name)| {
            !document.slice_definitions.iter().any(|slice| {
                slice.slice_type == SliceType::Translation && slice.external_triggers.contains(name)
            })
        })
        .map(|(element, name)| UnknownBoardElementReference {
            element_id: element.id.clone(),
            kind: element.kind,
            name: name.clone(),
        })
}

fn undeclared_external_event_bridge(document: &EventModelDocument) -> Option<DefinitionName> {
    document
        .board_slices
        .iter()
        .find_map(|board_slice| undeclared_external_event_bridge_in_slice(document, board_slice))
}

fn undeclared_external_event_bridge_in_slice(
    document: &EventModelDocument,
    board_slice: &BoardSliceGraph,
) -> Option<DefinitionName> {
    board_slice
        .elements
        .iter()
        .filter(|element| element.kind == BoardElementKind::ExternalEvent)
        .filter(|element| {
            !declared_external_event_name(document, element.name.as_ref().unwrap_or(&element.id))
        })
        .find(|element| board_element_bridges_read_model_to_command(board_slice, element))
        .map(|element| element.id.clone())
}

fn invalid_board_connection(document: &EventModelDocument) -> Option<InvalidBoardConnection> {
    document.board_slices.iter().find_map(|board_slice| {
        board_slice.connections.iter().find_map(|connection| {
            let from_kind = board_element_kind_by_id(board_slice, &connection.from)?;
            let to_kind = board_element_kind_by_id(board_slice, &connection.to)?;
            (!board_connection_kind_is_causal(from_kind, to_kind)).then(|| InvalidBoardConnection {
                from: connection.from.clone(),
                from_kind,
                to: connection.to.clone(),
                to_kind,
            })
        })
    })
}

fn board_connection_kind_is_causal(from: BoardElementKind, to: BoardElementKind) -> bool {
    matches!(
        (from, to),
        (BoardElementKind::Automation, BoardElementKind::Command)
            | (BoardElementKind::Command, BoardElementKind::Event)
            | (BoardElementKind::Event, BoardElementKind::Automation)
            | (BoardElementKind::Event, BoardElementKind::ReadModel)
            | (BoardElementKind::ExternalEvent, BoardElementKind::Command)
            | (BoardElementKind::ReadModel, BoardElementKind::Automation)
            | (BoardElementKind::ReadModel, BoardElementKind::View)
            | (BoardElementKind::View, BoardElementKind::Command)
    )
}

fn command_board_element_without_incoming_trigger(
    document: &EventModelDocument,
) -> Option<DefinitionName> {
    document.board_slices.iter().find_map(|board_slice| {
        board_slice
            .elements
            .iter()
            .filter(|element| element.kind == BoardElementKind::Command)
            .find(|element| {
                !command_board_element_has_incoming_trigger(document, board_slice, element)
            })
            .map(|element| element.id.clone())
    })
}

fn command_board_element_has_incoming_trigger(
    document: &EventModelDocument,
    board_slice: &BoardSliceGraph,
    element: &BoardElement,
) -> bool {
    board_slice_has_explicit_workflow_trigger(document, board_slice)
        || board_slice.connections.iter().any(|connection| {
            connection.to == element.id
                && board_element_kind_by_id(board_slice, &connection.from)
                    .is_some_and(board_element_kind_triggers_command)
        })
}

fn board_slice_has_explicit_workflow_trigger(
    document: &EventModelDocument,
    board_slice: &BoardSliceGraph,
) -> bool {
    document
        .workflow_steps
        .iter()
        .any(|step| step.has_trigger() && board_slice.has_slug(step.slice()))
}

fn board_element_kind_triggers_command(kind: BoardElementKind) -> bool {
    matches!(
        kind,
        BoardElementKind::Automation | BoardElementKind::ExternalEvent | BoardElementKind::View
    )
}

fn board_element_bridges_read_model_to_command(
    board_slice: &BoardSliceGraph,
    element: &BoardElement,
) -> bool {
    let has_read_model_input = board_slice.connections.iter().any(|connection| {
        connection.to == element.id
            && board_element_kind_by_id(board_slice, &connection.from)
                == Some(BoardElementKind::ReadModel)
    });
    let has_command_output = board_slice.connections.iter().any(|connection| {
        connection.from == element.id
            && board_element_kind_by_id(board_slice, &connection.to)
                == Some(BoardElementKind::Command)
    });
    has_read_model_input && has_command_output
}

fn board_element_kind_by_id(
    board_slice: &BoardSliceGraph,
    element_id: &DefinitionName,
) -> Option<BoardElementKind> {
    board_element_by_id(board_slice, element_id).map(|element| element.kind)
}

fn board_element_by_id<'a>(
    board_slice: &'a BoardSliceGraph,
    element_id: &DefinitionName,
) -> Option<&'a BoardElement> {
    board_slice
        .elements
        .iter()
        .find(|element| &element.id == element_id)
}

fn command_event_board_connection_without_producer(
    document: &EventModelDocument,
) -> Option<CommandEventBoardConnectionIssue> {
    document.board_slices.iter().find_map(|board_slice| {
        board_slice.connections.iter().find_map(|connection| {
            let command_element = board_element_by_id(board_slice, &connection.from)?;
            let event_element = board_element_by_id(board_slice, &connection.to)?;
            (command_element.kind == BoardElementKind::Command
                && event_element.kind == BoardElementKind::Event)
                .then(|| {
                    let command = board_element_definition_name(command_element);
                    let event = board_element_definition_name(event_element);
                    (!command_produces_event(document, command, event)).then(|| {
                        CommandEventBoardConnectionIssue {
                            command: command.clone(),
                            event: event.clone(),
                        }
                    })
                })
                .flatten()
        })
    })
}

fn board_element_definition_name(element: &BoardElement) -> &DefinitionName {
    element.name.as_ref().unwrap_or(&element.id)
}

fn command_produces_event(
    document: &EventModelDocument,
    command_name: &DefinitionName,
    event_name: &DefinitionName,
) -> bool {
    command_definition(document, command_name)
        .is_some_and(|command| command.produces.contains(event_name))
}

fn event_read_model_board_connection_without_projection(
    document: &EventModelDocument,
) -> Option<EventReadModelBoardConnectionIssue> {
    document.board_slices.iter().find_map(|board_slice| {
        board_slice.connections.iter().find_map(|connection| {
            let event_element = board_element_by_id(board_slice, &connection.from)?;
            let read_model_element = board_element_by_id(board_slice, &connection.to)?;
            (event_element.kind == BoardElementKind::Event
                && read_model_element.kind == BoardElementKind::ReadModel)
                .then(|| {
                    let event = board_element_definition_name(event_element);
                    let read_model = board_element_definition_name(read_model_element);
                    (!read_model_projects_event(document, read_model, event)).then(|| {
                        EventReadModelBoardConnectionIssue {
                            event: event.clone(),
                            read_model: read_model.clone(),
                        }
                    })
                })
                .flatten()
        })
    })
}

fn read_model_projects_event(
    document: &EventModelDocument,
    read_model_name: &DefinitionName,
    event_name: &DefinitionName,
) -> bool {
    read_model_definition(document, read_model_name).is_some_and(|read_model| {
        read_model
            .fields
            .iter()
            .any(|field| read_model_field_sources_event(field, event_name))
    })
}

fn read_model_field_sources_event(field: &ReadModelField, event_name: &DefinitionName) -> bool {
    matches!(
        &field.source,
        ReadModelFieldSource::EventAttribute(source_event, _) if source_event == event_name
    )
}

fn read_model_definition<'a>(
    document: &'a EventModelDocument,
    read_model_name: &DefinitionName,
) -> Option<&'a ReadModelDefinition> {
    document
        .read_model_definitions
        .iter()
        .find(|read_model| &read_model.name == read_model_name)
}

fn view_command_board_connection_without_control(
    document: &EventModelDocument,
) -> Option<ViewCommandBoardConnectionIssue> {
    document.board_slices.iter().find_map(|board_slice| {
        board_slice.connections.iter().find_map(|connection| {
            let view_element = board_element_by_id(board_slice, &connection.from)?;
            let command_element = board_element_by_id(board_slice, &connection.to)?;
            (view_element.kind == BoardElementKind::View
                && command_element.kind == BoardElementKind::Command)
                .then(|| {
                    let view = board_element_definition_name(view_element);
                    let command = board_element_definition_name(command_element);
                    (!view_invokes_board_command(document, view, command)).then(|| {
                        ViewCommandBoardConnectionIssue {
                            view: view.clone(),
                            command: command.clone(),
                        }
                    })
                })
                .flatten()
        })
    })
}

fn view_invokes_board_command(
    document: &EventModelDocument,
    view_name: &DefinitionName,
    command_name: &DefinitionName,
) -> bool {
    view_definition(document, view_name)
        .is_some_and(|view| view_invokes_command(view, command_name))
}

fn read_model_board_element_feeding_view_without_update(
    document: &EventModelDocument,
) -> Option<DefinitionName> {
    document.board_slices.iter().find_map(|board_slice| {
        board_slice
            .elements
            .iter()
            .filter(|element| element.kind == BoardElementKind::ReadModel)
            .find(|element| {
                read_model_board_element_feeds_view(board_slice, element)
                    && !read_model_board_element_has_incoming_update(board_slice, element)
            })
            .map(|element| element.id.clone())
    })
}

fn read_model_board_element_feeds_view(
    board_slice: &BoardSliceGraph,
    element: &BoardElement,
) -> bool {
    board_slice.connections.iter().any(|connection| {
        connection.from == element.id
            && board_element_kind_by_id(board_slice, &connection.to) == Some(BoardElementKind::View)
    })
}

fn read_model_board_element_has_incoming_update(
    board_slice: &BoardSliceGraph,
    element: &BoardElement,
) -> bool {
    board_slice.connections.iter().any(|connection| {
        connection.to == element.id
            && board_element_kind_by_id(board_slice, &connection.from)
                == Some(BoardElementKind::Event)
    })
}

fn declared_external_event_name(document: &EventModelDocument, name: &DefinitionName) -> bool {
    document.slice_definitions.iter().any(|slice| {
        slice.slice_type == SliceType::Translation && slice.external_triggers.contains(name)
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MissingTranslationPayloadVariantScenario {
    slice_name: DefinitionName,
    variant: DefinitionName,
}

fn missing_translation_payload_variant_scenario(
    slice: &SliceDefinition,
) -> Option<MissingTranslationPayloadVariantScenario> {
    slice
        .external_payload_variants
        .iter()
        .find(|payload_variant| !slice_has_payload_variant_scenario(slice, payload_variant))
        .map(|payload_variant| MissingTranslationPayloadVariantScenario {
            slice_name: slice.name.clone(),
            variant: payload_variant.variant.clone(),
        })
}

fn slice_has_payload_variant_scenario(
    slice: &SliceDefinition,
    payload_variant: &ExternalPayloadVariant,
) -> bool {
    slice
        .scenarios
        .iter()
        .any(|scenario| scenario_mentions_definition(scenario, &payload_variant.variant))
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

fn validate_automation_slice_trigger_scenarios(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .slice_definitions
        .iter()
        .filter(|slice| slice.slice_type == SliceType::Automation)
        .find_map(missing_automation_trigger_scenario)
        .map_or(Ok(()), |missing| {
            Err(validation_issue(format!(
                "automation slice '{}' must include a scenario for trigger event '{}'",
                missing.slice_name, missing.trigger_event
            )))
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MissingAutomationTriggerScenario {
    slice_name: DefinitionName,
    trigger_event: DefinitionName,
}

fn missing_automation_trigger_scenario(
    slice: &SliceDefinition,
) -> Option<MissingAutomationTriggerScenario> {
    match &slice.automation_trigger {
        AutomationTrigger::DeclaresTriggers(trigger_events) => trigger_events
            .iter()
            .find(|trigger_event| !slice_has_trigger_scenario(slice, trigger_event))
            .map(|trigger_event| MissingAutomationTriggerScenario {
                slice_name: slice.name.clone(),
                trigger_event: trigger_event.clone(),
            }),
        AutomationTrigger::MissingTrigger | AutomationTrigger::NotAutomation => None,
    }
}

fn slice_has_trigger_scenario(slice: &SliceDefinition, trigger_event: &DefinitionName) -> bool {
    slice
        .scenarios
        .iter()
        .any(|scenario| scenario_mentions_trigger(scenario, trigger_event))
}

fn scenario_mentions_trigger(scenario: &SliceScenario, trigger_event: &DefinitionName) -> bool {
    scenario_mentions_definition(scenario, trigger_event)
}

fn scenario_mentions_definition(
    scenario: &SliceScenario,
    definition_name: &DefinitionName,
) -> bool {
    scenario
        .scenario_step_references
        .iter()
        .any(|reference| reference.as_ref().contains(definition_name.as_ref()))
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

fn validate_external_system_navigation_targets(
    document: &EventModelDocument,
) -> Result<(), ValidationIssue> {
    document
        .view_definitions
        .iter()
        .flat_map(|view| view.controls.iter())
        .find(|control| {
            control.navigation_type == NavigationType::ExternalSystem
                && (control.external_system.is_none() || control.payload_contract.is_none())
        })
        .map_or(Ok(()), |_| {
            Err(validation_issue(
                "external system navigation must name the external system and returned payload contract",
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
