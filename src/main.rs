use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use emc::core::connection::{WorkflowConnection, connect_workflow};
use emc::core::emc::{EMCWorkflowImport, import_emc_workflow};
use emc::core::effect::FileContents;
use emc::core::effect::ProjectPath;
use emc::core::gherkin::{GherkinSuite, list_gherkin_features, run_gherkin_suite};
use emc::core::layout::{check_project, list_workflows, show_workflow};
use emc::core::project::{ProjectName, init_project};
use emc::core::review_gate::review_gate;
use emc::core::site::generate_site;
use emc::core::slice::{NewSlice, add_slice};
use emc::core::types::{ModelDescription, WorkflowSlug};
use emc::core::verify::verify_project;
use emc::core::workflow::{NewWorkflow, add_workflow, update_workflow_description};
use emc::event_model_validation::validate_target;
use emc::io::dto::{
    parse_browser_index_workflows, parse_connection_kind, parse_emc_slice_import,
    parse_emc_workflow_import, parse_gherkin_suite, parse_model_description, parse_model_name,
    parse_project_manifest_name, parse_slice_kind, parse_slice_slug, parse_transition_trigger_name,
    parse_workflow_slug,
};
use emc::mcp::{serve_http, serve_stdio};
use emc::shell::{ShellError, interpret};

struct Cli {
    command: Command,
}

enum Command {
    AddSlice {
        slice: NewSlice,
    },
    AddWorkflow {
        workflow: NewWorkflow,
    },
    Check,
    ConnectWorkflow {
        connection: WorkflowConnection,
    },
    GenerateSite {
        output: ProjectPath,
    },
    GherkinList {
        suite: GherkinSuite,
    },
    GherkinRun {
        suite: GherkinSuite,
    },
    ImportEMC {
        source: PathBuf,
    },
    Init {
        name: String,
    },
    ListWorkflows,
    McpStdio,
    McpHttp {
        host: String,
        port: u16,
        once: bool,
    },
    ReviewGate {
        slug: WorkflowSlug,
    },
    ShowWorkflow {
        slug: WorkflowSlug,
    },
    UpdateWorkflowDescription {
        slug: WorkflowSlug,
        description: ModelDescription,
    },
    Validate {
        target: PathBuf,
    },
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
        Command::AddSlice { slice } => {
            let workflow_document = fs::read_to_string(format!(
                "model/browser/data/workflows/{}.eventmodel.json",
                slice.workflow_slug().as_ref()
            ))
            .map_err(|error| ShellError::message(error.to_string()))
            .and_then(|contents| {
                FileContents::try_new(contents)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
            let plan = add_slice(workflow_document, slice)
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret(plan)
        }
        Command::AddWorkflow { workflow } => {
            let index = fs::read_to_string("model/browser/data/index.json")
                .map_err(|error| ShellError::message(error.to_string()))?;
            let existing_workflows = parse_browser_index_workflows(&index)
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret(add_workflow(existing_workflows, workflow))
        }
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
        Command::ConnectWorkflow { connection } => {
            let workflow_document = fs::read_to_string(format!(
                "model/browser/data/workflows/{}.eventmodel.json",
                connection.workflow_slug().as_ref()
            ))
            .map_err(|error| ShellError::message(error.to_string()))
            .and_then(|contents| {
                FileContents::try_new(contents)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
            let plan = connect_workflow(workflow_document, connection)
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret(plan)
        }
        Command::GenerateSite { output } => {
            let manifest = fs::read_to_string("emc.toml")
                .map_err(|error| ShellError::message(error.to_string()))?;
            let project_name =
                parse_project_manifest_name(&manifest).map_err(ShellError::project_name)?;
            interpret(generate_site(project_name, output))
        }
        Command::GherkinList { suite } => interpret(list_gherkin_features(suite)),
        Command::GherkinRun { suite } => interpret(run_gherkin_suite(suite)),
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
        Command::McpHttp { host, port, once } => serve_http(&host, port, once),
        Command::McpStdio => serve_stdio(),
        Command::ReviewGate { slug } => interpret(review_gate(slug)),
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
        Command::UpdateWorkflowDescription { slug, description } => {
            let index = fs::read_to_string("model/browser/data/index.json")
                .map_err(|error| ShellError::message(error.to_string()))?;
            let existing_workflows = parse_browser_index_workflows(&index)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let plan = update_workflow_description(existing_workflows, slug, description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            interpret(plan)
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
        [
            command,
            subject,
            workflow_flag,
            workflow,
            slug_flag,
            slug,
            name_flag,
            name,
            type_flag,
            slice_type,
            description_flag,
            description,
        ] if command == "add"
            && subject == "slice"
            && workflow_flag == "--workflow"
            && slug_flag == "--slug"
            && name_flag == "--name"
            && type_flag == "--type"
            && description_flag == "--description" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_kind = parse_slice_kind(slice_type)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let slice_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddSlice {
                    slice: NewSlice::new(
                        workflow_slug,
                        slice_slug,
                        slice_name,
                        slice_description,
                        slice_kind,
                    ),
                },
            })
        }
        [
            command,
            subject,
            slug_flag,
            slug,
            name_flag,
            name,
            description_flag,
            description,
        ] if command == "add"
            && subject == "workflow"
            && slug_flag == "--slug"
            && name_flag == "--name"
            && description_flag == "--description" =>
        {
            let workflow_slug = parse_workflow_slug(slug)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::AddWorkflow {
                    workflow: NewWorkflow::new(workflow_name, workflow_description, workflow_slug),
                },
            })
        }
        [command] if command == "check" => Ok(Cli {
            command: Command::Check,
        }),
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
        ] if command == "connect"
            && subject == "workflow"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_flag == "--to"
            && via_flag == "--via"
            && name_flag == "--name" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug =
                parse_slice_slug(target).map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::ConnectWorkflow {
                    connection: WorkflowConnection::new(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                    ),
                },
            })
        }
        [command, subject, output_flag, output]
            if command == "generate" && subject == "site" && output_flag == "--output" =>
        {
            ProjectPath::try_new(output.clone())
                .map(|output| Cli {
                    command: Command::GenerateSite { output },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, suite_flag, suite]
            if command == "gherkin" && subject == "list" && suite_flag == "--suite" =>
        {
            parse_gherkin_suite(suite)
                .map(|suite| Cli {
                    command: Command::GherkinList { suite },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, suite_flag, suite]
            if command == "gherkin" && subject == "run" && suite_flag == "--suite" =>
        {
            parse_gherkin_suite(suite)
                .map(|suite| Cli {
                    command: Command::GherkinRun { suite },
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
        [command, transport] if command == "mcp" && transport == "http" => Ok(Cli {
            command: Command::McpHttp {
                host: "127.0.0.1".to_owned(),
                port: 7331,
                once: false,
            },
        }),
        [
            command,
            transport,
            host_flag,
            host,
            port_flag,
            port,
            once_flag,
        ] if command == "mcp"
            && transport == "http"
            && host_flag == "--host"
            && port_flag == "--port"
            && once_flag == "--once" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: true,
                },
            })
        }
        [command, subject, workflow_flag, workflow]
            if command == "review" && subject == "gate" && workflow_flag == "--workflow" =>
        {
            parse_workflow_slug(workflow)
                .map(|slug| Cli {
                    command: Command::ReviewGate { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, slug] if command == "show" && subject == "workflow" => {
            parse_workflow_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowWorkflow { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [
            command,
            subject,
            slug_flag,
            slug,
            description_flag,
            description,
        ] if command == "update"
            && subject == "workflow"
            && slug_flag == "--slug"
            && description_flag == "--description" =>
        {
            let workflow_slug = parse_workflow_slug(slug)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateWorkflowDescription {
                    slug: workflow_slug,
                    description: workflow_description,
                },
            })
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

fn parse_port(port: &str) -> Result<u16, ShellError> {
    port.parse::<u16>()
        .map_err(|error| ShellError::message(format!("invalid MCP HTTP port: {error}")))
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
