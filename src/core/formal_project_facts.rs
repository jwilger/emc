// Copyright 2026 John Wilger

use crate::core::formal_slice_facts::{
    CommandErrorDefinitions, NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection,
    NewBoardElement, NewCommandDefinition, NewCommandErrorDefinition, NewCommandInput,
    NewControlDefinition, NewEventAttribute, NewEventDefinition, NewExternalPayloadDefinition,
    NewOutcomeDefinition, NewReadModelDefinition, NewReadModelField, NewSliceScenario,
    NewTranslationDefinition, NewViewDefinition, NewViewField, OutcomeEventNames, ScenarioKind,
};
use crate::core::layout::{ModeledProjectRootInventories, ModeledProjectRootInventoryParts};
use crate::core::types::{
    AutomationName, AutomationReactionDescription, AutomationTriggerName, BitEncodingSemantics,
    BoardConnectionEndpoint, BoardConnectionEndpointKind, BoardElementDeclaredName,
    BoardElementKind, BoardElementName, BoardLaneId, CommandErrorName, CommandErrorRecoveryKind,
    CommandInputSourceDescription, CommandInputSourceKind, CommandName, ContractKindName,
    ControlName, ControlRecoveryBehavior, CoveredDefinitionName, DataFlowSource,
    DataFlowSourceKind, DataFlowTarget, DatumName, EventAttributeName, EventAttributeSourceField,
    EventAttributeSourceKind, EventAttributeSourceName, EventName,
    GeneratedEventAttributeSourceKind, NavigationTargetName, NavigationTargetType,
    OutcomeLabelName, PayloadContractName, ProvenanceDescription, ReadModelFieldSourceKind,
    ReadModelName, ScenarioName, ScenarioStepText, SketchToken, SliceSlug, StreamName,
    TransformationSemantics, TranslationExternalEventName, TranslationName, ViewFieldName,
    ViewFieldSourceKind, ViewName, WorkflowSlug,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectStream {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    stream: StreamName,
}

impl NewProjectStream {
    pub(crate) fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        stream: StreamName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            stream,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectCommand {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    command: CommandName,
    command_inputs: Vec<NewProjectCommandInput>,
    command_errors: Vec<NewProjectCommandError>,
}

impl NewProjectCommand {
    pub(crate) fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        command: CommandName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            command,
            command_inputs: Vec::new(),
            command_errors: Vec::new(),
        }
    }

    pub(crate) fn with_input(mut self, input: &NewCommandInput) -> Self {
        self.command_inputs = vec![NewProjectCommandInput::from_command_input(&self, input)];
        self
    }

    pub(crate) fn from_command(
        workflow_slug: WorkflowSlug,
        command: &NewCommandDefinition,
    ) -> Self {
        Self::new(
            workflow_slug,
            command.slice_slug().clone(),
            command.name().clone(),
        )
        .with_input(command.input())
        .with_errors(command.errors().clone())
    }

    pub(crate) fn with_errors(mut self, errors: CommandErrorDefinitions) -> Self {
        self.command_errors = errors
            .as_slice()
            .iter()
            .map(|error| NewProjectCommandError::from_command_error(&self, error))
            .collect();
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectCommandInput {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    command: CommandName,
    input: DatumName,
    source_kind: CommandInputSourceKind,
    source_description: String,
    provenance_chain: Vec<String>,
    event_stream_source_event: String,
    event_stream_source_attribute: String,
    external_payload_source_name: String,
    external_payload_source_field: String,
    generated_source_name: String,
    generated_source_field: String,
    session_source_name: String,
    session_source_field: String,
    invocation_argument_source_name: String,
    invocation_argument_source_field: String,
}

impl NewProjectCommandInput {
    fn from_command_input(command: &NewProjectCommand, input: &NewCommandInput) -> Self {
        Self {
            workflow_slug: command.workflow_slug.clone(),
            slice_slug: command.slice_slug.clone(),
            command: command.command.clone(),
            input: input.name().clone(),
            source_kind: input.source_kind(),
            source_description: input.source_description().as_ref().to_owned(),
            provenance_chain: input
                .provenance_chain()
                .as_slice()
                .iter()
                .map(|hop| hop.as_ref().to_owned())
                .collect(),
            event_stream_source_event: input
                .event_stream_source_event()
                .map_or("", EventName::as_ref)
                .to_owned(),
            event_stream_source_attribute: input
                .event_stream_source_attribute()
                .map_or("", EventAttributeName::as_ref)
                .to_owned(),
            external_payload_source_name: input
                .external_payload_source_name()
                .map_or("", EventAttributeSourceName::as_ref)
                .to_owned(),
            external_payload_source_field: input
                .external_payload_source_field()
                .map_or("", EventAttributeSourceField::as_ref)
                .to_owned(),
            generated_source_name: input
                .generated_source_name()
                .map_or("", EventAttributeSourceName::as_ref)
                .to_owned(),
            generated_source_field: input
                .generated_source_field()
                .map_or("", EventAttributeSourceField::as_ref)
                .to_owned(),
            session_source_name: input
                .session_source_name()
                .map_or("", EventAttributeSourceName::as_ref)
                .to_owned(),
            session_source_field: input
                .session_source_field()
                .map_or("", EventAttributeSourceField::as_ref)
                .to_owned(),
            invocation_argument_source_name: input
                .invocation_argument_source_name()
                .map_or("", EventAttributeSourceName::as_ref)
                .to_owned(),
            invocation_argument_source_field: input
                .invocation_argument_source_field()
                .map_or("", EventAttributeSourceField::as_ref)
                .to_owned(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectCommandError {
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
            recovery: *error.recovery_kind(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectDataFlow {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    datum: DatumName,
    source_kind: DataFlowSourceKind,
    source: DataFlowSource,
    transformation: TransformationSemantics,
    target: DataFlowTarget,
    bit_encoding: BitEncodingSemantics,
}

impl NewProjectDataFlow {
    pub(crate) fn from_slice_data_flow(
        workflow_slug: WorkflowSlug,
        data_flow: &NewBitLevelDataFlow,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug: data_flow.slice_slug().clone(),
            datum: data_flow.datum().clone(),
            source_kind: *data_flow.source_kind(),
            source: data_flow.source().clone(),
            transformation: *data_flow.transformation(),
            target: data_flow.target().clone(),
            bit_encoding: data_flow.bit_encoding().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectReadModel {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    read_model: ReadModelName,
    read_model_definitions: Vec<NewProjectReadModelDefinition>,
    read_model_fields: Vec<NewProjectReadModelField>,
}

impl NewProjectReadModel {
    pub(crate) fn new(
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

    pub(crate) fn with_definition(mut self, read_model: &NewReadModelDefinition) -> Self {
        self.read_model_definitions = vec![NewProjectReadModelDefinition::from_read_model(
            &self, read_model,
        )];
        self
    }

    pub(crate) fn with_field(mut self, field: &NewReadModelField) -> Self {
        self.read_model_fields = vec![NewProjectReadModelField::from_read_model_field(
            &self, field,
        )];
        self
    }

    pub(crate) fn from_read_model(
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
pub(crate) struct NewProjectReadModelDefinition {
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
pub(crate) struct NewProjectReadModelField {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    read_model: ReadModelName,
    field: DatumName,
    source_kind: ReadModelFieldSourceKind,
    source_event: String,
    source_attribute: String,
    derivation_rule: String,
    derivation_source_fields: Vec<String>,
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
            source_kind: field.source_kind(),
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
            derivation_source_fields: field
                .derivation_source_fields()
                .as_slice()
                .iter()
                .map(|field| field.as_ref().to_owned())
                .collect(),
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
pub(crate) struct NewProjectView {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    view: ViewName,
    view_definition: Option<NewProjectViewDefinition>,
    view_controls: Vec<NewProjectViewControl>,
    view_fields: Vec<NewProjectViewField>,
}

impl NewProjectView {
    pub(crate) fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug, view: ViewName) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            view,
            view_definition: None,
            view_controls: Vec::new(),
            view_fields: Vec::new(),
        }
    }

    pub(crate) fn with_field(mut self, field: &NewViewField) -> Self {
        self.view_fields = vec![NewProjectViewField::from_view_field(&self, field)];
        self
    }

    pub(crate) fn from_view(workflow_slug: WorkflowSlug, view: &NewViewDefinition) -> Self {
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
            local_states: view
                .local_states()
                .as_slice()
                .iter()
                .map(|target| target.as_ref().to_owned())
                .collect(),
            filters: view
                .filters()
                .as_slice()
                .iter()
                .map(|target| target.as_ref().to_owned())
                .collect(),
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
            input_source_kind: *input.source_kind(),
            input_source_description: input.source_description().clone(),
            input_sketch_token: input.sketch_token().clone(),
            input_visible_to_actor: input.visible_to_actor(),
            input_decision_field: input.decision_field(),
            handled_errors: control.handled_errors().as_slice().to_vec(),
            recovery_behavior: *control.recovery_behavior(),
            control_sketch_token: control.sketch_token().clone(),
            navigation_type: *navigation.target_type(),
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
pub(crate) struct NewProjectBoardElement {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    element: BoardElementName,
    kind: BoardElementKind,
    lane: BoardLaneId,
    declared_name: BoardElementDeclaredName,
    main_path: bool,
}

impl NewProjectBoardElement {
    pub(crate) fn from_slice_board_element(
        workflow_slug: WorkflowSlug,
        element: &NewBoardElement,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug: element.slice_slug().clone(),
            element: element.name().clone(),
            kind: *element.kind(),
            lane: *element.lane(),
            declared_name: element.declared_name().clone(),
            main_path: element.main_path(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectBoardConnection {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    source: BoardConnectionEndpoint,
    source_kind: BoardConnectionEndpointKind,
    target: BoardConnectionEndpoint,
    target_kind: BoardConnectionEndpointKind,
}

impl NewProjectBoardConnection {
    pub(crate) fn from_slice_board_connection(
        workflow_slug: WorkflowSlug,
        connection: &NewBoardConnection,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug: connection.slice_slug().clone(),
            source: connection.source().clone(),
            source_kind: *connection.source_kind(),
            target: connection.target().clone(),
            target_kind: *connection.target_kind(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectViewField {
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
            source_kind: *field.source_kind(),
            source_read_model: field.source_read_model().clone(),
            source_field: field.source_field().clone(),
            provenance: field.provenance_description().clone(),
            bit_encoding: field.bit_encoding().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectAutomation {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    automation: AutomationName,
    automation_definition: Option<NewProjectAutomationDefinition>,
}

impl NewProjectAutomation {
    pub(crate) fn new(
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

    pub(crate) fn from_automation(
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
pub(crate) struct NewProjectTranslation {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    translation: TranslationName,
    translation_definition: Option<NewProjectTranslationDefinition>,
}

impl NewProjectTranslation {
    pub(crate) fn new(
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

    pub(crate) fn from_translation(
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
pub(crate) struct NewProjectExternalPayload {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    external_payload: EventAttributeSourceName,
    external_payload_field: Option<NewProjectExternalPayloadField>,
}

impl NewProjectExternalPayload {
    pub(crate) fn from_external_payload(
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
pub(crate) struct NewProjectExternalPayloadField {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    external_payload: EventAttributeSourceName,
    field: EventAttributeSourceField,
    provenance: ProvenanceDescription,
    bit_encoding: BitEncodingSemantics,
}

impl NewProjectExternalPayloadField {
    pub(crate) fn from_external_payload(
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
pub(crate) struct NewProjectScenario {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    scenario_kind: ScenarioKind,
    scenario: ScenarioName,
    scenario_definition: Option<NewProjectScenarioDefinition>,
}

impl NewProjectScenario {
    pub(crate) fn from_slice_scenario(
        workflow_slug: WorkflowSlug,
        scenario: &NewSliceScenario,
    ) -> Self {
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
pub(crate) struct NewProjectScenarioDefinition {
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
    pub(crate) fn from_slice_scenario(
        workflow_slug: WorkflowSlug,
        scenario: &NewSliceScenario,
    ) -> Self {
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
            contract_kind: scenario.contract_kind().copied(),
            covered_definition: scenario.covered_definition().cloned(),
            error_references: scenario.error_references().as_slice().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NewProjectOutcome {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    outcome: OutcomeLabelName,
    events: OutcomeEventNames,
    externally_relevant: bool,
}

impl NewProjectOutcome {
    pub(crate) fn new(
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
pub(crate) struct NewProjectEvent {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    event: EventName,
    stream: StreamName,
    event_attributes: Vec<NewProjectEventAttribute>,
}

impl NewProjectEvent {
    pub(crate) fn new(
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

    pub(crate) fn with_attribute(mut self, attribute: &NewEventAttribute) -> Self {
        self.event_attributes = vec![NewProjectEventAttribute::from_event_attribute(
            &self, attribute,
        )];
        self
    }

    pub(crate) fn from_event(workflow_slug: WorkflowSlug, event: &NewEventDefinition) -> Self {
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
pub(crate) struct NewProjectEventAttribute {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    event: EventName,
    attribute: EventAttributeName,
    source_kind: EventAttributeSourceKind,
    source_name: EventAttributeSourceName,
    source_field: EventAttributeSourceField,
    generated_source_kind: Option<GeneratedEventAttributeSourceKind>,
    provenance: ProvenanceDescription,
}

impl NewProjectEventAttribute {
    fn from_event_attribute(event: &NewProjectEvent, attribute: &NewEventAttribute) -> Self {
        Self {
            workflow_slug: event.workflow_slug.clone(),
            slice_slug: event.slice_slug.clone(),
            event: event.event.clone(),
            attribute: attribute.name().clone(),
            source_kind: *attribute.source_kind(),
            source_name: attribute.source_name().clone(),
            source_field: attribute.source_field().clone(),
            generated_source_kind: attribute.generated_source_kind().cloned(),
            provenance: attribute.provenance_description().clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectStream {
    workflow_slug: String,
    slice_slug: String,
    stream: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectCommand {
    workflow_slug: String,
    slice_slug: String,
    command: String,
}

impl ProjectCommand {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectCommandInput {
    workflow_slug: String,
    slice_slug: String,
    command: String,
    input: String,
    source_kind: CommandInputSourceKind,
    source_description: String,
    provenance_chain: Vec<String>,
    event_stream_source_event: String,
    event_stream_source_attribute: String,
    external_payload_source_name: String,
    external_payload_source_field: String,
    generated_source_name: String,
    generated_source_field: String,
    session_source_name: String,
    session_source_field: String,
    invocation_argument_source_name: String,
    invocation_argument_source_field: String,
}

impl ProjectCommandInput {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }

    pub(crate) fn input(&self) -> &str {
        &self.input
    }

    pub(crate) fn source_kind(&self) -> CommandInputSourceKind {
        self.source_kind
    }

    pub(crate) fn source_description(&self) -> &str {
        &self.source_description
    }

    pub(crate) fn provenance_chain(&self) -> &[String] {
        &self.provenance_chain
    }

    pub(crate) fn event_stream_source_event(&self) -> &str {
        &self.event_stream_source_event
    }

    pub(crate) fn event_stream_source_attribute(&self) -> &str {
        &self.event_stream_source_attribute
    }

    pub(crate) fn external_payload_source_name(&self) -> &str {
        &self.external_payload_source_name
    }

    pub(crate) fn external_payload_source_field(&self) -> &str {
        &self.external_payload_source_field
    }

    pub(crate) fn generated_source_name(&self) -> &str {
        &self.generated_source_name
    }

    pub(crate) fn generated_source_field(&self) -> &str {
        &self.generated_source_field
    }

    pub(crate) fn session_source_name(&self) -> &str {
        &self.session_source_name
    }

    pub(crate) fn session_source_field(&self) -> &str {
        &self.session_source_field
    }

    pub(crate) fn invocation_argument_source_name(&self) -> &str {
        &self.invocation_argument_source_name
    }

    pub(crate) fn invocation_argument_source_field(&self) -> &str {
        &self.invocation_argument_source_field
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectCommandError {
    workflow_slug: String,
    slice_slug: String,
    command: String,
    error: String,
    scenario: String,
    recovery: String,
}

impl ProjectCommandError {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }

    pub(crate) fn error(&self) -> &str {
        &self.error
    }

    pub(crate) fn scenario(&self) -> &str {
        &self.scenario
    }

    pub(crate) fn recovery(&self) -> &str {
        &self.recovery
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectDataFlow {
    workflow_slug: String,
    slice_slug: String,
    datum: String,
    source_kind: DataFlowSourceKind,
    source: String,
    transformation: String,
    target: String,
    bit_encoding: String,
}

impl ProjectDataFlow {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn datum(&self) -> &str {
        &self.datum
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn source_kind(&self) -> DataFlowSourceKind {
        self.source_kind
    }

    pub(crate) fn transformation(&self) -> &str {
        &self.transformation
    }

    pub(crate) fn target(&self) -> &str {
        &self.target
    }

    pub(crate) fn bit_encoding(&self) -> &str {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectReadModel {
    workflow_slug: String,
    slice_slug: String,
    read_model: String,
}

impl ProjectReadModel {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn read_model(&self) -> &str {
        &self.read_model
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectReadModelDefinition {
    workflow_slug: String,
    slice_slug: String,
    read_model: String,
    transitive: bool,
    relationship_fields: Vec<String>,
    transitive_rule: String,
    example_scenario_name: String,
}

impl ProjectReadModelDefinition {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn read_model(&self) -> &str {
        &self.read_model
    }

    pub(crate) fn transitive(&self) -> bool {
        self.transitive
    }

    pub(crate) fn relationship_fields(&self) -> &[String] {
        &self.relationship_fields
    }

    pub(crate) fn transitive_rule(&self) -> &str {
        &self.transitive_rule
    }

    pub(crate) fn example_scenario_name(&self) -> &str {
        &self.example_scenario_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectReadModelField {
    workflow_slug: String,
    slice_slug: String,
    read_model: String,
    field: String,
    source_kind: String,
    source_event: String,
    source_attribute: String,
    derivation_rule: String,
    derivation_source_fields: Vec<String>,
    absence_event: String,
    derivation_scenario_name: String,
    absence_scenario_name: String,
    provenance: String,
}

impl ProjectReadModelField {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn read_model(&self) -> &str {
        &self.read_model
    }

    pub(crate) fn field(&self) -> &str {
        &self.field
    }

    pub(crate) fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub(crate) fn source_event(&self) -> &str {
        &self.source_event
    }

    pub(crate) fn source_attribute(&self) -> &str {
        &self.source_attribute
    }

    pub(crate) fn derivation_rule(&self) -> &str {
        &self.derivation_rule
    }

    pub(crate) fn derivation_source_fields(&self) -> &[String] {
        &self.derivation_source_fields
    }

    pub(crate) fn absence_event(&self) -> &str {
        &self.absence_event
    }

    pub(crate) fn derivation_scenario_name(&self) -> &str {
        &self.derivation_scenario_name
    }

    pub(crate) fn absence_scenario_name(&self) -> &str {
        &self.absence_scenario_name
    }

    pub(crate) fn provenance(&self) -> &str {
        &self.provenance
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectView {
    workflow_slug: String,
    slice_slug: String,
    view: String,
}

impl ProjectView {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn view(&self) -> &str {
        &self.view
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectViewDefinition {
    workflow_slug: String,
    slice_slug: String,
    view: String,
    read_models: Vec<String>,
    sketch_tokens: Vec<String>,
    local_states: Vec<String>,
    filters: Vec<String>,
}

impl ProjectViewDefinition {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn view(&self) -> &str {
        &self.view
    }

    pub(crate) fn read_models(&self) -> &[String] {
        &self.read_models
    }

    pub(crate) fn sketch_tokens(&self) -> &[String] {
        &self.sketch_tokens
    }

    pub(crate) fn local_states(&self) -> &[String] {
        &self.local_states
    }

    pub(crate) fn filters(&self) -> &[String] {
        &self.filters
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectViewControl {
    workflow_slug: String,
    slice_slug: String,
    view: String,
    control: String,
    command: String,
    input: String,
    input_source_kind: CommandInputSourceKind,
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
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn view(&self) -> &str {
        &self.view
    }

    pub(crate) fn control(&self) -> &str {
        &self.control
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }

    pub(crate) fn input(&self) -> &str {
        &self.input
    }

    pub(crate) fn input_source_kind(&self) -> CommandInputSourceKind {
        self.input_source_kind
    }

    pub(crate) fn input_source_description(&self) -> &str {
        &self.input_source_description
    }

    pub(crate) fn input_sketch_token(&self) -> &str {
        &self.input_sketch_token
    }

    pub(crate) fn input_visible_to_actor(&self) -> bool {
        self.input_visible_to_actor
    }

    pub(crate) fn input_decision_field(&self) -> bool {
        self.input_decision_field
    }

    pub(crate) fn handled_errors(&self) -> &[String] {
        &self.handled_errors
    }

    pub(crate) fn recovery_behavior(&self) -> &str {
        &self.recovery_behavior
    }

    pub(crate) fn control_sketch_token(&self) -> &str {
        &self.control_sketch_token
    }

    pub(crate) fn navigation_type(&self) -> &str {
        &self.navigation_type
    }

    pub(crate) fn navigation_target(&self) -> &str {
        &self.navigation_target
    }

    pub(crate) fn external_workflow(&self) -> &str {
        &self.external_workflow
    }

    pub(crate) fn external_system(&self) -> &str {
        &self.external_system
    }

    pub(crate) fn handoff_contract(&self) -> &str {
        &self.handoff_contract
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectBoardElement {
    workflow_slug: String,
    slice_slug: String,
    element: String,
    kind: String,
    lane: String,
    declared_name: String,
    main_path: bool,
}

impl ProjectBoardElement {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn element(&self) -> &str {
        &self.element
    }

    pub(crate) fn kind(&self) -> &str {
        &self.kind
    }

    pub(crate) fn lane(&self) -> &str {
        &self.lane
    }

    pub(crate) fn declared_name(&self) -> &str {
        &self.declared_name
    }

    pub(crate) fn main_path(&self) -> bool {
        self.main_path
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectBoardConnection {
    workflow_slug: String,
    slice_slug: String,
    source: String,
    source_kind: String,
    target: String,
    target_kind: String,
}

impl ProjectBoardConnection {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub(crate) fn target(&self) -> &str {
        &self.target
    }

    pub(crate) fn target_kind(&self) -> &str {
        &self.target_kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectViewField {
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
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn view(&self) -> &str {
        &self.view
    }

    pub(crate) fn field(&self) -> &str {
        &self.field
    }

    pub(crate) fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub(crate) fn source_read_model(&self) -> &str {
        &self.source_read_model
    }

    pub(crate) fn source_field(&self) -> &str {
        &self.source_field
    }

    pub(crate) fn provenance(&self) -> &str {
        &self.provenance
    }

    pub(crate) fn bit_encoding(&self) -> &str {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectAutomation {
    workflow_slug: String,
    slice_slug: String,
    automation: String,
}

impl ProjectAutomation {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn automation(&self) -> &str {
        &self.automation
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectAutomationDefinition {
    workflow_slug: String,
    slice_slug: String,
    automation: String,
    trigger: String,
    command: String,
    handled_errors: Vec<String>,
    reaction: String,
}

impl ProjectAutomationDefinition {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn automation(&self) -> &str {
        &self.automation
    }

    pub(crate) fn trigger(&self) -> &str {
        &self.trigger
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }

    pub(crate) fn handled_errors(&self) -> &[String] {
        &self.handled_errors
    }

    pub(crate) fn reaction(&self) -> &str {
        &self.reaction
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectTranslation {
    workflow_slug: String,
    slice_slug: String,
    translation: String,
}

impl ProjectTranslation {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn translation(&self) -> &str {
        &self.translation
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectTranslationDefinition {
    workflow_slug: String,
    slice_slug: String,
    translation: String,
    external_event: String,
    payload_contract: String,
    command: String,
}

impl ProjectTranslationDefinition {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn translation(&self) -> &str {
        &self.translation
    }

    pub(crate) fn external_event(&self) -> &str {
        &self.external_event
    }

    pub(crate) fn payload_contract(&self) -> &str {
        &self.payload_contract
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectExternalPayload {
    workflow_slug: String,
    slice_slug: String,
    external_payload: String,
}

impl ProjectExternalPayload {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn external_payload(&self) -> &str {
        &self.external_payload
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectExternalPayloadField {
    workflow_slug: String,
    slice_slug: String,
    external_payload: String,
    field: String,
    provenance: String,
    bit_encoding: String,
}

impl ProjectExternalPayloadField {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn external_payload(&self) -> &str {
        &self.external_payload
    }

    pub(crate) fn field(&self) -> &str {
        &self.field
    }

    pub(crate) fn provenance(&self) -> &str {
        &self.provenance
    }

    pub(crate) fn bit_encoding(&self) -> &str {
        &self.bit_encoding
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectScenario {
    workflow_slug: String,
    slice_slug: String,
    scenario_kind: String,
    scenario: String,
}

impl ProjectScenario {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn scenario_kind(&self) -> &str {
        &self.scenario_kind
    }

    pub(crate) fn scenario(&self) -> &str {
        &self.scenario
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectScenarioDefinition {
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
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn scenario_kind(&self) -> &str {
        &self.scenario_kind
    }

    pub(crate) fn scenario(&self) -> &str {
        &self.scenario
    }

    pub(crate) fn given(&self) -> &str {
        &self.given
    }

    pub(crate) fn when(&self) -> &str {
        &self.when
    }

    pub(crate) fn then(&self) -> &str {
        &self.then
    }

    pub(crate) fn read_streams(&self) -> &[String] {
        &self.read_streams
    }

    pub(crate) fn written_streams(&self) -> &[String] {
        &self.written_streams
    }

    pub(crate) fn contract_kind(&self) -> &str {
        &self.contract_kind
    }

    pub(crate) fn covered_definition(&self) -> &str {
        &self.covered_definition
    }

    pub(crate) fn error_references(&self) -> &[String] {
        &self.error_references
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectOutcome {
    workflow_slug: String,
    slice_slug: String,
    outcome: String,
    events: Vec<String>,
    externally_relevant: bool,
}

impl ProjectOutcome {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn outcome(&self) -> &str {
        &self.outcome
    }

    pub(crate) fn events(&self) -> &[String] {
        &self.events
    }

    pub(crate) fn externally_relevant(&self) -> bool {
        self.externally_relevant
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectEvent {
    workflow_slug: String,
    slice_slug: String,
    event: String,
    stream: String,
}

impl ProjectEvent {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn event(&self) -> &str {
        &self.event
    }

    pub(crate) fn stream(&self) -> &str {
        &self.stream
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ProjectEventAttribute {
    workflow_slug: String,
    slice_slug: String,
    event: String,
    attribute: String,
    source_kind: String,
    source_name: String,
    source_field: String,
    generated_source_kind: String,
    provenance: String,
}

impl ProjectEventAttribute {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn event(&self) -> &str {
        &self.event
    }

    pub(crate) fn attribute(&self) -> &str {
        &self.attribute
    }

    pub(crate) fn source_kind(&self) -> &str {
        &self.source_kind
    }

    pub(crate) fn source_name(&self) -> &str {
        &self.source_name
    }

    pub(crate) fn source_field(&self) -> &str {
        &self.source_field
    }

    pub(crate) fn generated_source_kind(&self) -> &str {
        &self.generated_source_kind
    }

    pub(crate) fn provenance(&self) -> &str {
        &self.provenance
    }
}

impl ProjectStream {
    pub(crate) fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub(crate) fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub(crate) fn stream(&self) -> &str {
        &self.stream
    }
}

// ---------------------------------------------------------------------------
// Pure project-root inventory projection.
//
// `projected_project_root_inventories` reproduces the project-root inventory
// rows directly from the in-memory event-log projection. Historically those
// rows were materialised by REPLAY — writing each definition into the root
// artifact through the `add_project_*` builders — and then PARSED BACK out of
// the artifact text by the `parse_lean_project_*` readers. This projection
// removes the parse-back: each row is built from the same `NewProject*`
// constructors the replay uses, then converted to its parsed `Project*` shape
// using the exact field rendering the artifact records carry. Because the
// `add_project_*` builders all dedup via `append_record_if_missing` (append a
// record only when no identical record text already exists), each inventory
// list here is deduplicated by `Project*` value equality, which is equivalent
// to record-text equality (the record is a 1:1 rendering of those fields).
// ---------------------------------------------------------------------------

/// The slice-level facts (with their containing workflow's slug) that feed the
/// project-root inventories. Workflow-level facts never feed the root.
pub(crate) struct ProjectedSliceInventoryFacts<'a> {
    pub(crate) workflow_slug: &'a WorkflowSlug,
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

/// Project the project-root inventories from the slice facts of every slice in
/// the model, in projection order (workflows, then slices, then each slice's
/// facts in `ProjectedSlice::effects` order). Every list is deduplicated by
/// value, matching the `append_record_if_missing` semantics of the replay
/// builders.
pub(crate) fn project_root_inventories_from_slices<'a>(
    slices: impl IntoIterator<Item = ProjectedSliceInventoryFacts<'a>>,
) -> ModeledProjectRootInventories {
    let mut rows = RootInventoryRows::default();
    for facts in slices {
        let workflow_slug = facts.workflow_slug;
        // Order mirrors `ProjectedSlice::effects`: scenarios, outcomes,
        // external payloads, events (stream then event), commands, read models,
        // data flows, views, translations, automations, board elements, board
        // connections. The lists are disjoint per fact family, so only the
        // per-family arrival order (preserved here) affects any single list.
        for scenario in facts.scenarios {
            NewProjectScenario::from_slice_scenario(workflow_slug.clone(), scenario)
                .collect_into(&mut rows);
        }
        for outcome in facts.outcomes {
            NewProjectOutcome::new(
                workflow_slug.clone(),
                outcome.slice_slug().clone(),
                outcome.label().clone(),
                outcome.event_set().clone(),
                outcome.externally_relevant(),
            )
            .collect_into(&mut rows);
        }
        for external_payload in facts.external_payloads {
            NewProjectExternalPayload::from_external_payload(
                workflow_slug.clone(),
                external_payload,
            )
            .collect_into(&mut rows);
        }
        for event in facts.event_definitions {
            NewProjectStream::new(
                workflow_slug.clone(),
                event.slice_slug().clone(),
                event.stream().clone(),
            )
            .collect_into(&mut rows);
            NewProjectEvent::from_event(workflow_slug.clone(), event).collect_into(&mut rows);
        }
        for command in facts.command_definitions {
            NewProjectCommand::from_command(workflow_slug.clone(), command).collect_into(&mut rows);
        }
        for read_model in facts.read_models {
            NewProjectReadModel::from_read_model(workflow_slug.clone(), read_model)
                .collect_into(&mut rows);
        }
        for data_flow in facts.bit_level_data_flows {
            NewProjectDataFlow::from_slice_data_flow(workflow_slug.clone(), data_flow)
                .collect_into(&mut rows);
        }
        for view in facts.views {
            NewProjectView::from_view(workflow_slug.clone(), view).collect_into(&mut rows);
        }
        for translation in facts.translations {
            NewProjectTranslation::from_translation(workflow_slug.clone(), translation)
                .collect_into(&mut rows);
        }
        for automation in facts.automations {
            NewProjectAutomation::from_automation(workflow_slug.clone(), automation)
                .collect_into(&mut rows);
        }
        for board_element in facts.board_elements {
            NewProjectBoardElement::from_slice_board_element(workflow_slug.clone(), board_element)
                .collect_into(&mut rows);
        }
        for board_connection in facts.board_connections {
            NewProjectBoardConnection::from_slice_board_connection(
                workflow_slug.clone(),
                board_connection,
            )
            .collect_into(&mut rows);
        }
    }
    ModeledProjectRootInventories::from_parts(rows.into_parts())
}

#[derive(Default)]
struct RootInventoryRows {
    scenarios: Vec<ProjectScenario>,
    scenario_definitions: Vec<ProjectScenarioDefinition>,
    data_flows: Vec<ProjectDataFlow>,
    outcomes: Vec<ProjectOutcome>,
    command_errors: Vec<ProjectCommandError>,
    commands: Vec<ProjectCommand>,
    command_inputs: Vec<ProjectCommandInput>,
    read_models: Vec<ProjectReadModel>,
    read_model_definitions: Vec<ProjectReadModelDefinition>,
    read_model_fields: Vec<ProjectReadModelField>,
    views: Vec<ProjectView>,
    view_definitions: Vec<ProjectViewDefinition>,
    view_controls: Vec<ProjectViewControl>,
    board_elements: Vec<ProjectBoardElement>,
    board_connections: Vec<ProjectBoardConnection>,
    view_fields: Vec<ProjectViewField>,
    automations: Vec<ProjectAutomation>,
    automation_definitions: Vec<ProjectAutomationDefinition>,
    translations: Vec<ProjectTranslation>,
    translation_definitions: Vec<ProjectTranslationDefinition>,
    external_payloads: Vec<ProjectExternalPayload>,
    external_payload_fields: Vec<ProjectExternalPayloadField>,
    streams: Vec<ProjectStream>,
    events: Vec<ProjectEvent>,
    event_attributes: Vec<ProjectEventAttribute>,
}

fn push_unique<T: PartialEq>(list: &mut Vec<T>, row: T) {
    if !list.contains(&row) {
        list.push(row);
    }
}

impl RootInventoryRows {
    fn into_parts(self) -> ModeledProjectRootInventoryParts {
        ModeledProjectRootInventoryParts {
            scenarios: self.scenarios,
            scenario_definitions: self.scenario_definitions,
            data_flows: self.data_flows,
            outcomes: self.outcomes,
            command_errors: self.command_errors,
            commands: self.commands,
            command_inputs: self.command_inputs,
            read_models: self.read_models,
            read_model_definitions: self.read_model_definitions,
            read_model_fields: self.read_model_fields,
            views: self.views,
            view_definitions: self.view_definitions,
            view_controls: self.view_controls,
            board_elements: self.board_elements,
            board_connections: self.board_connections,
            view_fields: self.view_fields,
            automations: self.automations,
            automation_definitions: self.automation_definitions,
            translations: self.translations,
            translation_definitions: self.translation_definitions,
            external_payloads: self.external_payloads,
            external_payload_fields: self.external_payload_fields,
            streams: self.streams,
            events: self.events,
            event_attributes: self.event_attributes,
        }
    }
}

impl NewProjectStream {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.streams,
            ProjectStream {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                stream: self.stream.as_ref().to_owned(),
            },
        );
    }
}

impl NewProjectCommand {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.commands,
            ProjectCommand {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                command: self.command.as_ref().to_owned(),
            },
        );
        for input in &self.command_inputs {
            push_unique(&mut rows.command_inputs, input.project_row());
        }
        for error in &self.command_errors {
            push_unique(&mut rows.command_errors, error.project_row());
        }
    }
}

impl NewProjectCommandInput {
    fn project_row(&self) -> ProjectCommandInput {
        ProjectCommandInput {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            command: self.command.as_ref().to_owned(),
            input: self.input.as_ref().to_owned(),
            source_kind: self.source_kind,
            source_description: self.source_description.clone(),
            provenance_chain: self.provenance_chain.clone(),
            event_stream_source_event: self.event_stream_source_event.clone(),
            event_stream_source_attribute: self.event_stream_source_attribute.clone(),
            external_payload_source_name: self.external_payload_source_name.clone(),
            external_payload_source_field: self.external_payload_source_field.clone(),
            generated_source_name: self.generated_source_name.clone(),
            generated_source_field: self.generated_source_field.clone(),
            session_source_name: self.session_source_name.clone(),
            session_source_field: self.session_source_field.clone(),
            invocation_argument_source_name: self.invocation_argument_source_name.clone(),
            invocation_argument_source_field: self.invocation_argument_source_field.clone(),
        }
    }
}

impl NewProjectCommandError {
    fn project_row(&self) -> ProjectCommandError {
        ProjectCommandError {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            command: self.command.as_ref().to_owned(),
            error: self.error.as_ref().to_owned(),
            scenario: self.scenario.as_ref().to_owned(),
            recovery: self.recovery.as_ref().to_owned(),
        }
    }
}

impl NewProjectDataFlow {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.data_flows,
            ProjectDataFlow {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                datum: self.datum.as_ref().to_owned(),
                source_kind: self.source_kind,
                source: self.source.as_ref().to_owned(),
                transformation: self.transformation.as_ref().to_owned(),
                target: self.target.as_ref().to_owned(),
                bit_encoding: self.bit_encoding.as_ref().to_owned(),
            },
        );
    }
}

impl NewProjectReadModel {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.read_models,
            ProjectReadModel {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                read_model: self.read_model.as_ref().to_owned(),
            },
        );
        for definition in &self.read_model_definitions {
            push_unique(&mut rows.read_model_definitions, definition.project_row());
        }
        for field in &self.read_model_fields {
            push_unique(&mut rows.read_model_fields, field.project_row());
        }
    }
}

impl NewProjectReadModelDefinition {
    fn project_row(&self) -> ProjectReadModelDefinition {
        ProjectReadModelDefinition {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            read_model: self.read_model.as_ref().to_owned(),
            transitive: self.transitive,
            relationship_fields: self.relationship_fields.clone(),
            transitive_rule: self.transitive_rule.clone(),
            example_scenario_name: self.example_scenario_name.clone(),
        }
    }
}

impl NewProjectReadModelField {
    fn project_row(&self) -> ProjectReadModelField {
        ProjectReadModelField {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            read_model: self.read_model.as_ref().to_owned(),
            field: self.field.as_ref().to_owned(),
            source_kind: self.source_kind.as_ref().to_owned(),
            source_event: self.source_event.clone(),
            source_attribute: self.source_attribute.clone(),
            derivation_rule: self.derivation_rule.clone(),
            derivation_source_fields: self.derivation_source_fields.clone(),
            absence_event: self.absence_event.clone(),
            derivation_scenario_name: self.derivation_scenario_name.clone(),
            absence_scenario_name: self.absence_scenario_name.clone(),
            provenance: self.provenance.as_ref().to_owned(),
        }
    }
}

impl NewProjectView {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.views,
            ProjectView {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                view: self.view.as_ref().to_owned(),
            },
        );
        if let Some(definition) = &self.view_definition {
            push_unique(&mut rows.view_definitions, definition.project_row());
        }
        for control in &self.view_controls {
            push_unique(&mut rows.view_controls, control.project_row());
        }
        for field in &self.view_fields {
            push_unique(&mut rows.view_fields, field.project_row());
        }
    }
}

impl NewProjectViewDefinition {
    fn project_row(&self) -> ProjectViewDefinition {
        ProjectViewDefinition {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            view: self.view.as_ref().to_owned(),
            read_models: self
                .read_models
                .iter()
                .map(|read_model| read_model.as_ref().to_owned())
                .collect(),
            sketch_tokens: self
                .sketch_tokens
                .iter()
                .map(|token| token.as_ref().to_owned())
                .collect(),
            local_states: self.local_states.clone(),
            filters: self.filters.clone(),
        }
    }
}

impl NewProjectViewControl {
    fn project_row(&self) -> ProjectViewControl {
        ProjectViewControl {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            view: self.view.as_ref().to_owned(),
            control: self.control.as_ref().to_owned(),
            command: self.command.as_ref().to_owned(),
            input: self.input.as_ref().to_owned(),
            input_source_kind: self.input_source_kind,
            input_source_description: self.input_source_description.as_ref().to_owned(),
            input_sketch_token: self.input_sketch_token.as_ref().to_owned(),
            input_visible_to_actor: self.input_visible_to_actor,
            input_decision_field: self.input_decision_field,
            handled_errors: self
                .handled_errors
                .iter()
                .map(|error| error.as_ref().to_owned())
                .collect(),
            recovery_behavior: self.recovery_behavior.as_ref().to_owned(),
            control_sketch_token: self.control_sketch_token.as_ref().to_owned(),
            navigation_type: self.navigation_type.as_ref().to_owned(),
            navigation_target: self.navigation_target.as_ref().to_owned(),
            external_workflow: self.external_workflow.clone(),
            external_system: self.external_system.clone(),
            handoff_contract: self.handoff_contract.clone(),
        }
    }
}

impl NewProjectViewField {
    fn project_row(&self) -> ProjectViewField {
        ProjectViewField {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            view: self.view.as_ref().to_owned(),
            field: self.field.as_ref().to_owned(),
            source_kind: self.source_kind.as_ref().to_owned(),
            source_read_model: self.source_read_model.as_ref().to_owned(),
            source_field: self.source_field.as_ref().to_owned(),
            provenance: self.provenance.as_ref().to_owned(),
            bit_encoding: self.bit_encoding.as_ref().to_owned(),
        }
    }
}

impl NewProjectBoardElement {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.board_elements,
            ProjectBoardElement {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                element: self.element.as_ref().to_owned(),
                kind: self.kind.as_ref().to_owned(),
                lane: self.lane.as_ref().to_owned(),
                declared_name: self.declared_name.as_ref().to_owned(),
                main_path: self.main_path,
            },
        );
    }
}

impl NewProjectBoardConnection {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.board_connections,
            ProjectBoardConnection {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                source: self.source.as_ref().to_owned(),
                source_kind: self.source_kind.as_ref().to_owned(),
                target: self.target.as_ref().to_owned(),
                target_kind: self.target_kind.as_ref().to_owned(),
            },
        );
    }
}

impl NewProjectAutomation {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.automations,
            ProjectAutomation {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                automation: self.automation.as_ref().to_owned(),
            },
        );
        if let Some(definition) = &self.automation_definition {
            push_unique(&mut rows.automation_definitions, definition.project_row());
        }
    }
}

impl NewProjectAutomationDefinition {
    fn project_row(&self) -> ProjectAutomationDefinition {
        ProjectAutomationDefinition {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            automation: self.automation.as_ref().to_owned(),
            trigger: self.trigger.as_ref().to_owned(),
            command: self.command.as_ref().to_owned(),
            handled_errors: self
                .handled_errors
                .iter()
                .map(|error| error.as_ref().to_owned())
                .collect(),
            reaction: self.reaction.as_ref().to_owned(),
        }
    }
}

impl NewProjectTranslation {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.translations,
            ProjectTranslation {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                translation: self.translation.as_ref().to_owned(),
            },
        );
        if let Some(definition) = &self.translation_definition {
            push_unique(&mut rows.translation_definitions, definition.project_row());
        }
    }
}

impl NewProjectTranslationDefinition {
    fn project_row(&self) -> ProjectTranslationDefinition {
        ProjectTranslationDefinition {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            translation: self.translation.as_ref().to_owned(),
            external_event: self.external_event.as_ref().to_owned(),
            payload_contract: self.payload_contract.as_ref().to_owned(),
            command: self.command.as_ref().to_owned(),
        }
    }
}

impl NewProjectExternalPayload {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.external_payloads,
            ProjectExternalPayload {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                external_payload: self.external_payload.as_ref().to_owned(),
            },
        );
        if let Some(field) = &self.external_payload_field {
            push_unique(&mut rows.external_payload_fields, field.project_row());
        }
    }
}

impl NewProjectExternalPayloadField {
    fn project_row(&self) -> ProjectExternalPayloadField {
        ProjectExternalPayloadField {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            external_payload: self.external_payload.as_ref().to_owned(),
            field: self.field.as_ref().to_owned(),
            provenance: self.provenance.as_ref().to_owned(),
            bit_encoding: self.bit_encoding.as_ref().to_owned(),
        }
    }
}

impl NewProjectScenario {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.scenarios,
            ProjectScenario {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                scenario_kind: self.scenario_kind.as_str().to_owned(),
                scenario: self.scenario.as_ref().to_owned(),
            },
        );
        if let Some(definition) = &self.scenario_definition {
            push_unique(&mut rows.scenario_definitions, definition.project_row());
        }
    }
}

impl NewProjectScenarioDefinition {
    fn project_row(&self) -> ProjectScenarioDefinition {
        ProjectScenarioDefinition {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            scenario_kind: self.scenario_kind.as_str().to_owned(),
            scenario: self.scenario.as_ref().to_owned(),
            given: self.given.as_ref().to_owned(),
            when: self.when.as_ref().to_owned(),
            then: self.then.as_ref().to_owned(),
            read_streams: self
                .read_streams
                .iter()
                .map(|stream| stream.as_ref().to_owned())
                .collect(),
            written_streams: self
                .written_streams
                .iter()
                .map(|stream| stream.as_ref().to_owned())
                .collect(),
            contract_kind: self
                .contract_kind
                .as_ref()
                .map_or("", ContractKindName::as_ref)
                .to_owned(),
            covered_definition: self
                .covered_definition
                .as_ref()
                .map_or("", CoveredDefinitionName::as_ref)
                .to_owned(),
            error_references: self
                .error_references
                .iter()
                .map(|error| error.as_ref().to_owned())
                .collect(),
        }
    }
}

impl NewProjectOutcome {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.outcomes,
            ProjectOutcome {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                outcome: self.outcome.as_ref().to_owned(),
                events: self
                    .events
                    .as_slice()
                    .iter()
                    .map(|event| event.as_ref().to_owned())
                    .collect(),
                externally_relevant: self.externally_relevant,
            },
        );
    }
}

impl NewProjectEvent {
    fn collect_into(&self, rows: &mut RootInventoryRows) {
        push_unique(
            &mut rows.events,
            ProjectEvent {
                workflow_slug: self.workflow_slug.as_ref().to_owned(),
                slice_slug: self.slice_slug.as_ref().to_owned(),
                event: self.event.as_ref().to_owned(),
                stream: self.stream.as_ref().to_owned(),
            },
        );
        for attribute in &self.event_attributes {
            push_unique(&mut rows.event_attributes, attribute.project_row());
        }
    }
}

impl NewProjectEventAttribute {
    fn project_row(&self) -> ProjectEventAttribute {
        ProjectEventAttribute {
            workflow_slug: self.workflow_slug.as_ref().to_owned(),
            slice_slug: self.slice_slug.as_ref().to_owned(),
            event: self.event.as_ref().to_owned(),
            attribute: self.attribute.as_ref().to_owned(),
            source_kind: self.source_kind.as_ref().to_owned(),
            source_name: self.source_name.as_ref().to_owned(),
            source_field: self.source_field.as_ref().to_owned(),
            generated_source_kind: self
                .generated_source_kind
                .as_ref()
                .map_or("", GeneratedEventAttributeSourceKind::as_ref)
                .to_owned(),
            provenance: self.provenance.as_ref().to_owned(),
        }
    }
}
