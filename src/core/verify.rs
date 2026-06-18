// Copyright 2026 John Wilger

use std::collections::BTreeSet;

use crate::core::effect::{
    Effect, EffectPlan, ProcessArgument, ProcessInvocation, ProcessInvocations, ProgramName,
    ReportLine,
};
use crate::core::layout::{ModeledWorkflowLayout, ModeledWorkflowLayouts};
use crate::core::project::ProjectName;
use crate::core::types::WorkflowSliceDetails;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct QuintInvariantName(&'static str);

impl QuintInvariantName {
    pub(crate) const MODEL_DATA_FLOW_SOURCE_CHAINS_PRESERVE_BIT_ENCODING_SEMANTICS: Self =
        Self("modelDataFlowSourceChainsPreserveBitEncodingSemantics");
    pub(crate) const MODEL_WORKFLOW_BEHAVIOR_SURFACE_IS_COMPLETE: Self =
        Self("modelWorkflowBehaviorSurfaceIsComplete");
    pub(crate) const WORKFLOW_ONLY_EVENTS_MAY_BE_SHARED_ACROSS_SLICES: Self =
        Self("workflowOnlyEventsMayBeSharedAcrossSlices");
    pub(crate) const STATE_VIEW_SLICES_REPRESENT_SINGLE_VIEW_PROJECTION_BOUNDARY: Self =
        Self("stateViewSlicesRepresentSingleViewProjectionBoundary");
}

impl AsRef<str> for QuintInvariantName {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct QuintInvariantSet {
    invariants: &'static [QuintInvariantName],
}

impl QuintInvariantSet {
    pub(crate) fn project_root() -> Self {
        Self {
            invariants: PROJECT_ROOT_INVARIANTS,
        }
    }

    pub(crate) fn workflow() -> Self {
        Self {
            invariants: WORKFLOW_INVARIANTS,
        }
    }

    pub(crate) fn slice() -> Self {
        Self {
            invariants: SLICE_INVARIANTS,
        }
    }

    #[cfg(test)]
    pub(crate) fn contains(&self, invariant: QuintInvariantName) -> bool {
        self.invariants.contains(&invariant)
    }

    pub(crate) fn as_process_argument(&self) -> ProcessArgument {
        process_argument(
            self.invariants
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

const PROJECT_ROOT_INVARIANTS: &[QuintInvariantName] = &[
    QuintInvariantName("modelIdentityStable"),
    QuintInvariantName("modelVersionStable"),
    QuintInvariantName("modelDigestStable"),
    QuintInvariantName("modelWorkflowsAreDeclared"),
    QuintInvariantName("modelSlicesAreDeclared"),
    QuintInvariantName("modelSliceModulesAreDeclared"),
    QuintInvariantName("modelScenariosAreDeclared"),
    QuintInvariantName("modelScenarioDefinitionsAreDeclared"),
    QuintInvariantName("modelWorkflowCompositionStructureComplete"),
    QuintInvariantName::MODEL_WORKFLOW_BEHAVIOR_SURFACE_IS_COMPLETE,
    QuintInvariantName("modelScenarioDefinitionsHaveGwt"),
    QuintInvariantName("modelScenarioKindsAreFirstClass"),
    QuintInvariantName("modelDataFlowsAreDeclared"),
    QuintInvariantName("modelDataFlowsAreBitComplete"),
    QuintInvariantName("modelDataFlowSourceKindsAreModeled"),
    QuintInvariantName("modelDataFlowModeledSourcesResolve"),
    QuintInvariantName("modelDataFlowSourceChainsReachOriginals"),
    QuintInvariantName::MODEL_DATA_FLOW_SOURCE_CHAINS_PRESERVE_BIT_ENCODING_SEMANTICS,
    QuintInvariantName("modelDataFlowTransformationsAreModeled"),
    QuintInvariantName("modelMeaningfulDataFlowsAreCovered"),
    QuintInvariantName("modelDataFlowSourceBitEncodingsMatchModeledSources"),
    QuintInvariantName("modelViewFieldBitEncodingsMatchDataFlows"),
    QuintInvariantName("modelExternalPayloadFieldBitEncodingsMatchDataFlows"),
    QuintInvariantName("modelOutcomesAreDeclared"),
    QuintInvariantName("modelCommandErrorsAreDeclared"),
    QuintInvariantName("modelCommandsAreDeclared"),
    QuintInvariantName("modelCommandInputsAreDeclared"),
    QuintInvariantName("modelCommandInputsHaveProvenance"),
    QuintInvariantName("modelCommandInputsTraceToInvocationSources"),
    QuintInvariantName("modelReadModelsAreDeclared"),
    QuintInvariantName("modelReadModelDefinitionsAreDeclared"),
    QuintInvariantName("modelReadModelFieldsAreDeclared"),
    QuintInvariantName("modelReadModelFieldSourcesAreComplete"),
    QuintInvariantName("modelViewFieldSourcesAreComplete"),
    QuintInvariantName("modelViewFieldReadModelFieldSourcesResolve"),
    QuintInvariantName("modelDisplayedDataTraceToOriginalProvenance"),
    QuintInvariantName("modelExternalPayloadFieldsHaveProvenance"),
    QuintInvariantName("modelViewsAreDeclared"),
    QuintInvariantName("modelViewDefinitionsAreDeclared"),
    QuintInvariantName("modelViewControlsAreDeclared"),
    QuintInvariantName("modelBoardElementsAreDeclared"),
    QuintInvariantName("modelBoardConnectionsAreDeclared"),
    QuintInvariantName("modelViewFieldsAreDeclared"),
    QuintInvariantName("modelAutomationsAreDeclared"),
    QuintInvariantName("modelAutomationDefinitionsAreDeclared"),
    QuintInvariantName("modelTranslationsAreDeclared"),
    QuintInvariantName("modelTranslationDefinitionsAreDeclared"),
    QuintInvariantName("modelExternalPayloadsAreDeclared"),
    QuintInvariantName("modelExternalPayloadFieldsAreDeclared"),
    QuintInvariantName("modelStreamsAreDeclared"),
    QuintInvariantName("modelEventsAreDeclared"),
    QuintInvariantName("modelEventAttributesAreDeclared"),
    QuintInvariantName("modelViewControlsProvideCommandInputs"),
];

const WORKFLOW_INVARIANTS: &[QuintInvariantName] = &[
    QuintInvariantName("workflowIdentityStable"),
    QuintInvariantName("workflowSliceDetailsComplete"),
    QuintInvariantName("workflowSliceModulesComplete"),
    QuintInvariantName("workflowTransitionsStructured"),
    QuintInvariantName("workflowTransitionSourcesResolve"),
    QuintInvariantName("workflowTransitionTargetsResolve"),
    QuintInvariantName("workflowStepRelationshipsAreAllowed"),
    QuintInvariantName("workflowStepSlugsAreUnique"),
    QuintInvariantName("workflowHasExactlyOneEntryStep"),
    QuintInvariantName("workflowMainStepsHaveIncomingReachability"),
    QuintInvariantName("workflowNonSupportingStepsReachableFromEntry"),
    QuintInvariantName("workflowBranchAndAlternateStepsHaveTriggerOrRationale"),
    QuintInvariantName("workflowTransitionsHaveModeledKinds"),
    QuintInvariantName("workflowExitsNameTargetsAndRationale"),
    QuintInvariantName("workflowExternallyRelevantOutcomesHandled"),
    QuintInvariantName("workflowOutcomesSourceResolve"),
    QuintInvariantName("workflowCommandErrorsSourceResolve"),
    QuintInvariantName("workflowTransitionsDoNotUseCommandErrorsAsOutcomes"),
    QuintInvariantName("workflowNonEventDefinitionsAreUniquelyOwned"),
    QuintInvariantName("workflowSharedEventDefinitionsHaveIdenticalIdentity"),
    QuintInvariantName::WORKFLOW_ONLY_EVENTS_MAY_BE_SHARED_ACROSS_SLICES,
    QuintInvariantName("workflowCommandTransitionsTargetOwnedCommands"),
    QuintInvariantName("workflowCommandTransitionsSourceOwnedControls"),
    QuintInvariantName("workflowCommandTransitionsResolveControlsAndCommands"),
    QuintInvariantName("workflowStateViewCommandTransitionsTargetStateChanges"),
    QuintInvariantName("workflowEventTransitionsAreSharedByEndpointSlices"),
    QuintInvariantName("workflowEventTransitionsHaveParticipatingEndpointEvents"),
    QuintInvariantName("workflowNavigationTransitionsResolveControlsAndViews"),
    QuintInvariantName("workflowNavigationTransitionsResolveToEntryViews"),
    QuintInvariantName("workflowExternalTriggersDeclarePayloadContracts"),
    QuintInvariantName("workflowExternalTriggerPayloadContractsHaveProvenance"),
    QuintInvariantName("workflowTransitionsHaveRequiredEvidence"),
    QuintInvariantName("workflowEntryLifecycleStatesCoverRequiredStates"),
];

const SLICE_INVARIANTS: &[QuintInvariantName] = &[
    QuintInvariantName("sliceIdentityStable"),
    QuintInvariantName("sliceRepresentsOneCoherentModelUnit"),
    QuintInvariantName("sliceRepresentsSmallestUsefulBehaviorBoundary"),
    QuintInvariantName("sliceStateChangeRequiresEvent"),
    QuintInvariantName("sliceBitLevelDataFlowsStructured"),
    QuintInvariantName("modeledDataFlowsAreBitComplete"),
    QuintInvariantName("sliceScenariosHaveGwt"),
    QuintInvariantName("sliceScenarioNamesAreUnique"),
    QuintInvariantName("sliceNamedDefinitionsAreUniquelyOwned"),
    QuintInvariantName("sliceScenarioStreamsResolve"),
    QuintInvariantName("stateChangeScenariosNameStreams"),
    QuintInvariantName("acceptanceScenariosAreUserFacing"),
    QuintInvariantName("stateViewReadModelsHaveProjectorContracts"),
    QuintInvariantName("contractScenariosTargetKnownDefinitions"),
    QuintInvariantName("contractScenariosCoverModeledContracts"),
    QuintInvariantName("commandInputsHaveAllowedSources"),
    QuintInvariantName("commandInputsHaveProvenance"),
    QuintInvariantName("commandInputsWithoutIssuingControlsHaveProvenance"),
    QuintInvariantName("commandSessionInputsHaveDescriptions"),
    QuintInvariantName("commandInputsTraceToInvocationSources"),
    QuintInvariantName("commandInputsSourcedFromEventStreamsResolve"),
    QuintInvariantName("commandInputsSourcedFromExternalPayloadsResolve"),
    QuintInvariantName("commandInputsSourcedFromGeneratedValuesHaveCoordinates"),
    QuintInvariantName("commandInputsSourcedFromSessionValuesHaveCoordinates"),
    QuintInvariantName("commandErrorsAreDeclared"),
    QuintInvariantName("commandErrorsHaveAllowedRecovery"),
    QuintInvariantName("commandErrorsHaveScenarioCoverage"),
    QuintInvariantName("scenarioErrorReferencesAreDeclared"),
    QuintInvariantName("singletonCommandsDeclareRepeatBehavior"),
    QuintInvariantName("automationSlicesDeclareTriggers"),
    QuintInvariantName("automationSlicesRepresentOneReaction"),
    QuintInvariantName("automationsIssueKnownCommands"),
    QuintInvariantName("automationsHandleCommandErrors"),
    QuintInvariantName("translationSlicesDeclareExternalContracts"),
    QuintInvariantName("externalBoundariesHavePayloadContractsAndFieldProvenance"),
    QuintInvariantName("translationsTargetKnownCommands"),
    QuintInvariantName("translationsReferenceObservedExternalEvents"),
    QuintInvariantName("boardLanesAreCanonical"),
    QuintInvariantName("boardElementsUseCanonicalLanes"),
    QuintInvariantName("boardElementsReferenceDeclarations"),
    QuintInvariantName("automationBoardElementsAreDeclaredAutomations"),
    QuintInvariantName("externalBoardElementsAreObservedEvents"),
    QuintInvariantName("commandEventBoardEdgesMatchEmissions"),
    QuintInvariantName("eventReadModelBoardEdgesMatchProjectionSources"),
    QuintInvariantName("viewCommandBoardEdgesMatchControls"),
    QuintInvariantName("boardConnectionsHaveCausalSemantics"),
    QuintInvariantName("externalEventTriggersMatchTranslations"),
    QuintInvariantName("externalEventsDoNotUpdateReadModels"),
    QuintInvariantName("readModelsFeedingViewsHaveIncomingEventUpdates"),
    QuintInvariantName("commandsHaveIncomingTriggers"),
    QuintInvariantName("mainPathBoardHasNoDisconnectedIslands"),
    QuintInvariantName("outcomeLabelsAreUnique"),
    QuintInvariantName("outcomeEventSetsAreNonEmpty"),
    QuintInvariantName("outcomeEventSetsAreDistinct"),
    QuintInvariantName("outcomeEventsAreKnownToSlice"),
    QuintInvariantName("eventsReferenceKnownStreams"),
    QuintInvariantName("commandEmittedEventsAreKnown"),
    QuintInvariantName("locallyEmittedEventsAreProducedByCommands"),
    QuintInvariantName("externalPayloadFieldsHaveProvenance"),
    QuintInvariantName("eventAttributesHaveAllowedSources"),
    QuintInvariantName("eventAttributesHaveProvenance"),
    QuintInvariantName("eventAttributeSourcesAreComplete"),
    QuintInvariantName("storedEventFactsTraceToOriginalSources"),
    QuintInvariantName("readModelFieldsHaveAllowedSources"),
    QuintInvariantName("readModelFieldsHaveProvenance"),
    QuintInvariantName("readModelFieldSourcesAreComplete"),
    QuintInvariantName("readModelFieldEventAttributeSourcesResolve"),
    QuintInvariantName("derivedReadModelFieldsHaveScenarioCoverage"),
    QuintInvariantName("absenceReadModelFieldsHaveScenarioCoverage"),
    QuintInvariantName("transitiveReadModelsHaveSemantics"),
    QuintInvariantName("viewFieldsHaveAllowedSources"),
    QuintInvariantName("viewFieldsHaveProvenance"),
    QuintInvariantName("viewFieldSourcesAreComplete"),
    QuintInvariantName("viewFieldsSourceFromUsedReadModels"),
    QuintInvariantName("viewsHaveInformationSketches"),
    QuintInvariantName("viewFieldsAppearInSketch"),
    QuintInvariantName("viewSketchTokensMapToModeledElements"),
    QuintInvariantName("viewFieldReadModelFieldSourcesResolve"),
    QuintInvariantName("displayedDataTraceToOriginalProvenance"),
    QuintInvariantName("viewControlsHaveSketchTokens"),
    QuintInvariantName("viewControlsAppearInSketch"),
    QuintInvariantName("viewControlsReferenceKnownCommands"),
    QuintInvariantName("viewControlsProvideCommandInputs"),
    QuintInvariantName("viewControlInputsHaveAllowedSources"),
    QuintInvariantName("viewControlInputsHaveProvenance"),
    QuintInvariantName("viewControlInputsHaveDescriptions"),
    QuintInvariantName("viewControlSessionInputsHaveDescriptions"),
    QuintInvariantName("viewControlInputVisibilityIsModeled"),
    QuintInvariantName("viewControlDecisionFieldsAreVisible"),
    QuintInvariantName("viewControlActorInputsAreVisible"),
    QuintInvariantName("viewControlsHandleCommandErrors"),
    QuintInvariantName("viewControlRecoveryBehaviorIsModeled"),
    QuintInvariantName("stateViewSlicesDoNotOwnCommands"),
    QuintInvariantName("stateViewSlicesOwnViews"),
    QuintInvariantName("stateViewSlicesOwnReadModels"),
    QuintInvariantName("stateViewSlicesOwnProjectionPaths"),
    QuintInvariantName::STATE_VIEW_SLICES_REPRESENT_SINGLE_VIEW_PROJECTION_BOUNDARY,
    QuintInvariantName("stateChangeSlicesOwnCommands"),
    QuintInvariantName("stateChangeSlicesOwnEvents"),
    QuintInvariantName("stateChangeSlicesOwnOutcomes"),
    QuintInvariantName("stateChangeSlicesOwnErrors"),
    QuintInvariantName("stateChangeSlicesDoNotOwnReadModelsOrViews"),
    QuintInvariantName("stateChangeSlicesDoNotOwnAutomationsOrTranslations"),
    QuintInvariantName("stateChangeSlicesDoNotOwnControlsOrSketches"),
    QuintInvariantName("translationSlicesDoNotOwnViews"),
    QuintInvariantName("viewControlNavigationTypesAreModeled"),
    QuintInvariantName("viewControlNavigationTypesAreDeclared"),
    QuintInvariantName("viewControlModeledViewNavigationTargetsResolve"),
    QuintInvariantName("viewControlExternalWorkflowNavigationTargetsNamed"),
    QuintInvariantName("viewControlExternalSystemNavigationTargetsHaveContracts"),
    QuintInvariantName("viewControlNavigationTargetsAreComplete"),
];

pub(crate) fn verify_project(
    project_name: &ProjectName,
    modeled_workflows: ModeledWorkflowLayouts,
    workflow_slice_details: WorkflowSliceDetails,
) -> EffectPlan {
    let module_name = module_name_from_raw(project_name.as_ref());
    let modeled_workflows = modeled_workflows.into_inner();
    let slice_module_names = workflow_slice_details
        .into_inner()
        .into_iter()
        .map(|slice| module_name_from_raw(slice.name().as_ref()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut effects = vec![
        Effect::RunProcess(verify_project_root_lean(&module_name)),
        Effect::RunProcess(verify_project_root_quint_typecheck(&module_name)),
    ];
    effects.extend(
        modeled_workflows
            .iter()
            .map(|workflow| Effect::RunProcess(verify_modeled_workflow_lean(workflow))),
    );
    effects.extend(
        slice_module_names
            .iter()
            .map(|module_name| Effect::RunProcess(verify_modeled_slice_lean(module_name))),
    );

    let mut quint_verifications = vec![verify_project_root_quint(&module_name)];
    quint_verifications.extend(modeled_workflows.iter().map(verify_modeled_workflow_quint));
    quint_verifications.extend(
        slice_module_names
            .iter()
            .map(|module_name| verify_modeled_slice_quint(module_name)),
    );
    effects.push(Effect::RunProcessBatch(ProcessInvocations::new(
        quint_verifications,
    )));

    EffectPlan::new(effects)
}

fn verify_project_root_lean(module_name: &str) -> ProcessInvocation {
    ProcessInvocation::new(
        program_name("lake"),
        vec![
            process_argument("env"),
            process_argument("lean"),
            process_argument(format!("model/lean/{module_name}.lean")),
        ],
        report_line("Lean4 artifacts verified"),
    )
}

fn verify_project_root_quint_typecheck(module_name: &str) -> ProcessInvocation {
    ProcessInvocation::new(
        program_name("quint"),
        vec![
            process_argument("typecheck"),
            process_argument(format!("model/quint/{module_name}.qnt")),
        ],
        report_line("Quint artifacts verified"),
    )
}

fn verify_project_root_quint(module_name: &str) -> ProcessInvocation {
    ProcessInvocation::new(
        program_name("quint"),
        vec![
            process_argument("verify"),
            process_argument("--invariant"),
            QuintInvariantSet::project_root().as_process_argument(),
            process_argument(format!("model/quint/{module_name}.qnt")),
        ],
        report_line("Quint artifacts verified"),
    )
}

fn verify_modeled_workflow_lean(workflow: &ModeledWorkflowLayout) -> ProcessInvocation {
    ProcessInvocation::new(
        program_name("lake"),
        vec![
            process_argument("env"),
            process_argument("lean"),
            process_argument(workflow.lean_artifact_path().as_ref().to_owned()),
        ],
        report_line("Lean4 artifacts verified"),
    )
}

fn verify_modeled_workflow_quint(workflow: &ModeledWorkflowLayout) -> ProcessInvocation {
    ProcessInvocation::new(
        program_name("quint"),
        vec![
            process_argument("verify"),
            process_argument("--invariant"),
            QuintInvariantSet::workflow().as_process_argument(),
            process_argument(workflow.quint_artifact_path().as_ref().to_owned()),
        ],
        report_line("Quint artifacts verified"),
    )
}

fn verify_modeled_slice_lean(module_name: &str) -> ProcessInvocation {
    ProcessInvocation::new(
        program_name("lake"),
        vec![
            process_argument("env"),
            process_argument("lean"),
            process_argument(format!("model/lean/slices/{module_name}.lean")),
        ],
        report_line("Lean4 artifacts verified"),
    )
}

fn verify_modeled_slice_quint(module_name: &str) -> ProcessInvocation {
    ProcessInvocation::new(
        program_name("quint"),
        vec![
            process_argument("verify"),
            process_argument("--invariant"),
            QuintInvariantSet::slice().as_process_argument(),
            process_argument(format!("model/quint/slices/{module_name}.qnt")),
        ],
        report_line("Quint artifacts verified"),
    )
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
