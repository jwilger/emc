use nutype::nutype;

use crate::core::connection::WorkflowConnection;
use crate::core::slice::NewSlice;
use crate::core::types::{ModelDescription, WorkflowSlug};
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
    RequireReferencedSliceFiles(ProjectPath, ProjectPath, ReportLine),
    RequireReviewRecord(ProjectPath, ProjectPath, ReportLine),
    RequireWorkflowSliceJsonObjectKeysUnique(ProjectPath, ReportLine),
    RequireWorkflowSliceFiles(ProjectPath, ReportLine),
    RequireWorkflowSliceDetails(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RequireWorkflowSlices(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RequireWorkflowTransitions(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RunProcess(ProcessInvocation),
    ShowWorkflowFromWorkflow(WorkflowSlug),
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

#[nutype(
    sanitize(trim),
    validate(not_empty),
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
