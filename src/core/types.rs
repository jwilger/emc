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
    pub(crate) fn new(
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

    pub(crate) fn source_controls(&self) -> &[SourceControlReference] {
        &self.source_controls
    }

    pub(crate) fn section_labels(&self) -> &[DefinitionSectionLabel] {
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
    pub(crate) fn new(
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

    pub(crate) fn field_source_chains(&self) -> &[BrowserFieldSourceChainDetail] {
        &self.field_source_chains
    }

    pub(crate) fn control_effects(&self) -> &[BrowserControlEffectDetail] {
        &self.control_effects
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserFieldSourceChainDetail {
    field: ViewFieldName,
    hops: Vec<SourceChainHop>,
}

impl BrowserFieldSourceChainDetail {
    pub(crate) fn new(field: ViewFieldName, hops: Vec<SourceChainHop>) -> Self {
        Self { field, hops }
    }

    pub fn field(&self) -> &ViewFieldName {
        &self.field
    }

    pub(crate) fn hops(&self) -> &[SourceChainHop] {
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

fn is_utc_millisecond_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 24
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'.'
        && bytes[23] == b'Z'
        && ascii_digits(bytes, 0, 4)
        && ascii_digits(bytes, 5, 2)
        && ascii_digits(bytes, 8, 2)
        && ascii_digits(bytes, 11, 2)
        && ascii_digits(bytes, 14, 2)
        && ascii_digits(bytes, 17, 2)
        && ascii_digits(bytes, 20, 3)
        && numeric_range(bytes, 5, 2, 1, 12)
        && numeric_range(bytes, 8, 2, 1, 31)
        && numeric_range(bytes, 11, 2, 0, 23)
        && numeric_range(bytes, 14, 2, 0, 59)
        && numeric_range(bytes, 17, 2, 0, 59)
}

fn ascii_digits(bytes: &[u8], start: usize, length: usize) -> bool {
    bytes[start..start + length].iter().all(u8::is_ascii_digit)
}

fn numeric_range(bytes: &[u8], start: usize, length: usize, minimum: u32, maximum: u32) -> bool {
    ascii_digits(bytes, start, length)
        && (minimum..=maximum).contains(
            &bytes[start..start + length]
                .iter()
                .fold(0_u32, |value, byte| (value * 10) + u32::from(byte - b'0')),
        )
}

#[nutype(
    sanitize(trim),
    validate(not_empty, predicate = is_utc_millisecond_timestamp),
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
