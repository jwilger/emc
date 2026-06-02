use nutype::nutype;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    EnsureDirectory(ProjectPath),
    RequireFile(ProjectPath),
    WriteFile(ProjectPath, FileContents),
    WriteFileIfMissing(ProjectPath, FileContents),
    Report(ReportLine),
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
