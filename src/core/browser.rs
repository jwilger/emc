use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::iter;

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
    lane_ids: BrowserLaneIds,
    main_path_names: BrowserMainPathNames,
    branch_cards: BrowserBranchCards,
    transition_cards: BrowserTransitionCards,
    error_recovery_cards: BrowserErrorRecoveryCards,
    event_element_names: BrowserEventElementNames,
    review_overlays: BrowserReviewOverlays,
    command_definitions: BrowserCommandDefinitions,
    view_definitions: BrowserViewDefinitions,
}

impl BrowserWorkflow {
    pub fn lane_ids(&self) -> &BrowserLaneIds {
        &self.lane_ids
    }

    pub fn main_path_names(&self) -> &BrowserMainPathNames {
        &self.main_path_names
    }

    pub fn branch_cards(&self) -> &BrowserBranchCards {
        &self.branch_cards
    }

    pub fn transition_cards(&self) -> &BrowserTransitionCards {
        &self.transition_cards
    }

    pub fn error_recovery_cards(&self) -> &BrowserErrorRecoveryCards {
        &self.error_recovery_cards
    }

    pub fn event_element_names(&self) -> &BrowserEventElementNames {
        &self.event_element_names
    }

    pub fn review_overlays(&self) -> &BrowserReviewOverlays {
        &self.review_overlays
    }

    pub fn command_definitions(&self) -> &BrowserCommandDefinitions {
        &self.command_definitions
    }

    pub fn view_definitions(&self) -> &BrowserViewDefinitions {
        &self.view_definitions
    }
}

macro_rules! browser_collection {
    ($name:ident, $item:ty) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct $name {
            items: Vec<$item>,
        }

        impl $name {
            pub(crate) fn new(items: Vec<$item>) -> Self {
                Self { items }
            }

            pub fn iter(&self) -> impl Iterator<Item = &$item> {
                self.items.iter()
            }
        }
    };
}

browser_collection!(BrowserLaneIds, BoardLaneId);
browser_collection!(BrowserMainPathNames, WorkflowStepName);
browser_collection!(BrowserBranchCards, BrowserBranchCard);
browser_collection!(BrowserTransitionCards, BrowserTransitionCard);
browser_collection!(BrowserErrorRecoveryCards, BrowserErrorRecoveryCard);
browser_collection!(BrowserEventElementNames, BrowserEventElementName);
browser_collection!(BrowserReviewOverlays, BrowserReviewOverlay);
browser_collection!(BrowserCommandDefinitions, BrowserCommandDefinition);
browser_collection!(BrowserViewDefinitions, BrowserViewDefinition);
browser_collection!(SourceControlReferences, SourceControlReference);
browser_collection!(DefinitionSectionLabels, DefinitionSectionLabel);
browser_collection!(BrowserFieldSourceChains, BrowserFieldSourceChain);
browser_collection!(BrowserControlEffects, BrowserControlEffect);
browser_collection!(SourceChainHops, SourceChainHop);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserSliceDocuments {
    documents: Vec<FileContents>,
}

impl BrowserSliceDocuments {
    pub fn empty() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    pub fn from_documents(documents: impl IntoIterator<Item = FileContents>) -> Self {
        Self {
            documents: documents.into_iter().collect(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = &FileContents> {
        self.documents.iter()
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
    source_controls: SourceControlReferences,
    section_labels: DefinitionSectionLabels,
}

impl BrowserCommandDefinition {
    pub fn name(&self) -> &CommandName {
        &self.name
    }

    pub fn owning_slice(&self) -> &SliceName {
        &self.owning_slice
    }

    pub fn source_controls(&self) -> &SourceControlReferences {
        &self.source_controls
    }

    pub fn section_labels(&self) -> &DefinitionSectionLabels {
        &self.section_labels
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserViewDefinition {
    name: ViewName,
    field_source_chains: BrowserFieldSourceChains,
    control_effects: BrowserControlEffects,
}

impl BrowserViewDefinition {
    pub fn name(&self) -> &ViewName {
        &self.name
    }

    pub fn field_source_chains(&self) -> &BrowserFieldSourceChains {
        &self.field_source_chains
    }

    pub fn control_effects(&self) -> &BrowserControlEffects {
        &self.control_effects
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserFieldSourceChain {
    field: ViewFieldName,
    hops: SourceChainHops,
}

impl BrowserFieldSourceChain {
    pub fn field(&self) -> &ViewFieldName {
        &self.field
    }

    pub fn hops(&self) -> &SourceChainHops {
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
    slice_documents: BrowserSliceDocuments,
) -> Result<BrowserWorkflow, BrowserCompositionError> {
    let workflow_semantics = WorkflowDocument::parse(&workflow_document)
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
    let workflow_browser_data = BrowserDataDocument::parse(&workflow_document)
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?;
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
            source_controls: SourceControlReferences::new(detail.source_controls().to_vec()),
            section_labels: DefinitionSectionLabels::new(detail.section_labels().to_vec()),
        })
        .collect();
    let view_definitions = browser_data
        .view_definition_details()
        .map_err(|error| BrowserCompositionError::new(error.to_string()))?
        .into_iter()
        .map(|detail| BrowserViewDefinition {
            name: detail.name().clone(),
            field_source_chains: BrowserFieldSourceChains::new(
                detail
                    .field_source_chains()
                    .iter()
                    .map(|source_chain| BrowserFieldSourceChain {
                        field: source_chain.field().clone(),
                        hops: SourceChainHops::new(source_chain.hops().to_vec()),
                    })
                    .collect(),
            ),
            control_effects: BrowserControlEffects::new(
                detail
                    .control_effects()
                    .iter()
                    .map(|effect| BrowserControlEffect {
                        label: effect.label().clone(),
                        kind: effect.kind().clone(),
                        target: effect.target().clone(),
                    })
                    .collect(),
            ),
        })
        .collect();

    Ok(BrowserWorkflow {
        lane_ids: BrowserLaneIds::new(lane_ids),
        main_path_names: BrowserMainPathNames::new(main_path_names),
        branch_cards: BrowserBranchCards::new(branch_cards),
        transition_cards: BrowserTransitionCards::new(transition_cards),
        error_recovery_cards: BrowserErrorRecoveryCards::new(error_recovery_cards),
        event_element_names: BrowserEventElementNames::new(event_element_names),
        review_overlays: BrowserReviewOverlays::new(review_overlays),
        command_definitions: BrowserCommandDefinitions::new(command_definitions),
        view_definitions: BrowserViewDefinitions::new(view_definitions),
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
