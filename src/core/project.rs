// Copyright 2026 John Wilger

use nutype::nutype;
use sha2::{Digest, Sha256};

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::events::EventDraft;
use crate::core::formal_model::{
    FormalModelSlice, FormalModelSliceModule, lean_command_error_record_structure,
    lean_command_input_record_structure, lean_command_record_structure,
    lean_data_flow_record_structure, lean_event_inventory_record_structures, lean_model_slice_list,
    lean_model_slice_module_list, lean_outcome_record_structure, lean_read_model_record_structures,
    lean_scenario_record_structures, lean_slice_record_structures, lean_view_record_structures,
    quint_model_slice_list, quint_model_slice_module_list,
};
use crate::core::types::{LeanModuleName, SliceSlug, WorkflowSlug};

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct ProjectName(String);

const FORMAL_MODEL_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectSliceMembership {
    workflow_slug: WorkflowSlug,
    slice_slug: SliceSlug,
    slice_module: LeanModuleName,
}

impl ProjectSliceMembership {
    pub fn new(
        workflow_slug: WorkflowSlug,
        slice_slug: SliceSlug,
        slice_module: LeanModuleName,
    ) -> Self {
        Self {
            workflow_slug,
            slice_slug,
            slice_module,
        }
    }

    fn formal_model_slice(&self) -> FormalModelSlice {
        FormalModelSlice::new(self.workflow_slug.clone(), self.slice_slug.clone())
    }

    fn formal_model_slice_module(&self) -> FormalModelSliceModule {
        FormalModelSliceModule::new(
            self.workflow_slug.clone(),
            self.slice_slug.clone(),
            self.slice_module.clone(),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectSliceMemberships {
    memberships: Vec<ProjectSliceMembership>,
}

impl ProjectSliceMemberships {
    pub fn from_memberships(memberships: impl IntoIterator<Item = ProjectSliceMembership>) -> Self {
        Self {
            memberships: memberships.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[ProjectSliceMembership] {
        &self.memberships
    }
}

pub fn init_project(project_name: ProjectName) -> EffectPlan {
    let module_name = module_name(&project_name);
    let project_name_text = project_name.as_ref();

    EffectPlan::new(vec![
        Effect::write_file_if_missing(
            project_path("emc.toml"),
            file_contents(format!(
                "[project]\nname = \"{project_name_text}\"\nversion = \"{FORMAL_MODEL_VERSION}\"\nlean_module = \"{module_name}\"\nquint_module = \"{module_name}\"\n"
            )),
        ),
        Effect::EnsureDirectory(project_path("model/lean")),
        Effect::write_file_if_missing(
            project_path("model/lean/lean-toolchain"),
            file_contents("leanprover/lean4:4.29.1\n"),
        ),
        Effect::write_file_if_missing(
            project_path("model/lean/lakefile.lean"),
            file_contents("import Lake\nopen Lake DSL\npackage EMCModel where\n"),
        ),
        Effect::write_file_if_missing(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_project_root(&project_name, &[], &[]),
        ),
        Effect::write_file_if_missing(
            project_path("model/lean/slices/.gitkeep"),
            file_contents("\n"),
        ),
        Effect::EnsureDirectory(project_path("model/quint")),
        Effect::write_file_if_missing(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_project_root(&project_name, &[], &[]),
        ),
        Effect::write_file_if_missing(
            project_path("model/quint/slices/.gitkeep"),
            file_contents("\n"),
        ),
        Effect::EnsureDirectory(project_path("reviews")),
        Effect::write_file_if_missing(project_path("reviews/.gitkeep"), file_contents("\n")),
        Effect::EnsureDirectory(project_path("model/events/v1")),
        Effect::ExportEvent(EventDraft::project_initialized(&project_name)),
        Effect::Report(report_line(format!(
            "EMC project {project_name} layout is present"
        ))),
    ])
}

pub(crate) fn project_root_effects(
    project_name: ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> [Effect; 2] {
    let module_name = module_name(&project_name);
    [
        Effect::write_file(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_project_root(&project_name, workflow_slugs, slice_memberships),
        ),
        Effect::write_file(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_project_root(&project_name, workflow_slugs, slice_memberships),
        ),
    ]
}

fn emit_lean_project_root(
    project_name: &ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> FileContents {
    let module_name = module_name(project_name);
    let workflow_list = lean_workflow_slug_list(workflow_slugs);
    let workflow_count = workflow_slugs.len();
    let model_slices = formal_model_slices(slice_memberships);
    let model_slice_modules = formal_model_slice_modules(slice_memberships);
    let slice_list = lean_model_slice_list(&model_slices);
    let slice_module_list = lean_model_slice_module_list(&model_slice_modules);
    let slice_count = slice_memberships.len();
    let model_digest = model_digest(project_name, workflow_slugs, slice_memberships);
    let model_slice_structures = lean_slice_record_structures();
    let model_scenario_structures = lean_scenario_record_structures();
    let model_data_flow_structure = lean_data_flow_record_structure();
    let model_outcome_structure = lean_outcome_record_structure();
    let model_command_error_structure = lean_command_error_record_structure();
    let model_command_structure = lean_command_record_structure();
    let model_command_input_structure = lean_command_input_record_structure();
    let model_read_model_structures = lean_read_model_record_structures();
    let model_view_structures = lean_view_record_structures();
    let model_event_inventory_structures = lean_event_inventory_record_structures();
    let model_data_flow_invariant_definitions = lean_data_flow_invariant_definitions();
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC generated Lean4 model root.\n\ndef modelName := {project_name:?}\n\ndef modelVersion := \"{FORMAL_MODEL_VERSION}\"\n\ndef modelDigest := {model_digest:?}\n\ndef modelWorkflows : List String := {workflow_list}\n\n{model_slice_structures}\n\n{model_scenario_structures}\n\n{model_data_flow_structure}\n\n{model_outcome_structure}\n\n{model_command_error_structure}\n\n{model_command_structure}\n\n{model_command_input_structure}\n\n{model_read_model_structures}\n\n{model_view_structures}\n\n{model_event_inventory_structures}\n\ndef modelSlices : List ModelSlice := {slice_list}\n\ndef modelSliceModules : List ModelSliceModule := {slice_module_list}\n\ndef modelScenarios : List ModelScenario := []\n\ndef modelScenarioDefinitions : List ModelScenarioDefinition := []\n\ndef modelDataFlows : List ModelDataFlow := []\n\ndef modelOutcomes : List ModelOutcome := []\n\ndef modelCommandErrors : List ModelCommandError := []\n\ndef modelCommands : List ModelCommand := []\n\ndef modelCommandInputs : List ModelCommandInput := []\n\ndef modelReadModels : List ModelReadModel := []\n\ndef modelReadModelDefinitions : List ModelReadModelDefinition := []\n\ndef modelReadModelFields : List ModelReadModelField := []\n\ndef modelViews : List ModelView := []\n\ndef modelViewDefinitions : List ModelViewDefinition := []\n\ndef modelViewControls : List ModelViewControl := []\n\ndef modelBoardElements : List (String × String × String × String × String × String × Bool) := []\n\ndef modelBoardConnections : List (String × String × String × String × String × String) := []\n\ndef modelViewFields : List ModelViewField := []\n\ndef modelAutomations : List (String × String × String) := []\n\ndef modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := []\n\ndef modelTranslations : List (String × String × String) := []\n\ndef modelTranslationDefinitions : List (String × String × String × String × String × String) := []\n\ndef modelExternalPayloads : List (String × String × String) := []\n\ndef modelExternalPayloadFields : List (String × String × String × String × String × String) := []\n\ndef modelStreams : List ModelStream := []\n\ndef modelEvents : List ModelEvent := []\n\ndef modelEventAttributes : List ModelEventAttribute := []\n\ndef modelScenarioDefinitionHasGwt (scenario : ModelScenarioDefinition) : Bool := scenario.given.isEmpty == false && scenario.when.isEmpty == false && scenario.thenStep.isEmpty == false\n\ndef modelScenarioKindIsFirstClass (scenario : ModelScenarioDefinition) : Bool := scenario.scenarioKind == \"acceptance\" || scenario.scenarioKind == \"contract\"\n\ndef modelDataFlowIsBitComplete (dataFlow : ModelDataFlow) : Bool := dataFlow.datum.isEmpty == false && dataFlow.sourceKind.isEmpty == false && dataFlow.source.isEmpty == false && dataFlow.transformation.isEmpty == false && dataFlow.target.isEmpty == false && dataFlow.bitEncoding.isEmpty == false\n\n{model_data_flow_invariant_definitions}\n\ndef modelCommandInputHasProvenance (input : ModelCommandInput) : Bool := input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false\n\ndef modelCommandInputTracesToInvocationSource (input : ModelCommandInput) : Bool := input.sourceKind == \"actor\" || (input.sourceKind == \"event_stream_state\" && input.eventStreamSourceEvent.isEmpty == false && input.eventStreamSourceAttribute.isEmpty == false) || (input.sourceKind == \"external_payload\" && input.externalPayloadSourceName.isEmpty == false && input.externalPayloadSourceField.isEmpty == false) || (input.sourceKind == \"generated\" && input.generatedSourceName.isEmpty == false && input.generatedSourceField.isEmpty == false) || (input.sourceKind == \"session\" && input.sessionSourceName.isEmpty == false && input.sessionSourceField.isEmpty == false) || (input.sourceKind == \"invocation_argument\" && input.invocationArgumentSourceName.isEmpty == false && input.invocationArgumentSourceField.isEmpty == false)\n\ndef modelReadModelFieldSourceIsComplete (field : ModelReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false && field.derivationSourceFields.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)\n\ndef modelControlProvidesCommandInput (control : ModelViewControl) (input : ModelCommandInput) : Bool := control.workflow == input.workflow && control.command == input.command && control.input == input.input\n\ndef modelViewControlProvidesEveryCommandInput (control : ModelViewControl) : Bool := modelCommandInputs.all (fun input => input.workflow != control.workflow || input.command != control.command || modelViewControls.any (fun providedInput => providedInput.workflow == control.workflow && providedInput.slice == control.slice && providedInput.view == control.view && providedInput.control == control.control && providedInput.command == control.command && modelControlProvidesCommandInput providedInput input))\n\ntheorem modelIdentityIsStable : modelName = {project_name:?} := rfl\n\ntheorem modelVersionIsStable : modelVersion = \"{FORMAL_MODEL_VERSION}\" := rfl\n\ntheorem modelDigestIsStable : modelDigest = {model_digest:?} := rfl\n\ntheorem modelWorkflowsAreDeclared : modelWorkflows.length = {workflow_count} := rfl\n\ntheorem modelSlicesAreDeclared : modelSlices.length = {slice_count} := rfl\n\ntheorem modelSliceModulesAreDeclared : modelSliceModules.length = {slice_count} := rfl\n\ntheorem modelScenariosAreDeclared : modelScenarios.length = 0 := rfl\n\ntheorem modelScenarioDefinitionsAreDeclared : modelScenarioDefinitions.length = 0 := rfl\n\ntheorem modelScenarioDefinitionsHaveGwt : modelScenarioDefinitions.all modelScenarioDefinitionHasGwt = true := rfl\n\ntheorem modelScenarioKindsAreFirstClass : modelScenarioDefinitions.all modelScenarioKindIsFirstClass = true := rfl\n\ntheorem modelDataFlowsAreDeclared : modelDataFlows.length = 0 := rfl\n\ntheorem modelDataFlowsAreBitComplete : modelDataFlows.all modelDataFlowIsBitComplete = true := rfl\n\ntheorem modelOutcomesAreDeclared : modelOutcomes.length = 0 := rfl\n\ntheorem modelCommandErrorsAreDeclared : modelCommandErrors.length = 0 := rfl\n\ntheorem modelCommandsAreDeclared : modelCommands.length = 0 := rfl\n\ntheorem modelCommandInputsAreDeclared : modelCommandInputs.length = 0 := rfl\n\ntheorem modelCommandInputsHaveProvenance : modelCommandInputs.all modelCommandInputHasProvenance = true := rfl\n\ntheorem modelCommandInputsTraceToInvocationSources : modelCommandInputs.all modelCommandInputTracesToInvocationSource = true := rfl\n\ntheorem modelReadModelsAreDeclared : modelReadModels.length = 0 := rfl\n\ntheorem modelReadModelDefinitionsAreDeclared : modelReadModelDefinitions.length = 0 := rfl\n\ntheorem modelReadModelFieldsAreDeclared : modelReadModelFields.length = 0 := rfl\n\ntheorem modelReadModelFieldSourcesAreComplete : modelReadModelFields.all modelReadModelFieldSourceIsComplete = true := rfl\n\ntheorem modelViewsAreDeclared : modelViews.length = 0 := rfl\n\ntheorem modelViewDefinitionsAreDeclared : modelViewDefinitions.length = 0 := rfl\n\ntheorem modelViewControlsAreDeclared : modelViewControls.length = 0 := rfl\n\ntheorem modelViewControlsProvideCommandInputs : modelViewControls.all modelViewControlProvidesEveryCommandInput = true := rfl\n\ntheorem modelBoardElementsAreDeclared : modelBoardElements.length = 0 := rfl\n\ntheorem modelBoardConnectionsAreDeclared : modelBoardConnections.length = 0 := rfl\n\ntheorem modelViewFieldsAreDeclared : modelViewFields.length = 0 := rfl\n\ntheorem modelAutomationsAreDeclared : modelAutomations.length = 0 := rfl\n\ntheorem modelAutomationDefinitionsAreDeclared : modelAutomationDefinitions.length = 0 := rfl\n\ntheorem modelTranslationsAreDeclared : modelTranslations.length = 0 := rfl\n\ntheorem modelTranslationDefinitionsAreDeclared : modelTranslationDefinitions.length = 0 := rfl\n\ntheorem modelExternalPayloadsAreDeclared : modelExternalPayloads.length = 0 := rfl\n\ntheorem modelExternalPayloadFieldsAreDeclared : modelExternalPayloadFields.length = 0 := rfl\n\ntheorem modelStreamsAreDeclared : modelStreams.length = 0 := rfl\n\ntheorem modelEventsAreDeclared : modelEvents.length = 0 := rfl\n\ntheorem modelEventAttributesAreDeclared : modelEventAttributes.length = 0 := rfl\n\nend {module_name}\n",
        project_name = project_name.as_ref(),
    )
    .replace(
        "\ndef modelScenarios :",
        "\ndef modelSliceBelongsToDeclaredWorkflow (slice : ModelSlice) : Bool := modelWorkflows.any (fun workflow => workflow == slice.workflow)\n\ndef modelSliceHasModule (slice : ModelSlice) : Bool := modelSliceModules.any (fun sliceModule => sliceModule.workflow == slice.workflow && sliceModule.slice == slice.slice && sliceModule.formalModule.isEmpty == false)\n\ndef modelSliceModuleBelongsToDeclaredSlice (sliceModule : ModelSliceModule) : Bool := sliceModule.formalModule.isEmpty == false && modelSlices.any (fun slice => slice.workflow == sliceModule.workflow && slice.slice == sliceModule.slice)\n\ndef modelWorkflowSlicesHaveModules (workflow : String) : Bool := modelSlices.all (fun slice => slice.workflow != workflow || modelSliceHasModule slice)\n\ndef modelWorkflowHasCompositionStructure (workflow : String) : Bool := modelWorkflowSlicesHaveModules workflow\n\ndef modelScenarios :",
    )
    .replace(
        "\ntheorem modelScenariosAreDeclared",
        "\ntheorem modelWorkflowCompositionStructureComplete : (modelSlices.all modelSliceBelongsToDeclaredWorkflow && modelSlices.all modelSliceHasModule && modelSliceModules.all modelSliceModuleBelongsToDeclaredSlice && modelWorkflows.all modelWorkflowHasCompositionStructure) = true := rfl\n\ntheorem modelWorkflowBehaviorSurfaceIsCompleteIsStable : modelWorkflowBehaviorSurfaceIsComplete = true := rfl\n\ntheorem modelScenariosAreDeclared",
    )
    .replace(
        "\ntheorem modelIdentityIsStable",
        "\ndef modelOutcomeBranchIsModeled (outcome : ModelOutcome) : Bool := outcome.outcome.isEmpty == false && outcome.events.isEmpty == false\n\ndef modelCommandErrorRecoveryIsModeled (commandError : ModelCommandError) : Bool := commandError.command.isEmpty == false && commandError.error.isEmpty == false && commandError.scenario.isEmpty == false && commandError.recovery.isEmpty == false\n\ndef modelViewControlNavigationTargetIsModeled (control : ModelViewControl) : Bool := control.navigationType.isEmpty || ((control.navigationType == \"modeled_view\" || control.navigationType == \"local_view_state\") && control.navigationTarget.isEmpty == false) || (control.navigationType == \"external_workflow\" && control.externalWorkflow.isEmpty == false) || (control.navigationType == \"external_system\" && control.externalSystem.isEmpty == false && control.handoffContract.isEmpty == false)\n\ndef modelExternalBoundaryContractIsModeled (translation : String × String × String × String × String × String) : Bool := let (_, _, translationName, externalEvent, payloadContract, command) := translation; translationName.isEmpty == false && externalEvent.isEmpty == false && payloadContract.isEmpty == false && command.isEmpty == false\n\ndef modelWorkflowBehaviorSurfaceIsComplete : Bool := modelOutcomes.all modelOutcomeBranchIsModeled && modelCommandErrors.all modelCommandErrorRecoveryIsModeled && modelViewControls.all modelViewControlNavigationTargetIsModeled && modelTranslationDefinitions.all modelExternalBoundaryContractIsModeled\n\ntheorem modelIdentityIsStable",
    )
    .replace(
        "theorem modelDataFlowsAreBitComplete : modelDataFlows.all modelDataFlowIsBitComplete = true := rfl",
        "theorem modelDataFlowsAreBitComplete : modelDataFlows.all modelDataFlowIsBitComplete = true := rfl\n\ntheorem modelDataFlowSourceKindsAreModeled : modelDataFlows.all modelDataFlowHasModeledSourceKind = true := rfl\n\ntheorem modelDataFlowModeledSourcesResolve : modelDataFlows.all modelDataFlowModeledSourceResolves = true := rfl\n\ntheorem modelDataFlowTransformationsAreModeled : modelDataFlows.all modelDataFlowHasModeledTransformationSemantics = true := rfl\n\ntheorem modelMeaningfulDataFlowsAreCovered : modelMeaningfulDataHasModeledDataFlows = true := rfl\n\ntheorem modelDataFlowSourceBitEncodingsMatchModeledSources : modelDataFlows.all modelDataFlowSourceBitEncodingMatchesModeledSource = true := rfl\n\ntheorem modelViewFieldBitEncodingsMatchDataFlows : modelViewFields.all modelViewFieldBitEncodingMatchesDataFlow = true := rfl\n\ntheorem modelExternalPayloadFieldBitEncodingsMatchDataFlows : modelExternalPayloadFields.all modelExternalPayloadFieldBitEncodingMatchesDataFlow = true := rfl",
    )
    .replace(
        "theorem modelDataFlowModeledSourcesResolve : modelDataFlows.all modelDataFlowModeledSourceResolves = true := rfl\n\ntheorem modelDataFlowTransformationsAreModeled",
        "theorem modelDataFlowModeledSourcesResolve : modelDataFlows.all modelDataFlowModeledSourceResolves = true := rfl\n\ntheorem modelDataFlowSourceChainsReachOriginals : modelDataFlows.all modelDataFlowHasOriginalSourceChain = true := rfl\n\ntheorem modelDataFlowSourceChainsPreserveBitEncodingSemantics : modelDataFlows.all modelDataFlowHasBitPreservingOriginalSourceChain = true := rfl\n\ntheorem modelDataFlowTransformationsAreModeled",
    )
    .replace(
        "def modelReadModelFieldSourceIsComplete (field : ModelReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false && field.derivationSourceFields.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)",
        "def modelEventAttributeSourceIsComplete (eventAttribute : ModelEventAttribute) : Bool := eventAttribute.provenance.isEmpty == false && ((eventAttribute.sourceKind == \"command_input\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false) || (eventAttribute.sourceKind == \"external_payload\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false) || (eventAttribute.sourceKind == \"generated\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.generatedSourceKind.isEmpty == false) || (eventAttribute.sourceKind == \"session\" && eventAttribute.sourceName.isEmpty == false) || (eventAttribute.sourceKind == \"derivation\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false))\n\ndef modelReadModelFieldSourceIsComplete (field : ModelReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false && field.derivationSourceFields.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)\n\ndef modelReadModelFieldTracesToOriginalProvenance (field : ModelReadModelField) : Bool := field.provenance.isEmpty == false && ((field.sourceKind == \"event_attribute\" && modelEventAttributes.any (fun eventAttribute => eventAttribute.workflow == field.workflow && eventAttribute.slice == field.slice && eventAttribute.event == field.sourceEvent && eventAttribute.attributeName == field.sourceAttribute && modelEventAttributeSourceIsComplete eventAttribute)) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false && field.derivationSourceFields.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false))\n\ndef modelViewFieldSourceIsComplete (field : ModelViewField) : Bool := field.sourceKind == \"read_model\" && field.sourceReadModel.isEmpty == false && field.sourceField.isEmpty == false && field.provenance.isEmpty == false && field.bitEncoding.isEmpty == false\n\ndef modelViewFieldReadModelFieldSourceResolves (viewField : ModelViewField) : Bool := modelViewFieldSourceIsComplete viewField && modelReadModelFields.any (fun readModelField => readModelField.workflow == viewField.workflow && readModelField.slice == viewField.slice && readModelField.readModel == viewField.sourceReadModel && readModelField.field == viewField.sourceField && modelReadModelFieldSourceIsComplete readModelField)\n\ndef modelDisplayedDatumTracesToOriginalProvenance (viewField : ModelViewField) : Bool := modelViewFieldReadModelFieldSourceResolves viewField && modelReadModelFields.any (fun readModelField => readModelField.workflow == viewField.workflow && readModelField.slice == viewField.slice && readModelField.readModel == viewField.sourceReadModel && readModelField.field == viewField.sourceField && modelReadModelFieldTracesToOriginalProvenance readModelField)\n\ndef modelExternalPayloadFieldHasProvenance (field : String × String × String × String × String × String) : Bool := let (_, _, _, _, provenance, bitEncoding) := field; provenance.isEmpty == false && bitEncoding.isEmpty == false",
    )
    .replace(
        "theorem modelReadModelFieldSourcesAreComplete : modelReadModelFields.all modelReadModelFieldSourceIsComplete = true := rfl",
        "theorem modelEventAttributeSourcesAreComplete : modelEventAttributes.all modelEventAttributeSourceIsComplete = true := rfl\n\ntheorem modelReadModelFieldSourcesAreComplete : modelReadModelFields.all modelReadModelFieldSourceIsComplete = true := rfl\n\ntheorem modelViewFieldSourcesAreComplete : modelViewFields.all modelViewFieldSourceIsComplete = true := rfl\n\ntheorem modelViewFieldReadModelFieldSourcesResolve : modelViewFields.all modelViewFieldReadModelFieldSourceResolves = true := rfl\n\ntheorem modelDisplayedDataTraceToOriginalProvenance : modelViewFields.all modelDisplayedDatumTracesToOriginalProvenance = true := rfl\n\ntheorem modelExternalPayloadFieldsHaveProvenance : modelExternalPayloadFields.all modelExternalPayloadFieldHasProvenance = true := rfl",
    ))
}

fn lean_data_flow_invariant_definitions() -> &'static str {
    "def modelDataFlowCoversDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) : Bool := modelDataFlows.any (fun dataFlow => dataFlow.workflow == workflow && dataFlow.slice == slice && dataFlow.datum == datum && dataFlow.target == target && modelDataFlowIsBitComplete dataFlow)\n\ndef modelDataFlowBitEncodingMatchesDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) (bitEncoding : String) : Bool := modelDataFlows.any (fun dataFlow => dataFlow.workflow == workflow && dataFlow.slice == slice && dataFlow.datum == datum && dataFlow.target == target && dataFlow.bitEncoding == bitEncoding && modelDataFlowIsBitComplete dataFlow)\n\ndef modelDataFlowSourceBitEncodingMatchesModeledSource (dataFlow : ModelDataFlow) : Bool := (modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source) == false) || modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && sourceFlow.bitEncoding == dataFlow.bitEncoding && modelDataFlowIsBitComplete sourceFlow)\n\ndef modelDataFlowHasModeledTransformationSemantics (dataFlow : ModelDataFlow) : Bool := dataFlow.transformation == \"identity\" || dataFlow.transformation == \"projection\" || dataFlow.transformation == \"derivation\" || dataFlow.transformation == \"default\" || dataFlow.transformation == \"absence\" || dataFlow.transformation == \"transformation\"\n\ndef modelDataFlowHasModeledSourceKind (dataFlow : ModelDataFlow) : Bool := (dataFlow.sourceKind == \"original\" && dataFlow.source.isEmpty == false) || (dataFlow.sourceKind == \"modeled_target\" && dataFlow.source.isEmpty == false)\n\ndef modelDataFlowModeledSourceResolves (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind != \"modeled_target\" || modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && modelDataFlowIsBitComplete sourceFlow)\n\ndef modelSameDataFlowTarget (left : ModelDataFlow) (right : ModelDataFlow) : Bool := left.workflow == right.workflow && left.slice == right.slice && left.datum == right.datum && left.target == right.target\n\ndef modelDataFlowTargetsFromReachable (reachable : List ModelDataFlow) : List ModelDataFlow := modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"modeled_target\" && reachable.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && modelDataFlowIsBitComplete sourceFlow))\n\ndef modelDataFlowsReachableFromOriginalsAfterFuel : Nat -> List ModelDataFlow -> List ModelDataFlow\n  | Nat.zero, reachable => reachable\n  | Nat.succ fuel, reachable => modelDataFlowsReachableFromOriginalsAfterFuel fuel (reachable ++ modelDataFlowTargetsFromReachable reachable)\n\ndef modelDataFlowsReachableFromOriginals : List ModelDataFlow := modelDataFlowsReachableFromOriginalsAfterFuel modelDataFlows.length (modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"original\" && modelDataFlowIsBitComplete dataFlow))\n\ndef modelDataFlowHasOriginalSourceChain (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind == \"original\" || modelDataFlowsReachableFromOriginals.any (fun reachableFlow => modelSameDataFlowTarget reachableFlow dataFlow)\n\ndef modelDataFlowTargetsFromBitPreservingReachable (reachable : List ModelDataFlow) : List ModelDataFlow := modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"modeled_target\" && reachable.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && sourceFlow.bitEncoding == dataFlow.bitEncoding && modelDataFlowIsBitComplete sourceFlow))\n\ndef modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel : Nat -> List ModelDataFlow -> List ModelDataFlow\n  | Nat.zero, reachable => reachable\n  | Nat.succ fuel, reachable => modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel fuel (reachable ++ modelDataFlowTargetsFromBitPreservingReachable reachable)\n\ndef modelDataFlowsReachableFromOriginalsWithPreservedBits : List ModelDataFlow := modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel modelDataFlows.length (modelDataFlows.filter (fun dataFlow => dataFlow.sourceKind == \"original\" && modelDataFlowIsBitComplete dataFlow))\n\ndef modelDataFlowHasBitPreservingOriginalSourceChain (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind == \"original\" || modelDataFlowsReachableFromOriginalsWithPreservedBits.any (fun reachableFlow => modelSameDataFlowTarget reachableFlow dataFlow)\n\ndef modelCommandInputHasModeledDataFlow (input : ModelCommandInput) : Bool := modelDataFlowCoversDatumTarget input.workflow input.slice input.input input.command\n\ndef modelEventAttributeHasModeledDataFlow (eventAttribute : ModelEventAttribute) : Bool := modelDataFlowCoversDatumTarget eventAttribute.workflow eventAttribute.slice eventAttribute.attributeName eventAttribute.event\n\ndef modelReadModelFieldHasModeledDataFlow (field : ModelReadModelField) : Bool := modelDataFlowCoversDatumTarget field.workflow field.slice field.field field.readModel\n\ndef modelViewFieldHasModeledDataFlow (field : ModelViewField) : Bool := modelDataFlowCoversDatumTarget field.workflow field.slice field.field field.view\n\ndef modelViewFieldBitEncodingMatchesDataFlow (field : ModelViewField) : Bool := modelDataFlowBitEncodingMatchesDatumTarget field.workflow field.slice field.field field.view field.bitEncoding\n\ndef modelExternalPayloadFieldHasModeledDataFlow (field : String × String × String × String × String × String) : Bool := let (workflow, slice, targetPayload, datum, _, _) := field; modelDataFlowCoversDatumTarget workflow slice datum targetPayload\n\ndef modelExternalPayloadFieldBitEncodingMatchesDataFlow (field : String × String × String × String × String × String) : Bool := let (workflow, slice, targetPayload, datum, _, bitEncoding) := field; modelDataFlowBitEncodingMatchesDatumTarget workflow slice datum targetPayload bitEncoding\n\ndef modelMeaningfulDataHasModeledDataFlows : Bool := modelCommandInputs.all modelCommandInputHasModeledDataFlow && modelEventAttributes.all modelEventAttributeHasModeledDataFlow && modelReadModelFields.all modelReadModelFieldHasModeledDataFlow && modelViewFields.all modelViewFieldHasModeledDataFlow && modelExternalPayloadFields.all modelExternalPayloadFieldHasModeledDataFlow"
}

fn emit_quint_project_root(
    project_name: &ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> FileContents {
    let module_name = module_name(project_name);
    let workflow_list = quint_workflow_slug_list(workflow_slugs);
    let workflow_count = workflow_slugs.len();
    let model_slices = formal_model_slices(slice_memberships);
    let model_slice_modules = formal_model_slice_modules(slice_memberships);
    let slice_list = quint_model_slice_list(&model_slices);
    let slice_module_list = quint_model_slice_module_list(&model_slice_modules);
    let slice_count = slice_memberships.len();
    let model_digest = model_digest(project_name, workflow_slugs, slice_memberships);
    file_contents(format!(
        "module {module_name} {{\n  type ModelSlice = {{ workflow: str, slice: str }}\n  type ModelSliceModule = {{ workflow: str, slice: str, formalModule: str }}\n  type ModelScenario = {{ workflow: str, slice: str, scenarioKind: str, scenario: str }}\n  type ModelScenarioDefinition = {{ workflow: str, slice: str, scenarioKind: str, scenario: str, given: str, when: str, then: str, readStreams: List[str], writtenStreams: List[str], contractKind: str, coveredDefinition: str, errorReferences: List[str] }}\n  type ModelDataFlow = {{ workflow: str, slice: str, datum: str, sourceKind: str, source: str, transformation: str, target: str, bitEncoding: str }}\n  type ModelOutcome = {{ workflow: str, slice: str, outcome: str, events: List[str], externallyRelevant: bool }}\n  type ModelCommandError = {{ workflow: str, slice: str, command: str, error: str, scenario: str, recovery: str }}\n  type ModelCommand = {{ workflow: str, slice: str, command: str }}\n  type ModelCommandInput = {{ workflow: str, slice: str, command: str, input: str, sourceKind: str, sourceDescription: str, provenanceChain: List[str], eventStreamSourceEvent: str, eventStreamSourceAttribute: str, externalPayloadSourceName: str, externalPayloadSourceField: str, generatedSourceName: str, generatedSourceField: str, sessionSourceName: str, sessionSourceField: str, invocationArgumentSourceName: str, invocationArgumentSourceField: str }}\n  type ModelReadModel = {{ workflow: str, slice: str, readModel: str }}\n  type ModelReadModelDefinition = {{ workflow: str, slice: str, readModel: str, transitive: bool, relationshipFields: List[str], transitiveRule: str, exampleScenarioName: str }}\n  type ModelReadModelField = {{ workflow: str, slice: str, readModel: str, field: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, derivationSourceFields: List[str], absenceEvent: str, derivationScenarioName: str, absenceScenarioName: str, provenance: str }}\n  type ModelView = {{ workflow: str, slice: str, view: str }}\n  type ModelViewDefinition = {{ workflow: str, slice: str, view: str, readModels: List[str], sketchTokens: List[str], localStates: List[str], filters: List[str] }}\n  type ModelViewControl = {{ workflow: str, slice: str, view: str, control: str, command: str, input: str, inputSourceKind: str, inputSourceDescription: str, inputSketchToken: str, inputVisibleToActor: bool, inputDecisionField: bool, handledErrors: List[str], recoveryBehavior: str, controlSketchToken: str, navigationType: str, navigationTarget: str, externalWorkflow: str, externalSystem: str, handoffContract: str }}\n  type ModelBoardElement = {{ workflow: str, slice: str, element: str, kind: str, lane: str, declaredName: str, mainPath: bool }}\n  type ModelBoardConnection = {{ workflow: str, slice: str, source: str, sourceKind: str, target: str, targetKind: str }}\n  type ModelViewField = {{ workflow: str, slice: str, view: str, field: str, sourceKind: str, sourceReadModel: str, sourceField: str, provenance: str, bitEncoding: str }}\n  type ModelAutomation = {{ workflow: str, slice: str, automation: str }}\n  type ModelAutomationDefinition = {{ workflow: str, slice: str, automation: str, trigger: str, command: str, handledErrors: List[str], reaction: str }}\n  type ModelTranslation = {{ workflow: str, slice: str, translation: str }}\n  type ModelTranslationDefinition = {{ workflow: str, slice: str, translation: str, externalEvent: str, payloadContract: str, command: str }}\n  type ModelExternalPayload = {{ workflow: str, slice: str, externalPayload: str }}\n  type ModelExternalPayloadField = {{ workflow: str, slice: str, externalPayload: str, field: str, provenance: str, bitEncoding: str }}\n  type ModelStream = {{ workflow: str, slice: str, stream: str }}\n  type ModelEvent = {{ workflow: str, slice: str, event: str, stream: str }}\n  type ModelEventAttribute = {{ workflow: str, slice: str, event: str, attribute: str, sourceKind: str, sourceName: str, sourceField: str, generatedSourceKind: str, provenance: str }}\n  val modelName = {project_name:?}\n  val modelVersion = \"{FORMAL_MODEL_VERSION}\"\n  val modelDigest = {model_digest:?}\n  val modelWorkflows: List[str] = {workflow_list}\n  val modelSlices: List[ModelSlice] = {slice_list}\n  val modelSliceModules: List[ModelSliceModule] = {slice_module_list}\n  val modelScenarios: List[ModelScenario] = []\n  val modelScenarioDefinitions: List[ModelScenarioDefinition] = []\n  val modelDataFlows: List[ModelDataFlow] = []\n  val modelOutcomes: List[ModelOutcome] = []\n  val modelCommandErrors: List[ModelCommandError] = []\n  val modelCommands: List[ModelCommand] = []\n  val modelCommandInputs: List[ModelCommandInput] = []\n  val modelReadModels: List[ModelReadModel] = []\n  val modelReadModelDefinitions: List[ModelReadModelDefinition] = []\n  val modelReadModelFields: List[ModelReadModelField] = []\n  val modelViews: List[ModelView] = []\n  val modelViewDefinitions: List[ModelViewDefinition] = []\n  val modelViewControls: List[ModelViewControl] = []\n  val modelBoardElements: List[ModelBoardElement] = []\n  val modelBoardConnections: List[ModelBoardConnection] = []\n  val modelViewFields: List[ModelViewField] = []\n  val modelAutomations: List[ModelAutomation] = []\n  val modelAutomationDefinitions: List[ModelAutomationDefinition] = []\n  val modelTranslations: List[ModelTranslation] = []\n  val modelTranslationDefinitions: List[ModelTranslationDefinition] = []\n  val modelExternalPayloads: List[ModelExternalPayload] = []\n  val modelExternalPayloadFields: List[ModelExternalPayloadField] = []\n  val modelStreams: List[ModelStream] = []\n  val modelEvents: List[ModelEvent] = []\n  val modelEventAttributes: List[ModelEventAttribute] = []\n  val modelIdentityStable = modelName == {project_name:?}\n  val modelVersionStable = modelVersion == \"{FORMAL_MODEL_VERSION}\"\n  val modelDigestStable = modelDigest == {model_digest:?}\n  val modelWorkflowsAreDeclared = modelWorkflows.length() == {workflow_count}\n  val modelSlicesAreDeclared = modelSlices.length() == {slice_count}\n  val modelSliceModulesAreDeclared = modelSliceModules.length() == {slice_count}\n  val modelScenariosAreDeclared = modelScenarios.length() == 0\n  val modelScenarioDefinitionsAreDeclared = modelScenarioDefinitions.length() == 0\n  def modelScenarioDefinitionHasGwt(scenario) = scenario.given != \"\" and scenario.when != \"\" and scenario.then != \"\"\n  def modelScenarioKindIsFirstClass(scenario) = scenario.scenarioKind == \"acceptance\" or scenario.scenarioKind == \"contract\"\n  val modelScenarioDefinitionsHaveGwt = modelScenarioDefinitions.select(scenario => modelScenarioDefinitionHasGwt(scenario)).length() == modelScenarioDefinitions.length()\n  val modelScenarioKindsAreFirstClass = modelScenarioDefinitions.select(scenario => modelScenarioKindIsFirstClass(scenario)).length() == modelScenarioDefinitions.length()\n  val modelDataFlowsAreDeclared = modelDataFlows.length() == 0\n  def modelDataFlowIsBitComplete(dataFlow) = dataFlow.datum != \"\" and dataFlow.sourceKind != \"\" and dataFlow.source != \"\" and dataFlow.transformation != \"\" and dataFlow.target != \"\" and dataFlow.bitEncoding != \"\"\n  val modelDataFlowsAreBitComplete = modelDataFlows.select(dataFlow => modelDataFlowIsBitComplete(dataFlow)).length() == modelDataFlows.length()\n  val modelOutcomesAreDeclared = modelOutcomes.length() == 0\n  val modelCommandErrorsAreDeclared = modelCommandErrors.length() == 0\n  val modelCommandsAreDeclared = modelCommands.length() == 0\n  val modelCommandInputsAreDeclared = modelCommandInputs.length() == 0\n  val modelReadModelsAreDeclared = modelReadModels.length() == 0\n  val modelReadModelDefinitionsAreDeclared = modelReadModelDefinitions.length() == 0\n  val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == 0\n  val modelViewsAreDeclared = modelViews.length() == 0\n  val modelViewDefinitionsAreDeclared = modelViewDefinitions.length() == 0\n  val modelViewControlsAreDeclared = modelViewControls.length() == 0\n  def modelControlProvidesCommandInput(control, input) = control.workflow == input.workflow and control.command == input.command and control.input == input.input\n  def modelViewControlProvidesEveryCommandInput(control) = modelCommandInputs.select(input => input.workflow != control.workflow or input.command != control.command or modelViewControls.select(providedInput => providedInput.workflow == control.workflow and providedInput.slice == control.slice and providedInput.view == control.view and providedInput.control == control.control and providedInput.command == control.command and modelControlProvidesCommandInput(providedInput, input)).length() > 0).length() == modelCommandInputs.length()\n  val modelViewControlsProvideCommandInputs = modelViewControls.select(control => modelViewControlProvidesEveryCommandInput(control)).length() == modelViewControls.length()\n  val modelBoardElementsAreDeclared = modelBoardElements.length() == 0\n  val modelBoardConnectionsAreDeclared = modelBoardConnections.length() == 0\n  val modelViewFieldsAreDeclared = modelViewFields.length() == 0\n  val modelAutomationsAreDeclared = modelAutomations.length() == 0\n  val modelAutomationDefinitionsAreDeclared = modelAutomationDefinitions.length() == 0\n  val modelTranslationsAreDeclared = modelTranslations.length() == 0\n  val modelTranslationDefinitionsAreDeclared = modelTranslationDefinitions.length() == 0\n  val modelExternalPayloadsAreDeclared = modelExternalPayloads.length() == 0\n  val modelExternalPayloadFieldsAreDeclared = modelExternalPayloadFields.length() == 0\n  val modelStreamsAreDeclared = modelStreams.length() == 0\n  val modelEventsAreDeclared = modelEvents.length() == 0\n  val modelEventAttributesAreDeclared = modelEventAttributes.length() == 0\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}}\n",
        project_name = project_name.as_ref(),
    )
    .replace(
        "val modelCommandInputsAreDeclared = modelCommandInputs.length() == 0\n  val modelReadModelsAreDeclared = modelReadModels.length() == 0",
        "val modelCommandInputsAreDeclared = modelCommandInputs.length() == 0\n  def modelCommandInputHasProvenance(input) = input.sourceDescription != \"\" and input.provenanceChain.length() > 0\n  def modelCommandInputTracesToInvocationSource(input) = input.sourceKind == \"actor\" or (input.sourceKind == \"event_stream_state\" and input.eventStreamSourceEvent != \"\" and input.eventStreamSourceAttribute != \"\") or (input.sourceKind == \"external_payload\" and input.externalPayloadSourceName != \"\" and input.externalPayloadSourceField != \"\") or (input.sourceKind == \"generated\" and input.generatedSourceName != \"\" and input.generatedSourceField != \"\") or (input.sourceKind == \"session\" and input.sessionSourceName != \"\" and input.sessionSourceField != \"\") or (input.sourceKind == \"invocation_argument\" and input.invocationArgumentSourceName != \"\" and input.invocationArgumentSourceField != \"\")\n  val modelCommandInputsHaveProvenance = modelCommandInputs.select(input => modelCommandInputHasProvenance(input)).length() == modelCommandInputs.length()\n  val modelCommandInputsTraceToInvocationSources = modelCommandInputs.select(input => modelCommandInputTracesToInvocationSource(input)).length() == modelCommandInputs.length()\n  def modelEventAttributeSourceIsComplete(eventAttr) = eventAttr.provenance != \"\" and ((eventAttr.sourceKind == \"command_input\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\") or (eventAttr.sourceKind == \"external_payload\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\") or (eventAttr.sourceKind == \"generated\" and eventAttr.sourceName != \"\" and eventAttr.generatedSourceKind != \"\") or (eventAttr.sourceKind == \"session\" and eventAttr.sourceName != \"\") or (eventAttr.sourceKind == \"derivation\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\"))\n  val modelEventAttributeSourcesAreComplete = modelEventAttributes.select(eventAttr => modelEventAttributeSourceIsComplete(eventAttr)).length() == modelEventAttributes.length()\n  val modelReadModelsAreDeclared = modelReadModels.length() == 0",
    )
    .replace(
        "\n  val modelScenarios:",
        "\n  def modelSliceBelongsToDeclaredWorkflow(modelSlice) = modelWorkflows.select(workflow => workflow == modelSlice.workflow).length() > 0\n  def modelSliceHasModule(modelSlice) = modelSliceModules.select(sliceModule => sliceModule.workflow == modelSlice.workflow and sliceModule.slice == modelSlice.slice and sliceModule.formalModule != \"\").length() > 0\n  def modelSliceModuleBelongsToDeclaredSlice(sliceModule) = sliceModule.formalModule != \"\" and modelSlices.select(modelSlice => modelSlice.workflow == sliceModule.workflow and modelSlice.slice == sliceModule.slice).length() > 0\n  def modelWorkflowSlicesHaveModules(workflow) = modelSlices.select(modelSlice => modelSlice.workflow == workflow and not(modelSliceHasModule(modelSlice))).length() == 0\n  def modelWorkflowHasCompositionStructure(workflow) = modelWorkflowSlicesHaveModules(workflow)\n  val modelScenarios:",
    )
    .replace(
        "\n  val modelScenariosAreDeclared",
        "\n  val modelWorkflowCompositionStructureComplete = modelSlices.select(modelSlice => modelSliceBelongsToDeclaredWorkflow(modelSlice)).length() == modelSlices.length() and modelSlices.select(modelSlice => modelSliceHasModule(modelSlice)).length() == modelSlices.length() and modelSliceModules.select(sliceModule => modelSliceModuleBelongsToDeclaredSlice(sliceModule)).length() == modelSliceModules.length() and modelWorkflows.select(workflow => modelWorkflowHasCompositionStructure(workflow)).length() == modelWorkflows.length()\n  val modelScenariosAreDeclared",
    )
    .replace(
        "val modelViewControlsProvideCommandInputs = modelViewControls.select(control => modelViewControlProvidesEveryCommandInput(control)).length() == modelViewControls.length()",
        "val modelViewControlsProvideCommandInputs = modelViewControls.select(control => modelViewControlProvidesEveryCommandInput(control)).length() == modelViewControls.length()\n  def modelOutcomeBranchIsModeled(outcome) = outcome.outcome != \"\" and outcome.events.length() > 0\n  def modelCommandErrorRecoveryIsModeled(commandError) = commandError.command != \"\" and commandError.error != \"\" and commandError.scenario != \"\" and commandError.recovery != \"\"\n  def modelViewControlNavigationTargetIsModeled(control) = control.navigationType == \"\" or ((control.navigationType == \"modeled_view\" or control.navigationType == \"local_view_state\") and control.navigationTarget != \"\") or (control.navigationType == \"external_workflow\" and control.externalWorkflow != \"\") or (control.navigationType == \"external_system\" and control.externalSystem != \"\" and control.handoffContract != \"\")\n  def modelExternalBoundaryContractIsModeled(translation) = translation.translation != \"\" and translation.externalEvent != \"\" and translation.payloadContract != \"\" and translation.command != \"\"\n  val modelWorkflowBehaviorSurfaceIsComplete = modelOutcomes.select(outcome => modelOutcomeBranchIsModeled(outcome)).length() == modelOutcomes.length() and modelCommandErrors.select(commandError => modelCommandErrorRecoveryIsModeled(commandError)).length() == modelCommandErrors.length() and modelViewControls.select(control => modelViewControlNavigationTargetIsModeled(control)).length() == modelViewControls.length() and modelTranslationDefinitions.select(translation => modelExternalBoundaryContractIsModeled(translation)).length() == modelTranslationDefinitions.length()",
    )
    .replace(
        "val modelDataFlows: List[ModelDataFlow] = []",
        "val modelDataFlows: List[ModelDataFlow] = []\n  val modelDataFlowCount = 0",
    )
    .replace(
        "def modelDataFlowIsBitComplete(dataFlow) = dataFlow.datum != \"\" and dataFlow.sourceKind != \"\" and dataFlow.source != \"\" and dataFlow.transformation != \"\" and dataFlow.target != \"\" and dataFlow.bitEncoding != \"\"\n  val modelDataFlowsAreBitComplete = modelDataFlows.select(dataFlow => modelDataFlowIsBitComplete(dataFlow)).length() == modelDataFlows.length()",
        "def modelDataFlowIsBitComplete(dataFlow) = dataFlow.datum != \"\" and dataFlow.sourceKind != \"\" and dataFlow.source != \"\" and dataFlow.transformation != \"\" and dataFlow.target != \"\" and dataFlow.bitEncoding != \"\"\n  val modelDataFlowsAreBitComplete = modelDataFlows.select(dataFlow => modelDataFlowIsBitComplete(dataFlow)).length() == modelDataFlows.length()\n  def modelDataFlowHasModeledTransformationSemantics(dataFlow) = dataFlow.transformation == \"identity\" or dataFlow.transformation == \"projection\" or dataFlow.transformation == \"derivation\" or dataFlow.transformation == \"default\" or dataFlow.transformation == \"absence\" or dataFlow.transformation == \"transformation\"\n  def modelDataFlowHasModeledSourceKind(dataFlow) = (dataFlow.sourceKind == \"original\" and dataFlow.source != \"\") or (dataFlow.sourceKind == \"modeled_target\" and dataFlow.source != \"\")\n  def modelDataFlowModeledSourceResolves(dataFlow) = dataFlow.sourceKind != \"modeled_target\" or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and modelDataFlowIsBitComplete(sourceFlow)).length() > 0\n  val modelDataFlowSourceKindsAreModeled = modelDataFlows.select(dataFlow => modelDataFlowHasModeledSourceKind(dataFlow)).length() == modelDataFlows.length()\n  val modelDataFlowModeledSourcesResolve = modelDataFlows.select(dataFlow => modelDataFlowModeledSourceResolves(dataFlow)).length() == modelDataFlows.length()\n  val modelDataFlowTransformationsAreModeled = modelDataFlows.select(dataFlow => modelDataFlowHasModeledTransformationSemantics(dataFlow)).length() == modelDataFlows.length()\n  def modelDataFlowCoversDatumTarget(workflow, sliceName, datum, target) = modelDataFlows.select(dataFlow => dataFlow.workflow == workflow and dataFlow.slice == sliceName and dataFlow.datum == datum and dataFlow.target == target and modelDataFlowIsBitComplete(dataFlow)).length() > 0\n  def modelDataFlowBitEncodingMatchesDatumTarget(workflow, sliceName, datum, target, bitEncoding) = modelDataFlows.select(dataFlow => dataFlow.workflow == workflow and dataFlow.slice == sliceName and dataFlow.datum == datum and dataFlow.target == target and dataFlow.bitEncoding == bitEncoding and modelDataFlowIsBitComplete(dataFlow)).length() > 0\n  def modelDataFlowSourceBitEncodingMatchesModeledSource(dataFlow) = modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source).length() == 0 or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and sourceFlow.bitEncoding == dataFlow.bitEncoding and modelDataFlowIsBitComplete(sourceFlow)).length() > 0\n  def modelCommandInputHasModeledDataFlow(input) = modelDataFlowCoversDatumTarget(input.workflow, input.slice, input.input, input.command)\n  def modelEventAttributeHasModeledDataFlow(eventAttr) = modelDataFlowCoversDatumTarget(eventAttr.workflow, eventAttr.slice, eventAttr.attribute, eventAttr.event)\n  def modelReadModelFieldHasModeledDataFlow(readModelField) = modelDataFlowCoversDatumTarget(readModelField.workflow, readModelField.slice, readModelField.field, readModelField.readModel)\n  def modelViewFieldHasModeledDataFlow(viewField) = modelDataFlowCoversDatumTarget(viewField.workflow, viewField.slice, viewField.field, viewField.view)\n  def modelViewFieldBitEncodingMatchesDataFlow(viewField) = modelDataFlowBitEncodingMatchesDatumTarget(viewField.workflow, viewField.slice, viewField.field, viewField.view, viewField.bitEncoding)\n  def modelExternalPayloadFieldHasModeledDataFlow(externalPayloadField) = modelDataFlowCoversDatumTarget(externalPayloadField.workflow, externalPayloadField.slice, externalPayloadField.field, externalPayloadField.externalPayload)\n  def modelExternalPayloadFieldBitEncodingMatchesDataFlow(externalPayloadField) = modelDataFlowBitEncodingMatchesDatumTarget(externalPayloadField.workflow, externalPayloadField.slice, externalPayloadField.field, externalPayloadField.externalPayload, externalPayloadField.bitEncoding)\n  val modelMeaningfulDataHasModeledDataFlows = modelCommandInputs.select(input => modelCommandInputHasModeledDataFlow(input)).length() == modelCommandInputs.length() and modelEventAttributes.select(eventAttr => modelEventAttributeHasModeledDataFlow(eventAttr)).length() == modelEventAttributes.length() and modelReadModelFields.select(readModelField => modelReadModelFieldHasModeledDataFlow(readModelField)).length() == modelReadModelFields.length() and modelViewFields.select(viewField => modelViewFieldHasModeledDataFlow(viewField)).length() == modelViewFields.length() and modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldHasModeledDataFlow(externalPayloadField)).length() == modelExternalPayloadFields.length()\n  val modelMeaningfulDataFlowsAreCovered = modelMeaningfulDataHasModeledDataFlows\n  val modelDataFlowSourceBitEncodingsMatchModeledSources = modelDataFlows.select(dataFlow => modelDataFlowSourceBitEncodingMatchesModeledSource(dataFlow)).length() == modelDataFlows.length()\n  val modelViewFieldBitEncodingsMatchDataFlows = modelViewFields.select(viewField => modelViewFieldBitEncodingMatchesDataFlow(viewField)).length() == modelViewFields.length()\n  val modelExternalPayloadFieldBitEncodingsMatchDataFlows = modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldBitEncodingMatchesDataFlow(externalPayloadField)).length() == modelExternalPayloadFields.length()",
    )
    .replace(
        "def modelDataFlowModeledSourceResolves(dataFlow) = dataFlow.sourceKind != \"modeled_target\" or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and modelDataFlowIsBitComplete(sourceFlow)).length() > 0\n  val modelDataFlowSourceKindsAreModeled",
        "def modelDataFlowModeledSourceResolves(dataFlow) = dataFlow.sourceKind != \"modeled_target\" or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and modelDataFlowIsBitComplete(sourceFlow)).length() > 0\n  def modelSameDataFlowTarget(left, right) = left.workflow == right.workflow and left.slice == right.slice and left.datum == right.datum and left.target == right.target\n  def modelDataFlowTargetsFromReachable(reachable) = modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"modeled_target\" and reachable.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and modelDataFlowIsBitComplete(sourceFlow)).length() > 0)\n  def modelDataFlowsReachableFromOriginalsAfterFuel(fuel, reachable) = range(0, fuel).foldl(reachable, (currentReachable, _) => currentReachable.concat(modelDataFlowTargetsFromReachable(currentReachable)))\n  val modelDataFlowsReachableFromOriginals = modelDataFlowsReachableFromOriginalsAfterFuel(modelDataFlowCount, modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"original\" and modelDataFlowIsBitComplete(dataFlow)))\n  def modelDataFlowHasOriginalSourceChain(dataFlow) = dataFlow.sourceKind == \"original\" or modelDataFlowsReachableFromOriginals.select(reachableFlow => modelSameDataFlowTarget(reachableFlow, dataFlow)).length() > 0\n  def modelDataFlowTargetsFromBitPreservingReachable(reachable) = modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"modeled_target\" and reachable.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and sourceFlow.bitEncoding == dataFlow.bitEncoding and modelDataFlowIsBitComplete(sourceFlow)).length() > 0)\n  def modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel(fuel, reachable) = range(0, fuel).foldl(reachable, (currentReachable, _) => currentReachable.concat(modelDataFlowTargetsFromBitPreservingReachable(currentReachable)))\n  val modelDataFlowsReachableFromOriginalsWithPreservedBits = modelDataFlowsReachableFromOriginalsWithPreservedBitsAfterFuel(modelDataFlowCount, modelDataFlows.select(dataFlow => dataFlow.sourceKind == \"original\" and modelDataFlowIsBitComplete(dataFlow)))\n  def modelDataFlowHasBitPreservingOriginalSourceChain(dataFlow) = dataFlow.sourceKind == \"original\" or modelDataFlowsReachableFromOriginalsWithPreservedBits.select(reachableFlow => modelSameDataFlowTarget(reachableFlow, dataFlow)).length() > 0\n  val modelDataFlowSourceKindsAreModeled",
    )
    .replace(
        "val modelDataFlowModeledSourcesResolve = modelDataFlows.select(dataFlow => modelDataFlowModeledSourceResolves(dataFlow)).length() == modelDataFlows.length()\n  val modelDataFlowTransformationsAreModeled",
        "val modelDataFlowModeledSourcesResolve = modelDataFlows.select(dataFlow => modelDataFlowModeledSourceResolves(dataFlow)).length() == modelDataFlows.length()\n  val modelDataFlowSourceChainsReachOriginals = modelDataFlows.select(dataFlow => modelDataFlowHasOriginalSourceChain(dataFlow)).length() == modelDataFlows.length()\n  val modelDataFlowSourceChainsPreserveBitEncodingSemantics = modelDataFlows.select(dataFlow => modelDataFlowHasBitPreservingOriginalSourceChain(dataFlow)).length() == modelDataFlows.length()\n  val modelDataFlowTransformationsAreModeled",
    )
    .replace(
        "val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == 0\n  val modelViewsAreDeclared = modelViews.length() == 0",
        "val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == 0\n  def modelReadModelFieldSourceIsComplete(readModelField) = (readModelField.sourceKind == \"event_attribute\" and readModelField.sourceEvent != \"\" and readModelField.sourceAttribute != \"\") or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\" and readModelField.derivationSourceFields.length() > 0) or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\")\n  def modelReadModelFieldTracesToOriginalProvenance(readModelField) = readModelField.provenance != \"\" and ((readModelField.sourceKind == \"event_attribute\" and modelEventAttributes.select(eventAttr => eventAttr.workflow == readModelField.workflow and eventAttr.slice == readModelField.slice and eventAttr.event == readModelField.sourceEvent and eventAttr.attribute == readModelField.sourceAttribute and modelEventAttributeSourceIsComplete(eventAttr)).length() > 0) or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\" and readModelField.derivationSourceFields.length() > 0) or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\"))\n  val modelReadModelFieldSourcesAreComplete = modelReadModelFields.select(readModelField => modelReadModelFieldSourceIsComplete(readModelField)).length() == modelReadModelFields.length()\n  def modelViewFieldSourceIsComplete(viewField) = viewField.sourceKind == \"read_model\" and viewField.sourceReadModel != \"\" and viewField.sourceField != \"\" and viewField.provenance != \"\" and viewField.bitEncoding != \"\"\n  def modelViewFieldReadModelFieldSourceResolves(viewField) = modelViewFieldSourceIsComplete(viewField) and modelReadModelFields.select(readModelField => readModelField.workflow == viewField.workflow and readModelField.slice == viewField.slice and readModelField.readModel == viewField.sourceReadModel and readModelField.field == viewField.sourceField and modelReadModelFieldSourceIsComplete(readModelField)).length() > 0\n  val modelViewFieldReadModelFieldSourcesResolve = modelViewFields.select(viewField => modelViewFieldReadModelFieldSourceResolves(viewField)).length() == modelViewFields.length()\n  def modelDisplayedDatumTracesToOriginalProvenance(viewField) = modelViewFieldReadModelFieldSourceResolves(viewField) and modelReadModelFields.select(readModelField => readModelField.workflow == viewField.workflow and readModelField.slice == viewField.slice and readModelField.readModel == viewField.sourceReadModel and readModelField.field == viewField.sourceField and modelReadModelFieldTracesToOriginalProvenance(readModelField)).length() > 0\n  val modelDisplayedDataTraceToOriginalProvenance = modelViewFields.select(viewField => modelDisplayedDatumTracesToOriginalProvenance(viewField)).length() == modelViewFields.length()\n  val modelViewFieldSourcesAreComplete = modelViewFields.select(viewField => modelViewFieldSourceIsComplete(viewField)).length() == modelViewFields.length()\n  def modelExternalPayloadFieldHasProvenance(externalPayloadField) = externalPayloadField.provenance != \"\" and externalPayloadField.bitEncoding != \"\"\n  val modelExternalPayloadFieldsHaveProvenance = modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldHasProvenance(externalPayloadField)).length() == modelExternalPayloadFields.length()\n  val modelViewsAreDeclared = modelViews.length() == 0",
    ))
}

fn lean_workflow_slug_list(workflow_slugs: &[WorkflowSlug]) -> String {
    let mut workflow_slugs = workflow_slugs
        .iter()
        .map(|slug| slug.as_ref())
        .collect::<Vec<_>>();
    workflow_slugs.sort_unstable();
    format!(
        "[{}]",
        workflow_slugs
            .into_iter()
            .map(|slug| format!("{slug:?}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_workflow_slug_list(workflow_slugs: &[WorkflowSlug]) -> String {
    let mut workflow_slugs = workflow_slugs
        .iter()
        .map(|slug| slug.as_ref())
        .collect::<Vec<_>>();
    workflow_slugs.sort_unstable();
    format!(
        "[{}]",
        workflow_slugs
            .into_iter()
            .map(|slug| format!("{slug:?}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn formal_model_slices(slice_memberships: &[ProjectSliceMembership]) -> Vec<FormalModelSlice> {
    slice_memberships
        .iter()
        .map(ProjectSliceMembership::formal_model_slice)
        .collect()
}

fn formal_model_slice_modules(
    slice_memberships: &[ProjectSliceMembership],
) -> Vec<FormalModelSliceModule> {
    slice_memberships
        .iter()
        .map(ProjectSliceMembership::formal_model_slice_module)
        .collect()
}

fn model_digest(
    project_name: &ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> String {
    let canonical_source = format!(
        "project:name={};version={FORMAL_MODEL_VERSION};workflows={};slices={};scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=",
        project_name.as_ref(),
        digest_workflows(workflow_slugs),
        digest_slices(slice_memberships)
    );
    hex::encode(Sha256::digest(canonical_source.as_bytes()))
}

fn digest_workflows(workflow_slugs: &[WorkflowSlug]) -> String {
    let mut workflow_slugs = workflow_slugs
        .iter()
        .map(|slug| slug.as_ref())
        .collect::<Vec<_>>();
    workflow_slugs.sort_unstable();
    workflow_slugs.join(",")
}

fn digest_slices(slice_memberships: &[ProjectSliceMembership]) -> String {
    let mut slice_memberships = slice_memberships
        .iter()
        .map(|membership| {
            (
                membership.workflow_slug.as_ref(),
                membership.slice_slug.as_ref(),
                membership.slice_module.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    slice_memberships.sort_unstable();
    slice_memberships
        .into_iter()
        .map(|(workflow_slug, slice_slug, slice_module)| {
            format!("{workflow_slug}/{slice_slug}@{slice_module}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn module_name(project_name: &ProjectName) -> String {
    let mut capitalize_next = true;
    project_name
        .as_ref()
        .chars()
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

fn project_path(value: impl Into<String>) -> ProjectPath {
    ProjectPath::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static project path must be valid: {error}");
    })
}

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static file contents must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static report line must be valid: {error}");
    })
}
