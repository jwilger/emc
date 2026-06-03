use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::effect::{Effect, EffectPlan, ProcessInvocation, ProjectPath};
use serde_json::Value;

const REQUIRED_REVIEW_CATEGORIES: &[&str] = &[
    "lifecycle-entry",
    "canonical-lanes",
    "board-connections",
    "fake-intermediates",
    "slice-ownership",
    "source-chains",
    "workflow-reachability",
    "transition-resolution",
    "navigation-targets",
    "branch-shape",
    "outcomes-and-errors",
    "scenario-coverage",
    "timeline-rendering",
];

#[derive(Debug)]
pub struct ShellError {
    message: String,
}

impl ShellError {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn project_name(error: impl Display) -> Self {
        Self {
            message: format!("invalid project name: {error}"),
        }
    }

    fn io(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl Display for ShellError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ShellError {}

pub fn interpret(plan: EffectPlan) -> Result<(), ShellError> {
    interpret_collect_reports(plan).map(|reports| {
        reports.into_iter().for_each(|report| println!("{report}"));
    })
}

pub fn interpret_collect_reports(plan: EffectPlan) -> Result<Vec<String>, ShellError> {
    plan.effects()
        .iter()
        .try_fold(Vec::new(), |mut reports, effect| {
            if let Some(report) = interpret_effect(effect)? {
                reports.push(report);
            }
            Ok(reports)
        })
}

fn interpret_effect(effect: &Effect) -> Result<Option<String>, ShellError> {
    match effect {
        Effect::CopyDirectory(source, target) => {
            copy_directory(source.as_ref(), target.as_ref()).map(|()| None)
        }
        Effect::EnsureDirectory(path) => fs::create_dir_all(Path::new(path.as_ref()))
            .map(|()| None)
            .map_err(ShellError::io),
        Effect::Fail(message) => Err(ShellError::message(message.as_ref().to_owned())),
        Effect::RequireCanonicalDeclaration(path, prefix, marker, message) => {
            let contents = fs::read_to_string(Path::new(path.as_ref())).map_err(ShellError::io)?;
            if artifact_contains_one_canonical_declaration(
                &contents,
                prefix.as_ref(),
                marker.as_ref(),
            ) {
                Ok(None)
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireDigest(path, digest, message) => {
            let contents = fs::read_to_string(Path::new(path.as_ref())).map_err(ShellError::io)?;
            if contents.contains(digest.as_ref()) {
                Ok(None)
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireFile(path) => {
            if Path::new(path.as_ref()).is_file() {
                Ok(None)
            } else {
                Err(ShellError::message(format!(
                    "missing required project artifact {}",
                    path.as_ref()
                )))
            }
        }
        Effect::RequireIndexedWorkflowFiles(index_path, workflows_path, message) => {
            require_indexed_workflow_files(
                index_path.as_ref(),
                workflows_path.as_ref(),
                message.as_ref(),
            )
            .map(|()| None)
        }
        Effect::RequireJsonObjectKeysUnique(path, message) => {
            require_json_object_keys_unique(path.as_ref(), message.as_ref()).map(|()| None)
        }
        Effect::RequireOnlyModeledArtifacts(path, extension, allowed_paths, message) => {
            require_only_modeled_artifacts(
                path.as_ref(),
                extension.as_ref(),
                allowed_paths,
                message.as_ref(),
            )
            .map(|()| None)
        }
        Effect::RequireReferencedSliceFiles(workflows_path, slices_path, message) => {
            require_referenced_slice_files(
                workflows_path.as_ref(),
                slices_path.as_ref(),
                message.as_ref(),
            )
            .map(|()| None)
        }
        Effect::RequireReviewRecord(path, workflow_path, message) => {
            if Path::new(path.as_ref()).is_file() {
                require_clean_review_record(path.as_ref(), workflow_path.as_ref(), message.as_ref())
                    .map(|()| None)
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireWorkflowSliceFiles(workflow_path, message) => {
            let workflow_contents =
                fs::read_to_string(Path::new(workflow_path.as_ref())).map_err(ShellError::io)?;
            let slice_files =
                workflow_slice_file_paths(workflow_path.as_ref(), &workflow_contents)?;
            if slice_files.iter().all(|slice_file| slice_file.is_file()) {
                Ok(None)
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireWorkflowSliceDetails(
            workflow_path,
            artifact_path,
            marker_prefix,
            message,
        ) => {
            let workflow_contents =
                fs::read_to_string(Path::new(workflow_path.as_ref())).map_err(ShellError::io)?;
            let artifact_contents =
                fs::read_to_string(Path::new(artifact_path.as_ref())).map_err(ShellError::io)?;
            let marker = workflow_slice_detail_marker(marker_prefix.as_ref(), &workflow_contents)?;
            if artifact_contains_one_canonical_declaration(
                &artifact_contents,
                marker_prefix.as_ref(),
                &marker,
            ) {
                Ok(None)
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireWorkflowSlices(workflow_path, artifact_path, marker_prefix, message) => {
            let workflow_contents =
                fs::read_to_string(Path::new(workflow_path.as_ref())).map_err(ShellError::io)?;
            let artifact_contents =
                fs::read_to_string(Path::new(artifact_path.as_ref())).map_err(ShellError::io)?;
            let marker = workflow_slice_marker(marker_prefix.as_ref(), &workflow_contents)?;
            if artifact_contains_one_canonical_declaration(
                &artifact_contents,
                marker_prefix.as_ref(),
                &marker,
            ) {
                Ok(None)
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireWorkflowTransitions(
            workflow_path,
            artifact_path,
            marker_prefix,
            message,
        ) => {
            let workflow_contents =
                fs::read_to_string(Path::new(workflow_path.as_ref())).map_err(ShellError::io)?;
            let artifact_contents =
                fs::read_to_string(Path::new(artifact_path.as_ref())).map_err(ShellError::io)?;
            let marker = workflow_transition_marker(marker_prefix.as_ref(), &workflow_contents)?;
            if artifact_contains_one_canonical_declaration(
                &artifact_contents,
                marker_prefix.as_ref(),
                &marker,
            ) {
                Ok(None)
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RunProcess(invocation) => run_process(invocation),
        Effect::WriteFile(path, contents) => {
            write_file(path.as_ref(), contents.as_ref()).map(|()| None)
        }
        Effect::WriteFileIfMissing(path, contents) => {
            if Path::new(path.as_ref()).exists() {
                Ok(None)
            } else {
                write_file(path.as_ref(), contents.as_ref()).map(|()| None)
            }
        }
        Effect::Report(line) => Ok(Some(line.as_ref().to_owned())),
        Effect::ReportDocument(contents) => Ok(Some(contents.as_ref().to_owned())),
    }
}

fn require_indexed_workflow_files(
    index_path: &str,
    workflows_path: &str,
    message: &str,
) -> Result<(), ShellError> {
    let index_contents = fs::read_to_string(Path::new(index_path)).map_err(ShellError::io)?;
    let indexed_paths = indexed_workflow_paths(&index_contents)?;
    let mut workflow_files = fs::read_dir(Path::new(workflows_path))
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    workflow_files.sort();

    workflow_files
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .filter(|file_name| file_name.ends_with(".eventmodel.json"))
                .map(str::to_owned)
        })
        .find(|file_name| {
            let indexed_path = format!("data/workflows/{file_name}");
            !indexed_paths.contains(&indexed_path)
        })
        .map_or(Ok(()), |file_name| {
            Err(ShellError::message(format!("{message} for {file_name}")))
        })
}

fn indexed_workflow_paths(index_contents: &str) -> Result<BTreeSet<String>, ShellError> {
    let index = serde_json::from_str::<Value>(index_contents)
        .map_err(|error| ShellError::message(format!("invalid browser index JSON: {error}")))?;
    let workflows = index
        .get("workflows")
        .and_then(Value::as_array)
        .ok_or_else(|| ShellError::message("browser index is missing workflows"))?;

    Ok(workflows
        .iter()
        .filter_map(|workflow| workflow.get("path").and_then(Value::as_str))
        .map(str::to_owned)
        .collect())
}

fn require_json_object_keys_unique(path: &str, message: &str) -> Result<(), ShellError> {
    let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
    if JsonKeyScanner::new(&contents).contains_duplicate_key() {
        Err(ShellError::message(message.to_owned()))
    } else {
        Ok(())
    }
}

struct JsonKeyScanner<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> JsonKeyScanner<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn contains_duplicate_key(mut self) -> bool {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> bool {
        self.skip_whitespace();
        match self.input[self.cursor..].chars().next() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => {
                self.parse_string();
                false
            }
            Some(_) => {
                self.parse_scalar();
                false
            }
            None => false,
        }
    }

    fn parse_object(&mut self) -> bool {
        self.bump_char();
        let mut keys = BTreeSet::new();
        loop {
            self.skip_whitespace();
            if self.consume_char('}') {
                return false;
            }
            let Some(key) = self.parse_string() else {
                return false;
            };
            if !keys.insert(key) {
                return true;
            }
            self.skip_whitespace();
            if !self.consume_char(':') || self.parse_value() {
                return true;
            }
            self.skip_whitespace();
            if self.consume_char('}') {
                return false;
            }
            if !self.consume_char(',') {
                return false;
            }
        }
    }

    fn parse_array(&mut self) -> bool {
        self.bump_char();
        loop {
            self.skip_whitespace();
            if self.consume_char(']') {
                return false;
            }
            if self.parse_value() {
                return true;
            }
            self.skip_whitespace();
            if self.consume_char(']') {
                return false;
            }
            if !self.consume_char(',') {
                return false;
            }
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        if !self.consume_char('"') {
            return None;
        }
        let mut value = String::new();
        while let Some(character) = self.bump_char() {
            match character {
                '"' => return Some(value),
                '\\' => {
                    self.bump_char().into_iter().for_each(|escaped| {
                        value.push(escaped);
                    });
                }
                _ => value.push(character),
            }
        }
        None
    }

    fn parse_scalar(&mut self) {
        while self.input[self.cursor..]
            .chars()
            .next()
            .is_some_and(|character| !matches!(character, ',' | ']' | '}'))
        {
            self.bump_char();
        }
    }

    fn skip_whitespace(&mut self) {
        while self.input[self.cursor..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            self.bump_char();
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.input[self.cursor..].starts_with(expected) {
            self.bump_char();
            true
        } else {
            false
        }
    }

    fn bump_char(&mut self) -> Option<char> {
        let character = self.input[self.cursor..].chars().next()?;
        self.cursor += character.len_utf8();
        Some(character)
    }
}

fn require_only_modeled_artifacts(
    path: &str,
    extension: &str,
    allowed_paths: &[ProjectPath],
    message: &str,
) -> Result<(), ShellError> {
    let allowed_file_names = allowed_artifact_file_names(allowed_paths);
    let mut artifact_files = fs::read_dir(Path::new(path))
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    artifact_files.sort();

    artifact_files
        .into_iter()
        .filter_map(|artifact_path| {
            artifact_path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .filter(|file_name| file_name.ends_with(extension))
                .map(str::to_owned)
        })
        .find(|file_name| !allowed_file_names.contains(file_name))
        .map_or(Ok(()), |file_name| {
            Err(ShellError::message(format!("{message} for {file_name}")))
        })
}

fn allowed_artifact_file_names(allowed_paths: &[ProjectPath]) -> BTreeSet<String> {
    allowed_paths
        .iter()
        .filter_map(|path| {
            Path::new(path.as_ref())
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(str::to_owned)
        })
        .collect()
}

fn require_referenced_slice_files(
    workflows_path: &str,
    slices_path: &str,
    message: &str,
) -> Result<(), ShellError> {
    let referenced_slice_files = referenced_slice_file_names(workflows_path)?;
    let mut slice_files = fs::read_dir(Path::new(slices_path))
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    slice_files.sort();

    slice_files
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .filter(|file_name| file_name.ends_with(".eventmodel.json"))
                .map(str::to_owned)
        })
        .find(|file_name| !referenced_slice_files.contains(file_name))
        .map_or(Ok(()), |file_name| {
            Err(ShellError::message(format!("{message} for {file_name}")))
        })
}

fn referenced_slice_file_names(workflows_path: &str) -> Result<BTreeSet<String>, ShellError> {
    let mut workflow_files = fs::read_dir(Path::new(workflows_path))
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    workflow_files.sort();

    workflow_files
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .is_some_and(|file_name| file_name.ends_with(".eventmodel.json"))
        })
        .try_fold(BTreeSet::new(), |mut referenced_files, workflow_path| {
            let workflow_contents = fs::read_to_string(&workflow_path).map_err(ShellError::io)?;
            workflow_slice_file_names(&workflow_contents)?
                .into_iter()
                .for_each(|file_name| {
                    referenced_files.insert(file_name);
                });
            Ok(referenced_files)
        })
}

fn workflow_slice_file_names(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    let workflow = workflow_json(workflow_contents)?;
    Ok(workflow
        .get("slice_files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|slice_file| slice_file.rsplit('/').next())
        .map(str::to_owned)
        .collect())
}

fn require_clean_review_record(
    path: &str,
    workflow_path: &str,
    fallback_message: &str,
) -> Result<(), ShellError> {
    let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
    let record = serde_json::from_str::<Value>(&contents)
        .map_err(|_error| ShellError::message(fallback_message.to_owned()))?;
    let Some(record_object) = record.as_object() else {
        return Err(ShellError::message(fallback_message.to_owned()));
    };
    let expected_workflow_slug = review_record_workflow_slug(path)?;
    if record_object.get("workflow_slug").and_then(Value::as_str)
        != Some(expected_workflow_slug.as_str())
    {
        let observed = record_object
            .get("workflow_slug")
            .and_then(Value::as_str)
            .unwrap_or("");
        return Err(ShellError::message(format!(
            "review record workflow '{observed}' does not match '{expected_workflow_slug}'"
        )));
    }
    let current_digest = model_content_digest(workflow_path)?;
    if record_object.get("status").and_then(Value::as_str) != Some("clean") {
        if mandatory_findings_include_digest(record_object, &current_digest) {
            return Err(ShellError::message(
                "mandatory review findings remain for current model digest",
            ));
        }
        if mandatory_findings_are_present(record_object)
            && record_object
                .get("model_content_digest")
                .and_then(Value::as_str)
                != Some(current_digest.as_str())
        {
            return Err(ShellError::message(
                "corrected workflow requires clean follow-up review",
            ));
        }
        return Err(ShellError::message(fallback_message.to_owned()));
    }
    if record_object
        .get("model_content_digest")
        .and_then(Value::as_str)
        != Some(current_digest.as_str())
    {
        return Err(ShellError::message(
            "clean review is stale for current model digest",
        ));
    }
    let Some(category_results) = record_object
        .get("category_results")
        .and_then(Value::as_object)
    else {
        return Err(ShellError::message(fallback_message.to_owned()));
    };

    REQUIRED_REVIEW_CATEGORIES.iter().try_for_each(|category| {
        match category_results.get(*category).and_then(Value::as_str) {
            Some("clean") => Ok(()),
            Some(_) => Err(ShellError::message(format!(
                "review category '{category}' is not clean"
            ))),
            None => Err(ShellError::message(format!(
                "clean review is missing category '{category}'"
            ))),
        }
    })
}

fn review_record_workflow_slug(path: &str) -> Result<String, ShellError> {
    Path::new(path)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .and_then(|file_name| file_name.strip_suffix(".review.json"))
        .map(str::to_owned)
        .ok_or_else(|| ShellError::message("review record path is invalid"))
}

fn mandatory_findings_include_digest(
    record_object: &serde_json::Map<String, Value>,
    current_digest: &str,
) -> bool {
    record_object
        .get("mandatory_findings")
        .and_then(Value::as_array)
        .is_some_and(|findings| {
            findings.iter().any(|finding| {
                finding.get("model_content_digest").and_then(Value::as_str) == Some(current_digest)
            })
        })
}

fn mandatory_findings_are_present(record_object: &serde_json::Map<String, Value>) -> bool {
    record_object
        .get("mandatory_findings")
        .and_then(Value::as_array)
        .is_some_and(|findings| !findings.is_empty())
}

fn model_content_digest(workflow_path: &str) -> Result<String, ShellError> {
    let workflow_contents = fs::read_to_string(Path::new(workflow_path)).map_err(ShellError::io)?;
    let slice_files = workflow_slice_file_paths(workflow_path, &workflow_contents)?;
    let mut digest = StableDigest::new();
    digest.write(workflow_path);
    digest.write(&workflow_contents);
    slice_files.into_iter().try_for_each(|slice_file| {
        let slice_contents = fs::read_to_string(&slice_file).map_err(ShellError::io)?;
        digest.write(&slice_file.to_string_lossy());
        digest.write(&slice_contents);
        Ok::<(), ShellError>(())
    })?;
    Ok(digest.finish())
}

struct StableDigest {
    value: u64,
}

impl StableDigest {
    fn new() -> Self {
        Self {
            value: 0xcbf2_9ce4_8422_2325,
        }
    }

    fn write(&mut self, value: &str) {
        value.as_bytes().iter().for_each(|byte| {
            self.value ^= u64::from(*byte);
            self.value = self.value.wrapping_mul(0x0000_0100_0000_01b3);
        });
    }

    fn finish(self) -> String {
        format!("emc-fnv1a64:{:016x}", self.value)
    }
}

fn workflow_slice_marker(prefix: &str, workflow_contents: &str) -> Result<String, ShellError> {
    let labels = workflow_slice_labels(workflow_contents)?;
    let joined_labels = labels.join(",");
    Ok(format!("{prefix} [{joined_labels}]"))
}

fn workflow_slice_detail_marker(
    prefix: &str,
    workflow_contents: &str,
) -> Result<String, ShellError> {
    let labels = if prefix.starts_with("val ") {
        workflow_slice_detail_record_labels(workflow_contents)?
    } else {
        workflow_slice_detail_tuple_labels(workflow_contents)?
    };
    let joined_labels = labels.join(",");
    Ok(format!("{prefix} [{joined_labels}]"))
}

fn workflow_slice_file_paths(
    workflow_path: &str,
    workflow_contents: &str,
) -> Result<Vec<PathBuf>, ShellError> {
    let workflow = workflow_json(workflow_contents)?;
    let slice_files = workflow
        .get("slice_files")
        .and_then(Value::as_array)
        .ok_or_else(|| ShellError::message("workflow document is missing slice_files"))?;
    let base_path = Path::new(workflow_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    slice_files
        .iter()
        .filter_map(Value::as_str)
        .map(|slice_file| Ok(base_path.join(slice_file)))
        .collect()
}

fn workflow_transition_marker(prefix: &str, workflow_contents: &str) -> Result<String, ShellError> {
    let labels = workflow_transition_labels(workflow_contents)?;
    let joined_labels = labels.join(",");
    Ok(format!("{prefix} [{joined_labels}]"))
}

fn artifact_contains_one_canonical_declaration(
    artifact_contents: &str,
    prefix: &str,
    marker: &str,
) -> bool {
    let mut declarations = artifact_contents
        .lines()
        .filter_map(|line| canonical_declaration_line(line, prefix));

    matches!(
        (declarations.next(), declarations.next()),
        (Some(declaration), None) if declaration == marker
    )
}

fn canonical_declaration_line<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.starts_with(' ') && line.starts_with(prefix) {
        Some(line)
    } else {
        let trimmed = line.trim_start();
        trimmed.starts_with(prefix).then_some(trimmed)
    }
}

fn workflow_slice_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    let workflow = workflow_json(workflow_contents)?;
    let steps = workflow_steps(&workflow)?;

    steps
        .iter()
        .filter_map(|step| step.get("slice").and_then(Value::as_str))
        .map(|slice| json_string(slice.to_owned()))
        .collect()
}

fn workflow_slice_detail_tuple_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    let workflow = workflow_json(workflow_contents)?;
    let steps = workflow_steps(&workflow)?;

    steps
        .iter()
        .map(|step| {
            let slug = step
                .get("slice")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing slice"))?;
            let name = step
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing name"))?;
            let kind = step
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing type"))?;
            let description = step
                .get("description")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing description"))?;
            Ok(format!(
                "({}, {}, {}, {})",
                json_string(slug.to_owned())?,
                json_string(name.to_owned())?,
                json_string(kind.to_owned())?,
                json_string(description.to_owned())?
            ))
        })
        .collect()
}

fn workflow_slice_detail_record_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    let workflow = workflow_json(workflow_contents)?;
    let steps = workflow_steps(&workflow)?;

    steps
        .iter()
        .map(|step| {
            let slug = step
                .get("slice")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing slice"))?;
            let name = step
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing name"))?;
            let kind = step
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing type"))?;
            let description = step
                .get("description")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("workflow step is missing description"))?;
            Ok(format!(
                "{{ slug: {}, name: {}, kind: {}, description: {} }}",
                json_string(slug.to_owned())?,
                json_string(name.to_owned())?,
                json_string(kind.to_owned())?,
                json_string(description.to_owned())?
            ))
        })
        .collect()
}

fn workflow_transition_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    let workflow = workflow_json(workflow_contents)?;
    let steps = workflow_steps(&workflow)?;

    steps
        .iter()
        .filter_map(|step| {
            let source = step.get("slice").and_then(Value::as_str)?;
            let transitions = step.get("transitions").and_then(Value::as_array)?;
            Some((source, transitions))
        })
        .flat_map(|(source, transitions)| {
            transitions.iter().filter_map(move |transition| {
                let target = transition.get("to").and_then(Value::as_str)?;
                transition_label(source, target, transition)
            })
        })
        .map(json_string)
        .collect()
}

fn transition_label(source: &str, target: &str, transition: &Value) -> Option<String> {
    [
        ("via_command", "command"),
        ("via_event", "event"),
        ("via_navigation", "navigation"),
    ]
    .into_iter()
    .find_map(|(field, kind)| {
        transition
            .get(field)
            .and_then(Value::as_str)
            .map(|trigger| format!("{source}->{target}:{kind}:{trigger}"))
    })
}

fn workflow_json(workflow_contents: &str) -> Result<Value, ShellError> {
    serde_json::from_str::<Value>(workflow_contents)
        .map_err(|error| ShellError::message(format!("invalid workflow JSON: {error}")))
}

fn workflow_steps(workflow: &Value) -> Result<&Vec<Value>, ShellError> {
    workflow
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| ShellError::message("workflow document is missing steps"))
}

fn json_string(value: String) -> Result<String, ShellError> {
    serde_json::to_string(&value).map_err(|error| {
        ShellError::message(format!("failed to encode workflow transition: {error}"))
    })
}

fn write_file(path: &str, contents: &str) -> Result<(), ShellError> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(ShellError::io)?;
    }
    fs::write(Path::new(path), contents).map_err(ShellError::io)
}

fn copy_directory(source: &str, target: &str) -> Result<(), ShellError> {
    copy_directory_path(Path::new(source), Path::new(target))
}

fn copy_directory_path(source: &Path, target: &Path) -> Result<(), ShellError> {
    fs::create_dir_all(target).map_err(ShellError::io)?;
    let mut entries = fs::read_dir(source)
        .map_err(ShellError::io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    entries.sort_by_key(|entry| entry.path());

    entries.into_iter().try_for_each(|entry| {
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory_path(&source_path, &target_path)
        } else {
            fs::copy(source_path, target_path)
                .map(|_bytes| ())
                .map_err(ShellError::io)
        }
    })
}

fn run_process(invocation: &ProcessInvocation) -> Result<Option<String>, ShellError> {
    let status = Command::new(invocation.program().as_ref())
        .args(
            invocation
                .arguments()
                .iter()
                .map(|argument| argument.as_ref()),
        )
        .status()
        .map_err(|error| {
            ShellError::message(format!(
                "failed to run {}: {}. Install pinned EMC tooling or use the Nix package",
                invocation.program().as_ref(),
                error
            ))
        })?;

    if status.success() {
        Ok(Some(invocation.success().as_ref().to_owned()))
    } else {
        Err(ShellError::message(format!(
            "verification command {} failed with {}",
            invocation.program().as_ref(),
            status
        )))
    }
}
