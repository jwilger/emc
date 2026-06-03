use nutype::nutype;

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ModelName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ModelDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowSlug(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SliceSlug(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowSliceDetail {
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKindName,
    description: ModelDescription,
}

impl WorkflowSliceDetail {
    pub fn new(
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
    ) -> Self {
        Self {
            slug,
            name,
            kind,
            description,
        }
    }

    pub fn slug(&self) -> &SliceSlug {
        &self.slug
    }

    pub fn name(&self) -> &ModelName {
        &self.name
    }

    pub fn kind(&self) -> &SliceKindName {
        &self.kind
    }

    pub fn description(&self) -> &ModelDescription {
        &self.description
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SliceKindName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct LeanModuleName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct QuintModuleName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ModelDigest(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowTransitionLabel(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowTransitionEndpoint(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowSliceFileReference(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowStepRelationshipName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowTransitionFieldName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct TransitionTriggerName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardLaneId(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowStepName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowBranchLabel(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowTransitionKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowTransitionName(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionRecord {
    source: WorkflowTransitionEndpoint,
    target: WorkflowTransitionEndpoint,
    kind: WorkflowTransitionKind,
    trigger: TransitionTriggerName,
}

impl WorkflowTransitionRecord {
    pub fn new(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
        }
    }

    pub fn source(&self) -> &WorkflowTransitionEndpoint {
        &self.source
    }

    pub fn target(&self) -> &WorkflowTransitionEndpoint {
        &self.target
    }

    pub fn kind(&self) -> &WorkflowTransitionKind {
        &self.kind
    }

    pub fn trigger(&self) -> &TransitionTriggerName {
        &self.trigger
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CommandErrorName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ViewName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BrowserEventElementName(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowBranchDetail {
    name: WorkflowStepName,
    label: WorkflowBranchLabel,
}

impl WorkflowBranchDetail {
    pub fn new(name: WorkflowStepName, label: WorkflowBranchLabel) -> Self {
        Self { name, label }
    }

    pub fn name(&self) -> &WorkflowStepName {
        &self.name
    }

    pub fn label(&self) -> &WorkflowBranchLabel {
        &self.label
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionDetail {
    name: WorkflowTransitionName,
    source: WorkflowStepName,
    target: WorkflowStepName,
    kind: WorkflowTransitionKind,
    label: WorkflowTransitionLabel,
}

impl WorkflowTransitionDetail {
    pub fn new(
        name: WorkflowTransitionName,
        source: WorkflowStepName,
        target: WorkflowStepName,
        kind: WorkflowTransitionKind,
        label: WorkflowTransitionLabel,
    ) -> Self {
        Self {
            name,
            source,
            target,
            kind,
            label,
        }
    }

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
pub struct WorkflowReviewOverlayDetail {
    step: WorkflowStepName,
    status: ReviewStatus,
    missing_rule: ReviewRuleName,
}

impl WorkflowReviewOverlayDetail {
    pub fn new(step: WorkflowStepName, status: ReviewStatus, missing_rule: ReviewRuleName) -> Self {
        Self {
            step,
            status,
            missing_rule,
        }
    }

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
pub struct BrowserErrorRecoveryDetail {
    name: CommandErrorName,
    source_screen: ViewName,
}

impl BrowserErrorRecoveryDetail {
    pub fn new(name: CommandErrorName, source_screen: ViewName) -> Self {
        Self {
            name,
            source_screen,
        }
    }

    pub fn name(&self) -> &CommandErrorName {
        &self.name
    }

    pub fn source_screen(&self) -> &ViewName {
        &self.source_screen
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserCommandDefinitionDetail {
    name: CommandName,
    owning_slice: SliceName,
    source_controls: Vec<SourceControlReference>,
    section_labels: Vec<DefinitionSectionLabel>,
}

impl BrowserCommandDefinitionDetail {
    pub fn new(
        name: CommandName,
        owning_slice: SliceName,
        source_controls: Vec<SourceControlReference>,
        section_labels: Vec<DefinitionSectionLabel>,
    ) -> Self {
        Self {
            name,
            owning_slice,
            source_controls,
            section_labels,
        }
    }

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
pub struct BrowserViewDefinitionDetail {
    name: ViewName,
    field_source_chains: Vec<BrowserFieldSourceChainDetail>,
    control_effects: Vec<BrowserControlEffectDetail>,
}

impl BrowserViewDefinitionDetail {
    pub fn new(
        name: ViewName,
        field_source_chains: Vec<BrowserFieldSourceChainDetail>,
        control_effects: Vec<BrowserControlEffectDetail>,
    ) -> Self {
        Self {
            name,
            field_source_chains,
            control_effects,
        }
    }

    pub fn name(&self) -> &ViewName {
        &self.name
    }

    pub fn field_source_chains(&self) -> &[BrowserFieldSourceChainDetail] {
        &self.field_source_chains
    }

    pub fn control_effects(&self) -> &[BrowserControlEffectDetail] {
        &self.control_effects
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserFieldSourceChainDetail {
    field: ViewFieldName,
    hops: Vec<SourceChainHop>,
}

impl BrowserFieldSourceChainDetail {
    pub fn new(field: ViewFieldName, hops: Vec<SourceChainHop>) -> Self {
        Self { field, hops }
    }

    pub fn field(&self) -> &ViewFieldName {
        &self.field
    }

    pub fn hops(&self) -> &[SourceChainHop] {
        &self.hops
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserControlEffectDetail {
    label: ControlLabel,
    kind: ControlEffectKind,
    target: ControlEffectTarget,
}

impl BrowserControlEffectDetail {
    pub fn new(label: ControlLabel, kind: ControlEffectKind, target: ControlEffectTarget) -> Self {
        Self {
            label,
            kind,
            target,
        }
    }

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

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReviewStatus(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReviewRuleName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReviewerId(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReviewTimestamp(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CommandName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SliceName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SourceControlReference(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DefinitionSectionLabel(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ViewFieldName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SourceChainHop(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlLabel(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlEffectKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlEffectTarget(String);
