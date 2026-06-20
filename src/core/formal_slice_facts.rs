// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

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

    pub fn try_new(value: &str) -> Result<Self, ScenarioKindError> {
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
    fn new(value: &str) -> Self {
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

    pub(crate) fn with_updated_control(mut self, control: NewControlDefinition) -> Self {
        self.controls = self.controls.with_updated_control(control);
        self
    }

    pub(crate) fn with_removed_control(mut self, name: &ControlName) -> Self {
        self.controls = self.controls.with_removed_control(name);
        self
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

    fn with_updated_control(mut self, control: NewControlDefinition) -> Self {
        self.controls
            .retain(|existing| existing.name() != control.name());
        self.controls.push(control);
        self
    }

    fn with_removed_control(mut self, name: &ControlName) -> Self {
        self.controls.retain(|control| control.name() != name);
        self
    }
}

/// Whether the artifact uses Lean (`field := value`) or Quint (`field: value`)
/// record syntax. The two differ only in the separator between a field key and
/// its value, so a single merge implementation can serve both by carrying the
/// flavor along.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RecordFlavor {
    Lean,
    Quint,
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

/// Render a definition list body (`[..]`) by rendering every item in order — the
/// pure equivalent of folding `append_record` over the items (used for the lists
/// that append unconditionally).
fn render_append_list<T>(items: &[T], render_record: impl Fn(&T) -> String) -> String {
    let records = items
        .iter()
        .map(render_record)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{records}]")
}

/// Render a definition list body keeping only the first occurrence of each
/// distinct rendered record — the pure equivalent of folding
/// `append_record_if_missing` over the items (used for the reference lists).
fn render_dedup_list<T>(items: &[T], render_record: impl Fn(&T) -> String) -> String {
    let mut records: Vec<String> = Vec::new();
    for item in items {
        let record = render_record(item);
        if !records.contains(&record) {
            records.push(record);
        }
    }
    format!("[{}]", records.join(","))
}

/// Group `items` by name, preserving the first-seen order of both the groups and
/// the members within each group — the ordering the incremental merge builder
/// produces (a new same-name record merges into the first occurrence; a new name
/// appends a fresh group). The pure merge renderers union each group's child
/// lists in this order.
fn group_named<'a, T>(items: &'a [T], name_of: impl Fn(&'a T) -> &'a str) -> Vec<Vec<&'a T>> {
    let mut groups: Vec<Vec<&'a T>> = Vec::new();
    for item in items {
        if let Some(group) = groups.iter_mut().find(|group| {
            group
                .first()
                .is_some_and(|first| name_of(*first) == name_of(item))
        }) {
            group.push(item);
        } else {
            groups.push(vec![item]);
        }
    }
    groups
}

/// Render the whole `sliceEventDefinitions` list body (`[..]`) directly from the
/// projected event facts, byte-identical to folding them through the incremental
/// `merge_or_append_named_record` builder — but without ever parsing artifact
/// text. Events sharing a name are merged into one definition whose `attributes`
/// list is the in-order union of the members' attributes (the `Append` child
/// mode), with `stream`/`observed`/`shared` taken from the first member (the
/// builder enforces those agree).
fn render_slice_event_definitions(events: &[NewEventDefinition], flavor: RecordFlavor) -> String {
    let mut groups: Vec<(&NewEventDefinition, Vec<&NewEventAttribute>)> = Vec::new();
    for event in events {
        if let Some(group) = groups
            .iter_mut()
            .find(|(first, _)| first.name.as_ref() == event.name.as_ref())
        {
            group.1.push(&event.attribute);
        } else {
            groups.push((event, vec![&event.attribute]));
        }
    }
    let records = groups
        .iter()
        .map(|(first, attributes)| render_event_definition_record(first, attributes, flavor))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{records}]")
}

fn render_event_definition_record(
    event: &NewEventDefinition,
    attributes: &[&NewEventAttribute],
    flavor: RecordFlavor,
) -> String {
    let attribute_records = attributes
        .iter()
        .map(|attribute| match flavor {
            RecordFlavor::Lean => lean_event_attribute_record(attribute),
            RecordFlavor::Quint => quint_event_attribute_record(attribute),
        })
        .collect::<Vec<_>>()
        .join(",");
    match flavor {
        RecordFlavor::Lean => format!(
            "{{ name := {}, stream := {}, attributes := [{}], observed := {}, shared := {} }}",
            quoted(event.name.as_ref()),
            quoted(event.stream.as_ref()),
            attribute_records,
            lean_bool(event.observed),
            lean_bool(event.shared),
        ),
        RecordFlavor::Quint => format!(
            "{{ name: {}, stream: {}, attributes: [{}], observed: {}, shared: {} }}",
            quoted(event.name.as_ref()),
            quoted(event.stream.as_ref()),
            attribute_records,
            event.observed,
            event.shared,
        ),
    }
}

/// Render the whole `sliceReadModelDefinitions` list body directly from the
/// projected facts, byte-identical to folding them through
/// `merge_or_append_named_record` (child list `fields` = `Append`; the scalar
/// fields are taken from the first member, which the builder enforces agree).
/// Never parses artifact text.
fn render_slice_read_model_definitions(
    read_models: &[NewReadModelDefinition],
    flavor: RecordFlavor,
) -> String {
    let records = group_named(read_models, |read_model| read_model.name.as_ref())
        .iter()
        .map(|group| render_read_model_definition_record(group, flavor))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{records}]")
}

fn render_read_model_definition_record(
    group: &[&NewReadModelDefinition],
    flavor: RecordFlavor,
) -> String {
    // `group_named` never yields an empty group; an empty slice renders nothing
    // rather than panicking.
    let Some(&first) = group.first() else {
        return String::new();
    };
    let fields = group
        .iter()
        .map(|read_model| match flavor {
            RecordFlavor::Lean => lean_read_model_field_record(&read_model.field),
            RecordFlavor::Quint => quint_read_model_field_record(&read_model.field),
        })
        .collect::<Vec<_>>()
        .join(",");
    let transitive_rule = quoted(
        first
            .transitive_rule
            .as_ref()
            .map_or("", ReadModelTransitiveRule::as_ref),
    );
    let example_scenario_name = quoted(
        first
            .example_scenario_name
            .as_ref()
            .map_or("", ScenarioName::as_ref),
    );
    match flavor {
        RecordFlavor::Lean => format!(
            "{{ name := {}, fields := [{}], transitive := {}, relationshipFields := [{}], transitiveRule := {}, exampleScenarioName := {} }}",
            quoted(first.name.as_ref()),
            fields,
            lean_bool(first.transitive),
            lean_list(first.relationship_fields.as_slice()),
            transitive_rule,
            example_scenario_name,
        ),
        RecordFlavor::Quint => format!(
            "{{ name: {}, fields: [{}], transitive: {}, relationshipFields: [{}], transitiveRule: {}, exampleScenarioName: {} }}",
            quoted(first.name.as_ref()),
            fields,
            first.transitive,
            quint_list(first.relationship_fields.as_slice()),
            transitive_rule,
            example_scenario_name,
        ),
    }
}

/// Render the whole `sliceExternalPayloads` list body directly from the projected
/// facts, byte-identical to folding them through `merge_or_append_named_record`
/// (child list `fields` = `Append`; group by name). Never parses artifact text.
fn render_slice_external_payload_definitions(
    payloads: &[NewExternalPayloadDefinition],
    flavor: RecordFlavor,
) -> String {
    let records = group_named(payloads, |payload| payload.name.as_ref())
        .iter()
        .map(|group| render_external_payload_definition_record(group, flavor))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{records}]")
}

fn render_external_payload_definition_record(
    group: &[&NewExternalPayloadDefinition],
    flavor: RecordFlavor,
) -> String {
    // `group_named` never yields an empty group; an empty slice renders nothing
    // rather than panicking.
    let Some(&first) = group.first() else {
        return String::new();
    };
    let fields = group
        .iter()
        .map(|payload| external_payload_field_record(payload, flavor))
        .collect::<Vec<_>>()
        .join(",");
    let name = quoted(first.name.as_ref());
    // `fields` is the record's final field, so its value span runs up to the
    // closing `}` and includes the space before it. The incremental builder
    // overwrites that span (and thus the space) when it splices a second field
    // into an existing record, so a merged record closes `]}` while a
    // freshly-appended single-field record keeps the rendered `] }`. Reproduce
    // that exactly for byte-identity with on-disk artifacts.
    let closing = if group.len() > 1 { "]}" } else { "] }" };
    match flavor {
        RecordFlavor::Lean => format!("{{ name := {name}, fields := [{fields}{closing}"),
        RecordFlavor::Quint => format!("{{ name: {name}, fields: [{fields}{closing}"),
    }
}

fn external_payload_field_record(
    payload: &NewExternalPayloadDefinition,
    flavor: RecordFlavor,
) -> String {
    match flavor {
        RecordFlavor::Lean => format!(
            "{{ name := {}, provenanceDescription := {}, bitEncoding := {} }}",
            quoted(payload.field.as_ref()),
            quoted(payload.field_provenance.as_ref()),
            quoted(payload.bit_encoding.as_ref()),
        ),
        RecordFlavor::Quint => format!(
            "{{ name: {}, provenanceDescription: {}, bitEncoding: {} }}",
            quoted(payload.field.as_ref()),
            quoted(payload.field_provenance.as_ref()),
            quoted(payload.bit_encoding.as_ref()),
        ),
    }
}

/// Union the child rows of an `AppendIfMissing` list across a name group the way
/// the incremental builder does: the first member's rows are taken verbatim (the
/// record is appended whole), then each later member's rows are appended only
/// when not already present.
fn union_if_missing<T>(group: &[&T], render_rows: impl Fn(&T) -> Vec<String>) -> Vec<String> {
    let mut combined: Vec<String> = Vec::new();
    for (index, member) in group.iter().enumerate() {
        let rows = render_rows(member);
        if index == 0 {
            combined.extend(rows);
        } else {
            for row in rows {
                if !combined.contains(&row) {
                    combined.push(row);
                }
            }
        }
    }
    combined
}

/// Render the whole `sliceCommandDefinitions` list body directly from the
/// projected facts, byte-identical to folding them through
/// `merge_or_append_named_record` (child lists per `COMMAND_CHILD_LIST_FIELDS`:
/// `inputs` accumulate, `emittedEvents`/`observedStreams`/`errors` union by
/// identity; `singleton`/`repeatBehavior` come from the first member). Never
/// parses artifact text.
fn render_slice_command_definitions(
    commands: &[NewCommandDefinition],
    flavor: RecordFlavor,
) -> String {
    let records = group_named(commands, |command| command.name.as_ref())
        .iter()
        .map(|group| render_command_definition_record(group, flavor))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{records}]")
}

fn render_command_definition_record(
    group: &[&NewCommandDefinition],
    flavor: RecordFlavor,
) -> String {
    // `group_named` never yields an empty group; an empty slice renders nothing
    // rather than panicking.
    let Some(&first) = group.first() else {
        return String::new();
    };
    let inputs = group
        .iter()
        .map(|command| match flavor {
            RecordFlavor::Lean => lean_command_input_record(&command.input),
            RecordFlavor::Quint => quint_command_input_record(&command.input),
        })
        .collect::<Vec<_>>()
        .join(",");
    let emitted_events = union_if_missing(group, |command| {
        command
            .emitted_events
            .as_slice()
            .iter()
            .map(|event| match flavor {
                RecordFlavor::Lean => lean_event_reference_record(event.as_ref()),
                RecordFlavor::Quint => quint_event_reference_record(event.as_ref()),
            })
            .collect()
    })
    .join(",");
    let observed_streams = union_if_missing(group, |command| {
        command
            .observed_streams
            .as_slice()
            .iter()
            .map(|stream| match flavor {
                RecordFlavor::Lean => lean_stream_reference_record(stream.as_ref()),
                RecordFlavor::Quint => quint_stream_reference_record(stream.as_ref()),
            })
            .collect()
    })
    .join(",");
    let errors = union_if_missing(group, |command| {
        command
            .errors
            .as_slice()
            .iter()
            .map(|error| match flavor {
                RecordFlavor::Lean => lean_command_error_record(error),
                RecordFlavor::Quint => quint_command_error_record(error),
            })
            .collect()
    })
    .join(",");
    let singleton = first.singleton_repeat_behavior.is_some();
    let repeat_behavior = quoted(
        first
            .singleton_repeat_behavior
            .as_ref()
            .map_or("", |repeat_behavior| repeat_behavior.as_ref()),
    );
    match flavor {
        RecordFlavor::Lean => format!(
            "{{ name := {}, inputs := [{}], emittedEvents := [{}], observedStreams := [{}], errors := [{}], singleton := {}, repeatBehavior := {} }}",
            quoted(first.name.as_ref()),
            inputs,
            emitted_events,
            observed_streams,
            errors,
            singleton,
            repeat_behavior,
        ),
        RecordFlavor::Quint => format!(
            "{{ name: {}, inputs: [{}], emittedEvents: [{}], observedStreams: [{}], errors: [{}], singleton: {}, repeatBehavior: {} }}",
            quoted(first.name.as_ref()),
            inputs,
            emitted_events,
            observed_streams,
            errors,
            singleton,
            repeat_behavior,
        ),
    }
}

/// Render the whole `sliceViewDefinitions` list body directly from the projected
/// facts, byte-identical to folding them through `merge_or_append_named_record`
/// (child lists per `VIEW_CHILD_LIST_FIELDS`: `fields`/`controls` accumulate,
/// `readModels`/`sketchTokens`/`localStates`/`filters` union by identity). Never
/// parses artifact text.
fn render_slice_view_definitions(views: &[NewViewDefinition], flavor: RecordFlavor) -> String {
    let records = group_named(views, |view| view.name.as_ref())
        .iter()
        .map(|group| render_view_definition_record(group, flavor))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{records}]")
}

fn render_view_definition_record(group: &[&NewViewDefinition], flavor: RecordFlavor) -> String {
    // `group_named` never yields an empty group; an empty slice renders nothing
    // rather than panicking.
    let Some(&first) = group.first() else {
        return String::new();
    };
    let read_models = union_if_missing(group, |view| {
        vec![quoted(view.field.source_read_model.as_ref())]
    })
    .join(",");
    let fields = group
        .iter()
        .map(|view| match flavor {
            RecordFlavor::Lean => lean_view_field_record(&view.field),
            RecordFlavor::Quint => quint_view_field_record(&view.field),
        })
        .collect::<Vec<_>>()
        .join(",");
    let controls = group
        .iter()
        .flat_map(|view| {
            view.controls.as_slice().iter().map(|control| match flavor {
                RecordFlavor::Lean => lean_control_definition_record(control),
                RecordFlavor::Quint => quint_control_definition_record(control),
            })
        })
        .collect::<Vec<_>>()
        .join(",");
    let sketch_tokens = union_if_missing(group, |view| {
        view_sketch_tokens(view)
            .iter()
            .map(|token| quoted(token.as_ref()))
            .collect()
    })
    .join(",");
    let local_states = union_if_missing(group, |view| {
        view.local_states
            .as_slice()
            .iter()
            .map(|state| quoted(state.as_ref()))
            .collect()
    })
    .join(",");
    let filters = union_if_missing(group, |view| {
        view.filters
            .as_slice()
            .iter()
            .map(|filter| quoted(filter.as_ref()))
            .collect()
    })
    .join(",");
    // `filters` is the record's final field, so a splice into it consumes the
    // space before the closing brace (see the external-payload renderer). That
    // splice fires only when a member after the first contributes a non-empty
    // filters list; otherwise the rendered `] }` survives.
    let filters_spliced = group.len() > 1
        && group
            .get(1..)
            .into_iter()
            .flatten()
            .any(|view| !view.filters.as_slice().is_empty());
    let closing = if filters_spliced { "]}" } else { "] }" };
    match flavor {
        RecordFlavor::Lean => format!(
            "{{ name := {}, readModels := [{}], fields := [{}], controls := [{}], sketchTokens := [{}], localStates := [{}], filters := [{}{}",
            quoted(first.name.as_ref()),
            read_models,
            fields,
            controls,
            sketch_tokens,
            local_states,
            filters,
            closing,
        ),
        RecordFlavor::Quint => format!(
            "{{ name: {}, readModels: [{}], fields: [{}], controls: [{}], sketchTokens: [{}], localStates: [{}], filters: [{}{}",
            quoted(first.name.as_ref()),
            read_models,
            fields,
            controls,
            sketch_tokens,
            local_states,
            filters,
            closing,
        ),
    }
}

/// The projected slice facts a complete slice module is rendered from — the same
/// vectors `ProjectedSlice` carries, in arrival order.
pub(crate) struct SliceModuleFacts<'a> {
    pub(crate) scenarios: &'a [NewSliceScenario],
    pub(crate) outcomes: &'a [NewOutcomeDefinition],
    pub(crate) external_payloads: &'a [NewExternalPayloadDefinition],
    pub(crate) event_definitions: &'a [NewEventDefinition],
    pub(crate) command_definitions: &'a [NewCommandDefinition],
    pub(crate) read_models: &'a [NewReadModelDefinition],
    pub(crate) bit_level_data_flows: &'a [NewBitLevelDataFlow],
    pub(crate) views: &'a [NewViewDefinition],
    pub(crate) translations: &'a [NewTranslationDefinition],
    pub(crate) automations: &'a [NewAutomationDefinition],
    pub(crate) board_elements: &'a [NewBoardElement],
    pub(crate) board_connections: &'a [NewBoardConnection],
}

/// Populate every definition list of a freshly-emitted slice module `shell`
/// (whose lists are the `:= []` placeholders) directly from the projected facts,
/// replacing each placeholder with the fully-rendered list body. This is the pure
/// emit path: it renders each list from the facts in one shot instead of
/// replaying records through the incremental text-merge builders, so it never
/// parses an artifact. Each rendered body is parity-proven byte-identical to the
/// builder fold for that list (see `pure_emit_parity_tests`), so the populated
/// module is byte-identical to the replay-built module.
pub(crate) fn populate_slice_lists(
    shell: &str,
    facts: &SliceModuleFacts<'_>,
    flavor: RecordFlavor,
) -> String {
    let mut bodies: Vec<SliceListBody> = Vec::new();
    bodies.extend(scenario_list_bodies(facts, flavor));
    bodies.extend(outcome_and_payload_list_bodies(facts, flavor));
    bodies.extend(event_list_bodies(facts, flavor));
    bodies.extend(command_list_bodies(facts, flavor));
    bodies.extend(read_model_list_bodies(facts, flavor));
    bodies.extend(data_flow_and_view_list_bodies(facts, flavor));
    bodies.extend(translation_and_automation_list_bodies(facts, flavor));
    bodies.extend(board_list_bodies(facts, flavor));
    apply_slice_list_bodies(shell, flavor, &bodies)
}

/// One populated slice-module list: the Lean marker, the Quint marker, and the
/// rendered body that replaces the `:= []` (resp. `= []`) placeholder following
/// the flavor-appropriate marker.
type SliceListBody = (&'static str, &'static str, String);

/// Render the scenario list for one `kind`, in arrival order, using the
/// flavor-appropriate scenario record renderer.
fn scenario_list_body(
    facts: &SliceModuleFacts<'_>,
    flavor: RecordFlavor,
    kind: ScenarioKind,
) -> String {
    let matching: Vec<&NewSliceScenario> = facts
        .scenarios
        .iter()
        .filter(|scenario| scenario.kind == kind)
        .collect();
    render_append_list(&matching, |scenario| match flavor {
        RecordFlavor::Lean => lean_scenario_record(scenario),
        RecordFlavor::Quint => quint_scenario_record(scenario),
    })
}

/// `sliceReferencedCommands` accumulates (no dedup) the views' control command
/// references first, then one reference per translation, then per automation —
/// the order `ProjectedSlice::effects` enqueues those handlers.
fn referenced_commands_list_body(facts: &SliceModuleFacts<'_>, flavor: RecordFlavor) -> String {
    let mut referenced_commands: Vec<String> = Vec::new();
    for view in facts.views {
        referenced_commands.extend(match flavor {
            RecordFlavor::Lean => lean_view_referenced_command_records(view),
            RecordFlavor::Quint => quint_view_referenced_command_records(view),
        });
    }
    for translation in facts.translations {
        referenced_commands.push(match flavor {
            RecordFlavor::Lean => lean_command_reference_record(translation.command_name.as_ref()),
            RecordFlavor::Quint => {
                quint_command_reference_record(translation.command_name.as_ref())
            }
        });
    }
    for automation in facts.automations {
        referenced_commands.push(match flavor {
            RecordFlavor::Lean => lean_command_reference_record(automation.command_name.as_ref()),
            RecordFlavor::Quint => quint_command_reference_record(automation.command_name.as_ref()),
        });
    }
    format!("[{}]", referenced_commands.join(","))
}

fn scenario_list_bodies(facts: &SliceModuleFacts<'_>, flavor: RecordFlavor) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceAcceptanceScenarios : List EventModelScenario := ",
            "val sliceAcceptanceScenarios: List[EventModelScenario] = ",
            scenario_list_body(facts, flavor, ScenarioKind::Acceptance),
        ),
        (
            "def sliceContractScenarios : List EventModelScenario := ",
            "val sliceContractScenarios: List[EventModelScenario] = ",
            scenario_list_body(facts, flavor, ScenarioKind::Contract),
        ),
    ]
}

fn outcome_and_payload_list_bodies(
    facts: &SliceModuleFacts<'_>,
    flavor: RecordFlavor,
) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceOutcomeDefinitions : List OutcomeDefinition := ",
            "val sliceOutcomeDefinitions: List[OutcomeDefinition] = ",
            render_append_list(facts.outcomes, |outcome| match flavor {
                RecordFlavor::Lean => lean_outcome_definition_record(outcome),
                RecordFlavor::Quint => quint_outcome_definition_record(outcome),
            }),
        ),
        (
            "def sliceExternalPayloads : List ExternalPayloadDefinition := ",
            "val sliceExternalPayloads: List[ExternalPayloadDefinition] = ",
            render_slice_external_payload_definitions(facts.external_payloads, flavor),
        ),
    ]
}

fn event_list_bodies(facts: &SliceModuleFacts<'_>, flavor: RecordFlavor) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceEvents : List SliceEventReference := ",
            "val sliceEvents: List[SliceEventReference] = ",
            render_dedup_list(facts.event_definitions, |event| match flavor {
                RecordFlavor::Lean => lean_event_reference_record(event.name.as_ref()),
                RecordFlavor::Quint => quint_event_reference_record(event.name.as_ref()),
            }),
        ),
        (
            "def sliceStreams : List StreamDefinition := ",
            "val sliceStreams: List[StreamDefinition] = ",
            render_dedup_list(facts.event_definitions, |event| match flavor {
                RecordFlavor::Lean => lean_stream_record(event.stream.as_ref()),
                RecordFlavor::Quint => quint_stream_record(event.stream.as_ref()),
            }),
        ),
        (
            "def sliceEventDefinitions : List EventDefinition := ",
            "val sliceEventDefinitions: List[EventDefinition] = ",
            render_slice_event_definitions(facts.event_definitions, flavor),
        ),
    ]
}

fn command_list_bodies(facts: &SliceModuleFacts<'_>, flavor: RecordFlavor) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceCommands : List SliceCommandReference := ",
            "val sliceCommands: List[SliceCommandReference] = ",
            render_dedup_list(facts.command_definitions, |command| match flavor {
                RecordFlavor::Lean => lean_command_reference_record(command.name.as_ref()),
                RecordFlavor::Quint => quint_command_reference_record(command.name.as_ref()),
            }),
        ),
        (
            "def sliceCommandDefinitions : List CommandDefinition := ",
            "val sliceCommandDefinitions: List[CommandDefinition] = ",
            render_slice_command_definitions(facts.command_definitions, flavor),
        ),
    ]
}

fn read_model_list_bodies(
    facts: &SliceModuleFacts<'_>,
    flavor: RecordFlavor,
) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceReadModels : List SliceReadModelReference := ",
            "val sliceReadModels: List[SliceReadModelReference] = ",
            render_dedup_list(facts.read_models, |read_model| match flavor {
                RecordFlavor::Lean => lean_read_model_reference_record(read_model.name.as_ref()),
                RecordFlavor::Quint => quint_read_model_reference_record(read_model.name.as_ref()),
            }),
        ),
        (
            "def sliceReadModelDefinitions : List ReadModelDefinition := ",
            "val sliceReadModelDefinitions: List[ReadModelDefinition] = ",
            render_slice_read_model_definitions(facts.read_models, flavor),
        ),
    ]
}

fn data_flow_and_view_list_bodies(
    facts: &SliceModuleFacts<'_>,
    flavor: RecordFlavor,
) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceBitLevelDataFlows : List BitLevelDataFlow := ",
            "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = ",
            render_append_list(facts.bit_level_data_flows, |data_flow| match flavor {
                RecordFlavor::Lean => lean_data_flow_record(data_flow),
                RecordFlavor::Quint => quint_data_flow_record(data_flow),
            }),
        ),
        (
            "def sliceViews : List SliceViewReference := ",
            "val sliceViews: List[SliceViewReference] = ",
            render_dedup_list(facts.views, |view| match flavor {
                RecordFlavor::Lean => lean_view_reference_record(view.name.as_ref()),
                RecordFlavor::Quint => quint_view_reference_record(view.name.as_ref()),
            }),
        ),
        (
            "def sliceReferencedCommands : List SliceCommandReference := ",
            "val sliceReferencedCommands: List[SliceCommandReference] = ",
            referenced_commands_list_body(facts, flavor),
        ),
        (
            "def sliceViewDefinitions : List ViewDefinition := ",
            "val sliceViewDefinitions: List[ViewDefinition] = ",
            render_slice_view_definitions(facts.views, flavor),
        ),
    ]
}

fn translation_and_automation_list_bodies(
    facts: &SliceModuleFacts<'_>,
    flavor: RecordFlavor,
) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceTranslations : List TranslationDefinition := ",
            "val sliceTranslations: List[TranslationDefinition] = ",
            render_append_list(facts.translations, |translation| match flavor {
                RecordFlavor::Lean => lean_translation_definition_record(translation),
                RecordFlavor::Quint => quint_translation_definition_record(translation),
            }),
        ),
        (
            "def sliceAutomations : List AutomationDefinition := ",
            "val sliceAutomations: List[AutomationDefinition] = ",
            render_append_list(facts.automations, |automation| match flavor {
                RecordFlavor::Lean => lean_automation_definition_record(automation),
                RecordFlavor::Quint => quint_automation_definition_record(automation),
            }),
        ),
    ]
}

fn board_list_bodies(facts: &SliceModuleFacts<'_>, flavor: RecordFlavor) -> Vec<SliceListBody> {
    vec![
        (
            "def sliceBoardElements : List BoardElement := ",
            "val sliceBoardElements: List[BoardElement] = ",
            render_append_list(facts.board_elements, |element| match flavor {
                RecordFlavor::Lean => lean_board_element_record(element),
                RecordFlavor::Quint => quint_board_element_record(element),
            }),
        ),
        (
            "def sliceBoardConnections : List BoardConnection := ",
            "val sliceBoardConnections: List[BoardConnection] = ",
            render_append_list(facts.board_connections, |connection| match flavor {
                RecordFlavor::Lean => lean_board_connection_record(connection),
                RecordFlavor::Quint => quint_board_connection_record(connection),
            }),
        ),
    ]
}

/// Replace each list's `:= []` (resp. `= []`) placeholder in `shell` with the
/// rendered body, in the order the bodies appear.
fn apply_slice_list_bodies(shell: &str, flavor: RecordFlavor, bodies: &[SliceListBody]) -> String {
    let mut populated = shell.to_owned();
    for (lean_marker, quint_marker, body) in bodies {
        let marker = match flavor {
            RecordFlavor::Lean => lean_marker,
            RecordFlavor::Quint => quint_marker,
        };
        populated = populated.replace(&format!("{marker}[]"), &format!("{marker}{body}"));
    }
    populated
}

fn lean_event_reference_record(event_name: &str) -> String {
    format!("{{ name := {} }}", quoted(event_name))
}

fn quint_event_reference_record(event_name: &str) -> String {
    format!("{{ name: {} }}", quoted(event_name))
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
