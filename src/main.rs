use std::env;
use std::process::ExitCode;

use emc::core::connection::WorkflowConnection;
use emc::core::effect::{Effect, EffectPlan, ProjectPath};
use emc::core::gherkin::{
    GherkinSuite, list_gherkin_features, run_all_gherkin_suites, run_gherkin_suite,
};
use emc::core::project::{ProjectName, init_project};
use emc::core::review_gate::review_gate;
use emc::core::slice::NewSlice;
use emc::core::types::{ModelDescription, WorkflowSlug};
use emc::core::workflow::NewWorkflow;
use emc::io::dto::{
    parse_connection_kind, parse_gherkin_suite, parse_model_description, parse_model_name,
    parse_slice_kind, parse_slice_slug, parse_transition_trigger_name, parse_workflow_slug,
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
    GherkinRunAll,
    Init {
        name: String,
    },
    ListWorkflows,
    McpStdio,
    McpHttp {
        host: String,
        port: u16,
        once: bool,
        auth_token: Option<String>,
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
        target: ProjectPath,
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
            interpret(EffectPlan::new(vec![Effect::AddSliceFromWorkflow(slice)]))
        }
        Command::AddWorkflow { workflow } => {
            interpret(EffectPlan::new(vec![Effect::AddWorkflowFromIndex(
                workflow,
            )]))
        }
        Command::Check => interpret(EffectPlan::new(vec![Effect::CheckCurrentProject])),
        Command::ConnectWorkflow { connection } => {
            interpret(EffectPlan::new(vec![Effect::ConnectWorkflowFromWorkflow(
                connection,
            )]))
        }
        Command::GenerateSite { output } => {
            interpret(EffectPlan::new(vec![Effect::GenerateSiteFromManifest(
                output,
            )]))
        }
        Command::GherkinList { suite } => interpret(list_gherkin_features(suite)),
        Command::GherkinRunAll => interpret(run_all_gherkin_suites()),
        Command::GherkinRun { suite } => interpret(run_gherkin_suite(suite)),
        Command::Init { name } => {
            let project_name = ProjectName::try_new(name).map_err(ShellError::project_name)?;
            interpret(init_project(project_name))
        }
        Command::ListWorkflows => interpret(EffectPlan::new(vec![Effect::ListWorkflowsFromIndex])),
        Command::McpHttp {
            host,
            port,
            once,
            auth_token,
        } => serve_http(&host, port, once, auth_token.as_deref()),
        Command::McpStdio => serve_stdio(),
        Command::ReviewGate { slug } => interpret(review_gate(slug)),
        Command::ShowWorkflow { slug } => {
            interpret(EffectPlan::new(vec![Effect::ShowWorkflowFromWorkflow(
                slug,
            )]))
        }
        Command::UpdateWorkflowDescription { slug, description } => {
            interpret(EffectPlan::new(vec![
                Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(slug, description),
            ]))
        }
        Command::Validate { target } => {
            interpret(EffectPlan::new(vec![Effect::ValidateEventModelTarget(
                target,
            )]))
        }
        Command::Verify => interpret(EffectPlan::new(vec![Effect::VerifyProjectFromIndex])),
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
        [
            command,
            subject,
            workflow_flag,
            workflow,
            from_flag,
            source,
            to_workflow_flag,
            target,
            via_flag,
            via,
            name_flag,
            name,
            reason_flag,
            reason,
        ] if command == "connect"
            && subject == "workflow"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_workflow_flag == "--to-workflow"
            && via_flag == "--via"
            && name_flag == "--name"
            && reason_flag == "--reason" =>
        {
            let workflow_slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let source_slug =
                parse_slice_slug(source).map_err(|error| ShellError::message(error.to_string()))?;
            let target_slug = parse_workflow_slug(target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let connection_kind = parse_connection_kind(via)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let trigger = parse_transition_trigger_name(name)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let exit_reason = parse_model_description(reason)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::ConnectWorkflow {
                    connection: WorkflowConnection::new_workflow_exit(
                        workflow_slug,
                        source_slug,
                        target_slug,
                        connection_kind,
                        trigger,
                        exit_reason,
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
        [command, subject, all_flag]
            if command == "gherkin" && subject == "run" && all_flag == "--all" =>
        {
            Ok(Cli {
                command: Command::GherkinRunAll,
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
                auth_token: None,
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
                    auth_token: None,
                },
            })
        }
        [
            command,
            transport,
            host_flag,
            host,
            port_flag,
            port,
            auth_flag,
            auth_token,
            once_flag,
        ] if command == "mcp"
            && transport == "http"
            && host_flag == "--host"
            && port_flag == "--port"
            && auth_flag == "--auth-token"
            && once_flag == "--once" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: true,
                    auth_token: Some(auth_token.clone()),
                },
            })
        }
        [command, transport, host_flag, host, port_flag, port]
            if command == "mcp"
                && transport == "http"
                && host_flag == "--host"
                && port_flag == "--port" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: false,
                    auth_token: None,
                },
            })
        }
        [
            command,
            transport,
            host_flag,
            host,
            port_flag,
            port,
            auth_flag,
            auth_token,
        ] if command == "mcp"
            && transport == "http"
            && host_flag == "--host"
            && port_flag == "--port"
            && auth_flag == "--auth-token" =>
        {
            Ok(Cli {
                command: Command::McpHttp {
                    host: host.clone(),
                    port: parse_port(port)?,
                    once: false,
                    auth_token: Some(auth_token.clone()),
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
                target: ProjectPath::try_new(target.clone())
                    .map_err(|error| ShellError::message(error.to_string()))?,
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
