use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::types::{
    CommandName, EventName, ReadModelName, SliceSlug, StreamName, ViewName, WorkflowSlug,
};

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
pub struct NewProjectCommand {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    command: CommandName,
}

impl NewProjectCommand {
    pub fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug, command: CommandName) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            command,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectReadModel {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    read_model: ReadModelName,
}

impl NewProjectReadModel {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        read_model: ReadModelName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            read_model,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewProjectView {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    view: ViewName,
}

impl NewProjectView {
    pub fn new(workflow_slug: WorkflowSlug, slice_slug: SliceSlug, view: ViewName) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            view,
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
pub struct ProjectCommand {
    workflow_slug: String,
    slice_slug: String,
    command: String,
}

impl ProjectCommand {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn command(&self) -> &str {
        &self.command
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectReadModel {
    workflow_slug: String,
    slice_slug: String,
    read_model: String,
}

impl ProjectReadModel {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn read_model(&self) -> &str {
        &self.read_model
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectView {
    workflow_slug: String,
    slice_slug: String,
    view: String,
}

impl ProjectView {
    pub fn workflow_slug(&self) -> &str {
        &self.workflow_slug
    }

    pub fn slice_slug(&self) -> &str {
        &self.slice_slug
    }

    pub fn view(&self) -> &str {
        &self.view
    }
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

pub fn parse_lean_project_commands(
    contents: &FileContents,
) -> Result<Vec<ProjectCommand>, FormalProjectFactError> {
    command_entries_from_list(
        contents.as_ref(),
        "def modelCommands : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_commands(
    contents: &FileContents,
) -> Result<Vec<ProjectCommand>, FormalProjectFactError> {
    command_entries_from_list(
        contents.as_ref(),
        "val modelCommands: List[ModelCommand] = ",
    )
}

pub fn parse_lean_project_read_models(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModel>, FormalProjectFactError> {
    read_model_entries_from_list(
        contents.as_ref(),
        "def modelReadModels : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_read_models(
    contents: &FileContents,
) -> Result<Vec<ProjectReadModel>, FormalProjectFactError> {
    read_model_entries_from_list(
        contents.as_ref(),
        "val modelReadModels: List[ModelReadModel] = ",
    )
}

pub fn parse_lean_project_views(
    contents: &FileContents,
) -> Result<Vec<ProjectView>, FormalProjectFactError> {
    view_entries_from_list(
        contents.as_ref(),
        "def modelViews : List (String × String × String) := ",
    )
}

pub fn parse_quint_project_views(
    contents: &FileContents,
) -> Result<Vec<ProjectView>, FormalProjectFactError> {
    view_entries_from_list(contents.as_ref(), "val modelViews: List[ModelView] = ")
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

pub fn add_project_command(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    command: NewProjectCommand,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_command_record(&command);
    let quint_record = quint_command_record(&command);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelCommands : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let commands = command_entries_from_list(
            &contents,
            "def modelCommands : List (String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelCommandsAreDeclared :",
            &format!(
                "theorem modelCommandsAreDeclared : modelCommands.length = {} := rfl",
                commands.len()
            ),
        )
        .and_then(|contents| {
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelCommands: List[ModelCommand] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let commands =
            command_entries_from_list(&contents, "val modelCommands: List[ModelCommand] = ")?;
        replace_declaration(
            &contents,
            "val modelCommandsAreDeclared =",
            &format!(
                "val modelCommandsAreDeclared = modelCommands.length() == {}",
                commands.len()
            ),
        )
        .and_then(|contents| {
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added command {} to project root",
            command.command.as_ref()
        ))?),
    ]))
}

pub fn add_project_read_model(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    read_model: NewProjectReadModel,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_read_model_record(&read_model);
    let quint_record = quint_read_model_record(&read_model);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelReadModels : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let read_models = read_model_entries_from_list(
            &contents,
            "def modelReadModels : List (String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelReadModelsAreDeclared :",
            &format!(
                "theorem modelReadModelsAreDeclared : modelReadModels.length = {} := rfl",
                read_models.len()
            ),
        )
        .and_then(|contents| {
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelReadModels: List[ModelReadModel] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let read_models = read_model_entries_from_list(
            &contents,
            "val modelReadModels: List[ModelReadModel] = ",
        )?;
        replace_declaration(
            &contents,
            "val modelReadModelsAreDeclared =",
            &format!(
                "val modelReadModelsAreDeclared = modelReadModels.length() == {}",
                read_models.len()
            ),
        )
        .and_then(|contents| {
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added read model {} to project root",
            read_model.read_model.as_ref()
        ))?),
    ]))
}

pub fn add_project_view(
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
    view: NewProjectView,
) -> Result<EffectPlan, FormalProjectFactError> {
    let lean_record = lean_view_record(&view);
    let quint_record = quint_view_record(&view);
    let lean = append_record_if_missing(
        lean_contents.as_ref(),
        "def modelViews : List (String × String × String) := ",
        &lean_record,
    )
    .and_then(|contents| {
        let views = view_entries_from_list(
            &contents,
            "def modelViews : List (String × String × String) := ",
        )?;
        replace_declaration(
            &contents,
            "theorem modelViewsAreDeclared :",
            &format!(
                "theorem modelViewsAreDeclared : modelViews.length = {} := rfl",
                views.len()
            ),
        )
        .and_then(|contents| {
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let streams = parse_lean_project_streams_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelViews: List[ModelView] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let views = view_entries_from_list(&contents, "val modelViews: List[ModelView] = ")?;
        replace_declaration(
            &contents,
            "val modelViewsAreDeclared =",
            &format!(
                "val modelViewsAreDeclared = modelViews.length() == {}",
                views.len()
            ),
        )
        .and_then(|contents| {
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let streams = parse_quint_project_streams_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
    })?;

    Ok(EffectPlan::new(vec![
        Effect::WriteFile(lean_path, file_contents(lean)?),
        Effect::WriteFile(quint_path, file_contents(quint)?),
        Effect::Report(report_line(format!(
            "added view {} to project root",
            view.view.as_ref()
        ))?),
    ]))
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
            let commands = parse_lean_project_commands_from_contents_or_empty(&contents);
            let read_models = parse_lean_project_read_models_from_contents_or_empty(&contents);
            let views = parse_lean_project_views_from_contents_or_empty(&contents);
            let events = parse_lean_project_events_from_contents_or_empty(&contents);
            update_lean_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
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
            let commands = parse_quint_project_commands_from_contents_or_empty(&contents);
            let read_models = parse_quint_project_read_models_from_contents_or_empty(&contents);
            let views = parse_quint_project_views_from_contents_or_empty(&contents);
            let events = parse_quint_project_events_from_contents_or_empty(&contents);
            update_quint_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
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
        let commands = command_entries_from_list(
            &contents,
            "def modelCommands : List (String × String × String) := ",
        )?;
        let read_models = read_model_entries_from_list(
            &contents,
            "def modelReadModels : List (String × String × String) := ",
        )?;
        let views = view_entries_from_list(
            &contents,
            "def modelViews : List (String × String × String) := ",
        )?;
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
        .and_then(|contents| {
            update_lean_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
    })?;
    let quint = append_record_if_missing(
        quint_contents.as_ref(),
        "val modelEvents: List[ModelEvent] = ",
        &quint_record,
    )
    .and_then(|contents| {
        let commands =
            command_entries_from_list(&contents, "val modelCommands: List[ModelCommand] = ")?;
        let read_models = read_model_entries_from_list(
            &contents,
            "val modelReadModels: List[ModelReadModel] = ",
        )?;
        let views = view_entries_from_list(&contents, "val modelViews: List[ModelView] = ")?;
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
        .and_then(|contents| {
            update_quint_digest(
                &contents,
                &commands,
                &read_models,
                &views,
                &streams,
                &events,
            )
        })
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

fn parse_lean_project_commands_from_contents_or_empty(contents: &str) -> Vec<ProjectCommand> {
    command_entries_from_list(
        contents,
        "def modelCommands : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_commands_from_contents_or_empty(contents: &str) -> Vec<ProjectCommand> {
    command_entries_from_list(contents, "val modelCommands: List[ModelCommand] = ")
        .unwrap_or_default()
}

fn parse_lean_project_read_models_from_contents_or_empty(contents: &str) -> Vec<ProjectReadModel> {
    read_model_entries_from_list(
        contents,
        "def modelReadModels : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_read_models_from_contents_or_empty(contents: &str) -> Vec<ProjectReadModel> {
    read_model_entries_from_list(contents, "val modelReadModels: List[ModelReadModel] = ")
        .unwrap_or_default()
}

fn parse_lean_project_views_from_contents_or_empty(contents: &str) -> Vec<ProjectView> {
    view_entries_from_list(
        contents,
        "def modelViews : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_views_from_contents_or_empty(contents: &str) -> Vec<ProjectView> {
    view_entries_from_list(contents, "val modelViews: List[ModelView] = ").unwrap_or_default()
}

fn parse_lean_project_streams_from_contents_or_empty(contents: &str) -> Vec<ProjectStream> {
    stream_entries_from_list(
        contents,
        "def modelStreams : List (String × String × String) := ",
    )
    .unwrap_or_default()
}

fn parse_quint_project_streams_from_contents_or_empty(contents: &str) -> Vec<ProjectStream> {
    stream_entries_from_list(contents, "val modelStreams: List[ModelStream] = ").unwrap_or_default()
}

fn command_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectCommand>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut commands = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project command record is malformed",
                ))
            } else {
                Ok(ProjectCommand {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    command: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    commands.sort();
    commands.dedup();
    Ok(commands)
}

fn read_model_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectReadModel>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut read_models = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project read model record is malformed",
                ))
            } else {
                Ok(ProjectReadModel {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    read_model: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    read_models.sort();
    read_models.dedup();
    Ok(read_models)
}

fn view_entries_from_list(
    contents: &str,
    marker: &str,
) -> Result<Vec<ProjectView>, FormalProjectFactError> {
    let list = declaration_value(contents, marker)?;
    let mut views = split_top_level_records(list)?
        .into_iter()
        .map(|record| {
            let strings = quoted_strings(&record)?;
            if strings.len() < 3 {
                Err(FormalProjectFactError::new(
                    "formal project view record is malformed",
                ))
            } else {
                Ok(ProjectView {
                    workflow_slug: strings[0].clone(),
                    slice_slug: strings[1].clone(),
                    view: strings[2].clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, FormalProjectFactError>>()?;
    views.sort();
    views.dedup();
    Ok(views)
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
    commands: &[ProjectCommand],
    read_models: &[ProjectReadModel],
    views: &[ProjectView],
    streams: &[ProjectStream],
    events: &[ProjectEvent],
) -> Result<String, FormalProjectFactError> {
    let digest = digest_with_project_inventories(
        declaration_json_string(contents, "def modelDigest := ")?,
        commands,
        read_models,
        views,
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
    commands: &[ProjectCommand],
    read_models: &[ProjectReadModel],
    views: &[ProjectView],
    streams: &[ProjectStream],
    events: &[ProjectEvent],
) -> Result<String, FormalProjectFactError> {
    let digest = digest_with_project_inventories(
        declaration_json_string(contents, "val modelDigest = ")?,
        commands,
        read_models,
        views,
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

fn digest_with_project_inventories(
    current_digest: String,
    commands: &[ProjectCommand],
    read_models: &[ProjectReadModel],
    views: &[ProjectView],
    streams: &[ProjectStream],
    events: &[ProjectEvent],
) -> String {
    let prefix = current_digest
        .split_once(";commands=")
        .or_else(|| current_digest.split_once(";read-models="))
        .or_else(|| current_digest.split_once(";views="))
        .or_else(|| current_digest.split_once(";streams="))
        .map(|(prefix, _tail)| prefix.to_owned())
        .unwrap_or(current_digest);
    format!(
        "{prefix};commands={};read-models={};views={};streams={};events={}",
        digest_commands(commands),
        digest_read_models(read_models),
        digest_views(views),
        digest_streams(streams),
        digest_events(events)
    )
}

fn digest_commands(commands: &[ProjectCommand]) -> String {
    commands
        .iter()
        .map(|command| {
            format!(
                "{}/{}/{}",
                command.workflow_slug, command.slice_slug, command.command
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_read_models(read_models: &[ProjectReadModel]) -> String {
    read_models
        .iter()
        .map(|read_model| {
            format!(
                "{}/{}/{}",
                read_model.workflow_slug, read_model.slice_slug, read_model.read_model
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn digest_views(views: &[ProjectView]) -> String {
    views
        .iter()
        .map(|view| format!("{}/{}/{}", view.workflow_slug, view.slice_slug, view.view))
        .collect::<Vec<_>>()
        .join(",")
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

fn lean_command_record(command: &NewProjectCommand) -> String {
    format!(
        "({}, {}, {})",
        quoted(command.workflow_slug.as_ref()),
        quoted(command.slice_slug.as_ref()),
        quoted(command.command.as_ref())
    )
}

fn lean_read_model_record(read_model: &NewProjectReadModel) -> String {
    format!(
        "({}, {}, {})",
        quoted(read_model.workflow_slug.as_ref()),
        quoted(read_model.slice_slug.as_ref()),
        quoted(read_model.read_model.as_ref())
    )
}

fn quint_read_model_record(read_model: &NewProjectReadModel) -> String {
    format!(
        "{{ workflow: {}, slice: {}, readModel: {} }}",
        quoted(read_model.workflow_slug.as_ref()),
        quoted(read_model.slice_slug.as_ref()),
        quoted(read_model.read_model.as_ref())
    )
}

fn lean_view_record(view: &NewProjectView) -> String {
    format!(
        "({}, {}, {})",
        quoted(view.workflow_slug.as_ref()),
        quoted(view.slice_slug.as_ref()),
        quoted(view.view.as_ref())
    )
}

fn quint_view_record(view: &NewProjectView) -> String {
    format!(
        "{{ workflow: {}, slice: {}, view: {} }}",
        quoted(view.workflow_slug.as_ref()),
        quoted(view.slice_slug.as_ref()),
        quoted(view.view.as_ref())
    )
}

fn quint_command_record(command: &NewProjectCommand) -> String {
    format!(
        "{{ workflow: {}, slice: {}, command: {} }}",
        quoted(command.workflow_slug.as_ref()),
        quoted(command.slice_slug.as_ref()),
        quoted(command.command.as_ref())
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
