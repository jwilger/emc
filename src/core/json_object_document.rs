use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::FileContents;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsonObjectDocument;

impl JsonObjectDocument {
    pub fn parse(contents: &FileContents) -> Result<Self, JsonObjectDocumentError> {
        let value = serde_json::from_str::<Value>(contents.as_ref()).map_err(|error| {
            JsonObjectDocumentError::new(format!("invalid JSON object document: {error}"))
        })?;
        value
            .as_object()
            .ok_or_else(|| JsonObjectDocumentError::new("JSON document must be an object"))?;
        Ok(Self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsonObjectDocumentError {
    message: String,
}

impl JsonObjectDocumentError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for JsonObjectDocumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for JsonObjectDocumentError {}
