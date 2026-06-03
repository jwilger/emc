use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::core::validation::{
    EventModelDocument, EventModelFileKind, validate_event_model, validate_event_model_corpus,
};
use crate::io::dto::parse_event_model_document;
use crate::shell::ShellError;

pub fn validate_target(target: &Path) -> Result<(), ShellError> {
    let files = event_model_files(target)?;
    let documents = files
        .iter()
        .map(|path| parse_and_validate_event_model_file(path))
        .collect::<Result<Vec<_>, _>>()?;
    validate_event_model_corpus(&documents)
        .map_err(|issue| ShellError::message(format!("{} in {}", issue, target.display())))?;
    files
        .iter()
        .try_for_each(|path| validate_workflow_referenced_slice_files(path))
}

fn parse_and_validate_event_model_file(path: &Path) -> Result<EventModelDocument, ShellError> {
    let source =
        fs::read_to_string(path).map_err(|error| ShellError::message(error.to_string()))?;
    let document = parse_event_model_document(&source, event_model_file_kind(path))
        .map_err(|error| ShellError::message(format!("{} in {}", error, path.display())))?;
    validate_event_model(&document)
        .map_err(|issue| ShellError::message(format!("{} in {}", issue, path.display())))?;
    Ok(document)
}

fn validate_workflow_referenced_slice_files(path: &Path) -> Result<(), ShellError> {
    if event_model_file_kind(path) != EventModelFileKind::Workflow {
        return Ok(());
    }

    let source =
        fs::read_to_string(path).map_err(|error| ShellError::message(error.to_string()))?;
    let value = serde_json::from_str::<Value>(&source)
        .map_err(|error| ShellError::message(format!("{} in {}", error, path.display())))?;
    let Some(slice_files) = value.get("slice_files").and_then(Value::as_array) else {
        return Ok(());
    };
    let base_path = path.parent().unwrap_or_else(|| Path::new(""));
    slice_files
        .iter()
        .filter_map(Value::as_str)
        .map(|slice_file| base_path.join(slice_file))
        .try_for_each(|slice_file| validate_referenced_slice_file(path, &slice_file))
}

fn validate_referenced_slice_file(
    workflow_path: &Path,
    slice_file: &Path,
) -> Result<(), ShellError> {
    if !slice_file.is_file() {
        return Err(ShellError::message(format!(
            "missing referenced slice file {} in {}",
            slice_file.display(),
            workflow_path.display()
        )));
    }

    let source =
        fs::read_to_string(slice_file).map_err(|error| ShellError::message(error.to_string()))?;
    let document =
        parse_event_model_document(&source, EventModelFileKind::Slice).map_err(|error| {
            ShellError::message(format!(
                "referenced slice file {} is invalid: {}",
                slice_file.display(),
                error
            ))
        })?;
    validate_event_model(&document).map_err(|issue| {
        ShellError::message(format!(
            "referenced slice file {} is invalid: {}",
            slice_file.display(),
            issue
        ))
    })
}

fn event_model_file_kind(path: &Path) -> EventModelFileKind {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|file_name| file_name.to_str())
        .filter(|file_name| *file_name == "slices")
        .map_or(EventModelFileKind::Workflow, |_| EventModelFileKind::Slice)
}

fn event_model_files(directory: &Path) -> Result<Vec<PathBuf>, ShellError> {
    if directory.is_file() {
        return Ok(vec![directory.to_path_buf()]);
    }

    let mut files = fs::read_dir(directory)
        .map_err(|error| ShellError::message(error.to_string()))?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ShellError::message(error.to_string()))?
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .is_some_and(|file_name| file_name.ends_with(".eventmodel.json"))
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
