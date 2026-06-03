use nutype::nutype;
use std::path::{Component, Path};

use crate::core::connection::WorkflowConnection;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{ModelDescription, SliceSlug, WorkflowSlug};
use crate::core::workflow::NewWorkflow;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    AddSliceFromWorkflow(NewSlice),
    AddWorkflowFromIndex(NewWorkflow),
    CheckCurrentProject,
    ConnectWorkflowFromWorkflow(WorkflowConnection),
    CopyDirectory(ProjectPath, ProjectPath),
    EnsureDirectory(ProjectPath),
    Fail(ReportLine),
    GenerateSiteFromManifest(ProjectPath),
    ListSlicesFromIndex,
    ListTransitionsFromIndex,
    ListWorkflowsFromIndex,
    RequireCanonicalDeclaration(ProjectPath, ArtifactMarker, ArtifactMarker, ReportLine),
    RequireDigest(ProjectPath, ArtifactDigest, ReportLine),
    RequireFile(ProjectPath),
    RequireIndexedWorkflowFiles(ProjectPath, ProjectPath, ReportLine),
    RequireJsonObjectKeysUnique(ProjectPath, ReportLine),
    RequireOnlyModeledArtifacts(
        ProjectPath,
        ArtifactFileExtension,
        Vec<ProjectPath>,
        ReportLine,
    ),
    RequireOnlyModeledFormalSliceArtifacts(
        ProjectPath,
        ProjectPath,
        ArtifactFileExtension,
        ReportLine,
    ),
    RequireReferencedSliceFileIdentities(ProjectPath, ReportLine),
    RequireReferencedSliceFiles(ProjectPath, ProjectPath, ReportLine),
    RequireReviewRecord(ProjectPath, ProjectPath, ReportLine),
    RequireWorkflowSliceJsonObjects(ProjectPath, ReportLine),
    RequireWorkflowSliceJsonObjectKeysUnique(ProjectPath, ReportLine),
    RequireWorkflowSliceFiles(ProjectPath, ReportLine),
    RequireWorkflowFormalSliceArtifacts(
        ProjectPath,
        ProjectPath,
        ArtifactFileExtension,
        ReportLine,
    ),
    RequireWorkflowSliceDetails(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RequireWorkflowSlices(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RequireWorkflowDigest(ProjectPath, ProjectPath, WorkflowSlug, ReportLine),
    RequireWorkflowTransitions(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RunProcess(ProcessInvocation),
    ShowSliceFromSlice(SliceSlug),
    ShowWorkflowFromWorkflow(WorkflowSlug),
    UpdateSliceDescriptionFromWorkflow(SliceSlug, ModelDescription),
    UpdateSliceKindFromWorkflow(SliceSlug, SliceKind),
    UpdateWorkflowDescriptionFromIndexAndWorkflow(WorkflowSlug, ModelDescription),
    ValidateEventModelTarget(ProjectPath),
    VerifyProjectFromIndex,
    WriteFile(ProjectPath, FileContents),
    WriteFileIfMissing(ProjectPath, FileContents),
    Report(ReportLine),
    ReportDocument(FileContents),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectPlan {
    effects: Vec<Effect>,
}

impl EffectPlan {
    pub fn new(effects: Vec<Effect>) -> Self {
        Self { effects }
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProcessInvocation {
    program: ProgramName,
    arguments: Vec<ProcessArgument>,
    success: ReportLine,
}

impl ProcessInvocation {
    pub fn new(program: ProgramName, arguments: Vec<ProcessArgument>, success: ReportLine) -> Self {
        Self {
            program,
            arguments,
            success,
        }
    }

    pub fn program(&self) -> &ProgramName {
        &self.program
    }

    pub fn arguments(&self) -> &[ProcessArgument] {
        &self.arguments
    }

    pub fn success(&self) -> &ReportLine {
        &self.success
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
