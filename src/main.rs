use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use emc::core::emc::{EMCWorkflowImport, import_emc_workflow};
use emc::core::effect::FileContents;
use emc::core::effect::ProjectPath;
use emc::core::layout::{check_project, list_workflows, show_workflow};
use emc::core::project::{ProjectName, init_project};
use emc::core::site::generate_site;
use emc::core::types::WorkflowSlug;
use emc::core::verify::verify_project;
use emc::event_model_validation::validate_target;
use emc::io::dto::{
    parse_browser_index_workflows, parse_emc_slice_import, parse_emc_workflow_import,
    parse_project_manifest_name, parse_slice_slug, parse_workflow_slug,
};
use emc::mcp::serve_stdio;
use emc::shell::{ShellError, interpret};

struct Cli {
    command: Command,
}

enum Command {
    Check,
    GenerateSite { output: ProjectPath },
    ImportEMC { source: PathBuf },
    Init { name: String },
    ListWorkflows,
    McpStdio,
    ShowWorkflow { slug: WorkflowSlug },
    Validate { target: PathBuf },
    Verify,
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
        Command::GenerateSite { output } => interpret(generate_site(output)),
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
        Command::McpStdio => serve_stdio(),
        Command::ShowWorkflow { slug } => {
            let workflow_path = format!(
                "model/browser/data/workflows/{}.eventmodel.json",
                slug.as_ref()
            );
            let workflow_document = fs::read_to_string(workflow_path)
                .map_err(|error| ShellError::message(error.to_string()))
                .and_then(|contents| {
                    FileContents::try_new(contents)
                        .map_err(|error| ShellError::message(error.to_string()))
                })?;
            interpret(show_workflow(workflow_document))
        }
        Command::Validate { target } => validate_target(&target),
        Command::Verify => {
            let index = fs::read_to_string("model/browser/data/index.json")
                .map_err(|error| ShellError::message(error.to_string()))?;
            let imported_workflows = parse_browser_index_workflows(&index)
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret(verify_project(imported_workflows))
        }
    }
}

fn parse_cli(arguments: Vec<String>) -> Result<Cli, ShellError> {
    match arguments.as_slice() {
        [command] if command == "check" => Ok(Cli {
            command: Command::Check,
        }),
        [command, subject, output_flag, output]
            if command == "generate" && subject == "site" && output_flag == "--output" =>
        {
            ProjectPath::try_new(output.clone())
                .map(|output| Cli {
                    command: Command::GenerateSite { output },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
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
        [command, transport] if command == "mcp" && transport == "stdio" => Ok(Cli {
            command: Command::McpStdio,
        }),
        [command, subject, slug] if command == "show" && subject == "workflow" => {
            parse_workflow_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowWorkflow { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, target] if command == "validate" => Ok(Cli {
            command: Command::Validate {
                target: PathBuf::from(target),
            },
        }),
        [command] if command == "verify" => Ok(Cli {
            command: Command::Verify,
        }),
        _ => Err(ShellError::message("usage: emc init --name <project-name>")),
    }
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
