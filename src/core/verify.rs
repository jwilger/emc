use std::collections::BTreeSet;

use crate::core::effect::{
    Effect, EffectPlan, ProcessArgument, ProcessInvocation, ProgramName, ReportLine,
};
use crate::core::layout::{ModeledWorkflowLayout, ModeledWorkflowLayouts};
use crate::core::project::ProjectName;
use crate::core::types::{WorkflowSliceDetail, WorkflowSliceDetails};

pub fn verify_project(
    project_name: ProjectName,
    modeled_workflows: ModeledWorkflowLayouts,
    workflow_slice_details: WorkflowSliceDetails,
) -> EffectPlan {
    EffectPlan::new(
        verify_project_root(project_name)
            .into_iter()
            .chain(
                modeled_workflows
                    .into_inner()
                    .into_iter()
                    .flat_map(verify_modeled_workflow),
            )
            .chain(verify_modeled_slices(workflow_slice_details.into_inner()))
            .collect(),
    )
}

fn verify_project_root(project_name: ProjectName) -> Vec<Effect> {
    let module_name = module_name_from_raw(project_name.as_ref());
    vec![
        Effect::RunProcess(ProcessInvocation::new(
            program_name("lake"),
            vec![
                process_argument("env"),
                process_argument("lean"),
                process_argument(format!("model/lean/{module_name}.lean")),
            ],
            report_line("Lean4 artifacts verified"),
        )),
        Effect::RunProcess(ProcessInvocation::new(
            program_name("quint"),
            vec![
                process_argument("typecheck"),
                process_argument(format!("model/quint/{module_name}.qnt")),
            ],
            report_line("Quint artifacts verified"),
        )),
        Effect::RunProcess(ProcessInvocation::new(
            program_name("quint"),
            vec![
                process_argument("verify"),
                process_argument("--invariant"),
                process_argument(
                    "modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared",
                ),
                process_argument(format!("model/quint/{module_name}.qnt")),
            ],
            report_line("Quint artifacts verified"),
        )),
    ]
}

fn verify_modeled_workflow(workflow: ModeledWorkflowLayout) -> Vec<Effect> {
    vec![
        Effect::RunProcess(ProcessInvocation::new(
            program_name("lake"),
            vec![
                process_argument("env"),
                process_argument("lean"),
                process_argument(workflow.lean_artifact_path().as_ref().to_owned()),
            ],
            report_line("Lean4 artifacts verified"),
        )),
        Effect::RunProcess(ProcessInvocation::new(
            program_name("quint"),
            vec![
                process_argument("verify"),
                process_argument("--invariant"),
                process_argument(
                    "workflowIdentityStable,workflowSliceDetailsComplete,workflowSliceModulesComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowEventTransitionsAreSharedByEndpointSlices,workflowNavigationTransitionsResolveControlsAndViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates",
                ),
                process_argument(workflow.quint_artifact_path().as_ref().to_owned()),
            ],
            report_line("Quint artifacts verified"),
        )),
    ]
}

fn verify_modeled_slices(workflow_slice_details: Vec<WorkflowSliceDetail>) -> Vec<Effect> {
    workflow_slice_details
        .into_iter()
        .map(|slice| module_name_from_raw(slice.name().as_ref()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .flat_map(verify_modeled_slice)
        .collect()
}

fn verify_modeled_slice(module_name: String) -> Vec<Effect> {
    vec![
        Effect::RunProcess(ProcessInvocation::new(
            program_name("lake"),
            vec![
                process_argument("env"),
                process_argument("lean"),
                process_argument(format!("model/lean/slices/{module_name}.lean")),
            ],
            report_line("Lean4 artifacts verified"),
        )),
        Effect::RunProcess(ProcessInvocation::new(
            program_name("quint"),
            vec![
                process_argument("verify"),
                process_argument("--invariant"),
                process_argument(
                    "sliceIdentityStable,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceNamedDefinitionsAreUniquelyOwned,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputsHaveDescriptions,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTypesAreDeclared,viewControlModeledViewNavigationTargetsResolve,viewControlExternalWorkflowNavigationTargetsNamed,viewControlExternalSystemNavigationTargetsHaveContracts,viewControlNavigationTargetsAreComplete",
                ),
                process_argument(format!("model/quint/slices/{module_name}.qnt")),
            ],
            report_line("Quint artifacts verified"),
        )),
    ]
}

fn module_name_from_raw(raw: &str) -> String {
    raw.split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .map(|first| {
                    first.to_ascii_uppercase().to_string()
                        + characters.as_str().to_ascii_lowercase().as_str()
                })
                .unwrap_or_default()
        })
        .collect()
}

fn program_name(value: impl Into<String>) -> ProgramName {
    ProgramName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated program name must be valid: {error}");
    })
}

fn process_argument(value: impl Into<String>) -> ProcessArgument {
    ProcessArgument::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated process argument must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated report line must be valid: {error}");
    })
}
