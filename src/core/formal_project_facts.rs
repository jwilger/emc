use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::types::{EventName, SliceSlug, StreamName, WorkflowSlug};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectStream {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    stream: StreamName,
}

impl NewProjectStream {
    pub fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug, stream: StreamName) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            stream,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectEvent {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    event: EventName,
    stream: StreamName,
}

impl NewProjectEvent {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        event: EventName,
        stream: StreamName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            event,
            stream,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectStream {
    workflow_slug: String,
    slice_slug: String,
    stream: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectEvent {
    workflow_slug: String,
    slice_slug: String,
    event: String,
    stream: String,
}

impl ProjectEvent {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn stream(&self) -> &str {
        &self.stream
    }
}

impl ProjectStream {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn stream(&self) -> &str {
        &self.stream
    }
}

pub fn parse_lean_project_streams(
    contents: &FileContents,
) -> Result<Vec<ProjectStream>, FormalProjectFactError> {
    stream_entries_from_list(
        contents.as_ref(),
        "def modelStreams : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_streams(
    contents: &FileContents,
) -> Result<Vec<ProjectStream>, FormalProjectFactError> {
    stream_entries_from_list(contents.as_ref(), "val modelStreams: List[ModelStream] = ")
}

pub fn parse_lean_project_events(
    contents: &FileContents,
) -> Result<Vec<ProjectEvent>, FormalProjectFactError> {
    event_entries_from_list(
        contents.as_ref(),
        "def modelEvents : List (String × String × String × String) := ",
    )
}

pub fn parse_quint_project_events(
    contents: &FileContents,
) -> Result<Vec<ProjectEvent>, FormalProjectFactError> {
    event_entries_from_list(contents.as_ref(), "val modelEvents: List[ModelEvent] = ")
}

pub fn add_project_stream(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    stream: NewProjectStream,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_stream_record(&stream);
    let quint_record = quint_stream_record(&stream);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelStreams : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let streams = stream_entries_from_list(
            &contents,
            "def modelStreams : List (String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelStreamsAreDeclared :",
            &format!(
                "theorem modelStreamsAreDeclared : modelStreams.length = {} := rfl",
                streams.len()
            ),
        )
        .and_then(|contents| {
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            update_lean_digest(&contents, &streams, &events)
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelStreams: List[ModelStream] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let streams =
            stream_entries_from_list(&contents, "val modelStreams: List[ModelStream] = ")?;
        replace_declaration(
            &contents,
            "val modelStreamsAreDeclared =",
            &format!(
                "val modelStreamsAreDeclared = modelStreams.length() == {}",
                streams.len()
            ),
        )
        .and_then(|contents| {
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            update_quint_digest(&contents, &streams, &events)
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added stream {} to project root",
            stream.stream.as_ref()
        ))?),
    ]))
}

pub fn add_project_event(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    event: NewProjectEvent,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_event_record(&event);
    let quint_record = quint_event_record(&event);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelEvents : List (String × String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let streams = stream_entries_from_list(
            &contents,
            "def modelStreams : List (String × String × String) := ",
        )?;
        let events = event_entries_from_list(
            &contents,
            "def modelEvents : List (String × String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelEventsAreDeclared :",
            &format!(
                "theorem modelEventsAreDeclared : modelEvents.length = {} := rfl",
                events.len()
            ),
        )
        .and_then(|contents| update_lean_digest(&contents, &streams, &events))
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelEvents: List[ModelEvent] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let streams =
            stream_entries_from_list(&contents, "val modelStreams: List[ModelStream] = ")?;
        let events = event_entries_from_list(&contents, "val modelEvents: List[ModelEvent] = ")?;
        replace_declaration(
            &contents,
            "val modelEventsAreDeclared =",
            &format!(
                "val modelEventsAreDeclared = modelEvents.length() == {}",
                events.len()
            ),
        )
        .and_then(|contents| update_quint_digest(&contents, &streams, &events))
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added event {} to project root",
            event.event.as_ref()
        ))?),
    ]))
}

#[derive(Debug)]
pub struct FormalProjectFactError {
    message: String,
}

impl FormalProjectFactError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for FormalProjectFactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for FormalProjectFactError {}

fn append_record_if_missing(
    contents: &str,
    marker: &str,
    record: &str,
) -> Result<String, FormalProjectFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if let Some(current_list) = declaration.strip_prefix(marker) {
                replaced = true;
                Ok(format!(
                    "{indentation}{marker}{}",
                    append_list_record_if_missing(current_list, record)?
                ))
            } else {
                Ok(line.to_owned())
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;

    if replaced {
        Ok(join_lines_preserving_trailing_newline(contents, lines))
    } else {
        Err(FormalProjectFactError::new(format!(
            "formal project artifact is missing declaration {marker}"
        )))
    }
}

fn append_list_record_if_missing(
    current_list: &str,
    record: &str,
) -> Result<String, FormalProjectFactError> {
    let trimmed = current_list.trim();
    if trimmed == "[]" {
        return Ok(format!("[{record}]"));
    }
    let existing = trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .ok_or_else(|| {
            FormalProjectFactError::new("formal project list declaration is malformed")
        })?;
    if split_top_level_records(trimmed)?
        .iter()
        .any(|entry| entry == record)
    {
        Ok(trimmed.to_owned())
    } else {
        Ok(format!("[{existing},{record}]"))
    }
}

fn replace_declaration(
    contents: &str,
    marker: &str,
    replacement: &str,
) -> Result<String, FormalProjectFactError> {
    let mut replaced = false;
    let lines = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            if declaration.starts_with(marker) {
                replaced = true;
                format!("{indentation}{replacement}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>();

    if replaced {
        Ok(join_lines_preserving_trailing_newline(contents, lines))
    } else {
        Err(FormalProjectFactError::new(format!(
            "formal project artifact is missing declaration {marker}"
        )))
    }
}

fn stream_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectStream>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut streams = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project stream record is malformed",
                ))
            } else {
                Ok(ProjectStream {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    stream: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    streams.sort();
    streams.dedup();
    Ok(streams)
}

fn parse_lean_project_events_from_contents_or_empty(contents: &str) -> Vec<ProjectEvent> {
    event_entries_from_list(
        contents,
        "def modelEvents : List (String × String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_events_from_contents_or_empty(contents: &str) -> Vec<ProjectEvent> {
    event_entries_from_list(contents, "val modelEvents: List[ModelEvent] = ").unwrap_or_default()
}

fn event_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectEvent>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut events = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 4 {
                Err(FormalProjectFactError::new(
                    "formal project event record is malformed",
                ))
            } else {
                Ok(ProjectEvent {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    event: strings[2].clone(),
                    stream: strings[3].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    events.sort();
    events.dedup();
    Ok(events)
}

fn split_top_level_records(list: &str) -> Result<Vec<String>, FormalProjectFactError> {
    let trimmed = list.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .ok_or_else(|| {
            FormalProjectFactError::new("formal project list declaration is malformed")
        })?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in inner.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    FormalProjectFactError::new("formal project list declaration is malformed")
                })?;
            }
            ',' if depth == 0 => {
                records.push(inner[start..index].trim().to_owned());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    records.push(inner[start..].trim().to_owned());
    Ok(records)
}

fn quoted_strings(record: &str) -> Result<Vec<String>, FormalProjectFactError> {
    let mut strings = Vec::new();
    let mut start = None;
    let mut escaped = false;
    for (index, character) in record.char_indices() {
        if let Some(open) = start {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                strings.push(
                    serde_json::from_str(&record[open..=index]).map_err(|error| {
                        FormalProjectFactError::new(format!(
                            "formal project string literal is malformed: {error}"
                        ))
                    })?,
                );
                start = None;
            }
        } else if character == '"' {
            start = Some(index);
        }
    }
    if start.is_some() {
        Err(FormalProjectFactError::new(
            "formal project string literal is unterminated",
        ))
    } else {
        Ok(strings)
    }
}

fn update_lean_digest(
    contents: &str,
    streams: &[ProjectStream],
    events: &[ProjectEvent],
) -> Result<String, FormalProjectFactError> {
    let digest = digest_with_streams(
        declaration_json_string(contents, "def modelDigest := ")?,
        streams,
        events,
    );
    replace_declaration(
        contents,
        "def modelDigest :=",
        &format!("def modelDigest := {}", quoted(&digest)),
    )
    .and_then(|contents| {
        replace_declaration(
            &contents,
            "theorem modelDigestIsStable",
            &format!(
                "theorem modelDigestIsStable : modelDigest = {} := rfl",
                quoted(&digest)
            ),
        )
    })
}

fn update_quint_digest(
    contents: &str,
    streams: &[ProjectStream],
    events: &[ProjectEvent],
) -> Result<String, FormalProjectFactError> {
    let digest = digest_with_streams(
        declaration_json_string(contents, "val modelDigest = ")?,
        streams,
        events,
    );
    replace_declaration(
        contents,
        "val modelDigest =",
        &format!("val modelDigest = {}", quoted(&digest)),
    )
    .and_then(|contents| {
        replace_declaration(
            &contents,
            "val modelDigestStable =",
            &format!("val modelDigestStable = modelDigest == {}", quoted(&digest)),
        )
    })
}

fn digest_with_streams(
    current_digest: String,
    streams: &[ProjectStream],
    events: &[ProjectEvent],
) -> String {
    let prefix = current_digest
        .split_once(";streams=")
        .map(|(prefix, _streams)| prefix.to_owned())
        .unwrap_or(current_digest);
    format!(
        "{prefix};streams={};events={}",
        digest_streams(streams),
        digest_events(events)
    )
}

fn digest_streams(streams: &[ProjectStream]) -> String {
    streams
        .iter()
        .map(|stream| {
            format!(
                "{}/{}/{}",
                stream.workflow_slug, stream.slice_slug, stream.stream
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_events(events: &[ProjectEvent]) -> String {
    events
        .iter()
        .map(|event| {
            format!(
                "{}/{}/{}@{}",
                event.workflow_slug, event.slice_slug, event.event, event.stream
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn declaration_value<'a>(
    contents: &'a str,
    marker: &str,
) -> Result<&'a str, FormalProjectFactError> {
    contents
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(marker))
        .ok_or_else(|| {
            FormalProjectFactError::new(format!(
                "formal project artifact is missing declaration {marker}"
            ))
        })
}

fn declaration_json_string(contents: &str, marker: &str) -> Result<String, FormalProjectFactError> {
    serde_json::from_str(declaration_value(contents, marker)?.trim()).map_err(|error| {
        FormalProjectFactError::new(format!(
            "formal project model digest declaration is malformed: {error}"
        ))
    })
}

fn join_lines_preserving_trailing_newline(original: &str, lines: Vec<String>) -> String {
    let mut updated = lines.join("\n");
    if original.ends_with('\n') {
        updated.push('\n');
    }
    updated
}

fn lean_stream_record(stream: &NewProjectStream) -> String {
    format!(
        "({}, {}, {})",
        quoted(stream.workflow_slug.as_ref()),
        quoted(stream.slice_slug.as_ref()),
        quoted(stream.stream.as_ref())
    )
}

fn quint_stream_record(stream: &NewProjectStream) -> String {
    format!(
        "{{ workflow: {}, slice: {}, stream: {} }}",
        quoted(stream.workflow_slug.as_ref()),
        quoted(stream.slice_slug.as_ref()),
        quoted(stream.stream.as_ref())
    )
}

fn lean_event_record(event: &NewProjectEvent) -> String {
    format!(
        "({}, {}, {}, {})",
        quoted(event.workflow_slug.as_ref()),
        quoted(event.slice_slug.as_ref()),
        quoted(event.event.as_ref()),
        quoted(event.stream.as_ref())
    )
}

fn quint_event_record(event: &NewProjectEvent) -> String {
    format!(
        "{{ workflow: {}, slice: {}, event: {}, stream: {} }}",
        quoted(event.workflow_slug.as_ref()),
        quoted(event.slice_slug.as_ref()),
        quoted(event.event.as_ref()),
        quoted(event.stream.as_ref())
    )
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated formal project string literal must be valid: {error}");
    })
}

fn file_contents(value: String) -> Result<FileContents, FormalProjectFactError> {
    FileContents::try_new(value).map_err(|error| FormalProjectFactError::new(error.to_string()))
}

fn report_line(value: String) -> Result<ReportLine, FormalProjectFactError> {
    ReportLine::try_new(value).map_err(|error| FormalProjectFactError::new(error.to_string()))
}
