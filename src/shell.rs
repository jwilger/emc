use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::core::connection::connect_workflow;
use crate::core::digest::{artifact_digest_from_workflow_document, slice_artifact_digest};
use crate::core::effect::{
    ArtifactDigest, Effect, EffectPlan, FileContents, ProcessInvocation, ProjectPath,
};
use crate::core::json_object_document::JsonObjectDocument;
use crate::core::layout::{ModeledWorkflowLayout, check_project, list_workflows, show_workflow};
use crate::core::project::ProjectName;
use crate::core::review_record::{ReviewCategoryFinding, ReviewRecordDocument};
use crate::core::site::generate_site;
use crate::core::slice::add_slice;
use crate::core::types::{
    ReviewRuleName, WorkflowSliceDetail, WorkflowSlug, WorkflowTransitionRecord,
};
use crate::core::verify::verify_project;
use crate::core::workflow::{add_workflow, update_workflow_description};
use crate::core::workflow_document::WorkflowDocument;
use crate::event_model_validation::validate_event_model_sources;
use crate::io::dto::{
    parse_browser_index_workflows, parse_project_manifest_name, parse_slice_slug,
};

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

    pub fn project_path(error: impl Display) -> Self {
        Self {
            message: format!("invalid project path: {error}"),
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
            reports.extend(interpret_effect(effect)?);
            Ok(reports)
        })
}

fn interpret_effect(effect: &Effect) -> Result<Vec<String>, ShellError> {
    match effect {
        Effect::AddSliceFromWorkflow(slice) => {
            let modeled_workflows = read_browser_index_workflows()?;
            let workflow_layout =
                find_indexed_workflow(slice.workflow_slug(), modeled_workflows.as_slice())?;
            let workflow_document = read_indexed_workflow_document_from_layouts(
                slice.workflow_slug(),
                &modeled_workflows,
            )?;
            let plan = add_slice(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_document,
                slice.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowFromIndex(workflow) => {
            let existing_workflows = read_browser_index_workflows()?;
            let plan = add_workflow(existing_workflows, workflow.clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::CheckCurrentProject => {
            let project_name = read_project_manifest_name()?;
            let modeled_workflows = read_browser_index_workflows()?;
            interpret_collect_reports(check_project(project_name, modeled_workflows))
        }
        Effect::ConnectWorkflowFromWorkflow(connection) => {
            let modeled_workflows = read_browser_index_workflows()?;
            let workflow_layout =
                find_indexed_workflow(connection.workflow_slug(), modeled_workflows.as_slice())?;
            let workflow_document = read_indexed_workflow_document_from_layouts(
                connection.workflow_slug(),
                &modeled_workflows,
            )?;
            let plan = connect_workflow(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_document,
                connection.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::CopyDirectory(source, target) => {
            copy_directory(source.as_ref(), target.as_ref()).map(|()| Vec::new())
        }
        Effect::EnsureDirectory(path) => fs::create_dir_all(Path::new(path.as_ref()))
            .map(|()| Vec::new())
            .map_err(ShellError::io),
        Effect::Fail(message) => Err(ShellError::message(message.as_ref().to_owned())),
        Effect::GenerateSiteFromManifest(output) => {
            let project_name = read_project_manifest_name()?;
            interpret_collect_reports(generate_site(project_name, output.clone()))
        }
        Effect::ListWorkflowsFromIndex => {
            let modeled_workflows = read_browser_index_workflows()?;
            interpret_collect_reports(list_workflows(modeled_workflows))
        }
        Effect::RequireCanonicalDeclaration(path, prefix, marker, message) => {
            let contents = fs::read_to_string(Path::new(path.as_ref())).map_err(ShellError::io)?;
            if artifact_contains_one_canonical_declaration(
                &contents,
                prefix.as_ref(),
                marker.as_ref(),
            ) {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireDigest(path, digest, message) => {
            let contents = fs::read_to_string(Path::new(path.as_ref())).map_err(ShellError::io)?;
            if artifact_contains_one_digest_marker(&contents, digest.as_ref()) {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireFile(path) => {
            if Path::new(path.as_ref()).is_file() {
                Ok(Vec::new())
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
            .map(|()| Vec::new())
        }
        Effect::RequireJsonObjectKeysUnique(path, message) => {
            require_json_object_keys_unique(path.as_ref(), message.as_ref()).map(|()| Vec::new())
        }
        Effect::RequireOnlyModeledArtifacts(path, extension, allowed_paths, message) => {
            require_only_modeled_artifacts(
                path.as_ref(),
                extension.as_ref(),
                allowed_paths,
                message.as_ref(),
            )
            .map(|()| Vec::new())
        }
        Effect::RequireOnlyModeledFormalSliceArtifacts(
            workflows_path,
            artifact_directory,
            extension,
            message,
        ) => require_only_modeled_formal_slice_artifacts(
            workflows_path.as_ref(),
            artifact_directory.as_ref(),
            extension.as_ref(),
            message.as_ref(),
        )
        .map(|()| Vec::new()),
        Effect::RequireReferencedSliceFiles(workflows_path, slices_path, message) => {
            require_referenced_slice_files(
                workflows_path.as_ref(),
                slices_path.as_ref(),
                message.as_ref(),
            )
            .map(|()| Vec::new())
        }
        Effect::RequireReferencedSliceFileIdentities(workflows_path, message) => {
            require_referenced_slice_file_identities(workflows_path.as_ref(), message.as_ref())
                .map(|()| Vec::new())
        }
        Effect::RequireReviewRecord(path, workflow_path, message) => {
            if Path::new(path.as_ref()).is_file() {
                require_clean_review_record(path.as_ref(), workflow_path.as_ref(), message.as_ref())
                    .map(|()| Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireWorkflowSliceJsonObjects(workflow_path, message) => {
            require_workflow_slice_json_objects(workflow_path.as_ref(), message.as_ref())
                .map(|()| Vec::new())
        }
        Effect::RequireWorkflowSliceJsonObjectKeysUnique(workflow_path, message) => {
            require_workflow_slice_json_object_keys_unique(workflow_path.as_ref(), message.as_ref())
                .map(|()| Vec::new())
        }
        Effect::RequireWorkflowSliceFiles(workflow_path, message) => {
            let workflow_contents =
                fs::read_to_string(Path::new(workflow_path.as_ref())).map_err(ShellError::io)?;
            let slice_files =
                workflow_slice_file_paths(workflow_path.as_ref(), &workflow_contents)?;
            if slice_files.iter().all(|slice_file| slice_file.is_file()) {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireWorkflowFormalSliceArtifacts(
            workflow_path,
            artifact_directory,
            extension,
            message,
        ) => {
            let workflow_contents =
                fs::read_to_string(Path::new(workflow_path.as_ref())).map_err(ShellError::io)?;
            require_formal_slice_artifacts(
                &workflow_contents,
                artifact_directory.as_ref(),
                extension.as_ref(),
                message.as_ref(),
            )
            .map(|()| Vec::new())
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
                Ok(Vec::new())
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
                Ok(Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireWorkflowDigest(workflow_path, artifact_path, workflow_slug, message) => {
            let workflow_contents =
                fs::read_to_string(Path::new(workflow_path.as_ref())).map_err(ShellError::io)?;
            let artifact_contents =
                fs::read_to_string(Path::new(artifact_path.as_ref())).map_err(ShellError::io)?;
            let workflow_document = FileContents::try_new(workflow_contents)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let digest =
                artifact_digest_from_workflow_document(workflow_slug.clone(), workflow_document)
                    .map_err(|error| ShellError::message(error.to_string()))?;
            if artifact_contains_one_digest_marker(&artifact_contents, digest.as_ref()) {
                Ok(Vec::new())
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
                Ok(Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RunProcess(invocation) => run_process(invocation),
        Effect::ShowWorkflowFromWorkflow(slug) => {
            let workflow_document = read_indexed_workflow_document(slug)?;
            interpret_collect_reports(show_workflow(workflow_document))
        }
        Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(slug, description) => {
            let existing_workflows = read_browser_index_workflows()?;
            let workflow_document =
                read_indexed_workflow_document_from_layouts(slug, existing_workflows.as_slice())?;
            let plan = update_workflow_description(
                existing_workflows,
                workflow_document,
                slug.clone(),
                description.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::ValidateEventModelTarget(target) => validate_event_model_target(target.as_ref()),
        Effect::VerifyProjectFromIndex => {
            let modeled_workflows = read_browser_index_workflows()?;
            let modeled_slices = read_modeled_workflow_slice_details(&modeled_workflows)?;
            interpret_collect_reports(verify_project(modeled_workflows, modeled_slices))
        }
        Effect::WriteFile(path, contents) => {
            write_file(path.as_ref(), contents.as_ref()).map(|()| Vec::new())
        }
        Effect::WriteFileIfMissing(path, contents) => {
            if Path::new(path.as_ref()).exists() {
                Ok(Vec::new())
            } else {
                write_file(path.as_ref(), contents.as_ref()).map(|()| Vec::new())
            }
        }
        Effect::Report(line) => Ok(vec![line.as_ref().to_owned()]),
        Effect::ReportDocument(contents) => Ok(vec![contents.as_ref().to_owned()]),
    }
}

fn read_project_manifest_name() -> Result<ProjectName, ShellError> {
    fs::read_to_string("emc.toml")
        .map_err(ShellError::io)
        .and_then(|manifest| {
            parse_project_manifest_name(&manifest).map_err(ShellError::project_name)
        })
}

fn read_browser_index_workflows() -> Result<Vec<ModeledWorkflowLayout>, ShellError> {
    fs::read_to_string("model/browser/data/index.json")
        .map_err(ShellError::io)
        .and_then(|index| {
            parse_browser_index_workflows(&index)
                .map_err(|error| ShellError::message(error.to_string()))
        })
}

fn read_indexed_workflow_document(slug: &WorkflowSlug) -> Result<FileContents, ShellError> {
    let modeled_workflows = read_browser_index_workflows()?;
    read_indexed_workflow_document_from_layouts(slug, modeled_workflows.as_slice())
}

fn read_indexed_workflow_document_from_layouts(
    slug: &WorkflowSlug,
    modeled_workflows: &[ModeledWorkflowLayout],
) -> Result<FileContents, ShellError> {
    find_indexed_workflow(slug, modeled_workflows)
        .and_then(|_workflow| read_workflow_document(slug.as_ref()))
}

fn find_indexed_workflow<'a>(
    slug: &WorkflowSlug,
    modeled_workflows: &'a [ModeledWorkflowLayout],
) -> Result<&'a ModeledWorkflowLayout, ShellError> {
    modeled_workflows
        .iter()
        .find(|workflow| workflow.slug() == slug)
        .ok_or_else(|| ShellError::message(format!("workflow {} is not indexed", slug.as_ref())))
}

fn read_modeled_workflow_slice_details(
    modeled_workflows: &[ModeledWorkflowLayout],
) -> Result<Vec<WorkflowSliceDetail>, ShellError> {
    let mut slice_details = Vec::new();
    for workflow in modeled_workflows {
        let workflow_document =
            read_workflow_document(workflow.slug().as_ref()).and_then(|contents| {
                WorkflowDocument::parse(&contents)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
        slice_details.extend(
            workflow_document
                .slice_details()
                .map_err(|error| ShellError::message(error.to_string()))?,
        );
    }
    Ok(slice_details)
}

fn read_workflow_document(slug: &str) -> Result<FileContents, ShellError> {
    fs::read_to_string(format!(
        "model/browser/data/workflows/{slug}.eventmodel.json"
    ))
    .map_err(ShellError::io)
    .and_then(|contents| {
        FileContents::try_new(contents).map_err(|error| ShellError::message(error.to_string()))
    })
}

fn validate_event_model_target(target: &str) -> Result<Vec<String>, ShellError> {
    let target_path = ProjectPath::try_new(target.to_owned()).map_err(ShellError::project_path)?;
    let sources = read_event_model_sources(event_model_files(Path::new(target))?)?;
    let referenced_slice_files = referenced_event_model_slice_files(&sources)?;
    let referenced_sources = read_event_model_sources(referenced_slice_files)?;
    validate_event_model_sources(&target_path, &sources, &referenced_sources)
        .map(|()| vec![format!("event model is valid at {target}")])
}

fn read_event_model_sources(
    paths: Vec<PathBuf>,
) -> Result<Vec<(ProjectPath, FileContents)>, ShellError> {
    paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .map_err(ShellError::io)
                .and_then(|contents| {
                    let project_path = ProjectPath::try_new(path.to_string_lossy().into_owned())
                        .map_err(ShellError::project_path)?;
                    let file_contents = FileContents::try_new(contents)
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    Ok((project_path, file_contents))
                })
        })
        .collect()
}

fn referenced_event_model_slice_files(
    sources: &[(ProjectPath, FileContents)],
) -> Result<Vec<PathBuf>, ShellError> {
    sources
        .iter()
        .map(|(path, contents)| {
            let base_path = Path::new(path.as_ref())
                .parent()
                .unwrap_or_else(|| Path::new(""));
            Ok(WorkflowDocument::parse(contents)
                .ok()
                .and_then(|workflow| workflow.slice_files().ok())
                .unwrap_or_default()
                .into_iter()
                .map(|slice_file| base_path.join(slice_file.as_ref()))
                .filter_map(|slice_file| normalize_project_path(slice_file.as_path()).ok())
                .filter(|slice_file| slice_file.is_file())
                .collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|nested| nested.into_iter().flatten().collect())
}

fn event_model_files(target: &Path) -> Result<Vec<PathBuf>, ShellError> {
    if target.is_file() {
        return normalize_project_path(target).map(|path| vec![path]);
    }

    let mut files = fs::read_dir(target)
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .is_some_and(|file_name| file_name.ends_with(".eventmodel.json"))
        })
        .map(|path| normalize_project_path(path.as_path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    Ok(files)
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

    let unindexed_workflow = workflow_files
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .filter(|file_name| file_name.ends_with(".eventmodel.json"))
                .map(str::to_owned)
        })
        .try_fold(None, |found, file_name| {
            if found.is_some() {
                return Ok(found);
            }
            let indexed_path = ProjectPath::try_new(format!("data/workflows/{file_name}"))
                .map_err(ShellError::project_path)?;
            Ok(if indexed_paths.contains(&indexed_path) {
                None
            } else {
                Some(file_name)
            })
        })?;

    unindexed_workflow.map_or(Ok(()), |file_name| {
        Err(ShellError::message(format!("{message} for {file_name}")))
    })
}

fn indexed_workflow_paths(index_contents: &str) -> Result<Vec<ProjectPath>, ShellError> {
    Ok(parse_browser_index_workflows(index_contents)
        .map_err(|error| ShellError::message(error.to_string()))?
        .iter()
        .map(ModeledWorkflowLayout::browser_data_path)
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

fn require_workflow_slice_json_objects(
    workflow_path: &str,
    message: &str,
) -> Result<(), ShellError> {
    let workflow_contents = fs::read_to_string(Path::new(workflow_path)).map_err(ShellError::io)?;
    workflow_slice_file_paths(workflow_path, &workflow_contents)?
        .iter()
        .try_for_each(|slice_file| require_json_object(&slice_file.to_string_lossy(), message))
}

fn require_json_object(path: &str, message: &str) -> Result<(), ShellError> {
    let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
    let file_contents = FileContents::try_new(contents)
        .map_err(|_error| ShellError::message(message.to_owned()))?;
    JsonObjectDocument::parse(&file_contents)
        .map(|_document| ())
        .map_err(|_error| ShellError::message(message.to_owned()))
}

fn require_workflow_slice_json_object_keys_unique(
    workflow_path: &str,
    message: &str,
) -> Result<(), ShellError> {
    let workflow_contents = fs::read_to_string(Path::new(workflow_path)).map_err(ShellError::io)?;
    workflow_slice_file_paths(workflow_path, &workflow_contents)?
        .iter()
        .try_for_each(|slice_file| {
            require_json_object_keys_unique(&slice_file.to_string_lossy(), message)
        })
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

fn require_only_modeled_formal_slice_artifacts(
    workflows_path: &str,
    artifact_directory: &str,
    extension: &str,
    message: &str,
) -> Result<(), ShellError> {
    if !Path::new(artifact_directory).exists() {
        return Ok(());
    }
    let mut workflow_paths = fs::read_dir(Path::new(workflows_path))
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    workflow_paths.sort();

    let allowed_paths = workflow_paths
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .is_some_and(|file_name| file_name.ends_with(".eventmodel.json"))
        })
        .map(|path| {
            fs::read_to_string(path)
                .map_err(ShellError::io)
                .and_then(|contents| {
                    formal_slice_artifact_paths(&contents, artifact_directory, extension)
                })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    require_only_modeled_artifacts(artifact_directory, extension, &allowed_paths, message)
}

fn formal_slice_artifact_paths(
    workflow_contents: &str,
    artifact_directory: &str,
    extension: &str,
) -> Result<Vec<ProjectPath>, ShellError> {
    workflow_slice_details(workflow_contents)?
        .into_iter()
        .map(|slice| {
            ProjectPath::try_new(format!(
                "{}/{}{}",
                artifact_directory,
                module_name_from_raw(slice.name().as_ref()),
                extension
            ))
            .map_err(ShellError::project_path)
        })
        .collect()
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

fn require_referenced_slice_file_identities(
    workflows_path: &str,
    message: &str,
) -> Result<(), ShellError> {
    referenced_slice_file_names(workflows_path)?
        .into_iter()
        .find_map(|file_name| {
            canonical_slice_file_name(&file_name)
                .is_some_and(|canonical_file_name| canonical_file_name != file_name)
                .then_some(file_name)
        })
        .map_or(Ok(()), |file_name| {
            Err(ShellError::message(format!("{message} for {file_name}")))
        })
}

fn canonical_slice_file_name(file_name: &str) -> Option<String> {
    file_name
        .strip_suffix(".eventmodel.json")
        .and_then(|stem| parse_slice_slug(stem).ok())
        .map(|slug| format!("{}.eventmodel.json", slug.as_ref()))
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
    workflow_document(workflow_contents)
        .and_then(|workflow| {
            workflow
                .slice_files()
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .map(|slice_files| {
            slice_files
                .into_iter()
                .map(|slice_file| slice_file.as_ref().to_owned())
                .filter_map(|slice_file| slice_file.rsplit('/').next().map(str::to_owned))
                .collect()
        })
}

fn require_clean_review_record(
    path: &str,
    workflow_path: &str,
    fallback_message: &str,
) -> Result<(), ShellError> {
    let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
    let record_contents = FileContents::try_new(contents)
        .map_err(|_error| ShellError::message(fallback_message.to_owned()))?;
    let record = ReviewRecordDocument::parse(&record_contents)
        .map_err(|_error| ShellError::message(fallback_message.to_owned()))?;
    let expected_workflow_slug = review_record_workflow_slug(path)?;
    if !record.matches_workflow(&expected_workflow_slug) {
        let observed = record
            .workflow_slug()
            .map_or_else(String::new, |workflow_slug| {
                workflow_slug.as_ref().to_owned()
            });
        return Err(ShellError::message(format!(
            "review record workflow '{observed}' does not match '{expected_workflow_slug}'"
        )));
    }
    let current_digest = model_content_digest(workflow_path)?;
    if !record.is_clean() {
        if record.current_mandatory_findings_include(&current_digest) {
            return Err(ShellError::message(
                "mandatory review findings remain for current model digest",
            ));
        }
        if record.has_mandatory_findings() && !record.model_content_digest_matches(&current_digest)
        {
            return Err(ShellError::message(
                "corrected workflow requires clean follow-up review",
            ));
        }
        return Err(ShellError::message(fallback_message.to_owned()));
    }
    if !record.model_content_digest_matches(&current_digest) {
        return Err(ShellError::message(
            "clean review is stale for current model digest",
        ));
    }
    if !record.has_category_results() {
        return Err(ShellError::message(fallback_message.to_owned()));
    }

    let required_categories = required_review_categories()?;
    match record.first_non_clean_category(&required_categories) {
        Some(ReviewCategoryFinding::NotClean(category)) => Err(ShellError::message(format!(
            "review category '{category}' is not clean"
        ))),
        Some(ReviewCategoryFinding::Missing(category)) => Err(ShellError::message(format!(
            "clean review is missing category '{category}'"
        ))),
        None => Ok(()),
    }
}

fn review_record_workflow_slug(path: &str) -> Result<WorkflowSlug, ShellError> {
    Path::new(path)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .and_then(|file_name| file_name.strip_suffix(".review.json"))
        .ok_or_else(|| ShellError::message("review record path is invalid"))
        .and_then(|slug| {
            WorkflowSlug::try_new(slug.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })
}

fn required_review_categories() -> Result<Vec<ReviewRuleName>, ShellError> {
    REQUIRED_REVIEW_CATEGORIES
        .iter()
        .map(|category| {
            ReviewRuleName::try_new((*category).to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .collect()
}

fn model_content_digest(workflow_path: &str) -> Result<ArtifactDigest, ShellError> {
    let workflow_contents = fs::read_to_string(Path::new(workflow_path)).map_err(ShellError::io)?;
    let slice_files = workflow_slice_file_paths(workflow_path, &workflow_contents)?;
    let mut digest = StableDigest::new();
    digest.write(workflow_path);
    digest.write(&workflow_contents);
    slice_files.into_iter().try_for_each(|slice_file| {
        let slice_path = slice_file.to_string_lossy().into_owned();
        let slice_contents = fs::read_to_string(&slice_file).map_err(ShellError::io)?;
        digest.write(&slice_path);
        digest.write(&slice_contents);
        Ok::<(), ShellError>(())
    })?;
    ArtifactDigest::try_new(digest.finish()).map_err(|error| ShellError::message(error.to_string()))
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
    let base_path = Path::new(workflow_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    workflow_document(workflow_contents)
        .and_then(|workflow| {
            workflow
                .slice_files()
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .map(|slice_files| {
            slice_files
                .into_iter()
                .map(|slice_file| base_path.join(slice_file.as_ref()))
                .collect()
        })
}

fn require_formal_slice_artifacts(
    workflow_contents: &str,
    artifact_directory: &str,
    extension: &str,
    message: &str,
) -> Result<(), ShellError> {
    workflow_slice_details(workflow_contents)?
        .into_iter()
        .try_for_each(|slice| {
            let module_name = module_name_from_raw(slice.name().as_ref());
            let artifact_path =
                Path::new(artifact_directory).join(format!("{module_name}{extension}"));
            let artifact_contents = if artifact_path.is_file() {
                fs::read_to_string(&artifact_path).map_err(ShellError::io)?
            } else {
                return Err(ShellError::message(message.to_owned()));
            };
            if formal_slice_artifact_is_canonical(
                &slice,
                &module_name,
                &artifact_contents,
                extension,
            )? {
                Ok(())
            } else {
                Err(ShellError::message(message.to_owned()))
            }
        })
}

fn formal_slice_artifact_is_canonical(
    slice: &WorkflowSliceDetail,
    module_name: &str,
    artifact_contents: &str,
    extension: &str,
) -> Result<bool, ShellError> {
    let digest = slice_artifact_digest(
        slice.name().clone(),
        slice.slug().clone(),
        slice.kind().clone(),
        slice.description().clone(),
    );
    let slice_name = json_string(slice.name().as_ref().to_owned())?;
    let slice_slug = json_string(slice.slug().as_ref().to_owned())?;
    let slice_kind = json_string(slice.kind().as_ref().to_owned())?;
    let slice_description = json_string(slice.description().as_ref().to_owned())?;
    let declarations = if extension == ".lean" {
        vec![
            ("namespace ", format!("namespace {module_name}")),
            (
                "-- EMC-DIGEST: ",
                format!("-- EMC-DIGEST: {}", digest.as_ref()),
            ),
            ("def sliceName :=", format!("def sliceName := {slice_name}")),
            ("def sliceSlug :=", format!("def sliceSlug := {slice_slug}")),
            ("def sliceKind :=", format!("def sliceKind := {slice_kind}")),
            (
                "def sliceDescription :=",
                format!("def sliceDescription := {slice_description}"),
            ),
            (
                "theorem sliceIdentityIsStable :",
                format!("theorem sliceIdentityIsStable : sliceName = {slice_name} := rfl"),
            ),
            ("end ", format!("end {module_name}")),
        ]
    } else {
        vec![
            ("module ", format!("module {module_name} {{")),
            (
                "// EMC-DIGEST: ",
                format!("// EMC-DIGEST: {}", digest.as_ref()),
            ),
            ("val sliceName =", format!("val sliceName = {slice_name}")),
            ("val sliceSlug =", format!("val sliceSlug = {slice_slug}")),
            ("val sliceKind =", format!("val sliceKind = {slice_kind}")),
            (
                "val sliceDescription =",
                format!("val sliceDescription = {slice_description}"),
            ),
            (
                "val sliceIdentityStable =",
                format!("val sliceIdentityStable = sliceName == {slice_name}"),
            ),
        ]
    };

    Ok(declarations.into_iter().all(|(prefix, marker)| {
        artifact_contains_one_canonical_declaration(artifact_contents, prefix, &marker)
    }))
}

fn workflow_transition_marker(prefix: &str, workflow_contents: &str) -> Result<String, ShellError> {
    let labels = if prefix.starts_with("val ") {
        workflow_transition_record_labels(workflow_contents)?
    } else {
        workflow_transition_lean_record_labels(workflow_contents)?
    };
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

fn artifact_contains_one_digest_marker(artifact_contents: &str, digest: &str) -> bool {
    let mut declarations = artifact_contents
        .lines()
        .filter_map(canonical_digest_marker_line);

    matches!(
        (declarations.next(), declarations.next()),
        (Some(declaration), None) if declaration == digest
    )
}

fn canonical_digest_marker_line(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("-- EMC-DIGEST: ")
        .or_else(|| trimmed.strip_prefix("// EMC-DIGEST: "))
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
    workflow_slice_details(workflow_contents)?
        .into_iter()
        .map(|slice| json_string(slice.slug().as_ref().to_owned()))
        .collect()
}

fn workflow_slice_detail_tuple_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    workflow_slice_details(workflow_contents)?
        .into_iter()
        .map(|slice| {
            Ok(format!(
                "({}, {}, {}, {})",
                json_string(slice.slug().as_ref().to_owned())?,
                json_string(slice.name().as_ref().to_owned())?,
                json_string(slice.kind().as_ref().to_owned())?,
                json_string(slice.description().as_ref().to_owned())?
            ))
        })
        .collect()
}

fn workflow_slice_detail_record_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    workflow_slice_details(workflow_contents)?
        .into_iter()
        .map(|slice| {
            Ok(format!(
                "{{ slug: {}, name: {}, kind: {}, description: {} }}",
                json_string(slice.slug().as_ref().to_owned())?,
                json_string(slice.name().as_ref().to_owned())?,
                json_string(slice.kind().as_ref().to_owned())?,
                json_string(slice.description().as_ref().to_owned())?
            ))
        })
        .collect()
}

fn workflow_slice_details(workflow_contents: &str) -> Result<Vec<WorkflowSliceDetail>, ShellError> {
    workflow_document(workflow_contents).and_then(|workflow| {
        workflow
            .slice_details()
            .map_err(|error| ShellError::message(error.to_string()))
    })
}

fn workflow_transition_lean_record_labels(
    workflow_contents: &str,
) -> Result<Vec<String>, ShellError> {
    workflow_transitions(workflow_contents)?
        .into_iter()
        .map(|transition| {
            Ok(format!(
                "{{ source := {}, target := {}, kind := {}, trigger := {} }}",
                json_string(transition.source().as_ref().to_owned())?,
                json_string(transition.target().as_ref().to_owned())?,
                json_string(transition.kind().as_ref().to_owned())?,
                json_string(transition.trigger().as_ref().to_owned())?
            ))
        })
        .collect()
}

fn workflow_transition_record_labels(workflow_contents: &str) -> Result<Vec<String>, ShellError> {
    workflow_transitions(workflow_contents)?
        .into_iter()
        .map(|transition| {
            Ok(format!(
                "{{ source: {}, target: {}, kind: {}, trigger: {} }}",
                json_string(transition.source().as_ref().to_owned())?,
                json_string(transition.target().as_ref().to_owned())?,
                json_string(transition.kind().as_ref().to_owned())?,
                json_string(transition.trigger().as_ref().to_owned())?
            ))
        })
        .collect()
}

fn workflow_transitions(
    workflow_contents: &str,
) -> Result<Vec<WorkflowTransitionRecord>, ShellError> {
    workflow_document(workflow_contents).and_then(|workflow| {
        workflow
            .transitions()
            .map_err(|error| ShellError::message(error.to_string()))
    })
}

fn workflow_document(workflow_contents: &str) -> Result<WorkflowDocument, ShellError> {
    FileContents::try_new(workflow_contents.to_owned())
        .map_err(|error| ShellError::message(error.to_string()))
        .and_then(|contents| {
            WorkflowDocument::parse(&contents)
                .map_err(|error| ShellError::message(error.to_string()))
        })
}

fn module_name_from_raw(raw: &str) -> String {
    let mut capitalize_next = true;
    raw.chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                let next = if capitalize_next {
                    character.to_ascii_uppercase()
                } else {
                    character
                };
                capitalize_next = false;
                Some(next)
            } else {
                capitalize_next = true;
                None
            }
        })
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

fn run_process(invocation: &ProcessInvocation) -> Result<Vec<String>, ShellError> {
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
        Ok(vec![invocation.success().as_ref().to_owned()])
    } else {
        Err(ShellError::message(format!(
            "{} failed with {}. Run `emc check` to confirm generated artifacts are synchronized, then run `emc verify` again",
            verification_label(invocation),
            status
        )))
    }
}

fn verification_label(invocation: &ProcessInvocation) -> &str {
    if invocation.success().as_ref().starts_with("Lean4") {
        "Lean4 verification"
    } else if invocation.success().as_ref().starts_with("Quint") {
        "Quint verification"
    } else {
        "verification command"
    }
}
