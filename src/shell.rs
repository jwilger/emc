use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::core::effect::{Effect, EffectPlan, ProcessInvocation};
use serde_json::Value;

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
            if artifact_contents.contains(&marker) {
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

fn workflow_transition_marker(prefix: &str, workflow_contents: &str) -> Result<String, ShellError> {
    let labels = workflow_transition_labels(workflow_contents)?;
    let joined_labels = labels.join(",");
    Ok(format!("{prefix} [{joined_labels}]"))
}

fn workflow_transition_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    let workflow = serde_json::from_str::<Value>(workflow_contents)
        .map_err(|error| ShellError::message(format!("invalid workflow JSON: {error}")))?;
    let steps = workflow
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| ShellError::message("workflow document is missing steps"))?;

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
                transition
                    .get("via_navigation")
                    .and_then(Value::as_str)
                    .map(|trigger| format!("{source}->{target}:navigation:{trigger}"))
            })
        })
        .map(json_string)
        .collect()
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
