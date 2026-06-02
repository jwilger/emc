use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io;
use std::path::Path;

use crate::core::effect::{Effect, EffectPlan};

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
    plan.effects().iter().try_for_each(interpret_effect)
}

fn interpret_effect(effect: &Effect) -> Result<(), ShellError> {
    match effect {
        Effect::EnsureDirectory(path) => {
            fs::create_dir_all(Path::new(path.as_ref())).map_err(ShellError::io)
        }
        Effect::WriteFile(path, contents) => {
            if let Some(parent) = Path::new(path.as_ref()).parent() {
                fs::create_dir_all(parent).map_err(ShellError::io)?;
            }
            fs::write(Path::new(path.as_ref()), contents.as_ref()).map_err(ShellError::io)
        }
        Effect::Report(line) => {
            println!("{}", line.as_ref());
            Ok(())
        }
    }
}
