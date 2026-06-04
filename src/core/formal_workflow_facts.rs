use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::digest::{WorkflowArtifactDigestInput, artifact_digest};
use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::formal_graph::{parse_lean_workflow_graph, parse_quint_workflow_graph};
use crate::core::types::{
    WorkflowCommandErrorRecord, WorkflowOutcomeRecord, WorkflowOwnedDefinitionRecord, WorkflowSlug,
    WorkflowTransitionEvidenceRecord,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormalWorkflowFactError {
    message: String,
}

impl FormalWorkflowFactError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for FormalWorkflowFactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for FormalWorkflowFactError {}

pub fn add_workflow_outcome(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    workflow_slug: WorkflowSlug,
    outcome: WorkflowOutcomeRecord,
) -> Result<EffectPlan, FormalWorkflowFactError> {
    let lean_record = lean_workflow_outcome_record(&outcome);
    let quint_record = quint_workflow_outcome_record(&outcome);
    let lean = refresh_lean_digest(append_record(
        lean_contents.as_ref(),
        "def workflowOutcomes : List WorkflowOutcome := ",
        &lean_record,
    )?)?;
    let quint = refresh_quint_digest(append_record(
        quint_contents.as_ref(),
        "val workflowOutcomes: List[WorkflowOutcome] = ",
        &quint_record,
    )?)?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added workflow outcome {} to workflow {}",
            outcome.label().as_ref(),
            workflow_slug.as_ref()
        ))?),
    ]))
}

pub fn add_workflow_command_error(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    workflow_slug: WorkflowSlug,
    error: WorkflowCommandErrorRecord,
) -> Result<EffectPlan, FormalWorkflowFactError> {
    let lean_record = lean_workflow_command_error_record(&error);
    let quint_record = quint_workflow_command_error_record(&error);
    let lean = refresh_lean_digest(append_record(
        lean_contents.as_ref(),
        "def workflowCommandErrors : List WorkflowCommandError := ",
        &lean_record,
    )?)?;
    let quint = refresh_quint_digest(append_record(
        quint_contents.as_ref(),
        "val workflowCommandErrors: List[WorkflowCommandError] = ",
        &quint_record,
    )?)?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added workflow command error {} to workflow {}",
            error.error_name().as_ref(),
            workflow_slug.as_ref()
        ))?),
    ]))
}

pub fn add_workflow_owned_definition(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    workflow_slug: WorkflowSlug,
    definition: WorkflowOwnedDefinitionRecord,
) -> Result<EffectPlan, FormalWorkflowFactError> {
    let lean_record = lean_workflow_owned_definition_record(&definition);
    let quint_record = quint_workflow_owned_definition_record(&definition);
    let lean = refresh_lean_digest(append_record(
        lean_contents.as_ref(),
        "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := ",
        &lean_record,
    )?)?;
    let quint = refresh_quint_digest(append_record(
        quint_contents.as_ref(),
        "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = ",
        &quint_record,
    )?)?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added workflow owned definition {} {} to workflow {}",
            definition.definition_kind().as_ref(),
            definition.definition_name().as_ref(),
            workflow_slug.as_ref()
        ))?),
    ]))
}

pub fn add_workflow_transition_evidence(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    workflow_slug: WorkflowSlug,
    evidence: WorkflowTransitionEvidenceRecord,
) -> Result<EffectPlan, FormalWorkflowFactError> {
    let lean_record = lean_workflow_transition_evidence_record(&evidence);
    let quint_record = quint_workflow_transition_evidence_record(&evidence);
    let lean = refresh_lean_digest(append_record(
        lean_contents.as_ref(),
        "def workflowTransitionEvidences : List WorkflowTransitionEvidence := ",
        &lean_record,
    )?)?;
    let quint = refresh_quint_digest(append_record(
        quint_contents.as_ref(),
        "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = ",
        &quint_record,
    )?)?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added workflow transition evidence {} {} to workflow {}",
            evidence.kind().as_ref(),
            evidence.trigger().as_ref(),
            workflow_slug.as_ref()
        ))?),
    ]))
}

fn lean_workflow_outcome_record(outcome: &WorkflowOutcomeRecord) -> String {
    format!(
        "{{ sourceSlice := {}, label := {}, externallyRelevant := {} }}",
        quoted(outcome.source_slice().as_ref()),
        quoted(outcome.label().as_ref()),
        if outcome.externally_relevant() {
            "true"
        } else {
            "false"
        },
    )
}

fn quint_workflow_outcome_record(outcome: &WorkflowOutcomeRecord) -> String {
    format!(
        "{{ sourceSlice: {}, label: {}, externallyRelevant: {} }}",
        quoted(outcome.source_slice().as_ref()),
        quoted(outcome.label().as_ref()),
        outcome.externally_relevant(),
    )
}

fn lean_workflow_command_error_record(error: &WorkflowCommandErrorRecord) -> String {
    format!(
        "{{ sourceSlice := {}, commandName := {}, errorName := {} }}",
        quoted(error.source_slice().as_ref()),
        quoted(error.command_name().as_ref()),
        quoted(error.error_name().as_ref()),
    )
}

fn quint_workflow_command_error_record(error: &WorkflowCommandErrorRecord) -> String {
    format!(
        "{{ sourceSlice: {}, commandName: {}, errorName: {} }}",
        quoted(error.source_slice().as_ref()),
        quoted(error.command_name().as_ref()),
        quoted(error.error_name().as_ref()),
    )
}

fn lean_workflow_owned_definition_record(definition: &WorkflowOwnedDefinitionRecord) -> String {
    format!(
        "{{ sourceSlice := {}, definitionKind := {}, definitionName := {} }}",
        quoted(definition.source_slice().as_ref()),
        quoted(definition.definition_kind().as_ref()),
        quoted(definition.definition_name().as_ref()),
    )
}

fn quint_workflow_owned_definition_record(definition: &WorkflowOwnedDefinitionRecord) -> String {
    format!(
        "{{ sourceSlice: {}, definitionKind: {}, definitionName: {} }}",
        quoted(definition.source_slice().as_ref()),
        quoted(definition.definition_kind().as_ref()),
        quoted(definition.definition_name().as_ref()),
    )
}

fn lean_workflow_transition_evidence_record(evidence: &WorkflowTransitionEvidenceRecord) -> String {
    format!(
        "{{ source := {}, target := {}, kind := {}, trigger := {}, sourceEvidence := {}, targetEvidence := {} }}",
        quoted(evidence.source().as_ref()),
        quoted(evidence.target().as_ref()),
        quoted(evidence.kind().as_ref()),
        quoted(evidence.trigger().as_ref()),
        quoted(evidence.source_evidence().as_ref()),
        quoted(evidence.target_evidence().as_ref()),
    )
}

fn quint_workflow_transition_evidence_record(
    evidence: &WorkflowTransitionEvidenceRecord,
) -> String {
    format!(
        "{{ source: {}, target: {}, kind: {}, trigger: {}, sourceEvidence: {}, targetEvidence: {} }}",
        quoted(evidence.source().as_ref()),
        quoted(evidence.target().as_ref()),
        quoted(evidence.kind().as_ref()),
        quoted(evidence.trigger().as_ref()),
        quoted(evidence.source_evidence().as_ref()),
        quoted(evidence.target_evidence().as_ref()),
    )
}

fn append_record(
    contents: &str,
    marker: &str,
    record: &str,
) -> Result<String, FormalWorkflowFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if let Some(current_list) = declaration.strip_prefix(marker) {
                replaced = true;
                Ok(format!(
                    "{indentation}{marker}{}",
                    append_list_record(current_list, record)?
                ))
            } else {
                Ok(line.to_owned())
            }
        })
        .collect::<Result<Vec<_>, FormalWorkflowFactError>>()?;

    if replaced {
        let mut updated = lines.join("\n");
        if contents.ends_with('\n') {
            updated.push('\n');
        }
        Ok(updated)
    } else {
        Err(FormalWorkflowFactError::new(format!(
            "formal workflow artifact is missing declaration {marker}"
        )))
    }
}

fn append_list_record(current_list: &str, record: &str) -> Result<String, FormalWorkflowFactError> {
    let trimmed = current_list.trim();
    if trimmed == "[]" {
        return Ok(format!("[{record}]"));
    }
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| {
            FormalWorkflowFactError::new("formal workflow artifact list is malformed")
        })?;
    Ok(format!("[{inner},{record}]"))
}

fn refresh_lean_digest(contents: String) -> Result<String, FormalWorkflowFactError> {
    let artifact = file_contents(contents.clone())?;
    let graph = parse_lean_workflow_graph(&artifact)
        .map_err(|error| FormalWorkflowFactError::new(error.to_string()))?;
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: graph.name().clone(),
        workflow_slug: graph.slug().clone(),
        workflow_description: graph.description().clone(),
        workflow_slice_details: graph.slice_details().clone(),
        workflow_transitions: graph.transitions().clone(),
        workflow_outcomes: graph.outcomes().clone(),
        workflow_command_errors: graph.command_errors().clone(),
        workflow_owned_definitions: graph.owned_definitions().clone(),
        workflow_transition_evidences: graph.transition_evidences().clone(),
    });
    replace_digest_marker(contents, "-- EMC-DIGEST: ", digest.as_ref())
}

fn refresh_quint_digest(contents: String) -> Result<String, FormalWorkflowFactError> {
    let artifact = file_contents(contents.clone())?;
    let graph = parse_quint_workflow_graph(&artifact)
        .map_err(|error| FormalWorkflowFactError::new(error.to_string()))?;
    let digest = artifact_digest(WorkflowArtifactDigestInput {
        workflow_name: graph.name().clone(),
        workflow_slug: graph.slug().clone(),
        workflow_description: graph.description().clone(),
        workflow_slice_details: graph.slice_details().clone(),
        workflow_transitions: graph.transitions().clone(),
        workflow_outcomes: graph.outcomes().clone(),
        workflow_command_errors: graph.command_errors().clone(),
        workflow_owned_definitions: graph.owned_definitions().clone(),
        workflow_transition_evidences: graph.transition_evidences().clone(),
    });
    replace_digest_marker(contents, "// EMC-DIGEST: ", digest.as_ref())
}

fn replace_digest_marker(
    contents: String,
    marker: &str,
    digest: &str,
) -> Result<String, FormalWorkflowFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if declaration.starts_with(marker) {
                replaced = true;
                format!("{indentation}{marker}{digest}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>();

    if replaced {
        let mut updated = lines.join("\n");
        if contents.ends_with('\n') {
            updated.push('\n');
        }
        Ok(updated)
    } else {
        Err(FormalWorkflowFactError::new(
            "formal workflow artifact is missing digest marker",
        ))
    }
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated formal workflow string literal must be valid: {error}");
    })
}

fn file_contents(value: impl Into<String>) -> Result<FileContents, FormalWorkflowFactError> {
    FileContents::try_new(value.into()).map_err(|error| {
        FormalWorkflowFactError::new(format!("invalid formal workflow file contents: {error}"))
    })
}

fn report_line(value: impl Into<String>) -> Result<ReportLine, FormalWorkflowFactError> {
    ReportLine::try_new(value.into())
        .map_err(|error| FormalWorkflowFactError::new(format!("invalid report line: {error}")))
}
