// Copyright 2026 John Wilger

use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Component, Path};

use crate::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::events::EventDraft;
use crate::core::formal_slice_facts::{
    NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
    NewCommandDefinition, NewControlDefinition, NewEventDefinition, NewExternalPayloadDefinition,
    NewOutcomeDefinition, NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition,
};
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    AutomationName, CommandName, ControlName, EventName, ModelDescription, ModelName,
    OutcomeLabelName, ReadModelName, ReviewTimestamp, ReviewerId, ScenarioName, SliceSlug,
    TranslationName, ViewName, WorkflowCommandErrorRecord, WorkflowEntryLifecycleStateRecord,
    WorkflowOutcomeRecord, WorkflowOwnedDefinitionRecord, WorkflowSlug,
    WorkflowTransitionEvidenceRecord,
};
use crate::core::workflow::NewWorkflow;

macro_rules! semantic_artifact_digest {
    ($name:ident) => {
        #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(ArtifactDigest);

        impl $name {
            pub(crate) fn new(digest: ArtifactDigest) -> Self {
                Self(digest)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }
    };
}

semantic_artifact_digest!(ProjectionFingerprint);
semantic_artifact_digest!(ModelContentDigest);
semantic_artifact_digest!(ReviewEventId);
semantic_artifact_digest!(EventConflictId);
semantic_artifact_digest!(ChosenEventId);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceScenarioRemovalEffect {
    slice_slug: SliceSlug,
    scenario_name: ScenarioName,
}

impl SliceScenarioRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, scenario_name: ScenarioName) -> Self {
        Self {
            slice_slug,
            scenario_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn scenario_name(&self) -> &ScenarioName {
        &self.scenario_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceCommandDefinitionRemovalEffect {
    slice_slug: SliceSlug,
    command_name: CommandName,
}

impl SliceCommandDefinitionRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, command_name: CommandName) -> Self {
        Self {
            slice_slug,
            command_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn command_name(&self) -> &CommandName {
        &self.command_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceEventDefinitionRemovalEffect {
    slice_slug: SliceSlug,
    event_name: EventName,
}

impl SliceEventDefinitionRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, event_name: EventName) -> Self {
        Self {
            slice_slug,
            event_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn event_name(&self) -> &EventName {
        &self.event_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceOutcomeDefinitionRemovalEffect {
    slice_slug: SliceSlug,
    outcome_label: OutcomeLabelName,
}

impl SliceOutcomeDefinitionRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, outcome_label: OutcomeLabelName) -> Self {
        Self {
            slice_slug,
            outcome_label,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn outcome_label(&self) -> &OutcomeLabelName {
        &self.outcome_label
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceAutomationDefinitionRemovalEffect {
    slice_slug: SliceSlug,
    automation_name: AutomationName,
}

impl SliceAutomationDefinitionRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, automation_name: AutomationName) -> Self {
        Self {
            slice_slug,
            automation_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn automation_name(&self) -> &AutomationName {
        &self.automation_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceTranslationDefinitionRemovalEffect {
    slice_slug: SliceSlug,
    translation_name: TranslationName,
}

impl SliceTranslationDefinitionRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, translation_name: TranslationName) -> Self {
        Self {
            slice_slug,
            translation_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn translation_name(&self) -> &TranslationName {
        &self.translation_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceReadModelDefinitionRemovalEffect {
    slice_slug: SliceSlug,
    read_model_name: ReadModelName,
}

impl SliceReadModelDefinitionRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, read_model_name: ReadModelName) -> Self {
        Self {
            slice_slug,
            read_model_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn read_model_name(&self) -> &ReadModelName {
        &self.read_model_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceViewDefinitionRemovalEffect {
    slice_slug: SliceSlug,
    view_name: ViewName,
}

impl SliceViewDefinitionRemovalEffect {
    pub(crate) fn new(slice_slug: SliceSlug, view_name: ViewName) -> Self {
        Self {
            slice_slug,
            view_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn view_name(&self) -> &ViewName {
        &self.view_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceViewControlUpdateEffect {
    slice_slug: SliceSlug,
    view_name: ViewName,
    control: NewControlDefinition,
}

impl SliceViewControlUpdateEffect {
    pub(crate) fn new(
        slice_slug: SliceSlug,
        view_name: ViewName,
        control: NewControlDefinition,
    ) -> Self {
        Self {
            slice_slug,
            view_name,
            control,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn view_name(&self) -> &ViewName {
        &self.view_name
    }

    pub(crate) fn control(&self) -> &NewControlDefinition {
        &self.control
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceViewControlRemovalEffect {
    slice_slug: SliceSlug,
    view_name: ViewName,
    control_name: ControlName,
}

impl SliceViewControlRemovalEffect {
    pub(crate) fn new(
        slice_slug: SliceSlug,
        view_name: ViewName,
        control_name: ControlName,
    ) -> Self {
        Self {
            slice_slug,
            view_name,
            control_name,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn view_name(&self) -> &ViewName {
        &self.view_name
    }

    pub(crate) fn control_name(&self) -> &ControlName {
        &self.control_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowCommandErrorEffect {
    workflow_slug: WorkflowSlug,
    error: WorkflowCommandErrorRecord,
}

impl WorkflowCommandErrorEffect {
    pub(crate) fn new(workflow_slug: WorkflowSlug, error: WorkflowCommandErrorRecord) -> Self {
        Self {
            workflow_slug,
            error,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn error(&self) -> &WorkflowCommandErrorRecord {
        &self.error
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowOwnedDefinitionEffect {
    workflow_slug: WorkflowSlug,
    definition: WorkflowOwnedDefinitionRecord,
}

impl WorkflowOwnedDefinitionEffect {
    pub(crate) fn new(
        workflow_slug: WorkflowSlug,
        definition: WorkflowOwnedDefinitionRecord,
    ) -> Self {
        Self {
            workflow_slug,
            definition,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn definition(&self) -> &WorkflowOwnedDefinitionRecord {
        &self.definition
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowOutcomeEffect {
    workflow_slug: WorkflowSlug,
    outcome: WorkflowOutcomeRecord,
}

impl WorkflowOutcomeEffect {
    pub(crate) fn new(workflow_slug: WorkflowSlug, outcome: WorkflowOutcomeRecord) -> Self {
        Self {
            workflow_slug,
            outcome,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn outcome(&self) -> &WorkflowOutcomeRecord {
        &self.outcome
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowTransitionEvidenceEffect {
    workflow_slug: WorkflowSlug,
    evidence: WorkflowTransitionEvidenceRecord,
}

impl WorkflowTransitionEvidenceEffect {
    pub(crate) fn new(
        workflow_slug: WorkflowSlug,
        evidence: WorkflowTransitionEvidenceRecord,
    ) -> Self {
        Self {
            workflow_slug,
            evidence,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn evidence(&self) -> &WorkflowTransitionEvidenceRecord {
        &self.evidence
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowEntryLifecycleStateEffect {
    workflow_slug: WorkflowSlug,
    coverage: WorkflowEntryLifecycleStateRecord,
}

impl WorkflowEntryLifecycleStateEffect {
    pub(crate) fn new(
        workflow_slug: WorkflowSlug,
        coverage: WorkflowEntryLifecycleStateRecord,
    ) -> Self {
        Self {
            workflow_slug,
            coverage,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn coverage(&self) -> &WorkflowEntryLifecycleStateRecord {
        &self.coverage
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CleanReviewEffect {
    workflow_slug: WorkflowSlug,
    reviewer_id: ReviewerId,
    reviewed_at: ReviewTimestamp,
}

impl CleanReviewEffect {
    pub(crate) fn new(
        workflow_slug: WorkflowSlug,
        reviewer_id: ReviewerId,
        reviewed_at: ReviewTimestamp,
    ) -> Self {
        Self {
            workflow_slug,
            reviewer_id,
            reviewed_at,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn reviewer_id(&self) -> &ReviewerId {
        &self.reviewer_id
    }

    pub(crate) fn reviewed_at(&self) -> &ReviewTimestamp {
        &self.reviewed_at
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ReviewEventReference {
    Unrecorded,
    Recorded(ReviewEventId),
}

impl ReviewEventReference {
    pub(crate) fn unrecorded() -> Self {
        Self::Unrecorded
    }

    pub(crate) fn from_optional(event_id: Option<ReviewEventId>) -> Self {
        event_id.map_or(Self::Unrecorded, Self::Recorded)
    }

    pub(crate) fn as_review_event_id(&self) -> Option<&ReviewEventId> {
        match self {
            Self::Unrecorded => None,
            Self::Recorded(event_id) => Some(event_id),
        }
    }
}

impl Serialize for ReviewEventReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Unrecorded => serializer.serialize_none(),
            Self::Recorded(event_id) => event_id.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ReviewEventReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<ReviewEventId>::deserialize(deserializer).map(Self::from_optional)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowReadinessEffect {
    workflow_slug: WorkflowSlug,
    projection_fingerprint: ProjectionFingerprint,
    model_content_digest: ModelContentDigest,
    verified_at: ReviewTimestamp,
    verified_by: ReviewerId,
    review_event: ReviewEventReference,
}

impl WorkflowReadinessEffect {
    pub(crate) fn new(
        workflow_slug: WorkflowSlug,
        projection_fingerprint: ProjectionFingerprint,
        model_content_digest: ModelContentDigest,
        verified_at: ReviewTimestamp,
        verified_by: ReviewerId,
        review_event: ReviewEventReference,
    ) -> Self {
        Self {
            workflow_slug,
            projection_fingerprint,
            model_content_digest,
            verified_at,
            verified_by,
            review_event,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn projection_fingerprint(&self) -> &ProjectionFingerprint {
        &self.projection_fingerprint
    }

    pub(crate) fn model_content_digest(&self) -> &ModelContentDigest {
        &self.model_content_digest
    }

    pub(crate) fn verified_at(&self) -> &ReviewTimestamp {
        &self.verified_at
    }

    pub(crate) fn verified_by(&self) -> &ReviewerId {
        &self.verified_by
    }

    pub(crate) fn review_event(&self) -> &ReviewEventReference {
        &self.review_event
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct EventConflictResolution {
    conflict_id: EventConflictId,
    chosen_event_id: ChosenEventId,
}

impl EventConflictResolution {
    pub(crate) fn new(conflict_id: EventConflictId, chosen_event_id: ChosenEventId) -> Self {
        Self {
            conflict_id,
            chosen_event_id,
        }
    }

    pub(crate) fn conflict_id(&self) -> &EventConflictId {
        &self.conflict_id
    }

    pub(crate) fn chosen_event_id(&self) -> &ChosenEventId {
        &self.chosen_event_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceDescriptionUpdateEffect {
    slice_slug: SliceSlug,
    description: ModelDescription,
}

impl SliceDescriptionUpdateEffect {
    pub(crate) fn new(slice_slug: SliceSlug, description: ModelDescription) -> Self {
        Self {
            slice_slug,
            description,
        }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn description(&self) -> &ModelDescription {
        &self.description
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceKindUpdateEffect {
    slice_slug: SliceSlug,
    kind: SliceKind,
}

impl SliceKindUpdateEffect {
    pub(crate) fn new(slice_slug: SliceSlug, kind: SliceKind) -> Self {
        Self { slice_slug, kind }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn kind(&self) -> SliceKind {
        self.kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SliceNameUpdateEffect {
    slice_slug: SliceSlug,
    name: ModelName,
}

impl SliceNameUpdateEffect {
    pub(crate) fn new(slice_slug: SliceSlug, name: ModelName) -> Self {
        Self { slice_slug, name }
    }

    pub(crate) fn slice_slug(&self) -> &SliceSlug {
        &self.slice_slug
    }

    pub(crate) fn name(&self) -> &ModelName {
        &self.name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowDescriptionUpdateEffect {
    workflow_slug: WorkflowSlug,
    description: ModelDescription,
}

impl WorkflowDescriptionUpdateEffect {
    pub(crate) fn new(workflow_slug: WorkflowSlug, description: ModelDescription) -> Self {
        Self {
            workflow_slug,
            description,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn description(&self) -> &ModelDescription {
        &self.description
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WorkflowNameUpdateEffect {
    workflow_slug: WorkflowSlug,
    name: ModelName,
}

impl WorkflowNameUpdateEffect {
    pub(crate) fn new(workflow_slug: WorkflowSlug, name: ModelName) -> Self {
        Self {
            workflow_slug,
            name,
        }
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn name(&self) -> &ModelName {
        &self.name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CanonicalDeclarationRequirement {
    path: ProjectPath,
    prefix: CanonicalDeclarationPrefix,
    marker: CanonicalDeclarationMarker,
    message: ReportLine,
}

impl CanonicalDeclarationRequirement {
    pub(crate) fn new(
        path: ProjectPath,
        prefix: CanonicalDeclarationPrefix,
        marker: CanonicalDeclarationMarker,
        message: ReportLine,
    ) -> Self {
        Self {
            path,
            prefix,
            marker,
            message,
        }
    }

    pub(crate) fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub(crate) fn prefix(&self) -> &CanonicalDeclarationPrefix {
        &self.prefix
    }

    pub(crate) fn marker(&self) -> &CanonicalDeclarationMarker {
        &self.marker
    }

    pub(crate) fn message(&self) -> &ReportLine {
        &self.message
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ArtifactDigestRequirement {
    path: ProjectPath,
    digest: ArtifactDigest,
    message: ReportLine,
}

impl ArtifactDigestRequirement {
    pub(crate) fn new(path: ProjectPath, digest: ArtifactDigest, message: ReportLine) -> Self {
        Self {
            path,
            digest,
            message,
        }
    }

    pub(crate) fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub(crate) fn digest(&self) -> &ArtifactDigest {
        &self.digest
    }

    pub(crate) fn message(&self) -> &ReportLine {
        &self.message
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FileContentsRequirement {
    path: ProjectPath,
    expected: FileContents,
    message: ReportLine,
}

impl FileContentsRequirement {
    pub(crate) fn new(path: ProjectPath, expected: FileContents, message: ReportLine) -> Self {
        Self {
            path,
            expected,
            message,
        }
    }

    pub(crate) fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub(crate) fn expected(&self) -> &FileContents {
        &self.expected
    }

    pub(crate) fn message(&self) -> &ReportLine {
        &self.message
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ModeledArtifactPaths {
    paths: Vec<ProjectPath>,
}

impl ModeledArtifactPaths {
    pub(crate) fn new(paths: Vec<ProjectPath>) -> Self {
        Self { paths }
    }

    pub(crate) fn as_slice(&self) -> &[ProjectPath] {
        &self.paths
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ModeledArtifactsRequirement {
    path: ProjectPath,
    extension: ArtifactFileExtension,
    allowed_paths: ModeledArtifactPaths,
    message: ReportLine,
}

impl ModeledArtifactsRequirement {
    pub(crate) fn new(
        path: ProjectPath,
        extension: ArtifactFileExtension,
        allowed_paths: ModeledArtifactPaths,
        message: ReportLine,
    ) -> Self {
        Self {
            path,
            extension,
            allowed_paths,
            message,
        }
    }

    pub(crate) fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub(crate) fn extension(&self) -> &ArtifactFileExtension {
        &self.extension
    }

    pub(crate) fn allowed_paths(&self) -> &ModeledArtifactPaths {
        &self.allowed_paths
    }

    pub(crate) fn message(&self) -> &ReportLine {
        &self.message
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ReviewRecordRequirement {
    path: ProjectPath,
    workflow_slug: WorkflowSlug,
    message: ReportLine,
}

impl ReviewRecordRequirement {
    pub(crate) fn new(path: ProjectPath, workflow_slug: WorkflowSlug, message: ReportLine) -> Self {
        Self {
            path,
            workflow_slug,
            message,
        }
    }

    pub(crate) fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub(crate) fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn message(&self) -> &ReportLine {
        &self.message
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FileWriteEffect {
    path: ProjectPath,
    contents: FileContents,
}

impl FileWriteEffect {
    pub(crate) fn new(path: ProjectPath, contents: FileContents) -> Self {
        Self { path, contents }
    }

    pub(crate) fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub(crate) fn contents(&self) -> &FileContents {
        &self.contents
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalSliceArtifactWriteEffect {
    source: ProjectPath,
    target: ProjectPath,
    generated: FileContents,
}

impl FormalSliceArtifactWriteEffect {
    pub(crate) fn new(source: ProjectPath, target: ProjectPath, generated: FileContents) -> Self {
        Self {
            source,
            target,
            generated,
        }
    }

    pub(crate) fn source(&self) -> &ProjectPath {
        &self.source
    }

    pub(crate) fn target(&self) -> &ProjectPath {
        &self.target
    }

    pub(crate) fn generated(&self) -> &FileContents {
        &self.generated
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Effect {
    AddAutomationDefinitionFromSlice(NewAutomationDefinition),
    RemoveAutomationDefinitionFromSlice(SliceAutomationDefinitionRemovalEffect),
    UpdateAutomationDefinitionFromSlice(NewAutomationDefinition),
    AddBitLevelDataFlowFromSlice(NewBitLevelDataFlow),
    AddBoardConnectionFromSlice(NewBoardConnection),
    AddBoardElementFromSlice(NewBoardElement),
    AddCommandDefinitionFromSlice(NewCommandDefinition),
    RemoveCommandDefinitionFromSlice(SliceCommandDefinitionRemovalEffect),
    UpdateCommandDefinitionFromSlice(NewCommandDefinition),
    AddEventDefinitionFromSlice(NewEventDefinition),
    RemoveEventDefinitionFromSlice(SliceEventDefinitionRemovalEffect),
    UpdateEventDefinitionFromSlice(NewEventDefinition),
    AddExternalPayloadDefinitionFromSlice(NewExternalPayloadDefinition),
    AddOutcomeDefinitionFromSlice(NewOutcomeDefinition),
    RemoveOutcomeDefinitionFromSlice(SliceOutcomeDefinitionRemovalEffect),
    UpdateOutcomeDefinitionFromSlice(NewOutcomeDefinition),
    AddReadModelDefinitionFromSlice(NewReadModelDefinition),
    RemoveReadModelDefinitionFromSlice(SliceReadModelDefinitionRemovalEffect),
    UpdateReadModelDefinitionFromSlice(NewReadModelDefinition),
    AddViewDefinitionFromSlice(NewViewDefinition),
    RemoveViewDefinitionFromSlice(SliceViewDefinitionRemovalEffect),
    UpdateViewDefinitionFromSlice(NewViewDefinition),
    RemoveViewControlFromSlice(SliceViewControlRemovalEffect),
    UpdateViewControlFromSlice(SliceViewControlUpdateEffect),
    AddSliceFromWorkflow(NewSlice),
    AddSliceScenarioFromSlice(NewSliceScenario),
    AddTranslationDefinitionFromSlice(NewTranslationDefinition),
    RemoveTranslationDefinitionFromSlice(SliceTranslationDefinitionRemovalEffect),
    UpdateTranslationDefinitionFromSlice(NewTranslationDefinition),
    RemoveSliceScenarioFromSlice(SliceScenarioRemovalEffect),
    UpdateSliceScenarioFromSlice(NewSliceScenario),
    AddWorkflowFromIndex(NewWorkflow),
    AddWorkflowCommandErrorFromWorkflow(WorkflowCommandErrorEffect),
    AddWorkflowOwnedDefinitionFromWorkflow(WorkflowOwnedDefinitionEffect),
    AddWorkflowOutcomeFromWorkflow(WorkflowOutcomeEffect),
    AddWorkflowTransitionEvidenceFromWorkflow(WorkflowTransitionEvidenceEffect),
    AddWorkflowEntryLifecycleStateFromWorkflow(WorkflowEntryLifecycleStateEffect),
    CheckCurrentProject,
    ConnectWorkflowFromWorkflow(WorkflowConnection),
    EnsureDirectory(ProjectPath),
    ExportEvent(EventDraft),
    ListConflictsFromEvents,
    ListSlicesFromIndex,
    ListTransitionsFromIndex,
    ListWorkflowsFromIndex,
    RequireCanonicalDeclaration(CanonicalDeclarationRequirement),
    RequireDigest(ArtifactDigestRequirement),
    RequireFile(ProjectPath),
    RequireFileContentsWithAuthoredFormalFacts(FileContentsRequirement),
    RequireOnlyModeledArtifacts(ModeledArtifactsRequirement),
    RequireReviewRecord(ReviewRecordRequirement),
    RunProcess(ProcessInvocation),
    RunProcessBatch(ProcessInvocations),
    RecordCleanReviewFromWorkflow(CleanReviewEffect),
    DeclareWorkflowReadinessFromWorkflow(WorkflowReadinessEffect),
    RequireWorkflowEntryLifecycleCoverageFromWorkflow(WorkflowSlug),
    RemoveFile(ProjectPath),
    RemoveSliceFromWorkflow(SliceSlug),
    RemoveTransitionFromWorkflow(WorkflowTransitionRemoval),
    RemoveWorkflowFromIndex(WorkflowSlug),
    ResolveEventConflict(EventConflictResolution),
    ShowSliceFromSlice(SliceSlug),
    ShowWorkflowFromWorkflow(WorkflowSlug),
    UpdateSliceDescriptionFromWorkflow(SliceDescriptionUpdateEffect),
    UpdateSliceKindFromWorkflow(SliceKindUpdateEffect),
    UpdateSliceNameFromWorkflow(SliceNameUpdateEffect),
    UpdateWorkflowDescriptionFromIndexAndWorkflow(WorkflowDescriptionUpdateEffect),
    UpdateWorkflowNameFromIndexAndWorkflow(WorkflowNameUpdateEffect),
    VerifyProjectFromIndex,
    WriteFile(FileWriteEffect),
    WriteFormalSliceArtifactPreservingAuthoredFacts(FormalSliceArtifactWriteEffect),
    WriteFileIfMissing(FileWriteEffect),
    Report(ReportLine),
    ReportDocument(FileContents),
}

impl Effect {
    pub(crate) fn require_canonical_declaration(
        path: ProjectPath,
        prefix: CanonicalDeclarationPrefix,
        marker: CanonicalDeclarationMarker,
        message: ReportLine,
    ) -> Self {
        Self::RequireCanonicalDeclaration(CanonicalDeclarationRequirement::new(
            path, prefix, marker, message,
        ))
    }

    pub(crate) fn require_digest(
        path: ProjectPath,
        digest: ArtifactDigest,
        message: ReportLine,
    ) -> Self {
        Self::RequireDigest(ArtifactDigestRequirement::new(path, digest, message))
    }

    pub(crate) fn require_file_contents_with_authored_formal_facts(
        path: ProjectPath,
        expected: FileContents,
        message: ReportLine,
    ) -> Self {
        Self::RequireFileContentsWithAuthoredFormalFacts(FileContentsRequirement::new(
            path, expected, message,
        ))
    }

    pub(crate) fn require_only_modeled_artifacts(
        path: ProjectPath,
        extension: ArtifactFileExtension,
        allowed_paths: Vec<ProjectPath>,
        message: ReportLine,
    ) -> Self {
        Self::RequireOnlyModeledArtifacts(ModeledArtifactsRequirement::new(
            path,
            extension,
            ModeledArtifactPaths::new(allowed_paths),
            message,
        ))
    }

    pub(crate) fn require_review_record(
        path: ProjectPath,
        workflow_slug: WorkflowSlug,
        message: ReportLine,
    ) -> Self {
        Self::RequireReviewRecord(ReviewRecordRequirement::new(path, workflow_slug, message))
    }

    pub(crate) fn write_file(path: ProjectPath, contents: FileContents) -> Self {
        Self::WriteFile(FileWriteEffect::new(path, contents))
    }

    pub(crate) fn write_formal_slice_artifact_preserving_authored_facts(
        source: ProjectPath,
        target: ProjectPath,
        generated: FileContents,
    ) -> Self {
        Self::WriteFormalSliceArtifactPreservingAuthoredFacts(FormalSliceArtifactWriteEffect::new(
            source, target, generated,
        ))
    }

    pub(crate) fn write_file_if_missing(path: ProjectPath, contents: FileContents) -> Self {
        Self::WriteFileIfMissing(FileWriteEffect::new(path, contents))
    }

    pub(crate) fn declare_workflow_readiness(
        workflow_slug: WorkflowSlug,
        projection_fingerprint: ProjectionFingerprint,
        model_content_digest: ModelContentDigest,
        verified_at: ReviewTimestamp,
        verified_by: ReviewerId,
        review_event: ReviewEventReference,
    ) -> Self {
        Self::DeclareWorkflowReadinessFromWorkflow(WorkflowReadinessEffect::new(
            workflow_slug,
            projection_fingerprint,
            model_content_digest,
            verified_at,
            verified_by,
            review_event,
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectPlan {
    effects: Effects,
}

impl EffectPlan {
    pub(crate) fn new(effects: Vec<Effect>) -> Self {
        Self {
            effects: Effects::new(effects),
        }
    }

    pub(crate) fn effects(&self) -> &Effects {
        &self.effects
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Effects {
    effects: Vec<Effect>,
}

impl Effects {
    pub(crate) fn new(effects: Vec<Effect>) -> Self {
        Self { effects }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProcessInvocation {
    program: ProgramName,
    arguments: ProcessArguments,
    success: ReportLine,
}

impl ProcessInvocation {
    pub(crate) fn new(
        program: ProgramName,
        arguments: Vec<ProcessArgument>,
        success: ReportLine,
    ) -> Self {
        Self {
            program,
            arguments: ProcessArguments::new(arguments),
            success,
        }
    }

    pub(crate) fn program(&self) -> &ProgramName {
        &self.program
    }

    pub(crate) fn arguments(&self) -> &ProcessArguments {
        &self.arguments
    }

    pub(crate) fn success(&self) -> &ReportLine {
        &self.success
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProcessInvocations {
    invocations: Vec<ProcessInvocation>,
}

impl ProcessInvocations {
    pub(crate) fn new(invocations: Vec<ProcessInvocation>) -> Self {
        Self { invocations }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ProcessInvocation> {
        self.invocations.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProcessArguments {
    arguments: Vec<ProcessArgument>,
}

impl ProcessArguments {
    pub(crate) fn new(arguments: Vec<ProcessArgument>) -> Self {
        Self { arguments }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ProcessArgument> {
        self.arguments.iter()
    }
}

fn is_project_relative_path(value: &str) -> bool {
    Path::new(value)
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

#[nutype(
    sanitize(trim),
    validate(not_empty, predicate = is_project_relative_path),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub(crate) struct ProjectPath(String);

#[nutype(validate(not_empty), derive(Debug, Clone, Eq, PartialEq, AsRef))]
pub(crate) struct FileContents(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub(crate) struct ReportLine(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Serialize, Deserialize)
)]
pub struct ArtifactDigest(String);

#[nutype(validate(not_empty), derive(Debug, Clone, Eq, PartialEq, AsRef))]
pub(crate) struct CanonicalDeclarationPrefix(String);

#[nutype(validate(not_empty), derive(Debug, Clone, Eq, PartialEq, AsRef))]
pub(crate) struct CanonicalDeclarationMarker(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub(crate) struct ArtifactFileExtension(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub(crate) struct ProgramName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub(crate) struct ProcessArgument(String);
