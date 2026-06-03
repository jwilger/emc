use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::FileContents;
use crate::core::types::BoardLaneId;

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
