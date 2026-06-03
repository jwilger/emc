use std::collections::BTreeSet;
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

    pub fn reject_duplicate_keys(contents: &FileContents) -> Result<(), JsonObjectDocumentError> {
        if JsonKeyScanner::new(contents.as_ref()).contains_duplicate_key() {
            Err(JsonObjectDocumentError::new("duplicate JSON object key"))
        } else {
            Ok(())
        }
    }
}

struct JsonKeyScanner<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> JsonKeyScanner<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn contains_duplicate_key(mut self) -> bool {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> bool {
        self.skip_whitespace();
        match self.input[self.cursor..].chars().next() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => {
                self.parse_string();
                false
            }
            Some(_) => {
                self.parse_scalar();
                false
            }
            None => false,
        }
    }

    fn parse_object(&mut self) -> bool {
        self.bump_char();
        let mut keys = BTreeSet::new();
        loop {
            self.skip_whitespace();
            if self.consume_char('}') {
                return false;
            }
            let Some(key) = self.parse_string() else {
                return false;
            };
            if !keys.insert(key) {
                return true;
            }
            self.skip_whitespace();
            if !self.consume_char(':') || self.parse_value() {
                return true;
            }
            self.skip_whitespace();
            if self.consume_char('}') {
                return false;
            }
            if !self.consume_char(',') {
                return false;
            }
        }
    }

    fn parse_array(&mut self) -> bool {
        self.bump_char();
        loop {
            self.skip_whitespace();
            if self.consume_char(']') {
                return false;
            }
            if self.parse_value() {
                return true;
            }
            self.skip_whitespace();
            if self.consume_char(']') {
                return false;
            }
            if !self.consume_char(',') {
                return false;
            }
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        if !self.consume_char('"') {
            return None;
        }
        let mut value = String::new();
        while let Some(character) = self.bump_char() {
            match character {
                '"' => return Some(value),
                '\\' => {
                    self.bump_char().into_iter().for_each(|escaped| {
                        value.push(escaped);
                    });
                }
                _ => value.push(character),
            }
        }
        None
    }

    fn parse_scalar(&mut self) {
        while self.input[self.cursor..]
            .chars()
            .next()
            .is_some_and(|character| !matches!(character, ',' | ']' | '}'))
        {
            self.bump_char();
        }
    }

    fn skip_whitespace(&mut self) {
        while self.input[self.cursor..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            self.bump_char();
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.input[self.cursor..].starts_with(expected) {
            self.bump_char();
            true
        } else {
            false
        }
    }

    fn bump_char(&mut self) -> Option<char> {
        let character = self.input[self.cursor..].chars().next()?;
        self.cursor += character.len_utf8();
        Some(character)
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
