// Copyright 2026 John Wilger

use nutype::nutype;

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath, ReportLine};
use crate::core::types::{LeanModuleName, SliceSlug, WorkflowSlug};

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
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
        Effect::WriteFileIfMissing(
            project_path("emc.toml"),
            file_contents(format!(
                "[project]\nname = \"{project_name_text}\"\nversion = \"{FORMAL_MODEL_VERSION}\"\nlean_module = \"{module_name}\"\nquint_module = \"{module_name}\"\n"
            )),
        ),
        Effect::EnsureDirectory(project_path("model/lean")),
        Effect::WriteFileIfMissing(
            project_path("model/lean/lean-toolchain"),
            file_contents("leanprover/lean4:4.29.1\n"),
        ),
        Effect::WriteFileIfMissing(
            project_path("model/lean/lakefile.lean"),
            file_contents("package EMCModel\n"),
        ),
        Effect::WriteFileIfMissing(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_project_root(&project_name, &[], &[]),
        ),
        Effect::WriteFileIfMissing(
            project_path("model/lean/slices/.gitkeep"),
            file_contents("\n"),
        ),
        Effect::EnsureDirectory(project_path("model/quint")),
        Effect::WriteFileIfMissing(
            project_path("model/quint/quint.json"),
            file_contents(format!(
                "{{\n  \"main\": \"{module_name}.qnt\",\n  \"invariants\": [\n    \"workflowIdentityStable\",\n    \"workflowSliceDetailsComplete\",\n    \"workflowSliceModulesComplete\",\n    \"workflowTransitionsStructured\",\n    \"workflowTransitionSourcesResolve\",\n    \"workflowTransitionTargetsResolve\",\n    \"workflowStepRelationshipsAreAllowed\",\n    \"workflowStepSlugsAreUnique\",\n    \"workflowHasExactlyOneEntryStep\",\n    \"workflowMainStepsHaveIncomingReachability\",\n    \"workflowNonSupportingStepsReachableFromEntry\",\n    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\",\n    \"workflowTransitionsHaveModeledKinds\",\n    \"workflowExitsNameTargetsAndRationale\",\n    \"workflowExternallyRelevantOutcomesHandled\",\n    \"workflowOutcomesSourceResolve\",\n    \"workflowCommandErrorsSourceResolve\",\n    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\",\n    \"workflowNonEventDefinitionsAreUniquelyOwned\",\n    \"workflowSharedEventDefinitionsHaveIdenticalIdentity\",\n    \"workflowCommandTransitionsResolveControlsAndCommands\",\n    \"workflowEventTransitionsAreSharedByEndpointSlices\",\n    \"workflowNavigationTransitionsResolveControlsAndViews\",\n    \"workflowExternalTriggersDeclarePayloadContracts\",\n    \"workflowExternalTriggerPayloadContractsHaveProvenance\",\n    \"workflowTransitionsHaveRequiredEvidence\",\n    \"workflowEntryLifecycleStatesCoverRequiredStates\",\n    \"modelScenarioDefinitionsHaveGwt\",\n    \"modelScenarioKindsAreFirstClass\",\n    \"modelDataFlowsAreBitComplete\",\n    \"modelCommandInputsHaveProvenance\",\n    \"modelCommandInputsTraceToInvocationSources\",\n    \"modelReadModelFieldSourcesAreComplete\",\n    \"modelViewControlsProvideCommandInputs\"\n  ]\n}}\n"
            )),
        ),
        Effect::WriteFileIfMissing(
            project_path(format!("model/quint/{module_name}.qnt")),
            emit_quint_project_root(&project_name, &[], &[]),
        ),
        Effect::WriteFileIfMissing(
            project_path("model/quint/slices/.gitkeep"),
            file_contents("\n"),
        ),
        Effect::EnsureDirectory(project_path("reviews")),
        Effect::WriteFileIfMissing(project_path("reviews/.gitkeep"), file_contents("\n")),
        Effect::Report(report_line(format!(
            "EMC project {project_name} layout is present"
        ))),
    ])
}

pub fn project_root_effects(
    project_name: ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> [Effect; 2] {
    let module_name = module_name(&project_name);
    [
        Effect::WriteFile(
            project_path(format!("model/lean/{module_name}.lean")),
            emit_lean_project_root(&project_name, workflow_slugs, slice_memberships),
        ),
        Effect::WriteFile(
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
    let slice_list = lean_slice_membership_list(slice_memberships);
    let slice_module_list = lean_slice_module_list(slice_memberships);
    let slice_count = slice_memberships.len();
    let model_digest = model_digest(project_name, workflow_slugs, slice_memberships);
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC generated Lean4 model root.\n\ndef modelName := {project_name:?}\n\ndef modelVersion := \"{FORMAL_MODEL_VERSION}\"\n\ndef modelDigest := {model_digest:?}\n\ndef modelWorkflows : List String := {workflow_list}\n\ndef modelSlices : List (String × String) := {slice_list}\n\ndef modelSliceModules : List (String × String × String) := {slice_module_list}\n\ndef modelScenarios : List (String × String × String × String) := []\n\ndef modelScenarioDefinitions : List (String × String × String × String × String × String × String × List String × List String × String × String × List String) := []\n\ndef modelDataFlows : List (String × String × String × String × String × String × String) := []\n\ndef modelOutcomes : List (String × String × String × List String × Bool) := []\n\ndef modelCommandErrors : List (String × String × String × String × String × String) := []\n\ndef modelCommands : List (String × String × String) := []\n\ndef modelCommandInputs : List (String × String × String × String × String × String × List String × String × String × String × String × String × String × String × String) := []\n\ndef modelReadModels : List (String × String × String) := []\n\ndef modelReadModelDefinitions : List (String × String × String × Bool × List String × String × String) := []\n\ndef modelReadModelFields : List (String × String × String × String × String × String × String × String × List String × String × String × String × String) := []\n\ndef modelViews : List (String × String × String) := []\n\ndef modelViewDefinitions : List (String × String × String × List String × List String × List String × List String) := []\n\ndef modelViewControls : List (String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) := []\n\ndef modelBoardElements : List (String × String × String × String × String × String × Bool) := []\n\ndef modelBoardConnections : List (String × String × String × String × String × String) := []\n\ndef modelViewFields : List (String × String × String × String × String × String × String × String × String) := []\n\ndef modelAutomations : List (String × String × String) := []\n\ndef modelAutomationDefinitions : List (String × String × String × String × String × List String × String) := []\n\ndef modelTranslations : List (String × String × String) := []\n\ndef modelTranslationDefinitions : List (String × String × String × String × String × String) := []\n\ndef modelExternalPayloads : List (String × String × String) := []\n\ndef modelExternalPayloadFields : List (String × String × String × String × String × String) := []\n\ndef modelStreams : List (String × String × String) := []\n\ndef modelEvents : List (String × String × String × String) := []\n\ndef modelEventAttributes : List (String × String × String × String × String × String × String × String) := []\n\ndef modelScenarioDefinitionHasGwt (scenario : String × String × String × String × String × String × String × List String × List String × String × String × List String) : Bool := scenario.2.2.2.2.1.isEmpty == false && scenario.2.2.2.2.2.1.isEmpty == false && scenario.2.2.2.2.2.2.1.isEmpty == false\n\ndef modelScenarioKindIsFirstClass (scenario : String × String × String × String × String × String × String × List String × List String × String × String × List String) : Bool := scenario.2.2.1 == \"acceptance\" || scenario.2.2.1 == \"contract\"\n\ndef modelDataFlowIsBitComplete (dataFlow : String × String × String × String × String × String × String) : Bool := dataFlow.2.2.1.isEmpty == false && dataFlow.2.2.2.1.isEmpty == false && dataFlow.2.2.2.2.1.isEmpty == false && dataFlow.2.2.2.2.2.1.isEmpty == false && dataFlow.2.2.2.2.2.2.isEmpty == false\n\ndef modelCommandInputHasProvenance (input : String × String × String × String × String × String × List String × String × String × String × String × String × String × String × String) : Bool := input.2.2.2.2.2.1.isEmpty == false && input.2.2.2.2.2.2.1.isEmpty == false\n\ndef modelCommandInputTracesToInvocationSource (input : String × String × String × String × String × String × List String × String × String × String × String × String × String × String × String) : Bool := input.2.2.2.2.1 == \"actor\" || (input.2.2.2.2.1 == \"event_stream_state\" && input.2.2.2.2.2.2.2.1.isEmpty == false && input.2.2.2.2.2.2.2.2.1.isEmpty == false) || (input.2.2.2.2.1 == \"external_payload\" && input.2.2.2.2.2.2.2.2.2.1.isEmpty == false && input.2.2.2.2.2.2.2.2.2.2.1.isEmpty == false) || (input.2.2.2.2.1 == \"generated\" && input.2.2.2.2.2.2.2.2.2.2.2.1.isEmpty == false && input.2.2.2.2.2.2.2.2.2.2.2.2.1.isEmpty == false) || (input.2.2.2.2.1 == \"session\" && input.2.2.2.2.2.2.2.2.2.2.2.2.2.1.isEmpty == false && input.2.2.2.2.2.2.2.2.2.2.2.2.2.2.isEmpty == false)\n\ndef modelReadModelFieldSourceIsComplete (field : String × String × String × String × String × String × String × String × List String × String × String × String × String) : Bool := (field.2.2.2.2.1 == \"event_attribute\" && field.2.2.2.2.2.1.isEmpty == false && field.2.2.2.2.2.2.1.isEmpty == false) || (field.2.2.2.2.1 == \"derivation\" && field.2.2.2.2.2.2.2.1.isEmpty == false && field.2.2.2.2.2.2.2.2.1.isEmpty == false) || (field.2.2.2.2.1 == \"absence_default\" && field.2.2.2.2.2.2.2.2.2.1.isEmpty == false)\n\ndef modelControlProvidesCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) (input : String × String × String × String × String × String × List String × String × String × String × String × String × String × String × String) : Bool := control.1 == input.1 && control.2.2.2.2.1 == input.2.2.1 && control.2.2.2.2.2.1 == input.2.2.2.1\n\ndef modelViewControlProvidesEveryCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) : Bool := modelCommandInputs.all (fun input => input.1 != control.1 || input.2.2.1 != control.2.2.2.2.1 || modelViewControls.any (fun providedInput => providedInput.1 == control.1 && providedInput.2.1 == control.2.1 && providedInput.2.2.1 == control.2.2.1 && providedInput.2.2.2.1 == control.2.2.2.1 && providedInput.2.2.2.2.1 == control.2.2.2.2.1 && modelControlProvidesCommandInput providedInput input))\n\ntheorem modelIdentityIsStable : modelName = {project_name:?} := rfl\n\ntheorem modelVersionIsStable : modelVersion = \"{FORMAL_MODEL_VERSION}\" := rfl\n\ntheorem modelDigestIsStable : modelDigest = {model_digest:?} := rfl\n\ntheorem modelWorkflowsAreDeclared : modelWorkflows.length = {workflow_count} := rfl\n\ntheorem modelSlicesAreDeclared : modelSlices.length = {slice_count} := rfl\n\ntheorem modelSliceModulesAreDeclared : modelSliceModules.length = {slice_count} := rfl\n\ntheorem modelScenariosAreDeclared : modelScenarios.length = 0 := rfl\n\ntheorem modelScenarioDefinitionsAreDeclared : modelScenarioDefinitions.length = 0 := rfl\n\ntheorem modelScenarioDefinitionsHaveGwt : modelScenarioDefinitions.all modelScenarioDefinitionHasGwt = true := rfl\n\ntheorem modelScenarioKindsAreFirstClass : modelScenarioDefinitions.all modelScenarioKindIsFirstClass = true := rfl\n\ntheorem modelDataFlowsAreDeclared : modelDataFlows.length = 0 := rfl\n\ntheorem modelDataFlowsAreBitComplete : modelDataFlows.all modelDataFlowIsBitComplete = true := rfl\n\ntheorem modelOutcomesAreDeclared : modelOutcomes.length = 0 := rfl\n\ntheorem modelCommandErrorsAreDeclared : modelCommandErrors.length = 0 := rfl\n\ntheorem modelCommandsAreDeclared : modelCommands.length = 0 := rfl\n\ntheorem modelCommandInputsAreDeclared : modelCommandInputs.length = 0 := rfl\n\ntheorem modelCommandInputsHaveProvenance : modelCommandInputs.all modelCommandInputHasProvenance = true := rfl\n\ntheorem modelCommandInputsTraceToInvocationSources : modelCommandInputs.all modelCommandInputTracesToInvocationSource = true := rfl\n\ntheorem modelReadModelsAreDeclared : modelReadModels.length = 0 := rfl\n\ntheorem modelReadModelDefinitionsAreDeclared : modelReadModelDefinitions.length = 0 := rfl\n\ntheorem modelReadModelFieldsAreDeclared : modelReadModelFields.length = 0 := rfl\n\ntheorem modelReadModelFieldSourcesAreComplete : modelReadModelFields.all modelReadModelFieldSourceIsComplete = true := rfl\n\ntheorem modelViewsAreDeclared : modelViews.length = 0 := rfl\n\ntheorem modelViewDefinitionsAreDeclared : modelViewDefinitions.length = 0 := rfl\n\ntheorem modelViewControlsAreDeclared : modelViewControls.length = 0 := rfl\n\ntheorem modelViewControlsProvideCommandInputs : modelViewControls.all modelViewControlProvidesEveryCommandInput = true := rfl\n\ntheorem modelBoardElementsAreDeclared : modelBoardElements.length = 0 := rfl\n\ntheorem modelBoardConnectionsAreDeclared : modelBoardConnections.length = 0 := rfl\n\ntheorem modelViewFieldsAreDeclared : modelViewFields.length = 0 := rfl\n\ntheorem modelAutomationsAreDeclared : modelAutomations.length = 0 := rfl\n\ntheorem modelAutomationDefinitionsAreDeclared : modelAutomationDefinitions.length = 0 := rfl\n\ntheorem modelTranslationsAreDeclared : modelTranslations.length = 0 := rfl\n\ntheorem modelTranslationDefinitionsAreDeclared : modelTranslationDefinitions.length = 0 := rfl\n\ntheorem modelExternalPayloadsAreDeclared : modelExternalPayloads.length = 0 := rfl\n\ntheorem modelExternalPayloadFieldsAreDeclared : modelExternalPayloadFields.length = 0 := rfl\n\ntheorem modelStreamsAreDeclared : modelStreams.length = 0 := rfl\n\ntheorem modelEventsAreDeclared : modelEvents.length = 0 := rfl\n\ntheorem modelEventAttributesAreDeclared : modelEventAttributes.length = 0 := rfl\n\nend {module_name}\n",
        project_name = project_name.as_ref(),
    ))
}

fn emit_quint_project_root(
    project_name: &ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> FileContents {
    let module_name = module_name(project_name);
    let workflow_list = quint_workflow_slug_list(workflow_slugs);
    let workflow_count = workflow_slugs.len();
    let slice_list = quint_slice_membership_list(slice_memberships);
    let slice_module_list = quint_slice_module_list(slice_memberships);
    let slice_count = slice_memberships.len();
    let model_digest = model_digest(project_name, workflow_slugs, slice_memberships);
    file_contents(format!(
        "module {module_name} {{\n  type ModelSlice = {{ workflow: str, slice: str }}\n  type ModelSliceModule = {{ workflow: str, slice: str, formalModule: str }}\n  type ModelScenario = {{ workflow: str, slice: str, scenarioKind: str, scenario: str }}\n  type ModelScenarioDefinition = {{ workflow: str, slice: str, scenarioKind: str, scenario: str, given: str, when: str, then: str, readStreams: List[str], writtenStreams: List[str], contractKind: str, coveredDefinition: str, errorReferences: List[str] }}\n  type ModelDataFlow = {{ workflow: str, slice: str, datum: str, source: str, transformation: str, target: str, bitEncoding: str }}\n  type ModelOutcome = {{ workflow: str, slice: str, outcome: str, events: List[str], externallyRelevant: bool }}\n  type ModelCommandError = {{ workflow: str, slice: str, command: str, error: str, scenario: str, recovery: str }}\n  type ModelCommand = {{ workflow: str, slice: str, command: str }}\n  type ModelCommandInput = {{ workflow: str, slice: str, command: str, input: str, sourceKind: str, sourceDescription: str, provenanceChain: List[str], eventStreamSourceEvent: str, eventStreamSourceAttribute: str, externalPayloadSourceName: str, externalPayloadSourceField: str, generatedSourceName: str, generatedSourceField: str, sessionSourceName: str, sessionSourceField: str }}\n  type ModelReadModel = {{ workflow: str, slice: str, readModel: str }}\n  type ModelReadModelDefinition = {{ workflow: str, slice: str, readModel: str, transitive: bool, relationshipFields: List[str], transitiveRule: str, exampleScenarioName: str }}\n  type ModelReadModelField = {{ workflow: str, slice: str, readModel: str, field: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, derivationSourceFields: List[str], absenceEvent: str, derivationScenarioName: str, absenceScenarioName: str, provenance: str }}\n  type ModelView = {{ workflow: str, slice: str, view: str }}\n  type ModelViewDefinition = {{ workflow: str, slice: str, view: str, readModels: List[str], sketchTokens: List[str], localStates: List[str], filters: List[str] }}\n  type ModelViewControl = {{ workflow: str, slice: str, view: str, control: str, command: str, input: str, inputSourceKind: str, inputSourceDescription: str, inputSketchToken: str, inputVisibleToActor: bool, inputDecisionField: bool, handledErrors: List[str], recoveryBehavior: str, controlSketchToken: str, navigationType: str, navigationTarget: str, externalWorkflow: str, externalSystem: str, handoffContract: str }}\n  type ModelBoardElement = {{ workflow: str, slice: str, element: str, kind: str, lane: str, declaredName: str, mainPath: bool }}\n  type ModelBoardConnection = {{ workflow: str, slice: str, source: str, sourceKind: str, target: str, targetKind: str }}\n  type ModelViewField = {{ workflow: str, slice: str, view: str, field: str, sourceKind: str, sourceReadModel: str, sourceField: str, provenance: str, bitEncoding: str }}\n  type ModelAutomation = {{ workflow: str, slice: str, automation: str }}\n  type ModelAutomationDefinition = {{ workflow: str, slice: str, automation: str, trigger: str, command: str, handledErrors: List[str], reaction: str }}\n  type ModelTranslation = {{ workflow: str, slice: str, translation: str }}\n  type ModelTranslationDefinition = {{ workflow: str, slice: str, translation: str, externalEvent: str, payloadContract: str, command: str }}\n  type ModelExternalPayload = {{ workflow: str, slice: str, externalPayload: str }}\n  type ModelExternalPayloadField = {{ workflow: str, slice: str, externalPayload: str, field: str, provenance: str, bitEncoding: str }}\n  type ModelStream = {{ workflow: str, slice: str, stream: str }}\n  type ModelEvent = {{ workflow: str, slice: str, event: str, stream: str }}\n  type ModelEventAttribute = {{ workflow: str, slice: str, event: str, attribute: str, sourceKind: str, sourceName: str, sourceField: str, provenance: str }}\n  val modelName = {project_name:?}\n  val modelVersion = \"{FORMAL_MODEL_VERSION}\"\n  val modelDigest = {model_digest:?}\n  val modelWorkflows: List[str] = {workflow_list}\n  val modelSlices: List[ModelSlice] = {slice_list}\n  val modelSliceModules: List[ModelSliceModule] = {slice_module_list}\n  val modelScenarios: List[ModelScenario] = []\n  val modelScenarioDefinitions: List[ModelScenarioDefinition] = []\n  val modelDataFlows: List[ModelDataFlow] = []\n  val modelOutcomes: List[ModelOutcome] = []\n  val modelCommandErrors: List[ModelCommandError] = []\n  val modelCommands: List[ModelCommand] = []\n  val modelCommandInputs: List[ModelCommandInput] = []\n  val modelReadModels: List[ModelReadModel] = []\n  val modelReadModelDefinitions: List[ModelReadModelDefinition] = []\n  val modelReadModelFields: List[ModelReadModelField] = []\n  val modelViews: List[ModelView] = []\n  val modelViewDefinitions: List[ModelViewDefinition] = []\n  val modelViewControls: List[ModelViewControl] = []\n  val modelBoardElements: List[ModelBoardElement] = []\n  val modelBoardConnections: List[ModelBoardConnection] = []\n  val modelViewFields: List[ModelViewField] = []\n  val modelAutomations: List[ModelAutomation] = []\n  val modelAutomationDefinitions: List[ModelAutomationDefinition] = []\n  val modelTranslations: List[ModelTranslation] = []\n  val modelTranslationDefinitions: List[ModelTranslationDefinition] = []\n  val modelExternalPayloads: List[ModelExternalPayload] = []\n  val modelExternalPayloadFields: List[ModelExternalPayloadField] = []\n  val modelStreams: List[ModelStream] = []\n  val modelEvents: List[ModelEvent] = []\n  val modelEventAttributes: List[ModelEventAttribute] = []\n  val modelIdentityStable = modelName == {project_name:?}\n  val modelVersionStable = modelVersion == \"{FORMAL_MODEL_VERSION}\"\n  val modelDigestStable = modelDigest == {model_digest:?}\n  val modelWorkflowsAreDeclared = modelWorkflows.length() == {workflow_count}\n  val modelSlicesAreDeclared = modelSlices.length() == {slice_count}\n  val modelSliceModulesAreDeclared = modelSliceModules.length() == {slice_count}\n  val modelScenariosAreDeclared = modelScenarios.length() == 0\n  val modelScenarioDefinitionsAreDeclared = modelScenarioDefinitions.length() == 0\n  def modelScenarioDefinitionHasGwt(scenario) = scenario.given != \"\" and scenario.when != \"\" and scenario.then != \"\"\n  def modelScenarioKindIsFirstClass(scenario) = scenario.scenarioKind == \"acceptance\" or scenario.scenarioKind == \"contract\"\n  val modelScenarioDefinitionsHaveGwt = modelScenarioDefinitions.select(scenario => modelScenarioDefinitionHasGwt(scenario)).length() == modelScenarioDefinitions.length()\n  val modelScenarioKindsAreFirstClass = modelScenarioDefinitions.select(scenario => modelScenarioKindIsFirstClass(scenario)).length() == modelScenarioDefinitions.length()\n  val modelDataFlowsAreDeclared = modelDataFlows.length() == 0\n  def modelDataFlowIsBitComplete(dataFlow) = dataFlow.datum != \"\" and dataFlow.source != \"\" and dataFlow.transformation != \"\" and dataFlow.target != \"\" and dataFlow.bitEncoding != \"\"\n  val modelDataFlowsAreBitComplete = modelDataFlows.select(dataFlow => modelDataFlowIsBitComplete(dataFlow)).length() == modelDataFlows.length()\n  val modelOutcomesAreDeclared = modelOutcomes.length() == 0\n  val modelCommandErrorsAreDeclared = modelCommandErrors.length() == 0\n  val modelCommandsAreDeclared = modelCommands.length() == 0\n  val modelCommandInputsAreDeclared = modelCommandInputs.length() == 0\n  val modelReadModelsAreDeclared = modelReadModels.length() == 0\n  val modelReadModelDefinitionsAreDeclared = modelReadModelDefinitions.length() == 0\n  val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == 0\n  val modelViewsAreDeclared = modelViews.length() == 0\n  val modelViewDefinitionsAreDeclared = modelViewDefinitions.length() == 0\n  val modelViewControlsAreDeclared = modelViewControls.length() == 0\n  def modelControlProvidesCommandInput(control, input) = control.workflow == input.workflow and control.command == input.command and control.input == input.input\n  def modelViewControlProvidesEveryCommandInput(control) = modelCommandInputs.select(input => input.workflow != control.workflow or input.command != control.command or modelViewControls.select(providedInput => providedInput.workflow == control.workflow and providedInput.slice == control.slice and providedInput.view == control.view and providedInput.control == control.control and providedInput.command == control.command and modelControlProvidesCommandInput(providedInput, input)).length() > 0).length() == modelCommandInputs.length()\n  val modelViewControlsProvideCommandInputs = modelViewControls.select(control => modelViewControlProvidesEveryCommandInput(control)).length() == modelViewControls.length()\n  val modelBoardElementsAreDeclared = modelBoardElements.length() == 0\n  val modelBoardConnectionsAreDeclared = modelBoardConnections.length() == 0\n  val modelViewFieldsAreDeclared = modelViewFields.length() == 0\n  val modelAutomationsAreDeclared = modelAutomations.length() == 0\n  val modelAutomationDefinitionsAreDeclared = modelAutomationDefinitions.length() == 0\n  val modelTranslationsAreDeclared = modelTranslations.length() == 0\n  val modelTranslationDefinitionsAreDeclared = modelTranslationDefinitions.length() == 0\n  val modelExternalPayloadsAreDeclared = modelExternalPayloads.length() == 0\n  val modelExternalPayloadFieldsAreDeclared = modelExternalPayloadFields.length() == 0\n  val modelStreamsAreDeclared = modelStreams.length() == 0\n  val modelEventsAreDeclared = modelEvents.length() == 0\n  val modelEventAttributesAreDeclared = modelEventAttributes.length() == 0\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}}\n",
        project_name = project_name.as_ref(),
    )
    .replace(
        "val modelCommandInputsAreDeclared = modelCommandInputs.length() == 0\n  val modelReadModelsAreDeclared = modelReadModels.length() == 0",
        "val modelCommandInputsAreDeclared = modelCommandInputs.length() == 0\n  def modelCommandInputHasProvenance(input) = input.sourceDescription != \"\" and input.provenanceChain.length() > 0\n  def modelCommandInputTracesToInvocationSource(input) = input.sourceKind == \"actor\" or (input.sourceKind == \"event_stream_state\" and input.eventStreamSourceEvent != \"\" and input.eventStreamSourceAttribute != \"\") or (input.sourceKind == \"external_payload\" and input.externalPayloadSourceName != \"\" and input.externalPayloadSourceField != \"\") or (input.sourceKind == \"generated\" and input.generatedSourceName != \"\" and input.generatedSourceField != \"\") or (input.sourceKind == \"session\" and input.sessionSourceName != \"\" and input.sessionSourceField != \"\")\n  val modelCommandInputsHaveProvenance = modelCommandInputs.select(input => modelCommandInputHasProvenance(input)).length() == modelCommandInputs.length()\n  val modelCommandInputsTraceToInvocationSources = modelCommandInputs.select(input => modelCommandInputTracesToInvocationSource(input)).length() == modelCommandInputs.length()\n  val modelReadModelsAreDeclared = modelReadModels.length() == 0",
    )
    .replace(
        "val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == 0\n  val modelViewsAreDeclared = modelViews.length() == 0",
        "val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == 0\n  def modelReadModelFieldSourceIsComplete(readModelField) = (readModelField.sourceKind == \"event_attribute\" and readModelField.sourceEvent != \"\" and readModelField.sourceAttribute != \"\") or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\" and readModelField.derivationSourceFields.length() > 0) or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\")\n  val modelReadModelFieldSourcesAreComplete = modelReadModelFields.select(readModelField => modelReadModelFieldSourceIsComplete(readModelField)).length() == modelReadModelFields.length()\n  val modelViewsAreDeclared = modelViews.length() == 0",
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

fn lean_slice_membership_list(slice_memberships: &[ProjectSliceMembership]) -> String {
    let mut slice_memberships = slice_memberships
        .iter()
        .map(|membership| {
            (
                membership.workflow_slug.as_ref(),
                membership.slice_slug.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    slice_memberships.sort_unstable();
    format!(
        "[{}]",
        slice_memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug)| { format!("({workflow_slug:?}, {slice_slug:?})") })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_slice_membership_list(slice_memberships: &[ProjectSliceMembership]) -> String {
    let mut slice_memberships = slice_memberships
        .iter()
        .map(|membership| {
            (
                membership.workflow_slug.as_ref(),
                membership.slice_slug.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    slice_memberships.sort_unstable();
    format!(
        "[{}]",
        slice_memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug)| {
                format!("{{ workflow: {workflow_slug:?}, slice: {slice_slug:?} }}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn lean_slice_module_list(slice_memberships: &[ProjectSliceMembership]) -> String {
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
    format!(
        "[{}]",
        slice_memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug, slice_module)| {
                format!("({workflow_slug:?}, {slice_slug:?}, {slice_module:?})")
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn quint_slice_module_list(slice_memberships: &[ProjectSliceMembership]) -> String {
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
    format!(
        "[{}]",
        slice_memberships
            .into_iter()
            .map(|(workflow_slug, slice_slug, slice_module)| {
                format!(
                    "{{ workflow: {workflow_slug:?}, slice: {slice_slug:?}, formalModule: {slice_module:?} }}"
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn model_digest(
    project_name: &ProjectName,
    workflow_slugs: &[WorkflowSlug],
    slice_memberships: &[ProjectSliceMembership],
) -> String {
    format!(
        "project:name={};version={FORMAL_MODEL_VERSION};workflows={};slices={};scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=",
        project_name.as_ref(),
        digest_workflows(workflow_slugs),
        digest_slices(slice_memberships)
    )
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
