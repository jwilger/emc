use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use emc::core::emc::{EMCWorkflowImport, import_emc_workflow};
use emc::core::layout::{check_project, list_workflows};
use emc::core::project::{ProjectName, init_project};
use emc::core::validation::{
    EventModelDocument, EventModelFileKind, validate_event_model, validate_event_model_corpus,
};
use emc::io::dto::{
    parse_browser_index_workflows, parse_emc_slice_import, parse_emc_workflow_import,
    parse_event_model_document, parse_project_manifest_name, parse_slice_slug, parse_workflow_slug,
};
use emc::shell::{ShellError, interpret};
use serde_json::Value;

struct Cli {
    command: Command,
}

enum Command {
    Check,
    ImportEMC { source: PathBuf },
    Init { name: String },
    ListWorkflows,
    Validate { target: PathBuf },
}

fn main() -> ExitCode {
    match parse_cli(env::args().skip(1).collect()).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), ShellError> {
    match cli.command {
        Command::Check => {
            let manifest = fs::read_to_string("emc.toml")
                .map_err(|error| ShellError::message(error.to_string()))?;
            let project_name =
                parse_project_manifest_name(&manifest).map_err(ShellError::project_name)?;
            let index = fs::read_to_string("model/browser/data/index.json")
                .map_err(|error| ShellError::message(error.to_string()))?;
            let imported_workflows = parse_browser_index_workflows(&index)
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret(check_project(project_name, imported_workflows))
        }
        Command::ImportEMC { source } => {
            interpret(import_emc_workflow(load_emc_import(&source)?))
        }
        Command::Init { name } => {
            let project_name = ProjectName::try_new(name).map_err(ShellError::project_name)?;
            interpret(init_project(project_name))
        }
        Command::ListWorkflows => {
            let index = fs::read_to_string("model/browser/data/index.json")
                .map_err(|error| ShellError::message(error.to_string()))?;
            let imported_workflows = parse_browser_index_workflows(&index)
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret(list_workflows(imported_workflows))
        }
        Command::Validate { target } => validate_target(&target),
    }
}

fn parse_cli(arguments: Vec<String>) -> Result<Cli, ShellError> {
    match arguments.as_slice() {
        [command] if command == "check" => Ok(Cli {
            command: Command::Check,
        }),
        [command, kind, source_flag, source]
            if command == "import" && kind == "emc" && source_flag == "--source" =>
        {
            Ok(Cli {
                command: Command::ImportEMC {
                    source: PathBuf::from(source),
                },
            })
        }
        [command, name_flag, name] if command == "init" && name_flag == "--name" => Ok(Cli {
            command: Command::Init { name: name.clone() },
        }),
        [command, subject] if command == "list" && subject == "workflows" => Ok(Cli {
            command: Command::ListWorkflows,
        }),
        [command, target] if command == "validate" => Ok(Cli {
            command: Command::Validate {
                target: PathBuf::from(target),
            },
        }),
        _ => Err(ShellError::message("usage: emc init --name <project-name>")),
    }
}

fn validate_target(target: &Path) -> Result<(), ShellError> {
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

fn load_emc_import(source: &Path) -> Result<EMCWorkflowImport, ShellError> {
    let workflow_path = first_event_model_file(&source.join("workflows"))?;
    let workflow_slug = file_slug(&workflow_path).and_then(|slug| {
        parse_workflow_slug(&slug).map_err(|error| ShellError::message(error.to_string()))
    })?;
    let workflow_json = fs::read_to_string(&workflow_path)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let slices = event_model_files(&source.join("slices"))?
        .into_iter()
        .map(|slice_path| {
            let slice_slug = file_slug(&slice_path).and_then(|slug| {
                parse_slice_slug(&slug).map_err(|error| ShellError::message(error.to_string()))
            })?;
            let slice_json = fs::read_to_string(&slice_path)
                .map_err(|error| ShellError::message(error.to_string()))?;
            parse_emc_slice_import(slice_slug, &slice_json)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    parse_emc_workflow_import(workflow_slug, &workflow_json, slices)
        .map_err(|error| ShellError::message(error.to_string()))
}

fn first_event_model_file(directory: &Path) -> Result<PathBuf, ShellError> {
    event_model_files(directory)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            ShellError::message(format!(
                "no *.eventmodel.json files in {}",
                directory.display()
            ))
        })
}

fn event_model_files(directory: &Path) -> Result<Vec<PathBuf>, ShellError> {
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

fn file_slug(path: &Path) -> Result<String, ShellError> {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .and_then(|file_name| file_name.strip_suffix(".eventmodel.json"))
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ShellError::message(format!("invalid event model file name {}", path.display()))
        })
}
