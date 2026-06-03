use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::FileContents;
use crate::core::types::{
    BoardLaneId, BrowserErrorRecoveryDetail, BrowserEventElementName, CommandErrorName, ViewName,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserDataDocument {
    value: Value,
}

impl BrowserDataDocument {
    pub fn parse(contents: &FileContents) -> Result<Self, BrowserDataDocumentError> {
        let value = serde_json::from_str::<Value>(contents.as_ref()).map_err(|error| {
            BrowserDataDocumentError::new(format!("invalid browser data JSON: {error}"))
        })?;
        value.as_object().ok_or_else(|| {
            BrowserDataDocumentError::new("browser data document must be an object")
        })?;
        Ok(Self { value })
    }

    pub fn board_lane_ids(&self) -> Result<Vec<BoardLaneId>, BrowserDataDocumentError> {
        self.value
            .get("board")
            .and_then(|board| board.get("lanes"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|lane| lane.get("id").and_then(Value::as_str))
            .map(board_lane_id)
            .collect()
    }

    pub fn event_element_names(
        &self,
    ) -> Result<Vec<BrowserEventElementName>, BrowserDataDocumentError> {
        self.value
            .get("board")
            .and_then(|board| board.get("slices"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|slice| {
                slice
                    .get("elements")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter(|element| {
                element
                    .get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| kind == "event")
            })
            .filter_map(|element| element.get("name").and_then(Value::as_str))
            .map(event_element_name)
            .collect()
    }

    pub fn error_recovery_details(
        &self,
    ) -> Result<Vec<BrowserErrorRecoveryDetail>, BrowserDataDocumentError> {
        self.value
            .get("views")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(view_error_recovery_details)
            .collect::<Result<Vec<_>, _>>()
            .map(|details| details.into_iter().flatten().collect())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BrowserDataDocumentError {
    message: String,
}

impl BrowserDataDocumentError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for BrowserDataDocumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BrowserDataDocumentError {}

fn board_lane_id(raw: &str) -> Result<BoardLaneId, BrowserDataDocumentError> {
    BoardLaneId::try_new(raw.to_owned())
        .map_err(|error| BrowserDataDocumentError::new(format!("invalid board lane id: {error}")))
}

fn event_element_name(raw: &str) -> Result<BrowserEventElementName, BrowserDataDocumentError> {
    BrowserEventElementName::try_new(raw.to_owned()).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid event element name: {error}"))
    })
}

fn view_error_recovery_details(
    view: &Value,
) -> Result<Vec<BrowserErrorRecoveryDetail>, BrowserDataDocumentError> {
    let source_screen = view.get("name").and_then(Value::as_str);

    view.get("controls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|control| {
            control
                .get("error_handling")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|handling| {
            Some((
                source_screen?,
                handling.get("error").and_then(Value::as_str)?,
            ))
        })
        .map(|(source_screen, error_name)| {
            Ok(BrowserErrorRecoveryDetail::new(
                command_error_name(error_name)?,
                view_name(source_screen)?,
            ))
        })
        .collect()
}

fn command_error_name(raw: &str) -> Result<CommandErrorName, BrowserDataDocumentError> {
    CommandErrorName::try_new(raw.to_owned()).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid command error name: {error}"))
    })
}

fn view_name(raw: &str) -> Result<ViewName, BrowserDataDocumentError> {
    ViewName::try_new(raw.to_owned())
        .map_err(|error| BrowserDataDocumentError::new(format!("invalid view name: {error}")))
}
