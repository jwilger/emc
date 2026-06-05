use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::formal_slice_facts::{
    CommandErrorDefinitions, NewAutomationDefinition, NewBitLevelDataFlow, NewCommandDefinition,
    NewCommandErrorDefinition, NewCommandInput, NewControlDefinition, NewEventAttribute,
    NewEventDefinition, NewExternalPayloadDefinition, NewReadModelDefinition, NewReadModelField,
    NewSliceScenario, NewTranslationDefinition, NewViewDefinition, NewViewField, OutcomeEventNames,
    ScenarioKind,
};
use crate::core::types::{
    AutomationName, AutomationReactionDescription, AutomationTriggerName, BitEncodingSemantics,
    CommandErrorName, CommandErrorRecoveryKind, CommandInputSourceDescription,
    CommandInputSourceKind, CommandName, ContractKindName, ControlName, ControlRecoveryBehavior,
    CoveredDefinitionName, DataFlowSource, DataFlowTarget, DatumName, EventAttributeName,
    EventAttributeSourceField, EventAttributeSourceKind, EventAttributeSourceName, EventName,
    NavigationTargetName, NavigationTargetType, OutcomeLabelName, PayloadContractName,
    ProvenanceDescription, ReadModelFieldSourceKind, ReadModelName, ScenarioName, ScenarioStepText,
    SketchToken, SliceSlug, StreamName, TransformationSemantics, TranslationExternalEventName,
    TranslationName, ViewFieldName, ViewFieldSourceKind, ViewName, WorkflowSlug,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectStream {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    stream: StreamName,
}

impl NewProjectStream {
    pub fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug, stream: StreamName) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            stream,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectCommand {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    command: CommandName,
    command_inputs: Vec<NewProjectCommandInput>,
    command_errors: Vec<NewProjectCommandError>,
}

impl NewProjectCommand {
    pub fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug, command: CommandName) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            command,
            command_inputs: Vec::new(),
            command_errors: Vec::new(),
        }
    }

    pub fn with_input(mut self, input: &NewCommandInput) -> Self {
        self.command_inputs = vec![NewProjectCommandInput::from_command_input(&self, input)];
        self
    }

    pub fn from_command(workflow_slug: WorkflowSlug, command: &NewCommandDefinition) -> Self {
        Self::new(
            workflow_slug,
            command.slice_slug().clone(),
            command.name().clone(),
        )
        .with_input(command.input())
        .with_errors(command.errors().clone())
    }

    pub fn with_errors(mut self, errors: CommandErrorDefinitions) -> Self {
        self.command_errors = errors
            .as_slice()
            .iter()
            .map(|error| NewProjectCommandError::from_command_error(&self, error))
            .collect();
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectCommandInput {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    command: CommandName,
    input: DatumName,
    source_kind: String,
    source_description: String,
    provenance_chain: Vec<String>,
}

impl NewProjectCommandInput {
    fn from_command_input(command: &NewProjectCommand, input: &NewCommandInput) -> Self {
        Self {
            workflow_slug: command.workflow_slug.clone(),
            slice_slug: command.slice_slug.clone(),
            command: command.command.clone(),
            input: input.name().clone(),
            source_kind: input.source_kind().as_ref().to_owned(),
            source_description: input.source_description().as_ref().to_owned(),
            provenance_chain: input
                .provenance_chain()
                .as_slice()
                .iter()
                .map(|hop| hop.as_ref().to_owned())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectCommandError {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    command: CommandName,
    error: CommandErrorName,
    scenario: ScenarioName,
    recovery: CommandErrorRecoveryKind,
}

impl NewProjectCommandError {
    fn from_command_error(command: &NewProjectCommand, error: &NewCommandErrorDefinition) -> Self {
        Self {
            workflow_slug: command.workflow_slug.clone(),
            slice_slug: command.slice_slug.clone(),
            command: command.command.clone(),
            error: error.name().clone(),
            scenario: error.scenario_name().clone(),
            recovery: error.recovery_kind().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectDataFlow {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    datum: DatumName,
    source: DataFlowSource,
    transformation: TransformationSemantics,
    target: DataFlowTarget,
    bit_encoding: BitEncodingSemantics,
}

impl NewProjectDataFlow {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        datum: DatumName,
        source: DataFlowSource,
        transformation: TransformationSemantics,
        target: DataFlowTarget,
        bit_encoding: BitEncodingSemantics,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            datum,
            source,
            transformation,
            target,
            bit_encoding,
        }
    }

    pub fn from_slice_data_flow(
        workflow_slug: WorkflowSlug,
        data_flow: &NewBitLevelDataFlow,
    ) -> Self {
        Self::new(
            workflow_slug,
            data_flow.slice_slug().clone(),
            data_flow.datum().clone(),
            data_flow.source().clone(),
            data_flow.transformation().clone(),
            data_flow.target().clone(),
            data_flow.bit_encoding().clone(),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectReadModel {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    read_model: ReadModelName,
    read_model_definitions: Vec<NewProjectReadModelDefinition>,
    read_model_fields: Vec<NewProjectReadModelField>,
}

impl NewProjectReadModel {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        read_model: ReadModelName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            read_model,
            read_model_definitions: Vec::new(),
            read_model_fields: Vec::new(),
        }
    }

    pub fn with_definition(mut self, read_model: &NewReadModelDefinition) -> Self {
        self.read_model_definitions = vec![NewProjectReadModelDefinition::from_read_model(
            &self, read_model,
        )];
        self
    }

    pub fn with_field(mut self, field: &NewReadModelField) -> Self {
        self.read_model_fields = vec![NewProjectReadModelField::from_read_model_field(
            &self, field,
        )];
        self
    }

    pub fn from_read_model(
        workflow_slug: WorkflowSlug,
        read_model: &NewReadModelDefinition,
    ) -> Self {
        Self::new(
            workflow_slug,
            read_model.slice_slug().clone(),
            read_model.name().clone(),
        )
        .with_definition(read_model)
        .with_field(read_model.field())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectReadModelDefinition {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    read_model: ReadModelName,
    transitive: bool,
    relationship_fields: Vec<String>,
    transitive_rule: String,
    example_scenario_name: String,
}

impl NewProjectReadModelDefinition {
    fn from_read_model(parent: &NewProjectReadModel, read_model: &NewReadModelDefinition) -> Self {
        Self {
            workflow_slug: parent.workflow_slug.clone(),
            slice_slug: parent.slice_slug.clone(),
            read_model: parent.read_model.clone(),
            transitive: read_model.transitive(),
            relationship_fields: read_model
                .relationship_fields()
                .as_slice()
                .iter()
                .map(|field| field.as_ref().to_owned())
                .collect(),
            transitive_rule: read_model
                .transitive_rule()
                .map(|rule| rule.as_ref().to_owned())
                .unwrap_or_default(),
            example_scenario_name: read_model
                .example_scenario_name()
                .map(|scenario| scenario.as_ref().to_owned())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectReadModelField {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    read_model: ReadModelName,
    field: DatumName,
    source_kind: ReadModelFieldSourceKind,
    source_event: String,
    source_attribute: String,
    derivation_rule: String,
    absence_event: String,
    derivation_scenario_name: String,
    absence_scenario_name: String,
    provenance: ProvenanceDescription,
}

impl NewProjectReadModelField {
    fn from_read_model_field(read_model: &NewProjectReadModel, field: &NewReadModelField) -> Self {
        Self {
            workflow_slug: read_model.workflow_slug.clone(),
            slice_slug: read_model.slice_slug.clone(),
            read_model: read_model.read_model.clone(),
            field: field.name().clone(),
            source_kind: field.source_kind().clone(),
            source_event: field
                .source_event()
                .map(|event| event.as_ref().to_owned())
                .unwrap_or_default(),
            source_attribute: field
                .source_attribute()
                .map(|attribute| attribute.as_ref().to_owned())
                .unwrap_or_default(),
            derivation_rule: field
                .derivation_rule()
                .map(|rule| rule.as_ref().to_owned())
                .unwrap_or_default(),
            absence_event: field
                .absence_event()
                .map(|event| event.as_ref().to_owned())
                .unwrap_or_default(),
            derivation_scenario_name: field
                .derivation_scenario_name()
                .map(|scenario| scenario.as_ref().to_owned())
                .unwrap_or_default(),
            absence_scenario_name: field
                .absence_scenario_name()
                .map(|scenario| scenario.as_ref().to_owned())
                .unwrap_or_default(),
            provenance: field.provenance_description().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectView {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    view: ViewName,
    view_definition: Option<NewProjectViewDefinition>,
    view_controls: Vec<NewProjectViewControl>,
    view_fields: Vec<NewProjectViewField>,
}

impl NewProjectView {
    pub fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug, view: ViewName) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            view,
            view_definition: None,
            view_controls: Vec::new(),
            view_fields: Vec::new(),
        }
    }

    pub fn with_field(mut self, field: &NewViewField) -> Self {
        self.view_fields = vec![NewProjectViewField::from_view_field(&self, field)];
        self
    }

    pub fn from_view(workflow_slug: WorkflowSlug, view: &NewViewDefinition) -> Self {
        let mut project_view = Self::new(
            workflow_slug,
            view.slice_slug().clone(),
            view.name().clone(),
        )
        .with_field(view.field());
        project_view.view_definition =
            Some(NewProjectViewDefinition::from_view(&project_view, view));
        project_view.view_controls = view
            .controls()
            .as_slice()
            .iter()
            .map(|control| NewProjectViewControl::from_control(&project_view, control))
            .collect();
        project_view
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct NewProjectViewDefinition {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    view: ViewName,
    read_models: Vec<ReadModelName>,
    sketch_tokens: Vec<SketchToken>,
    local_states: Vec<String>,
    filters: Vec<String>,
}

impl NewProjectViewDefinition {
    fn from_view(project_view: &NewProjectView, view: &NewViewDefinition) -> Self {
        Self {
            workflow_slug: project_view.workflow_slug.clone(),
            slice_slug: project_view.slice_slug.clone(),
            view: project_view.view.clone(),
            read_models: vec![view.field().source_read_model().clone()],
            sketch_tokens: vec![view.field().sketch_token().clone()],
            local_states: Vec::new(),
            filters: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct NewProjectViewControl {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    view: ViewName,
    control: ControlName,
    command: CommandName,
    input: DatumName,
    input_source_kind: CommandInputSourceKind,
    input_source_description: CommandInputSourceDescription,
    input_sketch_token: SketchToken,
    input_visible_to_actor: bool,
    input_decision_field: bool,
    handled_errors: Vec<CommandErrorName>,
    recovery_behavior: ControlRecoveryBehavior,
    control_sketch_token: SketchToken,
    navigation_type: NavigationTargetType,
    navigation_target: NavigationTargetName,
    external_workflow: String,
    external_system: String,
    handoff_contract: String,
}

impl NewProjectViewControl {
    fn from_control(view: &NewProjectView, control: &NewControlDefinition) -> Self {
        let input = control.input();
        let navigation = control.navigation();
        Self {
            workflow_slug: view.workflow_slug.clone(),
            slice_slug: view.slice_slug.clone(),
            view: view.view.clone(),
            control: control.name().clone(),
            command: control.command_name().clone(),
            input: input.name().clone(),
            input_source_kind: input.source_kind().clone(),
            input_source_description: input.source_description().clone(),
            input_sketch_token: input.sketch_token().clone(),
            input_visible_to_actor: input.visible_to_actor(),
            input_decision_field: input.decision_field(),
            handled_errors: control.handled_errors().as_slice().to_vec(),
            recovery_behavior: control.recovery_behavior().clone(),
            control_sketch_token: control.sketch_token().clone(),
            navigation_type: navigation.target_type().clone(),
            navigation_target: navigation.target_name().clone(),
            external_workflow: navigation
                .external_workflow_name()
                .map_or("", NavigationTargetName::as_ref)
                .to_owned(),
            external_system: navigation
                .external_system_name()
                .map_or("", NavigationTargetName::as_ref)
                .to_owned(),
            handoff_contract: navigation
                .handoff_contract()
                .map_or("", PayloadContractName::as_ref)
                .to_owned(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectViewField {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    view: ViewName,
    field: ViewFieldName,
    source_kind: ViewFieldSourceKind,
    source_read_model: ReadModelName,
    source_field: ViewFieldName,
    provenance: ProvenanceDescription,
    bit_encoding: BitEncodingSemantics,
}

impl NewProjectViewField {
    fn from_view_field(view: &NewProjectView, field: &NewViewField) -> Self {
        Self {
            workflow_slug: view.workflow_slug.clone(),
            slice_slug: view.slice_slug.clone(),
            view: view.view.clone(),
            field: field.name().clone(),
            source_kind: field.source_kind().clone(),
            source_read_model: field.source_read_model().clone(),
            source_field: field.source_field().clone(),
            provenance: field.provenance_description().clone(),
            bit_encoding: field.bit_encoding().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectAutomation {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    automation: AutomationName,
    automation_definition: Option<NewProjectAutomationDefinition>,
}

impl NewProjectAutomation {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        automation: AutomationName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            automation,
            automation_definition: None,
        }
    }

    pub fn from_automation(
        workflow_slug: WorkflowSlug,
        automation: &NewAutomationDefinition,
    ) -> Self {
        let mut project_automation = Self::new(
            workflow_slug,
            automation.slice_slug().clone(),
            automation.name().clone(),
        );
        project_automation.automation_definition = Some(
            NewProjectAutomationDefinition::from_automation(&project_automation, automation),
        );
        project_automation
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct NewProjectAutomationDefinition {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    automation: AutomationName,
    trigger: AutomationTriggerName,
    command: CommandName,
    handled_errors: Vec<CommandErrorName>,
    reaction: AutomationReactionDescription,
}

impl NewProjectAutomationDefinition {
    fn from_automation(
        project_automation: &NewProjectAutomation,
        automation: &NewAutomationDefinition,
    ) -> Self {
        Self {
            workflow_slug: project_automation.workflow_slug.clone(),
            slice_slug: project_automation.slice_slug.clone(),
            automation: project_automation.automation.clone(),
            trigger: automation.trigger_name().clone(),
            command: automation.command_name().clone(),
            handled_errors: automation.handled_errors().as_slice().to_vec(),
            reaction: automation.reaction_description().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectTranslation {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    translation: TranslationName,
    translation_definition: Option<NewProjectTranslationDefinition>,
}

impl NewProjectTranslation {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        translation: TranslationName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            translation,
            translation_definition: None,
        }
    }

    pub fn from_translation(
        workflow_slug: WorkflowSlug,
        translation: &NewTranslationDefinition,
    ) -> Self {
        let mut project_translation = Self::new(
            workflow_slug,
            translation.slice_slug().clone(),
            translation.name().clone(),
        );
        project_translation.translation_definition = Some(
            NewProjectTranslationDefinition::from_translation(&project_translation, translation),
        );
        project_translation
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct NewProjectTranslationDefinition {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    translation: TranslationName,
    external_event: TranslationExternalEventName,
    payload_contract: PayloadContractName,
    command: CommandName,
}

impl NewProjectTranslationDefinition {
    fn from_translation(
        project_translation: &NewProjectTranslation,
        translation: &NewTranslationDefinition,
    ) -> Self {
        Self {
            workflow_slug: project_translation.workflow_slug.clone(),
            slice_slug: project_translation.slice_slug.clone(),
            translation: project_translation.translation.clone(),
            external_event: translation.external_event_name().clone(),
            payload_contract: translation.payload_contract_name().clone(),
            command: translation.command_name().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectExternalPayload {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    external_payload: EventAttributeSourceName,
    external_payload_field: Option<NewProjectExternalPayloadField>,
}

impl NewProjectExternalPayload {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        external_payload: EventAttributeSourceName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            external_payload,
            external_payload_field: None,
        }
    }

    pub fn from_external_payload(
        workflow_slug: WorkflowSlug,
        external_payload: &NewExternalPayloadDefinition,
    ) -> Self {
        Self {
            workflow_slug: workflow_slug.clone(),
            slice_slug: external_payload.slice_slug().clone(),
            external_payload: external_payload.name().clone(),
            external_payload_field: Some(NewProjectExternalPayloadField::from_external_payload(
                workflow_slug,
                external_payload,
            )),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectExternalPayloadField {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    external_payload: EventAttributeSourceName,
    field: EventAttributeSourceField,
    provenance: ProvenanceDescription,
    bit_encoding: BitEncodingSemantics,
}

impl NewProjectExternalPayloadField {
    pub fn from_external_payload(
        workflow_slug: WorkflowSlug,
        external_payload: &NewExternalPayloadDefinition,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug: external_payload.slice_slug().clone(),
            external_payload: external_payload.name().clone(),
            field: external_payload.field().clone(),
            provenance: external_payload.field_provenance().clone(),
            bit_encoding: external_payload.bit_encoding().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectScenario {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    scenario_kind: ScenarioKind,
    scenario: ScenarioName,
    scenario_definition: Option<NewProjectScenarioDefinition>,
}

impl NewProjectScenario {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        scenario_kind: ScenarioKind,
        scenario: ScenarioName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            scenario_kind,
            scenario,
            scenario_definition: None,
        }
    }

    pub fn from_slice_scenario(workflow_slug: WorkflowSlug, scenario: &NewSliceScenario) -> Self {
        Self {
            workflow_slug: workflow_slug.clone(),
            slice_slug: scenario.slice_slug().clone(),
            scenario_kind: scenario.kind(),
            scenario: scenario.name().clone(),
            scenario_definition: Some(NewProjectScenarioDefinition::from_slice_scenario(
                workflow_slug,
                scenario,
            )),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectScenarioDefinition {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
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

impl NewProjectScenarioDefinition {
    pub fn from_slice_scenario(workflow_slug: WorkflowSlug, scenario: &NewSliceScenario) -> Self {
        Self {
            workflow_slug,
            slice_slug: scenario.slice_slug().clone(),
            scenario_kind: scenario.kind(),
            scenario: scenario.name().clone(),
            given: scenario.given().clone(),
            when: scenario.when().clone(),
            then: scenario.then().clone(),
            read_streams: scenario.read_streams().as_slice().to_vec(),
            written_streams: scenario.written_streams().as_slice().to_vec(),
            contract_kind: scenario.contract_kind().cloned(),
            covered_definition: scenario.covered_definition().cloned(),
            error_references: scenario.error_references().as_slice().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectOutcome {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    outcome: OutcomeLabelName,
    events: OutcomeEventNames,
    externally_relevant: bool,
}

impl NewProjectOutcome {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        outcome: OutcomeLabelName,
        events: OutcomeEventNames,
        externally_relevant: bool,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            outcome,
            events,
            externally_relevant,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectEvent {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    event: EventName,
    stream: StreamName,
    event_attributes: Vec<NewProjectEventAttribute>,
}

impl NewProjectEvent {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        event: EventName,
        stream: StreamName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            event,
            stream,
            event_attributes: Vec::new(),
        }
    }

    pub fn with_attribute(mut self, attribute: &NewEventAttribute) -> Self {
        self.event_attributes = vec![NewProjectEventAttribute::from_event_attribute(
            &self, attribute,
        )];
        self
    }

    pub fn from_event(workflow_slug: WorkflowSlug, event: &NewEventDefinition) -> Self {
        Self::new(
            workflow_slug,
            event.slice_slug().clone(),
            event.name().clone(),
            event.stream().clone(),
        )
        .with_attribute(event.attribute())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectEventAttribute {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    event: EventName,
    attribute: EventAttributeName,
    source_kind: EventAttributeSourceKind,
    source_name: EventAttributeSourceName,
    source_field: EventAttributeSourceField,
    provenance: ProvenanceDescription,
}

impl NewProjectEventAttribute {
    fn from_event_attribute(event: &NewProjectEvent, attribute: &NewEventAttribute) -> Self {
        Self {
            workflow_slug: event.workflow_slug.clone(),
            slice_slug: event.slice_slug.clone(),
            event: event.event.clone(),
            attribute: attribute.name().clone(),
            source_kind: attribute.source_kind().clone(),
            source_name: attribute.source_name().clone(),
            source_field: attribute.source_field().clone(),
            provenance: attribute.provenance_description().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectStream {
    workflow_slug: String,
    slice_slug: String,
    stream: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectCommand {
    workflow_slug: String,
    slice_slug: String,
    command: String,
}

impl ProjectCommand {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn command(&self) -> &str {
        &self.command
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectCommandInput {
    workflow_slug: String,
    slice_slug: String,
    command: String,
    input: String,
    source_kind: String,
    source_description: String,
    provenance_chain: Vec<String>,
}

impl ProjectCommandInput {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub fn source_description(&self) -> &str {
        &self.source_description
    }

    pub fn provenance_chain(&self) -> &[String] {
        &self.provenance_chain
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectCommandError {
    workflow_slug: String,
    slice_slug: String,
    command: String,
    error: String,
    scenario: String,
    recovery: String,
}

impl ProjectCommandError {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn error(&self) -> &str {
        &self.error
    }

    pub fn scenario(&self) -> &str {
        &self.scenario
    }

    pub fn recovery(&self) -> &str {
        &self.recovery
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectDataFlow {
    workflow_slug: String,
    slice_slug: String,
    datum: String,
    source: String,
    transformation: String,
    target: String,
    bit_encoding: String,
}

impl ProjectDataFlow {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn datum(&self) -> &str {
        &self.datum
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn transformation(&self) -> &str {
        &self.transformation
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn bit_encoding(&self) -> &str {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectReadModel {
    workflow_slug: String,
    slice_slug: String,
    read_model: String,
}

impl ProjectReadModel {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn read_model(&self) -> &str {
        &self.read_model
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectReadModelDefinition {
    workflow_slug: String,
    slice_slug: String,
    read_model: String,
    transitive: bool,
    relationship_fields: Vec<String>,
    transitive_rule: String,
    example_scenario_name: String,
}

impl ProjectReadModelDefinition {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn read_model(&self) -> &str {
        &self.read_model
    }

    pub fn transitive(&self) -> bool {
        self.transitive
    }

    pub fn relationship_fields(&self) -> &[String] {
        &self.relationship_fields
    }

    pub fn transitive_rule(&self) -> &str {
        &self.transitive_rule
    }

    pub fn example_scenario_name(&self) -> &str {
        &self.example_scenario_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectReadModelField {
    workflow_slug: String,
    slice_slug: String,
    read_model: String,
    field: String,
    source_kind: String,
    source_event: String,
    source_attribute: String,
    derivation_rule: String,
    absence_event: String,
    derivation_scenario_name: String,
    absence_scenario_name: String,
    provenance: String,
}

impl ProjectReadModelField {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn read_model(&self) -> &str {
        &self.read_model
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub fn source_event(&self) -> &str {
        &self.source_event
    }

    pub fn source_attribute(&self) -> &str {
        &self.source_attribute
    }

    pub fn derivation_rule(&self) -> &str {
        &self.derivation_rule
    }

    pub fn absence_event(&self) -> &str {
        &self.absence_event
    }

    pub fn derivation_scenario_name(&self) -> &str {
        &self.derivation_scenario_name
    }

    pub fn absence_scenario_name(&self) -> &str {
        &self.absence_scenario_name
    }

    pub fn provenance(&self) -> &str {
        &self.provenance
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectView {
    workflow_slug: String,
    slice_slug: String,
    view: String,
}

impl ProjectView {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn view(&self) -> &str {
        &self.view
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectViewDefinition {
    workflow_slug: String,
    slice_slug: String,
    view: String,
    read_models: Vec<String>,
    sketch_tokens: Vec<String>,
    local_states: Vec<String>,
    filters: Vec<String>,
}

impl ProjectViewDefinition {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn view(&self) -> &str {
        &self.view
    }

    pub fn read_models(&self) -> &[String] {
        &self.read_models
    }

    pub fn sketch_tokens(&self) -> &[String] {
        &self.sketch_tokens
    }

    pub fn local_states(&self) -> &[String] {
        &self.local_states
    }

    pub fn filters(&self) -> &[String] {
        &self.filters
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectViewControl {
    workflow_slug: String,
    slice_slug: String,
    view: String,
    control: String,
    command: String,
    input: String,
    input_source_kind: String,
    input_source_description: String,
    input_sketch_token: String,
    input_visible_to_actor: bool,
    input_decision_field: bool,
    handled_errors: Vec<String>,
    recovery_behavior: String,
    control_sketch_token: String,
    navigation_type: String,
    navigation_target: String,
    external_workflow: String,
    external_system: String,
    handoff_contract: String,
}

impl ProjectViewControl {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn view(&self) -> &str {
        &self.view
    }

    pub fn control(&self) -> &str {
        &self.control
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn input_source_kind(&self) -> &str {
        &self.input_source_kind
    }

    pub fn input_source_description(&self) -> &str {
        &self.input_source_description
    }

    pub fn input_sketch_token(&self) -> &str {
        &self.input_sketch_token
    }

    pub fn input_visible_to_actor(&self) -> bool {
        self.input_visible_to_actor
    }

    pub fn input_decision_field(&self) -> bool {
        self.input_decision_field
    }

    pub fn handled_errors(&self) -> &[String] {
        &self.handled_errors
    }

    pub fn recovery_behavior(&self) -> &str {
        &self.recovery_behavior
    }

    pub fn control_sketch_token(&self) -> &str {
        &self.control_sketch_token
    }

    pub fn navigation_type(&self) -> &str {
        &self.navigation_type
    }

    pub fn navigation_target(&self) -> &str {
        &self.navigation_target
    }

    pub fn external_workflow(&self) -> &str {
        &self.external_workflow
    }

    pub fn external_system(&self) -> &str {
        &self.external_system
    }

    pub fn handoff_contract(&self) -> &str {
        &self.handoff_contract
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectViewField {
    workflow_slug: String,
    slice_slug: String,
    view: String,
    field: String,
    source_kind: String,
    source_read_model: String,
    source_field: String,
    provenance: String,
    bit_encoding: String,
}

impl ProjectViewField {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn view(&self) -> &str {
        &self.view
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub fn source_read_model(&self) -> &str {
        &self.source_read_model
    }

    pub fn source_field(&self) -> &str {
        &self.source_field
    }

    pub fn provenance(&self) -> &str {
        &self.provenance
    }

    pub fn bit_encoding(&self) -> &str {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectAutomation {
    workflow_slug: String,
    slice_slug: String,
    automation: String,
}

impl ProjectAutomation {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn automation(&self) -> &str {
        &self.automation
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectAutomationDefinition {
    workflow_slug: String,
    slice_slug: String,
    automation: String,
    trigger: String,
    command: String,
    handled_errors: Vec<String>,
    reaction: String,
}

impl ProjectAutomationDefinition {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn automation(&self) -> &str {
        &self.automation
    }

    pub fn trigger(&self) -> &str {
        &self.trigger
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn handled_errors(&self) -> &[String] {
        &self.handled_errors
    }

    pub fn reaction(&self) -> &str {
        &self.reaction
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectTranslation {
    workflow_slug: String,
    slice_slug: String,
    translation: String,
}

impl ProjectTranslation {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn translation(&self) -> &str {
        &self.translation
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectTranslationDefinition {
    workflow_slug: String,
    slice_slug: String,
    translation: String,
    external_event: String,
    payload_contract: String,
    command: String,
}

impl ProjectTranslationDefinition {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn translation(&self) -> &str {
        &self.translation
    }

    pub fn external_event(&self) -> &str {
        &self.external_event
    }

    pub fn payload_contract(&self) -> &str {
        &self.payload_contract
    }

    pub fn command(&self) -> &str {
        &self.command
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectExternalPayload {
    workflow_slug: String,
    slice_slug: String,
    external_payload: String,
}

impl ProjectExternalPayload {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn external_payload(&self) -> &str {
        &self.external_payload
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectExternalPayloadField {
    workflow_slug: String,
    slice_slug: String,
    external_payload: String,
    field: String,
    provenance: String,
    bit_encoding: String,
}

impl ProjectExternalPayloadField {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn external_payload(&self) -> &str {
        &self.external_payload
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn provenance(&self) -> &str {
        &self.provenance
    }

    pub fn bit_encoding(&self) -> &str {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectScenario {
    workflow_slug: String,
    slice_slug: String,
    scenario_kind: String,
    scenario: String,
}

impl ProjectScenario {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn scenario_kind(&self) -> &str {
        &self.scenario_kind
    }

    pub fn scenario(&self) -> &str {
        &self.scenario
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectScenarioDefinition {
    workflow_slug: String,
    slice_slug: String,
    scenario_kind: String,
    scenario: String,
    given: String,
    when: String,
    then: String,
    read_streams: Vec<String>,
    written_streams: Vec<String>,
    contract_kind: String,
    covered_definition: String,
    error_references: Vec<String>,
}

impl ProjectScenarioDefinition {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn scenario_kind(&self) -> &str {
        &self.scenario_kind
    }

    pub fn scenario(&self) -> &str {
        &self.scenario
    }

    pub fn given(&self) -> &str {
        &self.given
    }

    pub fn when(&self) -> &str {
        &self.when
    }

    pub fn then(&self) -> &str {
        &self.then
    }

    pub fn read_streams(&self) -> &[String] {
        &self.read_streams
    }

    pub fn written_streams(&self) -> &[String] {
        &self.written_streams
    }

    pub fn contract_kind(&self) -> &str {
        &self.contract_kind
    }

    pub fn covered_definition(&self) -> &str {
        &self.covered_definition
    }

    pub fn error_references(&self) -> &[String] {
        &self.error_references
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectOutcome {
    workflow_slug: String,
    slice_slug: String,
    outcome: String,
    events: Vec<String>,
    externally_relevant: bool,
}

impl ProjectOutcome {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn outcome(&self) -> &str {
        &self.outcome
    }

    pub fn events(&self) -> &[String] {
        &self.events
    }

    pub fn externally_relevant(&self) -> bool {
        self.externally_relevant
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectEvent {
    workflow_slug: String,
    slice_slug: String,
    event: String,
    stream: String,
}

impl ProjectEvent {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn stream(&self) -> &str {
        &self.stream
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectEventAttribute {
    workflow_slug: String,
    slice_slug: String,
    event: String,
    attribute: String,
    source_kind: String,
    source_name: String,
    source_field: String,
    provenance: String,
}

impl ProjectEventAttribute {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn attribute(&self) -> &str {
        &self.attribute
    }

    pub fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn source_field(&self) -> &str {
        &self.source_field
    }

    pub fn provenance(&self) -> &str {
        &self.provenance
    }
}

impl ProjectStream {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn stream(&self) -> &str {
        &self.stream
    }
}

pub fn parse_lean_project_streams(
    contents: &FileContents,
) -> Result<Vec<ProjectStream>, FormalProjectFactError> {
    stream_entries_from_list(
        contents.as_ref(),
        "def modelStreams : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_streams(
    contents: &FileContents,
) -> Result<Vec<ProjectStream>, FormalProjectFactError> {
    stream_entries_from_list(contents.as_ref(), "val modelStreams: List[ModelStream] = ")
}

pub fn parse_lean_project_scenarios(
    contents: &FileContents,
) -> Result<Vec<ProjectScenario>, FormalProjectFactError> {
    scenario_entries_from_list(
        contents.as_ref(),
        "def modelScenarios : List (String × String × String × String) := ",
    )
}

pub fn parse_quint_project_scenarios(
    contents: &FileContents,
) -> Result<Vec<ProjectScenario>, FormalProjectFactError> {
    scenario_entries_from_list(
        contents.as_ref(),
        "val modelScenarios: List[ModelScenario] = ",
    )
}

pub fn parse_lean_project_scenario_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectScenarioDefinition>, FormalProjectFactError> {
    scenario_definition_entries_from_list(
        contents.as_ref(),
        "def modelScenarioDefinitions : List (String × String × String × String × String × String × String × List String × List String × String × String × List String) := ",
        ScenarioDefinitionSyntax::Lean,
    )
}

pub fn parse_quint_project_scenario_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectScenarioDefinition>, FormalProjectFactError> {
    scenario_definition_entries_from_list(
        contents.as_ref(),
        "val modelScenarioDefinitions: List[ModelScenarioDefinition] = ",
        ScenarioDefinitionSyntax::Quint,
    )
}

pub fn parse_lean_project_data_flows(
    contents: &FileContents,
) -> Result<Vec<ProjectDataFlow>, FormalProjectFactError> {
    data_flow_entries_from_list(
        contents.as_ref(),
        "def modelDataFlows : List (String × String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_data_flows(
    contents: &FileContents,
) -> Result<Vec<ProjectDataFlow>, FormalProjectFactError> {
    data_flow_entries_from_list(
        contents.as_ref(),
        "val modelDataFlows: List[ModelDataFlow] = ",
    )
}

pub fn parse_lean_project_outcomes(
    contents: &FileContents,
) -> Result<Vec<ProjectOutcome>, FormalProjectFactError> {
    outcome_entries_from_list(
        contents.as_ref(),
        "def modelOutcomes : List (String × String × String × List String × Bool) := ",
    )
}

pub fn parse_quint_project_outcomes(
    contents: &FileContents,
) -> Result<Vec<ProjectOutcome>, FormalProjectFactError> {
    outcome_entries_from_list(
        contents.as_ref(),
        "val modelOutcomes: List[ModelOutcome] = ",
    )
}

pub fn parse_lean_project_commands(
    contents: &FileContents,
) -> Result<Vec<ProjectCommand>, FormalProjectFactError> {
    command_entries_from_list(
        contents.as_ref(),
        "def modelCommands : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_commands(
    contents: &FileContents,
) -> Result<Vec<ProjectCommand>, FormalProjectFactError> {
    command_entries_from_list(
        contents.as_ref(),
        "val modelCommands: List[ModelCommand] = ",
    )
}

pub fn parse_lean_project_command_inputs(
    contents: &FileContents,
) -> Result<Vec<ProjectCommandInput>, FormalProjectFactError> {
    command_input_entries_from_list(
        contents.as_ref(),
        "def modelCommandInputs : List (String × String × String × String × String × String × List String) := ",
    )
}

pub fn parse_quint_project_command_inputs(
    contents: &FileContents,
) -> Result<Vec<ProjectCommandInput>, FormalProjectFactError> {
    command_input_entries_from_list(
        contents.as_ref(),
        "val modelCommandInputs: List[ModelCommandInput] = ",
    )
}

pub fn parse_lean_project_command_errors(
    contents: &FileContents,
) -> Result<Vec<ProjectCommandError>, FormalProjectFactError> {
    command_error_entries_from_list(
        contents.as_ref(),
        "def modelCommandErrors : List (String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_command_errors(
    contents: &FileContents,
) -> Result<Vec<ProjectCommandError>, FormalProjectFactError> {
    command_error_entries_from_list(
        contents.as_ref(),
        "val modelCommandErrors: List[ModelCommandError] = ",
    )
}

pub fn parse_lean_project_read_models(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModel>, FormalProjectFactError> {
    read_model_entries_from_list(
        contents.as_ref(),
        "def modelReadModels : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_read_models(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModel>, FormalProjectFactError> {
    read_model_entries_from_list(
        contents.as_ref(),
        "val modelReadModels: List[ModelReadModel] = ",
    )
}

pub fn parse_lean_project_read_model_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModelDefinition>, FormalProjectFactError> {
    read_model_definition_entries_from_list(
        contents.as_ref(),
        "def modelReadModelDefinitions : List (String × String × String × Bool × List String × String × String) := ",
    )
}

pub fn parse_quint_project_read_model_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModelDefinition>, FormalProjectFactError> {
    read_model_definition_entries_from_list(
        contents.as_ref(),
        "val modelReadModelDefinitions: List[ModelReadModelDefinition] = ",
    )
}

pub fn parse_lean_project_read_model_fields(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModelField>, FormalProjectFactError> {
    read_model_field_entries_from_list(
        contents.as_ref(),
        "def modelReadModelFields : List (String × String × String × String × String × String × String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_read_model_fields(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModelField>, FormalProjectFactError> {
    read_model_field_entries_from_list(
        contents.as_ref(),
        "val modelReadModelFields: List[ModelReadModelField] = ",
    )
}

pub fn parse_lean_project_views(
    contents: &FileContents,
) -> Result<Vec<ProjectView>, FormalProjectFactError> {
    view_entries_from_list(
        contents.as_ref(),
        "def modelViews : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_views(
    contents: &FileContents,
) -> Result<Vec<ProjectView>, FormalProjectFactError> {
    view_entries_from_list(contents.as_ref(), "val modelViews: List[ModelView] = ")
}

pub fn parse_lean_project_view_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectViewDefinition>, FormalProjectFactError> {
    view_definition_entries_from_list(
        contents.as_ref(),
        "def modelViewDefinitions : List (String × String × String × List String × List String × List String × List String) := ",
    )
}

pub fn parse_quint_project_view_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectViewDefinition>, FormalProjectFactError> {
    view_definition_entries_from_list(
        contents.as_ref(),
        "val modelViewDefinitions: List[ModelViewDefinition] = ",
    )
}

pub fn parse_lean_project_view_controls(
    contents: &FileContents,
) -> Result<Vec<ProjectViewControl>, FormalProjectFactError> {
    view_control_entries_from_list(
        contents.as_ref(),
        "def modelViewControls : List (String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_view_controls(
    contents: &FileContents,
) -> Result<Vec<ProjectViewControl>, FormalProjectFactError> {
    view_control_entries_from_list(
        contents.as_ref(),
        "val modelViewControls: List[ModelViewControl] = ",
    )
}

pub fn parse_lean_project_view_fields(
    contents: &FileContents,
) -> Result<Vec<ProjectViewField>, FormalProjectFactError> {
    view_field_entries_from_list(
        contents.as_ref(),
        "def modelViewFields : List (String × String × String × String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_view_fields(
    contents: &FileContents,
) -> Result<Vec<ProjectViewField>, FormalProjectFactError> {
    view_field_entries_from_list(
        contents.as_ref(),
        "val modelViewFields: List[ModelViewField] = ",
    )
}

pub fn parse_lean_project_automations(
    contents: &FileContents,
) -> Result<Vec<ProjectAutomation>, FormalProjectFactError> {
    automation_entries_from_list(
        contents.as_ref(),
        "def modelAutomations : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_automations(
    contents: &FileContents,
) -> Result<Vec<ProjectAutomation>, FormalProjectFactError> {
    automation_entries_from_list(
        contents.as_ref(),
        "val modelAutomations: List[ModelAutomation] = ",
    )
}

pub fn parse_lean_project_automation_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectAutomationDefinition>, FormalProjectFactError> {
    automation_definition_entries_from_list(
        contents.as_ref(),
        "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := ",
    )
}

pub fn parse_quint_project_automation_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectAutomationDefinition>, FormalProjectFactError> {
    automation_definition_entries_from_list(
        contents.as_ref(),
        "val modelAutomationDefinitions: List[ModelAutomationDefinition] = ",
    )
}

pub fn parse_lean_project_translations(
    contents: &FileContents,
) -> Result<Vec<ProjectTranslation>, FormalProjectFactError> {
    translation_entries_from_list(
        contents.as_ref(),
        "def modelTranslations : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_translations(
    contents: &FileContents,
) -> Result<Vec<ProjectTranslation>, FormalProjectFactError> {
    translation_entries_from_list(
        contents.as_ref(),
        "val modelTranslations: List[ModelTranslation] = ",
    )
}

pub fn parse_lean_project_translation_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectTranslationDefinition>, FormalProjectFactError> {
    translation_definition_entries_from_list(
        contents.as_ref(),
        "def modelTranslationDefinitions : List (String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_translation_definitions(
    contents: &FileContents,
) -> Result<Vec<ProjectTranslationDefinition>, FormalProjectFactError> {
    translation_definition_entries_from_list(
        contents.as_ref(),
        "val modelTranslationDefinitions: List[ModelTranslationDefinition] = ",
    )
}

pub fn parse_lean_project_external_payloads(
    contents: &FileContents,
) -> Result<Vec<ProjectExternalPayload>, FormalProjectFactError> {
    external_payload_entries_from_list(
        contents.as_ref(),
        "def modelExternalPayloads : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_external_payloads(
    contents: &FileContents,
) -> Result<Vec<ProjectExternalPayload>, FormalProjectFactError> {
    external_payload_entries_from_list(
        contents.as_ref(),
        "val modelExternalPayloads: List[ModelExternalPayload] = ",
    )
}

pub fn parse_lean_project_external_payload_fields(
    contents: &FileContents,
) -> Result<Vec<ProjectExternalPayloadField>, FormalProjectFactError> {
    external_payload_field_entries_from_list(
        contents.as_ref(),
        "def modelExternalPayloadFields : List (String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_external_payload_fields(
    contents: &FileContents,
) -> Result<Vec<ProjectExternalPayloadField>, FormalProjectFactError> {
    external_payload_field_entries_from_list(
        contents.as_ref(),
        "val modelExternalPayloadFields: List[ModelExternalPayloadField] = ",
    )
}

pub fn parse_lean_project_events(
    contents: &FileContents,
) -> Result<Vec<ProjectEvent>, FormalProjectFactError> {
    event_entries_from_list(
        contents.as_ref(),
        "def modelEvents : List (String × String × String × String) := ",
    )
}

pub fn parse_quint_project_events(
    contents: &FileContents,
) -> Result<Vec<ProjectEvent>, FormalProjectFactError> {
    event_entries_from_list(contents.as_ref(), "val modelEvents: List[ModelEvent] = ")
}

pub fn parse_lean_project_event_attributes(
    contents: &FileContents,
) -> Result<Vec<ProjectEventAttribute>, FormalProjectFactError> {
    event_attribute_entries_from_list(
        contents.as_ref(),
        "def modelEventAttributes : List (String × String × String × String × String × String × String × String) := ",
    )
}

pub fn parse_quint_project_event_attributes(
    contents: &FileContents,
) -> Result<Vec<ProjectEventAttribute>, FormalProjectFactError> {
    event_attribute_entries_from_list(
        contents.as_ref(),
        "val modelEventAttributes: List[ModelEventAttribute] = ",
    )
}

pub fn add_project_scenario(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    scenario: NewProjectScenario,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_scenario_record(&scenario);
    let quint_record = quint_scenario_record(&scenario);
    let lean_definition_record = scenario
        .scenario_definition
        .as_ref()
        .map(lean_scenario_definition_record);
    let quint_definition_record = scenario
        .scenario_definition
        .as_ref()
        .map(quint_scenario_definition_record);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelScenarios : List (String × String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let scenarios = scenario_entries_from_list(
            &contents,
            "def modelScenarios : List (String × String × String × String) := ",
        )?;
        let contents = if let Some(record) = &lean_definition_record {
            append_record_if_missing(
                &contents,
                "def modelScenarioDefinitions : List (String × String × String × String × String × String × String × List String × List String × String × String × List String) := ",
                record,
            )?
        } else {
            contents
        };
        let scenario_definitions = parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
        let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
        replace_declaration(
            &contents,
            "def modelScenarios :",
            &format!(
                "def modelScenarios : List (String × String × String × String) := {}",
                lean_project_scenario_list(&scenarios)
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelScenariosAreDeclared :",
                &format!(
                    "theorem modelScenariosAreDeclared : modelScenarios.length = {} := rfl",
                    scenarios.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "def modelScenarioDefinitions :",
                &format!(
                    "def modelScenarioDefinitions : List (String × String × String × String × String × String × String × List String × List String × String × String × List String) := {}",
                    lean_project_scenario_definition_list(&scenario_definitions)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelScenarioDefinitionsAreDeclared :",
                &format!(
                    "theorem modelScenarioDefinitionsAreDeclared : modelScenarioDefinitions.length = {} := rfl",
                    scenario_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelScenarios: List[ModelScenario] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let scenarios =
            scenario_entries_from_list(&contents, "val modelScenarios: List[ModelScenario] = ")?;
        let contents = if let Some(record) = &quint_definition_record {
            append_record_if_missing(
                &contents,
                "val modelScenarioDefinitions: List[ModelScenarioDefinition] = ",
                record,
            )?
        } else {
            contents
        };
        let scenario_definitions =
            parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
        let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
        replace_declaration(
            &contents,
            "val modelScenarios:",
            &format!(
                "val modelScenarios: List[ModelScenario] = {}",
                quint_project_scenario_list(&scenarios)
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelScenariosAreDeclared =",
                &format!(
                    "val modelScenariosAreDeclared = modelScenarios.length() == {}",
                    scenarios.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelScenarioDefinitions:",
                &format!(
                    "val modelScenarioDefinitions: List[ModelScenarioDefinition] = {}",
                    quint_project_scenario_definition_list(&scenario_definitions)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelScenarioDefinitionsAreDeclared =",
                &format!(
                    "val modelScenarioDefinitionsAreDeclared = modelScenarioDefinitions.length() == {}",
                    scenario_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added {} scenario {} to project root",
            scenario.scenario_kind.as_str(),
            scenario.scenario.as_ref()
        ))?),
    ]))
}

pub fn add_project_data_flow(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    data_flow: NewProjectDataFlow,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_data_flow_record(&data_flow);
    let quint_record = quint_data_flow_record(&data_flow);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelDataFlows : List (String × String × String × String × String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let data_flows = data_flow_entries_from_list(
            &contents,
            "def modelDataFlows : List (String × String × String × String × String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "def modelDataFlows :",
            &format!(
                "def modelDataFlows : List (String × String × String × String × String × String × String) := {}",
                lean_project_data_flow_list(&data_flows)
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelDataFlowsAreDeclared :",
                &format!(
                    "theorem modelDataFlowsAreDeclared : modelDataFlows.length = {} := rfl",
                    data_flows.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions = parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs = parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions = parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields = parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes = parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelDataFlows: List[ModelDataFlow] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let data_flows =
            data_flow_entries_from_list(&contents, "val modelDataFlows: List[ModelDataFlow] = ")?;
        replace_declaration(
            &contents,
            "val modelDataFlows:",
            &format!(
                "val modelDataFlows: List[ModelDataFlow] = {}",
                quint_project_data_flow_list(&data_flows)
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelDataFlowsAreDeclared =",
                &format!(
                    "val modelDataFlowsAreDeclared = modelDataFlows.length() == {}",
                    data_flows.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls = parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added bit-level data flow {} to project root",
            data_flow.datum.as_ref()
        ))?),
    ]))
}

pub fn add_project_outcome(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    outcome: NewProjectOutcome,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_outcome_record(&outcome);
    let quint_record = quint_outcome_record(&outcome);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelOutcomes : List (String × String × String × List String × Bool) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let outcomes = outcome_entries_from_list(
            &contents,
            "def modelOutcomes : List (String × String × String × List String × Bool) := ",
        )?;
        replace_declaration(
            &contents,
            "def modelOutcomes :",
            &format!(
                "def modelOutcomes : List (String × String × String × List String × Bool) := {}",
                lean_project_outcome_list(&outcomes)
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelOutcomesAreDeclared :",
                &format!(
                    "theorem modelOutcomesAreDeclared : modelOutcomes.length = {} := rfl",
                    outcomes.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls = parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelOutcomes: List[ModelOutcome] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let outcomes =
            outcome_entries_from_list(&contents, "val modelOutcomes: List[ModelOutcome] = ")?;
        replace_declaration(
            &contents,
            "val modelOutcomes:",
            &format!(
                "val modelOutcomes: List[ModelOutcome] = {}",
                quint_project_outcome_list(&outcomes)
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelOutcomesAreDeclared =",
                &format!(
                    "val modelOutcomesAreDeclared = modelOutcomes.length() == {}",
                    outcomes.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls = parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added outcome {} to project root",
            outcome.outcome.as_ref()
        ))?),
    ]))
}

pub fn add_project_command(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    command: NewProjectCommand,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_command_record(&command);
    let quint_record = quint_command_record(&command);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelCommands : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        command
            .command_inputs
            .iter()
            .try_fold(contents, |contents, input| {
                append_record_if_missing(
                    &contents,
                    "def modelCommandInputs : List (String × String × String × String × String × String × List String) := ",
                    &lean_command_input_record(input),
                )
            })
    })
    .and_then(|contents| {
        command.command_errors.iter().try_fold(contents, |contents, error| {
            append_record_if_missing(
                &contents,
                "def modelCommandErrors : List (String × String × String × String × String × String) := ",
                &lean_command_error_record(error),
            )
        })
    })
    .and_then(|contents| {
        let commands = command_entries_from_list(
            &contents,
            "def modelCommands : List (String × String × String) := ",
        )?;
        let command_errors = command_error_entries_from_list(
            &contents,
            "def modelCommandErrors : List (String × String × String × String × String × String) := ",
        )?;
        let command_inputs = command_input_entries_from_list(
            &contents,
            "def modelCommandInputs : List (String × String × String × String × String × String × List String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelCommandsAreDeclared :",
            &format!(
                "theorem modelCommandsAreDeclared : modelCommands.length = {} := rfl",
                commands.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelCommandInputsAreDeclared :",
                &format!(
                    "theorem modelCommandInputsAreDeclared : modelCommandInputs.length = {} := rfl",
                    command_inputs.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelCommandErrorsAreDeclared :",
                &format!(
                    "theorem modelCommandErrorsAreDeclared : modelCommandErrors.length = {} := rfl",
                    command_errors.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions = parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors = parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let command_inputs = parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions = parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields = parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes = parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelCommands: List[ModelCommand] = ",
        &quint_record,
    )
    .and_then(|contents| {
        command
            .command_inputs
            .iter()
            .try_fold(contents, |contents, input| {
                append_record_if_missing(
                    &contents,
                    "val modelCommandInputs: List[ModelCommandInput] = ",
                    &quint_command_input_record(input),
                )
            })
    })
    .and_then(|contents| {
        command
            .command_errors
            .iter()
            .try_fold(contents, |contents, error| {
                append_record_if_missing(
                    &contents,
                    "val modelCommandErrors: List[ModelCommandError] = ",
                    &quint_command_error_record(error),
                )
            })
    })
    .and_then(|contents| {
        let commands =
            command_entries_from_list(&contents, "val modelCommands: List[ModelCommand] = ")?;
        let command_errors = command_error_entries_from_list(
            &contents,
            "val modelCommandErrors: List[ModelCommandError] = ",
        )?;
        let command_inputs = command_input_entries_from_list(
            &contents,
            "val modelCommandInputs: List[ModelCommandInput] = ",
        )?;
        replace_declaration(
            &contents,
            "val modelCommandsAreDeclared =",
            &format!(
                "val modelCommandsAreDeclared = modelCommands.length() == {}",
                commands.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelCommandInputsAreDeclared =",
                &format!(
                    "val modelCommandInputsAreDeclared = modelCommandInputs.length() == {}",
                    command_inputs.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelCommandErrorsAreDeclared =",
                &format!(
                    "val modelCommandErrorsAreDeclared = modelCommandErrors.length() == {}",
                    command_errors.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls = parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added command {} to project root",
            command.command.as_ref()
        ))?),
    ]))
}

pub fn add_project_read_model(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    read_model: NewProjectReadModel,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_read_model_record(&read_model);
    let quint_record = quint_read_model_record(&read_model);
    let lean_definition_records = read_model
        .read_model_definitions
        .iter()
        .map(lean_read_model_definition_record)
        .collect::<Vec<_>>();
    let lean_field_records = read_model
        .read_model_fields
        .iter()
        .map(lean_read_model_field_record)
        .collect::<Vec<_>>();
    let quint_definition_records = read_model
        .read_model_definitions
        .iter()
        .map(quint_read_model_definition_record)
        .collect::<Vec<_>>();
    let quint_field_records = read_model
        .read_model_fields
        .iter()
        .map(quint_read_model_field_record)
        .collect::<Vec<_>>();
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelReadModels : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        lean_definition_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "def modelReadModelDefinitions : List (String × String × String × Bool × List String × String × String) := ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        lean_field_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "def modelReadModelFields : List (String × String × String × String × String × String × String × String × String × String × String × String) := ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        let read_models = read_model_entries_from_list(
            &contents,
            "def modelReadModels : List (String × String × String) := ",
        )?;
        let read_model_definitions = read_model_definition_entries_from_list(
            &contents,
            "def modelReadModelDefinitions : List (String × String × String × Bool × List String × String × String) := ",
        )?;
        let read_model_fields = read_model_field_entries_from_list(
            &contents,
            "def modelReadModelFields : List (String × String × String × String × String × String × String × String × String × String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelReadModelsAreDeclared :",
            &format!(
                "theorem modelReadModelsAreDeclared : modelReadModels.length = {} := rfl",
                read_models.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelReadModelDefinitionsAreDeclared :",
                &format!(
                    "theorem modelReadModelDefinitionsAreDeclared : modelReadModelDefinitions.length = {} := rfl",
                    read_model_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelReadModelFieldsAreDeclared :",
                &format!(
                    "theorem modelReadModelFieldsAreDeclared : modelReadModelFields.length = {} := rfl",
                    read_model_fields.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions = parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelReadModels: List[ModelReadModel] = ",
        &quint_record,
    )
    .and_then(|contents| {
        quint_definition_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "val modelReadModelDefinitions: List[ModelReadModelDefinition] = ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        quint_field_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "val modelReadModelFields: List[ModelReadModelField] = ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        let read_models = read_model_entries_from_list(
            &contents,
            "val modelReadModels: List[ModelReadModel] = ",
        )?;
        let read_model_definitions = read_model_definition_entries_from_list(
            &contents,
            "val modelReadModelDefinitions: List[ModelReadModelDefinition] = ",
        )?;
        let read_model_fields = read_model_field_entries_from_list(
            &contents,
            "val modelReadModelFields: List[ModelReadModelField] = ",
        )?;
        replace_declaration(
            &contents,
            "val modelReadModelsAreDeclared =",
            &format!(
                "val modelReadModelsAreDeclared = modelReadModels.length() == {}",
                read_models.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelReadModelDefinitionsAreDeclared =",
                &format!(
                    "val modelReadModelDefinitionsAreDeclared = modelReadModelDefinitions.length() == {}",
                    read_model_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelReadModelFieldsAreDeclared =",
                &format!(
                    "val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == {}",
                    read_model_fields.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions = parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added read model {} to project root",
            read_model.read_model.as_ref()
        ))?),
    ]))
}

pub fn add_project_view(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    view: NewProjectView,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_view_record(&view);
    let quint_record = quint_view_record(&view);
    let lean_definition_record = view
        .view_definition
        .as_ref()
        .map(lean_view_definition_record);
    let quint_definition_record = view
        .view_definition
        .as_ref()
        .map(quint_view_definition_record);
    let lean_control_records = view
        .view_controls
        .iter()
        .map(lean_view_control_record)
        .collect::<Vec<_>>();
    let quint_control_records = view
        .view_controls
        .iter()
        .map(quint_view_control_record)
        .collect::<Vec<_>>();
    let lean_field_records = view
        .view_fields
        .iter()
        .map(lean_view_field_record)
        .collect::<Vec<_>>();
    let quint_field_records = view
        .view_fields
        .iter()
        .map(quint_view_field_record)
        .collect::<Vec<_>>();
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelViews : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        if let Some(record) = lean_definition_record.as_ref() {
            append_record_if_missing(
                &contents,
                "def modelViewDefinitions : List (String × String × String × List String × List String × List String × List String) := ",
                record,
            )
        } else {
            Ok(contents)
        }
    })
    .and_then(|contents| {
        lean_control_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "def modelViewControls : List (String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) := ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        lean_field_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "def modelViewFields : List (String × String × String × String × String × String × String × String × String) := ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        let views = view_entries_from_list(
            &contents,
            "def modelViews : List (String × String × String) := ",
        )?;
        let view_definitions = view_definition_entries_from_list(
            &contents,
            "def modelViewDefinitions : List (String × String × String × List String × List String × List String × List String) := ",
        )?;
        let view_controls = view_control_entries_from_list(
            &contents,
            "def modelViewControls : List (String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) := ",
        )?;
        let view_fields = view_field_entries_from_list(
            &contents,
            "def modelViewFields : List (String × String × String × String × String × String × String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelViewsAreDeclared :",
            &format!(
                "theorem modelViewsAreDeclared : modelViews.length = {} := rfl",
                views.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelViewDefinitionsAreDeclared :",
                &format!(
                    "theorem modelViewDefinitionsAreDeclared : modelViewDefinitions.length = {} := rfl",
                    view_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelViewControlsAreDeclared :",
                &format!(
                    "theorem modelViewControlsAreDeclared : modelViewControls.length = {} := rfl",
                    view_controls.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelViewFieldsAreDeclared :",
                &format!(
                    "theorem modelViewFieldsAreDeclared : modelViewFields.length = {} := rfl",
                    view_fields.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions = parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions = parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields = parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelViews: List[ModelView] = ",
        &quint_record,
    )
    .and_then(|contents| {
        if let Some(record) = quint_definition_record.as_ref() {
            append_record_if_missing(
                &contents,
                "val modelViewDefinitions: List[ModelViewDefinition] = ",
                record,
            )
        } else {
            Ok(contents)
        }
    })
    .and_then(|contents| {
        quint_control_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "val modelViewControls: List[ModelViewControl] = ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        quint_field_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "val modelViewFields: List[ModelViewField] = ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        let views = view_entries_from_list(&contents, "val modelViews: List[ModelView] = ")?;
        let view_definitions = view_definition_entries_from_list(
            &contents,
            "val modelViewDefinitions: List[ModelViewDefinition] = ",
        )?;
        let view_controls = view_control_entries_from_list(
            &contents,
            "val modelViewControls: List[ModelViewControl] = ",
        )?;
        let view_fields = view_field_entries_from_list(
            &contents,
            "val modelViewFields: List[ModelViewField] = ",
        )?;
        replace_declaration(
            &contents,
            "val modelViewsAreDeclared =",
            &format!(
                "val modelViewsAreDeclared = modelViews.length() == {}",
                views.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelViewDefinitionsAreDeclared =",
                &format!(
                    "val modelViewDefinitionsAreDeclared = modelViewDefinitions.length() == {}",
                    view_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelViewControlsAreDeclared =",
                &format!(
                    "val modelViewControlsAreDeclared = modelViewControls.length() == {}",
                    view_controls.len()
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelViewFieldsAreDeclared =",
                &format!(
                    "val modelViewFieldsAreDeclared = modelViewFields.length() == {}",
                    view_fields.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added view {} to project root",
            view.view.as_ref()
        ))?),
    ]))
}

pub fn add_project_automation(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    automation: NewProjectAutomation,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_automation_record(&automation);
    let quint_record = quint_automation_record(&automation);
    let lean_definition_record = automation
        .automation_definition
        .as_ref()
        .map(lean_automation_definition_record);
    let quint_definition_record = automation
        .automation_definition
        .as_ref()
        .map(quint_automation_definition_record);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelAutomations : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        if let Some(record) = lean_definition_record.as_deref() {
            append_record_if_missing(
                &contents,
                "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := ",
                record,
            )
        } else {
            Ok(contents)
        }
    })
    .and_then(|contents| {
        let automations = automation_entries_from_list(
            &contents,
            "def modelAutomations : List (String × String × String) := ",
        )?;
        let automation_definitions = automation_definition_entries_from_list(
            &contents,
            "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelAutomationsAreDeclared :",
            &format!(
                "theorem modelAutomationsAreDeclared : modelAutomations.length = {} := rfl",
                automations.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "def modelAutomationDefinitions :",
                &format!(
                    "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := {}",
                    lean_automation_definition_list(&automation_definitions)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelAutomationDefinitionsAreDeclared :",
                &format!(
                    "theorem modelAutomationDefinitionsAreDeclared : modelAutomationDefinitions.length = {} := rfl",
                    automation_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelAutomations: List[ModelAutomation] = ",
        &quint_record,
    )
    .and_then(|contents| {
        if let Some(record) = quint_definition_record.as_deref() {
            append_record_if_missing(
                &contents,
                "val modelAutomationDefinitions: List[ModelAutomationDefinition] = ",
                record,
            )
        } else {
            Ok(contents)
        }
    })
    .and_then(|contents| {
        let automations = automation_entries_from_list(
            &contents,
            "val modelAutomations: List[ModelAutomation] = ",
        )?;
        let automation_definitions = automation_definition_entries_from_list(
            &contents,
            "val modelAutomationDefinitions: List[ModelAutomationDefinition] = ",
        )?;
        replace_declaration(
            &contents,
            "val modelAutomationsAreDeclared =",
            &format!(
                "val modelAutomationsAreDeclared = modelAutomations.length() == {}",
                automations.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelAutomationDefinitions:",
                &format!(
                    "val modelAutomationDefinitions: List[ModelAutomationDefinition] = {}",
                    quint_automation_definition_list(&automation_definitions)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelAutomationDefinitionsAreDeclared =",
                &format!(
                    "val modelAutomationDefinitionsAreDeclared = modelAutomationDefinitions.length() == {}",
                    automation_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added automation {} to project root",
            automation.automation.as_ref()
        ))?),
    ]))
}

pub fn add_project_translation(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    translation: NewProjectTranslation,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_translation_record(&translation);
    let quint_record = quint_translation_record(&translation);
    let lean_definition_record = translation
        .translation_definition
        .as_ref()
        .map(lean_translation_definition_record);
    let quint_definition_record = translation
        .translation_definition
        .as_ref()
        .map(quint_translation_definition_record);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelTranslations : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        if let Some(record) = lean_definition_record.as_deref() {
            append_record_if_missing(
                &contents,
                "def modelTranslationDefinitions : List (String × String × String × String × String × String) := ",
                record,
            )
        } else {
            Ok(contents)
        }
    })
    .and_then(|contents| {
        let translations = translation_entries_from_list(
            &contents,
            "def modelTranslations : List (String × String × String) := ",
        )?;
        let translation_definitions = translation_definition_entries_from_list(
            &contents,
            "def modelTranslationDefinitions : List (String × String × String × String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelTranslationsAreDeclared :",
            &format!(
                "theorem modelTranslationsAreDeclared : modelTranslations.length = {} := rfl",
                translations.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "def modelTranslationDefinitions :",
                &format!(
                    "def modelTranslationDefinitions : List (String × String × String × String × String × String) := {}",
                    lean_translation_definition_list(&translation_definitions)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelTranslationDefinitionsAreDeclared :",
                &format!(
                    "theorem modelTranslationDefinitionsAreDeclared : modelTranslationDefinitions.length = {} := rfl",
                    translation_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelTranslations: List[ModelTranslation] = ",
        &quint_record,
    )
    .and_then(|contents| {
        if let Some(record) = quint_definition_record.as_deref() {
            append_record_if_missing(
                &contents,
                "val modelTranslationDefinitions: List[ModelTranslationDefinition] = ",
                record,
            )
        } else {
            Ok(contents)
        }
    })
    .and_then(|contents| {
        let translations = translation_entries_from_list(
            &contents,
            "val modelTranslations: List[ModelTranslation] = ",
        )?;
        let translation_definitions = translation_definition_entries_from_list(
            &contents,
            "val modelTranslationDefinitions: List[ModelTranslationDefinition] = ",
        )?;
        replace_declaration(
            &contents,
            "val modelTranslationsAreDeclared =",
            &format!(
                "val modelTranslationsAreDeclared = modelTranslations.length() == {}",
                translations.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelTranslationDefinitions:",
                &format!(
                    "val modelTranslationDefinitions: List[ModelTranslationDefinition] = {}",
                    quint_translation_definition_list(&translation_definitions)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelTranslationDefinitionsAreDeclared =",
                &format!(
                    "val modelTranslationDefinitionsAreDeclared = modelTranslationDefinitions.length() == {}",
                    translation_definitions.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added translation {} to project root",
            translation.translation.as_ref()
        ))?),
    ]))
}

pub fn add_project_external_payload(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    external_payload: NewProjectExternalPayload,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_external_payload_record(&external_payload);
    let quint_record = quint_external_payload_record(&external_payload);
    let lean_field_record = external_payload
        .external_payload_field
        .as_ref()
        .map(lean_external_payload_field_record);
    let quint_field_record = external_payload
        .external_payload_field
        .as_ref()
        .map(quint_external_payload_field_record);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelExternalPayloads : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let contents = if let Some(record) = &lean_field_record {
            append_record_if_missing(
                &contents,
                "def modelExternalPayloadFields : List (String × String × String × String × String × String) := ",
                record,
            )?
        } else {
            contents
        };
        let external_payloads = external_payload_entries_from_list(
            &contents,
            "def modelExternalPayloads : List (String × String × String) := ",
        )?;
        let external_payload_fields =
            parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
        replace_declaration(
            &contents,
            "theorem modelExternalPayloadsAreDeclared :",
            &format!(
                "theorem modelExternalPayloadsAreDeclared : modelExternalPayloads.length = {} := rfl",
                external_payloads.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "def modelExternalPayloadFields :",
                &format!(
                    "def modelExternalPayloadFields : List (String × String × String × String × String × String) := {}",
                    lean_external_payload_field_list(&external_payload_fields)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelExternalPayloadFieldsAreDeclared :",
                &format!(
                    "theorem modelExternalPayloadFieldsAreDeclared : modelExternalPayloadFields.length = {} := rfl",
                    external_payload_fields.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelExternalPayloads: List[ModelExternalPayload] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let contents = if let Some(record) = &quint_field_record {
            append_record_if_missing(
                &contents,
                "val modelExternalPayloadFields: List[ModelExternalPayloadField] = ",
                record,
            )?
        } else {
            contents
        };
        let external_payloads = external_payload_entries_from_list(
            &contents,
            "val modelExternalPayloads: List[ModelExternalPayload] = ",
        )?;
        let external_payload_fields =
            parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
        replace_declaration(
            &contents,
            "val modelExternalPayloadsAreDeclared =",
            &format!(
                "val modelExternalPayloadsAreDeclared = modelExternalPayloads.length() == {}",
                external_payloads.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelExternalPayloadFields:",
                &format!(
                    "val modelExternalPayloadFields: List[ModelExternalPayloadField] = {}",
                    quint_external_payload_field_list(&external_payload_fields)
                ),
            )
        })
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelExternalPayloadFieldsAreDeclared =",
                &format!(
                    "val modelExternalPayloadFieldsAreDeclared = modelExternalPayloadFields.length() == {}",
                    external_payload_fields.len()
                ),
            )
        })
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls =
                parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added external payload {} to project root",
            external_payload.external_payload.as_ref()
        ))?),
    ]))
}

pub fn add_project_stream(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    stream: NewProjectStream,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_stream_record(&stream);
    let quint_record = quint_stream_record(&stream);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelStreams : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let streams = stream_entries_from_list(
            &contents,
            "def modelStreams : List (String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelStreamsAreDeclared :",
            &format!(
                "theorem modelStreamsAreDeclared : modelStreams.length = {} := rfl",
                streams.len()
            ),
        )
        .and_then(|contents| {
            let scenarios = parse_lean_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_lean_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_lean_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_lean_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_lean_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_lean_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_lean_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls = parse_lean_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_lean_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_lean_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_lean_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_lean_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_lean_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_lean_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_lean_project_external_payload_fields_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_lean_project_event_attributes_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelStreams: List[ModelStream] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let streams =
            stream_entries_from_list(&contents, "val modelStreams: List[ModelStream] = ")?;
        replace_declaration(
            &contents,
            "val modelStreamsAreDeclared =",
            &format!(
                "val modelStreamsAreDeclared = modelStreams.length() == {}",
                streams.len()
            ),
        )
        .and_then(|contents| {
            let scenarios = parse_quint_project_scenarios_from_contents_or_empty(&contents);
            let scenario_definitions =
                parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
            let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
            let outcomes = parse_quint_project_outcomes_from_contents_or_empty(&contents);
            let command_errors =
                parse_quint_project_command_errors_from_contents_or_empty(&contents);
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let command_inputs =
                parse_quint_project_command_inputs_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let read_model_definitions =
                parse_quint_project_read_model_definitions_from_contents_or_empty(&contents);
            let read_model_fields =
                parse_quint_project_read_model_fields_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let view_definitions =
                parse_quint_project_view_definitions_from_contents_or_empty(&contents);
            let view_controls = parse_quint_project_view_controls_from_contents_or_empty(&contents);
            let view_fields = parse_quint_project_view_fields_from_contents_or_empty(&contents);
            let automations = parse_quint_project_automations_from_contents_or_empty(&contents);
            let automation_definitions =
                parse_quint_project_automation_definitions_from_contents_or_empty(&contents);
            let translations = parse_quint_project_translations_from_contents_or_empty(&contents);
            let translation_definitions =
                parse_quint_project_translation_definitions_from_contents_or_empty(&contents);
            let external_payloads =
                parse_quint_project_external_payloads_from_contents_or_empty(&contents);
            let external_payload_fields =
                parse_quint_project_external_payload_fields_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            let event_attributes =
                parse_quint_project_event_attributes_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added stream {} to project root",
            stream.stream.as_ref()
        ))?),
    ]))
}

pub fn add_project_event(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    event: NewProjectEvent,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_event_record(&event);
    let quint_record = quint_event_record(&event);
    let lean_attribute_records = event
        .event_attributes
        .iter()
        .map(lean_event_attribute_record)
        .collect::<Vec<_>>();
    let quint_attribute_records = event
        .event_attributes
        .iter()
        .map(quint_event_attribute_record)
        .collect::<Vec<_>>();
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelEvents : List (String × String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        lean_attribute_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "def modelEventAttributes : List (String × String × String × String × String × String × String × String) := ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        let scenarios = scenario_entries_from_list(
            &contents,
            "def modelScenarios : List (String × String × String × String) := ",
        )?;
        let scenario_definitions =
            parse_lean_project_scenario_definitions_from_contents_or_empty(&contents);
        let data_flows = parse_lean_project_data_flows_from_contents_or_empty(&contents);
        let outcomes = outcome_entries_from_list(
            &contents,
            "def modelOutcomes : List (String × String × String × List String × Bool) := ",
        )?;
        let command_errors = parse_lean_project_command_errors_from_contents_or_empty(&contents);
        let commands = command_entries_from_list(
            &contents,
            "def modelCommands : List (String × String × String) := ",
        )?;
        let command_inputs = command_input_entries_from_list(
            &contents,
            "def modelCommandInputs : List (String × String × String × String × String × String × List String) := ",
        )?;
        let read_models = read_model_entries_from_list(
            &contents,
            "def modelReadModels : List (String × String × String) := ",
        )?;
        let read_model_definitions = read_model_definition_entries_from_list(
            &contents,
            "def modelReadModelDefinitions : List (String × String × String × Bool × List String × String × String) := ",
        )?;
        let read_model_fields = read_model_field_entries_from_list(
            &contents,
            "def modelReadModelFields : List (String × String × String × String × String × String × String × String × String × String × String × String) := ",
        )?;
        let views = view_entries_from_list(
            &contents,
            "def modelViews : List (String × String × String) := ",
        )?;
        let view_definitions = view_definition_entries_from_list(
            &contents,
            "def modelViewDefinitions : List (String × String × String × List String × List String × List String × List String) := ",
        )?;
        let view_controls = view_control_entries_from_list(
            &contents,
            "def modelViewControls : List (String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) := ",
        )?;
        let view_fields = view_field_entries_from_list(
            &contents,
            "def modelViewFields : List (String × String × String × String × String × String × String × String × String) := ",
        )?;
        let automations = automation_entries_from_list(
            &contents,
            "def modelAutomations : List (String × String × String) := ",
        )?;
        let automation_definitions = automation_definition_entries_from_list(
            &contents,
            "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := ",
        )?;
        let translations = translation_entries_from_list(
            &contents,
            "def modelTranslations : List (String × String × String) := ",
        )?;
        let translation_definitions = translation_definition_entries_from_list(
            &contents,
            "def modelTranslationDefinitions : List (String × String × String × String × String × String) := ",
        )?;
        let external_payloads = external_payload_entries_from_list(
            &contents,
            "def modelExternalPayloads : List (String × String × String) := ",
        )?;
        let external_payload_fields = external_payload_field_entries_from_list(
            &contents,
            "def modelExternalPayloadFields : List (String × String × String × String × String × String) := ",
        )?;
        let streams = stream_entries_from_list(
            &contents,
            "def modelStreams : List (String × String × String) := ",
        )?;
        let events = event_entries_from_list(
            &contents,
            "def modelEvents : List (String × String × String × String) := ",
        )?;
        let event_attributes = event_attribute_entries_from_list(
            &contents,
            "def modelEventAttributes : List (String × String × String × String × String × String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelEventsAreDeclared :",
            &format!(
                "theorem modelEventsAreDeclared : modelEvents.length = {} := rfl",
                events.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "theorem modelEventAttributesAreDeclared :",
                &format!(
                    "theorem modelEventAttributesAreDeclared : modelEventAttributes.length = {} := rfl",
                    event_attributes.len()
                ),
            )
        })
        .and_then(|contents| {
            update_lean_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelEvents: List[ModelEvent] = ",
        &quint_record,
    )
    .and_then(|contents| {
        quint_attribute_records
            .iter()
            .try_fold(contents, |contents, record| {
                append_record_if_missing(
                    &contents,
                    "val modelEventAttributes: List[ModelEventAttribute] = ",
                    record,
                )
            })
    })
    .and_then(|contents| {
        let scenarios =
            scenario_entries_from_list(&contents, "val modelScenarios: List[ModelScenario] = ")?;
        let scenario_definitions =
            parse_quint_project_scenario_definitions_from_contents_or_empty(&contents);
        let data_flows = parse_quint_project_data_flows_from_contents_or_empty(&contents);
        let outcomes =
            outcome_entries_from_list(&contents, "val modelOutcomes: List[ModelOutcome] = ")?;
        let command_errors = parse_quint_project_command_errors_from_contents_or_empty(&contents);
        let commands =
            command_entries_from_list(&contents, "val modelCommands: List[ModelCommand] = ")?;
        let command_inputs = command_input_entries_from_list(
            &contents,
            "val modelCommandInputs: List[ModelCommandInput] = ",
        )?;
        let read_models = read_model_entries_from_list(
            &contents,
            "val modelReadModels: List[ModelReadModel] = ",
        )?;
        let read_model_definitions = read_model_definition_entries_from_list(
            &contents,
            "val modelReadModelDefinitions: List[ModelReadModelDefinition] = ",
        )?;
        let read_model_fields = read_model_field_entries_from_list(
            &contents,
            "val modelReadModelFields: List[ModelReadModelField] = ",
        )?;
        let views = view_entries_from_list(&contents, "val modelViews: List[ModelView] = ")?;
        let view_definitions = view_definition_entries_from_list(
            &contents,
            "val modelViewDefinitions: List[ModelViewDefinition] = ",
        )?;
        let view_controls = view_control_entries_from_list(
            &contents,
            "val modelViewControls: List[ModelViewControl] = ",
        )?;
        let view_fields = view_field_entries_from_list(
            &contents,
            "val modelViewFields: List[ModelViewField] = ",
        )?;
        let automations = automation_entries_from_list(
            &contents,
            "val modelAutomations: List[ModelAutomation] = ",
        )?;
        let automation_definitions = automation_definition_entries_from_list(
            &contents,
            "val modelAutomationDefinitions: List[ModelAutomationDefinition] = ",
        )?;
        let translations = translation_entries_from_list(
            &contents,
            "val modelTranslations: List[ModelTranslation] = ",
        )?;
        let translation_definitions = translation_definition_entries_from_list(
            &contents,
            "val modelTranslationDefinitions: List[ModelTranslationDefinition] = ",
        )?;
        let external_payloads = external_payload_entries_from_list(
            &contents,
            "val modelExternalPayloads: List[ModelExternalPayload] = ",
        )?;
        let external_payload_fields = external_payload_field_entries_from_list(
            &contents,
            "val modelExternalPayloadFields: List[ModelExternalPayloadField] = ",
        )?;
        let streams =
            stream_entries_from_list(&contents, "val modelStreams: List[ModelStream] = ")?;
        let events = event_entries_from_list(&contents, "val modelEvents: List[ModelEvent] = ")?;
        let event_attributes = event_attribute_entries_from_list(
            &contents,
            "val modelEventAttributes: List[ModelEventAttribute] = ",
        )?;
        replace_declaration(
            &contents,
            "val modelEventsAreDeclared =",
            &format!(
                "val modelEventsAreDeclared = modelEvents.length() == {}",
                events.len()
            ),
        )
        .and_then(|contents| {
            replace_declaration(
                &contents,
                "val modelEventAttributesAreDeclared =",
                &format!(
                    "val modelEventAttributesAreDeclared = modelEventAttributes.length() == {}",
                    event_attributes.len()
                ),
            )
        })
        .and_then(|contents| {
            update_quint_digest(
                &contents,
                ProjectDigestInventories {
                    scenarios: &scenarios,
                    scenario_definitions: &scenario_definitions,
                    data_flows: &data_flows,
                    outcomes: &outcomes,
                    command_errors: &command_errors,
                    commands: &commands,
                    command_inputs: &command_inputs,
                    read_models: &read_models,
                    read_model_definitions: &read_model_definitions,
                    read_model_fields: &read_model_fields,
                    views: &views,
                    view_definitions: &view_definitions,
                    view_controls: &view_controls,
                    view_fields: &view_fields,
                    automations: &automations,
                    automation_definitions: &automation_definitions,
                    translations: &translations,
                    translation_definitions: &translation_definitions,
                    external_payloads: &external_payloads,
                    external_payload_fields: &external_payload_fields,
                    streams: &streams,
                    events: &events,
                    event_attributes: &event_attributes,
                },
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added event {} to project root",
            event.event.as_ref()
        ))?),
    ]))
}

#[derive(Debug)]
pub struct FormalProjectFactError {
    message: String,
}

impl FormalProjectFactError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for FormalProjectFactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for FormalProjectFactError {}

fn append_record_if_missing(
    contents: &str,
    marker: &str,
    record: &str,
) -> Result<String, FormalProjectFactError> {
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
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;

    if replaced {
        Ok(join_lines_preserving_trailing_newline(contents, lines))
    } else {
        Err(FormalProjectFactError::new(format!(
            "formal project artifact is missing declaration {marker}"
        )))
    }
}

fn append_list_record_if_missing(
    current_list: &str,
    record: &str,
) -> Result<String, FormalProjectFactError> {
    let trimmed = current_list.trim();
    if trimmed == "[]" {
        return Ok(format!("[{record}]"));
    }
    let existing = trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .ok_or_else(|| {
            FormalProjectFactError::new("formal project list declaration is malformed")
        })?;
    if split_top_level_records(trimmed)?
        .iter()
        .any(|entry| entry == record)
    {
        Ok(trimmed.to_owned())
    } else {
        Ok(format!("[{existing},{record}]"))
    }
}

fn replace_declaration(
    contents: &str,
    marker: &str,
    replacement: &str,
) -> Result<String, FormalProjectFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if declaration.starts_with(marker) {
                replaced = true;
                format!("{indentation}{replacement}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>();

    if replaced {
        Ok(join_lines_preserving_trailing_newline(contents, lines))
    } else {
        Err(FormalProjectFactError::new(format!(
            "formal project artifact is missing declaration {marker}"
        )))
    }
}

fn stream_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectStream>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut streams = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project stream record is malformed",
                ))
            } else {
                Ok(ProjectStream {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    stream: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    streams.sort();
    streams.dedup();
    Ok(streams)
}

fn parse_lean_project_commands_from_contents_or_empty(contents: &str) -> Vec<ProjectCommand> {
    command_entries_from_list(
        contents,
        "def modelCommands : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_command_inputs_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectCommandInput> {
    command_input_entries_from_list(
        contents,
        "def modelCommandInputs : List (String × String × String × String × String × String × List String) := ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_scenarios_from_contents_or_empty(contents: &str) -> Vec<ProjectScenario> {
    scenario_entries_from_list(
        contents,
        "def modelScenarios : List (String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_scenarios_from_contents_or_empty(contents: &str) -> Vec<ProjectScenario> {
    scenario_entries_from_list(contents, "val modelScenarios: List[ModelScenario] = ")
        .unwrap_or_default()
}

fn parse_lean_project_scenario_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectScenarioDefinition> {
    scenario_definition_entries_from_list(
        contents,
        "def modelScenarioDefinitions : List (String × String × String × String × String × String × String × List String × List String × String × String × List String) := ",
        ScenarioDefinitionSyntax::Lean,
    )
    .unwrap_or_default()
}

fn parse_quint_project_scenario_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectScenarioDefinition> {
    scenario_definition_entries_from_list(
        contents,
        "val modelScenarioDefinitions: List[ModelScenarioDefinition] = ",
        ScenarioDefinitionSyntax::Quint,
    )
    .unwrap_or_default()
}

fn parse_lean_project_data_flows_from_contents_or_empty(contents: &str) -> Vec<ProjectDataFlow> {
    data_flow_entries_from_list(
        contents,
        "def modelDataFlows : List (String × String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_data_flows_from_contents_or_empty(contents: &str) -> Vec<ProjectDataFlow> {
    data_flow_entries_from_list(contents, "val modelDataFlows: List[ModelDataFlow] = ")
        .unwrap_or_default()
}

fn parse_lean_project_outcomes_from_contents_or_empty(contents: &str) -> Vec<ProjectOutcome> {
    outcome_entries_from_list(
        contents,
        "def modelOutcomes : List (String × String × String × List String × Bool) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_outcomes_from_contents_or_empty(contents: &str) -> Vec<ProjectOutcome> {
    outcome_entries_from_list(contents, "val modelOutcomes: List[ModelOutcome] = ")
        .unwrap_or_default()
}

fn parse_quint_project_commands_from_contents_or_empty(contents: &str) -> Vec<ProjectCommand> {
    command_entries_from_list(contents, "val modelCommands: List[ModelCommand] = ")
        .unwrap_or_default()
}

fn parse_quint_project_command_inputs_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectCommandInput> {
    command_input_entries_from_list(
        contents,
        "val modelCommandInputs: List[ModelCommandInput] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_command_errors_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectCommandError> {
    command_error_entries_from_list(
        contents,
        "def modelCommandErrors : List (String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_command_errors_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectCommandError> {
    command_error_entries_from_list(
        contents,
        "val modelCommandErrors: List[ModelCommandError] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_read_models_from_contents_or_empty(contents: &str) -> Vec<ProjectReadModel> {
    read_model_entries_from_list(
        contents,
        "def modelReadModels : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_read_models_from_contents_or_empty(contents: &str) -> Vec<ProjectReadModel> {
    read_model_entries_from_list(contents, "val modelReadModels: List[ModelReadModel] = ")
        .unwrap_or_default()
}

fn parse_lean_project_read_model_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectReadModelDefinition> {
    read_model_definition_entries_from_list(
        contents,
        "def modelReadModelDefinitions : List (String × String × String × Bool × List String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_read_model_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectReadModelDefinition> {
    read_model_definition_entries_from_list(
        contents,
        "val modelReadModelDefinitions: List[ModelReadModelDefinition] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_read_model_fields_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectReadModelField> {
    read_model_field_entries_from_list(
        contents,
        "def modelReadModelFields : List (String × String × String × String × String × String × String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_read_model_fields_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectReadModelField> {
    read_model_field_entries_from_list(
        contents,
        "val modelReadModelFields: List[ModelReadModelField] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_views_from_contents_or_empty(contents: &str) -> Vec<ProjectView> {
    view_entries_from_list(
        contents,
        "def modelViews : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_views_from_contents_or_empty(contents: &str) -> Vec<ProjectView> {
    view_entries_from_list(contents, "val modelViews: List[ModelView] = ").unwrap_or_default()
}

fn parse_lean_project_view_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectViewDefinition> {
    view_definition_entries_from_list(
        contents,
        "def modelViewDefinitions : List (String × String × String × List String × List String × List String × List String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_view_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectViewDefinition> {
    view_definition_entries_from_list(
        contents,
        "val modelViewDefinitions: List[ModelViewDefinition] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_view_controls_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectViewControl> {
    view_control_entries_from_list(
        contents,
        "def modelViewControls : List (String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_view_controls_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectViewControl> {
    view_control_entries_from_list(contents, "val modelViewControls: List[ModelViewControl] = ")
        .unwrap_or_default()
}

fn parse_lean_project_view_fields_from_contents_or_empty(contents: &str) -> Vec<ProjectViewField> {
    view_field_entries_from_list(
        contents,
        "def modelViewFields : List (String × String × String × String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_view_fields_from_contents_or_empty(contents: &str) -> Vec<ProjectViewField> {
    view_field_entries_from_list(contents, "val modelViewFields: List[ModelViewField] = ")
        .unwrap_or_default()
}

fn parse_lean_project_automations_from_contents_or_empty(contents: &str) -> Vec<ProjectAutomation> {
    automation_entries_from_list(
        contents,
        "def modelAutomations : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_automations_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectAutomation> {
    automation_entries_from_list(contents, "val modelAutomations: List[ModelAutomation] = ")
        .unwrap_or_default()
}

fn parse_lean_project_automation_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectAutomationDefinition> {
    automation_definition_entries_from_list(
        contents,
        "def modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_automation_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectAutomationDefinition> {
    automation_definition_entries_from_list(
        contents,
        "val modelAutomationDefinitions: List[ModelAutomationDefinition] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_translations_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectTranslation> {
    translation_entries_from_list(
        contents,
        "def modelTranslations : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_translations_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectTranslation> {
    translation_entries_from_list(contents, "val modelTranslations: List[ModelTranslation] = ")
        .unwrap_or_default()
}

fn parse_lean_project_translation_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectTranslationDefinition> {
    translation_definition_entries_from_list(
        contents,
        "def modelTranslationDefinitions : List (String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_translation_definitions_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectTranslationDefinition> {
    translation_definition_entries_from_list(
        contents,
        "val modelTranslationDefinitions: List[ModelTranslationDefinition] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_external_payloads_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectExternalPayload> {
    external_payload_entries_from_list(
        contents,
        "def modelExternalPayloads : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_external_payloads_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectExternalPayload> {
    external_payload_entries_from_list(
        contents,
        "val modelExternalPayloads: List[ModelExternalPayload] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_streams_from_contents_or_empty(contents: &str) -> Vec<ProjectStream> {
    stream_entries_from_list(
        contents,
        "def modelStreams : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_streams_from_contents_or_empty(contents: &str) -> Vec<ProjectStream> {
    stream_entries_from_list(contents, "val modelStreams: List[ModelStream] = ").unwrap_or_default()
}

fn command_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectCommand>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut commands = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project command record is malformed",
                ))
            } else {
                Ok(ProjectCommand {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    command: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    commands.sort();
    commands.dedup();
    Ok(commands)
}

fn command_input_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectCommandInput>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut command_inputs = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 7 {
                Err(FormalProjectFactError::new(
                    "formal project command input record is malformed",
                ))
            } else {
                Ok(ProjectCommandInput {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    command: strings[2].clone(),
                    input: strings[3].clone(),
                    source_kind: strings[4].clone(),
                    source_description: strings[5].clone(),
                    provenance_chain: strings[6..].to_vec(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    command_inputs.sort();
    command_inputs.dedup();
    Ok(command_inputs)
}

fn command_error_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectCommandError>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut command_errors = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 6 {
                Err(FormalProjectFactError::new(
                    "formal project command error record is malformed",
                ))
            } else {
                Ok(ProjectCommandError {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    command: strings[2].clone(),
                    error: strings[3].clone(),
                    scenario: strings[4].clone(),
                    recovery: strings[5].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    command_errors.sort();
    command_errors.dedup();
    Ok(command_errors)
}

fn scenario_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectScenario>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut scenarios = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 4 {
                Err(FormalProjectFactError::new(
                    "formal project scenario record is malformed",
                ))
            } else {
                Ok(ProjectScenario {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    scenario_kind: strings[2].clone(),
                    scenario: strings[3].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    scenarios.sort();
    scenarios.dedup();
    Ok(scenarios)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ScenarioDefinitionSyntax {
    Lean,
    Quint,
}

fn scenario_definition_entries_from_list(
    contents: &str,
    marker: &str,
    syntax: ScenarioDefinitionSyntax,
) -> Result<Vec<ProjectScenarioDefinition>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut scenario_definitions = split_top_level_records(list)?
        .into_iter()
        .map(|record| scenario_definition_entry_from_record(&record, syntax))
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    scenario_definitions.sort();
    scenario_definitions.dedup();
    Ok(scenario_definitions)
}

fn scenario_definition_entry_from_record(
    record: &str,
    syntax: ScenarioDefinitionSyntax,
) -> Result<ProjectScenarioDefinition, FormalProjectFactError> {
    let fields = split_top_level_fields(record)?;
    if fields.len() != 12 {
        return Err(FormalProjectFactError::new(
            "formal project scenario definition record is malformed",
        ));
    }

    let value = |index: usize| -> Result<&str, FormalProjectFactError> {
        match syntax {
            ScenarioDefinitionSyntax::Lean => Ok(fields[index].trim()),
            ScenarioDefinitionSyntax::Quint => record_field_value(&fields[index]),
        }
    };

    Ok(ProjectScenarioDefinition {
        workflow_slug: single_quoted_string(value(0)?)?,
        slice_slug: single_quoted_string(value(1)?)?,
        scenario_kind: single_quoted_string(value(2)?)?,
        scenario: single_quoted_string(value(3)?)?,
        given: single_quoted_string(value(4)?)?,
        when: single_quoted_string(value(5)?)?,
        then: single_quoted_string(value(6)?)?,
        read_streams: string_list_value(value(7)?)?,
        written_streams: string_list_value(value(8)?)?,
        contract_kind: single_quoted_string(value(9)?)?,
        covered_definition: single_quoted_string(value(10)?)?,
        error_references: string_list_value(value(11)?)?,
    })
}

fn data_flow_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectDataFlow>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut data_flows = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 7 {
                Err(FormalProjectFactError::new(
                    "formal project data-flow record is malformed",
                ))
            } else {
                Ok(ProjectDataFlow {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    datum: strings[2].clone(),
                    source: strings[3].clone(),
                    transformation: strings[4].clone(),
                    target: strings[5].clone(),
                    bit_encoding: strings[6].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    data_flows.sort();
    data_flows.dedup();
    Ok(data_flows)
}

fn outcome_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectOutcome>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut outcomes = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 4 {
                Err(FormalProjectFactError::new(
                    "formal project outcome record is malformed",
                ))
            } else {
                Ok(ProjectOutcome {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    outcome: strings[2].clone(),
                    events: strings[3..].to_vec(),
                    externally_relevant: record_bool_tail(&record)?,
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    outcomes.sort();
    outcomes.dedup();
    Ok(outcomes)
}

fn read_model_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectReadModel>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut read_models = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project read model record is malformed",
                ))
            } else {
                Ok(ProjectReadModel {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    read_model: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    read_models.sort();
    read_models.dedup();
    Ok(read_models)
}

fn read_model_definition_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectReadModelDefinition>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut read_model_definitions = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 5 {
                Err(FormalProjectFactError::new(
                    "formal project read model definition record is malformed",
                ))
            } else {
                let transitive = record_read_model_definition_transitive(&record)?;
                let relationship_end = strings.len() - 2;
                Ok(ProjectReadModelDefinition {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    read_model: strings[2].clone(),
                    transitive,
                    relationship_fields: strings[3..relationship_end].to_vec(),
                    transitive_rule: strings[relationship_end].clone(),
                    example_scenario_name: strings[relationship_end + 1].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    read_model_definitions.sort();
    read_model_definitions.dedup();
    Ok(read_model_definitions)
}

fn record_read_model_definition_transitive(record: &str) -> Result<bool, FormalProjectFactError> {
    if record.contains(", true,") || record.contains("transitive: true") {
        Ok(true)
    } else if record.contains(", false,") || record.contains("transitive: false") {
        Ok(false)
    } else {
        Err(FormalProjectFactError::new(
            "formal project read model definition transitive field is malformed",
        ))
    }
}

fn read_model_field_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectReadModelField>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut read_model_fields = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 12 {
                Err(FormalProjectFactError::new(
                    "formal project read model field record is malformed",
                ))
            } else {
                Ok(ProjectReadModelField {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    read_model: strings[2].clone(),
                    field: strings[3].clone(),
                    source_kind: strings[4].clone(),
                    source_event: strings[5].clone(),
                    source_attribute: strings[6].clone(),
                    derivation_rule: strings[7].clone(),
                    absence_event: strings[8].clone(),
                    derivation_scenario_name: strings[9].clone(),
                    absence_scenario_name: strings[10].clone(),
                    provenance: strings[11].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    read_model_fields.sort();
    read_model_fields.dedup();
    Ok(read_model_fields)
}

fn view_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectView>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut views = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project view record is malformed",
                ))
            } else {
                Ok(ProjectView {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    view: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    views.sort();
    views.dedup();
    Ok(views)
}

fn view_definition_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectViewDefinition>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut definitions = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                return Err(FormalProjectFactError::new(
                    "formal project view definition record is malformed",
                ));
            }
            let lists = string_lists(&record)?;
            if lists.len() != 4 {
                return Err(FormalProjectFactError::new(
                    "formal project view definition list fields are malformed",
                ));
            }
            Ok(ProjectViewDefinition {
                workflow_slug: strings[0].clone(),
                slice_slug: strings[1].clone(),
                view: strings[2].clone(),
                read_models: lists[0].clone(),
                sketch_tokens: lists[1].clone(),
                local_states: lists[2].clone(),
                filters: lists[3].clone(),
            })
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    definitions.sort();
    definitions.dedup();
    Ok(definitions)
}

fn view_control_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectViewControl>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut controls = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let values = record_values(&record)?;
            if values.len() != 19 {
                return Err(FormalProjectFactError::new(
                    "formal project view control record is malformed",
                ));
            }
            Ok(ProjectViewControl {
                workflow_slug: single_quoted_string(&values[0])?,
                slice_slug: single_quoted_string(&values[1])?,
                view: single_quoted_string(&values[2])?,
                control: single_quoted_string(&values[3])?,
                command: single_quoted_string(&values[4])?,
                input: single_quoted_string(&values[5])?,
                input_source_kind: single_quoted_string(&values[6])?,
                input_source_description: single_quoted_string(&values[7])?,
                input_sketch_token: single_quoted_string(&values[8])?,
                input_visible_to_actor: bool_value(&values[9])?,
                input_decision_field: bool_value(&values[10])?,
                handled_errors: string_list_value(&values[11])?,
                recovery_behavior: single_quoted_string(&values[12])?,
                control_sketch_token: single_quoted_string(&values[13])?,
                navigation_type: single_quoted_string(&values[14])?,
                navigation_target: single_quoted_string(&values[15])?,
                external_workflow: single_quoted_string(&values[16])?,
                external_system: single_quoted_string(&values[17])?,
                handoff_contract: single_quoted_string(&values[18])?,
            })
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    controls.sort();
    controls.dedup();
    Ok(controls)
}

fn view_field_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectViewField>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut view_fields = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 9 {
                Err(FormalProjectFactError::new(
                    "formal project view field record is malformed",
                ))
            } else {
                Ok(ProjectViewField {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    view: strings[2].clone(),
                    field: strings[3].clone(),
                    source_kind: strings[4].clone(),
                    source_read_model: strings[5].clone(),
                    source_field: strings[6].clone(),
                    provenance: strings[7].clone(),
                    bit_encoding: strings[8].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    view_fields.sort();
    view_fields.dedup();
    Ok(view_fields)
}

fn automation_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectAutomation>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut automations = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project automation record is malformed",
                ))
            } else {
                Ok(ProjectAutomation {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    automation: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    automations.sort();
    automations.dedup();
    Ok(automations)
}

fn automation_definition_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectAutomationDefinition>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut definitions = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 6 {
                Err(FormalProjectFactError::new(
                    "formal project automation definition record is malformed",
                ))
            } else {
                let reaction_index = strings.len() - 1;
                Ok(ProjectAutomationDefinition {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    automation: strings[2].clone(),
                    trigger: strings[3].clone(),
                    command: strings[4].clone(),
                    handled_errors: strings[5..reaction_index].to_vec(),
                    reaction: strings[reaction_index].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    definitions.sort();
    definitions.dedup();
    Ok(definitions)
}

fn translation_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectTranslation>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut translations = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project translation record is malformed",
                ))
            } else {
                Ok(ProjectTranslation {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    translation: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    translations.sort();
    translations.dedup();
    Ok(translations)
}

fn translation_definition_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectTranslationDefinition>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut definitions = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() != 6 {
                Err(FormalProjectFactError::new(
                    "formal project translation definition record is malformed",
                ))
            } else {
                Ok(ProjectTranslationDefinition {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    translation: strings[2].clone(),
                    external_event: strings[3].clone(),
                    payload_contract: strings[4].clone(),
                    command: strings[5].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    definitions.sort();
    definitions.dedup();
    Ok(definitions)
}

fn external_payload_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectExternalPayload>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut external_payloads = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project external payload record is malformed",
                ))
            } else {
                Ok(ProjectExternalPayload {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    external_payload: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    external_payloads.sort();
    external_payloads.dedup();
    Ok(external_payloads)
}

fn external_payload_field_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectExternalPayloadField>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut fields = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 6 {
                Err(FormalProjectFactError::new(
                    "formal project external payload field record is malformed",
                ))
            } else {
                Ok(ProjectExternalPayloadField {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    external_payload: strings[2].clone(),
                    field: strings[3].clone(),
                    provenance: strings[4].clone(),
                    bit_encoding: strings[5].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    fields.sort();
    fields.dedup();
    Ok(fields)
}

fn parse_lean_project_external_payload_fields_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectExternalPayloadField> {
    external_payload_field_entries_from_list(
        contents,
        "def modelExternalPayloadFields : List (String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_external_payload_fields_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectExternalPayloadField> {
    external_payload_field_entries_from_list(
        contents,
        "val modelExternalPayloadFields: List[ModelExternalPayloadField] = ",
    )
    .unwrap_or_default()
}

fn parse_lean_project_events_from_contents_or_empty(contents: &str) -> Vec<ProjectEvent> {
    event_entries_from_list(
        contents,
        "def modelEvents : List (String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_events_from_contents_or_empty(contents: &str) -> Vec<ProjectEvent> {
    event_entries_from_list(contents, "val modelEvents: List[ModelEvent] = ").unwrap_or_default()
}

fn parse_lean_project_event_attributes_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectEventAttribute> {
    event_attribute_entries_from_list(
        contents,
        "def modelEventAttributes : List (String × String × String × String × String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_event_attributes_from_contents_or_empty(
    contents: &str,
) -> Vec<ProjectEventAttribute> {
    event_attribute_entries_from_list(
        contents,
        "val modelEventAttributes: List[ModelEventAttribute] = ",
    )
    .unwrap_or_default()
}

fn event_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectEvent>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut events = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 4 {
                Err(FormalProjectFactError::new(
                    "formal project event record is malformed",
                ))
            } else {
                Ok(ProjectEvent {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    event: strings[2].clone(),
                    stream: strings[3].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    events.sort();
    events.dedup();
    Ok(events)
}

fn event_attribute_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectEventAttribute>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut event_attributes = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 8 {
                Err(FormalProjectFactError::new(
                    "formal project event attribute record is malformed",
                ))
            } else {
                Ok(ProjectEventAttribute {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    event: strings[2].clone(),
                    attribute: strings[3].clone(),
                    source_kind: strings[4].clone(),
                    source_name: strings[5].clone(),
                    source_field: strings[6].clone(),
                    provenance: strings[7].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    event_attributes.sort();
    event_attributes.dedup();
    Ok(event_attributes)
}

fn split_top_level_records(list: &str) -> Result<Vec<String>, FormalProjectFactError> {
    let trimmed = list.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .ok_or_else(|| {
            FormalProjectFactError::new("formal project list declaration is malformed")
        })?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in inner.char_indices() {
        if in_string {
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
            '"' => in_string = true,
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    FormalProjectFactError::new("formal project list declaration is malformed")
                })?;
            }
            ',' if depth == 0 => {
                records.push(inner[start..index].trim().to_owned());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    records.push(inner[start..].trim().to_owned());
    Ok(records)
}

fn split_top_level_fields(record: &str) -> Result<Vec<String>, FormalProjectFactError> {
    let trimmed = record.trim();
    let inner = trimmed
        .strip_prefix('(')
        .and_then(|without_open| without_open.strip_suffix(')'))
        .or_else(|| {
            trimmed
                .strip_prefix('{')
                .and_then(|without_open| without_open.strip_suffix('}'))
        })
        .ok_or_else(|| {
            FormalProjectFactError::new("formal project record declaration is malformed")
        })?;

    let mut fields = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in inner.char_indices() {
        if in_string {
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
            '"' => in_string = true,
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    FormalProjectFactError::new("formal project record declaration is malformed")
                })?;
            }
            ',' if depth == 0 => {
                fields.push(inner[start..index].trim().to_owned());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    fields.push(inner[start..].trim().to_owned());
    Ok(fields)
}

fn record_field_value(field: &str) -> Result<&str, FormalProjectFactError> {
    field
        .split_once(':')
        .map(|(_name, value)| value.trim())
        .ok_or_else(|| FormalProjectFactError::new("formal project record field is malformed"))
}

fn record_values(record: &str) -> Result<Vec<String>, FormalProjectFactError> {
    let uses_named_fields = record.trim_start().starts_with('{');
    let fields = split_top_level_fields(record)?;
    Ok(fields
        .iter()
        .map(|field| {
            if uses_named_fields {
                field
                    .split_once(':')
                    .map(|(_name, value)| value.trim())
                    .unwrap_or(field.trim())
            } else {
                field.trim()
            }
            .to_owned()
        })
        .collect())
}

fn single_quoted_string(value: &str) -> Result<String, FormalProjectFactError> {
    let strings = quoted_strings(value)?;
    match strings.as_slice() {
        [string] => Ok(string.clone()),
        _ => Err(FormalProjectFactError::new(
            "formal project string field is malformed",
        )),
    }
}

fn bool_value(value: &str) -> Result<bool, FormalProjectFactError> {
    match value.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(FormalProjectFactError::new(
            "formal project boolean field is malformed",
        )),
    }
}

fn string_list_value(value: &str) -> Result<Vec<String>, FormalProjectFactError> {
    let trimmed = value.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(FormalProjectFactError::new(
            "formal project string list field is malformed",
        ));
    }
    quoted_strings(trimmed)
}

fn string_lists(record: &str) -> Result<Vec<Vec<String>>, FormalProjectFactError> {
    let uses_named_fields = record.trim_start().starts_with('{');
    split_top_level_fields(record)?
        .into_iter()
        .filter_map(|field| {
            let value = if uses_named_fields {
                field
                    .split_once(':')
                    .map(|(_name, value)| value.trim())
                    .unwrap_or(field.trim())
            } else {
                field.trim()
            };
            if value.starts_with('[') {
                Some(string_list_value(value))
            } else {
                None
            }
        })
        .collect()
}

fn quoted_strings(record: &str) -> Result<Vec<String>, FormalProjectFactError> {
    let mut strings = Vec::new();
    let mut start = None;
    let mut escaped = false;
    for (index, character) in record.char_indices() {
        if let Some(open) = start {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                strings.push(
                    serde_json::from_str(&record[open..=index]).map_err(|error| {
                        FormalProjectFactError::new(format!(
                            "formal project string literal is malformed: {error}"
                        ))
                    })?,
                );
                start = None;
            }
        } else if character == '"' {
            start = Some(index);
        }
    }
    if start.is_some() {
        Err(FormalProjectFactError::new(
            "formal project string literal is unterminated",
        ))
    } else {
        Ok(strings)
    }
}

fn record_bool_tail(record: &str) -> Result<bool, FormalProjectFactError> {
    let normalized = record
        .trim()
        .trim_end_matches(')')
        .trim_end_matches('}')
        .trim();
    if normalized.ends_with("true") {
        Ok(true)
    } else if normalized.ends_with("false") {
        Ok(false)
    } else {
        Err(FormalProjectFactError::new(
            "formal project boolean field is malformed",
        ))
    }
}

struct ProjectDigestInventories<'a> {
    scenarios: &'a [ProjectScenario],
    scenario_definitions: &'a [ProjectScenarioDefinition],
    data_flows: &'a [ProjectDataFlow],
    outcomes: &'a [ProjectOutcome],
    command_errors: &'a [ProjectCommandError],
    commands: &'a [ProjectCommand],
    command_inputs: &'a [ProjectCommandInput],
    read_models: &'a [ProjectReadModel],
    read_model_definitions: &'a [ProjectReadModelDefinition],
    read_model_fields: &'a [ProjectReadModelField],
    views: &'a [ProjectView],
    view_definitions: &'a [ProjectViewDefinition],
    view_controls: &'a [ProjectViewControl],
    view_fields: &'a [ProjectViewField],
    automations: &'a [ProjectAutomation],
    automation_definitions: &'a [ProjectAutomationDefinition],
    translations: &'a [ProjectTranslation],
    translation_definitions: &'a [ProjectTranslationDefinition],
    external_payloads: &'a [ProjectExternalPayload],
    external_payload_fields: &'a [ProjectExternalPayloadField],
    streams: &'a [ProjectStream],
    events: &'a [ProjectEvent],
    event_attributes: &'a [ProjectEventAttribute],
}

fn update_lean_digest(
    contents: &str,
    inventories: ProjectDigestInventories<'_>,
) -> Result<String, FormalProjectFactError> {
    let digest = digest_with_project_inventories(
        declaration_json_string(contents, "def modelDigest := ")?,
        &inventories,
    );
    replace_declaration(
        contents,
        "def modelDigest :=",
        &format!("def modelDigest := {}", quoted(&digest)),
    )
    .and_then(|contents| {
        replace_declaration(
            &contents,
            "theorem modelDigestIsStable",
            &format!(
                "theorem modelDigestIsStable : modelDigest = {} := rfl",
                quoted(&digest)
            ),
        )
    })
}

fn update_quint_digest(
    contents: &str,
    inventories: ProjectDigestInventories<'_>,
) -> Result<String, FormalProjectFactError> {
    let digest = digest_with_project_inventories(
        declaration_json_string(contents, "val modelDigest = ")?,
        &inventories,
    );
    replace_declaration(
        contents,
        "val modelDigest =",
        &format!("val modelDigest = {}", quoted(&digest)),
    )
    .and_then(|contents| {
        replace_declaration(
            &contents,
            "val modelDigestStable =",
            &format!("val modelDigestStable = modelDigest == {}", quoted(&digest)),
        )
    })
}

fn digest_with_project_inventories(
    current_digest: String,
    inventories: &ProjectDigestInventories<'_>,
) -> String {
    let prefix = current_digest
        .split_once(";scenarios=")
        .or_else(|| current_digest.split_once(";scenario-definitions="))
        .or_else(|| current_digest.split_once(";data-flows="))
        .or_else(|| current_digest.split_once(";outcomes="))
        .or_else(|| current_digest.split_once(";command-errors="))
        .or_else(|| current_digest.split_once(";commands="))
        .or_else(|| current_digest.split_once(";command-inputs="))
        .or_else(|| current_digest.split_once(";read-models="))
        .or_else(|| current_digest.split_once(";read-model-definitions="))
        .or_else(|| current_digest.split_once(";read-model-fields="))
        .or_else(|| current_digest.split_once(";views="))
        .or_else(|| current_digest.split_once(";view-definitions="))
        .or_else(|| current_digest.split_once(";view-controls="))
        .or_else(|| current_digest.split_once(";view-fields="))
        .or_else(|| current_digest.split_once(";automations="))
        .or_else(|| current_digest.split_once(";automation-definitions="))
        .or_else(|| current_digest.split_once(";translations="))
        .or_else(|| current_digest.split_once(";translation-definitions="))
        .or_else(|| current_digest.split_once(";external-payloads="))
        .or_else(|| current_digest.split_once(";external-payload-fields="))
        .or_else(|| current_digest.split_once(";streams="))
        .or_else(|| current_digest.split_once(";events="))
        .map(|(prefix, _tail)| prefix.to_owned())
        .unwrap_or(current_digest);
    format!(
        "{prefix};scenarios={};scenario-definitions={};data-flows={};outcomes={};command-errors={};commands={};command-inputs={};read-models={};read-model-definitions={};read-model-fields={};views={};view-definitions={};view-controls={};view-fields={};automations={};automation-definitions={};translations={};translation-definitions={};external-payloads={};external-payload-fields={};streams={};events={};event-attributes={}",
        digest_scenarios(inventories.scenarios),
        digest_scenario_definitions(inventories.scenario_definitions),
        digest_data_flows(inventories.data_flows),
        digest_outcomes(inventories.outcomes),
        digest_command_errors(inventories.command_errors),
        digest_commands(inventories.commands),
        digest_command_inputs(inventories.command_inputs),
        digest_read_models(inventories.read_models),
        digest_read_model_definitions(inventories.read_model_definitions),
        digest_read_model_fields(inventories.read_model_fields),
        digest_views(inventories.views),
        digest_view_definitions(inventories.view_definitions),
        digest_view_controls(inventories.view_controls),
        digest_view_fields(inventories.view_fields),
        digest_automations(inventories.automations),
        digest_automation_definitions(inventories.automation_definitions),
        digest_translations(inventories.translations),
        digest_translation_definitions(inventories.translation_definitions),
        digest_external_payloads(inventories.external_payloads),
        digest_external_payload_fields(inventories.external_payload_fields),
        digest_streams(inventories.streams),
        digest_events(inventories.events),
        digest_event_attributes(inventories.event_attributes)
    )
}

fn digest_data_flows(data_flows: &[ProjectDataFlow]) -> String {
    data_flows
        .iter()
        .map(|data_flow| {
            format!(
                "{}/{}/{}@{}~{}~{}#{}",
                data_flow.workflow_slug,
                data_flow.slice_slug,
                data_flow.datum,
                data_flow.source,
                data_flow.transformation,
                data_flow.target,
                data_flow.bit_encoding
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_outcomes(outcomes: &[ProjectOutcome]) -> String {
    outcomes
        .iter()
        .map(|outcome| {
            format!(
                "{}/{}/{}@{}#{}",
                outcome.workflow_slug,
                outcome.slice_slug,
                outcome.outcome,
                outcome.events.join("+"),
                outcome.externally_relevant
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_scenarios(scenarios: &[ProjectScenario]) -> String {
    scenarios
        .iter()
        .map(|scenario| {
            format!(
                "{}/{}/{}/{}",
                scenario.workflow_slug,
                scenario.slice_slug,
                scenario.scenario_kind,
                scenario.scenario
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_scenario_definitions(scenario_definitions: &[ProjectScenarioDefinition]) -> String {
    scenario_definitions
        .iter()
        .map(|scenario| {
            format!(
                "{}/{}/{}/{}@{}~{}~{}#{}#{}#{}#{}#{}",
                scenario.workflow_slug,
                scenario.slice_slug,
                scenario.scenario_kind,
                scenario.scenario,
                scenario.given,
                scenario.when,
                scenario.then,
                scenario.read_streams.join("+"),
                scenario.written_streams.join("+"),
                scenario.contract_kind,
                scenario.covered_definition,
                scenario.error_references.join("+")
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_command_errors(command_errors: &[ProjectCommandError]) -> String {
    command_errors
        .iter()
        .map(|command_error| {
            format!(
                "{}/{}/{}/{}@{}#{}",
                command_error.workflow_slug,
                command_error.slice_slug,
                command_error.command,
                command_error.error,
                command_error.scenario,
                command_error.recovery
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_commands(commands: &[ProjectCommand]) -> String {
    commands
        .iter()
        .map(|command| {
            format!(
                "{}/{}/{}",
                command.workflow_slug, command.slice_slug, command.command
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_command_inputs(command_inputs: &[ProjectCommandInput]) -> String {
    command_inputs
        .iter()
        .map(|command_input| {
            format!(
                "{}/{}/{}/{}@{}#{}#{}",
                command_input.workflow_slug,
                command_input.slice_slug,
                command_input.command,
                command_input.input,
                command_input.source_kind,
                command_input.source_description,
                command_input.provenance_chain.join(" -> ")
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_read_models(read_models: &[ProjectReadModel]) -> String {
    read_models
        .iter()
        .map(|read_model| {
            format!(
                "{}/{}/{}",
                read_model.workflow_slug, read_model.slice_slug, read_model.read_model
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_read_model_definitions(read_model_definitions: &[ProjectReadModelDefinition]) -> String {
    read_model_definitions
        .iter()
        .map(|definition| {
            format!(
                "{}/{}/{}@{}#{}#{}#{}",
                definition.workflow_slug,
                definition.slice_slug,
                definition.read_model,
                definition.transitive,
                definition.relationship_fields.join("+"),
                definition.transitive_rule,
                definition.example_scenario_name
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_read_model_fields(read_model_fields: &[ProjectReadModelField]) -> String {
    read_model_fields
        .iter()
        .map(|field| {
            format!(
                "{}/{}/{}/{}@{}#{}.{}#{}#{}#{}#{}#{}",
                field.workflow_slug,
                field.slice_slug,
                field.read_model,
                field.field,
                field.source_kind,
                field.source_event,
                field.source_attribute,
                field.derivation_rule,
                field.absence_event,
                field.derivation_scenario_name,
                field.absence_scenario_name,
                field.provenance
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_views(views: &[ProjectView]) -> String {
    views
        .iter()
        .map(|view| format!("{}/{}/{}", view.workflow_slug, view.slice_slug, view.view))
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_view_definitions(definitions: &[ProjectViewDefinition]) -> String {
    definitions
        .iter()
        .map(|definition| {
            format!(
                "{}/{}/{}@{}#{}#{}#{}",
                definition.workflow_slug,
                definition.slice_slug,
                definition.view,
                definition.read_models.join("|"),
                definition.sketch_tokens.join("|"),
                definition.local_states.join("|"),
                definition.filters.join("|")
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_view_controls(controls: &[ProjectViewControl]) -> String {
    controls
        .iter()
        .map(|control| {
            format!(
                "{}/{}/{}/{}@{}#{}:{}:{}:{}:{}:{}#{}#{}#{}#{}:{}:{}:{}:{}",
                control.workflow_slug,
                control.slice_slug,
                control.view,
                control.control,
                control.command,
                control.input,
                control.input_source_kind,
                control.input_source_description,
                control.input_sketch_token,
                control.input_visible_to_actor,
                control.input_decision_field,
                control.handled_errors.join("|"),
                control.recovery_behavior,
                control.control_sketch_token,
                control.navigation_type,
                control.navigation_target,
                control.external_workflow,
                control.external_system,
                control.handoff_contract
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_view_fields(view_fields: &[ProjectViewField]) -> String {
    view_fields
        .iter()
        .map(|field| {
            format!(
                "{}/{}/{}/{}@{}#{}.{}#{}#{}",
                field.workflow_slug,
                field.slice_slug,
                field.view,
                field.field,
                field.source_kind,
                field.source_read_model,
                field.source_field,
                field.provenance,
                field.bit_encoding
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_automations(automations: &[ProjectAutomation]) -> String {
    automations
        .iter()
        .map(|automation| {
            format!(
                "{}/{}/{}",
                automation.workflow_slug, automation.slice_slug, automation.automation
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_automation_definitions(definitions: &[ProjectAutomationDefinition]) -> String {
    definitions
        .iter()
        .map(|definition| {
            format!(
                "{}/{}/{}@{}#{}#{}#{}",
                definition.workflow_slug,
                definition.slice_slug,
                definition.automation,
                definition.trigger,
                definition.command,
                definition.handled_errors.join("|"),
                definition.reaction
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_translations(translations: &[ProjectTranslation]) -> String {
    translations
        .iter()
        .map(|translation| {
            format!(
                "{}/{}/{}",
                translation.workflow_slug, translation.slice_slug, translation.translation
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_translation_definitions(definitions: &[ProjectTranslationDefinition]) -> String {
    definitions
        .iter()
        .map(|definition| {
            format!(
                "{}/{}/{}@{}#{}#{}",
                definition.workflow_slug,
                definition.slice_slug,
                definition.translation,
                definition.external_event,
                definition.payload_contract,
                definition.command
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_external_payloads(external_payloads: &[ProjectExternalPayload]) -> String {
    external_payloads
        .iter()
        .map(|external_payload| {
            format!(
                "{}/{}/{}",
                external_payload.workflow_slug,
                external_payload.slice_slug,
                external_payload.external_payload
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_external_payload_fields(
    external_payload_fields: &[ProjectExternalPayloadField],
) -> String {
    external_payload_fields
        .iter()
        .map(|field| {
            format!(
                "{}/{}/{}/{}@{}#{}",
                field.workflow_slug,
                field.slice_slug,
                field.external_payload,
                field.field,
                field.provenance,
                field.bit_encoding
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_streams(streams: &[ProjectStream]) -> String {
    streams
        .iter()
        .map(|stream| {
            format!(
                "{}/{}/{}",
                stream.workflow_slug, stream.slice_slug, stream.stream
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_events(events: &[ProjectEvent]) -> String {
    events
        .iter()
        .map(|event| {
            format!(
                "{}/{}/{}@{}",
                event.workflow_slug, event.slice_slug, event.event, event.stream
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_event_attributes(event_attributes: &[ProjectEventAttribute]) -> String {
    event_attributes
        .iter()
        .map(|attribute| {
            format!(
                "{}/{}/{}/{}@{}#{}.{}#{}",
                attribute.workflow_slug,
                attribute.slice_slug,
                attribute.event,
                attribute.attribute,
                attribute.source_kind,
                attribute.source_name,
                attribute.source_field,
                attribute.provenance
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn declaration_value<'a>(
    contents: &'a str,
    marker: &str,
) -> Result<&'a str, FormalProjectFactError> {
    contents
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(marker))
        .ok_or_else(|| {
            FormalProjectFactError::new(format!(
                "formal project artifact is missing declaration {marker}"
            ))
        })
}

fn declaration_json_string(contents: &str, marker: &str) -> Result<String, FormalProjectFactError> {
    serde_json::from_str(declaration_value(contents, marker)?.trim()).map_err(|error| {
        FormalProjectFactError::new(format!(
            "formal project model digest declaration is malformed: {error}"
        ))
    })
}

fn join_lines_preserving_trailing_newline(original: &str, lines: Vec<String>) -> String {
    let mut updated = lines.join("\n");
    if original.ends_with('\n') {
        updated.push('\n');
    }
    updated
}

fn lean_stream_record(stream: &NewProjectStream) -> String {
    format!(
        "({}, {}, {})",
        quoted(stream.workflow_slug.as_ref()),
        quoted(stream.slice_slug.as_ref()),
        quoted(stream.stream.as_ref())
    )
}

fn lean_command_record(command: &NewProjectCommand) -> String {
    format!(
        "({}, {}, {})",
        quoted(command.workflow_slug.as_ref()),
        quoted(command.slice_slug.as_ref()),
        quoted(command.command.as_ref())
    )
}

fn lean_command_input_record(command_input: &NewProjectCommandInput) -> String {
    format!(
        "({}, {}, {}, {}, {}, {}, [{}])",
        quoted(command_input.workflow_slug.as_ref()),
        quoted(command_input.slice_slug.as_ref()),
        quoted(command_input.command.as_ref()),
        quoted(command_input.input.as_ref()),
        quoted(&command_input.source_kind),
        quoted(&command_input.source_description),
        quoted_string_list(&command_input.provenance_chain)
    )
}

fn quint_command_input_record(command_input: &NewProjectCommandInput) -> String {
    format!(
        "{{ workflow: {}, slice: {}, command: {}, input: {}, sourceKind: {}, sourceDescription: {}, provenanceChain: [{}] }}",
        quoted(command_input.workflow_slug.as_ref()),
        quoted(command_input.slice_slug.as_ref()),
        quoted(command_input.command.as_ref()),
        quoted(command_input.input.as_ref()),
        quoted(&command_input.source_kind),
        quoted(&command_input.source_description),
        quoted_string_list(&command_input.provenance_chain)
    )
}

fn lean_command_error_record(command_error: &NewProjectCommandError) -> String {
    format!(
        "({}, {}, {}, {}, {}, {})",
        quoted(command_error.workflow_slug.as_ref()),
        quoted(command_error.slice_slug.as_ref()),
        quoted(command_error.command.as_ref()),
        quoted(command_error.error.as_ref()),
        quoted(command_error.scenario.as_ref()),
        quoted(command_error.recovery.as_ref())
    )
}

fn lean_scenario_record(scenario: &NewProjectScenario) -> String {
    format!(
        "({}, {}, {}, {})",
        quoted(scenario.workflow_slug.as_ref()),
        quoted(scenario.slice_slug.as_ref()),
        quoted(scenario.scenario_kind.as_str()),
        quoted(scenario.scenario.as_ref())
    )
}

fn lean_scenario_definition_record(scenario: &NewProjectScenarioDefinition) -> String {
    format!(
        "({}, {}, {}, {}, {}, {}, {}, [{}], [{}], {}, {}, [{}])",
        quoted(scenario.workflow_slug.as_ref()),
        quoted(scenario.slice_slug.as_ref()),
        quoted(scenario.scenario_kind.as_str()),
        quoted(scenario.scenario.as_ref()),
        quoted(scenario.given.as_ref()),
        quoted(scenario.when.as_ref()),
        quoted(scenario.then.as_ref()),
        scenario
            .read_streams
            .iter()
            .map(|stream| quoted(stream.as_ref()))
            .collect::<Vec<_>>()
            .join(","),
        scenario
            .written_streams
            .iter()
            .map(|stream| quoted(stream.as_ref()))
            .collect::<Vec<_>>()
            .join(","),
        quoted(
            scenario
                .contract_kind
                .as_ref()
                .map(|contract_kind| contract_kind.as_ref())
                .unwrap_or("")
        ),
        quoted(
            scenario
                .covered_definition
                .as_ref()
                .map(|covered_definition| covered_definition.as_ref())
                .unwrap_or("")
        ),
        scenario
            .error_references
            .iter()
            .map(|error| quoted(error.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_data_flow_record(data_flow: &NewProjectDataFlow) -> String {
    format!(
        "({}, {}, {}, [{}], [{}], [{}], [{}])",
        quoted(data_flow.workflow_slug.as_ref()),
        quoted(data_flow.slice_slug.as_ref()),
        quoted(data_flow.datum.as_ref()),
        quoted(data_flow.source.as_ref()),
        quoted(data_flow.transformation.as_ref()),
        quoted(data_flow.target.as_ref()),
        quoted(data_flow.bit_encoding.as_ref())
    )
}

fn quint_data_flow_record(data_flow: &NewProjectDataFlow) -> String {
    format!(
        "{{ workflow: {}, slice: {}, datum: {}, source: {}, transformation: {}, target: {}, bitEncoding: {} }}",
        quoted(data_flow.workflow_slug.as_ref()),
        quoted(data_flow.slice_slug.as_ref()),
        quoted(data_flow.datum.as_ref()),
        quoted(data_flow.source.as_ref()),
        quoted(data_flow.transformation.as_ref()),
        quoted(data_flow.target.as_ref()),
        quoted(data_flow.bit_encoding.as_ref())
    )
}

fn quint_scenario_record(scenario: &NewProjectScenario) -> String {
    format!(
        "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {} }}",
        quoted(scenario.workflow_slug.as_ref()),
        quoted(scenario.slice_slug.as_ref()),
        quoted(scenario.scenario_kind.as_str()),
        quoted(scenario.scenario.as_ref())
    )
}

fn quint_scenario_definition_record(scenario: &NewProjectScenarioDefinition) -> String {
    format!(
        "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {}, given: {}, when: {}, then: {}, readStreams: [{}], writtenStreams: [{}], contractKind: {}, coveredDefinition: {}, errorReferences: [{}] }}",
        quoted(scenario.workflow_slug.as_ref()),
        quoted(scenario.slice_slug.as_ref()),
        quoted(scenario.scenario_kind.as_str()),
        quoted(scenario.scenario.as_ref()),
        quoted(scenario.given.as_ref()),
        quoted(scenario.when.as_ref()),
        quoted(scenario.then.as_ref()),
        scenario
            .read_streams
            .iter()
            .map(|stream| quoted(stream.as_ref()))
            .collect::<Vec<_>>()
            .join(","),
        scenario
            .written_streams
            .iter()
            .map(|stream| quoted(stream.as_ref()))
            .collect::<Vec<_>>()
            .join(","),
        quoted(
            scenario
                .contract_kind
                .as_ref()
                .map(|contract_kind| contract_kind.as_ref())
                .unwrap_or("")
        ),
        quoted(
            scenario
                .covered_definition
                .as_ref()
                .map(|covered_definition| covered_definition.as_ref())
                .unwrap_or("")
        ),
        scenario
            .error_references
            .iter()
            .map(|error| quoted(error.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_outcome_record(outcome: &NewProjectOutcome) -> String {
    format!(
        "({}, {}, {}, [{}], {})",
        quoted(outcome.workflow_slug.as_ref()),
        quoted(outcome.slice_slug.as_ref()),
        quoted(outcome.outcome.as_ref()),
        lean_event_list(outcome.events.as_slice()),
        outcome.externally_relevant
    )
}

fn quint_outcome_record(outcome: &NewProjectOutcome) -> String {
    format!(
        "{{ workflow: {}, slice: {}, outcome: {}, events: [{}], externallyRelevant: {} }}",
        quoted(outcome.workflow_slug.as_ref()),
        quoted(outcome.slice_slug.as_ref()),
        quoted(outcome.outcome.as_ref()),
        quint_event_list(outcome.events.as_slice()),
        outcome.externally_relevant
    )
}

fn lean_event_list(events: &[EventName]) -> String {
    events
        .iter()
        .map(|event| quoted(event.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn quint_event_list(events: &[EventName]) -> String {
    events
        .iter()
        .map(|event| quoted(event.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn lean_string_list(events: &[String]) -> String {
    events
        .iter()
        .map(|event| quoted(event))
        .collect::<Vec<_>>()
        .join(",")
}

fn quint_string_list(events: &[String]) -> String {
    events
        .iter()
        .map(|event| quoted(event))
        .collect::<Vec<_>>()
        .join(",")
}

fn lean_project_scenario_list(project_scenarios: &[ProjectScenario]) -> String {
    let mut project_scenarios = project_scenarios
        .iter()
        .map(|scenario| {
            (
                scenario.workflow_slug(),
                scenario.slice_slug(),
                scenario.scenario_kind(),
                scenario.scenario(),
            )
        })
        .collect::<Vec<_>>();
    project_scenarios.sort_unstable();
    format!(
        "[{}]",
        project_scenarios
            .into_iter()
            .map(|(workflow_slug, slice_slug, scenario_kind, scenario)| {
                format!(
                    "({}, {}, {}, {})",
                    quoted(workflow_slug),
                    quoted(slice_slug),
                    quoted(scenario_kind),
                    quoted(scenario)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_project_scenario_list(project_scenarios: &[ProjectScenario]) -> String {
    let mut project_scenarios = project_scenarios
        .iter()
        .map(|scenario| {
            (
                scenario.workflow_slug(),
                scenario.slice_slug(),
                scenario.scenario_kind(),
                scenario.scenario(),
            )
        })
        .collect::<Vec<_>>();
    project_scenarios.sort_unstable();
    format!(
        "[{}]",
        project_scenarios
            .into_iter()
            .map(|(workflow_slug, slice_slug, scenario_kind, scenario)| {
                format!(
                    "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {} }}",
                    quoted(workflow_slug),
                    quoted(slice_slug),
                    quoted(scenario_kind),
                    quoted(scenario)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_project_scenario_definition_list(
    scenario_definitions: &[ProjectScenarioDefinition],
) -> String {
    let mut scenario_definitions = scenario_definitions.to_vec();
    scenario_definitions.sort();
    format!(
        "[{}]",
        scenario_definitions
            .into_iter()
            .map(|scenario| {
                format!(
                    "({}, {}, {}, {}, {}, {}, {}, [{}], [{}], {}, {}, [{}])",
                    quoted(scenario.workflow_slug()),
                    quoted(scenario.slice_slug()),
                    quoted(scenario.scenario_kind()),
                    quoted(scenario.scenario()),
                    quoted(scenario.given()),
                    quoted(scenario.when()),
                    quoted(scenario.then()),
                    lean_string_list(scenario.read_streams()),
                    lean_string_list(scenario.written_streams()),
                    quoted(scenario.contract_kind()),
                    quoted(scenario.covered_definition()),
                    lean_string_list(scenario.error_references())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_project_scenario_definition_list(
    scenario_definitions: &[ProjectScenarioDefinition],
) -> String {
    let mut scenario_definitions = scenario_definitions.to_vec();
    scenario_definitions.sort();
    format!(
        "[{}]",
        scenario_definitions
            .into_iter()
            .map(|scenario| {
                format!(
                    "{{ workflow: {}, slice: {}, scenarioKind: {}, scenario: {}, given: {}, when: {}, then: {}, readStreams: [{}], writtenStreams: [{}], contractKind: {}, coveredDefinition: {}, errorReferences: [{}] }}",
                    quoted(scenario.workflow_slug()),
                    quoted(scenario.slice_slug()),
                    quoted(scenario.scenario_kind()),
                    quoted(scenario.scenario()),
                    quoted(scenario.given()),
                    quoted(scenario.when()),
                    quoted(scenario.then()),
                    quint_string_list(scenario.read_streams()),
                    quint_string_list(scenario.written_streams()),
                    quoted(scenario.contract_kind()),
                    quoted(scenario.covered_definition()),
                    quint_string_list(scenario.error_references())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_project_data_flow_list(project_data_flows: &[ProjectDataFlow]) -> String {
    let mut project_data_flows = project_data_flows.iter().collect::<Vec<_>>();
    project_data_flows.sort_unstable();
    format!(
        "[{}]",
        project_data_flows
            .into_iter()
            .map(|data_flow| {
                format!(
                    "({}, {}, {}, {}, {}, {}, {})",
                    quoted(data_flow.workflow_slug()),
                    quoted(data_flow.slice_slug()),
                    quoted(data_flow.datum()),
                    quoted(data_flow.source()),
                    quoted(data_flow.transformation()),
                    quoted(data_flow.target()),
                    quoted(data_flow.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_project_data_flow_list(project_data_flows: &[ProjectDataFlow]) -> String {
    let mut project_data_flows = project_data_flows.iter().collect::<Vec<_>>();
    project_data_flows.sort_unstable();
    format!(
        "[{}]",
        project_data_flows
            .into_iter()
            .map(|data_flow| {
                format!(
                    "{{ workflow: {}, slice: {}, datum: {}, source: {}, transformation: {}, target: {}, bitEncoding: {} }}",
                    quoted(data_flow.workflow_slug()),
                    quoted(data_flow.slice_slug()),
                    quoted(data_flow.datum()),
                    quoted(data_flow.source()),
                    quoted(data_flow.transformation()),
                    quoted(data_flow.target()),
                    quoted(data_flow.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_project_outcome_list(project_outcomes: &[ProjectOutcome]) -> String {
    let mut project_outcomes = project_outcomes.iter().collect::<Vec<_>>();
    project_outcomes.sort_unstable();
    format!(
        "[{}]",
        project_outcomes
            .into_iter()
            .map(|outcome| {
                format!(
                    "({}, {}, {}, [{}], {})",
                    quoted(outcome.workflow_slug()),
                    quoted(outcome.slice_slug()),
                    quoted(outcome.outcome()),
                    lean_string_list(outcome.events()),
                    outcome.externally_relevant()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_project_outcome_list(project_outcomes: &[ProjectOutcome]) -> String {
    let mut project_outcomes = project_outcomes.iter().collect::<Vec<_>>();
    project_outcomes.sort_unstable();
    format!(
        "[{}]",
        project_outcomes
            .into_iter()
            .map(|outcome| {
                format!(
                    "{{ workflow: {}, slice: {}, outcome: {}, events: [{}], externallyRelevant: {} }}",
                    quoted(outcome.workflow_slug()),
                    quoted(outcome.slice_slug()),
                    quoted(outcome.outcome()),
                    quint_string_list(outcome.events()),
                    outcome.externally_relevant()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_read_model_record(read_model: &NewProjectReadModel) -> String {
    format!(
        "({}, {}, {})",
        quoted(read_model.workflow_slug.as_ref()),
        quoted(read_model.slice_slug.as_ref()),
        quoted(read_model.read_model.as_ref())
    )
}

fn quint_read_model_record(read_model: &NewProjectReadModel) -> String {
    format!(
        "{{ workflow: {}, slice: {}, readModel: {} }}",
        quoted(read_model.workflow_slug.as_ref()),
        quoted(read_model.slice_slug.as_ref()),
        quoted(read_model.read_model.as_ref())
    )
}

fn lean_read_model_definition_record(definition: &NewProjectReadModelDefinition) -> String {
    format!(
        "({}, {}, {}, {}, [{}], {}, {})",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.read_model.as_ref()),
        definition.transitive,
        quoted_string_list(&definition.relationship_fields),
        quoted(&definition.transitive_rule),
        quoted(&definition.example_scenario_name)
    )
}

fn quint_read_model_definition_record(definition: &NewProjectReadModelDefinition) -> String {
    format!(
        "{{ workflow: {}, slice: {}, readModel: {}, transitive: {}, relationshipFields: [{}], transitiveRule: {}, exampleScenarioName: {} }}",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.read_model.as_ref()),
        definition.transitive,
        quoted_string_list(&definition.relationship_fields),
        quoted(&definition.transitive_rule),
        quoted(&definition.example_scenario_name)
    )
}

fn lean_read_model_field_record(field: &NewProjectReadModelField) -> String {
    format!(
        "({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
        quoted(field.workflow_slug.as_ref()),
        quoted(field.slice_slug.as_ref()),
        quoted(field.read_model.as_ref()),
        quoted(field.field.as_ref()),
        quoted(field.source_kind.as_ref()),
        quoted(&field.source_event),
        quoted(&field.source_attribute),
        quoted(&field.derivation_rule),
        quoted(&field.absence_event),
        quoted(&field.derivation_scenario_name),
        quoted(&field.absence_scenario_name),
        quoted(field.provenance.as_ref())
    )
}

fn quint_read_model_field_record(field: &NewProjectReadModelField) -> String {
    format!(
        "{{ workflow: {}, slice: {}, readModel: {}, field: {}, sourceKind: {}, sourceEvent: {}, sourceAttribute: {}, derivationRule: {}, absenceEvent: {}, derivationScenarioName: {}, absenceScenarioName: {}, provenance: {} }}",
        quoted(field.workflow_slug.as_ref()),
        quoted(field.slice_slug.as_ref()),
        quoted(field.read_model.as_ref()),
        quoted(field.field.as_ref()),
        quoted(field.source_kind.as_ref()),
        quoted(&field.source_event),
        quoted(&field.source_attribute),
        quoted(&field.derivation_rule),
        quoted(&field.absence_event),
        quoted(&field.derivation_scenario_name),
        quoted(&field.absence_scenario_name),
        quoted(field.provenance.as_ref())
    )
}

fn lean_view_record(view: &NewProjectView) -> String {
    format!(
        "({}, {}, {})",
        quoted(view.workflow_slug.as_ref()),
        quoted(view.slice_slug.as_ref()),
        quoted(view.view.as_ref())
    )
}

fn lean_view_definition_record(definition: &NewProjectViewDefinition) -> String {
    let read_models = definition
        .read_models
        .iter()
        .map(|read_model| read_model.as_ref().to_owned())
        .collect::<Vec<_>>();
    let sketch_tokens = definition
        .sketch_tokens
        .iter()
        .map(|token| token.as_ref().to_owned())
        .collect::<Vec<_>>();
    format!(
        "({}, {}, {}, [{}], [{}], [{}], [{}])",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.view.as_ref()),
        quoted_string_list(&read_models),
        quoted_string_list(&sketch_tokens),
        quoted_string_list(&definition.local_states),
        quoted_string_list(&definition.filters)
    )
}

fn quint_view_record(view: &NewProjectView) -> String {
    format!(
        "{{ workflow: {}, slice: {}, view: {} }}",
        quoted(view.workflow_slug.as_ref()),
        quoted(view.slice_slug.as_ref()),
        quoted(view.view.as_ref())
    )
}

fn quint_view_definition_record(definition: &NewProjectViewDefinition) -> String {
    let read_models = definition
        .read_models
        .iter()
        .map(|read_model| read_model.as_ref().to_owned())
        .collect::<Vec<_>>();
    let sketch_tokens = definition
        .sketch_tokens
        .iter()
        .map(|token| token.as_ref().to_owned())
        .collect::<Vec<_>>();
    format!(
        "{{ workflow: {}, slice: {}, view: {}, readModels: [{}], sketchTokens: [{}], localStates: [{}], filters: [{}] }}",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.view.as_ref()),
        quoted_string_list(&read_models),
        quoted_string_list(&sketch_tokens),
        quoted_string_list(&definition.local_states),
        quoted_string_list(&definition.filters)
    )
}

fn lean_view_control_record(control: &NewProjectViewControl) -> String {
    let handled_errors = control
        .handled_errors
        .iter()
        .map(|error| error.as_ref().to_owned())
        .collect::<Vec<_>>();
    format!(
        "({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, [{}], {}, {}, {}, {}, {}, {}, {})",
        quoted(control.workflow_slug.as_ref()),
        quoted(control.slice_slug.as_ref()),
        quoted(control.view.as_ref()),
        quoted(control.control.as_ref()),
        quoted(control.command.as_ref()),
        quoted(control.input.as_ref()),
        quoted(control.input_source_kind.as_ref()),
        quoted(control.input_source_description.as_ref()),
        quoted(control.input_sketch_token.as_ref()),
        control.input_visible_to_actor,
        control.input_decision_field,
        quoted_string_list(&handled_errors),
        quoted(control.recovery_behavior.as_ref()),
        quoted(control.control_sketch_token.as_ref()),
        quoted(control.navigation_type.as_ref()),
        quoted(control.navigation_target.as_ref()),
        quoted(&control.external_workflow),
        quoted(&control.external_system),
        quoted(&control.handoff_contract)
    )
}

fn quint_view_control_record(control: &NewProjectViewControl) -> String {
    let handled_errors = control
        .handled_errors
        .iter()
        .map(|error| error.as_ref().to_owned())
        .collect::<Vec<_>>();
    format!(
        "{{ workflow: {}, slice: {}, view: {}, control: {}, command: {}, input: {}, inputSourceKind: {}, inputSourceDescription: {}, inputSketchToken: {}, inputVisibleToActor: {}, inputDecisionField: {}, handledErrors: [{}], recoveryBehavior: {}, controlSketchToken: {}, navigationType: {}, navigationTarget: {}, externalWorkflow: {}, externalSystem: {}, handoffContract: {} }}",
        quoted(control.workflow_slug.as_ref()),
        quoted(control.slice_slug.as_ref()),
        quoted(control.view.as_ref()),
        quoted(control.control.as_ref()),
        quoted(control.command.as_ref()),
        quoted(control.input.as_ref()),
        quoted(control.input_source_kind.as_ref()),
        quoted(control.input_source_description.as_ref()),
        quoted(control.input_sketch_token.as_ref()),
        control.input_visible_to_actor,
        control.input_decision_field,
        quoted_string_list(&handled_errors),
        quoted(control.recovery_behavior.as_ref()),
        quoted(control.control_sketch_token.as_ref()),
        quoted(control.navigation_type.as_ref()),
        quoted(control.navigation_target.as_ref()),
        quoted(&control.external_workflow),
        quoted(&control.external_system),
        quoted(&control.handoff_contract)
    )
}

fn lean_view_field_record(field: &NewProjectViewField) -> String {
    format!(
        "({}, {}, {}, {}, {}, {}, {}, {}, {})",
        quoted(field.workflow_slug.as_ref()),
        quoted(field.slice_slug.as_ref()),
        quoted(field.view.as_ref()),
        quoted(field.field.as_ref()),
        quoted(field.source_kind.as_ref()),
        quoted(field.source_read_model.as_ref()),
        quoted(field.source_field.as_ref()),
        quoted(field.provenance.as_ref()),
        quoted(field.bit_encoding.as_ref())
    )
}

fn quint_view_field_record(field: &NewProjectViewField) -> String {
    format!(
        "{{ workflow: {}, slice: {}, view: {}, field: {}, sourceKind: {}, sourceReadModel: {}, sourceField: {}, provenance: {}, bitEncoding: {} }}",
        quoted(field.workflow_slug.as_ref()),
        quoted(field.slice_slug.as_ref()),
        quoted(field.view.as_ref()),
        quoted(field.field.as_ref()),
        quoted(field.source_kind.as_ref()),
        quoted(field.source_read_model.as_ref()),
        quoted(field.source_field.as_ref()),
        quoted(field.provenance.as_ref()),
        quoted(field.bit_encoding.as_ref())
    )
}

fn lean_automation_record(automation: &NewProjectAutomation) -> String {
    format!(
        "({}, {}, {})",
        quoted(automation.workflow_slug.as_ref()),
        quoted(automation.slice_slug.as_ref()),
        quoted(automation.automation.as_ref())
    )
}

fn quint_automation_record(automation: &NewProjectAutomation) -> String {
    format!(
        "{{ workflow: {}, slice: {}, automation: {} }}",
        quoted(automation.workflow_slug.as_ref()),
        quoted(automation.slice_slug.as_ref()),
        quoted(automation.automation.as_ref())
    )
}

fn lean_automation_definition_record(definition: &NewProjectAutomationDefinition) -> String {
    let handled_errors = command_error_name_strings(definition.handled_errors.as_slice());
    format!(
        "({}, {}, {}, {}, {}, [{}], {})",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.automation.as_ref()),
        quoted(definition.trigger.as_ref()),
        quoted(definition.command.as_ref()),
        lean_string_list(&handled_errors),
        quoted(definition.reaction.as_ref())
    )
}

fn quint_automation_definition_record(definition: &NewProjectAutomationDefinition) -> String {
    let handled_errors = command_error_name_strings(definition.handled_errors.as_slice());
    format!(
        "{{ workflow: {}, slice: {}, automation: {}, trigger: {}, command: {}, handledErrors: [{}], reaction: {} }}",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.automation.as_ref()),
        quoted(definition.trigger.as_ref()),
        quoted(definition.command.as_ref()),
        quint_string_list(&handled_errors),
        quoted(definition.reaction.as_ref())
    )
}

fn command_error_name_strings(errors: &[CommandErrorName]) -> Vec<String> {
    errors
        .iter()
        .map(|error| error.as_ref().to_owned())
        .collect()
}

fn lean_automation_definition_list(definitions: &[ProjectAutomationDefinition]) -> String {
    let mut definitions = definitions.to_vec();
    definitions.sort();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "({}, {}, {}, {}, {}, [{}], {})",
                    quoted(&definition.workflow_slug),
                    quoted(&definition.slice_slug),
                    quoted(&definition.automation),
                    quoted(&definition.trigger),
                    quoted(&definition.command),
                    quoted_string_list(definition.handled_errors.as_slice()),
                    quoted(&definition.reaction)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_automation_definition_list(definitions: &[ProjectAutomationDefinition]) -> String {
    let mut definitions = definitions.to_vec();
    definitions.sort();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "{{ workflow: {}, slice: {}, automation: {}, trigger: {}, command: {}, handledErrors: [{}], reaction: {} }}",
                    quoted(&definition.workflow_slug),
                    quoted(&definition.slice_slug),
                    quoted(&definition.automation),
                    quoted(&definition.trigger),
                    quoted(&definition.command),
                    quoted_string_list(definition.handled_errors.as_slice()),
                    quoted(&definition.reaction)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_translation_record(translation: &NewProjectTranslation) -> String {
    format!(
        "({}, {}, {})",
        quoted(translation.workflow_slug.as_ref()),
        quoted(translation.slice_slug.as_ref()),
        quoted(translation.translation.as_ref())
    )
}

fn quint_translation_record(translation: &NewProjectTranslation) -> String {
    format!(
        "{{ workflow: {}, slice: {}, translation: {} }}",
        quoted(translation.workflow_slug.as_ref()),
        quoted(translation.slice_slug.as_ref()),
        quoted(translation.translation.as_ref())
    )
}

fn lean_translation_definition_record(definition: &NewProjectTranslationDefinition) -> String {
    format!(
        "({}, {}, {}, {}, {}, {})",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.translation.as_ref()),
        quoted(definition.external_event.as_ref()),
        quoted(definition.payload_contract.as_ref()),
        quoted(definition.command.as_ref())
    )
}

fn quint_translation_definition_record(definition: &NewProjectTranslationDefinition) -> String {
    format!(
        "{{ workflow: {}, slice: {}, translation: {}, externalEvent: {}, payloadContract: {}, command: {} }}",
        quoted(definition.workflow_slug.as_ref()),
        quoted(definition.slice_slug.as_ref()),
        quoted(definition.translation.as_ref()),
        quoted(definition.external_event.as_ref()),
        quoted(definition.payload_contract.as_ref()),
        quoted(definition.command.as_ref())
    )
}

fn lean_translation_definition_list(definitions: &[ProjectTranslationDefinition]) -> String {
    let mut definitions = definitions.to_vec();
    definitions.sort();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "({}, {}, {}, {}, {}, {})",
                    quoted(&definition.workflow_slug),
                    quoted(&definition.slice_slug),
                    quoted(&definition.translation),
                    quoted(&definition.external_event),
                    quoted(&definition.payload_contract),
                    quoted(&definition.command)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_translation_definition_list(definitions: &[ProjectTranslationDefinition]) -> String {
    let mut definitions = definitions.to_vec();
    definitions.sort();
    format!(
        "[{}]",
        definitions
            .into_iter()
            .map(|definition| {
                format!(
                    "{{ workflow: {}, slice: {}, translation: {}, externalEvent: {}, payloadContract: {}, command: {} }}",
                    quoted(&definition.workflow_slug),
                    quoted(&definition.slice_slug),
                    quoted(&definition.translation),
                    quoted(&definition.external_event),
                    quoted(&definition.payload_contract),
                    quoted(&definition.command)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn lean_external_payload_record(external_payload: &NewProjectExternalPayload) -> String {
    format!(
        "({}, {}, {})",
        quoted(external_payload.workflow_slug.as_ref()),
        quoted(external_payload.slice_slug.as_ref()),
        quoted(external_payload.external_payload.as_ref())
    )
}

fn quint_external_payload_record(external_payload: &NewProjectExternalPayload) -> String {
    format!(
        "{{ workflow: {}, slice: {}, externalPayload: {} }}",
        quoted(external_payload.workflow_slug.as_ref()),
        quoted(external_payload.slice_slug.as_ref()),
        quoted(external_payload.external_payload.as_ref())
    )
}

fn lean_external_payload_field_record(field: &NewProjectExternalPayloadField) -> String {
    format!(
        "({}, {}, {}, {}, {}, {})",
        quoted(field.workflow_slug.as_ref()),
        quoted(field.slice_slug.as_ref()),
        quoted(field.external_payload.as_ref()),
        quoted(field.field.as_ref()),
        quoted(field.provenance.as_ref()),
        quoted(field.bit_encoding.as_ref())
    )
}

fn quint_external_payload_field_record(field: &NewProjectExternalPayloadField) -> String {
    format!(
        "{{ workflow: {}, slice: {}, externalPayload: {}, field: {}, provenance: {}, bitEncoding: {} }}",
        quoted(field.workflow_slug.as_ref()),
        quoted(field.slice_slug.as_ref()),
        quoted(field.external_payload.as_ref()),
        quoted(field.field.as_ref()),
        quoted(field.provenance.as_ref()),
        quoted(field.bit_encoding.as_ref())
    )
}

fn lean_external_payload_field_list(fields: &[ProjectExternalPayloadField]) -> String {
    let mut fields = fields.iter().collect::<Vec<_>>();
    fields.sort_unstable();
    format!(
        "[{}]",
        fields
            .into_iter()
            .map(|field| {
                format!(
                    "({}, {}, {}, {}, {}, {})",
                    quoted(field.workflow_slug()),
                    quoted(field.slice_slug()),
                    quoted(field.external_payload()),
                    quoted(field.field()),
                    quoted(field.provenance()),
                    quoted(field.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_external_payload_field_list(fields: &[ProjectExternalPayloadField]) -> String {
    let mut fields = fields.iter().collect::<Vec<_>>();
    fields.sort_unstable();
    format!(
        "[{}]",
        fields
            .into_iter()
            .map(|field| {
                format!(
                    "{{ workflow: {}, slice: {}, externalPayload: {}, field: {}, provenance: {}, bitEncoding: {} }}",
                    quoted(field.workflow_slug()),
                    quoted(field.slice_slug()),
                    quoted(field.external_payload()),
                    quoted(field.field()),
                    quoted(field.provenance()),
                    quoted(field.bit_encoding())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_command_record(command: &NewProjectCommand) -> String {
    format!(
        "{{ workflow: {}, slice: {}, command: {} }}",
        quoted(command.workflow_slug.as_ref()),
        quoted(command.slice_slug.as_ref()),
        quoted(command.command.as_ref())
    )
}

fn quint_command_error_record(command_error: &NewProjectCommandError) -> String {
    format!(
        "{{ workflow: {}, slice: {}, command: {}, error: {}, scenario: {}, recovery: {} }}",
        quoted(command_error.workflow_slug.as_ref()),
        quoted(command_error.slice_slug.as_ref()),
        quoted(command_error.command.as_ref()),
        quoted(command_error.error.as_ref()),
        quoted(command_error.scenario.as_ref()),
        quoted(command_error.recovery.as_ref())
    )
}

fn quint_stream_record(stream: &NewProjectStream) -> String {
    format!(
        "{{ workflow: {}, slice: {}, stream: {} }}",
        quoted(stream.workflow_slug.as_ref()),
        quoted(stream.slice_slug.as_ref()),
        quoted(stream.stream.as_ref())
    )
}

fn lean_event_record(event: &NewProjectEvent) -> String {
    format!(
        "({}, {}, {}, {})",
        quoted(event.workflow_slug.as_ref()),
        quoted(event.slice_slug.as_ref()),
        quoted(event.event.as_ref()),
        quoted(event.stream.as_ref())
    )
}

fn lean_event_attribute_record(attribute: &NewProjectEventAttribute) -> String {
    format!(
        "({}, {}, {}, {}, {}, {}, {}, {})",
        quoted(attribute.workflow_slug.as_ref()),
        quoted(attribute.slice_slug.as_ref()),
        quoted(attribute.event.as_ref()),
        quoted(attribute.attribute.as_ref()),
        quoted(attribute.source_kind.as_ref()),
        quoted(attribute.source_name.as_ref()),
        quoted(attribute.source_field.as_ref()),
        quoted(attribute.provenance.as_ref())
    )
}

fn quint_event_record(event: &NewProjectEvent) -> String {
    format!(
        "{{ workflow: {}, slice: {}, event: {}, stream: {} }}",
        quoted(event.workflow_slug.as_ref()),
        quoted(event.slice_slug.as_ref()),
        quoted(event.event.as_ref()),
        quoted(event.stream.as_ref())
    )
}

fn quint_event_attribute_record(attribute: &NewProjectEventAttribute) -> String {
    format!(
        "{{ workflow: {}, slice: {}, event: {}, attribute: {}, sourceKind: {}, sourceName: {}, sourceField: {}, provenance: {} }}",
        quoted(attribute.workflow_slug.as_ref()),
        quoted(attribute.slice_slug.as_ref()),
        quoted(attribute.event.as_ref()),
        quoted(attribute.attribute.as_ref()),
        quoted(attribute.source_kind.as_ref()),
        quoted(attribute.source_name.as_ref()),
        quoted(attribute.source_field.as_ref()),
        quoted(attribute.provenance.as_ref())
    )
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated formal project string literal must be valid: {error}");
    })
}

fn quoted_string_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| quoted(value))
        .collect::<Vec<_>>()
        .join(",")
}

fn file_contents(value: String) -> Result<FileContents, FormalProjectFactError> {
    FileContents::try_new(value).map_err(|error| FormalProjectFactError::new(error.to_string()))
}

fn report_line(value: String) -> Result<ReportLine, FormalProjectFactError> {
    ReportLine::try_new(value).map_err(|error| FormalProjectFactError::new(error.to_string()))
}
