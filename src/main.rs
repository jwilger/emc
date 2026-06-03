use std::env;
use std::process::ExitCode;

use clap::{Arg, Command as ClapCommand};
use emc::command;
use emc::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use emc::core::effect::ProjectPath;
use emc::core::gherkin::GherkinSuite;
use emc::core::project::ProjectName;
use emc::core::slice::{NewSlice, SliceKind};
use emc::core::types::{
    ModelDescription, ModelName, ReviewTimestamp, ReviewerId, SliceSlug, WorkflowSlug,
};
use emc::core::workflow::NewWorkflow;
use emc::io::dto::{
    parse_connection_kind, parse_gherkin_suite, parse_model_description, parse_model_name,
    parse_project_name, parse_project_path, parse_review_timestamp, parse_reviewer_id,
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
    Help,
    Init {
        name: ProjectName,
    },
    ListSlices,
    ListTransitions,
    ListWorkflows,
    McpStdio,
    McpHttp {
        host: String,
        port: u16,
        once: bool,
        auth_token: Option<String>,
    },
    RemoveTransition {
        removal: WorkflowTransitionRemoval,
    },
    RemoveSlice {
        slug: SliceSlug,
    },
    ReviewGate {
        slug: WorkflowSlug,
    },
    RecordCleanReview {
        slug: WorkflowSlug,
        reviewer: ReviewerId,
        reviewed_at: ReviewTimestamp,
    },
    RemoveWorkflow {
        slug: WorkflowSlug,
    },
    ShowSlice {
        slug: SliceSlug,
    },
    ShowWorkflow {
        slug: WorkflowSlug,
    },
    UpdateSliceDescription {
        slug: SliceSlug,
        description: ModelDescription,
    },
    UpdateSliceKind {
        slug: SliceSlug,
        kind: SliceKind,
    },
    UpdateSliceName {
        slug: SliceSlug,
        name: ModelName,
    },
    UpdateWorkflowDescription {
        slug: WorkflowSlug,
        description: ModelDescription,
    },
    UpdateWorkflowName {
        slug: WorkflowSlug,
        name: ModelName,
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
        Command::AddSlice { slice } => interpret(command::add_slice(slice)),
        Command::AddWorkflow { workflow } => interpret(command::add_workflow(workflow)),
        Command::Check => interpret(command::check_project()),
        Command::ConnectWorkflow { connection } => interpret(command::connect_workflow(connection)),
        Command::GenerateSite { output } => interpret(command::generate_site(output)),
        Command::GherkinList { suite } => interpret(command::gherkin_list(suite)),
        Command::GherkinRunAll => interpret(command::gherkin_run_all()),
        Command::GherkinRun { suite } => interpret(command::gherkin_run(suite)),
        Command::Help => print_help(),
        Command::Init { name } => interpret(command::init(name)),
        Command::ListSlices => interpret(command::list_slices()),
        Command::ListTransitions => interpret(command::list_transitions()),
        Command::ListWorkflows => interpret(command::list_workflows()),
        Command::McpHttp {
            host,
            port,
            once,
            auth_token,
        } => serve_http(&host, port, once, auth_token.as_deref()),
        Command::McpStdio => serve_stdio(),
        Command::ReviewGate { slug } => interpret(command::review_gate_for_workflow(slug)),
        Command::RecordCleanReview {
            slug,
            reviewer,
            reviewed_at,
        } => interpret(command::record_clean_review(slug, reviewer, reviewed_at)),
        Command::RemoveSlice { slug } => interpret(command::remove_slice(slug)),
        Command::RemoveTransition { removal } => interpret(command::remove_transition(removal)),
        Command::RemoveWorkflow { slug } => interpret(command::remove_workflow(slug)),
        Command::ShowSlice { slug } => interpret(command::show_slice(slug)),
        Command::ShowWorkflow { slug } => interpret(command::show_workflow(slug)),
        Command::UpdateSliceDescription { slug, description } => {
            interpret(command::update_slice_description(slug, description))
        }
        Command::UpdateSliceKind { slug, kind } => {
            interpret(command::update_slice_kind(slug, kind))
        }
        Command::UpdateSliceName { slug, name } => {
            interpret(command::update_slice_name(slug, name))
        }
        Command::UpdateWorkflowDescription { slug, description } => {
            interpret(command::update_workflow_description(slug, description))
        }
        Command::UpdateWorkflowName { slug, name } => {
            interpret(command::update_workflow_name(slug, name))
        }
        Command::Validate { target } => interpret(command::validate(target)),
        Command::Verify => interpret(command::verify()),
    }
}

fn parse_cli(arguments: Vec<String>) -> Result<Cli, ShellError> {
    match arguments.as_slice() {
        [] => Ok(Cli {
            command: Command::Help,
        }),
        [flag] if flag == "--help" || flag == "-h" => Ok(Cli {
            command: Command::Help,
        }),
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
        ] if command == "remove"
            && subject == "transition"
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
                command: Command::RemoveTransition {
                    removal: WorkflowTransitionRemoval::new(
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
        ] if command == "remove"
            && subject == "transition"
            && workflow_flag == "--workflow"
            && from_flag == "--from"
            && to_workflow_flag == "--to-workflow"
            && via_flag == "--via"
            && name_flag == "--name" =>
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
            Ok(Cli {
                command: Command::RemoveTransition {
                    removal: WorkflowTransitionRemoval::new_workflow_exit(
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
            parse_project_path(output)
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
            command: Command::Init {
                name: parse_project_name(name)
                    .map_err(|error| ShellError::message(error.to_string()))?,
            },
        }),
        [command, subject] if command == "list" && subject == "workflows" => Ok(Cli {
            command: Command::ListWorkflows,
        }),
        [command, subject] if command == "list" && subject == "slices" => Ok(Cli {
            command: Command::ListSlices,
        }),
        [command, subject] if command == "list" && subject == "transitions" => Ok(Cli {
            command: Command::ListTransitions,
        }),
        [command, subject, slug_flag, slug]
            if command == "remove" && subject == "slice" && slug_flag == "--slug" =>
        {
            parse_slice_slug(slug)
                .map(|slug| Cli {
                    command: Command::RemoveSlice { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, slug_flag, slug]
            if command == "remove" && subject == "workflow" && slug_flag == "--slug" =>
        {
            parse_workflow_slug(slug)
                .map(|slug| Cli {
                    command: Command::RemoveWorkflow { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
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
        [
            command,
            subject,
            workflow_flag,
            workflow,
            reviewer_flag,
            reviewer,
            reviewed_at_flag,
            reviewed_at,
        ] if command == "review"
            && subject == "record"
            && workflow_flag == "--workflow"
            && reviewer_flag == "--reviewer"
            && reviewed_at_flag == "--reviewed-at" =>
        {
            let slug = parse_workflow_slug(workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let reviewer = parse_reviewer_id(reviewer)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let reviewed_at = parse_review_timestamp(reviewed_at)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::RecordCleanReview {
                    slug,
                    reviewer,
                    reviewed_at,
                },
            })
        }
        [command, subject, slug] if command == "show" && subject == "workflow" => {
            parse_workflow_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowWorkflow { slug },
                })
                .map_err(|error| ShellError::message(error.to_string()))
        }
        [command, subject, slug] if command == "show" && subject == "slice" => {
            parse_slice_slug(slug)
                .map(|slug| Cli {
                    command: Command::ShowSlice { slug },
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
            && subject == "slice"
            && slug_flag == "--slug"
            && description_flag == "--description" =>
        {
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_description = parse_model_description(description)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateSliceDescription {
                    slug: slice_slug,
                    description: slice_description,
                },
            })
        }
        [command, subject, slug_flag, slug, type_flag, slice_type]
            if command == "update"
                && subject == "slice"
                && slug_flag == "--slug"
                && type_flag == "--type" =>
        {
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_kind = parse_slice_kind(slice_type)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateSliceKind {
                    slug: slice_slug,
                    kind: slice_kind,
                },
            })
        }
        [command, subject, slug_flag, slug, name_flag, name]
            if command == "update"
                && subject == "slice"
                && slug_flag == "--slug"
                && name_flag == "--name" =>
        {
            let slice_slug =
                parse_slice_slug(slug).map_err(|error| ShellError::message(error.to_string()))?;
            let slice_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateSliceName {
                    slug: slice_slug,
                    name: slice_name,
                },
            })
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
        [command, subject, slug_flag, slug, name_flag, name]
            if command == "update"
                && subject == "workflow"
                && slug_flag == "--slug"
                && name_flag == "--name" =>
        {
            let workflow_slug = parse_workflow_slug(slug)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let workflow_name =
                parse_model_name(name).map_err(|error| ShellError::message(error.to_string()))?;
            Ok(Cli {
                command: Command::UpdateWorkflowName {
                    slug: workflow_slug,
                    name: workflow_name,
                },
            })
        }
        [command, target] if command == "validate" => Ok(Cli {
            command: Command::Validate {
                target: parse_project_path(target)
                    .map_err(|error| ShellError::message(error.to_string()))?,
            },
        }),
        [command] if command == "verify" => Ok(Cli {
            command: Command::Verify,
        }),
        _ => Err(ShellError::message("usage: emc init --name <project-name>")),
    }
}

fn print_help() -> Result<(), ShellError> {
    help_command()
        .print_help()
        .map_err(|error| ShellError::message(error.to_string()))?;
    println!();
    Ok(())
}

fn help_command() -> ClapCommand {
    ClapCommand::new("emc")
        .about("Event Model Compiler")
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .subcommand(
            ClapCommand::new("init")
                .about("Create a deterministic EMC project")
                .arg(Arg::new("name").long("name").value_name("PROJECT_NAME")),
        )
        .subcommand(
            ClapCommand::new("list").about("Read model indexes").subcommand(
                ClapCommand::new("workflows").about("List modeled workflows in the project"),
            )
            .subcommand(ClapCommand::new("slices").about("List modeled slices in the project"))
            .subcommand(
                ClapCommand::new("transitions")
                    .about("List modeled workflow transitions in the project"),
            ),
        )
        .subcommand(
            ClapCommand::new("show")
                .about("Read modeled artifacts")
                .subcommand(ClapCommand::new("workflow").about("Show a workflow by slug"))
                .subcommand(ClapCommand::new("slice").about("Show a slice by slug")),
        )
        .subcommand(
            ClapCommand::new("add")
                .about("Create modeled business artifacts")
                .subcommand(
                    ClapCommand::new("workflow")
                        .about("Add a workflow and synchronized formal artifacts"),
                )
                .subcommand(
                    ClapCommand::new("slice")
                        .about("Add a slice and synchronized formal artifacts"),
                ),
        )
        .subcommand(
            ClapCommand::new("update")
                .about("Modify modeled business artifacts")
                .subcommand(
                    ClapCommand::new("workflow")
                        .about("Update a workflow and synchronized formal artifacts"),
                )
                .subcommand(
                    ClapCommand::new("slice")
                        .about("Update a slice and synchronized formal artifacts"),
                ),
        )
        .subcommand(
            ClapCommand::new("connect")
                .about("Connect modeled workflow steps")
                .subcommand(
                    ClapCommand::new("workflow")
                        .about("Add a workflow transition and synchronized formal artifacts"),
                ),
        )
        .subcommand(ClapCommand::new("validate").about("Validate event-model JSON files"))
        .subcommand(ClapCommand::new("verify").about("Run Lean4 and Quint verification"))
        .subcommand(ClapCommand::new("check").about("Check project artifact synchronization"))
        .subcommand(
            ClapCommand::new("generate")
                .about("Generate derived artifacts")
                .subcommand(ClapCommand::new("site").about("Generate the browsable event-model site")),
        )
        .subcommand(
            ClapCommand::new("gherkin")
                .about("List or run checked-in event-model rule suites")
                .subcommand(ClapCommand::new("list").about("List configured feature files"))
                .subcommand(ClapCommand::new("run").about("Run configured rule-suite coverage")),
        )
        .subcommand(
            ClapCommand::new("review")
                .about("Evaluate review gates")
                .subcommand(ClapCommand::new("gate").about("Check a workflow review gate"))
                .subcommand(
                    ClapCommand::new("record").about("Record a clean workflow review"),
                ),
        )
        .subcommand(
            ClapCommand::new("mcp")
                .about("Serve EMC tools over MCP")
                .subcommand(ClapCommand::new("stdio").about("Serve MCP over stdio"))
                .subcommand(ClapCommand::new("http").about("Serve MCP over HTTP")),
        )
        .after_help(
            "Common commands:
  emc init --name <project-name>
  emc add workflow --slug <slug> --name <name> --description <text>
  emc update workflow --slug <workflow> --name <name>
  emc remove workflow --slug <workflow>
  emc add slice --workflow <workflow> --slug <slug> --name <name> --type <kind> --description <text>
  emc update slice --slug <slice> --description <text>
  emc update slice --slug <slice> --type <kind>
  emc update slice --slug <slice> --name <name>
  emc remove slice --slug <slice>
  emc connect workflow --workflow <workflow> --from <slice> --to <slice> --via <kind> --name <trigger>
  emc remove transition --workflow <workflow> --from <slice> --to <slice> --via <kind> --name <trigger>
  emc remove transition --workflow <workflow> --from <slice> --to-workflow <workflow> --via outcome --name <trigger>
  emc list slices
  emc list transitions
  emc validate <path>
  emc verify
  emc check
  emc generate site --output <directory>
  emc gherkin list --suite <suite>
  emc gherkin run --suite <suite>
  emc gherkin run --all
  emc review record --workflow <workflow> --reviewer <reviewer> --reviewed-at <timestamp>
  emc mcp stdio
  emc mcp http --host 127.0.0.1 --port 7331",
        )
}

fn parse_port(port: &str) -> Result<u16, ShellError> {
    port.parse::<u16>()
        .map_err(|error| ShellError::message(format!("invalid MCP HTTP port: {error}")))
}
