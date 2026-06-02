use std::env;
use std::fs;
use std::process::ExitCode;

use emc::core::layout::check_project;
use emc::core::project::{ProjectName, init_project};
use emc::io::dto::parse_project_manifest_name;
use emc::shell::{ShellError, interpret};

struct Cli {
    command: Command,
}

enum Command {
    Check,
    Init { name: String },
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
            interpret(check_project(project_name))
        }
        Command::Init { name } => {
            let project_name = ProjectName::try_new(name).map_err(ShellError::project_name)?;
            interpret(init_project(project_name))
        }
    }
}

fn parse_cli(arguments: Vec<String>) -> Result<Cli, ShellError> {
    match arguments.as_slice() {
        [command] if command == "check" => Ok(Cli {
            command: Command::Check,
        }),
        [command, name_flag, name] if command == "init" && name_flag == "--name" => Ok(Cli {
            command: Command::Init { name: name.clone() },
        }),
        _ => Err(ShellError::message("usage: emc init --name <project-name>")),
    }
}
