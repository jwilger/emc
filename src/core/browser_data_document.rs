use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Value;

use crate::core::effect::FileContents;
use crate::core::types::{
    BoardLaneId, BrowserCommandDefinitionDetail, BrowserControlEffectDetail,
    BrowserErrorRecoveryDetail, BrowserEventElementName, BrowserFieldSourceChainDetail,
    BrowserViewDefinitionDetail, CommandErrorName, CommandName, ControlEffectKind,
    ControlEffectTarget, ControlLabel, DefinitionSectionLabel, SliceName, SourceChainHop,
    SourceControlReference, ViewFieldName, ViewName,
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

    pub(crate) fn board_lane_ids(&self) -> Result<Vec<BoardLaneId>, BrowserDataDocumentError> {
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

    pub(crate) fn event_element_names(
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

    pub(crate) fn error_recovery_details(
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
pub struct BrowserDataCorpus {
    documents: Vec<BrowserDataDocument>,
}

impl BrowserDataCorpus {
    pub(crate) fn new(documents: Vec<BrowserDataDocument>) -> Self {
        Self { documents }
    }

    pub(crate) fn board_lane_ids(&self) -> Result<Vec<BoardLaneId>, BrowserDataDocumentError> {
        self.documents
            .iter()
            .map(BrowserDataDocument::board_lane_ids)
            .collect::<Result<Vec<_>, _>>()
            .map(|lane_groups| {
                lane_groups.into_iter().flatten().fold(
                    Vec::<BoardLaneId>::new(),
                    |mut lanes, lane| {
                        if !lanes.iter().any(|existing| existing == &lane) {
                            lanes.push(lane);
                        }
                        lanes
                    },
                )
            })
    }

    pub(crate) fn event_element_names(
        &self,
    ) -> Result<Vec<BrowserEventElementName>, BrowserDataDocumentError> {
        self.documents
            .iter()
            .map(BrowserDataDocument::event_element_names)
            .collect::<Result<Vec<_>, _>>()
            .map(|names| names.into_iter().flatten().collect())
    }

    pub(crate) fn error_recovery_details(
        &self,
    ) -> Result<Vec<BrowserErrorRecoveryDetail>, BrowserDataDocumentError> {
        self.documents
            .iter()
            .map(BrowserDataDocument::error_recovery_details)
            .collect::<Result<Vec<_>, _>>()
            .map(|details| details.into_iter().flatten().collect())
    }

    pub(crate) fn command_definition_details(
        &self,
    ) -> Result<Vec<BrowserCommandDefinitionDetail>, BrowserDataDocumentError> {
        self.documents
            .iter()
            .flat_map(command_definition_names)
            .map(|name| {
                Ok(BrowserCommandDefinitionDetail::new(
                    command_name(name)?,
                    self.command_owning_slice(name)?,
                    self.command_source_controls(name)?,
                    command_section_labels()?,
                ))
            })
            .collect()
    }

    pub(crate) fn view_definition_details(
        &self,
    ) -> Result<Vec<BrowserViewDefinitionDetail>, BrowserDataDocumentError> {
        self.documents
            .iter()
            .flat_map(view_definitions)
            .filter_map(|view| {
                view.get("name")
                    .and_then(Value::as_str)
                    .map(|name| (name, view))
            })
            .map(|(name, view)| {
                Ok(BrowserViewDefinitionDetail::new(
                    view_name(name)?,
                    self.view_field_source_chains(view)?,
                    view_control_effects(view)?,
                ))
            })
            .collect()
    }

    fn command_owning_slice(
        &self,
        command_name: &str,
    ) -> Result<SliceName, BrowserDataDocumentError> {
        self.documents
            .iter()
            .flat_map(slice_definitions)
            .find(|slice| {
                slice
                    .get("commands")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .any(|command| {
                        command
                            .as_str()
                            .is_some_and(|slice_command| slice_command == command_name)
                    })
            })
            .and_then(|slice| slice.get("name").and_then(Value::as_str))
            .ok_or_else(|| {
                BrowserDataDocumentError::new(format!(
                    "command '{command_name}' has no owning slice"
                ))
            })
            .and_then(slice_name)
    }

    fn command_source_controls(
        &self,
        command_name: &str,
    ) -> Result<Vec<SourceControlReference>, BrowserDataDocumentError> {
        self.documents
            .iter()
            .flat_map(view_definitions)
            .flat_map(|view| view_command_source_controls(view, command_name))
            .map(source_control_reference)
            .collect()
    }

    fn view_field_source_chains(
        &self,
        view: &Value,
    ) -> Result<Vec<BrowserFieldSourceChainDetail>, BrowserDataDocumentError> {
        view.get("fields")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|field| {
                Some((
                    field.get("name").and_then(Value::as_str)?,
                    field.get("source").and_then(Value::as_str)?,
                ))
            })
            .map(|(field, source)| {
                Ok(BrowserFieldSourceChainDetail::new(
                    view_field_name(field)?,
                    self.source_chain_hops(source)?,
                ))
            })
            .collect()
    }

    fn source_chain_hops(
        &self,
        source: &str,
    ) -> Result<Vec<SourceChainHop>, BrowserDataDocumentError> {
        [
            Some(source),
            self.read_model_field_source(source),
            self.event_attribute_source(source),
        ]
        .into_iter()
        .flatten()
        .map(source_chain_hop)
        .collect()
    }

    fn read_model_field_source(&self, source: &str) -> Option<&str> {
        let (read_model_name, field_name) = source
            .strip_prefix("read_model.")
            .and_then(|source| source.split_once('.'))?;

        self.documents
            .iter()
            .flat_map(read_model_definitions)
            .find(|read_model| {
                read_model
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name == read_model_name)
            })
            .and_then(|read_model| {
                read_model
                    .get("fields")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .find(|field| {
                        field
                            .get("name")
                            .and_then(Value::as_str)
                            .is_some_and(|name| name == field_name)
                    })
            })
            .and_then(|field| field.get("source").and_then(Value::as_str))
    }

    fn event_attribute_source(&self, source: &str) -> Option<&str> {
        let (event_name, attribute_name) = self.read_model_field_source(source)?.split_once('.')?;

        self.documents
            .iter()
            .flat_map(event_definitions)
            .find(|event| {
                event
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name == event_name)
            })
            .and_then(|event| {
                event
                    .get("attributes")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .find(|attribute| {
                        attribute
                            .get("name")
                            .and_then(Value::as_str)
                            .is_some_and(|name| name == attribute_name)
                    })
            })
            .and_then(|attribute| attribute.get("source").and_then(Value::as_str))
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

fn command_definition_names(document: &BrowserDataDocument) -> impl Iterator<Item = &str> {
    document
        .value
        .get("commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|command| command.get("name").and_then(Value::as_str))
}

fn slice_definitions(document: &BrowserDataDocument) -> impl Iterator<Item = &Value> {
    document
        .value
        .get("slices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn view_definitions(document: &BrowserDataDocument) -> impl Iterator<Item = &Value> {
    document
        .value
        .get("views")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn read_model_definitions(document: &BrowserDataDocument) -> impl Iterator<Item = &Value> {
    document
        .value
        .get("read_models")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn event_definitions(document: &BrowserDataDocument) -> impl Iterator<Item = &Value> {
    document
        .value
        .get("events")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn view_command_source_controls(view: &Value, command_name: &str) -> Vec<String> {
    view.get("name")
        .and_then(Value::as_str)
        .map(|view_name| {
            view.get("controls")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|control| {
                    control
                        .get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|command| command == command_name)
                })
                .filter_map(|control| control.get("label").and_then(Value::as_str))
                .map(|label| format!("{view_name} / {label}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn command_section_labels() -> Result<Vec<DefinitionSectionLabel>, BrowserDataDocumentError> {
    [
        "Produced events",
        "Read models",
        "Returned errors",
        "Workflow transitions",
    ]
    .into_iter()
    .map(definition_section_label)
    .collect()
}

fn command_name(raw: &str) -> Result<CommandName, BrowserDataDocumentError> {
    CommandName::try_new(raw.to_owned())
        .map_err(|error| BrowserDataDocumentError::new(format!("invalid command name: {error}")))
}

fn slice_name(raw: &str) -> Result<SliceName, BrowserDataDocumentError> {
    SliceName::try_new(raw.to_owned()).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid owning slice name: {error}"))
    })
}

fn source_control_reference(
    raw: String,
) -> Result<SourceControlReference, BrowserDataDocumentError> {
    SourceControlReference::try_new(raw).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid source control reference: {error}"))
    })
}

fn definition_section_label(raw: &str) -> Result<DefinitionSectionLabel, BrowserDataDocumentError> {
    DefinitionSectionLabel::try_new(raw.to_owned()).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid definition section label: {error}"))
    })
}

fn view_control_effects(
    view: &Value,
) -> Result<Vec<BrowserControlEffectDetail>, BrowserDataDocumentError> {
    view.get("controls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|control| {
            Some((
                control.get("label").and_then(Value::as_str)?,
                control_effect_kind_and_target(control)?,
            ))
        })
        .map(|(label, (kind, target))| {
            Ok(BrowserControlEffectDetail::new(
                control_label(label)?,
                control_effect_kind(kind)?,
                control_effect_target(target)?,
            ))
        })
        .collect()
}

fn control_effect_kind_and_target(control: &Value) -> Option<(&'static str, &str)> {
    control
        .get("command")
        .and_then(Value::as_str)
        .map(|command| ("command", command))
        .or_else(|| {
            control
                .get("navigation")
                .and_then(Value::as_str)
                .map(|navigation| (navigation_effect_kind(control), navigation))
        })
}

fn navigation_effect_kind(control: &Value) -> &'static str {
    match control.get("navigation_type").and_then(Value::as_str) {
        Some("external_workflow") => "workflow navigation",
        Some("external_system") => "external navigation",
        Some("local_view_state") => "local navigation",
        Some("modeled_view") => "view navigation",
        _ => "navigation",
    }
}

fn view_field_name(raw: &str) -> Result<ViewFieldName, BrowserDataDocumentError> {
    ViewFieldName::try_new(raw.to_owned())
        .map_err(|error| BrowserDataDocumentError::new(format!("invalid view field name: {error}")))
}

fn source_chain_hop(raw: &str) -> Result<SourceChainHop, BrowserDataDocumentError> {
    SourceChainHop::try_new(raw.to_owned()).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid source chain hop: {error}"))
    })
}

fn control_label(raw: &str) -> Result<ControlLabel, BrowserDataDocumentError> {
    ControlLabel::try_new(raw.to_owned())
        .map_err(|error| BrowserDataDocumentError::new(format!("invalid control label: {error}")))
}

fn control_effect_kind(raw: &str) -> Result<ControlEffectKind, BrowserDataDocumentError> {
    ControlEffectKind::try_new(raw.to_owned()).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid control effect kind: {error}"))
    })
}

fn control_effect_target(raw: &str) -> Result<ControlEffectTarget, BrowserDataDocumentError> {
    ControlEffectTarget::try_new(raw.to_owned()).map_err(|error| {
        BrowserDataDocumentError::new(format!("invalid control effect target: {error}"))
    })
}
