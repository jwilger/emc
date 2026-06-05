use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io;
use std::path::Path;
use std::process::{self, Command};

use crate::core::connection::{connect_workflow, remove_transition};
use crate::core::effect::{
    ArtifactDigest, Effect, EffectPlan, FileContents, ProcessInvocation, ProjectPath,
};
use crate::core::formal_graph::{
    FormalGraphError, FormalWorkflowGraph, FormalWorkflowGraphs, parse_lean_workflow_graph,
    parse_quint_workflow_graph,
};
use crate::core::formal_slice_facts::{
    add_automation_definition, add_bit_level_data_flow, add_board_connection, add_board_element,
    add_command_definition, add_event_definition, add_external_payload_definition,
    add_outcome_definition, add_read_model_definition, add_slice_scenario,
    add_translation_definition, add_view_definition,
};
use crate::core::formal_workflow_facts::{
    add_workflow_command_error, add_workflow_entry_lifecycle_state, add_workflow_outcome,
    add_workflow_owned_definition, add_workflow_transition_evidence,
    require_workflow_entry_lifecycle_coverage,
};
use crate::core::json_object_document::JsonObjectDocument;
use crate::core::layout::{
    ModeledWorkflowLayout, ModeledWorkflowLayouts, ModeledWorkflowSliceDetails,
    ModeledWorkflowTransitions, check_project, list_slices, list_transitions, list_workflows,
    show_document, show_workflow,
};
use crate::core::project::ProjectName;
use crate::core::review_record::{
    RequiredReviewCategories, ReviewCategoryFinding, ReviewRecordDocument, record_clean_review,
};
use crate::core::slice::{
    add_slice, remove_slice, update_slice_description, update_slice_kind, update_slice_name,
};
use crate::core::types::{
    ReviewRuleName, SliceSlug, WorkflowSliceDetail, WorkflowSliceDetails, WorkflowSlug,
    WorkflowTransitionRecord,
};
use crate::core::verify::verify_project;
use crate::core::workflow::{
    IndexedWorkflowGraph, IndexedWorkflowGraphs, add_workflow, remove_workflow,
    update_workflow_description, update_workflow_name,
};
use crate::io::dto::parse_project_manifest_name;

const REQUIRED_REVIEW_CATEGORIES: &[&str] = &[
    "lifecycle-entry",
    "canonical-lanes",
    "board-connections",
    "fake-intermediates",
    "slice-ownership",
    "source-chains",
    "workflow-reachability",
    "transition-resolution",
    "navigation-targets",
    "branch-shape",
    "outcomes-and-errors",
    "scenario-coverage",
    "timeline-rendering",
];

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

    pub fn project_path(error: impl Display) -> Self {
        Self {
            message: format!("invalid project path: {error}"),
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
    interpret_collect_reports(plan).map(|reports| {
        reports.into_iter().for_each(|report| println!("{report}"));
    })
}

pub fn interpret_collect_reports(plan: EffectPlan) -> Result<Vec<String>, ShellError> {
    plan.effects()
        .iter()
        .try_fold(Vec::new(), |mut reports, effect| {
            reports.extend(interpret_effect(effect)?);
            Ok(reports)
        })
}

fn interpret_effect(effect: &Effect) -> Result<Vec<String>, ShellError> {
    match effect {
        Effect::AddAutomationDefinitionFromSlice(automation) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(automation.slice_slug())?;
            let plan = add_automation_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                automation.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddBitLevelDataFlowFromSlice(data_flow) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(data_flow.slice_slug())?;
            let plan = add_bit_level_data_flow(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                data_flow.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddBoardConnectionFromSlice(connection) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(connection.slice_slug())?;
            let plan = add_board_connection(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                connection.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddBoardElementFromSlice(element) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(element.slice_slug())?;
            let plan = add_board_element(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                element.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddCommandDefinitionFromSlice(command) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(command.slice_slug())?;
            let plan = add_command_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                command.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddEventDefinitionFromSlice(event) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(event.slice_slug())?;
            let plan = add_event_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                event.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddExternalPayloadDefinitionFromSlice(external_payload) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(external_payload.slice_slug())?;
            let plan = add_external_payload_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                external_payload.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddOutcomeDefinitionFromSlice(outcome) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(outcome.slice_slug())?;
            let plan = add_outcome_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                outcome.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddReadModelDefinitionFromSlice(read_model) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(read_model.slice_slug())?;
            let plan = add_read_model_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                read_model.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddViewDefinitionFromSlice(view) => {
            let slice_artifacts = read_formal_slice_artifact_paths_and_contents(view.slice_slug())?;
            let plan = add_view_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                view.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddSliceFromWorkflow(slice) => {
            let (workflow_layout, workflow_graph) =
                read_formal_workflow_layout_and_graph(slice.workflow_slug())?;
            let plan = add_slice(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_graph,
                slice.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddSliceScenarioFromSlice(scenario) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(scenario.slice_slug())?;
            let plan = add_slice_scenario(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                scenario.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddTranslationDefinitionFromSlice(translation) => {
            let slice_artifacts =
                read_formal_slice_artifact_paths_and_contents(translation.slice_slug())?;
            let plan = add_translation_definition(
                slice_artifacts.lean_path,
                slice_artifacts.lean_contents,
                slice_artifacts.quint_path,
                slice_artifacts.quint_contents,
                translation.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowFromIndex(workflow) => {
            let project_name = read_project_manifest_name()?;
            let existing_workflows =
                formal_workflow_layouts(read_synchronized_formal_workflow_graphs()?);
            let plan = add_workflow(
                project_name,
                ModeledWorkflowLayouts::new(existing_workflows),
                workflow.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowCommandErrorFromWorkflow(workflow_slug, error) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(workflow_slug)?;
            let plan = add_workflow_command_error(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                workflow_slug.clone(),
                error.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowOwnedDefinitionFromWorkflow(workflow_slug, definition) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(workflow_slug)?;
            let plan = add_workflow_owned_definition(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                workflow_slug.clone(),
                definition.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowOutcomeFromWorkflow(workflow_slug, outcome) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(workflow_slug)?;
            let plan = add_workflow_outcome(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                workflow_slug.clone(),
                outcome.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowTransitionEvidenceFromWorkflow(workflow_slug, evidence) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(workflow_slug)?;
            let plan = add_workflow_transition_evidence(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                workflow_slug.clone(),
                evidence.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::RequireWorkflowEntryLifecycleCoverageFromWorkflow(workflow_slug) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(workflow_slug)?;
            let plan = require_workflow_entry_lifecycle_coverage(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                workflow_slug.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::AddWorkflowEntryLifecycleStateFromWorkflow(workflow_slug, coverage) => {
            let workflow_artifacts =
                read_formal_workflow_artifact_paths_and_contents(workflow_slug)?;
            let plan = add_workflow_entry_lifecycle_state(
                workflow_artifacts.lean_path,
                workflow_artifacts.lean_contents,
                workflow_artifacts.quint_path,
                workflow_artifacts.quint_contents,
                workflow_slug.clone(),
                coverage.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::CheckCurrentProject => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?;
            interpret_collect_reports(check_project(project_name, formal_workflows))
        }
        Effect::ConnectWorkflowFromWorkflow(connection) => {
            let (workflow_layout, workflow_graph) =
                read_formal_workflow_layout_and_graph(connection.workflow_slug())?;
            let plan = connect_workflow(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_graph,
                connection.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::CopyDirectory(source, target) => {
            copy_directory(source.as_ref(), target.as_ref()).map(|()| Vec::new())
        }
        Effect::EnsureDirectory(path) => fs::create_dir_all(Path::new(path.as_ref()))
            .map(|()| Vec::new())
            .map_err(ShellError::io),
        Effect::Fail(message) => Err(ShellError::message(message.as_ref().to_owned())),
        Effect::ListWorkflowsFromIndex => {
            let modeled_workflows =
                formal_workflow_layouts(read_synchronized_formal_workflow_graphs()?);
            interpret_collect_reports(list_workflows(ModeledWorkflowLayouts::new(
                modeled_workflows,
            )))
        }
        Effect::ListSlicesFromIndex => {
            let modeled_slices =
                formal_workflow_slice_details(read_synchronized_formal_workflow_graphs()?);
            interpret_collect_reports(list_slices(ModeledWorkflowSliceDetails::new(
                modeled_slices,
            )))
        }
        Effect::ListTransitionsFromIndex => {
            let modeled_transitions =
                formal_workflow_transitions(read_synchronized_formal_workflow_graphs()?);
            interpret_collect_reports(list_transitions(ModeledWorkflowTransitions::new(
                modeled_transitions,
            )))
        }
        Effect::RequireCanonicalDeclaration(path, prefix, marker, message) => {
            let contents = fs::read_to_string(Path::new(path.as_ref())).map_err(ShellError::io)?;
            if artifact_contains_one_canonical_declaration(
                &contents,
                prefix.as_ref(),
                marker.as_ref(),
            ) {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireDigest(path, digest, message) => {
            let contents = fs::read_to_string(Path::new(path.as_ref())).map_err(ShellError::io)?;
            if artifact_contains_one_digest_marker(&contents, digest.as_ref()) {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RequireFile(path) => {
            if Path::new(path.as_ref()).is_file() {
                Ok(Vec::new())
            } else {
                Err(ShellError::message(format!(
                    "missing required project artifact {}",
                    path.as_ref()
                )))
            }
        }
        Effect::RequireFileContents(path, expected, message) => {
            require_file_contents(path.as_ref(), expected.as_ref(), message.as_ref())
                .map(|()| Vec::new())
        }
        Effect::RequireFileContentsWithAuthoredFormalFacts(path, expected, message) => {
            require_file_contents_with_authored_formal_facts(
                path.as_ref(),
                expected.as_ref(),
                message.as_ref(),
            )
            .map(|()| Vec::new())
        }
        Effect::RequireJsonObjectKeysUnique(path, message) => {
            require_json_object_keys_unique(path.as_ref(), message.as_ref()).map(|()| Vec::new())
        }
        Effect::RequireOnlyModeledArtifacts(path, extension, allowed_paths, message) => {
            require_only_modeled_artifacts(
                path.as_ref(),
                extension.as_ref(),
                allowed_paths,
                message.as_ref(),
            )
            .map(|()| Vec::new())
        }
        Effect::RequireReviewRecord(path, workflow_slug, message) => {
            if Path::new(path.as_ref()).is_file() {
                require_clean_review_record(path.as_ref(), workflow_slug, message.as_ref())
                    .map(|()| Vec::new())
            } else {
                Err(ShellError::message(message.as_ref().to_owned()))
            }
        }
        Effect::RunProcess(invocation) => run_process(invocation),
        Effect::RecordCleanReviewFromWorkflow(slug, reviewer_id, reviewed_at) => {
            let current_digest = formal_model_content_digest(slug)?;
            let required_categories = required_review_categories()?;
            let plan = record_clean_review(
                slug.clone(),
                current_digest,
                reviewer_id.clone(),
                reviewed_at.clone(),
                RequiredReviewCategories::new(required_categories),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::RemoveDirectory(path) => {
            remove_directory_if_present(path.as_ref()).map(|()| Vec::new())
        }
        Effect::RemoveFile(path) => remove_file_if_present(path.as_ref()).map(|()| Vec::new()),
        Effect::RemoveSliceFromWorkflow(slug) => {
            let (workflow_layout, workflow_graph) = find_formal_workflow_containing_slice(slug)?;
            let plan = remove_slice(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                slug.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::RemoveTransitionFromWorkflow(removal) => {
            let (workflow_layout, workflow_graph) =
                read_formal_workflow_layout_and_graph(removal.workflow_slug())?;
            let plan = remove_transition(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_graph,
                removal.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::RemoveWorkflowFromIndex(slug) => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let existing_workflows = formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(
                formal_workflows.clone(),
            ));
            let workflow_graphs = indexed_formal_workflow_graphs(&formal_workflows);
            let plan = remove_workflow(
                project_name,
                ModeledWorkflowLayouts::new(existing_workflows),
                IndexedWorkflowGraphs::new(workflow_graphs),
                slug.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::ShowSliceFromSlice(slug) => {
            let slice_document = read_formal_slice_artifacts(slug)?;
            interpret_collect_reports(show_document(slice_document))
        }
        Effect::ShowWorkflowFromWorkflow(slug) => {
            let workflow_document = read_formal_workflow_artifacts(slug)?;
            interpret_collect_reports(show_workflow(workflow_document))
        }
        Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(slug, description) => {
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let existing_workflows = formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(
                formal_workflows.clone(),
            ));
            let workflow_graph = read_formal_workflow_graph(slug)?;
            let plan = update_workflow_description(
                ModeledWorkflowLayouts::new(existing_workflows),
                workflow_graph,
                slug.clone(),
                description.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateWorkflowNameFromIndexAndWorkflow(slug, name) => {
            let formal_workflows = read_synchronized_formal_workflow_graphs()?.into_inner();
            let existing_workflows = formal_workflow_layouts(FormalWorkflowGraphs::from_graphs(
                formal_workflows.clone(),
            ));
            let workflow_graph = read_formal_workflow_graph(slug)?;
            let plan = update_workflow_name(
                ModeledWorkflowLayouts::new(existing_workflows),
                workflow_graph,
                slug.clone(),
                name.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateSliceDescriptionFromWorkflow(slug, description) => {
            let (workflow_layout, workflow_graph) = find_formal_workflow_containing_slice(slug)?;
            let plan = update_slice_description(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                slug.clone(),
                description.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateSliceKindFromWorkflow(slug, kind) => {
            let (workflow_layout, workflow_graph) = find_formal_workflow_containing_slice(slug)?;
            let plan = update_slice_kind(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                slug.clone(),
                *kind,
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::UpdateSliceNameFromWorkflow(slug, name) => {
            let (workflow_layout, workflow_graph) = find_formal_workflow_containing_slice(slug)?;
            let plan = update_slice_name(
                workflow_layout.name().clone(),
                workflow_layout.description().clone(),
                workflow_layout.slug().clone(),
                workflow_graph,
                slug.clone(),
                name.clone(),
            )
            .map_err(|error| ShellError::message(error.to_string()))?;
            interpret_collect_reports(plan)
        }
        Effect::VerifyProjectFromIndex => {
            let project_name = read_project_manifest_name()?;
            let formal_workflows = read_synchronized_formal_workflow_graphs()?;
            let modeled_slices = formal_workflow_slice_details(formal_workflows.clone());
            let modeled_workflows = formal_workflow_layouts(formal_workflows);
            interpret_collect_reports(verify_project(
                project_name,
                ModeledWorkflowLayouts::new(modeled_workflows),
                WorkflowSliceDetails::from_details(modeled_slices),
            ))
        }
        Effect::WriteFile(path, contents) => {
            write_file(path.as_ref(), contents.as_ref()).map(|()| Vec::new())
        }
        Effect::WriteFormalSliceArtifactPreservingAuthoredFacts(source, target, generated) => {
            write_formal_slice_artifact_preserving_authored_facts(
                source.as_ref(),
                target.as_ref(),
                generated.as_ref(),
            )
            .map(|()| Vec::new())
        }
        Effect::WriteFileIfMissing(path, contents) => {
            if Path::new(path.as_ref()).exists() {
                Ok(Vec::new())
            } else {
                write_file(path.as_ref(), contents.as_ref()).map(|()| Vec::new())
            }
        }
        Effect::Report(line) => Ok(vec![line.as_ref().to_owned()]),
        Effect::ReportDocument(contents) => Ok(vec![contents.as_ref().to_owned()]),
    }
}

fn read_project_manifest_name() -> Result<ProjectName, ShellError> {
    fs::read_to_string("emc.toml")
        .map_err(ShellError::io)
        .and_then(|manifest| {
            parse_project_manifest_name(&manifest).map_err(ShellError::project_name)
        })
}

fn read_synchronized_formal_workflow_graphs() -> Result<FormalWorkflowGraphs, ShellError> {
    let lean_graphs = read_formal_workflow_graphs(
        Path::new("model/lean"),
        ".lean",
        "def workflowName :=",
        parse_lean_workflow_graph,
    )?;
    let quint_graphs = read_formal_workflow_graphs(
        Path::new("model/quint"),
        ".qnt",
        "val workflowName =",
        parse_quint_workflow_graph,
    )?;

    let quint_by_slug = formal_graphs_by_slug(quint_graphs, "Quint")?;
    let mut matched_slugs = BTreeSet::new();
    let synchronized_graphs = lean_graphs
        .into_iter()
        .map(|lean_graph| {
            let quint_graph = quint_by_slug
                .get(lean_graph.slug().as_ref())
                .ok_or_else(|| {
                    ShellError::message(format!(
                        "Quint workflow artifact is missing for workflow {}",
                        lean_graph.slug().as_ref()
                    ))
                })?;
            if &lean_graph == quint_graph {
                Ok(lean_graph)
            } else {
                Err(ShellError::message(format!(
                    "Lean and Quint workflow artifacts disagree for workflow {}",
                    lean_graph.slug().as_ref()
                )))
            }
        })
        .inspect(|result| {
            if let Ok(graph) = result {
                matched_slugs.insert(graph.slug().as_ref().to_owned());
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(unmatched_slug) = quint_by_slug
        .keys()
        .find(|slug| !matched_slugs.contains(*slug))
    {
        Err(ShellError::message(format!(
            "Lean workflow artifact is missing for workflow {unmatched_slug}"
        )))
    } else {
        Ok(FormalWorkflowGraphs::from_graphs(synchronized_graphs))
    }
}

fn read_formal_workflow_graphs(
    directory: &Path,
    extension: &str,
    workflow_marker: &str,
    parser: fn(&FileContents) -> Result<FormalWorkflowGraph, FormalGraphError>,
) -> Result<Vec<FormalWorkflowGraph>, ShellError> {
    let mut paths = fs::read_dir(directory)
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    paths.sort();

    paths
        .into_iter()
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == extension.strip_prefix('.')
        })
        .map(|path| {
            fs::read_to_string(&path)
                .map_err(ShellError::io)
                .and_then(|contents| {
                    let file_contents = FileContents::try_new(contents)
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    Ok((path, file_contents))
                })
        })
        .filter_map(|result| match result {
            Ok((path, contents)) if contents.as_ref().contains(workflow_marker) => {
                Some(Ok((path, contents)))
            }
            Ok((_path, _contents)) => None,
            Err(error) => Some(Err(error)),
        })
        .map(|result| {
            let (path, contents) = result?;
            parser(&contents).map_err(|error| {
                ShellError::message(format!(
                    "failed to parse formal workflow artifact {}: {error}",
                    path.display()
                ))
            })
        })
        .collect()
}

fn formal_graphs_by_slug(
    graphs: Vec<FormalWorkflowGraph>,
    artifact_family: &str,
) -> Result<BTreeMap<String, FormalWorkflowGraph>, ShellError> {
    graphs
        .into_iter()
        .try_fold(BTreeMap::new(), |mut indexed, graph| {
            let slug = graph.slug().as_ref().to_owned();
            if indexed.insert(slug.clone(), graph).is_none() {
                Ok(indexed)
            } else {
                Err(ShellError::message(format!(
                    "{artifact_family} workflow artifact slug {slug} is duplicated"
                )))
            }
        })
}

fn formal_workflow_layouts(graphs: FormalWorkflowGraphs) -> Vec<ModeledWorkflowLayout> {
    graphs
        .into_inner()
        .into_iter()
        .map(|graph| formal_workflow_layout(&graph))
        .collect()
}

fn formal_workflow_layout(graph: &FormalWorkflowGraph) -> ModeledWorkflowLayout {
    ModeledWorkflowLayout::new(
        graph.name().clone(),
        graph.description().clone(),
        graph.slug().clone(),
    )
}

fn formal_workflow_slice_details(graphs: FormalWorkflowGraphs) -> Vec<WorkflowSliceDetail> {
    graphs
        .into_inner()
        .into_iter()
        .flat_map(|graph| graph.slice_details().as_slice().to_owned())
        .collect()
}

fn formal_workflow_transitions(graphs: FormalWorkflowGraphs) -> Vec<WorkflowTransitionRecord> {
    graphs
        .into_inner()
        .into_iter()
        .flat_map(|graph| graph.transitions().as_slice().to_owned())
        .collect()
}

fn read_formal_workflow_graph(slug: &WorkflowSlug) -> Result<FormalWorkflowGraph, ShellError> {
    read_synchronized_formal_workflow_graphs()?
        .into_inner()
        .into_iter()
        .find(|graph| graph.slug() == slug)
        .ok_or_else(|| ShellError::message(format!("workflow {} is not modeled", slug.as_ref())))
}

fn read_formal_workflow_artifacts(slug: &WorkflowSlug) -> Result<FileContents, ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    let module_name = module_name_from_raw(graph.name().as_ref());
    formal_artifact_bundle(&[
        format!("model/lean/{module_name}.lean"),
        format!("model/quint/{module_name}.qnt"),
    ])
}

fn read_formal_slice_artifacts(slug: &SliceSlug) -> Result<FileContents, ShellError> {
    read_synchronized_formal_workflow_graphs()?
        .into_inner()
        .into_iter()
        .find_map(|graph| {
            graph
                .slice_details()
                .as_slice()
                .iter()
                .find(|slice| slice.slug() == slug)
                .map(|slice| module_name_from_raw(slice.name().as_ref()))
        })
        .map(|module_name| {
            formal_artifact_bundle(&[
                format!("model/lean/slices/{module_name}.lean"),
                format!("model/quint/slices/{module_name}.qnt"),
            ])
        })
        .unwrap_or_else(|| {
            Err(ShellError::message(format!(
                "slice {} is not referenced by any modeled workflow",
                slug.as_ref()
            )))
        })
}

struct FormalSliceArtifactDocuments {
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
}

struct FormalWorkflowArtifactDocuments {
    lean_path: ProjectPath,
    lean_contents: FileContents,
    quint_path: ProjectPath,
    quint_contents: FileContents,
}

fn read_formal_workflow_artifact_paths_and_contents(
    slug: &WorkflowSlug,
) -> Result<FormalWorkflowArtifactDocuments, ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    let module_name = module_name_from_raw(graph.name().as_ref());
    let lean_path = format!("model/lean/{module_name}.lean");
    let quint_path = format!("model/quint/{module_name}.qnt");
    Ok(FormalWorkflowArtifactDocuments {
        lean_path: project_path(lean_path.clone())?,
        lean_contents: read_file_contents(&lean_path)?,
        quint_path: project_path(quint_path.clone())?,
        quint_contents: read_file_contents(&quint_path)?,
    })
}

fn read_formal_slice_artifact_paths_and_contents(
    slug: &SliceSlug,
) -> Result<FormalSliceArtifactDocuments, ShellError> {
    let module_name = find_formal_slice_module_name(slug)?;
    let lean_path = format!("model/lean/slices/{module_name}.lean");
    let quint_path = format!("model/quint/slices/{module_name}.qnt");
    Ok(FormalSliceArtifactDocuments {
        lean_path: project_path(lean_path.clone())?,
        lean_contents: read_file_contents(&lean_path)?,
        quint_path: project_path(quint_path.clone())?,
        quint_contents: read_file_contents(&quint_path)?,
    })
}

fn find_formal_slice_module_name(slug: &SliceSlug) -> Result<String, ShellError> {
    read_synchronized_formal_workflow_graphs()?
        .into_inner()
        .into_iter()
        .find_map(|graph| {
            graph
                .slice_details()
                .as_slice()
                .iter()
                .find(|slice| slice.slug() == slug)
                .map(|slice| module_name_from_raw(slice.name().as_ref()))
        })
        .ok_or_else(|| {
            ShellError::message(format!(
                "slice {} is not referenced by any modeled workflow",
                slug.as_ref()
            ))
        })
}

fn project_path(path: String) -> Result<ProjectPath, ShellError> {
    ProjectPath::try_new(path).map_err(ShellError::project_path)
}

fn read_file_contents(path: &str) -> Result<FileContents, ShellError> {
    FileContents::try_new(fs::read_to_string(Path::new(path)).map_err(ShellError::io)?)
        .map_err(|error| ShellError::message(error.to_string()))
}

fn formal_artifact_bundle(paths: &[String]) -> Result<FileContents, ShellError> {
    let mut bundle = String::new();
    for path in paths {
        let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
        bundle.push_str("# ");
        bundle.push_str(path);
        bundle.push('\n');
        bundle.push_str(&contents);
        if !contents.ends_with('\n') {
            bundle.push('\n');
        }
        bundle.push('\n');
    }
    FileContents::try_new(bundle).map_err(|error| ShellError::message(error.to_string()))
}

fn read_formal_workflow_layout_and_graph(
    slug: &WorkflowSlug,
) -> Result<(ModeledWorkflowLayout, FormalWorkflowGraph), ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    Ok((formal_workflow_layout(&graph), graph))
}

fn indexed_formal_workflow_graphs(graphs: &[FormalWorkflowGraph]) -> Vec<IndexedWorkflowGraph> {
    graphs
        .iter()
        .map(|graph| IndexedWorkflowGraph::new(graph.slug().clone(), graph.clone()))
        .collect()
}

fn find_formal_workflow_containing_slice(
    slug: &SliceSlug,
) -> Result<(ModeledWorkflowLayout, FormalWorkflowGraph), ShellError> {
    read_synchronized_formal_workflow_graphs()?
        .into_inner()
        .into_iter()
        .find(|graph| {
            graph
                .slice_details()
                .as_slice()
                .iter()
                .any(|slice| slice.slug() == slug)
        })
        .map(|graph| (formal_workflow_layout(&graph), graph))
        .ok_or_else(|| {
            ShellError::message(format!(
                "slice {} is not referenced by any indexed workflow",
                slug.as_ref()
            ))
        })
}

fn require_file_contents(path: &str, expected: &str, message: &str) -> Result<(), ShellError> {
    let actual = fs::read_to_string(Path::new(path))
        .map_err(|_error| ShellError::message(message.to_owned()))?;
    if actual == expected {
        Ok(())
    } else {
        Err(ShellError::message(message.to_owned()))
    }
}

fn require_file_contents_with_authored_formal_facts(
    path: &str,
    expected: &str,
    message: &str,
) -> Result<(), ShellError> {
    let actual = fs::read_to_string(Path::new(path))
        .map_err(|_error| ShellError::message(message.to_owned()))?;
    if normalize_authored_formal_fact_lists(&actual)
        == normalize_authored_formal_fact_lists(expected)
    {
        Ok(())
    } else {
        Err(ShellError::message(message.to_owned()))
    }
}

fn normalize_authored_formal_fact_lists(contents: &str) -> String {
    const MARKERS: &[&str] = &[
        "def sliceCommands : List String := ",
        "def sliceCommandDefinitions : List CommandDefinition := ",
        "def sliceReferencedCommands : List String := ",
        "def sliceAutomations : List AutomationDefinition := ",
        "def sliceTranslations : List TranslationDefinition := ",
        "def sliceBoardElements : List BoardElement := ",
        "def sliceBoardConnections : List BoardConnection := ",
        "def sliceOutcomeDefinitions : List OutcomeDefinition := ",
        "def sliceEvents : List String := ",
        "def sliceStreams : List StreamDefinition := ",
        "def sliceExternalPayloads : List ExternalPayloadDefinition := ",
        "def sliceEventDefinitions : List EventDefinition := ",
        "def sliceReadModels : List String := ",
        "def sliceReadModelDefinitions : List ReadModelDefinition := ",
        "def sliceViews : List String := ",
        "def sliceViewDefinitions : List ViewDefinition := ",
        "def sliceAcceptanceScenarios : List EventModelScenario := ",
        "def sliceContractScenarios : List EventModelScenario := ",
        "def sliceBitLevelDataFlows : List BitLevelDataFlow := ",
        "val sliceCommands: List[str] = ",
        "val sliceCommandDefinitions: List[CommandDefinition] = ",
        "val sliceReferencedCommands: List[str] = ",
        "val sliceAutomations: List[AutomationDefinition] = ",
        "val sliceTranslations: List[TranslationDefinition] = ",
        "val sliceBoardElements: List[BoardElement] = ",
        "val sliceBoardConnections: List[BoardConnection] = ",
        "val sliceOutcomeDefinitions: List[OutcomeDefinition] = ",
        "val sliceEvents: List[str] = ",
        "val sliceStreams: List[StreamDefinition] = ",
        "val sliceExternalPayloads: List[ExternalPayloadDefinition] = ",
        "val sliceEventDefinitions: List[EventDefinition] = ",
        "val sliceReadModels: List[str] = ",
        "val sliceReadModelDefinitions: List[ReadModelDefinition] = ",
        "val sliceViews: List[str] = ",
        "val sliceViewDefinitions: List[ViewDefinition] = ",
        "val sliceAcceptanceScenarios: List[EventModelScenario] = ",
        "val sliceContractScenarios: List[EventModelScenario] = ",
        "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = ",
    ];
    let mut normalized = contents
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            MARKERS
                .iter()
                .find_map(|marker| {
                    declaration
                        .starts_with(marker)
                        .then(|| format!("{indentation}{marker}[]"))
                })
                .unwrap_or_else(|| line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n");
    if contents.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn write_formal_slice_artifact_preserving_authored_facts(
    source: &str,
    target: &str,
    generated: &str,
) -> Result<(), ShellError> {
    let contents = match fs::read_to_string(Path::new(source)) {
        Ok(existing) => preserve_authored_formal_fact_lists(&existing, generated),
        Err(error) if error.kind() == io::ErrorKind::NotFound => generated.to_owned(),
        Err(error) => return Err(ShellError::io(error)),
    };
    write_file(target, &contents)
}

fn preserve_authored_formal_fact_lists(existing: &str, generated: &str) -> String {
    const MARKERS: &[&str] = &[
        "def sliceCommands : List String := ",
        "def sliceCommandDefinitions : List CommandDefinition := ",
        "def sliceReferencedCommands : List String := ",
        "def sliceAutomations : List AutomationDefinition := ",
        "def sliceTranslations : List TranslationDefinition := ",
        "def sliceBoardElements : List BoardElement := ",
        "def sliceBoardConnections : List BoardConnection := ",
        "def sliceOutcomeDefinitions : List OutcomeDefinition := ",
        "def sliceEvents : List String := ",
        "def sliceStreams : List StreamDefinition := ",
        "def sliceExternalPayloads : List ExternalPayloadDefinition := ",
        "def sliceEventDefinitions : List EventDefinition := ",
        "def sliceReadModels : List String := ",
        "def sliceReadModelDefinitions : List ReadModelDefinition := ",
        "def sliceViews : List String := ",
        "def sliceViewDefinitions : List ViewDefinition := ",
        "def sliceAcceptanceScenarios : List EventModelScenario := ",
        "def sliceContractScenarios : List EventModelScenario := ",
        "def sliceBitLevelDataFlows : List BitLevelDataFlow := ",
        "val sliceCommands: List[str] = ",
        "val sliceCommandDefinitions: List[CommandDefinition] = ",
        "val sliceReferencedCommands: List[str] = ",
        "val sliceAutomations: List[AutomationDefinition] = ",
        "val sliceTranslations: List[TranslationDefinition] = ",
        "val sliceBoardElements: List[BoardElement] = ",
        "val sliceBoardConnections: List[BoardConnection] = ",
        "val sliceOutcomeDefinitions: List[OutcomeDefinition] = ",
        "val sliceEvents: List[str] = ",
        "val sliceStreams: List[StreamDefinition] = ",
        "val sliceExternalPayloads: List[ExternalPayloadDefinition] = ",
        "val sliceEventDefinitions: List[EventDefinition] = ",
        "val sliceReadModels: List[str] = ",
        "val sliceReadModelDefinitions: List[ReadModelDefinition] = ",
        "val sliceViews: List[str] = ",
        "val sliceViewDefinitions: List[ViewDefinition] = ",
        "val sliceAcceptanceScenarios: List[EventModelScenario] = ",
        "val sliceContractScenarios: List[EventModelScenario] = ",
        "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = ",
    ];
    let existing_declarations = MARKERS
        .iter()
        .filter_map(|marker| {
            authored_formal_fact_declaration(existing, marker)
                .map(|declaration| (*marker, declaration))
        })
        .collect::<Vec<_>>();
    let mut preserved = generated
        .lines()
        .map(|line| {
            let indentation_length = line.len() - line.trim_start().len();
            let (indentation, declaration) = line.split_at(indentation_length);
            existing_declarations
                .iter()
                .find_map(|(marker, authored_declaration)| {
                    declaration
                        .starts_with(marker)
                        .then(|| format!("{indentation}{authored_declaration}"))
                })
                .unwrap_or_else(|| line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n");
    if generated.ends_with('\n') {
        preserved.push('\n');
    }
    preserved
}

fn authored_formal_fact_declaration(contents: &str, marker: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let declaration = line.trim_start();
        declaration
            .starts_with(marker)
            .then(|| declaration.to_owned())
    })
}

fn require_json_object_keys_unique(path: &str, message: &str) -> Result<(), ShellError> {
    let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
    let file_contents = FileContents::try_new(contents)
        .map_err(|_error| ShellError::message(message.to_owned()))?;
    JsonObjectDocument::reject_duplicate_keys(&file_contents)
        .map_err(|_error| ShellError::message(message.to_owned()))
}

fn require_only_modeled_artifacts(
    path: &str,
    extension: &str,
    allowed_paths: &[ProjectPath],
    message: &str,
) -> Result<(), ShellError> {
    let allowed_file_names = allowed_artifact_file_names(allowed_paths);
    let mut artifact_files = fs::read_dir(Path::new(path))
        .map_err(ShellError::io)?
        .map(|entry| entry.map(|directory_entry| directory_entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    artifact_files.sort();

    artifact_files
        .into_iter()
        .filter_map(|artifact_path| {
            artifact_path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .filter(|file_name| file_name.ends_with(extension))
                .map(str::to_owned)
        })
        .find(|file_name| !allowed_file_names.contains(file_name))
        .map_or(Ok(()), |file_name| {
            Err(ShellError::message(format!("{message} for {file_name}")))
        })
}

fn allowed_artifact_file_names(allowed_paths: &[ProjectPath]) -> BTreeSet<String> {
    allowed_paths
        .iter()
        .filter_map(|path| {
            Path::new(path.as_ref())
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(str::to_owned)
        })
        .collect()
}

fn require_clean_review_record(
    path: &str,
    workflow_slug: &WorkflowSlug,
    fallback_message: &str,
) -> Result<(), ShellError> {
    let contents = fs::read_to_string(Path::new(path)).map_err(ShellError::io)?;
    let record_contents = FileContents::try_new(contents)
        .map_err(|_error| ShellError::message(fallback_message.to_owned()))?;
    let record = ReviewRecordDocument::parse(&record_contents)
        .map_err(|_error| ShellError::message(fallback_message.to_owned()))?;
    let expected_workflow_slug = review_record_workflow_slug(path)?;
    if !record.matches_workflow(&expected_workflow_slug) {
        let observed = record
            .workflow_slug()
            .map_or_else(String::new, |workflow_slug| {
                workflow_slug.as_ref().to_owned()
            });
        return Err(ShellError::message(format!(
            "review record workflow '{observed}' does not match '{expected_workflow_slug}'"
        )));
    }
    let current_digest = formal_model_content_digest(workflow_slug)?;
    if !record.is_clean() {
        if record.current_mandatory_findings_include(&current_digest) {
            return Err(ShellError::message(
                "mandatory review findings remain for current model digest",
            ));
        }
        if record.has_mandatory_findings() && !record.model_content_digest_matches(&current_digest)
        {
            return Err(ShellError::message(
                "corrected workflow requires clean follow-up review",
            ));
        }
        return Err(ShellError::message(fallback_message.to_owned()));
    }
    if !record.model_content_digest_matches(&current_digest) {
        return Err(ShellError::message(
            "clean review is stale for current model digest",
        ));
    }
    if !record.has_category_results() {
        return Err(ShellError::message(fallback_message.to_owned()));
    }

    let required_categories = required_review_categories()?;
    match record.first_non_clean_category(&required_categories) {
        Some(ReviewCategoryFinding::NotClean(category)) => Err(ShellError::message(format!(
            "review category '{category}' is not clean"
        ))),
        Some(ReviewCategoryFinding::Missing(category)) => Err(ShellError::message(format!(
            "clean review is missing category '{category}'"
        ))),
        None => Ok(()),
    }
}

fn review_record_workflow_slug(path: &str) -> Result<WorkflowSlug, ShellError> {
    Path::new(path)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .and_then(|file_name| file_name.strip_suffix(".review.json"))
        .ok_or_else(|| ShellError::message("review record path is invalid"))
        .and_then(|slug| {
            WorkflowSlug::try_new(slug.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })
}

fn required_review_categories() -> Result<Vec<ReviewRuleName>, ShellError> {
    REQUIRED_REVIEW_CATEGORIES
        .iter()
        .map(|category| {
            ReviewRuleName::try_new((*category).to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .collect()
}

fn formal_model_content_digest(slug: &WorkflowSlug) -> Result<ArtifactDigest, ShellError> {
    let graph = read_formal_workflow_graph(slug)?;
    let mut digest = StableDigest::new();
    write_formal_artifact_digest(
        &mut digest,
        &format!(
            "model/lean/{}.lean",
            module_name_from_raw(graph.name().as_ref())
        ),
    )?;
    write_formal_artifact_digest(
        &mut digest,
        &format!(
            "model/quint/{}.qnt",
            module_name_from_raw(graph.name().as_ref())
        ),
    )?;
    graph
        .slice_details()
        .as_slice()
        .iter()
        .try_for_each(|slice| {
            let module_name = module_name_from_raw(slice.name().as_ref());
            write_formal_artifact_digest(
                &mut digest,
                &format!("model/lean/slices/{module_name}.lean"),
            )?;
            write_formal_artifact_digest(
                &mut digest,
                &format!("model/quint/slices/{module_name}.qnt"),
            )
        })?;
    ArtifactDigest::try_new(digest.finish()).map_err(|error| ShellError::message(error.to_string()))
}

fn write_formal_artifact_digest(digest: &mut StableDigest, path: &str) -> Result<(), ShellError> {
    let contents = fs::read_to_string(path).map_err(ShellError::io)?;
    digest.write(path);
    digest.write(&contents);
    Ok(())
}

struct StableDigest {
    value: u64,
}

impl StableDigest {
    fn new() -> Self {
        Self {
            value: 0xcbf2_9ce4_8422_2325,
        }
    }

    fn write(&mut self, value: &str) {
        value.as_bytes().iter().for_each(|byte| {
            self.value ^= u64::from(*byte);
            self.value = self.value.wrapping_mul(0x0000_0100_0000_01b3);
        });
    }

    fn finish(self) -> String {
        format!("emc-fnv1a64:{:016x}", self.value)
    }
}

fn artifact_contains_one_canonical_declaration(
    artifact_contents: &str,
    prefix: &str,
    marker: &str,
) -> bool {
    let mut declarations = artifact_contents
        .lines()
        .filter_map(|line| canonical_declaration_line(line, prefix));

    matches!(
        (declarations.next(), declarations.next()),
        (Some(declaration), None) if declaration == marker
    )
}

fn artifact_contains_one_digest_marker(artifact_contents: &str, digest: &str) -> bool {
    let mut declarations = artifact_contents
        .lines()
        .filter_map(canonical_digest_marker_line);

    matches!(
        (declarations.next(), declarations.next()),
        (Some(declaration), None) if declaration == digest
    )
}

fn canonical_digest_marker_line(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("-- EMC-DIGEST: ")
        .or_else(|| trimmed.strip_prefix("// EMC-DIGEST: "))
}

fn canonical_declaration_line<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.starts_with(' ') && line.starts_with(prefix) {
        Some(line)
    } else {
        let trimmed = line.trim_start();
        trimmed.starts_with(prefix).then_some(trimmed)
    }
}

fn module_name_from_raw(raw: &str) -> String {
    let mut capitalize_next = true;
    raw.chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                let next = if capitalize_next {
                    character.to_ascii_uppercase()
                } else {
                    character
                };
                capitalize_next = false;
                Some(next)
            } else {
                capitalize_next = true;
                None
            }
        })
        .collect()
}

fn write_file(path: &str, contents: &str) -> Result<(), ShellError> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(ShellError::io)?;
    }
    fs::write(Path::new(path), contents).map_err(ShellError::io)
}

fn copy_directory(source: &str, target: &str) -> Result<(), ShellError> {
    let target_path = Path::new(target);
    if target_path.exists() {
        fs::remove_dir_all(target_path).map_err(ShellError::io)?;
    }
    copy_directory_path(Path::new(source), target_path)
}

fn copy_directory_path(source: &Path, target: &Path) -> Result<(), ShellError> {
    fs::create_dir_all(target).map_err(ShellError::io)?;
    let mut entries = fs::read_dir(source)
        .map_err(ShellError::io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ShellError::io)?;
    entries.sort_by_key(|entry| entry.path());

    entries.into_iter().try_for_each(|entry| {
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory_path(&source_path, &target_path)
        } else {
            fs::copy(source_path, target_path)
                .map(|_bytes| ())
                .map_err(ShellError::io)
        }
    })
}

fn remove_directory_if_present(path: &str) -> Result<(), ShellError> {
    match fs::remove_dir_all(Path::new(path)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ShellError::io(error)),
    }
}

fn remove_file_if_present(path: &str) -> Result<(), ShellError> {
    match fs::remove_file(Path::new(path)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ShellError::io(error)),
    }
}

fn run_process(invocation: &ProcessInvocation) -> Result<Vec<String>, ShellError> {
    let arguments = process_arguments(invocation);
    let output = Command::new(invocation.program().as_ref())
        .args(arguments)
        .output()
        .map_err(|error| {
            ShellError::message(format!(
                "failed to run {}: {}. Use `nix run . -- verify` from this repository or install the pinned EMC tooling from the Nix package",
                invocation.program().as_ref(),
                error
            ))
        })?;

    if output.status.success() {
        Ok(vec![invocation.success().as_ref().to_owned()])
    } else {
        Err(ShellError::message(format!(
            "{} failed with {}{}. Run `emc check` to confirm generated artifacts are synchronized, then run `emc verify` again",
            verification_label(invocation),
            output.status,
            process_diagnostics(&output.stdout, &output.stderr)
        )))
    }
}

fn process_arguments(invocation: &ProcessInvocation) -> Vec<String> {
    let mut arguments = invocation
        .arguments()
        .iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<_>>();

    if invocation.program().as_ref() == "quint"
        && arguments.first().map(String::as_str) == Some("verify")
        && !arguments
            .iter()
            .any(|argument| argument == "--server-endpoint")
    {
        let input_position = arguments.len().saturating_sub(1);
        arguments.insert(input_position, "--server-endpoint".to_owned());
        arguments.insert(input_position + 1, quint_server_endpoint());
    }

    arguments
}

fn quint_server_endpoint() -> String {
    env::var("EMC_QUINT_SERVER_ENDPOINT").unwrap_or_else(|_| {
        let port = 20_000 + (process::id() % 40_000);
        format!("127.0.0.1:{port}")
    })
}

fn process_diagnostics(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    let diagnostics = [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if diagnostics.is_empty() {
        String::new()
    } else {
        format!(":\n{diagnostics}")
    }
}

fn verification_label(invocation: &ProcessInvocation) -> &str {
    if invocation.success().as_ref().starts_with("Lean4") {
        "Lean4 verification"
    } else if invocation.success().as_ref().starts_with("Quint") {
        "Quint verification"
    } else {
        "verification command"
    }
}
