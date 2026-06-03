use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::iter;

use serde_json::Value;

use crate::core::browser_data_document::{BrowserDataCorpus, BrowserDataDocument};
use crate::core::effect::FileContents;
use crate::core::types::{
    BoardLaneId, BrowserEventElementName, CommandErrorName, CommandName, ControlEffectKind,
    ControlEffectTarget, ControlLabel, DefinitionSectionLabel, ReviewRuleName, ReviewStatus,
    SliceName, SourceChainHop, SourceControlReference, ViewFieldName, ViewName,
    WorkflowBranchLabel, WorkflowStepName, WorkflowTransitionKind, WorkflowTransitionLabel,
    WorkflowTransitionName,
};
use crate::core::workflow_document::WorkflowDocument;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserWorkflow {
    lane_ids: Vec<BoardLaneId>,
    main_path_names: Vec<WorkflowStepName>,
    branch_cards: Vec<BrowserBranchCard>,
    transition_cards: Vec<BrowserTransitionCard>,
    error_recovery_cards: Vec<BrowserErrorRecoveryCard>,
    event_element_names: Vec<BrowserEventElementName>,
    review_overlays: Vec<BrowserReviewOverlay>,
    command_definitions: Vec<BrowserCommandDefinition>,
    view_definitions: Vec<BrowserViewDefinition>,
}

impl BrowserWorkflow {
    pub fn lane_ids(&self) -> &[BoardLaneId] {
        &self.lane_ids
    }

    pub fn main_path_names(&self) -> &[WorkflowStepName] {
        &self.main_path_names
    }

    pub fn branch_cards(&self) -> &[BrowserBranchCard] {
        &self.branch_cards
    }

    pub fn transition_cards(&self) -> &[BrowserTransitionCard] {
        &self.transition_cards
    }

    pub fn error_recovery_cards(&self) -> &[BrowserErrorRecoveryCard] {
        &self.error_recovery_cards
    }

    pub fn event_element_names(&self) -> &[BrowserEventElementName] {
        &self.event_element_names
    }

    pub fn review_overlays(&self) -> &[BrowserReviewOverlay] {
        &self.review_overlays
    }

    pub fn command_definitions(&self) -> &[BrowserCommandDefinition] {
        &self.command_definitions
    }

    pub fn view_definitions(&self) -> &[BrowserViewDefinition] {
        &self.view_definitions
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserBranchCard {
    name: WorkflowStepName,
    label: WorkflowBranchLabel,
}

impl BrowserBranchCard {
    pub fn name(&self) -> &WorkflowStepName {
        &self.name
    }

    pub fn label(&self) -> &WorkflowBranchLabel {
        &self.label
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserTransitionCard {
    name: WorkflowTransitionName,
    source: WorkflowStepName,
    target: WorkflowStepName,
    kind: WorkflowTransitionKind,
    label: WorkflowTransitionLabel,
}

impl BrowserTransitionCard {
    pub fn name(&self) -> &WorkflowTransitionName {
        &self.name
    }

    pub fn source(&self) -> &WorkflowStepName {
        &self.source
    }

    pub fn target(&self) -> &WorkflowStepName {
        &self.target
    }

    pub fn kind(&self) -> &WorkflowTransitionKind {
        &self.kind
    }

    pub fn label(&self) -> &WorkflowTransitionLabel {
        &self.label
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserErrorRecoveryCard {
    name: CommandErrorName,
    source_screen: ViewName,
}

impl BrowserErrorRecoveryCard {
    pub fn name(&self) -> &CommandErrorName {
        &self.name
    }

    pub fn source_screen(&self) -> &ViewName {
        &self.source_screen
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserReviewOverlay {
    step: WorkflowStepName,
    status: ReviewStatus,
    missing_rule: ReviewRuleName,
}

impl BrowserReviewOverlay {
    pub fn step(&self) -> &WorkflowStepName {
        &self.step
    }

    pub fn status(&self) -> &ReviewStatus {
        &self.status
    }

    pub fn missing_rule(&self) -> &ReviewRuleName {
        &self.missing_rule
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserCommandDefinition {
    name: CommandName,
    owning_slice: SliceName,
    source_controls: Vec<SourceControlReference>,
    section_labels: Vec<DefinitionSectionLabel>,
}

impl BrowserCommandDefinition {
    pub fn name(&self) -> &CommandName {
        &self.name
    }

    pub fn owning_slice(&self) -> &SliceName {
        &self.owning_slice
    }

    pub fn source_controls(&self) -> &[SourceControlReference] {
        &self.source_controls
    }

    pub fn section_labels(&self) -> &[DefinitionSectionLabel] {
        &self.section_labels
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserViewDefinition {
    name: ViewName,
    field_source_chains: Vec<BrowserFieldSourceChain>,
    control_effects: Vec<BrowserControlEffect>,
}

impl BrowserViewDefinition {
    pub fn name(&self) -> &ViewName {
        &self.name
    }

    pub fn field_source_chains(&self) -> &[BrowserFieldSourceChain] {
        &self.field_source_chains
    }

    pub fn control_effects(&self) -> &[BrowserControlEffect] {
        &self.control_effects
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserFieldSourceChain {
    field: ViewFieldName,
    hops: Vec<SourceChainHop>,
}

impl BrowserFieldSourceChain {
    pub fn field(&self) -> &ViewFieldName {
        &self.field
    }

    pub fn hops(&self) -> &[SourceChainHop] {
        &self.hops
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserControlEffect {
    label: ControlLabel,
    kind: ControlEffectKind,
    target: ControlEffectTarget,
}

impl BrowserControlEffect {
    pub fn label(&self) -> &ControlLabel {
        &self.label
    }

    pub fn kind(&self) -> &ControlEffectKind {
        &self.kind
    }

    pub fn target(&self) -> &ControlEffectTarget {
        &self.target
    }
}

pub fn compose_browser_workflow(
    workflow_document: FileContents,
    slice_documents: Vec<FileContents>,
) -> Result<BrowserWorkflow, BrowserCompositionError> {
    let workflow_semantics = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
    let workflow_browser_data = BrowserDataDocument::parse(&workflow_document)
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
    let workflow_value = parse_json(workflow_document.as_ref())?;
    let slice_browser_data = slice_documents
        .iter()
        .map(BrowserDataDocument::parse)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
    let browser_data = BrowserDataCorpus::new(
        iter::once(workflow_browser_data)
            .chain(slice_browser_data)
            .collect(),
    );
    let slice_values = slice_documents
        .iter()
        .map(|document| parse_json(document.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;
    let lane_ids = browser_data
        .board_lane_ids()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
    let main_path_names = workflow_semantics
        .main_path_step_names()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
    let branch_cards = workflow_semantics
        .branch_details()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?
        .into_iter()
        .map(|detail| BrowserBranchCard {
            name: detail.name().clone(),
            label: detail.label().clone(),
        })
        .collect();
    let transition_cards = workflow_semantics
        .transition_details()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?
        .into_iter()
        .map(|detail| BrowserTransitionCard {
            name: detail.name().clone(),
            source: detail.source().clone(),
            target: detail.target().clone(),
            kind: detail.kind().clone(),
            label: detail.label().clone(),
        })
        .collect();
    let composed_values = iter::once(&workflow_value)
        .chain(slice_values.iter())
        .collect::<Vec<_>>();
    let error_recovery_cards = browser_data
        .error_recovery_details()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?
        .into_iter()
        .map(|detail| BrowserErrorRecoveryCard {
            name: detail.name().clone(),
            source_screen: detail.source_screen().clone(),
        })
        .collect();
    let event_element_names = browser_data
        .event_element_names()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
    let review_overlays = workflow_semantics
        .review_overlay_details()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?
        .into_iter()
        .map(|detail| BrowserReviewOverlay {
            step: detail.step().clone(),
            status: detail.status().clone(),
            missing_rule: detail.missing_rule().clone(),
        })
        .collect();
    let command_definitions = browser_data
        .command_definition_details()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?
        .into_iter()
        .map(|detail| BrowserCommandDefinition {
            name: detail.name().clone(),
            owning_slice: detail.owning_slice().clone(),
            source_controls: detail.source_controls().to_vec(),
            section_labels: detail.section_labels().to_vec(),
        })
        .collect();
    let view_definitions = view_definitions(&composed_values)?;

    Ok(BrowserWorkflow {
        lane_ids,
        main_path_names,
        branch_cards,
        transition_cards,
        error_recovery_cards,
        event_element_names,
        review_overlays,
        command_definitions,
        view_definitions,
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserCompositionError {
    message: String,
}

impl BrowserCompositionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for BrowserCompositionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BrowserCompositionError {}

fn parse_json(source: &str) -> Result<Value, BrowserCompositionError> {
    serde_json::from_str::<Value>(source)
        .map_err(|error| BrowserCompositionError::new(format!("invalid browser JSON: {error}")))
}

fn view_definitions(
    values: &[&Value],
) -> Result<Vec<BrowserViewDefinition>, BrowserCompositionError> {
    values
        .iter()
        .flat_map(|value| {
            value
                .get("views")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|view| {
            view.get("name")
                .and_then(Value::as_str)
                .map(|name| (name, view))
        })
        .map(|(name, view)| {
            Ok(BrowserViewDefinition {
                name: ViewName::try_new(name.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid view name: {error}"))
                })?,
                field_source_chains: view_field_source_chains(values, view)?,
                control_effects: view_control_effects(view)?,
            })
        })
        .collect()
}

fn view_field_source_chains(
    values: &[&Value],
    view: &Value,
) -> Result<Vec<BrowserFieldSourceChain>, BrowserCompositionError> {
    view.get("fields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|field| {
            Some((
                field.get("name").and_then(Value::as_str)?,
                field.get("source").and_then(Value::as_str)?,
            ))
        })
        .map(|(field, source)| {
            Ok(BrowserFieldSourceChain {
                field: ViewFieldName::try_new(field.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid view field name: {error}"))
                })?,
                hops: source_chain_hops(values, source)?,
            })
        })
        .collect()
}

fn view_control_effects(
    view: &Value,
) -> Result<Vec<BrowserControlEffect>, BrowserCompositionError> {
    view.get("controls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|control| {
            Some((
                control.get("label").and_then(Value::as_str)?,
                control_effect_kind_and_target(control)?,
            ))
        })
        .map(|(label, (kind, target))| {
            Ok(BrowserControlEffect {
                label: ControlLabel::try_new(label.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid control label: {error}"))
                })?,
                kind: ControlEffectKind::try_new(kind.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid control effect kind: {error}"))
                })?,
                target: ControlEffectTarget::try_new(target.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid control effect target: {error}"))
                })?,
            })
        })
        .collect()
}

fn control_effect_kind_and_target(control: &Value) -> Option<(&'static str, &str)> {
    control
        .get("command")
        .and_then(Value::as_str)
        .map(|command| ("command", command))
        .or_else(|| {
            control
                .get("navigation")
                .and_then(Value::as_str)
                .map(|navigation| (navigation_effect_kind(control), navigation))
        })
}

fn navigation_effect_kind(control: &Value) -> &'static str {
    match control.get("navigation_type").and_then(Value::as_str) {
        Some("external_workflow") => "workflow navigation",
        Some("external_system") => "external navigation",
        Some("local_view_state") => "local navigation",
        Some("modeled_view") => "view navigation",
        _ => "navigation",
    }
}

fn source_chain_hops(
    values: &[&Value],
    source: &str,
) -> Result<Vec<SourceChainHop>, BrowserCompositionError> {
    let raw_hops = iter::once(source)
        .chain(read_model_field_source(values, source))
        .chain(event_attribute_source(values, source))
        .collect::<Vec<_>>();

    raw_hops
        .into_iter()
        .map(|hop| {
            SourceChainHop::try_new(hop.to_owned()).map_err(|error| {
                BrowserCompositionError::new(format!("invalid source chain hop: {error}"))
            })
        })
        .collect()
}

fn read_model_field_source<'a>(values: &'a [&Value], source: &str) -> Option<&'a str> {
    let (read_model_name, field_name) = source
        .strip_prefix("read_model.")
        .and_then(|source| source.split_once('.'))?;

    values
        .iter()
        .flat_map(|value| {
            value
                .get("read_models")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find(|read_model| {
            read_model
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == read_model_name)
        })
        .and_then(|read_model| {
            read_model
                .get("fields")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .find(|field| {
                    field
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| name == field_name)
                })
        })
        .and_then(|field| field.get("source").and_then(Value::as_str))
}

fn event_attribute_source<'a>(values: &'a [&Value], source: &str) -> Option<&'a str> {
    let (event_name, attribute_name) = read_model_field_source(values, source)?.split_once('.')?;

    values
        .iter()
        .flat_map(|value| {
            value
                .get("events")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find(|event| {
            event
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == event_name)
        })
        .and_then(|event| {
            event
                .get("attributes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .find(|attribute| {
                    attribute
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| name == attribute_name)
                })
        })
        .and_then(|attribute| attribute.get("source").and_then(Value::as_str))
}
