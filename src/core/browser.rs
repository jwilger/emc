use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::iter;

use serde_json::Value;

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
    let workflow_value = parse_json(workflow_document.as_ref())?;
    let slice_values = slice_documents
        .iter()
        .map(|document| parse_json(document.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;
    let lane_ids = iter::once(&workflow_value)
        .chain(slice_values.iter())
        .flat_map(board_lane_ids)
        .try_fold(Vec::<BoardLaneId>::new(), |mut lanes, lane| {
            let parsed = BoardLaneId::try_new(lane.to_owned()).map_err(|error| {
                BrowserCompositionError::new(format!("invalid board lane id: {error}"))
            })?;
            if !lanes.iter().any(|existing| existing == &parsed) {
                lanes.push(parsed);
            }
            Ok(lanes)
        })?;
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
    let transition_cards = workflow_transition_cards(&workflow_value)?;
    let composed_values = iter::once(&workflow_value)
        .chain(slice_values.iter())
        .collect::<Vec<_>>();
    let error_recovery_cards = error_recovery_cards(&composed_values)?;
    let event_element_names = event_element_names(&composed_values)?;
    let review_overlays = review_overlays(&workflow_value)?;
    let command_definitions = command_definitions(&composed_values)?;
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

fn board_lane_ids(value: &Value) -> impl Iterator<Item = &str> {
    value
        .get("board")
        .and_then(|board| board.get("lanes"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|lane| lane.get("id").and_then(Value::as_str))
}

fn workflow_transition_cards(
    value: &Value,
) -> Result<Vec<BrowserTransitionCard>, BrowserCompositionError> {
    value
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|step| step_transition_cards(value, step))
        .collect::<Result<Vec<_>, _>>()
        .map(|cards| cards.into_iter().flatten().collect())
}

fn step_transition_cards(
    workflow_value: &Value,
    step: &Value,
) -> Result<Vec<BrowserTransitionCard>, BrowserCompositionError> {
    let source = step.get("name").and_then(Value::as_str);

    step.get("transitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|transition| {
            let (kind, label) = transition_kind_and_label(transition)?;
            Some((
                transition_display_name(transition, label),
                source?,
                transition_target_name(workflow_value, transition)?,
                (kind, label),
            ))
        })
        .map(|(name, source, target, (kind, label))| {
            Ok(BrowserTransitionCard {
                name: parse_workflow_transition_name(name)?,
                source: parse_workflow_step_name(source)?,
                target: parse_workflow_step_name(target)?,
                kind: WorkflowTransitionKind::try_new(kind.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!(
                        "invalid workflow transition kind: {error}"
                    ))
                })?,
                label: WorkflowTransitionLabel::try_new(label.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!(
                        "invalid workflow transition label: {error}"
                    ))
                })?,
            })
        })
        .collect()
}

fn transition_target_name<'a>(workflow_value: &'a Value, transition: &'a Value) -> Option<&'a str> {
    transition
        .get("to")
        .and_then(Value::as_str)
        .map(|target_slice| {
            workflow_step_name_for_slice(workflow_value, target_slice).unwrap_or(target_slice)
        })
        .or_else(|| transition.get("target_name").and_then(Value::as_str))
        .or_else(|| transition.get("to_workflow").and_then(Value::as_str))
}

fn workflow_step_name_for_slice<'a>(workflow_value: &'a Value, slice: &str) -> Option<&'a str> {
    workflow_value
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|step| {
            step.get("slice")
                .and_then(Value::as_str)
                .is_some_and(|step_slice| step_slice == slice)
        })
        .and_then(|step| step.get("name").and_then(Value::as_str))
}

fn transition_kind_and_label(transition: &Value) -> Option<(&'static str, &str)> {
    transition
        .get("retry")
        .and_then(Value::as_bool)
        .filter(|retry| *retry)
        .map(|_| ("retry", "retry"))
        .or_else(|| {
            transition
                .get("via_navigation")
                .and_then(Value::as_str)
                .map(|label| ("navigation", label))
        })
        .or_else(|| {
            transition
                .get("via_command")
                .and_then(Value::as_str)
                .map(|label| ("command", label))
        })
        .or_else(|| {
            transition
                .get("via_event")
                .and_then(Value::as_str)
                .map(|label| ("event", label))
        })
        .or_else(|| {
            transition
                .get("via_external_trigger")
                .and_then(Value::as_str)
                .map(|label| ("external trigger", label))
        })
        .or_else(|| {
            transition
                .get("via_outcome")
                .and_then(Value::as_str)
                .map(|label| ("workflow exit", label))
        })
}

fn transition_display_name<'a>(transition: &'a Value, label: &'a str) -> &'a str {
    transition
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(label)
}

fn parse_workflow_step_name(value: &str) -> Result<WorkflowStepName, BrowserCompositionError> {
    WorkflowStepName::try_new(value.to_owned()).map_err(|error| {
        BrowserCompositionError::new(format!("invalid workflow step name: {error}"))
    })
}

fn parse_workflow_transition_name(
    value: &str,
) -> Result<WorkflowTransitionName, BrowserCompositionError> {
    WorkflowTransitionName::try_new(value.to_owned()).map_err(|error| {
        BrowserCompositionError::new(format!("invalid workflow transition name: {error}"))
    })
}

fn error_recovery_cards(
    values: &[&Value],
) -> Result<Vec<BrowserErrorRecoveryCard>, BrowserCompositionError> {
    values
        .iter()
        .flat_map(|value| {
            value
                .get("views")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .map(view_error_recovery_cards)
        .collect::<Result<Vec<_>, _>>()
        .map(|cards| cards.into_iter().flatten().collect())
}

fn view_error_recovery_cards(
    view: &Value,
) -> Result<Vec<BrowserErrorRecoveryCard>, BrowserCompositionError> {
    let source_screen = view.get("name").and_then(Value::as_str);

    view.get("controls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|control| {
            control
                .get("error_handling")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|handling| {
            Some((
                source_screen?,
                handling.get("error").and_then(Value::as_str)?,
            ))
        })
        .map(|(source_screen, error_name)| {
            Ok(BrowserErrorRecoveryCard {
                name: CommandErrorName::try_new(error_name.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid command error name: {error}"))
                })?,
                source_screen: ViewName::try_new(source_screen.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid source screen name: {error}"))
                })?,
            })
        })
        .collect()
}

fn event_element_names(
    values: &[&Value],
) -> Result<Vec<BrowserEventElementName>, BrowserCompositionError> {
    values
        .iter()
        .flat_map(|value| {
            value
                .get("board")
                .and_then(|board| board.get("slices"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .flat_map(|slice| {
            slice
                .get("elements")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|element| {
            element
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "event")
        })
        .filter_map(|element| element.get("name").and_then(Value::as_str))
        .map(|name| {
            BrowserEventElementName::try_new(name.to_owned()).map_err(|error| {
                BrowserCompositionError::new(format!("invalid event element name: {error}"))
            })
        })
        .collect()
}

fn review_overlays(value: &Value) -> Result<Vec<BrowserReviewOverlay>, BrowserCompositionError> {
    value
        .get("review_diagnostics")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|diagnostic| {
            Some((
                diagnostic.get("step").and_then(Value::as_str)?,
                diagnostic.get("status").and_then(Value::as_str)?,
                diagnostic.get("missing_rule").and_then(Value::as_str)?,
            ))
        })
        .map(|(step, status, missing_rule)| {
            Ok(BrowserReviewOverlay {
                step: parse_workflow_step_name(step)?,
                status: ReviewStatus::try_new(status.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid review status: {error}"))
                })?,
                missing_rule: ReviewRuleName::try_new(missing_rule.to_owned()).map_err(
                    |error| {
                        BrowserCompositionError::new(format!("invalid review rule name: {error}"))
                    },
                )?,
            })
        })
        .collect()
}

fn command_definitions(
    values: &[&Value],
) -> Result<Vec<BrowserCommandDefinition>, BrowserCompositionError> {
    values
        .iter()
        .flat_map(|value| {
            value
                .get("commands")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|command| command.get("name").and_then(Value::as_str))
        .map(|name| {
            Ok(BrowserCommandDefinition {
                name: CommandName::try_new(name.to_owned()).map_err(|error| {
                    BrowserCompositionError::new(format!("invalid command name: {error}"))
                })?,
                owning_slice: command_owning_slice(values, name)?,
                source_controls: command_source_controls(values, name)?,
                section_labels: command_section_labels()?,
            })
        })
        .collect()
}

fn command_owning_slice(
    values: &[&Value],
    command_name: &str,
) -> Result<SliceName, BrowserCompositionError> {
    values
        .iter()
        .flat_map(|value| {
            value
                .get("slices")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find(|slice| {
            slice
                .get("commands")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|command| {
                    command
                        .as_str()
                        .is_some_and(|slice_command| slice_command == command_name)
                })
        })
        .and_then(|slice| slice.get("name").and_then(Value::as_str))
        .ok_or_else(|| {
            BrowserCompositionError::new(format!("command '{command_name}' has no owning slice"))
        })
        .and_then(|slice| {
            SliceName::try_new(slice.to_owned()).map_err(|error| {
                BrowserCompositionError::new(format!("invalid owning slice name: {error}"))
            })
        })
}

fn command_source_controls(
    values: &[&Value],
    command_name: &str,
) -> Result<Vec<SourceControlReference>, BrowserCompositionError> {
    values
        .iter()
        .flat_map(|value| {
            value
                .get("views")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .flat_map(|view| view_command_source_controls(view, command_name))
        .map(|source| {
            SourceControlReference::try_new(source).map_err(|error| {
                BrowserCompositionError::new(format!("invalid source control reference: {error}"))
            })
        })
        .collect()
}

fn view_command_source_controls(view: &Value, command_name: &str) -> Vec<String> {
    view.get("name")
        .and_then(Value::as_str)
        .map(|view_name| {
            view.get("controls")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|control| {
                    control
                        .get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|command| command == command_name)
                })
                .filter_map(|control| control.get("label").and_then(Value::as_str))
                .map(|label| format!("{view_name} / {label}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn command_section_labels() -> Result<Vec<DefinitionSectionLabel>, BrowserCompositionError> {
    [
        "Produced events",
        "Read models",
        "Returned errors",
        "Workflow transitions",
    ]
    .into_iter()
    .map(|label| {
        DefinitionSectionLabel::try_new(label.to_owned()).map_err(|error| {
            BrowserCompositionError::new(format!("invalid definition section label: {error}"))
        })
    })
    .collect()
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
