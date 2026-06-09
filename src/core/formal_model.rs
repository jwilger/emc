// Copyright 2026 John Wilger

use std::cmp::Ordering;

use crate::core::emit::{
    lean_model_command_input_source_kind, quint_model_command_input_source_kind,
};
use crate::core::formal_slice_facts::ScenarioKind;
use crate::core::types::{
    BitEncodingSemantics, CommandErrorName, CommandErrorRecoveryKind,
    CommandInputSourceDescription, CommandInputSourceKind, CommandName, ContractKindName,
    CoveredDefinitionName, DataFlowSource, DataFlowSourceKind, DataFlowTarget, DatumName,
    EventAttributeName, EventAttributeSourceField, EventAttributeSourceName, EventName,
    LeanModuleName, OutcomeLabelName, ScenarioName, ScenarioStepText, SliceSlug, SourceChainHop,
    StreamName, TransformationSemantics, WorkflowSlug,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelWorkflow {
    workflow: WorkflowSlug,
}

impl FormalModelWorkflow {
    pub(crate) fn new(workflow: WorkflowSlug) -> Self {
        Self { workflow }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelSlice {
    workflow: WorkflowSlug,
    slice: SliceSlug,
}

impl FormalModelSlice {
    pub(crate) fn new(workflow: WorkflowSlug, slice: SliceSlug) -> Self {
        Self { workflow, slice }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelSliceModule {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    formal_module: LeanModuleName,
}

impl FormalModelSliceModule {
    pub(crate) fn new(
        workflow: WorkflowSlug,
        slice: SliceSlug,
        formal_module: LeanModuleName,
    ) -> Self {
        Self {
            workflow,
            slice,
            formal_module,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelScenario {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    scenario_kind: ScenarioKind,
    scenario: ScenarioName,
}

impl FormalModelScenario {
    pub(crate) fn new(
        workflow: WorkflowSlug,
        slice: SliceSlug,
        scenario_kind: ScenarioKind,
        scenario: ScenarioName,
    ) -> Self {
        Self {
            workflow,
            slice,
            scenario_kind,
            scenario,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelScenarioDefinition {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    scenario_kind: ScenarioKind,
    scenario: ScenarioName,
    given: ScenarioStepText,
    when: ScenarioStepText,
    then: ScenarioStepText,
    read_streams: Vec<StreamName>,
    written_streams: Vec<StreamName>,
    contract_kind: Option<ContractKindName>,
    covered_definition: Option<CoveredDefinitionName>,
    error_references: Vec<CommandErrorName>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelScenarioDefinitionFields {
    pub(crate) workflow: WorkflowSlug,
    pub(crate) slice: SliceSlug,
    pub(crate) scenario_kind: ScenarioKind,
    pub(crate) scenario: ScenarioName,
    pub(crate) given: ScenarioStepText,
    pub(crate) when: ScenarioStepText,
    pub(crate) then: ScenarioStepText,
    pub(crate) read_streams: Vec<StreamName>,
    pub(crate) written_streams: Vec<StreamName>,
    pub(crate) contract_kind: Option<ContractKindName>,
    pub(crate) covered_definition: Option<CoveredDefinitionName>,
    pub(crate) error_references: Vec<CommandErrorName>,
}

impl FormalModelScenarioDefinition {
    pub(crate) fn new(fields: FormalModelScenarioDefinitionFields) -> Self {
        Self {
            workflow: fields.workflow,
            slice: fields.slice,
            scenario_kind: fields.scenario_kind,
            scenario: fields.scenario,
            given: fields.given,
            when: fields.when,
            then: fields.then,
            read_streams: fields.read_streams,
            written_streams: fields.written_streams,
            contract_kind: fields.contract_kind,
            covered_definition: fields.covered_definition,
            error_references: fields.error_references,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelDataFlow {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    datum: DatumName,
    source_kind: DataFlowSourceKind,
    source: DataFlowSource,
    transformation: TransformationSemantics,
    target: DataFlowTarget,
    bit_encoding: BitEncodingSemantics,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelDataFlowFields {
    pub(crate) workflow: WorkflowSlug,
    pub(crate) slice: SliceSlug,
    pub(crate) datum: DatumName,
    pub(crate) source_kind: DataFlowSourceKind,
    pub(crate) source: DataFlowSource,
    pub(crate) transformation: TransformationSemantics,
    pub(crate) target: DataFlowTarget,
    pub(crate) bit_encoding: BitEncodingSemantics,
}

impl FormalModelDataFlow {
    pub(crate) fn new(fields: FormalModelDataFlowFields) -> Self {
        Self {
            workflow: fields.workflow,
            slice: fields.slice,
            datum: fields.datum,
            source_kind: fields.source_kind,
            source: fields.source,
            transformation: fields.transformation,
            target: fields.target,
            bit_encoding: fields.bit_encoding,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelOutcome {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    outcome: OutcomeLabelName,
    events: Vec<EventName>,
    externally_relevant: bool,
}

impl FormalModelOutcome {
    pub(crate) fn new(
        workflow: WorkflowSlug,
        slice: SliceSlug,
        outcome: OutcomeLabelName,
        events: Vec<EventName>,
        externally_relevant: bool,
    ) -> Self {
        Self {
            workflow,
            slice,
            outcome,
            events,
            externally_relevant,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelCommandError {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    command: CommandName,
    error: CommandErrorName,
    scenario: ScenarioName,
    recovery: CommandErrorRecoveryKind,
}

impl FormalModelCommandError {
    pub(crate) fn new(
        workflow: WorkflowSlug,
        slice: SliceSlug,
        command: CommandName,
        error: CommandErrorName,
        scenario: ScenarioName,
        recovery: CommandErrorRecoveryKind,
    ) -> Self {
        Self {
            workflow,
            slice,
            command,
            error,
            scenario,
            recovery,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelCommand {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    command: CommandName,
}

impl FormalModelCommand {
    pub(crate) fn new(workflow: WorkflowSlug, slice: SliceSlug, command: CommandName) -> Self {
        Self {
            workflow,
            slice,
            command,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelCommandInput {
    workflow: WorkflowSlug,
    slice: SliceSlug,
    command: CommandName,
    input: DatumName,
    source_kind: CommandInputSourceKind,
    source_description: CommandInputSourceDescription,
    provenance_chain: Vec<SourceChainHop>,
    event_stream_source_event: Option<EventName>,
    event_stream_source_attribute: Option<EventAttributeName>,
    external_payload_source_name: Option<EventAttributeSourceName>,
    external_payload_source_field: Option<EventAttributeSourceField>,
    generated_source_name: Option<EventAttributeSourceName>,
    generated_source_field: Option<EventAttributeSourceField>,
    session_source_name: Option<EventAttributeSourceName>,
    session_source_field: Option<EventAttributeSourceField>,
    invocation_argument_source_name: Option<EventAttributeSourceName>,
    invocation_argument_source_field: Option<EventAttributeSourceField>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalModelCommandInputFields {
    pub(crate) workflow: WorkflowSlug,
    pub(crate) slice: SliceSlug,
    pub(crate) command: CommandName,
    pub(crate) input: DatumName,
    pub(crate) source_kind: CommandInputSourceKind,
    pub(crate) source_description: CommandInputSourceDescription,
    pub(crate) provenance_chain: Vec<SourceChainHop>,
    pub(crate) event_stream_source_event: Option<EventName>,
    pub(crate) event_stream_source_attribute: Option<EventAttributeName>,
    pub(crate) external_payload_source_name: Option<EventAttributeSourceName>,
    pub(crate) external_payload_source_field: Option<EventAttributeSourceField>,
    pub(crate) generated_source_name: Option<EventAttributeSourceName>,
    pub(crate) generated_source_field: Option<EventAttributeSourceField>,
    pub(crate) session_source_name: Option<EventAttributeSourceName>,
    pub(crate) session_source_field: Option<EventAttributeSourceField>,
    pub(crate) invocation_argument_source_name: Option<EventAttributeSourceName>,
    pub(crate) invocation_argument_source_field: Option<EventAttributeSourceField>,
}

impl FormalModelCommandInput {
    pub(crate) fn new(fields: FormalModelCommandInputFields) -> Self {
        Self {
            workflow: fields.workflow,
            slice: fields.slice,
            command: fields.command,
            input: fields.input,
            source_kind: fields.source_kind,
            source_description: fields.source_description,
            provenance_chain: fields.provenance_chain,
            event_stream_source_event: fields.event_stream_source_event,
            event_stream_source_attribute: fields.event_stream_source_attribute,
            external_payload_source_name: fields.external_payload_source_name,
            external_payload_source_field: fields.external_payload_source_field,
            generated_source_name: fields.generated_source_name,
            generated_source_field: fields.generated_source_field,
            session_source_name: fields.session_source_name,
            session_source_field: fields.session_source_field,
            invocation_argument_source_name: fields.invocation_argument_source_name,
            invocation_argument_source_field: fields.invocation_argument_source_field,
        }
    }
}

pub(crate) fn lean_slice_record_structures() -> &'static str {
    "structure ModelSlice where\n  workflow : String\n  slice : String\n\nstructure ModelSliceModule where\n  workflow : String\n  slice : String\n  formalModule : String"
}

pub(crate) fn lean_workflow_record_structure() -> &'static str {
    "structure ModelWorkflow where\n  workflow : String"
}

pub(crate) fn lean_scenario_record_structures() -> &'static str {
    "structure ModelScenario where\n  workflow : String\n  slice : String\n  scenarioKind : String\n  scenario : String\n\nstructure ModelScenarioDefinition where\n  workflow : String\n  slice : String\n  scenarioKind : String\n  scenario : String\n  given : String\n  when : String\n  thenStep : String\n  readStreams : List String\n  writtenStreams : List String\n  contractKind : String\n  coveredDefinition : String\n  errorReferences : List String"
}

pub(crate) fn lean_data_flow_record_structure() -> &'static str {
    "inductive ModelDataFlowSourceKind where\n  | original\n  | modeledTarget\nderiving BEq, DecidableEq, Repr\n\nstructure ModelDataFlow where\n  workflow : String\n  slice : String\n  datum : String\n  sourceKind : ModelDataFlowSourceKind\n  source : String\n  transformation : String\n  target : String\n  bitEncoding : String"
}

pub(crate) fn lean_outcome_record_structure() -> &'static str {
    "structure ModelOutcome where\n  workflow : String\n  slice : String\n  outcome : String\n  events : List String\n  externallyRelevant : Bool"
}

pub(crate) fn lean_command_error_record_structure() -> &'static str {
    "structure ModelCommandError where\n  workflow : String\n  slice : String\n  command : String\n  error : String\n  scenario : String\n  recovery : String"
}

pub(crate) fn lean_command_record_structure() -> &'static str {
    "structure ModelCommand where\n  workflow : String\n  slice : String\n  command : String"
}

pub(crate) fn lean_command_input_record_structure() -> &'static str {
    "inductive ModelCommandInputSourceKind where\n  | actor\n  | session\n  | generated\n  | externalPayload\n  | eventStreamState\n  | invocationArgument\nderiving BEq, DecidableEq, Repr\n\nstructure ModelCommandInput where\n  workflow : String\n  slice : String\n  command : String\n  input : String\n  sourceKind : ModelCommandInputSourceKind\n  sourceDescription : String\n  provenanceChain : List String\n  eventStreamSourceEvent : String\n  eventStreamSourceAttribute : String\n  externalPayloadSourceName : String\n  externalPayloadSourceField : String\n  generatedSourceName : String\n  generatedSourceField : String\n  sessionSourceName : String\n  sessionSourceField : String\n  invocationArgumentSourceName : String\n  invocationArgumentSourceField : String"
}

pub(crate) fn lean_read_model_record_structures() -> &'static str {
    "structure ModelReadModel where\n  workflow : String\n  slice : String\n  readModel : String\n\nstructure ModelReadModelDefinition where\n  workflow : String\n  slice : String\n  readModel : String\n  transitive : Bool\n  relationshipFields : List String\n  transitiveRule : String\n  exampleScenarioName : String\n\nstructure ModelReadModelField where\n  workflow : String\n  slice : String\n  readModel : String\n  field : String\n  sourceKind : String\n  sourceEvent : String\n  sourceAttribute : String\n  derivationRule : String\n  derivationSourceFields : List String\n  absenceEvent : String\n  derivationScenarioName : String\n  absenceScenarioName : String\n  provenance : String"
}

pub(crate) fn lean_view_record_structures() -> &'static str {
    "structure ModelView where\n  workflow : String\n  slice : String\n  view : String\n\nstructure ModelViewDefinition where\n  workflow : String\n  slice : String\n  view : String\n  readModels : List String\n  sketchTokens : List String\n  localStates : List String\n  filters : List String\n\nstructure ModelViewControl where\n  workflow : String\n  slice : String\n  view : String\n  control : String\n  command : String\n  input : String\n  inputSourceKind : ModelCommandInputSourceKind\n  inputSourceDescription : String\n  inputSketchToken : String\n  inputVisibleToActor : Bool\n  inputDecisionField : Bool\n  handledErrors : List String\n  recoveryBehavior : String\n  controlSketchToken : String\n  navigationType : String\n  navigationTarget : String\n  externalWorkflow : String\n  externalSystem : String\n  handoffContract : String\n\nstructure ModelViewField where\n  workflow : String\n  slice : String\n  view : String\n  field : String\n  sourceKind : String\n  sourceReadModel : String\n  sourceField : String\n  provenance : String\n  bitEncoding : String"
}

pub(crate) fn lean_board_record_structures() -> &'static str {
    "structure ModelBoardElement where\n  workflow : String\n  slice : String\n  element : String\n  kind : String\n  lane : String\n  declaredName : String\n  mainPath : Bool\n\nstructure ModelBoardConnection where\n  workflow : String\n  slice : String\n  source : String\n  sourceKind : String\n  target : String\n  targetKind : String"
}

pub(crate) fn lean_automation_record_structures() -> &'static str {
    "structure ModelAutomation where\n  workflow : String\n  slice : String\n  automation : String\n\nstructure ModelAutomationDefinition where\n  workflow : String\n  slice : String\n  automation : String\n  trigger : String\n  command : String\n  handledErrors : List String\n  reaction : String"
}

pub(crate) fn lean_translation_record_structures() -> &'static str {
    "structure ModelTranslation where\n  workflow : String\n  slice : String\n  translation : String\n\nstructure ModelTranslationDefinition where\n  workflow : String\n  slice : String\n  translation : String\n  externalEvent : String\n  payloadContract : String\n  command : String"
}

pub(crate) fn lean_external_payload_record_structures() -> &'static str {
    "structure ModelExternalPayload where\n  workflow : String\n  slice : String\n  externalPayload : String\n\nstructure ModelExternalPayloadField where\n  workflow : String\n  slice : String\n  externalPayload : String\n  field : String\n  provenance : String\n  bitEncoding : String"
}

pub(crate) fn lean_event_inventory_record_structures() -> &'static str {
    "structure ModelStream where\n  workflow : String\n  slice : String\n  stream : String\n\nstructure ModelEvent where\n  workflow : String\n  slice : String\n  event : String\n  stream : String\n\nstructure ModelEventAttribute where\n  workflow : String\n  slice : String\n  event : String\n  attributeName : String\n  sourceKind : String\n  sourceName : String\n  sourceField : String\n  generatedSourceKind : String\n  provenance : String"
}

pub(crate) fn lean_model_workflow_list(workflows: &[FormalModelWorkflow]) -> String {
    let mut workflows = workflows
        .iter()
        .map(|workflow| workflow.workflow.as_ref())
        .collect::<Vec<_>>();
    workflows.sort_unstable();
    render_list(
        workflows
            .into_iter()
            .map(|workflow| format!("{{ workflow := {} }}", quoted(workflow))),
    )
}

pub(crate) fn lean_model_slice_list(slices: &[FormalModelSlice]) -> String {
    render_list(sorted_slices(slices).into_iter().map(|slice| {
        format!(
            "{{ workflow := {}, slice := {} }}",
            quoted(slice.workflow.as_ref()),
            quoted(slice.slice.as_ref())
        )
    }))
}

pub(crate) fn lean_model_slice_module_list(modules: &[FormalModelSliceModule]) -> String {
    render_list(sorted_modules(modules).into_iter().map(|module| {
        format!(
            "{{ workflow := {}, slice := {}, formalModule := {} }}",
            quoted(module.workflow.as_ref()),
            quoted(module.slice.as_ref()),
            quoted(module.formal_module.as_ref())
        )
    }))
}

pub(crate) fn lean_model_scenario_list(scenarios: &[FormalModelScenario]) -> String {
    render_list(sorted_scenarios(scenarios).into_iter().map(|scenario| {
        format!(
            "{{ workflow := {}, slice := {}, scenarioKind := {}, scenario := {} }}",
            quoted(scenario.workflow.as_ref()),
            quoted(scenario.slice.as_ref()),
            quoted(scenario.scenario_kind.as_str()),
            quoted(scenario.scenario.as_ref())
        )
    }))
}

pub(crate) fn lean_model_scenario_definition_list(
    definitions: &[FormalModelScenarioDefinition],
) -> String {
    render_list(
        sorted_scenario_definitions(definitions)
            .into_iter()
            .map(|definition| {
                format!(
                    "{{ workflow := {}, slice := {}, scenarioKind := {}, scenario := {}, given := {}, when := {}, thenStep := {}, readStreams := {}, writtenStreams := {}, contractKind := {}, coveredDefinition := {}, errorReferences := {} }}",
                    quoted(definition.workflow.as_ref()),
                    quoted(definition.slice.as_ref()),
                    quoted(definition.scenario_kind.as_str()),
                    quoted(definition.scenario.as_ref()),
                    quoted(definition.given.as_ref()),
                    quoted(definition.when.as_ref()),
                    quoted(definition.then.as_ref()),
                    quoted_list(&definition.read_streams),
                    quoted_list(&definition.written_streams),
                    quoted(optional_text(&definition.contract_kind)),
                    quoted(optional_text(&definition.covered_definition)),
                    quoted_list(&definition.error_references)
                )
            }),
    )
}

pub(crate) fn lean_model_data_flow_list(data_flows: &[FormalModelDataFlow]) -> String {
    render_list(sorted_data_flows(data_flows).into_iter().map(|data_flow| {
        format!(
            "{{ workflow := {}, slice := {}, datum := {}, sourceKind := {}, source := {}, transformation := {}, target := {}, bitEncoding := {} }}",
            quoted(data_flow.workflow.as_ref()),
            quoted(data_flow.slice.as_ref()),
            quoted(data_flow.datum.as_ref()),
            lean_data_flow_source_kind(data_flow.source_kind),
            quoted(data_flow.source.as_ref()),
            quoted(data_flow.transformation.as_ref()),
            quoted(data_flow.target.as_ref()),
            quoted(data_flow.bit_encoding.as_ref())
        )
    }))
}

pub(crate) fn lean_model_outcome_list(outcomes: &[FormalModelOutcome]) -> String {
    render_list(sorted_outcomes(outcomes).into_iter().map(|outcome| {
        format!(
            "{{ workflow := {}, slice := {}, outcome := {}, events := {}, externallyRelevant := {} }}",
            quoted(outcome.workflow.as_ref()),
            quoted(outcome.slice.as_ref()),
            quoted(outcome.outcome.as_ref()),
            quoted_list(&outcome.events),
            outcome.externally_relevant
        )
    }))
}

pub(crate) fn lean_model_command_error_list(command_errors: &[FormalModelCommandError]) -> String {
    render_list(
        sorted_command_errors(command_errors)
            .into_iter()
            .map(|command_error| {
                format!(
                    "{{ workflow := {}, slice := {}, command := {}, error := {}, scenario := {}, recovery := {} }}",
                    quoted(command_error.workflow.as_ref()),
                    quoted(command_error.slice.as_ref()),
                    quoted(command_error.command.as_ref()),
                    quoted(command_error.error.as_ref()),
                    quoted(command_error.scenario.as_ref()),
                    quoted(command_error.recovery.as_ref())
                )
            }),
    )
}

pub(crate) fn lean_model_command_list(commands: &[FormalModelCommand]) -> String {
    render_list(sorted_commands(commands).into_iter().map(|command| {
        format!(
            "{{ workflow := {}, slice := {}, command := {} }}",
            quoted(command.workflow.as_ref()),
            quoted(command.slice.as_ref()),
            quoted(command.command.as_ref())
        )
    }))
}

pub(crate) fn lean_model_command_input_list(command_inputs: &[FormalModelCommandInput]) -> String {
    render_list(
        sorted_command_inputs(command_inputs)
            .into_iter()
            .map(|command_input| {
                format!(
                    "{{ workflow := {}, slice := {}, command := {}, input := {}, sourceKind := {}, sourceDescription := {}, provenanceChain := {}, eventStreamSourceEvent := {}, eventStreamSourceAttribute := {}, externalPayloadSourceName := {}, externalPayloadSourceField := {}, generatedSourceName := {}, generatedSourceField := {}, sessionSourceName := {}, sessionSourceField := {}, invocationArgumentSourceName := {}, invocationArgumentSourceField := {} }}",
                    quoted(command_input.workflow.as_ref()),
                    quoted(command_input.slice.as_ref()),
                    quoted(command_input.command.as_ref()),
                    quoted(command_input.input.as_ref()),
                    lean_model_command_input_source_kind(command_input.source_kind),
                    quoted(command_input.source_description.as_ref()),
                    quoted_list(&command_input.provenance_chain),
                    quoted(optional_text(&command_input.event_stream_source_event)),
                    quoted(optional_text(&command_input.event_stream_source_attribute)),
                    quoted(optional_text(&command_input.external_payload_source_name)),
                    quoted(optional_text(&command_input.external_payload_source_field)),
                    quoted(optional_text(&command_input.generated_source_name)),
                    quoted(optional_text(&command_input.generated_source_field)),
                    quoted(optional_text(&command_input.session_source_name)),
                    quoted(optional_text(&command_input.session_source_field)),
                    quoted(optional_text(&command_input.invocation_argument_source_name)),
                    quoted(optional_text(&command_input.invocation_argument_source_field))
                )
            }),
    )
}

pub(crate) fn quint_model_slice_list(slices: &[FormalModelSlice]) -> String {
    render_spaced_list(sorted_slices(slices).into_iter().map(|slice| {
        format!(
            "{{ workflow: {}, slice: {} }}",
            quoted(slice.workflow.as_ref()),
            quoted(slice.slice.as_ref())
        )
    }))
}

pub(crate) fn quint_model_slice_module_list(modules: &[FormalModelSliceModule]) -> String {
    render_spaced_list(sorted_modules(modules).into_iter().map(|module| {
        format!(
            "{{ workflow: {}, slice: {}, formalModule: {} }}",
            quoted(module.workflow.as_ref()),
            quoted(module.slice.as_ref()),
            quoted(module.formal_module.as_ref())
        )
    }))
}

pub(crate) fn quint_model_scenario_list(scenarios: &[FormalModelScenario]) -> String {
    render_list(sorted_scenarios(scenarios).into_iter().map(|scenario| {
        format!(
            "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {} }}",
            quoted(scenario.workflow.as_ref()),
            quoted(scenario.slice.as_ref()),
            quoted(scenario.scenario_kind.as_str()),
            quoted(scenario.scenario.as_ref())
        )
    }))
}

pub(crate) fn quint_model_scenario_definition_list(
    definitions: &[FormalModelScenarioDefinition],
) -> String {
    render_list(
        sorted_scenario_definitions(definitions)
            .into_iter()
            .map(|definition| {
                format!(
                    "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {}, given: {}, when: {}, then: {}, readStreams: {}, writtenStreams: {}, contractKind: {}, coveredDefinition: {}, errorReferences: {} }}",
                    quoted(definition.workflow.as_ref()),
                    quoted(definition.slice.as_ref()),
                    quoted(definition.scenario_kind.as_str()),
                    quoted(definition.scenario.as_ref()),
                    quoted(definition.given.as_ref()),
                    quoted(definition.when.as_ref()),
                    quoted(definition.then.as_ref()),
                    quoted_list(&definition.read_streams),
                    quoted_list(&definition.written_streams),
                    quoted(optional_text(&definition.contract_kind)),
                    quoted(optional_text(&definition.covered_definition)),
                    quoted_list(&definition.error_references)
                )
            }),
    )
}

pub(crate) fn quint_model_data_flow_list(data_flows: &[FormalModelDataFlow]) -> String {
    render_list(sorted_data_flows(data_flows).into_iter().map(|data_flow| {
        format!(
            "{{ workflow: {}, slice: {}, datum: {}, sourceKind: {}, source: {}, transformation: {}, target: {}, bitEncoding: {} }}",
            quoted(data_flow.workflow.as_ref()),
            quoted(data_flow.slice.as_ref()),
            quoted(data_flow.datum.as_ref()),
            quint_data_flow_source_kind(data_flow.source_kind),
            quoted(data_flow.source.as_ref()),
            quoted(data_flow.transformation.as_ref()),
            quoted(data_flow.target.as_ref()),
            quoted(data_flow.bit_encoding.as_ref())
        )
    }))
}

pub(crate) fn quint_model_outcome_list(outcomes: &[FormalModelOutcome]) -> String {
    render_list(sorted_outcomes(outcomes).into_iter().map(|outcome| {
        format!(
            "{{ workflow: {}, slice: {}, outcome: {}, events: {}, externallyRelevant: {} }}",
            quoted(outcome.workflow.as_ref()),
            quoted(outcome.slice.as_ref()),
            quoted(outcome.outcome.as_ref()),
            quoted_list(&outcome.events),
            outcome.externally_relevant
        )
    }))
}

pub(crate) fn quint_model_command_error_list(command_errors: &[FormalModelCommandError]) -> String {
    render_list(
        sorted_command_errors(command_errors)
            .into_iter()
            .map(|command_error| {
                format!(
                    "{{ workflow: {}, slice: {}, command: {}, error: {}, scenario: {}, recovery: {} }}",
                    quoted(command_error.workflow.as_ref()),
                    quoted(command_error.slice.as_ref()),
                    quoted(command_error.command.as_ref()),
                    quoted(command_error.error.as_ref()),
                    quoted(command_error.scenario.as_ref()),
                    quoted(command_error.recovery.as_ref())
                )
            }),
    )
}

pub(crate) fn quint_model_command_list(commands: &[FormalModelCommand]) -> String {
    render_list(sorted_commands(commands).into_iter().map(|command| {
        format!(
            "{{ workflow: {}, slice: {}, command: {} }}",
            quoted(command.workflow.as_ref()),
            quoted(command.slice.as_ref()),
            quoted(command.command.as_ref())
        )
    }))
}

pub(crate) fn quint_model_command_input_list(command_inputs: &[FormalModelCommandInput]) -> String {
    render_list(
        sorted_command_inputs(command_inputs)
            .into_iter()
            .map(|command_input| {
                format!(
                    "{{ workflow: {}, slice: {}, command: {}, input: {}, sourceKind: {}, sourceDescription: {}, provenanceChain: {}, eventStreamSourceEvent: {}, eventStreamSourceAttribute: {}, externalPayloadSourceName: {}, externalPayloadSourceField: {}, generatedSourceName: {}, generatedSourceField: {}, sessionSourceName: {}, sessionSourceField: {}, invocationArgumentSourceName: {}, invocationArgumentSourceField: {} }}",
                    quoted(command_input.workflow.as_ref()),
                    quoted(command_input.slice.as_ref()),
                    quoted(command_input.command.as_ref()),
                    quoted(command_input.input.as_ref()),
                    quint_model_command_input_source_kind(command_input.source_kind),
                    quoted(command_input.source_description.as_ref()),
                    quoted_list(&command_input.provenance_chain),
                    quoted(optional_text(&command_input.event_stream_source_event)),
                    quoted(optional_text(&command_input.event_stream_source_attribute)),
                    quoted(optional_text(&command_input.external_payload_source_name)),
                    quoted(optional_text(&command_input.external_payload_source_field)),
                    quoted(optional_text(&command_input.generated_source_name)),
                    quoted(optional_text(&command_input.generated_source_field)),
                    quoted(optional_text(&command_input.session_source_name)),
                    quoted(optional_text(&command_input.session_source_field)),
                    quoted(optional_text(&command_input.invocation_argument_source_name)),
                    quoted(optional_text(&command_input.invocation_argument_source_field))
                )
            }),
    )
}

fn lean_data_flow_source_kind(source_kind: DataFlowSourceKind) -> &'static str {
    match source_kind {
        DataFlowSourceKind::Original => "ModelDataFlowSourceKind.original",
        DataFlowSourceKind::ModeledTarget => "ModelDataFlowSourceKind.modeledTarget",
    }
}

fn quint_data_flow_source_kind(source_kind: DataFlowSourceKind) -> &'static str {
    match source_kind {
        DataFlowSourceKind::Original => "Original",
        DataFlowSourceKind::ModeledTarget => "ModeledTarget",
    }
}

fn sorted_slices(slices: &[FormalModelSlice]) -> Vec<&FormalModelSlice> {
    let mut slices = slices.iter().collect::<Vec<_>>();
    slices.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
    });
    slices
}

fn sorted_modules(modules: &[FormalModelSliceModule]) -> Vec<&FormalModelSliceModule> {
    let mut modules = modules.iter().collect::<Vec<_>>();
    modules.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| {
                left.formal_module
                    .as_ref()
                    .cmp(right.formal_module.as_ref())
            })
    });
    modules
}

fn sorted_scenarios(scenarios: &[FormalModelScenario]) -> Vec<&FormalModelScenario> {
    let mut scenarios = scenarios.iter().collect::<Vec<_>>();
    scenarios.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| {
                left.scenario_kind
                    .as_str()
                    .cmp(right.scenario_kind.as_str())
            })
            .then_with(|| left.scenario.as_ref().cmp(right.scenario.as_ref()))
    });
    scenarios
}

fn sorted_scenario_definitions(
    definitions: &[FormalModelScenarioDefinition],
) -> Vec<&FormalModelScenarioDefinition> {
    let mut definitions = definitions.iter().collect::<Vec<_>>();
    definitions.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| {
                left.scenario_kind
                    .as_str()
                    .cmp(right.scenario_kind.as_str())
            })
            .then_with(|| left.scenario.as_ref().cmp(right.scenario.as_ref()))
            .then_with(|| left.given.as_ref().cmp(right.given.as_ref()))
            .then_with(|| left.when.as_ref().cmp(right.when.as_ref()))
            .then_with(|| left.then.as_ref().cmp(right.then.as_ref()))
            .then_with(|| compare_semantic_list(&left.read_streams, &right.read_streams))
            .then_with(|| compare_semantic_list(&left.written_streams, &right.written_streams))
            .then_with(|| {
                optional_text(&left.contract_kind).cmp(optional_text(&right.contract_kind))
            })
            .then_with(|| {
                optional_text(&left.covered_definition)
                    .cmp(optional_text(&right.covered_definition))
            })
            .then_with(|| compare_semantic_list(&left.error_references, &right.error_references))
    });
    definitions
}

fn sorted_data_flows(data_flows: &[FormalModelDataFlow]) -> Vec<&FormalModelDataFlow> {
    let mut data_flows = data_flows.iter().collect::<Vec<_>>();
    data_flows.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| left.datum.as_ref().cmp(right.datum.as_ref()))
            .then_with(|| left.source_kind.as_ref().cmp(right.source_kind.as_ref()))
            .then_with(|| left.source.as_ref().cmp(right.source.as_ref()))
            .then_with(|| {
                left.transformation
                    .as_ref()
                    .cmp(right.transformation.as_ref())
            })
            .then_with(|| left.target.as_ref().cmp(right.target.as_ref()))
            .then_with(|| left.bit_encoding.as_ref().cmp(right.bit_encoding.as_ref()))
    });
    data_flows
}

fn sorted_outcomes(outcomes: &[FormalModelOutcome]) -> Vec<&FormalModelOutcome> {
    let mut outcomes = outcomes.iter().collect::<Vec<_>>();
    outcomes.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| left.outcome.as_ref().cmp(right.outcome.as_ref()))
            .then_with(|| compare_semantic_list(&left.events, &right.events))
            .then_with(|| left.externally_relevant.cmp(&right.externally_relevant))
    });
    outcomes
}

fn sorted_command_errors(
    command_errors: &[FormalModelCommandError],
) -> Vec<&FormalModelCommandError> {
    let mut command_errors = command_errors.iter().collect::<Vec<_>>();
    command_errors.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| left.command.as_ref().cmp(right.command.as_ref()))
            .then_with(|| left.error.as_ref().cmp(right.error.as_ref()))
            .then_with(|| left.scenario.as_ref().cmp(right.scenario.as_ref()))
            .then_with(|| left.recovery.as_ref().cmp(right.recovery.as_ref()))
    });
    command_errors
}

fn sorted_commands(commands: &[FormalModelCommand]) -> Vec<&FormalModelCommand> {
    let mut commands = commands.iter().collect::<Vec<_>>();
    commands.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| left.command.as_ref().cmp(right.command.as_ref()))
    });
    commands
}

fn sorted_command_inputs(
    command_inputs: &[FormalModelCommandInput],
) -> Vec<&FormalModelCommandInput> {
    let mut command_inputs = command_inputs.iter().collect::<Vec<_>>();
    command_inputs.sort_by(|left, right| {
        left.workflow
            .as_ref()
            .cmp(right.workflow.as_ref())
            .then_with(|| left.slice.as_ref().cmp(right.slice.as_ref()))
            .then_with(|| left.command.as_ref().cmp(right.command.as_ref()))
            .then_with(|| left.input.as_ref().cmp(right.input.as_ref()))
            .then_with(|| left.source_kind.as_ref().cmp(right.source_kind.as_ref()))
            .then_with(|| {
                left.source_description
                    .as_ref()
                    .cmp(right.source_description.as_ref())
            })
            .then_with(|| compare_semantic_list(&left.provenance_chain, &right.provenance_chain))
            .then_with(|| {
                optional_text(&left.event_stream_source_event)
                    .cmp(optional_text(&right.event_stream_source_event))
            })
            .then_with(|| {
                optional_text(&left.event_stream_source_attribute)
                    .cmp(optional_text(&right.event_stream_source_attribute))
            })
            .then_with(|| {
                optional_text(&left.external_payload_source_name)
                    .cmp(optional_text(&right.external_payload_source_name))
            })
            .then_with(|| {
                optional_text(&left.external_payload_source_field)
                    .cmp(optional_text(&right.external_payload_source_field))
            })
            .then_with(|| {
                optional_text(&left.generated_source_name)
                    .cmp(optional_text(&right.generated_source_name))
            })
            .then_with(|| {
                optional_text(&left.generated_source_field)
                    .cmp(optional_text(&right.generated_source_field))
            })
            .then_with(|| {
                optional_text(&left.session_source_name)
                    .cmp(optional_text(&right.session_source_name))
            })
            .then_with(|| {
                optional_text(&left.session_source_field)
                    .cmp(optional_text(&right.session_source_field))
            })
            .then_with(|| {
                optional_text(&left.invocation_argument_source_name)
                    .cmp(optional_text(&right.invocation_argument_source_name))
            })
            .then_with(|| {
                optional_text(&left.invocation_argument_source_field)
                    .cmp(optional_text(&right.invocation_argument_source_field))
            })
    });
    command_inputs
}

fn compare_semantic_list<T: AsRef<str>>(left: &[T], right: &[T]) -> Ordering {
    left.iter()
        .map(|value| value.as_ref())
        .cmp(right.iter().map(|value| value.as_ref()))
}

fn render_list(items: impl Iterator<Item = String>) -> String {
    format!("[{}]", items.collect::<Vec<_>>().join(","))
}

fn render_spaced_list(items: impl Iterator<Item = String>) -> String {
    format!("[{}]", items.collect::<Vec<_>>().join(", "))
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC semantic formal model text must serialize as JSON string: {error}");
    })
}

fn quoted_list<T: AsRef<str>>(values: &[T]) -> String {
    render_list(values.iter().map(|value| quoted(value.as_ref())))
}

fn optional_text<T: AsRef<str>>(value: &Option<T>) -> &str {
    value.as_ref().map(AsRef::as_ref).unwrap_or("")
}
