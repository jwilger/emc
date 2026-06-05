// Copyright 2026 John Wilger

use nutype::nutype;

use crate::core::effect::ArtifactDigest;

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
    relationship: WorkflowStepRelationshipName,
}

impl WorkflowSliceDetail {
    pub fn new(
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
    ) -> Self {
        Self::new_with_relationship(
            slug,
            name,
            kind,
            description,
            workflow_step_relationship_name("main"),
        )
    }

    pub fn new_with_relationship(
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
        relationship: WorkflowStepRelationshipName,
    ) -> Self {
        Self {
            slug,
            name,
            kind,
            description,
            relationship,
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

    pub fn relationship(&self) -> &WorkflowStepRelationshipName {
        &self.relationship
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowSliceDetails {
    details: Vec<WorkflowSliceDetail>,
}

impl WorkflowSliceDetails {
    pub fn from_details(details: impl IntoIterator<Item = WorkflowSliceDetail>) -> Self {
        Self {
            details: details.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowSliceDetail] {
        &self.details
    }

    pub(crate) fn into_inner(self) -> Vec<WorkflowSliceDetail> {
        self.details
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
pub struct WorkflowTransitionEndpoint(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowStepRelationshipName(String);

fn workflow_step_relationship_name(value: &str) -> WorkflowStepRelationshipName {
    WorkflowStepRelationshipName::try_new(value.to_owned()).unwrap_or_else(|error| {
        unreachable!("EMC generated workflow step relationship must be valid: {error}");
    })
}

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
pub struct PayloadContractName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct TranslationName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct TranslationExternalEventName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct OutcomeLabelName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ScenarioName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ScenarioStepText(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ContractKindName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CoveredDefinitionName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DatumName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DataFlowSource(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct TransformationSemantics(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReadModelDerivationRule(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReadModelTransitiveRule(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DataFlowTarget(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BitEncodingSemantics(String);

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
pub struct BoardElementName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardElementKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardElementDeclaredName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardConnectionEndpoint(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardConnectionEndpointKind(String);

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
pub struct WorkflowTransitionEvidenceText(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowEntryLifecycleStateName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowEntryLifecycleEvidenceText(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionRecord {
    source: WorkflowTransitionEndpoint,
    target: WorkflowTransitionEndpoint,
    kind: WorkflowTransitionKind,
    trigger: TransitionTriggerName,
    rationale: Option<ModelDescription>,
    payload_contract: Option<PayloadContractName>,
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
            rationale: None,
            payload_contract: None,
        }
    }

    pub fn new_with_rationale(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
        rationale: ModelDescription,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
            rationale: Some(rationale),
            payload_contract: None,
        }
    }

    pub fn new_with_payload_contract(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
        payload_contract: PayloadContractName,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
            rationale: None,
            payload_contract: Some(payload_contract),
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

    pub fn rationale(&self) -> Option<&ModelDescription> {
        self.rationale.as_ref()
    }

    pub fn payload_contract(&self) -> Option<&PayloadContractName> {
        self.payload_contract.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionRecords {
    records: Vec<WorkflowTransitionRecord>,
}

impl WorkflowTransitionRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowTransitionRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowTransitionRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOutcomeRecord {
    source_slice: WorkflowTransitionEndpoint,
    label: OutcomeLabelName,
    externally_relevant: bool,
}

impl WorkflowOutcomeRecord {
    pub fn new(
        source_slice: WorkflowTransitionEndpoint,
        label: OutcomeLabelName,
        externally_relevant: bool,
    ) -> Self {
        Self {
            source_slice,
            label,
            externally_relevant,
        }
    }

    pub fn source_slice(&self) -> &WorkflowTransitionEndpoint {
        &self.source_slice
    }

    pub fn label(&self) -> &OutcomeLabelName {
        &self.label
    }

    pub fn externally_relevant(&self) -> bool {
        self.externally_relevant
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOutcomeRecords {
    records: Vec<WorkflowOutcomeRecord>,
}

impl WorkflowOutcomeRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowOutcomeRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowOutcomeRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowCommandErrorRecord {
    source_slice: WorkflowTransitionEndpoint,
    command_name: CommandName,
    error_name: CommandErrorName,
}

impl WorkflowCommandErrorRecord {
    pub fn new(
        source_slice: WorkflowTransitionEndpoint,
        command_name: CommandName,
        error_name: CommandErrorName,
    ) -> Self {
        Self {
            source_slice,
            command_name,
            error_name,
        }
    }

    pub fn source_slice(&self) -> &WorkflowTransitionEndpoint {
        &self.source_slice
    }

    pub fn command_name(&self) -> &CommandName {
        &self.command_name
    }

    pub fn error_name(&self) -> &CommandErrorName {
        &self.error_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowCommandErrorRecords {
    records: Vec<WorkflowCommandErrorRecord>,
}

impl WorkflowCommandErrorRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowCommandErrorRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowCommandErrorRecord] {
        &self.records
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowOwnedDefinitionKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowOwnedDefinitionName(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOwnedDefinitionRecord {
    source_slice: WorkflowTransitionEndpoint,
    definition_kind: WorkflowOwnedDefinitionKind,
    definition_name: WorkflowOwnedDefinitionName,
    definition_stream: Option<StreamName>,
    source_provenance: Option<ModelDescription>,
}

impl WorkflowOwnedDefinitionRecord {
    pub fn new(
        source_slice: WorkflowTransitionEndpoint,
        definition_kind: WorkflowOwnedDefinitionKind,
        definition_name: WorkflowOwnedDefinitionName,
    ) -> Self {
        Self {
            source_slice,
            definition_kind,
            definition_name,
            definition_stream: None,
            source_provenance: None,
        }
    }

    pub fn new_with_event_identity(
        source_slice: WorkflowTransitionEndpoint,
        definition_kind: WorkflowOwnedDefinitionKind,
        definition_name: WorkflowOwnedDefinitionName,
        definition_stream: StreamName,
        source_provenance: ModelDescription,
    ) -> Self {
        Self {
            source_slice,
            definition_kind,
            definition_name,
            definition_stream: Some(definition_stream),
            source_provenance: Some(source_provenance),
        }
    }

    pub fn source_slice(&self) -> &WorkflowTransitionEndpoint {
        &self.source_slice
    }

    pub fn definition_kind(&self) -> &WorkflowOwnedDefinitionKind {
        &self.definition_kind
    }

    pub fn definition_name(&self) -> &WorkflowOwnedDefinitionName {
        &self.definition_name
    }

    pub fn definition_stream(&self) -> Option<&StreamName> {
        self.definition_stream.as_ref()
    }

    pub fn source_provenance(&self) -> Option<&ModelDescription> {
        self.source_provenance.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOwnedDefinitionRecords {
    records: Vec<WorkflowOwnedDefinitionRecord>,
}

impl WorkflowOwnedDefinitionRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowOwnedDefinitionRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowOwnedDefinitionRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionEvidenceRecord {
    source: WorkflowTransitionEndpoint,
    target: WorkflowTransitionEndpoint,
    kind: WorkflowTransitionKind,
    trigger: TransitionTriggerName,
    source_evidence: WorkflowTransitionEvidenceText,
    target_evidence: WorkflowTransitionEvidenceText,
}

impl WorkflowTransitionEvidenceRecord {
    pub fn new(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
        source_evidence: WorkflowTransitionEvidenceText,
        target_evidence: WorkflowTransitionEvidenceText,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
            source_evidence,
            target_evidence,
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

    pub fn source_evidence(&self) -> &WorkflowTransitionEvidenceText {
        &self.source_evidence
    }

    pub fn target_evidence(&self) -> &WorkflowTransitionEvidenceText {
        &self.target_evidence
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionEvidenceRecords {
    records: Vec<WorkflowTransitionEvidenceRecord>,
}

impl WorkflowTransitionEvidenceRecords {
    pub fn from_records(
        records: impl IntoIterator<Item = WorkflowTransitionEvidenceRecord>,
    ) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowTransitionEvidenceRecord] {
        &self.records
    }
}

impl Default for WorkflowTransitionEvidenceRecords {
    fn default() -> Self {
        Self::from_records([])
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowEntryLifecycleStateRecord {
    state: WorkflowEntryLifecycleStateName,
    step: WorkflowTransitionEndpoint,
    evidence: WorkflowEntryLifecycleEvidenceText,
}

impl WorkflowEntryLifecycleStateRecord {
    pub fn new(
        state: WorkflowEntryLifecycleStateName,
        step: WorkflowTransitionEndpoint,
        evidence: WorkflowEntryLifecycleEvidenceText,
    ) -> Self {
        Self {
            state,
            step,
            evidence,
        }
    }

    pub fn state(&self) -> &WorkflowEntryLifecycleStateName {
        &self.state
    }

    pub fn step(&self) -> &WorkflowTransitionEndpoint {
        &self.step
    }

    pub fn evidence(&self) -> &WorkflowEntryLifecycleEvidenceText {
        &self.evidence
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowEntryLifecycleStateRecords {
    records: Vec<WorkflowEntryLifecycleStateRecord>,
}

impl WorkflowEntryLifecycleStateRecords {
    pub fn from_records(
        records: impl IntoIterator<Item = WorkflowEntryLifecycleStateRecord>,
    ) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowEntryLifecycleStateRecord] {
        &self.records
    }
}

impl Default for WorkflowEntryLifecycleStateRecords {
    fn default() -> Self {
        Self::from_records([])
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowModuleData {
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_slice_details: WorkflowSliceDetails,
    workflow_transitions: WorkflowTransitionRecords,
    workflow_outcomes: WorkflowOutcomeRecords,
    workflow_command_errors: WorkflowCommandErrorRecords,
    workflow_owned_definitions: WorkflowOwnedDefinitionRecords,
    workflow_transition_evidences: WorkflowTransitionEvidenceRecords,
    workflow_requires_entry_lifecycle_coverage: bool,
    workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
    digest: ArtifactDigest,
}

impl WorkflowModuleData {
    pub fn new(
        workflow_name: ModelName,
        workflow_description: ModelDescription,
        workflow_slug: WorkflowSlug,
        digest: ArtifactDigest,
    ) -> Self {
        Self {
            workflow_name,
            workflow_description,
            workflow_slug,
            workflow_slice_details: WorkflowSliceDetails::from_details([]),
            workflow_transitions: WorkflowTransitionRecords::from_records([]),
            workflow_outcomes: WorkflowOutcomeRecords::from_records([]),
            workflow_command_errors: WorkflowCommandErrorRecords::from_records([]),
            workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records([]),
            workflow_transition_evidences: WorkflowTransitionEvidenceRecords::from_records([]),
            workflow_requires_entry_lifecycle_coverage: false,
            workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords::from_records([]),
            digest,
        }
    }

    pub fn with_slice_details(mut self, workflow_slice_details: WorkflowSliceDetails) -> Self {
        self.workflow_slice_details = workflow_slice_details;
        self
    }

    pub fn with_transitions(mut self, workflow_transitions: WorkflowTransitionRecords) -> Self {
        self.workflow_transitions = workflow_transitions;
        self
    }

    pub fn with_outcomes(mut self, workflow_outcomes: WorkflowOutcomeRecords) -> Self {
        self.workflow_outcomes = workflow_outcomes;
        self
    }

    pub fn with_command_errors(
        mut self,
        workflow_command_errors: WorkflowCommandErrorRecords,
    ) -> Self {
        self.workflow_command_errors = workflow_command_errors;
        self
    }

    pub fn with_owned_definitions(
        mut self,
        workflow_owned_definitions: WorkflowOwnedDefinitionRecords,
    ) -> Self {
        self.workflow_owned_definitions = workflow_owned_definitions;
        self
    }

    pub fn with_transition_evidences(
        mut self,
        workflow_transition_evidences: WorkflowTransitionEvidenceRecords,
    ) -> Self {
        self.workflow_transition_evidences = workflow_transition_evidences;
        self
    }

    pub fn with_entry_lifecycle_required(
        mut self,
        workflow_requires_entry_lifecycle_coverage: bool,
    ) -> Self {
        self.workflow_requires_entry_lifecycle_coverage =
            workflow_requires_entry_lifecycle_coverage;
        self
    }

    pub fn with_entry_lifecycle_states(
        mut self,
        workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
    ) -> Self {
        self.workflow_entry_lifecycle_states = workflow_entry_lifecycle_states;
        self
    }

    pub fn workflow_name(&self) -> &ModelName {
        &self.workflow_name
    }

    pub fn workflow_description(&self) -> &ModelDescription {
        &self.workflow_description
    }

    pub fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn workflow_slice_details(&self) -> &WorkflowSliceDetails {
        &self.workflow_slice_details
    }

    pub(crate) fn workflow_transitions(&self) -> &WorkflowTransitionRecords {
        &self.workflow_transitions
    }

    pub(crate) fn workflow_outcomes(&self) -> &WorkflowOutcomeRecords {
        &self.workflow_outcomes
    }

    pub(crate) fn workflow_command_errors(&self) -> &WorkflowCommandErrorRecords {
        &self.workflow_command_errors
    }

    pub(crate) fn workflow_owned_definitions(&self) -> &WorkflowOwnedDefinitionRecords {
        &self.workflow_owned_definitions
    }

    pub(crate) fn workflow_transition_evidences(&self) -> &WorkflowTransitionEvidenceRecords {
        &self.workflow_transition_evidences
    }

    pub(crate) fn workflow_requires_entry_lifecycle_coverage(&self) -> bool {
        self.workflow_requires_entry_lifecycle_coverage
    }

    pub(crate) fn workflow_entry_lifecycle_states(&self) -> &WorkflowEntryLifecycleStateRecords {
        &self.workflow_entry_lifecycle_states
    }

    pub fn digest(&self) -> &ArtifactDigest {
        &self.digest
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
pub struct CommandErrorName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CommandErrorRecoveryKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SingletonRepeatBehavior(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct AutomationName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct AutomationTriggerName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct AutomationReactionDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlRecoveryBehavior(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct NavigationTargetType(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct NavigationTargetName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CommandInputSourceKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CommandInputSourceDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct StreamName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventAttributeName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventAttributeSourceKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventAttributeSourceName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventAttributeSourceField(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ProvenanceDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReadModelName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReadModelFieldSourceKind(String);

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
pub struct ViewFieldSourceKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SketchToken(String);

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
