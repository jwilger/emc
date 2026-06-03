use nutype::nutype;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    CopyDirectory(ProjectPath, ProjectPath),
    EnsureDirectory(ProjectPath),
    Fail(ReportLine),
    RequireDigest(ProjectPath, ArtifactDigest, ReportLine),
    RequireFile(ProjectPath),
    RequireIndexedWorkflowFiles(ProjectPath, ProjectPath, ReportLine),
    RequireOnlyModeledArtifacts(
        ProjectPath,
        ArtifactFileExtension,
        Vec<ProjectPath>,
        ReportLine,
    ),
    RequireReferencedSliceFiles(ProjectPath, ProjectPath, ReportLine),
    RequireReviewRecord(ProjectPath, ProjectPath, ReportLine),
    RequireWorkflowSliceFiles(ProjectPath, ReportLine),
    RequireWorkflowSlices(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RequireWorkflowTransitions(ProjectPath, ProjectPath, ArtifactDigest, ReportLine),
    RunProcess(ProcessInvocation),
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
