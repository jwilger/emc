// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::emit::{lean_command_input_source_kind, quint_command_input_source_kind};
use crate::core::types::{
    AutomationName, AutomationReactionDescription, AutomationTriggerName, BitEncodingSemantics,
    BoardConnectionEndpoint, BoardConnectionEndpointKind, BoardElementDeclaredName,
    BoardElementKind, BoardElementName, BoardLaneId, CommandErrorName, CommandErrorRecoveryKind,
    CommandInputSourceDescription, CommandInputSourceKind, CommandName, ContractKindName,
    ControlName, ControlRecoveryBehavior, CoveredDefinitionName, DataFlowSource,
    DataFlowSourceKind, DataFlowTarget, DatumName, EventAttributeName, EventAttributeSourceField,
    EventAttributeSourceKind, EventAttributeSourceName, EventName,
    GeneratedEventAttributeSourceKind, NavigationTargetName, NavigationTargetType,
    OutcomeLabelName, PayloadContractName, ProvenanceDescription, ReadModelDerivationRule,
    ReadModelFieldSourceKind, ReadModelName, ReadModelTransitiveRule, ScenarioName,
    ScenarioStepText, SingletonRepeatBehavior, SketchToken, SliceSlug, SourceChainHop, StreamName,
    TransformationSemantics, TranslationExternalEventName, TranslationName, ViewFieldName,
    ViewFieldSourceKind, ViewName,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScenarioKind {
    Acceptance,
    Contract,
}

impl ScenarioKind {
    pub fn acceptance() -> Self {
        Self::Acceptance
    }

    pub fn contract() -> Self {
        Self::Contract
    }

    pub fn try_new(value: String) -> Result<Self, ScenarioKindError> {
        match value.trim() {
            "acceptance" => Ok(Self::Acceptance),
            "contract" => Ok(Self::Contract),
            _ => Err(ScenarioKindError::new(value)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Acceptance => "acceptance",
            Self::Contract => "contract",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScenarioKindError {
    message: String,
}

impl ScenarioKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled scenario kind, got '{value}'"),
        }
    }
}

impl Display for ScenarioKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ScenarioKindError {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewSliceScenario {
    slice_slug: SliceSlug,
    kind: ScenarioKind,
    name: ScenarioName,
    given: ScenarioStepText,
    when: ScenarioStepText,
    then: ScenarioStepText,
    read_streams: ScenarioStreamNames,
    written_streams: ScenarioStreamNames,
    contract_kind: Option<ContractKindName>,
    covered_definition: Option<CoveredDefinitionName>,
    error_references: CommandErrorNames,
}

impl NewSliceScenario {
    pub fn new(
        slice_slug: SliceSlug,
        kind: ScenarioKind,
        name: ScenarioName,
        given: ScenarioStepText,
        when: ScenarioStepText,
        then: ScenarioStepText,
    ) -> Self {
        Self {
            slice_slug,
            kind,
            name,
            given,
            when,
            then,
            read_streams: ScenarioStreamNames::empty(),
            written_streams: ScenarioStreamNames::empty(),
            contract_kind: None,
            covered_definition: None,
            error_references: CommandErrorNames::empty(),
        }
    }

    pub fn new_contract(
        slice_slug: SliceSlug,
        name: ScenarioName,
        given: ScenarioStepText,
        when: ScenarioStepText,
        then: ScenarioStepText,
        contract_kind: ContractKindName,
        covered_definition: CoveredDefinitionName,
    ) -> Self {
        Self {
            slice_slug,
            kind: ScenarioKind::Contract,
            name,
            given,
            when,
            then,
            read_streams: ScenarioStreamNames::empty(),
            written_streams: ScenarioStreamNames::empty(),
            contract_kind: Some(contract_kind),
            covered_definition: Some(covered_definition),
            error_references: CommandErrorNames::empty(),
        }
    }

    pub fn with_streams(
        mut self,
        read_streams: ScenarioStreamNames,
        written_streams: ScenarioStreamNames,
    ) -> Self {
        self.read_streams = read_streams;
        self.written_streams = written_streams;
        self
    }

    pub fn with_error_references(mut self, error_references: CommandErrorNames) -> Self {
        self.error_references = error_references;
        self
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn kind(&self) -> ScenarioKind {
        self.kind
    }

    pub fn name(&self) -> &ScenarioName {
        &self.name
    }

    pub fn given(&self) -> &ScenarioStepText {
        &self.given
    }

    pub fn when(&self) -> &ScenarioStepText {
        &self.when
    }

    pub fn then(&self) -> &ScenarioStepText {
        &self.then
    }

    pub fn read_streams(&self) -> &ScenarioStreamNames {
        &self.read_streams
    }

    pub fn written_streams(&self) -> &ScenarioStreamNames {
        &self.written_streams
    }

    pub fn contract_kind(&self) -> Option<&ContractKindName> {
        self.contract_kind.as_ref()
    }

    pub fn covered_definition(&self) -> Option<&CoveredDefinitionName> {
        self.covered_definition.as_ref()
    }

    pub fn error_references(&self) -> &CommandErrorNames {
        &self.error_references
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScenarioStreamNames {
    streams: Vec<StreamName>,
}

impl ScenarioStreamNames {
    pub fn empty() -> Self {
        Self {
            streams: Vec::new(),
        }
    }

    pub fn from_streams(streams: impl IntoIterator<Item = StreamName>) -> Self {
        Self {
            streams: streams.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[StreamName] {
        &self.streams
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewBitLevelDataFlow {
    slice_slug: SliceSlug,
    datum: DatumName,
    source_kind: DataFlowSourceKind,
    source: DataFlowSource,
    transformation: TransformationSemantics,
    target: DataFlowTarget,
    bit_encoding: BitEncodingSemantics,
}

impl NewBitLevelDataFlow {
    pub fn new(
        slice_slug: SliceSlug,
        datum: DatumName,
        source_kind: DataFlowSourceKind,
        source: DataFlowSource,
        transformation: TransformationSemantics,
        target: DataFlowTarget,
        bit_encoding: BitEncodingSemantics,
    ) -> Self {
        Self {
            slice_slug,
            datum,
            source_kind,
            source,
            transformation,
            target,
            bit_encoding,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn datum(&self) -> &DatumName {
        &self.datum
    }

    pub fn source(&self) -> &DataFlowSource {
        &self.source
    }

    pub fn source_kind(&self) -> &DataFlowSourceKind {
        &self.source_kind
    }

    pub fn transformation(&self) -> &TransformationSemantics {
        &self.transformation
    }

    pub fn target(&self) -> &DataFlowTarget {
        &self.target
    }

    pub fn bit_encoding(&self) -> &BitEncodingSemantics {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandInputEventStreamSource {
    event: EventName,
    attribute: EventAttributeName,
}

impl CommandInputEventStreamSource {
    fn new(event: EventName, attribute: EventAttributeName) -> Self {
        Self { event, attribute }
    }

    pub fn event(&self) -> &EventName {
        &self.event
    }

    pub fn attribute(&self) -> &EventAttributeName {
        &self.attribute
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandInputNamedFieldSource {
    name: EventAttributeSourceName,
    field: EventAttributeSourceField,
}

impl CommandInputNamedFieldSource {
    fn new(name: EventAttributeSourceName, field: EventAttributeSourceField) -> Self {
        Self { name, field }
    }

    pub fn name(&self) -> &EventAttributeSourceName {
        &self.name
    }

    pub fn field(&self) -> &EventAttributeSourceField {
        &self.field
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CommandInputSource {
    Actor,
    EventStreamState(CommandInputEventStreamSource),
    ExternalPayload(CommandInputNamedFieldSource),
    Generated(CommandInputNamedFieldSource),
    Session(CommandInputNamedFieldSource),
    InvocationArgument(CommandInputNamedFieldSource),
}

impl CommandInputSource {
    pub fn actor() -> Self {
        Self::Actor
    }

    pub fn event_stream_state(event: EventName, attribute: EventAttributeName) -> Self {
        Self::EventStreamState(CommandInputEventStreamSource::new(event, attribute))
    }

    pub fn external_payload(
        payload: EventAttributeSourceName,
        field: EventAttributeSourceField,
    ) -> Self {
        Self::ExternalPayload(CommandInputNamedFieldSource::new(payload, field))
    }

    pub fn generated(source: EventAttributeSourceName, field: EventAttributeSourceField) -> Self {
        Self::Generated(CommandInputNamedFieldSource::new(source, field))
    }

    pub fn session(source: EventAttributeSourceName, field: EventAttributeSourceField) -> Self {
        Self::Session(CommandInputNamedFieldSource::new(source, field))
    }

    pub fn invocation_argument(
        argument: EventAttributeSourceName,
        field: EventAttributeSourceField,
    ) -> Self {
        Self::InvocationArgument(CommandInputNamedFieldSource::new(argument, field))
    }

    pub fn kind(&self) -> CommandInputSourceKind {
        match self {
            Self::Actor => CommandInputSourceKind::Actor,
            Self::EventStreamState(_) => CommandInputSourceKind::EventStreamState,
            Self::ExternalPayload(_) => CommandInputSourceKind::ExternalPayload,
            Self::Generated(_) => CommandInputSourceKind::Generated,
            Self::Session(_) => CommandInputSourceKind::Session,
            Self::InvocationArgument(_) => CommandInputSourceKind::InvocationArgument,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewCommandInput {
    name: DatumName,
    source: CommandInputSource,
    source_description: CommandInputSourceDescription,
    provenance_chain: CommandInputProvenanceChain,
}

impl NewCommandInput {
    pub fn new(
        name: DatumName,
        source: CommandInputSource,
        source_description: CommandInputSourceDescription,
        provenance_chain: CommandInputProvenanceChain,
    ) -> Self {
        Self {
            name,
            source,
            source_description,
            provenance_chain,
        }
    }

    pub fn name(&self) -> &DatumName {
        &self.name
    }

    pub fn source_kind(&self) -> CommandInputSourceKind {
        self.source.kind()
    }

    pub fn source_description(&self) -> &CommandInputSourceDescription {
        &self.source_description
    }

    pub fn provenance_chain(&self) -> &CommandInputProvenanceChain {
        &self.provenance_chain
    }

    pub fn event_stream_source_event(&self) -> Option<&EventName> {
        match &self.source {
            CommandInputSource::EventStreamState(source) => Some(source.event()),
            _ => None,
        }
    }

    pub fn event_stream_source_attribute(&self) -> Option<&EventAttributeName> {
        match &self.source {
            CommandInputSource::EventStreamState(source) => Some(source.attribute()),
            _ => None,
        }
    }

    pub fn external_payload_source_name(&self) -> Option<&EventAttributeSourceName> {
        match &self.source {
            CommandInputSource::ExternalPayload(source) => Some(source.name()),
            _ => None,
        }
    }

    pub fn external_payload_source_field(&self) -> Option<&EventAttributeSourceField> {
        match &self.source {
            CommandInputSource::ExternalPayload(source) => Some(source.field()),
            _ => None,
        }
    }

    pub fn generated_source_name(&self) -> Option<&EventAttributeSourceName> {
        match &self.source {
            CommandInputSource::Generated(source) => Some(source.name()),
            _ => None,
        }
    }

    pub fn generated_source_field(&self) -> Option<&EventAttributeSourceField> {
        match &self.source {
            CommandInputSource::Generated(source) => Some(source.field()),
            _ => None,
        }
    }

    pub fn session_source_name(&self) -> Option<&EventAttributeSourceName> {
        match &self.source {
            CommandInputSource::Session(source) => Some(source.name()),
            _ => None,
        }
    }

    pub fn session_source_field(&self) -> Option<&EventAttributeSourceField> {
        match &self.source {
            CommandInputSource::Session(source) => Some(source.field()),
            _ => None,
        }
    }

    pub fn invocation_argument_source_name(&self) -> Option<&EventAttributeSourceName> {
        match &self.source {
            CommandInputSource::InvocationArgument(source) => Some(source.name()),
            _ => None,
        }
    }

    pub fn invocation_argument_source_field(&self) -> Option<&EventAttributeSourceField> {
        match &self.source {
            CommandInputSource::InvocationArgument(source) => Some(source.field()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandInputProvenanceChain {
    hops: Vec<SourceChainHop>,
}

impl CommandInputProvenanceChain {
    pub fn from_hops(hops: impl IntoIterator<Item = SourceChainHop>) -> Self {
        Self {
            hops: hops.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[SourceChainHop] {
        &self.hops
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandObservedStreams {
    streams: Vec<StreamName>,
}

impl CommandObservedStreams {
    pub fn empty() -> Self {
        Self {
            streams: Vec::new(),
        }
    }

    pub fn from_streams(streams: impl IntoIterator<Item = StreamName>) -> Self {
        Self {
            streams: streams.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[StreamName] {
        &self.streams
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewCommandDefinition {
    slice_slug: SliceSlug,
    name: CommandName,
    input: NewCommandInput,
    emitted_events: EmittedEventNames,
    observed_streams: CommandObservedStreams,
    errors: CommandErrorDefinitions,
    singleton_repeat_behavior: Option<SingletonRepeatBehavior>,
}

impl NewCommandDefinition {
    pub fn new(
        slice_slug: SliceSlug,
        name: CommandName,
        input: NewCommandInput,
        emitted_events: EmittedEventNames,
    ) -> Self {
        Self {
            slice_slug,
            name,
            input,
            emitted_events,
            observed_streams: CommandObservedStreams::empty(),
            errors: CommandErrorDefinitions::empty(),
            singleton_repeat_behavior: None,
        }
    }

    pub fn with_observed_streams(mut self, observed_streams: CommandObservedStreams) -> Self {
        self.observed_streams = observed_streams;
        self
    }

    pub fn with_errors(mut self, errors: CommandErrorDefinitions) -> Self {
        self.errors = errors;
        self
    }

    pub fn with_singleton_repeat_behavior(
        mut self,
        repeat_behavior: SingletonRepeatBehavior,
    ) -> Self {
        self.singleton_repeat_behavior = Some(repeat_behavior);
        self
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn name(&self) -> &CommandName {
        &self.name
    }

    pub fn input(&self) -> &NewCommandInput {
        &self.input
    }

    pub fn emitted_events(&self) -> &EmittedEventNames {
        &self.emitted_events
    }

    pub fn observed_streams(&self) -> &CommandObservedStreams {
        &self.observed_streams
    }

    pub fn errors(&self) -> &CommandErrorDefinitions {
        &self.errors
    }

    pub fn singleton_repeat_behavior(&self) -> Option<&SingletonRepeatBehavior> {
        self.singleton_repeat_behavior.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewCommandErrorDefinition {
    name: CommandErrorName,
    scenario_name: ScenarioName,
    recovery_kind: CommandErrorRecoveryKind,
}

impl NewCommandErrorDefinition {
    pub fn new(
        name: CommandErrorName,
        scenario_name: ScenarioName,
        recovery_kind: CommandErrorRecoveryKind,
    ) -> Self {
        Self {
            name,
            scenario_name,
            recovery_kind,
        }
    }

    pub fn name(&self) -> &CommandErrorName {
        &self.name
    }

    pub fn scenario_name(&self) -> &ScenarioName {
        &self.scenario_name
    }

    pub fn recovery_kind(&self) -> &CommandErrorRecoveryKind {
        &self.recovery_kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandErrorDefinitions {
    errors: Vec<NewCommandErrorDefinition>,
}

impl CommandErrorDefinitions {
    pub fn empty() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn from_errors(errors: impl IntoIterator<Item = NewCommandErrorDefinition>) -> Self {
        Self {
            errors: errors.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[NewCommandErrorDefinition] {
        &self.errors
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EmittedEventNames {
    events: Vec<EventName>,
}

impl EmittedEventNames {
    pub fn from_events(events: impl IntoIterator<Item = EventName>) -> Self {
        Self {
            events: events.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[EventName] {
        &self.events
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OutcomeEventNames {
    events: Vec<EventName>,
}

impl OutcomeEventNames {
    pub fn from_events(events: impl IntoIterator<Item = EventName>) -> Self {
        Self {
            events: events.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[EventName] {
        &self.events
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewOutcomeDefinition {
    slice_slug: SliceSlug,
    label: OutcomeLabelName,
    event_set: OutcomeEventNames,
    externally_relevant: bool,
}

impl NewOutcomeDefinition {
    pub fn new(
        slice_slug: SliceSlug,
        label: OutcomeLabelName,
        event_set: OutcomeEventNames,
        externally_relevant: bool,
    ) -> Self {
        Self {
            slice_slug,
            label,
            event_set,
            externally_relevant,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn label(&self) -> &OutcomeLabelName {
        &self.label
    }

    pub fn event_set(&self) -> &OutcomeEventNames {
        &self.event_set
    }

    pub fn externally_relevant(&self) -> bool {
        self.externally_relevant
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewExternalPayloadDefinition {
    slice_slug: SliceSlug,
    name: EventAttributeSourceName,
    field: EventAttributeSourceField,
    field_provenance: ProvenanceDescription,
    bit_encoding: BitEncodingSemantics,
}

impl NewExternalPayloadDefinition {
    pub fn new(
        slice_slug: SliceSlug,
        name: EventAttributeSourceName,
        field: EventAttributeSourceField,
        field_provenance: ProvenanceDescription,
        bit_encoding: BitEncodingSemantics,
    ) -> Self {
        Self {
            slice_slug,
            name,
            field,
            field_provenance,
            bit_encoding,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn name(&self) -> &EventAttributeSourceName {
        &self.name
    }

    pub fn field(&self) -> &EventAttributeSourceField {
        &self.field
    }

    pub fn field_provenance(&self) -> &ProvenanceDescription {
        &self.field_provenance
    }

    pub fn bit_encoding(&self) -> &BitEncodingSemantics {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewEventAttribute {
    name: EventAttributeName,
    source_kind: EventAttributeSourceKind,
    source_name: EventAttributeSourceName,
    source_field: EventAttributeSourceField,
    generated_source_kind: Option<GeneratedEventAttributeSourceKind>,
    provenance_description: ProvenanceDescription,
}

impl NewEventAttribute {
    pub fn new(
        name: EventAttributeName,
        source_kind: EventAttributeSourceKind,
        source_name: EventAttributeSourceName,
        source_field: EventAttributeSourceField,
        provenance_description: ProvenanceDescription,
    ) -> Self {
        Self {
            name,
            source_kind,
            source_name,
            source_field,
            generated_source_kind: None,
            provenance_description,
        }
    }

    pub fn new_with_generated_source_kind(
        name: EventAttributeName,
        source_kind: EventAttributeSourceKind,
        source_name: EventAttributeSourceName,
        source_field: EventAttributeSourceField,
        generated_source_kind: GeneratedEventAttributeSourceKind,
        provenance_description: ProvenanceDescription,
    ) -> Self {
        Self {
            name,
            source_kind,
            source_name,
            source_field,
            generated_source_kind: Some(generated_source_kind),
            provenance_description,
        }
    }

    pub fn name(&self) -> &EventAttributeName {
        &self.name
    }

    pub fn source_kind(&self) -> &EventAttributeSourceKind {
        &self.source_kind
    }

    pub fn source_name(&self) -> &EventAttributeSourceName {
        &self.source_name
    }

    pub fn source_field(&self) -> &EventAttributeSourceField {
        &self.source_field
    }

    pub fn generated_source_kind(&self) -> Option<&GeneratedEventAttributeSourceKind> {
        self.generated_source_kind.as_ref()
    }

    pub fn provenance_description(&self) -> &ProvenanceDescription {
        &self.provenance_description
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewEventDefinition {
    slice_slug: SliceSlug,
    name: EventName,
    stream: StreamName,
    attribute: NewEventAttribute,
    observed: bool,
    shared: bool,
}

impl NewEventDefinition {
    pub fn new(
        slice_slug: SliceSlug,
        name: EventName,
        stream: StreamName,
        attribute: NewEventAttribute,
    ) -> Self {
        Self {
            slice_slug,
            name,
            stream,
            attribute,
            observed: false,
            shared: false,
        }
    }

    pub fn new_observed(
        slice_slug: SliceSlug,
        name: EventName,
        stream: StreamName,
        attribute: NewEventAttribute,
    ) -> Self {
        Self {
            slice_slug,
            name,
            stream,
            attribute,
            observed: true,
            shared: false,
        }
    }

    pub fn new_shared(
        slice_slug: SliceSlug,
        name: EventName,
        stream: StreamName,
        attribute: NewEventAttribute,
    ) -> Self {
        Self {
            slice_slug,
            name,
            stream,
            attribute,
            observed: false,
            shared: true,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn stream(&self) -> &StreamName {
        &self.stream
    }

    pub fn name(&self) -> &EventName {
        &self.name
    }

    pub fn attribute(&self) -> &NewEventAttribute {
        &self.attribute
    }

    pub fn observed(&self) -> bool {
        self.observed
    }

    pub fn shared(&self) -> bool {
        self.shared
    }
}

/// Reject an event attribute that sources a command input no command in the
/// slice actually provides.
///
/// An attribute with [`EventAttributeSourceKind::CommandInput`] names a command
/// input through its `source_name`; the slice's `eventAttributeSourcesAreComplete`
/// verification later requires some command that *emits this event* to declare
/// an input with that exact name. A common authoring mistake is passing the
/// COMMAND name (or any non-input identifier) to `--attribute-source-name`,
/// which silently persists and only surfaces much later as a verification
/// failure — with no per-attribute correction path. This catches that mistake
/// at write time, answering it cheaply from the projected model.
///
/// The check is deliberately conservative so it never blocks legitimate
/// incremental authoring: it fires only once a command emitting the event
/// exists. While no emitting command has been declared yet (an event authored
/// before its command), the reference cannot be judged from the current model
/// and is allowed — the verification gate still enforces completeness before the
/// workflow can advance to the next phase.
pub(crate) fn validate_event_attribute_source(
    event: &NewEventDefinition,
    slice_commands: &[NewCommandDefinition],
) -> Result<(), FormalSliceFactError> {
    let attribute = event.attribute();
    if *attribute.source_kind() != EventAttributeSourceKind::CommandInput {
        return Ok(());
    }

    let mut available_inputs: Vec<&str> = slice_commands
        .iter()
        .filter(|command| {
            command
                .emitted_events()
                .as_slice()
                .iter()
                .any(|emitted| emitted == event.name())
        })
        .map(|command| command.input().name().as_ref())
        .collect();

    // No command emits this event yet: the source cannot be judged from the
    // current model, so defer to the verification gate.
    if available_inputs.is_empty() {
        return Ok(());
    }

    let source_name = attribute.source_name().as_ref();
    if available_inputs.contains(&source_name) {
        return Ok(());
    }

    available_inputs.sort_unstable();
    available_inputs.dedup();
    Err(FormalSliceFactError::new(format!(
        "event '{event_name}' attribute '{attribute_name}' sources command input \
         '{source_name}', but no command emitting '{event_name}' in slice '{slice}' declares \
         that input (available inputs: {available}); pass the command INPUT name to \
         --attribute-source-name",
        event_name = event.name().as_ref(),
        attribute_name = attribute.name().as_ref(),
        slice = event.slice_slug().as_ref(),
        available = available_inputs.join(", "),
    )))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelEventAttributeSource {
    event: EventName,
    attribute: EventAttributeName,
}

impl ReadModelEventAttributeSource {
    fn new(event: EventName, attribute: EventAttributeName) -> Self {
        Self { event, attribute }
    }

    pub fn event(&self) -> &EventName {
        &self.event
    }

    pub fn attribute(&self) -> &EventAttributeName {
        &self.attribute
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelDerivationSource {
    rule: ReadModelDerivationRule,
    source_fields: ReadModelDerivationSourceFields,
    scenario_name: ScenarioName,
}

impl ReadModelDerivationSource {
    fn new(
        rule: ReadModelDerivationRule,
        source_fields: ReadModelDerivationSourceFields,
        scenario_name: ScenarioName,
    ) -> Self {
        Self {
            rule,
            source_fields,
            scenario_name,
        }
    }

    pub fn rule(&self) -> &ReadModelDerivationRule {
        &self.rule
    }

    pub fn source_fields(&self) -> &ReadModelDerivationSourceFields {
        &self.source_fields
    }

    pub fn scenario_name(&self) -> &ScenarioName {
        &self.scenario_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelAbsenceDefaultSource {
    event: EventName,
    scenario_name: ScenarioName,
}

impl ReadModelAbsenceDefaultSource {
    fn new(event: EventName, scenario_name: ScenarioName) -> Self {
        Self {
            event,
            scenario_name,
        }
    }

    pub fn event(&self) -> &EventName {
        &self.event
    }

    pub fn scenario_name(&self) -> &ScenarioName {
        &self.scenario_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadModelFieldSource {
    EventAttribute(ReadModelEventAttributeSource),
    Derivation(ReadModelDerivationSource),
    AbsenceDefault(ReadModelAbsenceDefaultSource),
}

impl ReadModelFieldSource {
    pub fn event_attribute(event: EventName, attribute: EventAttributeName) -> Self {
        Self::EventAttribute(ReadModelEventAttributeSource::new(event, attribute))
    }

    pub fn derivation(
        rule: ReadModelDerivationRule,
        source_fields: ReadModelDerivationSourceFields,
        scenario_name: ScenarioName,
    ) -> Self {
        Self::Derivation(ReadModelDerivationSource::new(
            rule,
            source_fields,
            scenario_name,
        ))
    }

    pub fn absence_default(event: EventName, scenario_name: ScenarioName) -> Self {
        Self::AbsenceDefault(ReadModelAbsenceDefaultSource::new(event, scenario_name))
    }

    pub fn kind(&self) -> ReadModelFieldSourceKind {
        match self {
            Self::EventAttribute(_) => ReadModelFieldSourceKind::EventAttribute,
            Self::Derivation(_) => ReadModelFieldSourceKind::Derivation,
            Self::AbsenceDefault(_) => ReadModelFieldSourceKind::AbsenceDefault,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewReadModelField {
    name: DatumName,
    source: ReadModelFieldSource,
    provenance_description: ProvenanceDescription,
}

impl NewReadModelField {
    pub fn new(
        name: DatumName,
        source: ReadModelFieldSource,
        provenance_description: ProvenanceDescription,
    ) -> Self {
        Self {
            name,
            source,
            provenance_description,
        }
    }

    pub fn name(&self) -> &DatumName {
        &self.name
    }

    pub fn source_kind(&self) -> ReadModelFieldSourceKind {
        self.source.kind()
    }

    pub fn source_event(&self) -> Option<&EventName> {
        match &self.source {
            ReadModelFieldSource::EventAttribute(source) => Some(source.event()),
            _ => None,
        }
    }

    pub fn source_attribute(&self) -> Option<&EventAttributeName> {
        match &self.source {
            ReadModelFieldSource::EventAttribute(source) => Some(source.attribute()),
            _ => None,
        }
    }

    pub fn derivation_rule(&self) -> Option<&ReadModelDerivationRule> {
        match &self.source {
            ReadModelFieldSource::Derivation(source) => Some(source.rule()),
            _ => None,
        }
    }

    pub fn derivation_source_fields(&self) -> &ReadModelDerivationSourceFields {
        match &self.source {
            ReadModelFieldSource::Derivation(source) => source.source_fields(),
            _ => &EMPTY_READ_MODEL_DERIVATION_SOURCE_FIELDS,
        }
    }

    pub fn absence_event(&self) -> Option<&EventName> {
        match &self.source {
            ReadModelFieldSource::AbsenceDefault(source) => Some(source.event()),
            _ => None,
        }
    }

    pub fn derivation_scenario_name(&self) -> Option<&ScenarioName> {
        match &self.source {
            ReadModelFieldSource::Derivation(source) => Some(source.scenario_name()),
            _ => None,
        }
    }

    pub fn absence_scenario_name(&self) -> Option<&ScenarioName> {
        match &self.source {
            ReadModelFieldSource::AbsenceDefault(source) => Some(source.scenario_name()),
            _ => None,
        }
    }

    pub fn provenance_description(&self) -> &ProvenanceDescription {
        &self.provenance_description
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewReadModelDefinition {
    slice_slug: SliceSlug,
    name: ReadModelName,
    field: NewReadModelField,
    transitive: bool,
    relationship_fields: ReadModelRelationshipFields,
    transitive_rule: Option<ReadModelTransitiveRule>,
    example_scenario_name: Option<ScenarioName>,
}

impl NewReadModelDefinition {
    pub fn new(slice_slug: SliceSlug, name: ReadModelName, field: NewReadModelField) -> Self {
        Self {
            slice_slug,
            name,
            field,
            transitive: false,
            relationship_fields: ReadModelRelationshipFields::empty(),
            transitive_rule: None,
            example_scenario_name: None,
        }
    }

    pub fn with_transitive_semantics(
        mut self,
        relationship_fields: ReadModelRelationshipFields,
        transitive_rule: ReadModelTransitiveRule,
        example_scenario_name: ScenarioName,
    ) -> Self {
        self.transitive = true;
        self.relationship_fields = relationship_fields;
        self.transitive_rule = Some(transitive_rule);
        self.example_scenario_name = Some(example_scenario_name);
        self
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn name(&self) -> &ReadModelName {
        &self.name
    }

    pub fn field(&self) -> &NewReadModelField {
        &self.field
    }

    pub fn transitive(&self) -> bool {
        self.transitive
    }

    pub fn relationship_fields(&self) -> &ReadModelRelationshipFields {
        &self.relationship_fields
    }

    pub fn transitive_rule(&self) -> Option<&ReadModelTransitiveRule> {
        self.transitive_rule.as_ref()
    }

    pub fn example_scenario_name(&self) -> Option<&ScenarioName> {
        self.example_scenario_name.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelRelationshipFields {
    fields: Vec<DatumName>,
}

impl ReadModelRelationshipFields {
    pub fn empty() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn from_fields(fields: impl IntoIterator<Item = DatumName>) -> Self {
        Self {
            fields: fields.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[DatumName] {
        &self.fields
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelDerivationSourceFields {
    fields: Vec<DatumName>,
}

static EMPTY_READ_MODEL_DERIVATION_SOURCE_FIELDS: ReadModelDerivationSourceFields =
    ReadModelDerivationSourceFields { fields: Vec::new() };

impl ReadModelDerivationSourceFields {
    pub fn from_fields(fields: impl IntoIterator<Item = DatumName>) -> Self {
        Self {
            fields: fields.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[DatumName] {
        &self.fields
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewViewField {
    name: ViewFieldName,
    source_kind: ViewFieldSourceKind,
    source_read_model: ReadModelName,
    source_field: ViewFieldName,
    sketch_token: SketchToken,
    provenance_description: ProvenanceDescription,
    bit_encoding: BitEncodingSemantics,
}

impl NewViewField {
    pub fn new(
        name: ViewFieldName,
        source_kind: ViewFieldSourceKind,
        source_read_model: ReadModelName,
        source_field: ViewFieldName,
        sketch_token: SketchToken,
        provenance_description: ProvenanceDescription,
        bit_encoding: BitEncodingSemantics,
    ) -> Self {
        Self {
            name,
            source_kind,
            source_read_model,
            source_field,
            sketch_token,
            provenance_description,
            bit_encoding,
        }
    }

    pub fn name(&self) -> &ViewFieldName {
        &self.name
    }

    pub fn source_kind(&self) -> &ViewFieldSourceKind {
        &self.source_kind
    }

    pub fn source_read_model(&self) -> &ReadModelName {
        &self.source_read_model
    }

    pub fn source_field(&self) -> &ViewFieldName {
        &self.source_field
    }

    pub fn sketch_token(&self) -> &SketchToken {
        &self.sketch_token
    }

    pub fn provenance_description(&self) -> &ProvenanceDescription {
        &self.provenance_description
    }

    pub fn bit_encoding(&self) -> &BitEncodingSemantics {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewViewDefinition {
    slice_slug: SliceSlug,
    name: ViewName,
    field: NewViewField,
    controls: ViewControls,
    local_states: ViewLocalStates,
    filters: ViewFilters,
}

impl NewViewDefinition {
    pub fn new(slice_slug: SliceSlug, name: ViewName, field: NewViewField) -> Self {
        Self {
            slice_slug,
            name,
            field,
            controls: ViewControls::empty(),
            local_states: ViewLocalStates::empty(),
            filters: ViewFilters::empty(),
        }
    }

    pub fn with_controls(mut self, controls: ViewControls) -> Self {
        self.controls = controls;
        self
    }

    pub fn with_local_states(mut self, local_states: ViewLocalStates) -> Self {
        self.local_states = local_states;
        self
    }

    pub fn with_filters(mut self, filters: ViewFilters) -> Self {
        self.filters = filters;
        self
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn name(&self) -> &ViewName {
        &self.name
    }

    pub fn field(&self) -> &NewViewField {
        &self.field
    }

    pub fn controls(&self) -> &ViewControls {
        &self.controls
    }

    pub fn local_states(&self) -> &ViewLocalStates {
        &self.local_states
    }

    pub fn filters(&self) -> &ViewFilters {
        &self.filters
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewLocalStates {
    targets: Vec<NavigationTargetName>,
}

impl ViewLocalStates {
    pub fn empty() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    pub fn from_targets(targets: impl IntoIterator<Item = NavigationTargetName>) -> Self {
        Self {
            targets: targets.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[NavigationTargetName] {
        &self.targets
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewFilters {
    targets: Vec<NavigationTargetName>,
}

impl ViewFilters {
    pub fn empty() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    pub fn from_targets(targets: impl IntoIterator<Item = NavigationTargetName>) -> Self {
        Self {
            targets: targets.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[NavigationTargetName] {
        &self.targets
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewControlInputProvision {
    name: DatumName,
    source_kind: CommandInputSourceKind,
    source_description: CommandInputSourceDescription,
    sketch_token: SketchToken,
    visible_to_actor: bool,
    decision_field: bool,
}

impl NewControlInputProvision {
    pub fn new(
        name: DatumName,
        source_kind: CommandInputSourceKind,
        source_description: CommandInputSourceDescription,
        sketch_token: SketchToken,
        visible_to_actor: bool,
        decision_field: bool,
    ) -> Self {
        Self {
            name,
            source_kind,
            source_description,
            sketch_token,
            visible_to_actor,
            decision_field,
        }
    }

    pub fn name(&self) -> &DatumName {
        &self.name
    }

    pub fn source_kind(&self) -> &CommandInputSourceKind {
        &self.source_kind
    }

    pub fn source_description(&self) -> &CommandInputSourceDescription {
        &self.source_description
    }

    pub fn sketch_token(&self) -> &SketchToken {
        &self.sketch_token
    }

    pub fn visible_to_actor(&self) -> bool {
        self.visible_to_actor
    }

    pub fn decision_field(&self) -> bool {
        self.decision_field
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewNavigationTarget {
    target_type: NavigationTargetType,
    target_name: NavigationTargetName,
    external_workflow_name: Option<NavigationTargetName>,
    external_system_name: Option<NavigationTargetName>,
    handoff_contract: Option<PayloadContractName>,
}

impl NewNavigationTarget {
    pub fn new(target_type: NavigationTargetType, target_name: NavigationTargetName) -> Self {
        Self {
            target_type,
            target_name,
            external_workflow_name: None,
            external_system_name: None,
            handoff_contract: None,
        }
    }

    pub fn with_external_system(
        mut self,
        external_system_name: NavigationTargetName,
        handoff_contract: PayloadContractName,
    ) -> Self {
        self.external_system_name = Some(external_system_name);
        self.handoff_contract = Some(handoff_contract);
        self
    }

    pub fn with_external_workflow(mut self, external_workflow_name: NavigationTargetName) -> Self {
        self.external_workflow_name = Some(external_workflow_name);
        self
    }

    pub fn target_type(&self) -> &NavigationTargetType {
        &self.target_type
    }

    pub fn target_name(&self) -> &NavigationTargetName {
        &self.target_name
    }

    pub fn external_workflow_name(&self) -> Option<&NavigationTargetName> {
        self.external_workflow_name.as_ref()
    }

    pub fn external_system_name(&self) -> Option<&NavigationTargetName> {
        self.external_system_name.as_ref()
    }

    pub fn handoff_contract(&self) -> Option<&PayloadContractName> {
        self.handoff_contract.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewAutomationDefinition {
    slice_slug: SliceSlug,
    name: AutomationName,
    trigger_name: AutomationTriggerName,
    command_name: CommandName,
    handled_errors: CommandErrorNames,
    reaction_description: AutomationReactionDescription,
}

impl NewAutomationDefinition {
    pub fn new(
        slice_slug: SliceSlug,
        name: AutomationName,
        trigger_name: AutomationTriggerName,
        command_name: CommandName,
        handled_errors: CommandErrorNames,
        reaction_description: AutomationReactionDescription,
    ) -> Self {
        Self {
            slice_slug,
            name,
            trigger_name,
            command_name,
            handled_errors,
            reaction_description,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn name(&self) -> &AutomationName {
        &self.name
    }

    pub fn trigger_name(&self) -> &AutomationTriggerName {
        &self.trigger_name
    }

    pub fn command_name(&self) -> &CommandName {
        &self.command_name
    }

    pub fn handled_errors(&self) -> &CommandErrorNames {
        &self.handled_errors
    }

    pub fn reaction_description(&self) -> &AutomationReactionDescription {
        &self.reaction_description
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewTranslationDefinition {
    slice_slug: SliceSlug,
    name: TranslationName,
    external_event_name: TranslationExternalEventName,
    payload_contract_name: PayloadContractName,
    command_name: CommandName,
}

impl NewTranslationDefinition {
    pub fn new(
        slice_slug: SliceSlug,
        name: TranslationName,
        external_event_name: TranslationExternalEventName,
        payload_contract_name: PayloadContractName,
        command_name: CommandName,
    ) -> Self {
        Self {
            slice_slug,
            name,
            external_event_name,
            payload_contract_name,
            command_name,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn name(&self) -> &TranslationName {
        &self.name
    }

    pub fn external_event_name(&self) -> &TranslationExternalEventName {
        &self.external_event_name
    }

    pub fn payload_contract_name(&self) -> &PayloadContractName {
        &self.payload_contract_name
    }

    pub fn command_name(&self) -> &CommandName {
        &self.command_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewBoardElement {
    slice_slug: SliceSlug,
    name: BoardElementName,
    kind: BoardElementKind,
    lane: BoardLaneId,
    declared_name: BoardElementDeclaredName,
    main_path: bool,
}

impl NewBoardElement {
    pub fn new(
        slice_slug: SliceSlug,
        name: BoardElementName,
        kind: BoardElementKind,
        lane: BoardLaneId,
        declared_name: BoardElementDeclaredName,
        main_path: bool,
    ) -> Self {
        Self {
            slice_slug,
            name,
            kind,
            lane,
            declared_name,
            main_path,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn name(&self) -> &BoardElementName {
        &self.name
    }

    pub fn kind(&self) -> &BoardElementKind {
        &self.kind
    }

    pub fn lane(&self) -> &BoardLaneId {
        &self.lane
    }

    pub fn declared_name(&self) -> &BoardElementDeclaredName {
        &self.declared_name
    }

    pub fn main_path(&self) -> bool {
        self.main_path
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewBoardConnection {
    slice_slug: SliceSlug,
    source: BoardConnectionEndpoint,
    source_kind: BoardConnectionEndpointKind,
    target: BoardConnectionEndpoint,
    target_kind: BoardConnectionEndpointKind,
}

impl NewBoardConnection {
    pub fn new(
        slice_slug: SliceSlug,
        source: BoardConnectionEndpoint,
        source_kind: BoardConnectionEndpointKind,
        target: BoardConnectionEndpoint,
        target_kind: BoardConnectionEndpointKind,
    ) -> Self {
        Self {
            slice_slug,
            source,
            source_kind,
            target,
            target_kind,
        }
    }

    pub fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub fn source(&self) -> &BoardConnectionEndpoint {
        &self.source
    }

    pub fn source_kind(&self) -> &BoardConnectionEndpointKind {
        &self.source_kind
    }

    pub fn target(&self) -> &BoardConnectionEndpoint {
        &self.target
    }

    pub fn target_kind(&self) -> &BoardConnectionEndpointKind {
        &self.target_kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewControlDefinition {
    name: ControlName,
    command_name: CommandName,
    input: NewControlInputProvision,
    handled_errors: CommandErrorNames,
    recovery_behavior: ControlRecoveryBehavior,
    sketch_token: SketchToken,
    navigation: NewNavigationTarget,
}

impl NewControlDefinition {
    pub fn new(
        name: ControlName,
        command_name: CommandName,
        input: NewControlInputProvision,
        handled_errors: CommandErrorNames,
        recovery_behavior: ControlRecoveryBehavior,
        sketch_token: SketchToken,
        navigation: NewNavigationTarget,
    ) -> Self {
        Self {
            name,
            command_name,
            input,
            handled_errors,
            recovery_behavior,
            sketch_token,
            navigation,
        }
    }

    pub fn name(&self) -> &ControlName {
        &self.name
    }

    pub fn command_name(&self) -> &CommandName {
        &self.command_name
    }

    pub fn input(&self) -> &NewControlInputProvision {
        &self.input
    }

    pub fn handled_errors(&self) -> &CommandErrorNames {
        &self.handled_errors
    }

    pub fn recovery_behavior(&self) -> &ControlRecoveryBehavior {
        &self.recovery_behavior
    }

    pub fn sketch_token(&self) -> &SketchToken {
        &self.sketch_token
    }

    pub fn navigation(&self) -> &NewNavigationTarget {
        &self.navigation
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandErrorNames {
    names: Vec<CommandErrorName>,
}

impl CommandErrorNames {
    pub fn empty() -> Self {
        Self { names: Vec::new() }
    }

    pub fn from_names(names: impl IntoIterator<Item = CommandErrorName>) -> Self {
        Self {
            names: names.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[CommandErrorName] {
        &self.names
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewControls {
    controls: Vec<NewControlDefinition>,
}

impl ViewControls {
    pub fn empty() -> Self {
        Self {
            controls: Vec::new(),
        }
    }

    pub fn from_controls(controls: impl IntoIterator<Item = NewControlDefinition>) -> Self {
        Self {
            controls: controls.into_iter().collect(),
        }
    }

    pub fn as_slice(&self) -> &[NewControlDefinition] {
        &self.controls
    }
}

pub(crate) fn add_slice_scenario(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    scenario: NewSliceScenario,
) -> Result<EffectPlan, FormalSliceFactError> {
    let (lean_marker, quint_marker) = match scenario.kind {
        ScenarioKind::Acceptance => (
            "def sliceAcceptanceScenarios : List EventModelScenario := ",
            "val sliceAcceptanceScenarios: List[EventModelScenario] = ",
        ),
        ScenarioKind::Contract => (
            "def sliceContractScenarios : List EventModelScenario := ",
            "val sliceContractScenarios: List[EventModelScenario] = ",
        ),
    };
    let lean_record = lean_scenario_record(&scenario);
    let quint_record = quint_scenario_record(&scenario);
    let lean = append_record(lean_contents.as_ref(), lean_marker, &lean_record)?;
    let quint = append_record(quint_contents.as_ref(), quint_marker, &quint_record)?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added {} scenario {} to slice {}",
            scenario.kind.as_str(),
            scenario.name.as_ref(),
            scenario.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_event_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    event: NewEventDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_event_reference = lean_event_reference_record(event.name.as_ref());
    let quint_event_reference = quint_event_reference_record(event.name.as_ref());
    let lean_stream_record = lean_stream_record(event.stream.as_ref());
    let quint_stream_record = quint_stream_record(event.stream.as_ref());
    let lean_event_record = lean_event_definition_record(&event);
    let quint_event_record = quint_event_definition_record(&event);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def sliceEvents : List SliceEventReference := ",
        &lean_event_reference,
    )
    .and_then(|contents| {
        append_record_if_missing(
            &contents,
            "def sliceStreams : List StreamDefinition := ",
            &lean_stream_record,
        )
    })
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "def sliceEventDefinitions : List EventDefinition := ",
            &lean_event_record,
            RecordFlavor::Lean,
            &["stream", "observed", "shared"],
            &[ChildListField {
                key: "attributes",
                mode: ChildMergeMode::Append,
            }],
        )
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val sliceEvents: List[SliceEventReference] = ",
        &quint_event_reference,
    )
    .and_then(|contents| {
        append_record_if_missing(
            &contents,
            "val sliceStreams: List[StreamDefinition] = ",
            &quint_stream_record,
        )
    })
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "val sliceEventDefinitions: List[EventDefinition] = ",
            &quint_event_record,
            RecordFlavor::Quint,
            &["stream", "observed", "shared"],
            &[ChildListField {
                key: "attributes",
                mode: ChildMergeMode::Append,
            }],
        )
    })?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added event {} to slice {}",
            event.name.as_ref(),
            event.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_outcome_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    outcome: NewOutcomeDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_record = lean_outcome_definition_record(&outcome);
    let quint_record = quint_outcome_definition_record(&outcome);
    let lean = append_record(
        lean_contents.as_ref(),
        "def sliceOutcomeDefinitions : List OutcomeDefinition := ",
        &lean_record,
    )?;
    let quint = append_record(
        quint_contents.as_ref(),
        "val sliceOutcomeDefinitions: List[OutcomeDefinition] = ",
        &quint_record,
    )?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added outcome {} to slice {}",
            outcome.label.as_ref(),
            outcome.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_external_payload_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    external_payload: NewExternalPayloadDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_record = lean_external_payload_definition_record(&external_payload);
    let quint_record = quint_external_payload_definition_record(&external_payload);
    let lean = merge_or_append_named_record(
        lean_contents.as_ref(),
        "def sliceExternalPayloads : List ExternalPayloadDefinition := ",
        &lean_record,
        RecordFlavor::Lean,
        &[],
        &[ChildListField {
            key: "fields",
            mode: ChildMergeMode::Append,
        }],
    )?;
    let quint = merge_or_append_named_record(
        quint_contents.as_ref(),
        "val sliceExternalPayloads: List[ExternalPayloadDefinition] = ",
        &quint_record,
        RecordFlavor::Quint,
        &[],
        &[ChildListField {
            key: "fields",
            mode: ChildMergeMode::Append,
        }],
    )?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added external payload {} to slice {}",
            external_payload.name.as_ref(),
            external_payload.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_automation_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    automation: NewAutomationDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_command_reference = lean_command_reference_record(automation.command_name.as_ref());
    let quint_command_reference = quint_command_reference_record(automation.command_name.as_ref());
    let lean = append_record(
        lean_contents.as_ref(),
        "def sliceReferencedCommands : List SliceCommandReference := ",
        &lean_command_reference,
    )
    .and_then(|contents| {
        append_record(
            &contents,
            "def sliceAutomations : List AutomationDefinition := ",
            &lean_automation_definition_record(&automation),
        )
    })?;
    let quint = append_record(
        quint_contents.as_ref(),
        "val sliceReferencedCommands: List[SliceCommandReference] = ",
        &quint_command_reference,
    )
    .and_then(|contents| {
        append_record(
            &contents,
            "val sliceAutomations: List[AutomationDefinition] = ",
            &quint_automation_definition_record(&automation),
        )
    })?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added automation {} to slice {}",
            automation.name.as_ref(),
            automation.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_translation_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    translation: NewTranslationDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_command_reference = lean_command_reference_record(translation.command_name.as_ref());
    let quint_command_reference = quint_command_reference_record(translation.command_name.as_ref());
    let lean = append_record(
        lean_contents.as_ref(),
        "def sliceReferencedCommands : List SliceCommandReference := ",
        &lean_command_reference,
    )
    .and_then(|contents| {
        append_record(
            &contents,
            "def sliceTranslations : List TranslationDefinition := ",
            &lean_translation_definition_record(&translation),
        )
    })?;
    let quint = append_record(
        quint_contents.as_ref(),
        "val sliceReferencedCommands: List[SliceCommandReference] = ",
        &quint_command_reference,
    )
    .and_then(|contents| {
        append_record(
            &contents,
            "val sliceTranslations: List[TranslationDefinition] = ",
            &quint_translation_definition_record(&translation),
        )
    })?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added translation {} to slice {}",
            translation.name.as_ref(),
            translation.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_board_element(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    element: NewBoardElement,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean = append_record(
        lean_contents.as_ref(),
        "def sliceBoardElements : List BoardElement := ",
        &lean_board_element_record(&element),
    )?;
    let quint = append_record(
        quint_contents.as_ref(),
        "val sliceBoardElements: List[BoardElement] = ",
        &quint_board_element_record(&element),
    )?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added board element {} to slice {}",
            element.name.as_ref(),
            element.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_board_connection(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    connection: NewBoardConnection,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean = append_record(
        lean_contents.as_ref(),
        "def sliceBoardConnections : List BoardConnection := ",
        &lean_board_connection_record(&connection),
    )?;
    let quint = append_record(
        quint_contents.as_ref(),
        "val sliceBoardConnections: List[BoardConnection] = ",
        &quint_board_connection_record(&connection),
    )?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added board connection {} -> {} to slice {}",
            connection.source.as_ref(),
            connection.target.as_ref(),
            connection.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_command_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    command: NewCommandDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_command_reference = lean_command_reference_record(command.name.as_ref());
    let quint_command_reference = quint_command_reference_record(command.name.as_ref());
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def sliceCommands : List SliceCommandReference := ",
        &lean_command_reference,
    )
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "def sliceCommandDefinitions : List CommandDefinition := ",
            &lean_command_definition_record(&command),
            RecordFlavor::Lean,
            &["singleton", "repeatBehavior"],
            &COMMAND_CHILD_LIST_FIELDS,
        )
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val sliceCommands: List[SliceCommandReference] = ",
        &quint_command_reference,
    )
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "val sliceCommandDefinitions: List[CommandDefinition] = ",
            &quint_command_definition_record(&command),
            RecordFlavor::Quint,
            &["singleton", "repeatBehavior"],
            &COMMAND_CHILD_LIST_FIELDS,
        )
    })?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added command {} to slice {}",
            command.name.as_ref(),
            command.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_read_model_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    read_model: NewReadModelDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_read_model_reference = lean_read_model_reference_record(read_model.name.as_ref());
    let quint_read_model_reference = quint_read_model_reference_record(read_model.name.as_ref());
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def sliceReadModels : List SliceReadModelReference := ",
        &lean_read_model_reference,
    )
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "def sliceReadModelDefinitions : List ReadModelDefinition := ",
            &lean_read_model_definition_record(&read_model),
            RecordFlavor::Lean,
            &[],
            &[ChildListField {
                key: "fields",
                mode: ChildMergeMode::Append,
            }],
        )
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val sliceReadModels: List[SliceReadModelReference] = ",
        &quint_read_model_reference,
    )
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "val sliceReadModelDefinitions: List[ReadModelDefinition] = ",
            &quint_read_model_definition_record(&read_model),
            RecordFlavor::Quint,
            &[],
            &[ChildListField {
                key: "fields",
                mode: ChildMergeMode::Append,
            }],
        )
    })?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added read model {} to slice {}",
            read_model.name.as_ref(),
            read_model.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_view_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    view: NewViewDefinition,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_view_reference = lean_view_reference_record(view.name.as_ref());
    let quint_view_reference = quint_view_reference_record(view.name.as_ref());
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def sliceViews : List SliceViewReference := ",
        &lean_view_reference,
    )
    .and_then(|contents| {
        append_records(
            &contents,
            "def sliceReferencedCommands : List SliceCommandReference := ",
            &lean_view_referenced_command_records(&view),
        )
    })
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "def sliceViewDefinitions : List ViewDefinition := ",
            &lean_view_definition_record(&view),
            RecordFlavor::Lean,
            &[],
            &VIEW_CHILD_LIST_FIELDS,
        )
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val sliceViews: List[SliceViewReference] = ",
        &quint_view_reference,
    )
    .and_then(|contents| {
        append_records(
            &contents,
            "val sliceReferencedCommands: List[SliceCommandReference] = ",
            &quint_view_referenced_command_records(&view),
        )
    })
    .and_then(|contents| {
        merge_or_append_named_record(
            &contents,
            "val sliceViewDefinitions: List[ViewDefinition] = ",
            &quint_view_definition_record(&view),
            RecordFlavor::Quint,
            &[],
            &VIEW_CHILD_LIST_FIELDS,
        )
    })?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added view {} to slice {}",
            view.name.as_ref(),
            view.slice_slug.as_ref()
        ))?),
    ]))
}

pub(crate) fn add_bit_level_data_flow(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    data_flow: NewBitLevelDataFlow,
) -> Result<EffectPlan, FormalSliceFactError> {
    let lean_record = lean_data_flow_record(&data_flow);
    let quint_record = quint_data_flow_record(&data_flow);
    let lean = append_record(
        lean_contents.as_ref(),
        "def sliceBitLevelDataFlows : List BitLevelDataFlow := ",
        &lean_record,
    )?;
    let quint = append_record(
        quint_contents.as_ref(),
        "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = ",
        &quint_record,
    )?;

    Ok(EffectPlan::new(vec![
        Effect::write_file(lean_path, file_contents(lean)?),
        Effect::write_file(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added bit-level data flow {} to slice {}",
            data_flow.datum.as_ref(),
            data_flow.slice_slug.as_ref()
        ))?),
    ]))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormalSliceFactError {
    message: String,
}

impl FormalSliceFactError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for FormalSliceFactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for FormalSliceFactError {}

fn append_record(
    contents: &str,
    marker: &str,
    record: &str,
) -> Result<String, FormalSliceFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if let Some(current_list) = declaration.strip_prefix(marker) {
                replaced = true;
                Ok(format!(
                    "{indentation}{marker}{}",
                    append_list_record(current_list, record)?
                ))
            } else {
                Ok(line.to_owned())
            }
        })
        .collect::<Result<Vec<_>, FormalSliceFactError>>()?;

    if replaced {
        let mut updated = lines.join("\n");
        if contents.ends_with('\n') {
            updated.push('\n');
        }
        Ok(updated)
    } else {
        Err(FormalSliceFactError::new(format!(
            "formal slice artifact is missing declaration {marker}"
        )))
    }
}

fn append_record_if_missing(
    contents: &str,
    marker: &str,
    record: &str,
) -> Result<String, FormalSliceFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if let Some(current_list) = declaration.strip_prefix(marker) {
                replaced = true;
                Ok(format!(
                    "{indentation}{marker}{}",
                    append_list_record_if_missing(current_list, record)?
                ))
            } else {
                Ok(line.to_owned())
            }
        })
        .collect::<Result<Vec<_>, FormalSliceFactError>>()?;

    if replaced {
        let mut updated = lines.join("\n");
        if contents.ends_with('\n') {
            updated.push('\n');
        }
        Ok(updated)
    } else {
        Err(FormalSliceFactError::new(format!(
            "formal slice artifact is missing declaration {marker}"
        )))
    }
}

fn append_records(
    contents: &str,
    marker: &str,
    records: &[String],
) -> Result<String, FormalSliceFactError> {
    records
        .iter()
        .try_fold(contents.to_owned(), |current, record| {
            append_record(&current, marker, record)
        })
}

fn append_list_record(current_list: &str, record: &str) -> Result<String, FormalSliceFactError> {
    let trimmed = current_list.trim();
    if trimmed == "[]" {
        return Ok(format!("[{record}]"));
    }
    trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .map(|existing| format!("[{existing},{record}]"))
        .ok_or_else(|| FormalSliceFactError::new("formal slice list declaration is malformed"))
}

fn append_list_record_if_missing(
    current_list: &str,
    record: &str,
) -> Result<String, FormalSliceFactError> {
    let trimmed = current_list.trim();
    if trimmed == "[]" {
        return Ok(format!("[{record}]"));
    }
    let existing = trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .ok_or_else(|| FormalSliceFactError::new("formal slice list declaration is malformed"))?;
    if existing
        .split(',')
        .any(|existing_record| existing_record == record)
    {
        Ok(trimmed.to_owned())
    } else {
        Ok(format!("[{existing},{record}]"))
    }
}

/// Whether the artifact uses Lean (`field := value`) or Quint (`field: value`)
/// record syntax. The two differ only in the separator between a field key and
/// its value, so a single merge implementation can serve both by carrying the
/// flavor along.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RecordFlavor {
    Lean,
    Quint,
}

impl RecordFlavor {
    /// The exact byte sequence that separates a field key from its value,
    /// including the surrounding spaces emitted by the record builders.
    fn separator(self) -> &'static str {
        match self {
            Self::Lean => " := ",
            Self::Quint => ": ",
        }
    }
}

/// How a single child collection inside a named definition record should be
/// reconciled when a later same-name `add` call arrives.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ChildMergeMode {
    /// Splice the incoming record's items in unconditionally (allows duplicate
    /// child rows, e.g. command inputs or event attributes which are
    /// intentionally additive per call).
    Append,
    /// Splice the incoming record's items in only when an identical item is not
    /// already present (e.g. a command's emitted events or observed streams,
    /// which a repeated call may legitimately re-state).
    AppendIfMissing,
}

/// A child list field within a named definition record that participates in the
/// merge (its items accumulate across same-name `add` calls).
#[derive(Debug, Clone, Copy)]
struct ChildListField {
    key: &'static str,
    mode: ChildMergeMode,
}

/// Child collections accumulated when a view is authored across several
/// `add view` calls. Each call carries one field (and optionally one control);
/// the read model references, sketch tokens, local states, and filters are
/// re-stated each call, so they are deduplicated while fields and controls
/// accumulate.
const VIEW_CHILD_LIST_FIELDS: [ChildListField; 6] = [
    ChildListField {
        key: "readModels",
        mode: ChildMergeMode::AppendIfMissing,
    },
    ChildListField {
        key: "fields",
        mode: ChildMergeMode::Append,
    },
    ChildListField {
        key: "controls",
        mode: ChildMergeMode::Append,
    },
    ChildListField {
        key: "sketchTokens",
        mode: ChildMergeMode::AppendIfMissing,
    },
    ChildListField {
        key: "localStates",
        mode: ChildMergeMode::AppendIfMissing,
    },
    ChildListField {
        key: "filters",
        mode: ChildMergeMode::AppendIfMissing,
    },
];

/// Child collections accumulated when a `state_change` slice's single command is
/// authored across several `add command` calls: each call carries one input plus
/// optionally repeated emitted events, observed streams, and errors.
const COMMAND_CHILD_LIST_FIELDS: [ChildListField; 4] = [
    ChildListField {
        key: "inputs",
        mode: ChildMergeMode::Append,
    },
    ChildListField {
        key: "emittedEvents",
        mode: ChildMergeMode::AppendIfMissing,
    },
    ChildListField {
        key: "observedStreams",
        mode: ChildMergeMode::AppendIfMissing,
    },
    ChildListField {
        key: "errors",
        mode: ChildMergeMode::AppendIfMissing,
    },
];

/// Merge `new_record` into the list declared by `marker`, accumulating child
/// rows onto the existing same-`name` record rather than appending a duplicate
/// definition.
///
/// `scalar_consistency_keys` names the fields whose values must agree between an
/// existing record and a later same-name call (e.g. an event's `stream`); a
/// disagreement is an authoring error and surfaces as a `FormalSliceFactError`.
/// `child_list_fields` names the collections whose items accumulate.
///
/// When no record with a matching `name` exists yet, the new record is appended
/// verbatim, preserving the original create-new-definition behavior.
fn merge_or_append_named_record(
    contents: &str,
    marker: &str,
    new_record: &str,
    flavor: RecordFlavor,
    scalar_consistency_keys: &[&str],
    child_list_fields: &[ChildListField],
) -> Result<String, FormalSliceFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if let Some(current_list) = declaration.strip_prefix(marker) {
                replaced = true;
                Ok(format!(
                    "{indentation}{marker}{}",
                    merge_list_record(
                        current_list,
                        new_record,
                        flavor,
                        scalar_consistency_keys,
                        child_list_fields,
                    )?
                ))
            } else {
                Ok(line.to_owned())
            }
        })
        .collect::<Result<Vec<_>, FormalSliceFactError>>()?;

    if replaced {
        let mut updated = lines.join("\n");
        if contents.ends_with('\n') {
            updated.push('\n');
        }
        Ok(updated)
    } else {
        Err(FormalSliceFactError::new(format!(
            "formal slice artifact is missing declaration {marker}"
        )))
    }
}

fn merge_list_record(
    current_list: &str,
    new_record: &str,
    flavor: RecordFlavor,
    scalar_consistency_keys: &[&str],
    child_list_fields: &[ChildListField],
) -> Result<String, FormalSliceFactError> {
    let trimmed = current_list.trim();
    if trimmed == "[]" {
        return Ok(format!("[{new_record}]"));
    }
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .ok_or_else(|| FormalSliceFactError::new("formal slice list declaration is malformed"))?;

    let mut records = split_top_level_records(inner)?;
    let new_name = record_field_value(new_record, "name", flavor)
        .ok_or_else(|| FormalSliceFactError::new("formal slice record is missing a name field"))?;

    for existing in &mut records {
        let existing_name = record_field_value(existing, "name", flavor).ok_or_else(|| {
            FormalSliceFactError::new("formal slice record is missing a name field")
        })?;
        if existing_name == new_name {
            *existing = merge_named_records(
                existing,
                new_record,
                flavor,
                scalar_consistency_keys,
                child_list_fields,
            )?;
            return Ok(format!("[{}]", records.join(",")));
        }
    }

    Ok(format!("[{inner},{new_record}]"))
}

/// Splice the child collections of `new_record` onto `existing`, after checking
/// that the scalar consistency fields agree.
fn merge_named_records(
    existing: &str,
    new_record: &str,
    flavor: RecordFlavor,
    scalar_consistency_keys: &[&str],
    child_list_fields: &[ChildListField],
) -> Result<String, FormalSliceFactError> {
    for key in scalar_consistency_keys {
        let existing_value = record_field_value(existing, key, flavor);
        let new_value = record_field_value(new_record, key, flavor);
        if existing_value != new_value {
            return Err(FormalSliceFactError::new(format!(
                "formal slice definition '{}' was added again with a conflicting {key} value",
                record_field_value(existing, "name", flavor).unwrap_or_default(),
            )));
        }
    }

    let mut merged = existing.to_owned();
    for field in child_list_fields {
        let new_items = record_list_field_items(new_record, field.key, flavor)?;
        if new_items.is_empty() {
            continue;
        }
        merged = splice_child_list(&merged, field.key, &new_items, field.mode, flavor)?;
    }
    Ok(merged)
}

/// Return the items currently spliced into the `key` child list of `record`.
fn record_list_field_items(
    record: &str,
    key: &str,
    flavor: RecordFlavor,
) -> Result<Vec<String>, FormalSliceFactError> {
    let value = record_field_value(record, key, flavor).ok_or_else(|| {
        FormalSliceFactError::new(format!("formal slice record is missing field {key}"))
    })?;
    let trimmed = value.trim();
    if trimmed == "[]" {
        return Ok(Vec::new());
    }
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .ok_or_else(|| {
            FormalSliceFactError::new(format!("formal slice record field {key} is not a list"))
        })?;
    split_top_level_records(inner)
}

/// Append `new_items` into the `key` child list of `record`, honoring `mode`.
fn splice_child_list(
    record: &str,
    key: &str,
    new_items: &[String],
    mode: ChildMergeMode,
    flavor: RecordFlavor,
) -> Result<String, FormalSliceFactError> {
    let existing_items = record_list_field_items(record, key, flavor)?;
    let mut combined = existing_items.clone();
    for item in new_items {
        if mode == ChildMergeMode::AppendIfMissing
            && combined.iter().any(|existing| existing == item)
        {
            continue;
        }
        combined.push(item.clone());
    }
    let rebuilt = format!("[{}]", combined.join(","));
    replace_record_field_value(record, key, &rebuilt, flavor)
}

/// Extract the value substring of field `key` from a brace-balanced record body
/// such as `{ name := "X", attributes := [..] }`. Returns the raw value text
/// (trimmed of surrounding whitespace) or `None` when the field is absent.
fn record_field_value(record: &str, key: &str, flavor: RecordFlavor) -> Option<String> {
    let (start, end) = record_field_value_span(record, key, flavor)?;
    Some(record[start..end].trim().to_owned())
}

/// Replace the value of field `key` in `record` with `value`, preserving the
/// rest of the record verbatim.
fn replace_record_field_value(
    record: &str,
    key: &str,
    value: &str,
    flavor: RecordFlavor,
) -> Result<String, FormalSliceFactError> {
    let (start, end) = record_field_value_span(record, key, flavor).ok_or_else(|| {
        FormalSliceFactError::new(format!("formal slice record is missing field {key}"))
    })?;
    let mut rebuilt = String::with_capacity(record.len());
    rebuilt.push_str(&record[..start]);
    rebuilt.push_str(value);
    rebuilt.push_str(&record[end..]);
    Ok(rebuilt)
}

/// Locate the byte span of field `key`'s value within `record`. The start is the
/// first byte after the `key<separator>` token; the end is the byte just before
/// the top-level `,` or closing ` }` that terminates the value. Nested braces
/// and brackets are tracked so values that are themselves records or lists are
/// captured whole.
fn record_field_value_span(
    record: &str,
    key: &str,
    flavor: RecordFlavor,
) -> Option<(usize, usize)> {
    let needle = format!("{key}{}", flavor.separator());
    let mut search_from = 0;
    while let Some(relative) = record[search_from..].find(&needle) {
        let key_start = search_from + relative;
        // Require the match to start at a field boundary ('{' or ',' just
        // before the key, ignoring surrounding spaces) so that a key which is a
        // suffix of a longer key cannot be matched by accident.
        let preceding = record[..key_start].trim_end();
        let at_boundary = preceding.ends_with('{') || preceding.ends_with(',');
        if !at_boundary {
            search_from = key_start + needle.len();
            continue;
        }
        let value_start = key_start + needle.len();
        let mut depth: i32 = 0;
        for (offset, character) in record[value_start..].char_indices() {
            match character {
                '{' | '[' => depth += 1,
                '}' | ']' => {
                    if depth == 0 {
                        // Closing brace of the enclosing record terminates the
                        // final field's value.
                        return Some((value_start, value_start + offset));
                    }
                    depth -= 1;
                }
                ',' if depth == 0 => return Some((value_start, value_start + offset)),
                _ => {}
            }
        }
        return Some((value_start, record.len()));
    }
    None
}

/// Split a comma-separated sequence of brace-balanced records (the inner text of
/// a `[..]` list, with the surrounding brackets already removed) into individual
/// record strings. Commas nested inside `{}` or `[]` are not split points.
fn split_top_level_records(inner: &str) -> Result<Vec<String>, FormalSliceFactError> {
    let mut records = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in inner.chars() {
        if in_string {
            current.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => {
                in_string = true;
                current.push(character);
            }
            '{' | '[' => {
                depth += 1;
                current.push(character);
            }
            '}' | ']' => {
                depth -= 1;
                current.push(character);
            }
            ',' if depth == 0 => {
                records.push(current.trim().to_owned());
                current.clear();
            }
            _ => current.push(character),
        }
    }
    if depth != 0 || in_string {
        return Err(FormalSliceFactError::new(
            "formal slice list declaration is malformed",
        ));
    }
    let trailing = current.trim();
    if !trailing.is_empty() {
        records.push(trailing.to_owned());
    }
    Ok(records)
}

fn lean_scenario_record(scenario: &NewSliceScenario) -> String {
    format!(
        "{{ name := {}, givenSteps := [{}], whenSteps := [{}], thenSteps := [{}], readStreams := [{}], writtenStreams := [{}], contractKind := {}, coveredDefinition := {}, errorReferences := [{}] }}",
        quoted(scenario.name.as_ref()),
        quoted(scenario.given.as_ref()),
        quoted(scenario.when.as_ref()),
        quoted(scenario.then.as_ref()),
        lean_list(scenario.read_streams.as_slice()),
        lean_list(scenario.written_streams.as_slice()),
        quoted(optional_ref(
            scenario
                .contract_kind
                .as_ref()
                .map(ContractKindName::as_ref)
        )),
        quoted(optional_ref(
            scenario
                .covered_definition
                .as_ref()
                .map(CoveredDefinitionName::as_ref)
        )),
        lean_list(scenario.error_references.as_slice()),
    )
}

fn quint_scenario_record(scenario: &NewSliceScenario) -> String {
    format!(
        "{{ name: {}, givenSteps: [{}], whenSteps: [{}], thenSteps: [{}], readStreams: [{}], writtenStreams: [{}], contractKind: {}, coveredDefinition: {}, errorReferences: [{}] }}",
        quoted(scenario.name.as_ref()),
        quoted(scenario.given.as_ref()),
        quoted(scenario.when.as_ref()),
        quoted(scenario.then.as_ref()),
        quint_list(scenario.read_streams.as_slice()),
        quint_list(scenario.written_streams.as_slice()),
        quoted(optional_ref(
            scenario
                .contract_kind
                .as_ref()
                .map(ContractKindName::as_ref)
        )),
        quoted(optional_ref(
            scenario
                .covered_definition
                .as_ref()
                .map(CoveredDefinitionName::as_ref)
        )),
        quint_list(scenario.error_references.as_slice()),
    )
}

fn lean_command_definition_record(command: &NewCommandDefinition) -> String {
    format!(
        "{{ name := {}, inputs := [{}], emittedEvents := [{}], observedStreams := [{}], errors := [{}], singleton := {}, repeatBehavior := {} }}",
        quoted(command.name.as_ref()),
        lean_command_input_record(&command.input),
        lean_event_reference_records(command.emitted_events.as_slice()),
        lean_stream_reference_records(command.observed_streams.as_slice()),
        command
            .errors
            .as_slice()
            .iter()
            .map(lean_command_error_record)
            .collect::<Vec<_>>()
            .join(","),
        command.singleton_repeat_behavior.is_some(),
        quoted(
            command
                .singleton_repeat_behavior
                .as_ref()
                .map_or("", |repeat_behavior| repeat_behavior.as_ref()),
        ),
    )
}

fn quint_command_definition_record(command: &NewCommandDefinition) -> String {
    format!(
        "{{ name: {}, inputs: [{}], emittedEvents: [{}], observedStreams: [{}], errors: [{}], singleton: {}, repeatBehavior: {} }}",
        quoted(command.name.as_ref()),
        quint_command_input_record(&command.input),
        quint_event_reference_records(command.emitted_events.as_slice()),
        quint_stream_reference_records(command.observed_streams.as_slice()),
        command
            .errors
            .as_slice()
            .iter()
            .map(quint_command_error_record)
            .collect::<Vec<_>>()
            .join(","),
        command.singleton_repeat_behavior.is_some(),
        quoted(
            command
                .singleton_repeat_behavior
                .as_ref()
                .map_or("", |repeat_behavior| repeat_behavior.as_ref()),
        ),
    )
}

fn lean_command_error_record(error: &NewCommandErrorDefinition) -> String {
    format!(
        "{{ name := {}, scenarioName := {}, recoveryKind := {} }}",
        quoted(error.name.as_ref()),
        quoted(error.scenario_name.as_ref()),
        quoted(error.recovery_kind.as_ref()),
    )
}

fn quint_command_error_record(error: &NewCommandErrorDefinition) -> String {
    format!(
        "{{ name: {}, scenarioName: {}, recoveryKind: {} }}",
        quoted(error.name.as_ref()),
        quoted(error.scenario_name.as_ref()),
        quoted(error.recovery_kind.as_ref()),
    )
}

fn lean_automation_definition_record(automation: &NewAutomationDefinition) -> String {
    format!(
        "{{ name := {}, triggerName := {}, commandName := {}, handledErrors := [{}], reactionDescription := {} }}",
        quoted(automation.name.as_ref()),
        quoted(automation.trigger_name.as_ref()),
        quoted(automation.command_name.as_ref()),
        lean_list(automation.handled_errors.as_slice()),
        quoted(automation.reaction_description.as_ref()),
    )
}

fn quint_automation_definition_record(automation: &NewAutomationDefinition) -> String {
    format!(
        "{{ name: {}, triggerName: {}, commandName: {}, handledErrors: [{}], reactionDescription: {} }}",
        quoted(automation.name.as_ref()),
        quoted(automation.trigger_name.as_ref()),
        quoted(automation.command_name.as_ref()),
        quint_list(automation.handled_errors.as_slice()),
        quoted(automation.reaction_description.as_ref()),
    )
}

fn lean_translation_definition_record(translation: &NewTranslationDefinition) -> String {
    format!(
        "{{ name := {}, externalEventName := {}, payloadContractName := {}, commandName := {} }}",
        quoted(translation.name.as_ref()),
        quoted(translation.external_event_name.as_ref()),
        quoted(translation.payload_contract_name.as_ref()),
        quoted(translation.command_name.as_ref()),
    )
}

fn quint_translation_definition_record(translation: &NewTranslationDefinition) -> String {
    format!(
        "{{ name: {}, externalEventName: {}, payloadContractName: {}, commandName: {} }}",
        quoted(translation.name.as_ref()),
        quoted(translation.external_event_name.as_ref()),
        quoted(translation.payload_contract_name.as_ref()),
        quoted(translation.command_name.as_ref()),
    )
}

fn lean_board_element_record(element: &NewBoardElement) -> String {
    format!(
        "{{ name := {}, kind := {}, lane := {}, declaredName := {}, mainPath := {} }}",
        quoted(element.name.as_ref()),
        quoted(element.kind.as_ref()),
        quoted(element.lane.as_ref()),
        quoted(element.declared_name.as_ref()),
        lean_bool(element.main_path),
    )
}

fn quint_board_element_record(element: &NewBoardElement) -> String {
    format!(
        "{{ name: {}, kind: {}, lane: {}, declaredName: {}, mainPath: {} }}",
        quoted(element.name.as_ref()),
        quoted(element.kind.as_ref()),
        quoted(element.lane.as_ref()),
        quoted(element.declared_name.as_ref()),
        lean_bool(element.main_path),
    )
}

fn lean_board_connection_record(connection: &NewBoardConnection) -> String {
    format!(
        "{{ source := {}, sourceKind := {}, target := {}, targetKind := {} }}",
        quoted(connection.source.as_ref()),
        quoted(connection.source_kind.as_ref()),
        quoted(connection.target.as_ref()),
        quoted(connection.target_kind.as_ref()),
    )
}

fn quint_board_connection_record(connection: &NewBoardConnection) -> String {
    format!(
        "{{ source: {}, sourceKind: {}, target: {}, targetKind: {} }}",
        quoted(connection.source.as_ref()),
        quoted(connection.source_kind.as_ref()),
        quoted(connection.target.as_ref()),
        quoted(connection.target_kind.as_ref()),
    )
}

fn lean_outcome_definition_record(outcome: &NewOutcomeDefinition) -> String {
    format!(
        "{{ label := {}, eventSet := [{}], externallyRelevant := {} }}",
        quoted(outcome.label.as_ref()),
        lean_list(outcome.event_set.as_slice()),
        lean_bool(outcome.externally_relevant),
    )
}

fn quint_outcome_definition_record(outcome: &NewOutcomeDefinition) -> String {
    format!(
        "{{ label: {}, eventSet: [{}], externallyRelevant: {} }}",
        quoted(outcome.label.as_ref()),
        quint_list(outcome.event_set.as_slice()),
        outcome.externally_relevant,
    )
}

fn lean_stream_record(stream: &str) -> String {
    format!("{{ name := {} }}", quoted(stream))
}

fn quint_stream_record(stream: &str) -> String {
    format!("{{ name: {} }}", quoted(stream))
}

fn lean_stream_reference_record(stream_name: &str) -> String {
    format!("{{ name := {} }}", quoted(stream_name))
}

fn quint_stream_reference_record(stream_name: &str) -> String {
    format!("{{ name: {} }}", quoted(stream_name))
}

fn lean_event_reference_record(event_name: &str) -> String {
    format!("{{ name := {} }}", quoted(event_name))
}

fn quint_event_reference_record(event_name: &str) -> String {
    format!("{{ name: {} }}", quoted(event_name))
}

fn lean_event_reference_records(event_names: &[EventName]) -> String {
    event_names
        .iter()
        .map(|event_name| lean_event_reference_record(event_name.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn quint_event_reference_records(event_names: &[EventName]) -> String {
    event_names
        .iter()
        .map(|event_name| quint_event_reference_record(event_name.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn lean_stream_reference_records(stream_names: &[StreamName]) -> String {
    stream_names
        .iter()
        .map(|stream_name| lean_stream_reference_record(stream_name.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn quint_stream_reference_records(stream_names: &[StreamName]) -> String {
    stream_names
        .iter()
        .map(|stream_name| quint_stream_reference_record(stream_name.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn lean_command_reference_record(command_name: &str) -> String {
    format!("{{ name := {} }}", quoted(command_name))
}

fn quint_command_reference_record(command_name: &str) -> String {
    format!("{{ name: {} }}", quoted(command_name))
}

fn lean_read_model_reference_record(read_model_name: &str) -> String {
    format!("{{ name := {} }}", quoted(read_model_name))
}

fn quint_read_model_reference_record(read_model_name: &str) -> String {
    format!("{{ name: {} }}", quoted(read_model_name))
}

fn lean_view_reference_record(view_name: &str) -> String {
    format!("{{ name := {} }}", quoted(view_name))
}

fn quint_view_reference_record(view_name: &str) -> String {
    format!("{{ name: {} }}", quoted(view_name))
}

fn lean_external_payload_definition_record(
    external_payload: &NewExternalPayloadDefinition,
) -> String {
    format!(
        "{{ name := {}, fields := [{{ name := {}, provenanceDescription := {}, bitEncoding := {} }}] }}",
        quoted(external_payload.name.as_ref()),
        quoted(external_payload.field.as_ref()),
        quoted(external_payload.field_provenance.as_ref()),
        quoted(external_payload.bit_encoding.as_ref()),
    )
}

fn quint_external_payload_definition_record(
    external_payload: &NewExternalPayloadDefinition,
) -> String {
    format!(
        "{{ name: {}, fields: [{{ name: {}, provenanceDescription: {}, bitEncoding: {} }}] }}",
        quoted(external_payload.name.as_ref()),
        quoted(external_payload.field.as_ref()),
        quoted(external_payload.field_provenance.as_ref()),
        quoted(external_payload.bit_encoding.as_ref()),
    )
}

fn lean_event_definition_record(event: &NewEventDefinition) -> String {
    format!(
        "{{ name := {}, stream := {}, attributes := [{}], observed := {}, shared := {} }}",
        quoted(event.name.as_ref()),
        quoted(event.stream.as_ref()),
        lean_event_attribute_record(&event.attribute),
        lean_bool(event.observed),
        lean_bool(event.shared),
    )
}

fn quint_event_definition_record(event: &NewEventDefinition) -> String {
    format!(
        "{{ name: {}, stream: {}, attributes: [{}], observed: {}, shared: {} }}",
        quoted(event.name.as_ref()),
        quoted(event.stream.as_ref()),
        quint_event_attribute_record(&event.attribute),
        event.observed,
        event.shared,
    )
}

fn lean_event_attribute_record(attribute: &NewEventAttribute) -> String {
    format!(
        "{{ name := {}, sourceKind := {}, sourceName := {}, sourceField := {}, generatedSourceKind := {}, provenanceDescription := {} }}",
        quoted(attribute.name.as_ref()),
        quoted(attribute.source_kind.as_ref()),
        quoted(attribute.source_name.as_ref()),
        quoted(attribute.source_field.as_ref()),
        quoted(
            attribute
                .generated_source_kind
                .as_ref()
                .map_or("", GeneratedEventAttributeSourceKind::as_ref),
        ),
        quoted(attribute.provenance_description.as_ref()),
    )
}

fn quint_event_attribute_record(attribute: &NewEventAttribute) -> String {
    format!(
        "{{ name: {}, sourceKind: {}, sourceName: {}, sourceField: {}, generatedSourceKind: {}, provenanceDescription: {} }}",
        quoted(attribute.name.as_ref()),
        quoted(attribute.source_kind.as_ref()),
        quoted(attribute.source_name.as_ref()),
        quoted(attribute.source_field.as_ref()),
        quoted(
            attribute
                .generated_source_kind
                .as_ref()
                .map_or("", GeneratedEventAttributeSourceKind::as_ref),
        ),
        quoted(attribute.provenance_description.as_ref()),
    )
}

fn lean_read_model_definition_record(read_model: &NewReadModelDefinition) -> String {
    format!(
        "{{ name := {}, fields := [{}], transitive := {}, relationshipFields := [{}], transitiveRule := {}, exampleScenarioName := {} }}",
        quoted(read_model.name.as_ref()),
        lean_read_model_field_record(&read_model.field),
        lean_bool(read_model.transitive),
        lean_list(read_model.relationship_fields.as_slice()),
        quoted(
            read_model
                .transitive_rule
                .as_ref()
                .map_or("", ReadModelTransitiveRule::as_ref),
        ),
        quoted(
            read_model
                .example_scenario_name
                .as_ref()
                .map_or("", ScenarioName::as_ref),
        ),
    )
}

fn quint_read_model_definition_record(read_model: &NewReadModelDefinition) -> String {
    format!(
        "{{ name: {}, fields: [{}], transitive: {}, relationshipFields: [{}], transitiveRule: {}, exampleScenarioName: {} }}",
        quoted(read_model.name.as_ref()),
        quint_read_model_field_record(&read_model.field),
        read_model.transitive,
        quint_list(read_model.relationship_fields.as_slice()),
        quoted(
            read_model
                .transitive_rule
                .as_ref()
                .map_or("", ReadModelTransitiveRule::as_ref),
        ),
        quoted(
            read_model
                .example_scenario_name
                .as_ref()
                .map_or("", ScenarioName::as_ref),
        ),
    )
}

fn lean_read_model_field_record(field: &NewReadModelField) -> String {
    format!(
        "{{ name := {}, sourceKind := {}, sourceEvent := {}, sourceAttribute := {}, derivationRule := {}, derivationSourceFields := [{}], absenceEvent := {}, derivationScenarioName := {}, absenceScenarioName := {}, provenanceDescription := {} }}",
        quoted(field.name.as_ref()),
        quoted(field.source_kind().as_ref()),
        quoted(field.source_event().map_or("", EventName::as_ref)),
        quoted(
            field
                .source_attribute()
                .map_or("", EventAttributeName::as_ref),
        ),
        quoted(
            field
                .derivation_rule()
                .map_or("", ReadModelDerivationRule::as_ref),
        ),
        lean_list(field.derivation_source_fields().as_slice()),
        quoted(field.absence_event().map_or("", EventName::as_ref)),
        quoted(
            field
                .derivation_scenario_name()
                .map_or("", ScenarioName::as_ref),
        ),
        quoted(
            field
                .absence_scenario_name()
                .map_or("", ScenarioName::as_ref),
        ),
        quoted(field.provenance_description.as_ref()),
    )
}

fn quint_read_model_field_record(field: &NewReadModelField) -> String {
    format!(
        "{{ name: {}, sourceKind: {}, sourceEvent: {}, sourceAttribute: {}, derivationRule: {}, derivationSourceFields: [{}], absenceEvent: {}, derivationScenarioName: {}, absenceScenarioName: {}, provenanceDescription: {} }}",
        quoted(field.name.as_ref()),
        quoted(field.source_kind().as_ref()),
        quoted(field.source_event().map_or("", EventName::as_ref)),
        quoted(
            field
                .source_attribute()
                .map_or("", EventAttributeName::as_ref),
        ),
        quoted(
            field
                .derivation_rule()
                .map_or("", ReadModelDerivationRule::as_ref),
        ),
        quint_list(field.derivation_source_fields().as_slice()),
        quoted(field.absence_event().map_or("", EventName::as_ref)),
        quoted(
            field
                .derivation_scenario_name()
                .map_or("", ScenarioName::as_ref),
        ),
        quoted(
            field
                .absence_scenario_name()
                .map_or("", ScenarioName::as_ref),
        ),
        quoted(field.provenance_description.as_ref()),
    )
}

fn lean_view_definition_record(view: &NewViewDefinition) -> String {
    format!(
        "{{ name := {}, readModels := [{}], fields := [{}], controls := [{}], sketchTokens := [{}], localStates := [{}], filters := [{}] }}",
        quoted(view.name.as_ref()),
        quoted(view.field.source_read_model.as_ref()),
        lean_view_field_record(&view.field),
        view.controls
            .as_slice()
            .iter()
            .map(lean_control_definition_record)
            .collect::<Vec<_>>()
            .join(","),
        lean_list(&view_sketch_tokens(view)),
        lean_list(view.local_states.as_slice()),
        lean_list(view.filters.as_slice()),
    )
}

fn quint_view_definition_record(view: &NewViewDefinition) -> String {
    format!(
        "{{ name: {}, readModels: [{}], fields: [{}], controls: [{}], sketchTokens: [{}], localStates: [{}], filters: [{}] }}",
        quoted(view.name.as_ref()),
        quoted(view.field.source_read_model.as_ref()),
        quint_view_field_record(&view.field),
        view.controls
            .as_slice()
            .iter()
            .map(quint_control_definition_record)
            .collect::<Vec<_>>()
            .join(","),
        quint_list(&view_sketch_tokens(view)),
        quint_list(view.local_states.as_slice()),
        quint_list(view.filters.as_slice()),
    )
}

fn lean_view_referenced_command_records(view: &NewViewDefinition) -> Vec<String> {
    view.controls
        .as_slice()
        .iter()
        .map(|control| lean_command_reference_record(control.command_name.as_ref()))
        .collect()
}

fn quint_view_referenced_command_records(view: &NewViewDefinition) -> Vec<String> {
    view.controls
        .as_slice()
        .iter()
        .map(|control| quint_command_reference_record(control.command_name.as_ref()))
        .collect()
}

fn view_sketch_tokens(view: &NewViewDefinition) -> Vec<&SketchToken> {
    let mut tokens = vec![&view.field.sketch_token];
    for control in view.controls.as_slice() {
        tokens.push(&control.sketch_token);
        tokens.push(&control.input.sketch_token);
    }
    tokens
}

fn lean_control_definition_record(control: &NewControlDefinition) -> String {
    format!(
        "{{ name := {}, commandName := {}, inputs := [{}], handledErrors := [{}], recoveryBehavior := {}, sketchToken := {}, navigation := {} }}",
        quoted(control.name.as_ref()),
        quoted(control.command_name.as_ref()),
        lean_control_input_record(&control.input),
        lean_list(control.handled_errors.as_slice()),
        quoted(control.recovery_behavior.as_ref()),
        quoted(control.sketch_token.as_ref()),
        lean_navigation_target_record(&control.navigation),
    )
}

fn quint_control_definition_record(control: &NewControlDefinition) -> String {
    format!(
        "{{ name: {}, commandName: {}, inputs: [{}], handledErrors: [{}], recoveryBehavior: {}, sketchToken: {}, navigation: {} }}",
        quoted(control.name.as_ref()),
        quoted(control.command_name.as_ref()),
        quint_control_input_record(&control.input),
        quint_list(control.handled_errors.as_slice()),
        quoted(control.recovery_behavior.as_ref()),
        quoted(control.sketch_token.as_ref()),
        quint_navigation_target_record(&control.navigation),
    )
}

fn lean_control_input_record(input: &NewControlInputProvision) -> String {
    format!(
        "{{ name := {}, sourceKind := {}, sourceDescription := {}, sketchToken := {}, visibleToActor := {}, decisionField := {} }}",
        quoted(input.name.as_ref()),
        lean_command_input_source_kind(*input.source_kind()),
        quoted(input.source_description.as_ref()),
        quoted(input.sketch_token.as_ref()),
        lean_bool(input.visible_to_actor),
        lean_bool(input.decision_field),
    )
}

fn quint_control_input_record(input: &NewControlInputProvision) -> String {
    format!(
        "{{ name: {}, sourceKind: {}, sourceDescription: {}, sketchToken: {}, visibleToActor: {}, decisionField: {} }}",
        quoted(input.name.as_ref()),
        quint_command_input_source_kind(*input.source_kind()),
        quoted(input.source_description.as_ref()),
        quoted(input.sketch_token.as_ref()),
        lean_bool(input.visible_to_actor),
        lean_bool(input.decision_field),
    )
}

fn lean_navigation_target_record(navigation: &NewNavigationTarget) -> String {
    format!(
        "{{ targetType := {}, targetName := {}, externalWorkflowName := {}, externalSystemName := {}, handoffContract := {} }}",
        quoted(navigation.target_type.as_ref()),
        quoted(navigation.target_name.as_ref()),
        quoted(
            navigation
                .external_workflow_name
                .as_ref()
                .map_or("", NavigationTargetName::as_ref),
        ),
        quoted(
            navigation
                .external_system_name
                .as_ref()
                .map_or("", NavigationTargetName::as_ref),
        ),
        quoted(
            navigation
                .handoff_contract
                .as_ref()
                .map_or("", PayloadContractName::as_ref),
        ),
    )
}

fn quint_navigation_target_record(navigation: &NewNavigationTarget) -> String {
    format!(
        "{{ targetType: {}, targetName: {}, externalWorkflowName: {}, externalSystemName: {}, handoffContract: {} }}",
        quoted(navigation.target_type.as_ref()),
        quoted(navigation.target_name.as_ref()),
        quoted(
            navigation
                .external_workflow_name
                .as_ref()
                .map_or("", NavigationTargetName::as_ref),
        ),
        quoted(
            navigation
                .external_system_name
                .as_ref()
                .map_or("", NavigationTargetName::as_ref),
        ),
        quoted(
            navigation
                .handoff_contract
                .as_ref()
                .map_or("", PayloadContractName::as_ref),
        ),
    )
}

fn lean_view_field_record(field: &NewViewField) -> String {
    format!(
        "{{ name := {}, sourceKind := {}, sourceReadModel := {}, sourceField := {}, sketchToken := {}, provenanceDescription := {}, bitEncoding := {} }}",
        quoted(field.name.as_ref()),
        quoted(field.source_kind.as_ref()),
        quoted(field.source_read_model.as_ref()),
        quoted(field.source_field.as_ref()),
        quoted(field.sketch_token.as_ref()),
        quoted(field.provenance_description.as_ref()),
        quoted(field.bit_encoding.as_ref()),
    )
}

fn quint_view_field_record(field: &NewViewField) -> String {
    format!(
        "{{ name: {}, sourceKind: {}, sourceReadModel: {}, sourceField: {}, sketchToken: {}, provenanceDescription: {}, bitEncoding: {} }}",
        quoted(field.name.as_ref()),
        quoted(field.source_kind.as_ref()),
        quoted(field.source_read_model.as_ref()),
        quoted(field.source_field.as_ref()),
        quoted(field.sketch_token.as_ref()),
        quoted(field.provenance_description.as_ref()),
        quoted(field.bit_encoding.as_ref()),
    )
}

fn lean_command_input_record(input: &NewCommandInput) -> String {
    format!(
        "{{ name := {}, sourceKind := {}, sourceDescription := {}, provenanceChain := [{}], eventStreamSourceEvent := {}, eventStreamSourceAttribute := {}, externalPayloadSourceName := {}, externalPayloadSourceField := {}, generatedSourceName := {}, generatedSourceField := {}, sessionSourceName := {}, sessionSourceField := {}, invocationArgumentSourceName := {}, invocationArgumentSourceField := {} }}",
        quoted(input.name.as_ref()),
        lean_command_input_source_kind(input.source_kind()),
        quoted(input.source_description.as_ref()),
        lean_list(input.provenance_chain.as_slice()),
        quoted(
            input
                .event_stream_source_event()
                .map_or("", EventName::as_ref),
        ),
        quoted(
            input
                .event_stream_source_attribute()
                .map_or("", EventAttributeName::as_ref),
        ),
        quoted(
            input
                .external_payload_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .external_payload_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
        quoted(
            input
                .generated_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .generated_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
        quoted(
            input
                .session_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .session_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
        quoted(
            input
                .invocation_argument_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .invocation_argument_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
    )
}

fn quint_command_input_record(input: &NewCommandInput) -> String {
    format!(
        "{{ name: {}, sourceKind: {}, sourceDescription: {}, provenanceChain: [{}], eventStreamSourceEvent: {}, eventStreamSourceAttribute: {}, externalPayloadSourceName: {}, externalPayloadSourceField: {}, generatedSourceName: {}, generatedSourceField: {}, sessionSourceName: {}, sessionSourceField: {}, invocationArgumentSourceName: {}, invocationArgumentSourceField: {} }}",
        quoted(input.name.as_ref()),
        quint_command_input_source_kind(input.source_kind()),
        quoted(input.source_description.as_ref()),
        quint_list(input.provenance_chain.as_slice()),
        quoted(
            input
                .event_stream_source_event()
                .map_or("", EventName::as_ref),
        ),
        quoted(
            input
                .event_stream_source_attribute()
                .map_or("", EventAttributeName::as_ref),
        ),
        quoted(
            input
                .external_payload_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .external_payload_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
        quoted(
            input
                .generated_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .generated_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
        quoted(
            input
                .session_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .session_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
        quoted(
            input
                .invocation_argument_source_name()
                .map_or("", EventAttributeSourceName::as_ref),
        ),
        quoted(
            input
                .invocation_argument_source_field()
                .map_or("", EventAttributeSourceField::as_ref),
        ),
    )
}

fn lean_list<T: AsRef<str>>(values: &[T]) -> String {
    values
        .iter()
        .map(|value| quoted(value.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn quint_list<T: AsRef<str>>(values: &[T]) -> String {
    lean_list(values)
}

fn lean_data_flow_record(data_flow: &NewBitLevelDataFlow) -> String {
    format!(
        "{{ datum := {}, sourceKind := {}, source := {}, transformationSemantics := {}, target := {}, bitEncoding := {} }}",
        quoted(data_flow.datum.as_ref()),
        quoted(data_flow.source_kind.as_ref()),
        quoted(data_flow.source.as_ref()),
        quoted(data_flow.transformation.as_ref()),
        quoted(data_flow.target.as_ref()),
        quoted(data_flow.bit_encoding.as_ref()),
    )
}

fn quint_data_flow_record(data_flow: &NewBitLevelDataFlow) -> String {
    format!(
        "{{ datum: {}, sourceKind: {}, source: {}, transformationSemantics: {}, target: {}, bitEncoding: {} }}",
        quoted(data_flow.datum.as_ref()),
        quoted(data_flow.source_kind.as_ref()),
        quoted(data_flow.source.as_ref()),
        quoted(data_flow.transformation.as_ref()),
        quoted(data_flow.target.as_ref()),
        quoted(data_flow.bit_encoding.as_ref()),
    )
}

fn optional_ref(value: Option<&str>) -> &str {
    value.unwrap_or("")
}

fn lean_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated formal slice string literal must be valid: {error}");
    })
}

fn file_contents(value: String) -> Result<FileContents, FormalSliceFactError> {
    FileContents::try_new(value).map_err(|error| FormalSliceFactError::new(error.to_string()))
}

fn report_line(value: String) -> Result<ReportLine, FormalSliceFactError> {
    ReportLine::try_new(value).map_err(|error| FormalSliceFactError::new(error.to_string()))
}
