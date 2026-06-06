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
            "model/quint/quint.json",
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
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDigest := \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\""
            ),
            "Lean project root must carry a deterministic project model digest"
        );
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
                "theorem modelDigestIsStable : modelDigest = \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\" := rfl"
            ),
            "Lean project root must prove project model digest stability"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelControlProvidesCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) (input : String × String × String × String × String × String × List String × String × String × String × String × String × String × String × String × String × String) : Bool := control.1 == input.1 && control.2.2.2.2.1 == input.2.2.1 && control.2.2.2.2.2.1 == input.2.2.2.1"
            ),
            "Lean project root must be able to prove controls provide target command inputs across composed slices"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelViewControlProvidesEveryCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) : Bool := modelCommandInputs.all (fun input => input.1 != control.1 || input.2.2.1 != control.2.2.2.2.1 || modelViewControls.any (fun providedInput => providedInput.1 == control.1 && providedInput.2.1 == control.2.1 && providedInput.2.2.1 == control.2.2.1 && providedInput.2.2.2.1 == control.2.2.2.1 && providedInput.2.2.2.2.1 == control.2.2.2.2.1 && modelControlProvidesCommandInput providedInput input))"
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
                "def modelScenarioDefinitionHasGwt (scenario : String × String × String × String × String × String × String × List String × List String × String × String × List String) : Bool := scenario.2.2.2.2.1.isEmpty == false && scenario.2.2.2.2.2.1.isEmpty == false && scenario.2.2.2.2.2.2.1.isEmpty == false"
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
                "def modelCommandInputHasProvenance (input : String × String × String × String × String × String × List String × String × String × String × String × String × String × String × String × String × String) : Bool := let (_, _, _, _, _, sourceDescription, provenanceChain, _, _, _, _, _, _, _, _, _, _) := input; sourceDescription.isEmpty == false && provenanceChain.isEmpty == false"
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
                "def modelDataFlowCoversDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) : Bool :="
            ),
            "Lean project root must define target-aware datum-to-data-flow coverage in the formal artifact"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "flowWorkflow == workflow && flowSlice == slice && flowDatum == datum && flowTarget == target && modelDataFlowIsBitComplete dataFlow"
            ),
            "Lean project root must require the matching data-flow coverage row to carry complete source, transformation, target, and bit encoding semantics"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDataFlowBitEncodingMatchesDatumTarget (workflow : String) (slice : String) (datum : String) (target : String) (bitEncoding : String) : Bool := modelDataFlows.any (fun dataFlow => let (flowWorkflow, flowSlice, flowDatum, _, _, flowTarget, flowBitEncoding) := dataFlow; flowWorkflow == workflow && flowSlice == slice && flowDatum == datum && flowTarget == target && flowBitEncoding == bitEncoding && modelDataFlowIsBitComplete dataFlow)"
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
                "val modelDigest = \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\""
            ),
            "Quint project root must carry a deterministic project model digest"
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
                "val modelDigestStable = modelDigest == \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\""
            ),
            "Quint project root must expose the project model digest invariant"
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
                "val modelMeaningfulDataFlowsAreCovered = modelMeaningfulDataHasModeledDataFlows"
            ),
            "Quint project root must expose every meaningful datum coverage as an invariant"
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("model/lean/lakefile.lean"))?,
            "import Lake\nopen Lake DSL\npackage EMCModel where\n"
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("model/quint/quint.json"))?,
            "{\n  \"main\": \"RepairDesk.qnt\",\n  \"invariants\": [\n    \"workflowIdentityStable\",\n    \"workflowSliceDetailsComplete\",\n    \"workflowSliceModulesComplete\",\n    \"workflowTransitionsStructured\",\n    \"workflowTransitionSourcesResolve\",\n    \"workflowTransitionTargetsResolve\",\n    \"workflowStepRelationshipsAreAllowed\",\n    \"workflowStepSlugsAreUnique\",\n    \"workflowHasExactlyOneEntryStep\",\n    \"workflowMainStepsHaveIncomingReachability\",\n    \"workflowNonSupportingStepsReachableFromEntry\",\n    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\",\n    \"workflowTransitionsHaveModeledKinds\",\n    \"workflowExitsNameTargetsAndRationale\",\n    \"workflowExternallyRelevantOutcomesHandled\",\n    \"workflowOutcomesSourceResolve\",\n    \"workflowCommandErrorsSourceResolve\",\n    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\",\n    \"workflowNonEventDefinitionsAreUniquelyOwned\",\n    \"workflowSharedEventDefinitionsHaveIdenticalIdentity\",\n    \"workflowCommandTransitionsResolveControlsAndCommands\",\n    \"workflowStateViewCommandTransitionsTargetStateChanges\",\n    \"workflowEventTransitionsAreSharedByEndpointSlices\",\n    \"workflowEventTransitionsHaveParticipatingEndpointEvents\",\n    \"workflowNavigationTransitionsResolveControlsAndViews\",\n    \"workflowNavigationTransitionsResolveToEntryViews\",\n    \"workflowExternalTriggersDeclarePayloadContracts\",\n    \"workflowExternalTriggerPayloadContractsHaveProvenance\",\n    \"workflowTransitionsHaveRequiredEvidence\",\n    \"workflowEntryLifecycleStatesCoverRequiredStates\",\n    \"modelScenarioDefinitionsHaveGwt\",\n    \"modelScenarioKindsAreFirstClass\",\n    \"modelDataFlowsAreBitComplete\",\n    \"modelMeaningfulDataFlowsAreCovered\",\n    \"modelViewFieldBitEncodingsMatchDataFlows\",\n    \"modelExternalPayloadFieldBitEncodingsMatchDataFlows\",\n    \"modelCommandInputsHaveProvenance\",\n    \"modelCommandInputsTraceToInvocationSources\",\n    \"modelEventAttributeSourcesAreComplete\",\n    \"modelReadModelFieldSourcesAreComplete\",\n    \"modelViewFieldSourcesAreComplete\",\n    \"modelViewFieldReadModelFieldSourcesResolve\",\n    \"modelDisplayedDataTraceToOriginalProvenance\",\n    \"modelExternalPayloadFieldsHaveProvenance\",\n    \"modelViewControlsProvideCommandInputs\"\n  ]\n}\n"
        );
        Ok(())
    }

    #[test]
    fn init_does_not_overwrite_existing_project_files() -> Result<(), Box<dyn Error>> {
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
        assert_eq!(
            actual_manifest, user_manifest,
            "re-running init must not overwrite existing project files"
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
}
