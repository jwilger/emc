// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn init_creates_deterministic_project_layout() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "EMC project Repair Desk layout is present",
            ));

        let expected_paths = [
            "emc.toml",
            "model/lean/lakefile.lean",
            "model/lean/lean-toolchain",
            "model/lean/RepairDesk.lean",
            "model/lean/slices/.gitkeep",
            "model/quint/RepairDesk.qnt",
            "model/quint/slices/.gitkeep",
            "reviews/.gitkeep",
        ];

        expected_paths
            .iter()
            .map(|relative_path| temp_dir.path().join(relative_path))
            .for_each(|path| assert!(path.exists(), "expected {} to exist", path.display()));

        assert_eq!(
            fs::read_to_string(temp_dir.path().join("model/lean/lean-toolchain"))?,
            "leanprover/lean4:4.29.1\n"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("emc.toml"))?.contains("version = \"0.1.0\""),
            "project manifest must carry the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("def modelVersion := \"0.1.0\""),
            "Lean project root must carry the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("def modelName := \"Repair Desk\""),
            "Lean project root must carry the project model name"
        );
        assert_project_root_digests_are_canonical_hashes(
            &fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?,
            &fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?,
        )?;
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("theorem modelVersionIsStable : modelVersion = \"0.1.0\" := rfl"),
            "Lean project root must prove the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("theorem modelIdentityIsStable : modelName = \"Repair Desk\" := rfl"),
            "Lean project root must prove project model identity"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelSliceBelongsToDeclaredWorkflow (slice : ModelSlice) : Bool := modelWorkflows.any (fun workflow => workflow == slice.workflow)"
            ),
            "Lean project root must encode workflow composition slice membership"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelWorkflowCompositionStructureComplete : (modelSlices.all modelSliceBelongsToDeclaredWorkflow && modelSlices.all modelSliceHasModule && modelSliceModules.all modelSliceModuleBelongsToDeclaredSlice && modelWorkflows.all modelWorkflowHasCompositionStructure) = true := rfl"
            ),
            "Lean project root must prove workflow composition structure completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "structure ModelOutcome where\n  workflow : String\n  slice : String\n  outcome : String\n  events : List String\n  externallyRelevant : Bool"
            ),
            "Lean project root must model outcomes as named records"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelOutcomeBranchIsModeled (outcome : ModelOutcome) : Bool := outcome.outcome.isEmpty == false && outcome.events.isEmpty == false"
            ),
            "Lean project root must check outcome branches through named fields"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "structure ModelCommandError where\n  workflow : String\n  slice : String\n  command : String\n  error : String\n  scenario : String\n  recovery : String"
            ),
            "Lean project root must type command errors as named records"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "structure ModelCommand where\n  workflow : String\n  slice : String\n  command : String"
            ),
            "Lean project root must type commands as named records"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("def modelCommands : List ModelCommand := []"),
            "Lean project root must initialize commands as named records"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "structure ModelCommandInput where\n  workflow : String\n  slice : String\n  command : String\n  input : String\n  sourceKind : String\n  sourceDescription : String\n  provenanceChain : List String\n  eventStreamSourceEvent : String\n  eventStreamSourceAttribute : String\n  externalPayloadSourceName : String\n  externalPayloadSourceField : String\n  generatedSourceName : String\n  generatedSourceField : String\n  sessionSourceName : String\n  sessionSourceField : String\n  invocationArgumentSourceName : String\n  invocationArgumentSourceField : String"
            ),
            "Lean project root must type command inputs as named records"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("def modelCommandInputs : List ModelCommandInput := []"),
            "Lean project root must initialize command inputs as named records"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelCommandErrorRecoveryIsModeled (commandError : ModelCommandError) : Bool := commandError.command.isEmpty == false && commandError.error.isEmpty == false && commandError.scenario.isEmpty == false && commandError.recovery.isEmpty == false"
            ),
            "Lean project root must check command-error recovery through named fields"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelWorkflowBehaviorSurfaceIsComplete : Bool := modelOutcomes.all modelOutcomeBranchIsModeled && modelCommandErrors.all modelCommandErrorRecoveryIsModeled && modelViewControls.all modelViewControlNavigationTargetIsModeled && modelTranslationDefinitions.all modelExternalBoundaryContractIsModeled"
            ),
            "Lean project root must aggregate workflow branch, outcome, command-error, navigation, external-boundary, and recovery modeling"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelWorkflowBehaviorSurfaceIsCompleteIsStable : modelWorkflowBehaviorSurfaceIsComplete = true := rfl"
            ),
            "Lean project root must prove the modeled workflow behavior surface is complete"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelControlProvidesCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) (input : ModelCommandInput) : Bool := control.1 == input.workflow && control.2.2.2.2.1 == input.command && control.2.2.2.2.2.1 == input.input"
            ),
            "Lean project root must be able to prove controls provide target command inputs across composed slices"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelViewControlProvidesEveryCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) : Bool := modelCommandInputs.all (fun input => input.workflow != control.1 || input.command != control.2.2.2.2.1 || modelViewControls.any (fun providedInput => providedInput.1 == control.1 && providedInput.2.1 == control.2.1 && providedInput.2.2.1 == control.2.2.1 && providedInput.2.2.2.1 == control.2.2.2.1 && providedInput.2.2.2.2.1 == control.2.2.2.2.1 && modelControlProvidesCommandInput providedInput input))"
            ),
            "Lean project root must prove each control invocation supplies every input required by its target command"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelViewControlsProvideCommandInputs : modelViewControls.all modelViewControlProvidesEveryCommandInput = true := rfl"
            ),
            "Lean project root must expose cross-slice control input completeness as a theorem"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelScenarioDefinitionHasGwt (scenario : ModelScenarioDefinition) : Bool := scenario.given.isEmpty == false && scenario.when.isEmpty == false && scenario.thenStep.isEmpty == false"
            ),
            "Lean project root must prove first-class scenario definitions include Given/When/Then"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelScenarioKindsAreFirstClass : modelScenarioDefinitions.all modelScenarioKindIsFirstClass = true := rfl"
            ),
            "Lean project root must prove scenario definitions stay in the first-class scenario sets"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelCommandInputHasProvenance (input : ModelCommandInput) : Bool := input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false"
            ),
            "Lean project root must prove command inputs carry source descriptions and provenance chains"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelCommandInputsTraceToInvocationSources : modelCommandInputs.all modelCommandInputTracesToInvocationSource = true := rfl"
            ),
            "Lean project root must prove command inputs trace to modeled invocation sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelEventAttributeSourceIsComplete (eventAttribute : String × String × String × String × String × String × String × String × String) : Bool := let (_, _, _, _, sourceKind, sourceName, sourceField, generatedSourceKind, provenance) := eventAttribute; provenance.isEmpty == false && ((sourceKind == \"command_input\" && sourceName.isEmpty == false && sourceField.isEmpty == false) || (sourceKind == \"external_payload\" && sourceName.isEmpty == false && sourceField.isEmpty == false) || (sourceKind == \"generated\" && sourceName.isEmpty == false && generatedSourceKind.isEmpty == false) || (sourceKind == \"session\" && sourceName.isEmpty == false) || (sourceKind == \"derivation\" && sourceName.isEmpty == false && sourceField.isEmpty == false))"
            ),
            "Lean project root must encode stored event fact source/provenance completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelEventAttributeSourcesAreComplete : modelEventAttributes.all modelEventAttributeSourceIsComplete = true := rfl"
            ),
            "Lean project root must prove stored event facts trace to modeled source semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelReadModelFieldSourceIsComplete (field : String × String × String × String × String × String × String × String × List String × String × String × String × String) : Bool := (field.2.2.2.2.1 == \"event_attribute\" && field.2.2.2.2.2.1.isEmpty == false && field.2.2.2.2.2.2.1.isEmpty == false) || (field.2.2.2.2.1 == \"derivation\" && field.2.2.2.2.2.2.2.1.isEmpty == false && field.2.2.2.2.2.2.2.2.1.isEmpty == false) || (field.2.2.2.2.1 == \"absence_default\" && field.2.2.2.2.2.2.2.2.2.1.isEmpty == false)"
            ),
            "Lean project root must encode read-model field source completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelReadModelFieldSourcesAreComplete : modelReadModelFields.all modelReadModelFieldSourceIsComplete = true := rfl"
            ),
            "Lean project root must prove read-model field source completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelViewFieldSourceIsComplete (field : String × String × String × String × String × String × String × String × String) : Bool := let (_, _, _, _, sourceKind, sourceReadModel, sourceField, provenance, bitEncoding) := field; sourceKind == \"read_model\" && sourceReadModel.isEmpty == false && sourceField.isEmpty == false && provenance.isEmpty == false && bitEncoding.isEmpty == false"
            ),
            "Lean project root must encode displayed datum source/provenance/bit completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelViewFieldSourcesAreComplete : modelViewFields.all modelViewFieldSourceIsComplete = true := rfl"
            ),
            "Lean project root must prove displayed datum source/provenance/bit completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelViewFieldReadModelFieldSourceResolves (viewField : String × String × String × String × String × String × String × String × String) : Bool := let (workflow, slice, _, _, _, sourceReadModel, sourceField, _, _) := viewField; modelViewFieldSourceIsComplete viewField && modelReadModelFields.any (fun readModelField => readModelField.1 == workflow && readModelField.2.1 == slice && readModelField.2.2.1 == sourceReadModel && readModelField.2.2.2.1 == sourceField && modelReadModelFieldSourceIsComplete readModelField)"
            ),
            "Lean project root must resolve displayed data through declared read-model fields"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDisplayedDatumTracesToOriginalProvenance (viewField : String × String × String × String × String × String × String × String × String) : Bool := let (workflow, slice, _, _, _, sourceReadModel, sourceField, _, _) := viewField; modelViewFieldReadModelFieldSourceResolves viewField && modelReadModelFields.any (fun readModelField => readModelField.1 == workflow && readModelField.2.1 == slice && readModelField.2.2.1 == sourceReadModel && readModelField.2.2.2.1 == sourceField && modelReadModelFieldTracesToOriginalProvenance readModelField)"
            ),
            "Lean project root must trace displayed data through read-model fields to original provenance"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDisplayedDataTraceToOriginalProvenance : modelViewFields.all modelDisplayedDatumTracesToOriginalProvenance = true := rfl"
            ),
            "Lean project root must prove displayed data traces to original provenance"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelExternalPayloadFieldHasProvenance (field : String × String × String × String × String × String) : Bool := let (_, _, _, _, provenance, bitEncoding) := field; provenance.isEmpty == false && bitEncoding.isEmpty == false"
            ),
            "Lean project root must encode external payload field provenance and bit semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelExternalPayloadFieldsHaveProvenance : modelExternalPayloadFields.all modelExternalPayloadFieldHasProvenance = true := rfl"
            ),
            "Lean project root must prove external payload field provenance and bit semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "structure ModelDataFlow where\n  workflow : String\n  slice : String\n  datum : String\n  sourceKind : String\n  source : String\n  transformation : String\n  target : String\n  bitEncoding : String"
            ),
            "Lean project root must model data flows as named records"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowCoversDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) : Bool :="
            ),
            "Lean project root must define target-aware datum-to-data-flow coverage in the formal artifact"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "dataFlow.workflow == workflow && dataFlow.slice == slice && dataFlow.datum == datum && dataFlow.target == target && modelDataFlowIsBitComplete dataFlow"
            ),
            "Lean project root must require the matching data-flow coverage row to carry complete source, transformation, target, and bit encoding semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowBitEncodingMatchesDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) (bitEncoding : String) : Bool := modelDataFlows.any (fun dataFlow => dataFlow.workflow == workflow && dataFlow.slice == slice && dataFlow.datum == datum && dataFlow.target == target && dataFlow.bitEncoding == bitEncoding && modelDataFlowIsBitComplete dataFlow)"
            ),
            "Lean project root must define datum-to-data-flow bit encoding consistency"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelViewFieldBitEncodingMatchesDataFlow (field : String × String × String × String × String × String × String × String × String) : Bool := let (workflow, slice, targetView, datum, _, _, _, _, bitEncoding) := field; modelDataFlowBitEncodingMatchesDatumTarget workflow slice datum targetView bitEncoding"
            ),
            "Lean project root must compare displayed datum bit semantics with its data-flow row"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelExternalPayloadFieldBitEncodingMatchesDataFlow (field : String × String × String × String × String × String) : Bool := let (workflow, slice, targetPayload, datum, _, bitEncoding) := field; modelDataFlowBitEncodingMatchesDatumTarget workflow slice datum targetPayload bitEncoding"
            ),
            "Lean project root must compare external payload field bit semantics with its data-flow row"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowSourceBitEncodingMatchesModeledSource (dataFlow : ModelDataFlow) : Bool := (modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source) == false) || modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && sourceFlow.bitEncoding == dataFlow.bitEncoding && modelDataFlowIsBitComplete sourceFlow)"
            ),
            "Lean project root must compare modeled data-flow source bit semantics with the source data-flow row"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowHasModeledTransformationSemantics (dataFlow : ModelDataFlow) : Bool := dataFlow.transformation == \"identity\" || dataFlow.transformation == \"projection\" || dataFlow.transformation == \"derivation\" || dataFlow.transformation == \"default\" || dataFlow.transformation == \"absence\" || dataFlow.transformation == \"transformation\""
            ),
            "Lean project root must classify data-flow transformation semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowHasModeledSourceKind (dataFlow : ModelDataFlow) : Bool := (dataFlow.sourceKind == \"original\" && dataFlow.source.isEmpty == false) || (dataFlow.sourceKind == \"modeled_target\" && dataFlow.source.isEmpty == false)"
            ),
            "Lean project root must classify data-flow source semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowModeledSourceResolves (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind != \"modeled_target\" || modelDataFlows.any (fun sourceFlow => sourceFlow.workflow == dataFlow.workflow && sourceFlow.slice == dataFlow.slice && sourceFlow.datum == dataFlow.datum && sourceFlow.target == dataFlow.source && modelDataFlowIsBitComplete sourceFlow)"
            ),
            "Lean project root must resolve modeled-target data-flow sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowsReachableFromOriginalsAfterFuel : Nat -> List ModelDataFlow -> List ModelDataFlow"
            ),
            "Lean project root must compute finite data-flow source-chain reachability from original sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowHasOriginalSourceChain (dataFlow : ModelDataFlow) : Bool := dataFlow.sourceKind == \"original\" || modelDataFlowsReachableFromOriginals.any (fun reachableFlow => modelSameDataFlowTarget reachableFlow dataFlow)"
            ),
            "Lean project root must require modeled-target data-flow chains to terminate at reachable original sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelViewFieldBitEncodingsMatchDataFlows : modelViewFields.all modelViewFieldBitEncodingMatchesDataFlow = true := rfl"
            ),
            "Lean project root must prove displayed datum bit encodings match data-flow rows"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelExternalPayloadFieldBitEncodingsMatchDataFlows : modelExternalPayloadFields.all modelExternalPayloadFieldBitEncodingMatchesDataFlow = true := rfl"
            ),
            "Lean project root must prove external payload bit encodings match data-flow rows"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDataFlowSourceBitEncodingsMatchModeledSources : modelDataFlows.all modelDataFlowSourceBitEncodingMatchesModeledSource = true := rfl"
            ),
            "Lean project root must prove modeled data-flow sources preserve bit encodings"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDataFlowTransformationsAreModeled : modelDataFlows.all modelDataFlowHasModeledTransformationSemantics = true := rfl"
            ),
            "Lean project root must prove every data-flow transformation uses modeled semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDataFlowSourceKindsAreModeled : modelDataFlows.all modelDataFlowHasModeledSourceKind = true := rfl"
            ),
            "Lean project root must prove every data-flow source has modeled source semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDataFlowModeledSourcesResolve : modelDataFlows.all modelDataFlowModeledSourceResolves = true := rfl"
            ),
            "Lean project root must prove modeled-target data-flow sources resolve"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDataFlowSourceChainsReachOriginals : modelDataFlows.all modelDataFlowHasOriginalSourceChain = true := rfl"
            ),
            "Lean project root must prove modeled-target data-flow source chains reach original sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelMeaningfulDataFlowsAreCovered : modelMeaningfulDataHasModeledDataFlows = true := rfl"
            ),
            "Lean project root must prove every meaningful datum has a modeled bit-level data flow"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelVersion = \"0.1.0\""),
            "Quint project root must carry the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelName = \"Repair Desk\""),
            "Quint project root must carry the project model name"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "type ModelDataFlow = { workflow: str, slice: str, datum: str, sourceKind: str, source: str, transformation: str, target: str, bitEncoding: str }"
            ),
            "Quint project root must make data-flow source kind part of the formal record"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelDataFlowCount = 0"),
            "Quint project root must expose a concrete data-flow count for finite reachability fuel"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelVersionStable = modelVersion == \"0.1.0\""),
            "Quint project root must expose the formal model version check"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelIdentityStable = modelName == \"Repair Desk\""),
            "Quint project root must expose the project model identity check"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelSliceBelongsToDeclaredWorkflow(modelSlice) = modelWorkflows.select(workflow => workflow == modelSlice.workflow).length() > 0"
            ),
            "Quint project root must encode workflow composition slice membership"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelWorkflowCompositionStructureComplete = modelSlices.select(modelSlice => modelSliceBelongsToDeclaredWorkflow(modelSlice)).length() == modelSlices.length() and modelSlices.select(modelSlice => modelSliceHasModule(modelSlice)).length() == modelSlices.length() and modelSliceModules.select(sliceModule => modelSliceModuleBelongsToDeclaredSlice(sliceModule)).length() == modelSliceModules.length() and modelWorkflows.select(workflow => modelWorkflowHasCompositionStructure(workflow)).length() == modelWorkflows.length()"
            ),
            "Quint project root must expose workflow composition structure completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelWorkflowBehaviorSurfaceIsComplete = modelOutcomes.select(outcome => modelOutcomeBranchIsModeled(outcome)).length() == modelOutcomes.length() and modelCommandErrors.select(commandError => modelCommandErrorRecoveryIsModeled(commandError)).length() == modelCommandErrors.length() and modelViewControls.select(control => modelViewControlNavigationTargetIsModeled(control)).length() == modelViewControls.length() and modelTranslationDefinitions.select(translation => modelExternalBoundaryContractIsModeled(translation)).length() == modelTranslationDefinitions.length()"
            ),
            "Quint project root must expose modeled workflow behavior surface completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelControlProvidesCommandInput(control, input) = control.workflow == input.workflow and control.command == input.command and control.input == input.input"
            ),
            "Quint project root must be able to verify controls provide target command inputs across composed slices"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelViewControlProvidesEveryCommandInput(control) = modelCommandInputs.select(input => input.workflow != control.workflow or input.command != control.command or modelViewControls.select(providedInput => providedInput.workflow == control.workflow and providedInput.slice == control.slice and providedInput.view == control.view and providedInput.control == control.control and providedInput.command == control.command and modelControlProvidesCommandInput(providedInput, input)).length() > 0).length() == modelCommandInputs.length()"
            ),
            "Quint project root must verify each control invocation supplies every input required by its target command"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelViewControlsProvideCommandInputs = modelViewControls.select(control => modelViewControlProvidesEveryCommandInput(control)).length() == modelViewControls.length()"
            ),
            "Quint project root must expose cross-slice control input completeness as an invariant"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelScenarioDefinitionsHaveGwt = modelScenarioDefinitions.select(scenario => modelScenarioDefinitionHasGwt(scenario)).length() == modelScenarioDefinitions.length()"
            ),
            "Quint project root must expose first-class scenario GWT completeness as an invariant"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelScenarioKindsAreFirstClass = modelScenarioDefinitions.select(scenario => modelScenarioKindIsFirstClass(scenario)).length() == modelScenarioDefinitions.length()"
            ),
            "Quint project root must expose first-class scenario kind membership as an invariant"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelCommandInputHasProvenance(input) = input.sourceDescription != \"\" and input.provenanceChain.length() > 0"
            ),
            "Quint project root must expose command input provenance completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelCommandInputsTraceToInvocationSources = modelCommandInputs.select(input => modelCommandInputTracesToInvocationSource(input)).length() == modelCommandInputs.length()"
            ),
            "Quint project root must expose command input invocation-source tracing"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelEventAttributeSourceIsComplete(eventAttr) = eventAttr.provenance != \"\" and ((eventAttr.sourceKind == \"command_input\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\") or (eventAttr.sourceKind == \"external_payload\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\") or (eventAttr.sourceKind == \"generated\" and eventAttr.sourceName != \"\" and eventAttr.generatedSourceKind != \"\") or (eventAttr.sourceKind == \"session\" and eventAttr.sourceName != \"\") or (eventAttr.sourceKind == \"derivation\" and eventAttr.sourceName != \"\" and eventAttr.sourceField != \"\"))"
            ),
            "Quint project root must encode stored event fact source/provenance completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelEventAttributeSourcesAreComplete = modelEventAttributes.select(eventAttr => modelEventAttributeSourceIsComplete(eventAttr)).length() == modelEventAttributes.length()"
            ),
            "Quint project root must expose stored event fact source/provenance completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelReadModelFieldSourceIsComplete(readModelField) = (readModelField.sourceKind == \"event_attribute\" and readModelField.sourceEvent != \"\" and readModelField.sourceAttribute != \"\") or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\" and readModelField.derivationSourceFields.length() > 0) or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\")"
            ),
            "Quint project root must encode read-model field source completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelReadModelFieldSourcesAreComplete = modelReadModelFields.select(readModelField => modelReadModelFieldSourceIsComplete(readModelField)).length() == modelReadModelFields.length()"
            ),
            "Quint project root must expose read-model field source completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelViewFieldSourceIsComplete(viewField) = viewField.sourceKind == \"read_model\" and viewField.sourceReadModel != \"\" and viewField.sourceField != \"\" and viewField.provenance != \"\" and viewField.bitEncoding != \"\""
            ),
            "Quint project root must encode displayed datum source/provenance/bit completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelViewFieldSourcesAreComplete = modelViewFields.select(viewField => modelViewFieldSourceIsComplete(viewField)).length() == modelViewFields.length()"
            ),
            "Quint project root must expose displayed datum source/provenance/bit completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelViewFieldReadModelFieldSourceResolves(viewField) = modelViewFieldSourceIsComplete(viewField) and modelReadModelFields.select(readModelField => readModelField.workflow == viewField.workflow and readModelField.slice == viewField.slice and readModelField.readModel == viewField.sourceReadModel and readModelField.field == viewField.sourceField and modelReadModelFieldSourceIsComplete(readModelField)).length() > 0"
            ),
            "Quint project root must resolve displayed data through declared read-model fields"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDisplayedDatumTracesToOriginalProvenance(viewField) = modelViewFieldReadModelFieldSourceResolves(viewField) and modelReadModelFields.select(readModelField => readModelField.workflow == viewField.workflow and readModelField.slice == viewField.slice and readModelField.readModel == viewField.sourceReadModel and readModelField.field == viewField.sourceField and modelReadModelFieldTracesToOriginalProvenance(readModelField)).length() > 0"
            ),
            "Quint project root must trace displayed data through read-model fields to original provenance"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDisplayedDataTraceToOriginalProvenance = modelViewFields.select(viewField => modelDisplayedDatumTracesToOriginalProvenance(viewField)).length() == modelViewFields.length()"
            ),
            "Quint project root must verify displayed data traces to original provenance"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelExternalPayloadFieldHasProvenance(externalPayloadField) = externalPayloadField.provenance != \"\" and externalPayloadField.bitEncoding != \"\""
            ),
            "Quint project root must encode external payload field provenance and bit semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelExternalPayloadFieldsHaveProvenance = modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldHasProvenance(externalPayloadField)).length() == modelExternalPayloadFields.length()"
            ),
            "Quint project root must expose external payload field provenance and bit semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowCoversDatumTarget(workflow, sliceName, datum, target) ="
            ),
            "Quint project root must define target-aware datum-to-data-flow coverage in the formal artifact"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "dataFlow.workflow == workflow and dataFlow.slice == sliceName and dataFlow.datum == datum and dataFlow.target == target and modelDataFlowIsBitComplete(dataFlow)"
            ),
            "Quint project root must require the matching data-flow coverage row to carry complete source, transformation, target, and bit encoding semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowBitEncodingMatchesDatumTarget(workflow, sliceName, datum, target, bitEncoding) = modelDataFlows.select(dataFlow => dataFlow.workflow == workflow and dataFlow.slice == sliceName and dataFlow.datum == datum and dataFlow.target == target and dataFlow.bitEncoding == bitEncoding and modelDataFlowIsBitComplete(dataFlow)).length() > 0"
            ),
            "Quint project root must define datum-to-data-flow bit encoding consistency"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelViewFieldBitEncodingMatchesDataFlow(viewField) = modelDataFlowBitEncodingMatchesDatumTarget(viewField.workflow, viewField.slice, viewField.field, viewField.view, viewField.bitEncoding)"
            ),
            "Quint project root must compare displayed datum bit semantics with its data-flow row"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelExternalPayloadFieldBitEncodingMatchesDataFlow(externalPayloadField) = modelDataFlowBitEncodingMatchesDatumTarget(externalPayloadField.workflow, externalPayloadField.slice, externalPayloadField.field, externalPayloadField.externalPayload, externalPayloadField.bitEncoding)"
            ),
            "Quint project root must compare external payload field bit semantics with its data-flow row"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowSourceBitEncodingMatchesModeledSource(dataFlow) = modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source).length() == 0 or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and sourceFlow.bitEncoding == dataFlow.bitEncoding and modelDataFlowIsBitComplete(sourceFlow)).length() > 0"
            ),
            "Quint project root must compare modeled data-flow source bit semantics with the source data-flow row"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowHasModeledTransformationSemantics(dataFlow) = dataFlow.transformation == \"identity\" or dataFlow.transformation == \"projection\" or dataFlow.transformation == \"derivation\" or dataFlow.transformation == \"default\" or dataFlow.transformation == \"absence\" or dataFlow.transformation == \"transformation\""
            ),
            "Quint project root must classify data-flow transformation semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowHasModeledSourceKind(dataFlow) = (dataFlow.sourceKind == \"original\" and dataFlow.source != \"\") or (dataFlow.sourceKind == \"modeled_target\" and dataFlow.source != \"\")"
            ),
            "Quint project root must classify data-flow source semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowModeledSourceResolves(dataFlow) = dataFlow.sourceKind != \"modeled_target\" or modelDataFlows.select(sourceFlow => sourceFlow.workflow == dataFlow.workflow and sourceFlow.slice == dataFlow.slice and sourceFlow.datum == dataFlow.datum and sourceFlow.target == dataFlow.source and modelDataFlowIsBitComplete(sourceFlow)).length() > 0"
            ),
            "Quint project root must resolve modeled-target data-flow sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowsReachableFromOriginalsAfterFuel(fuel, reachable) = range(0, fuel).foldl(reachable, (currentReachable, _) => currentReachable.concat(modelDataFlowTargetsFromReachable(currentReachable)))"
            ),
            "Quint project root must compute finite data-flow source-chain reachability from original sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelDataFlowHasOriginalSourceChain(dataFlow) = dataFlow.sourceKind == \"original\" or modelDataFlowsReachableFromOriginals.select(reachableFlow => modelSameDataFlowTarget(reachableFlow, dataFlow)).length() > 0"
            ),
            "Quint project root must require modeled-target data-flow chains to terminate at reachable original sources"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelViewFieldBitEncodingsMatchDataFlows = modelViewFields.select(viewField => modelViewFieldBitEncodingMatchesDataFlow(viewField)).length() == modelViewFields.length()"
            ),
            "Quint project root must verify displayed datum bit encodings match data-flow rows"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelExternalPayloadFieldBitEncodingsMatchDataFlows = modelExternalPayloadFields.select(externalPayloadField => modelExternalPayloadFieldBitEncodingMatchesDataFlow(externalPayloadField)).length() == modelExternalPayloadFields.length()"
            ),
            "Quint project root must verify external payload bit encodings match data-flow rows"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDataFlowSourceBitEncodingsMatchModeledSources = modelDataFlows.select(dataFlow => modelDataFlowSourceBitEncodingMatchesModeledSource(dataFlow)).length() == modelDataFlows.length()"
            ),
            "Quint project root must verify modeled data-flow sources preserve bit encodings"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDataFlowTransformationsAreModeled = modelDataFlows.select(dataFlow => modelDataFlowHasModeledTransformationSemantics(dataFlow)).length() == modelDataFlows.length()"
            ),
            "Quint project root must verify every data-flow transformation uses modeled semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDataFlowSourceKindsAreModeled = modelDataFlows.select(dataFlow => modelDataFlowHasModeledSourceKind(dataFlow)).length() == modelDataFlows.length()"
            ),
            "Quint project root must verify every data-flow source has modeled source semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDataFlowModeledSourcesResolve = modelDataFlows.select(dataFlow => modelDataFlowModeledSourceResolves(dataFlow)).length() == modelDataFlows.length()"
            ),
            "Quint project root must verify modeled-target data-flow sources resolve"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDataFlowSourceChainsReachOriginals = modelDataFlows.select(dataFlow => modelDataFlowHasOriginalSourceChain(dataFlow)).length() == modelDataFlows.length()"
            ),
            "Quint project root must expose data-flow source-chain completeness"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDataFlowSourceChainsPreserveBitEncodingSemantics : modelDataFlows.all modelDataFlowHasBitPreservingOriginalSourceChain = true := rfl"
            ),
            "Lean project root must prove source chains preserve bit encoding semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDataFlowSourceChainsPreserveBitEncodingSemantics = modelDataFlows.select(dataFlow => modelDataFlowHasBitPreservingOriginalSourceChain(dataFlow)).length() == modelDataFlows.length()"
            ),
            "Quint project root must expose bit-preserving source-chain semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelMeaningfulDataFlowsAreCovered = modelMeaningfulDataHasModeledDataFlows"
            ),
            "Quint project root must expose every meaningful datum coverage as an invariant"
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("model/lean/lakefile.lean"))?,
            "import Lake\nopen Lake DSL\npackage EMCModel where\n"
        );
        Ok(())
    }

    #[test]
    fn init_repairs_generated_project_manifest_from_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let manifest_path = temp_dir.path().join("emc.toml");
        let user_manifest = "[project]\nname = \"User Edited\"\n";

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::write(&manifest_path, user_manifest)?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let actual_manifest = fs::read_to_string(&manifest_path)?;
        assert!(
            actual_manifest.contains("name = \"Repair Desk\""),
            "re-running init must repair generated manifest drift from exported events"
        );
        assert!(
            actual_manifest.contains("version = \"0.1.0\""),
            "re-running init must restore the generated manifest version"
        );

        Ok(())
    }

    #[test]
    fn init_requires_exact_name_flag() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--wrong-name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc init --name <project-name>",
            ));

        assert!(
            !temp_dir.path().join("emc.toml").exists(),
            "unsupported init syntax must not create project files"
        );

        Ok(())
    }

    fn assert_project_root_digests_are_canonical_hashes(
        lean_root: &str,
        quint_root: &str,
    ) -> Result<(), Box<dyn Error>> {
        let lean_digest = generated_model_digest(lean_root, "def modelDigest := \"")?;
        let quint_digest = generated_model_digest(quint_root, "val modelDigest = \"")?;

        assert_eq!(
            lean_digest, quint_digest,
            "Lean and Quint project roots must use the same generated model digest"
        );
        assert!(
            is_lowercase_sha256_hex(lean_digest),
            "project root digest must be a canonical SHA-256 content hash"
        );
        assert!(
            lean_root.contains(&format!(
                "theorem modelDigestIsStable : modelDigest = \"{lean_digest}\" := rfl"
            )),
            "Lean project root must prove project model digest stability"
        );
        assert!(
            quint_root.contains(&format!(
                "val modelDigestStable = modelDigest == \"{quint_digest}\""
            )),
            "Quint project root must expose the project model digest invariant"
        );

        Ok(())
    }

    fn generated_model_digest<'a>(
        artifact: &'a str,
        prefix: &str,
    ) -> Result<&'a str, Box<dyn Error>> {
        let start = artifact
            .find(prefix)
            .ok_or_else(|| format!("generated artifact must contain {prefix}"))?
            + prefix.len();
        let tail = &artifact[start..];
        let end = tail
            .find('"')
            .ok_or("generated artifact model digest must terminate with a quote")?;

        Ok(&tail[..end])
    }

    fn is_lowercase_sha256_hex(value: &str) -> bool {
        value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
    }
}
