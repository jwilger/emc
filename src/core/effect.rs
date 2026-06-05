use nutype::nutype;
use std::path::{Component, Path};

use crate::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::formal_slice_facts::{
    NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
    NewCommandDefinition, NewEventDefinition, NewExternalPayloadDefinition, NewOutcomeDefinition,
    NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition, NewViewDefinition,
};
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    ModelDescription, ModelName, ReviewTimestamp, ReviewerId, SliceSlug,
    WorkflowCommandErrorRecord, WorkflowEntryLifecycleStateRecord, WorkflowOutcomeRecord,
    WorkflowOwnedDefinitionRecord, WorkflowSlug, WorkflowTransitionEvidenceRecord,
};
use crate::core::workflow::NewWorkflow;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    AddAutomationDefinitionFromSlice(NewAutomationDefinition),
    AddBitLevelDataFlowFromSlice(NewBitLevelDataFlow),
    AddBoardConnectionFromSlice(NewBoardConnection),
    AddBoardElementFromSlice(NewBoardElement),
    AddCommandDefinitionFromSlice(NewCommandDefinition),
    AddEventDefinitionFromSlice(NewEventDefinition),
    AddExternalPayloadDefinitionFromSlice(NewExternalPayloadDefinition),
    AddOutcomeDefinitionFromSlice(NewOutcomeDefinition),
    AddReadModelDefinitionFromSlice(NewReadModelDefinition),
    AddViewDefinitionFromSlice(NewViewDefinition),
    AddSliceFromWorkflow(NewSlice),
    AddSliceScenarioFromSlice(NewSliceScenario),
    AddTranslationDefinitionFromSlice(NewTranslationDefinition),
    AddWorkflowFromIndex(NewWorkflow),
    AddWorkflowCommandErrorFromWorkflow(WorkflowSlug, WorkflowCommandErrorRecord),
    AddWorkflowOwnedDefinitionFromWorkflow(WorkflowSlug, WorkflowOwnedDefinitionRecord),
    AddWorkflowOutcomeFromWorkflow(WorkflowSlug, WorkflowOutcomeRecord),
    AddWorkflowTransitionEvidenceFromWorkflow(WorkflowSlug, WorkflowTransitionEvidenceRecord),
    AddWorkflowEntryLifecycleStateFromWorkflow(WorkflowSlug, WorkflowEntryLifecycleStateRecord),
    CheckCurrentProject,
    ConnectWorkflowFromWorkflow(WorkflowConnection),
    CopyDirectory(ProjectPath, ProjectPath),
    EnsureDirectory(ProjectPath),
    Fail(ReportLine),
    ListSlicesFromIndex,
    ListTransitionsFromIndex,
    ListWorkflowsFromIndex,
    RequireCanonicalDeclaration(ProjectPath, ArtifactMarker, ArtifactMarker, ReportLine),
    RequireDigest(ProjectPath, ArtifactDigest, ReportLine),
    RequireFile(ProjectPath),
    RequireFileContents(ProjectPath, FileContents, ReportLine),
    RequireFileContentsWithAuthoredFormalFacts(ProjectPath, FileContents, ReportLine),
    RequireJsonObjectKeysUnique(ProjectPath, ReportLine),
    RequireOnlyModeledArtifacts(
        ProjectPath,
        ArtifactFileExtension,
        Vec<ProjectPath>,
        ReportLine,
    ),
    RequireReviewRecord(ProjectPath, WorkflowSlug, ReportLine),
    RunProcess(ProcessInvocation),
    RecordCleanReviewFromWorkflow(WorkflowSlug, ReviewerId, ReviewTimestamp),
    RequireWorkflowEntryLifecycleCoverageFromWorkflow(WorkflowSlug),
    RemoveDirectory(ProjectPath),
    RemoveFile(ProjectPath),
    RemoveSliceFromWorkflow(SliceSlug),
    RemoveTransitionFromWorkflow(WorkflowTransitionRemoval),
    RemoveWorkflowFromIndex(WorkflowSlug),
    ShowSliceFromSlice(SliceSlug),
    ShowWorkflowFromWorkflow(WorkflowSlug),
    UpdateSliceDescriptionFromWorkflow(SliceSlug, ModelDescription),
    UpdateSliceKindFromWorkflow(SliceSlug, SliceKind),
    UpdateSliceNameFromWorkflow(SliceSlug, ModelName),
    UpdateWorkflowDescriptionFromIndexAndWorkflow(WorkflowSlug, ModelDescription),
    UpdateWorkflowNameFromIndexAndWorkflow(WorkflowSlug, ModelName),
    VerifyProjectFromIndex,
    WriteFile(ProjectPath, FileContents),
    WriteFormalSliceArtifactPreservingAuthoredFacts(ProjectPath, ProjectPath, FileContents),
    WriteFileIfMissing(ProjectPath, FileContents),
    Report(ReportLine),
    ReportDocument(FileContents),
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

    pub fn effects(&self) -> &Effects {
        &self.effects
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Effects {
    effects: Vec<Effect>,
}

impl Effects {
    pub(crate) fn new(effects: Vec<Effect>) -> Self {
        Self { effects }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProcessInvocation {
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

    pub fn program(&self) -> &ProgramName {
        &self.program
    }

    pub fn arguments(&self) -> &ProcessArguments {
        &self.arguments
    }

    pub fn success(&self) -> &ReportLine {
        &self.success
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProcessArguments {
    arguments: Vec<ProcessArgument>,
}

impl ProcessArguments {
    pub(crate) fn new(arguments: Vec<ProcessArgument>) -> Self {
        Self { arguments }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProcessArgument> {
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
pub struct ProjectPath(String);

#[nutype(validate(not_empty), derive(Debug, Clone, Eq, PartialEq, AsRef))]
pub struct FileContents(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub struct ReportLine(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub struct ArtifactDigest(String);

#[nutype(validate(not_empty), derive(Debug, Clone, Eq, PartialEq, AsRef))]
pub struct ArtifactMarker(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub struct ArtifactFileExtension(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub struct ProgramName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef)
)]
pub struct ProcessArgument(String);
