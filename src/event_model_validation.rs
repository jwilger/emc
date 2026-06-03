use std::path::{Component, Path, PathBuf};

use crate::core::effect::{FileContents, ProjectPath};
use crate::core::types::WorkflowSliceFileReference;
use crate::core::validation::{
    EventModelDocument, EventModelFileKind, validate_event_model, validate_event_model_corpus,
};
use crate::core::workflow_document::WorkflowDocument;
use crate::io::dto::{parse_event_model_document, parse_slice_slug};
use crate::shell::ShellError;

pub fn validate_event_model_sources(
    target: &ProjectPath,
    sources: &[(ProjectPath, FileContents)],
    referenced_sources: &[(ProjectPath, FileContents)],
) -> Result<(), ShellError> {
    sources.iter().try_for_each(|(path, source)| {
        validate_workflow_referenced_slice_file_paths(path, source)
    })?;
    let documents = sources
        .iter()
        .map(|(path, source)| parse_and_validate_event_model_file(path, source))
        .collect::<Result<Vec<_>, _>>()?;
    validate_event_model_corpus(&documents)
        .map_err(|issue| ShellError::message(format!("{} in {}", issue, target.as_ref())))?;
    let lookup_sources = sources
        .iter()
        .cloned()
        .chain(referenced_sources.iter().cloned())
        .collect::<Vec<_>>();
    sources.iter().try_for_each(|(path, source)| {
        validate_workflow_referenced_slice_files(&lookup_sources, path, source)
    })
}

fn validate_workflow_referenced_slice_file_paths(
    path: &ProjectPath,
    source: &FileContents,
) -> Result<(), ShellError> {
    if event_model_file_kind(path) != EventModelFileKind::Workflow {
        return Ok(());
    }

    let Some(slice_files) = optional_workflow_slice_files(path, source)? else {
        return Ok(());
    };
    slice_files
        .iter()
        .try_for_each(|slice_file| validate_referenced_slice_file_path(path, slice_file.as_ref()))
}

fn validate_referenced_slice_file_path(
    workflow_path: &ProjectPath,
    slice_file: &str,
) -> Result<(), ShellError> {
    let referenced_path = referenced_slice_path(workflow_path, slice_file)?;
    let file_name = Path::new(referenced_path.as_ref())
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| ShellError::message("referenced slice file path is invalid"))?;
    let Some(stem) = file_name.strip_suffix(".eventmodel.json") else {
        return Err(ShellError::message(format!(
            "referenced slice file path is invalid: {file_name}"
        )));
    };
    let slug = parse_slice_slug(stem).map_err(|error| {
        ShellError::message(format!("referenced slice file path is invalid: {error}"))
    })?;
    let canonical_file_name = format!("{}.eventmodel.json", slug.as_ref());
    if canonical_file_name == file_name {
        Ok(())
    } else {
        Err(ShellError::message(format!(
            "referenced slice file path is noncanonical: {file_name}"
        )))
    }
}

fn parse_and_validate_event_model_file(
    path: &ProjectPath,
    source: &FileContents,
) -> Result<EventModelDocument, ShellError> {
    let document = parse_event_model_document(source.as_ref(), event_model_file_kind(path))
        .map_err(|error| ShellError::message(format!("{} in {}", error, path.as_ref())))?;
    validate_event_model(&document)
        .map_err(|issue| ShellError::message(format!("{} in {}", issue, path.as_ref())))?;
    Ok(document)
}

fn validate_workflow_referenced_slice_files(
    sources: &[(ProjectPath, FileContents)],
    path: &ProjectPath,
    source: &FileContents,
) -> Result<(), ShellError> {
    if event_model_file_kind(path) != EventModelFileKind::Workflow {
        return Ok(());
    }

    let Some(slice_files) = optional_workflow_slice_files(path, source)? else {
        return Ok(());
    };
    slice_files
        .iter()
        .map(|slice_file| referenced_slice_path(path, slice_file.as_ref()))
        .try_for_each(|slice_file| validate_referenced_slice_file(sources, path, slice_file?))
}

fn optional_workflow_slice_files(
    path: &ProjectPath,
    source: &FileContents,
) -> Result<Option<Vec<WorkflowSliceFileReference>>, ShellError> {
    let Some(workflow) = WorkflowDocument::parse_optional(source).map_err(|error| {
        ShellError::message(format!("invalid JSON: {} in {}", error, path.as_ref()))
    })?
    else {
        return Ok(None);
    };
    workflow
        .optional_slice_files()
        .map_err(|error| ShellError::message(format!("{} in {}", error, path.as_ref())))
}

fn validate_referenced_slice_file(
    sources: &[(ProjectPath, FileContents)],
    workflow_path: &ProjectPath,
    slice_file: ProjectPath,
) -> Result<(), ShellError> {
    let Some((_path, source)) = sources
        .iter()
        .find(|(source_path, _contents)| source_path == &slice_file)
    else {
        return Err(ShellError::message(format!(
            "missing referenced slice file {} in {}",
            slice_file.as_ref(),
            workflow_path.as_ref()
        )));
    };
    let document = parse_event_model_document(source.as_ref(), EventModelFileKind::Slice).map_err(
        |error| {
            ShellError::message(format!(
                "referenced slice file {} is invalid: {}",
                slice_file.as_ref(),
                error
            ))
        },
    )?;
    validate_event_model(&document).map_err(|issue| {
        ShellError::message(format!(
            "referenced slice file {} is invalid: {}",
            slice_file.as_ref(),
            issue
        ))
    })
}

fn referenced_slice_path(
    workflow_path: &ProjectPath,
    slice_file: &str,
) -> Result<ProjectPath, ShellError> {
    let base_path = Path::new(workflow_path.as_ref())
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let path = normalize_project_path(base_path.join(slice_file).as_path())?;
    ProjectPath::try_new(path.to_string_lossy().into_owned()).map_err(ShellError::project_path)
}

fn normalize_project_path(path: &Path) -> Result<PathBuf, ShellError> {
    path.components().try_fold(
        PathBuf::new(),
        |mut normalized, component| match component {
            Component::Normal(segment) => {
                normalized.push(segment);
                Ok(normalized)
            }
            Component::CurDir => Ok(normalized),
            Component::ParentDir => {
                if normalized.pop() {
                    Ok(normalized)
                } else {
                    Err(ShellError::project_path("path escapes project root"))
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                Err(ShellError::project_path("path must be project-relative"))
            }
        },
    )
}

fn event_model_file_kind(path: &ProjectPath) -> EventModelFileKind {
    Path::new(path.as_ref())
        .parent()
        .and_then(Path::file_name)
        .and_then(|file_name| file_name.to_str())
        .filter(|file_name| *file_name == "slices")
        .map_or(EventModelFileKind::Workflow, |_| EventModelFileKind::Slice)
}
