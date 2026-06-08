// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::core::connection::ConnectionKind;
    use crate::core::effect::{
        ArtifactDigest, ChosenEventId, EventConflictId, FileContents, ModelContentDigest,
        ProjectPath, ProjectionFingerprint, ReviewEventId, ReviewEventReference,
    };
    use crate::core::event_commands::{EmcEvent, SliceFactEvent, SliceFactInput};
    use crate::core::events::{
        EventDraft, EventDraftBody, EventDraftType, EventStreamId, ExportedEvent,
    };
    use crate::core::formal_slice_facts::{NewOutcomeDefinition, OutcomeEventNames};
    use crate::core::review_record::ReviewRecordDocument;
    use crate::core::types::{
        CommandErrorName, CommandName, EventName, OutcomeLabelName, PayloadContractName,
        ReviewRuleName, ReviewStatus, ReviewTimestamp, ReviewerId, SliceKindName, StreamName,
        TransitionTriggerName, WorkflowEntryLifecycleEvidenceText, WorkflowEntryLifecycleStateName,
        WorkflowEventParticipation, WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName,
        WorkflowStepRelationshipName, WorkflowTransitionKind, WorkflowTransitionSourceEvidenceText,
        WorkflowTransitionTargetEvidenceText,
    };
    use crate::core::verify::{QuintInvariantName, QuintInvariantSet};
    use crate::core::workflow::NewWorkflow;
    use crate::io::dto::{
        parse_board_connection_endpoint_kind, parse_board_element_kind, parse_board_lane_id,
        parse_command_error_recovery_kind, parse_command_input_source_kind,
        parse_contract_kind_name, parse_control_recovery_behavior, parse_data_flow_source_kind,
        parse_event_attribute_source_kind, parse_lean_module_name, parse_model_description,
        parse_model_digest, parse_model_name, parse_navigation_target_type, parse_project_name,
        parse_quint_module_name, parse_read_model_field_source_kind, parse_review_timestamp,
        parse_reviewer_id, parse_scenario_kind, parse_singleton_repeat_behavior, parse_slice_slug,
        parse_transformation_semantics, parse_transition_trigger_name,
        parse_view_field_source_kind, parse_workflow_entry_lifecycle_state_name,
        parse_workflow_event_participation, parse_workflow_owned_definition_kind,
        parse_workflow_slug, parse_workflow_transition_endpoint, parse_workflow_transition_kind,
        parse_workflow_view_role,
    };

    #[test]
    fn boundary_parsers_convert_raw_strings_to_semantic_types() -> Result<(), Box<dyn Error>> {
        let project_name = parse_project_name(" Repair Desk ")?;
        let model_name = parse_model_name(" Repair Desk ")?;
        let model_description = parse_model_description(" Repairs organization access ")?;
        let workflow_slug = parse_workflow_slug(" Organization Access ")?;
        let slice_slug = parse_slice_slug(" Resolve Application Entry ")?;
        let lean_module = parse_lean_module_name(" RepairDesk ")?;
        let quint_module = parse_quint_module_name(" RepairDesk ")?;
        let digest = parse_model_digest(" abc123 ")?;
        let transition_trigger = parse_transition_trigger_name(" Open request ")?;
        let transition_endpoint = parse_workflow_transition_endpoint(" capture-ticket ")?;
        let transition_kind = parse_workflow_transition_kind(" navigation ")?;
        let reviewer = parse_reviewer_id(" event-model-reviewer ")?;
        let reviewed_at = parse_review_timestamp(" 2026-06-03T00:00:00.000Z ")?;

        assert_eq!(
            project_name.as_ref(),
            "Repair Desk",
            "project name is trimmed"
        );
        assert_eq!(model_name.as_ref(), "Repair Desk", "model name is trimmed");
        assert_eq!(
            model_description.as_ref(),
            "Repairs organization access",
            "model description is trimmed"
        );
        assert_eq!(
            workflow_slug.as_ref(),
            "organization-access",
            "workflow slug is normalized"
        );
        assert_eq!(
            slice_slug.as_ref(),
            "resolve-application-entry",
            "slice slug is normalized"
        );
        assert_eq!(
            lean_module.as_ref(),
            "RepairDesk",
            "Lean module name is trimmed"
        );
        assert_eq!(
            quint_module.as_ref(),
            "RepairDesk",
            "Quint module name is trimmed"
        );
        assert_eq!(digest.as_ref(), "abc123", "digest is trimmed");
        assert_eq!(
            transition_trigger.as_ref(),
            "Open request",
            "transition trigger is trimmed"
        );
        assert_eq!(
            transition_endpoint.as_ref(),
            "capture-ticket",
            "workflow transition endpoint is trimmed"
        );
        assert_eq!(
            transition_kind.as_ref(),
            "navigation",
            "workflow transition kind is trimmed"
        );
        assert_eq!(
            reviewer.as_ref(),
            "event-model-reviewer",
            "reviewer id is trimmed"
        );
        assert_eq!(
            reviewed_at.as_ref(),
            "2026-06-03T00:00:00.000Z",
            "review timestamp is trimmed"
        );

        Ok(())
    }

    #[test]
    fn boundary_parsers_reject_empty_semantic_values() {
        assert!(
            parse_project_name("   ").is_err(),
            "blank project names must not enter the core"
        );
        assert!(
            parse_model_name("   ").is_err(),
            "blank model names must not enter the core"
        );
        assert!(
            parse_model_description("   ").is_err(),
            "blank model descriptions must not enter the core"
        );
        assert!(
            parse_workflow_slug("   ").is_err(),
            "blank workflow slugs must not enter the core"
        );
        assert!(
            parse_slice_slug("   ").is_err(),
            "blank slice slugs must not enter the core"
        );
        assert!(
            parse_lean_module_name("   ").is_err(),
            "blank Lean module names must not enter the core"
        );
        assert!(
            parse_quint_module_name("   ").is_err(),
            "blank Quint module names must not enter the core"
        );
        assert!(
            parse_model_digest("   ").is_err(),
            "blank model digests must not enter the core"
        );
        assert!(
            parse_transition_trigger_name("   ").is_err(),
            "blank transition triggers must not enter the core"
        );
        assert!(
            parse_workflow_transition_endpoint("   ").is_err(),
            "blank workflow transition endpoints must not enter the core"
        );
        assert!(
            parse_workflow_transition_kind("   ").is_err(),
            "blank workflow transition kinds must not enter the core"
        );
        assert!(
            parse_reviewer_id("   ").is_err(),
            "blank reviewer ids must not enter the core"
        );
        assert!(
            parse_review_timestamp("   ").is_err(),
            "blank review timestamps must not enter the core"
        );
    }

    #[test]
    fn review_timestamps_must_use_utc_millisecond_instants() {
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00.000Z").is_ok(),
            "review timestamps use UTC instants with millisecond precision"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00Z").is_err(),
            "review timestamps must include millisecond precision"
        );
        assert!(
            parse_review_timestamp("2026-06-03 00:00:00.000Z").is_err(),
            "review timestamps must use the RFC3339 separator"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00.000-07:00").is_err(),
            "review timestamps must be normalized to UTC"
        );
        assert!(
            parse_review_timestamp("202A-06-03T00:00:00.000Z").is_err(),
            "review timestamp years must be numeric"
        );
        assert!(
            parse_review_timestamp("2026-0:-03T00:00:00.000Z").is_err(),
            "review timestamp months must be numeric"
        );
        assert!(
            parse_review_timestamp("2026-19-03T00:00:00.000Z").is_err(),
            "review timestamp months must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T24:00:00.000Z").is_err(),
            "review timestamp hours must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:60:00.000Z").is_err(),
            "review timestamp minutes must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:60.000Z").is_err(),
            "review timestamp seconds must be in range"
        );
        assert!(
            parse_review_timestamp("2026-06-03T00:00:00.A00Z").is_err(),
            "review timestamp milliseconds must be numeric"
        );
        assert!(
            parse_review_timestamp("not-a-timestamp").is_err(),
            "review timestamps must not accept arbitrary text"
        );
    }

    #[test]
    fn workflow_transition_kinds_are_closed_domain_terms() {
        [
            "command",
            "event",
            "navigation",
            "external_trigger",
            "outcome",
            "workflow_exit:command",
            "workflow_exit:event",
            "workflow_exit:navigation",
            "workflow_exit:external_trigger",
            "workflow_exit:outcome",
        ]
        .into_iter()
        .for_each(|kind| {
            assert!(
                parse_workflow_transition_kind(kind).is_ok(),
                "{kind} should be accepted as a modeled transition kind"
            );
        });

        assert!(
            parse_workflow_transition_kind("screen-flow").is_err(),
            "arbitrary transition kind text must not enter the core"
        );
        assert!(
            parse_workflow_transition_kind("workflow_exit").is_err(),
            "workflow exit transition kinds must preserve the triggering transition kind"
        );
    }

    #[test]
    fn workflow_owned_definition_kinds_are_closed_domain_terms() {
        [
            "command",
            "event",
            "view",
            "control",
            "read_model",
            "outcome",
            "error",
            "automation",
            "translation",
            "external_payload",
        ]
        .into_iter()
        .for_each(|kind| {
            assert!(
                parse_workflow_owned_definition_kind(kind).is_ok(),
                "{kind} should be accepted as a modeled owned-definition kind"
            );
        });

        assert!(
            parse_workflow_owned_definition_kind("spreadsheet").is_err(),
            "arbitrary owned-definition kind text must not enter the core"
        );
    }

    #[test]
    fn workflow_owned_definition_roles_are_closed_domain_terms() {
        ["emitted", "observed"].into_iter().for_each(|role| {
            assert!(
                parse_workflow_event_participation(role).is_ok(),
                "{role} should be accepted as a modeled event participation"
            );
        });

        assert!(
            parse_workflow_event_participation("subscribed").is_err(),
            "arbitrary event participation text must not enter the core"
        );
        assert!(
            parse_workflow_view_role("entry").is_ok(),
            "entry should be accepted as a modeled workflow view role"
        );
        assert!(
            parse_workflow_view_role("summary").is_err(),
            "arbitrary view role text must not enter the core"
        );
    }

    #[test]
    fn slice_kind_names_are_closed_domain_terms() {
        ["state_view", "state_change", "translation", "automation"]
            .into_iter()
            .for_each(|kind| {
                assert!(
                    SliceKindName::try_new(kind.to_owned()).is_ok(),
                    "{kind} should be accepted as a modeled slice kind"
                );
            });

        assert!(
            SliceKindName::try_new("spreadsheet".to_owned()).is_err(),
            "arbitrary slice kind text must not enter formal workflow graphs"
        );
    }

    #[test]
    fn workflow_step_relationships_are_closed_domain_terms() {
        [
            "entry",
            "main",
            "branch",
            "alternate",
            "async_lifecycle",
            "supporting",
        ]
        .into_iter()
        .for_each(|relationship| {
            assert!(
                WorkflowStepRelationshipName::try_new(relationship.to_owned()).is_ok(),
                "{relationship} should be accepted as a modeled workflow step relationship"
            );
        });

        assert!(
            WorkflowStepRelationshipName::try_new("optional".to_owned()).is_err(),
            "arbitrary workflow step relationship text must not enter formal workflow graphs"
        );
    }

    #[test]
    fn workflow_entry_lifecycle_states_are_closed_domain_terms() {
        [
            "fresh_uninitialized",
            "initialized_unauthenticated",
            "initialized_authenticated",
            "partially_configured",
            "fully_configured",
        ]
        .into_iter()
        .for_each(|state| {
            assert!(
                parse_workflow_entry_lifecycle_state_name(state).is_ok(),
                "{state} should be accepted as a modeled workflow entry lifecycle state"
            );
        });

        assert!(
            parse_workflow_entry_lifecycle_state_name("logged_in").is_err(),
            "arbitrary workflow entry lifecycle states must not enter formal workflow graphs"
        );
    }

    #[test]
    fn command_input_source_kinds_are_closed_domain_terms() {
        [
            "actor",
            "session",
            "generated",
            "external_payload",
            "event_stream_state",
            "invocation_argument",
        ]
        .into_iter()
        .for_each(|source_kind| {
            assert!(
                parse_command_input_source_kind(source_kind).is_ok(),
                "{source_kind} should be accepted as a modeled command input source kind"
            );
        });

        assert!(
            parse_command_input_source_kind("spreadsheet").is_err(),
            "arbitrary command input source kinds must not enter command/control definitions"
        );
    }

    #[test]
    fn command_error_recovery_kinds_are_closed_domain_terms() {
        [
            "retry",
            "stay_on_screen",
            "navigation",
            "explicit_recovery_action",
        ]
        .into_iter()
        .for_each(|recovery_kind| {
            assert!(
                parse_command_error_recovery_kind(recovery_kind).is_ok(),
                "{recovery_kind} should be accepted as a modeled command error recovery kind"
            );
        });

        assert!(
            parse_command_error_recovery_kind("email_support").is_err(),
            "arbitrary command error recovery kinds must not enter command/control definitions"
        );
    }

    #[test]
    fn control_recovery_behaviors_are_closed_domain_terms() {
        [
            "retry",
            "stay_on_screen",
            "navigation",
            "explicit_recovery_action",
        ]
        .into_iter()
        .for_each(|recovery_behavior| {
            assert!(
                parse_control_recovery_behavior(recovery_behavior).is_ok(),
                "{recovery_behavior} should be accepted as a modeled control recovery behavior"
            );
        });

        assert!(
            parse_control_recovery_behavior("email_support").is_err(),
            "arbitrary recovery behavior must not enter view controls"
        );
    }

    #[test]
    fn singleton_repeat_behaviors_are_closed_domain_terms() {
        ["already_exists_error", "idempotent"]
            .into_iter()
            .for_each(|repeat_behavior| {
                assert!(
                    parse_singleton_repeat_behavior(repeat_behavior).is_ok(),
                    "{repeat_behavior} should be accepted as a modeled singleton repeat behavior"
                );
            });

        assert!(
            parse_singleton_repeat_behavior("replace_existing").is_err(),
            "arbitrary singleton repeat behavior must not enter command definitions"
        );
    }

    #[test]
    fn read_model_field_source_kinds_are_closed_domain_terms() {
        ["event_attribute", "derivation", "absence_default"]
            .into_iter()
            .for_each(|source_kind| {
                assert!(
                    parse_read_model_field_source_kind(source_kind).is_ok(),
                    "{source_kind} should be accepted as a modeled read-model field source kind"
                );
            });

        assert!(
            parse_read_model_field_source_kind("spreadsheet").is_err(),
            "arbitrary read-model field source kinds must not enter read-model definitions"
        );
    }

    #[test]
    fn view_field_source_kinds_are_closed_domain_terms() {
        assert!(
            parse_view_field_source_kind("read_model").is_ok(),
            "read_model should be accepted as a modeled view field source kind"
        );
        assert!(
            parse_view_field_source_kind("event_attribute").is_err(),
            "view fields must source from read models, not arbitrary source kinds"
        );
    }

    #[test]
    fn navigation_target_types_are_closed_domain_terms() {
        [
            "modeled_view",
            "local_view_state",
            "external_system",
            "external_workflow",
        ]
        .into_iter()
        .for_each(|target_type| {
            assert!(
                parse_navigation_target_type(target_type).is_ok(),
                "{target_type} should be accepted as a modeled navigation target type"
            );
        });

        assert!(
            parse_navigation_target_type("url").is_err(),
            "arbitrary navigation target types must not enter view control definitions"
        );
    }

    #[test]
    fn event_attribute_source_kinds_are_closed_domain_terms() {
        [
            "command_input",
            "external_payload",
            "generated",
            "session",
            "derivation",
        ]
        .into_iter()
        .for_each(|source_kind| {
            assert!(
                parse_event_attribute_source_kind(source_kind).is_ok(),
                "{source_kind} should be accepted as a modeled event attribute source kind"
            );
        });

        assert!(
            parse_event_attribute_source_kind("spreadsheet").is_err(),
            "arbitrary event attribute source kinds must not enter event definitions"
        );
    }

    #[test]
    fn board_lane_ids_are_closed_domain_terms() {
        ["ux", "actions", "events"].into_iter().for_each(|lane| {
            assert!(
                parse_board_lane_id(lane).is_ok(),
                "{lane} should be accepted as a canonical board lane"
            );
        });

        assert!(
            parse_board_lane_id("support").is_err(),
            "arbitrary board lanes must not enter board elements"
        );
    }

    #[test]
    fn board_element_kinds_are_closed_domain_terms() {
        [
            "view",
            "automation",
            "external_event",
            "command",
            "read_model",
            "event",
        ]
        .into_iter()
        .for_each(|kind| {
            assert!(
                parse_board_element_kind(kind).is_ok(),
                "{kind} should be accepted as a modeled board element kind"
            );
        });

        assert!(
            parse_board_element_kind("workflow_trigger").is_err(),
            "connection-only endpoint kinds must not enter board elements"
        );
    }

    #[test]
    fn board_connection_endpoint_kinds_are_closed_domain_terms() {
        [
            "view",
            "automation",
            "external_event",
            "workflow_trigger",
            "command",
            "event",
            "read_model",
        ]
        .into_iter()
        .for_each(|kind| {
            assert!(
                parse_board_connection_endpoint_kind(kind).is_ok(),
                "{kind} should be accepted as a modeled board connection endpoint kind"
            );
        });

        assert!(
            parse_board_connection_endpoint_kind("scheduler").is_err(),
            "arbitrary endpoint kinds must not enter board connections"
        );
    }

    #[test]
    fn data_flow_source_kinds_are_closed_domain_terms() {
        ["original", "modeled_target"]
            .into_iter()
            .for_each(|source_kind| {
                assert!(
                    parse_data_flow_source_kind(source_kind).is_ok(),
                    "{source_kind} should be accepted as a modeled data-flow source kind"
                );
            });

        assert!(
            parse_data_flow_source_kind("event_attribute").is_err(),
            "arbitrary source kinds must not enter bit-level data flows"
        );
    }

    #[test]
    fn transformation_semantics_are_closed_domain_terms() {
        [
            "identity",
            "projection",
            "derivation",
            "default",
            "absence",
            "transformation",
        ]
        .into_iter()
        .for_each(|transformation| {
            assert!(
                parse_transformation_semantics(transformation).is_ok(),
                "{transformation} should be accepted as modeled transformation semantics"
            );
        });

        assert!(
            parse_transformation_semantics("custom_mapping").is_err(),
            "arbitrary transformation semantics must not enter bit-level data flows"
        );
    }

    #[test]
    fn contract_kinds_are_closed_domain_terms() {
        [
            "projector",
            "command",
            "automation",
            "translation",
            "derivation",
            "absence",
            "transitive",
        ]
        .into_iter()
        .for_each(|contract_kind| {
            assert!(
                parse_contract_kind_name(contract_kind).is_ok(),
                "{contract_kind} should be accepted as a modeled contract kind"
            );
        });

        assert!(
            parse_contract_kind_name("integration").is_err(),
            "arbitrary contract kinds must not enter contract scenarios"
        );
    }

    #[test]
    fn scenario_kinds_are_closed_domain_terms() {
        ["acceptance", "contract"].into_iter().for_each(|kind| {
            assert!(
                parse_scenario_kind(kind).is_ok(),
                "{kind} should be accepted as a modeled scenario kind"
            );
        });

        assert!(
            parse_scenario_kind("journey").is_err(),
            "arbitrary scenario kinds must not enter slice scenarios"
        );
    }

    #[test]
    fn review_statuses_are_closed_domain_terms() {
        ["clean", "changes_requested"]
            .into_iter()
            .for_each(|status| {
                assert!(
                    ReviewStatus::try_new(status.to_owned()).is_ok(),
                    "{status} should be accepted as a modeled review status"
                );
            });

        assert!(
            ReviewStatus::try_new("approved".to_owned()).is_err(),
            "arbitrary review statuses must not enter review records"
        );
    }

    #[test]
    fn review_record_documents_reject_unmodeled_statuses_at_parse() -> Result<(), Box<dyn Error>> {
        let document = FileContents::try_new(
            r#"{
  "workflow_slug": "open-ticket",
  "model_content_digest": "emc-fnv1a64:0000000000000000",
  "reviewer_id": "event-model-reviewer",
  "status": "approved",
  "category_results": {},
  "mandatory_findings": [],
  "reviewed_at": "2026-06-03T00:00:00.000Z"
}"#
            .to_owned(),
        )?;

        assert!(
            ReviewRecordDocument::parse(&document).is_err(),
            "review records must parse status into the closed review status domain immediately"
        );

        Ok(())
    }

    #[test]
    fn review_rule_names_are_closed_domain_terms() {
        [
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
        ]
        .into_iter()
        .for_each(|category| {
            assert!(
                ReviewRuleName::try_new(category.to_owned()).is_ok(),
                "{category} should be accepted as a modeled review category"
            );
        });

        assert!(
            ReviewRuleName::try_new("manual-approval".to_owned()).is_err(),
            "arbitrary review categories must not enter review records"
        );
    }

    #[test]
    fn project_paths_are_relative_to_the_current_project() -> Result<(), Box<dyn Error>> {
        let path = ProjectPath::try_new("model/lean/OpenTicket.lean".to_owned())?;

        assert_eq!(path.as_ref(), "model/lean/OpenTicket.lean");
        assert!(
            ProjectPath::try_new("/tmp/site".to_owned()).is_err(),
            "absolute paths must not enter project-local effects"
        );
        assert!(
            ProjectPath::try_new("../outside-model".to_owned()).is_err(),
            "parent traversal must not enter project-local effects"
        );

        Ok(())
    }

    #[test]
    fn exported_event_types_are_closed_domain_terms() {
        [
            "ProjectInitialized",
            "WorkflowAdded",
            "WorkflowUpdated",
            "WorkflowRemoved",
            "WorkflowOutcomeAdded",
            "WorkflowCommandErrorAdded",
            "WorkflowOwnedDefinitionAdded",
            "WorkflowTransitionEvidenceAdded",
            "WorkflowEntryLifecycleCoverageRequired",
            "WorkflowEntryLifecycleStateAdded",
            "WorkflowReadinessDeclared",
            "WorkflowConnected",
            "WorkflowTransitionRemoved",
            "SliceAdded",
            "SliceUpdated",
            "SliceRemoved",
            "SliceScenarioAdded",
            "SliceOutcomeAdded",
            "SliceExternalPayloadAdded",
            "SliceEventDefinitionAdded",
            "SliceCommandDefinitionAdded",
            "SliceReadModelAdded",
            "SliceViewAdded",
            "SliceBitLevelDataFlowAdded",
            "SliceTranslationAdded",
            "SliceAutomationAdded",
            "SliceBoardElementAdded",
            "SliceBoardConnectionAdded",
            "ReviewRecorded",
            "ConflictResolved",
        ]
        .into_iter()
        .for_each(|event_type| {
            assert!(
                EventDraftType::try_new(event_type.to_owned()).is_ok(),
                "{event_type} should be accepted as a modeled exported event type"
            );
        });

        assert!(
            EventDraftType::try_new("SliceFactAdded".to_owned()).is_err(),
            "internal eventcore aggregate events must not be exported event draft types"
        );
        assert!(
            EventDraftType::try_new("WorkflowRenamed".to_owned()).is_err(),
            "arbitrary event type strings must not enter exported event drafts"
        );
    }

    #[test]
    fn exported_event_stream_ids_are_modeled_domain_terms() -> Result<(), Box<dyn Error>> {
        [
            ("project", "project"),
            ("workflow::open-ticket", "workflow::open-ticket"),
            ("slice::capture-ticket", "slice::capture-ticket"),
            ("review::open-ticket", "review::open-ticket"),
        ]
        .into_iter()
        .try_for_each(|(raw, expected)| -> Result<(), Box<dyn Error>> {
            let stream_id = EventStreamId::try_new(raw.to_owned())?;

            assert_eq!(stream_id.to_string(), expected);

            Ok(())
        })?;

        [
            "",
            "artifact::open-ticket",
            "workflow::",
            "slice::",
            "review::",
            "workflow::open::ticket",
        ]
        .into_iter()
        .for_each(|raw| {
            assert!(
                EventStreamId::try_new(raw.to_owned()).is_err(),
                "{raw:?} must not enter exported event drafts as a stream id"
            );
        });

        Ok(())
    }

    #[test]
    fn exported_events_preserve_semantic_bodies_until_json_boundaries() -> Result<(), Box<dyn Error>>
    {
        let workflow = NewWorkflow::new(
            parse_model_name("Open ticket")?,
            parse_model_description("Actor opens a repair ticket.")?,
            parse_workflow_slug("open-ticket")?,
        );

        let draft = EventDraft::workflow_added(&workflow);

        assert_eq!(draft.event_type(), EventDraftType::WorkflowAdded);
        assert_eq!(
            draft.body(),
            &EventDraftBody::WorkflowAdded {
                workflow: workflow.clone()
            },
            "exported event drafts should carry semantic data, not raw JSON payloads"
        );

        let exported = ExportedEvent::from_draft_for_test(&draft)?;

        assert_eq!(exported.event_type(), EventDraftType::WorkflowAdded);
        assert_eq!(exported.body(), draft.body());
        assert_eq!(
            exported.payload_json(),
            serde_json::json!({
                "slug": "open-ticket",
                "name": "Open ticket",
                "description": "Actor opens a repair ticket."
            }),
            "JSON should be produced only at the export boundary"
        );

        Ok(())
    }

    #[test]
    fn exported_event_json_is_parsed_to_a_semantic_body() -> Result<(), Box<dyn Error>> {
        let exported = ExportedEvent::from_json_for_test(&serde_json::json!({
            "schema_version": "emc.events.v1",
            "event_id": "workflow-added-1",
            "command_id": "workflow-added-1",
            "command_ordinal": 0,
            "stream_id": "workflow::open-ticket",
            "parents": [],
            "type": "WorkflowAdded",
            "payload": {
                "slug": "open-ticket",
                "name": "Open ticket",
                "description": "Actor opens a repair ticket."
            }
        }))?;

        assert_eq!(exported.event_type(), EventDraftType::WorkflowAdded);
        assert_eq!(
            exported.body(),
            &EventDraftBody::WorkflowAdded {
                workflow: NewWorkflow::new(
                    parse_model_name("Open ticket")?,
                    parse_model_description("Actor opens a repair ticket.")?,
                    parse_workflow_slug("open-ticket")?,
                )
            },
            "exported event JSON should be parsed once into semantic event data"
        );

        Ok(())
    }

    #[test]
    fn eventcore_project_initialized_events_serialize_semantic_project_names()
    -> Result<(), Box<dyn Error>> {
        let event = EmcEvent::ProjectInitialized {
            stream_id: eventcore::StreamId::try_new("project".to_owned())?,
            name: parse_project_name("Repair Desk")?,
        };

        let event_data = serde_json::to_value(event)?;

        assert_eq!(
            event_data,
            serde_json::json!({
                "ProjectInitialized": {
                    "stream_id": "project",
                    "name": "Repair Desk"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_workflow_lifecycle_events_serialize_semantic_workflow_fields()
    -> Result<(), Box<dyn Error>> {
        let added = EmcEvent::WorkflowAdded {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            slug: parse_workflow_slug("open-ticket")?,
            name: parse_model_name("Open ticket")?,
            description: parse_model_description("Actor opens a repair ticket.")?,
        };
        let updated = EmcEvent::WorkflowUpdated {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            slug: parse_workflow_slug("open-ticket")?,
            name: parse_model_name("Open repair ticket")?,
            description: parse_model_description("Actor opens a repair ticket with priority.")?,
        };
        let removed = EmcEvent::WorkflowRemoved {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            slug: parse_workflow_slug("open-ticket")?,
        };

        assert_eq!(
            serde_json::to_value(added)?,
            serde_json::json!({
                "WorkflowAdded": {
                    "stream_id": "workflow::open-ticket",
                    "slug": "open-ticket",
                    "name": "Open ticket",
                    "description": "Actor opens a repair ticket."
                }
            })
        );
        assert_eq!(
            serde_json::to_value(updated)?,
            serde_json::json!({
                "WorkflowUpdated": {
                    "stream_id": "workflow::open-ticket",
                    "slug": "open-ticket",
                    "name": "Open repair ticket",
                    "description": "Actor opens a repair ticket with priority."
                }
            })
        );
        assert_eq!(
            serde_json::to_value(removed)?,
            serde_json::json!({
                "WorkflowRemoved": {
                    "stream_id": "workflow::open-ticket",
                    "slug": "open-ticket"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_slice_lifecycle_events_serialize_semantic_slice_fields()
    -> Result<(), Box<dyn Error>> {
        let added = EmcEvent::SliceAdded {
            stream_id: eventcore::StreamId::try_new("slice::capture-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            slug: parse_slice_slug("capture-ticket")?,
            name: parse_model_name("Capture ticket")?,
            kind: SliceKindName::StateView,
            description: parse_model_description("Actor captures repair ticket details.")?,
        };
        let updated = EmcEvent::SliceUpdated {
            stream_id: eventcore::StreamId::try_new("slice::capture-ticket".to_owned())?,
            slug: parse_slice_slug("capture-ticket")?,
            name: parse_model_name("Capture repair ticket")?,
            kind: SliceKindName::StateChange,
            description: parse_model_description("Actor edits repair ticket details.")?,
        };
        let removed = EmcEvent::SliceRemoved {
            stream_id: eventcore::StreamId::try_new("slice::capture-ticket".to_owned())?,
            slug: parse_slice_slug("capture-ticket")?,
        };

        assert_eq!(
            serde_json::to_value(added)?,
            serde_json::json!({
                "SliceAdded": {
                    "stream_id": "slice::capture-ticket",
                    "workflow": "open-ticket",
                    "slug": "capture-ticket",
                    "name": "Capture ticket",
                    "kind": "state_view",
                    "description": "Actor captures repair ticket details."
                }
            })
        );
        assert_eq!(
            serde_json::to_value(updated)?,
            serde_json::json!({
                "SliceUpdated": {
                    "stream_id": "slice::capture-ticket",
                    "slug": "capture-ticket",
                    "name": "Capture repair ticket",
                    "kind": "state_change",
                    "description": "Actor edits repair ticket details."
                }
            })
        );
        assert_eq!(
            serde_json::to_value(removed)?,
            serde_json::json!({
                "SliceRemoved": {
                    "stream_id": "slice::capture-ticket",
                    "slug": "capture-ticket"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_workflow_fact_events_serialize_semantic_outcome_and_error_fields()
    -> Result<(), Box<dyn Error>> {
        let outcome = EmcEvent::WorkflowOutcomeAdded {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            source_slice: parse_workflow_transition_endpoint("capture-ticket")?,
            label: OutcomeLabelName::try_new("ticket-captured".to_owned())?,
            externally_relevant: true,
        };
        let command_error = EmcEvent::WorkflowCommandErrorAdded {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            source_slice: parse_workflow_transition_endpoint("capture-ticket")?,
            command: CommandName::try_new("SubmitTicket".to_owned())?,
            error: CommandErrorName::try_new("DuplicateTicket".to_owned())?,
        };

        assert_eq!(
            serde_json::to_value(outcome)?,
            serde_json::json!({
                "WorkflowOutcomeAdded": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "source_slice": "capture-ticket",
                    "label": "ticket-captured",
                    "externally_relevant": true
                }
            })
        );
        assert_eq!(
            serde_json::to_value(command_error)?,
            serde_json::json!({
                "WorkflowCommandErrorAdded": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "source_slice": "capture-ticket",
                    "command": "SubmitTicket",
                    "error": "DuplicateTicket"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_workflow_lifecycle_fact_events_serialize_semantic_fields()
    -> Result<(), Box<dyn Error>> {
        let transition_evidence = EmcEvent::WorkflowTransitionEvidenceAdded {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            source: parse_workflow_transition_endpoint("capture-ticket")?,
            target: parse_workflow_transition_endpoint("review-ticket")?,
            via: WorkflowTransitionKind::Navigation,
            name: TransitionTriggerName::try_new("Review ticket".to_owned())?,
            source_evidence: WorkflowTransitionSourceEvidenceText::try_new(
                "Ticket capture is complete.".to_owned(),
            )?,
            target_evidence: WorkflowTransitionTargetEvidenceText::try_new(
                "Review screen is reachable.".to_owned(),
            )?,
        };
        let coverage_required = EmcEvent::WorkflowEntryLifecycleCoverageRequired {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
        };
        let lifecycle_state = EmcEvent::WorkflowEntryLifecycleStateAdded {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            state: WorkflowEntryLifecycleStateName::InitializedAuthenticated,
            step: parse_workflow_transition_endpoint("capture-ticket")?,
            evidence: WorkflowEntryLifecycleEvidenceText::try_new(
                "Authenticated entry is represented.".to_owned(),
            )?,
        };

        assert_eq!(
            serde_json::to_value(transition_evidence)?,
            serde_json::json!({
                "WorkflowTransitionEvidenceAdded": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "source": "capture-ticket",
                    "target": "review-ticket",
                    "via": "navigation",
                    "name": "Review ticket",
                    "source_evidence": "Ticket capture is complete.",
                    "target_evidence": "Review screen is reachable."
                }
            })
        );
        assert_eq!(
            serde_json::to_value(coverage_required)?,
            serde_json::json!({
                "WorkflowEntryLifecycleCoverageRequired": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket"
                }
            })
        );
        assert_eq!(
            serde_json::to_value(lifecycle_state)?,
            serde_json::json!({
                "WorkflowEntryLifecycleStateAdded": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "state": "initialized_authenticated",
                    "step": "capture-ticket",
                    "evidence": "Authenticated entry is represented."
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_workflow_readiness_events_serialize_semantic_fields() -> Result<(), Box<dyn Error>>
    {
        let review_event_id =
            ReviewEventId::new(ArtifactDigest::try_new("review-event-1".to_owned())?);
        let readiness = EmcEvent::WorkflowReadinessDeclared {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            projection_fingerprint: ProjectionFingerprint::new(ArtifactDigest::try_new(
                "sha256:frontier".to_owned(),
            )?),
            model_content_digest: ModelContentDigest::new(ArtifactDigest::try_new(
                "emc-fnv1a64:0000000000000000".to_owned(),
            )?),
            verified_at: ReviewTimestamp::try_new("2026-06-03T00:00:00.000Z".to_owned())?,
            verified_by: ReviewerId::try_new("event-model-reviewer".to_owned())?,
            review_event: ReviewEventReference::from_optional(Some(review_event_id)),
        };

        assert_eq!(
            serde_json::to_value(readiness)?,
            serde_json::json!({
                "WorkflowReadinessDeclared": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "projection_fingerprint": "sha256:frontier",
                    "model_content_digest": "emc-fnv1a64:0000000000000000",
                    "verified_at": "2026-06-03T00:00:00.000Z",
                    "verified_by": "event-model-reviewer",
                    "review_event_id": "review-event-1"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_workflow_connection_events_serialize_semantic_fields() -> Result<(), Box<dyn Error>>
    {
        let connection = EmcEvent::WorkflowConnected {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            source: parse_slice_slug("capture-ticket")?,
            target_slice: Some(parse_slice_slug("review-ticket")?),
            target_workflow: None,
            via: ConnectionKind::Navigation,
            name: TransitionTriggerName::try_new("Open review".to_owned())?,
            payload_contract: Some(PayloadContractName::try_new(
                "TicketReviewPayload".to_owned(),
            )?),
            reason: None,
        };
        let removal = EmcEvent::WorkflowTransitionRemoved {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            source: parse_slice_slug("review-ticket")?,
            target_slice: None,
            target_workflow: Some(parse_workflow_slug("close-ticket")?),
            via: ConnectionKind::Outcome,
            name: TransitionTriggerName::try_new("Close ticket".to_owned())?,
        };

        assert_eq!(
            serde_json::to_value(connection)?,
            serde_json::json!({
                "WorkflowConnected": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "source": "capture-ticket",
                    "target_slice": "review-ticket",
                    "target_workflow": null,
                    "via": "navigation",
                    "name": "Open review",
                    "payload_contract": "TicketReviewPayload",
                    "reason": null
                }
            })
        );
        assert_eq!(
            serde_json::to_value(removal)?,
            serde_json::json!({
                "WorkflowTransitionRemoved": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "source": "review-ticket",
                    "target_slice": null,
                    "target_workflow": "close-ticket",
                    "via": "outcome",
                    "name": "Close ticket"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_review_conflict_and_owned_definition_events_serialize_semantic_fields()
    -> Result<(), Box<dyn Error>> {
        let owned_definition = EmcEvent::WorkflowOwnedDefinitionAdded {
            stream_id: eventcore::StreamId::try_new("workflow::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            source_slice: parse_workflow_transition_endpoint("capture-ticket")?,
            definition_kind: WorkflowOwnedDefinitionKind::Event,
            definition_name: WorkflowOwnedDefinitionName::try_new("TicketCaptured".to_owned())?,
            definition_stream: Some(StreamName::try_new("ticket-events".to_owned())?),
            source_provenance: Some(parse_model_description("Capture emits the event.")?),
            event_participation: Some(WorkflowEventParticipation::Emitted),
            view_role: None,
        };
        let review = EmcEvent::ReviewRecorded {
            stream_id: eventcore::StreamId::try_new("review::open-ticket".to_owned())?,
            workflow: parse_workflow_slug("open-ticket")?,
            model_content_digest: ModelContentDigest::new(ArtifactDigest::try_new(
                "emc-fnv1a64:0000000000000000".to_owned(),
            )?),
            reviewer_id: ReviewerId::try_new("event-model-reviewer".to_owned())?,
            reviewed_at: ReviewTimestamp::try_new("2026-06-03T00:00:00.000Z".to_owned())?,
            categories: vec![
                ReviewRuleName::LifecycleEntry,
                ReviewRuleName::ScenarioCoverage,
            ],
        };
        let conflict = EmcEvent::ConflictResolved {
            stream_id: eventcore::StreamId::try_new("project".to_owned())?,
            conflict_id: EventConflictId::new(ArtifactDigest::try_new("conflict-1".to_owned())?),
            chosen_event_id: ChosenEventId::new(ArtifactDigest::try_new("chosen-1".to_owned())?),
        };

        assert_eq!(
            serde_json::to_value(owned_definition)?,
            serde_json::json!({
                "WorkflowOwnedDefinitionAdded": {
                    "stream_id": "workflow::open-ticket",
                    "workflow": "open-ticket",
                    "source_slice": "capture-ticket",
                    "definition_kind": "event",
                    "definition_name": "TicketCaptured",
                    "definition_stream": "ticket-events",
                    "source_provenance": "Capture emits the event.",
                    "event_participation": "emitted",
                    "view_role": null
                }
            })
        );
        assert_eq!(
            serde_json::to_value(review)?,
            serde_json::json!({
                "ReviewRecorded": {
                    "stream_id": "review::open-ticket",
                    "workflow": "open-ticket",
                    "model_content_digest": "emc-fnv1a64:0000000000000000",
                    "reviewer_id": "event-model-reviewer",
                    "reviewed_at": "2026-06-03T00:00:00.000Z",
                    "categories": ["lifecycle-entry", "scenario-coverage"]
                }
            })
        );
        assert_eq!(
            serde_json::to_value(conflict)?,
            serde_json::json!({
                "ConflictResolved": {
                    "stream_id": "project",
                    "conflict_id": "conflict-1",
                    "chosen_event_id": "chosen-1"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn eventcore_slice_fact_events_serialize_semantic_payloads() -> Result<(), Box<dyn Error>> {
        let fact = SliceFactInput::Outcome(NewOutcomeDefinition::new(
            parse_slice_slug("capture-ticket")?,
            OutcomeLabelName::try_new("ticket-captured".to_owned())?,
            OutcomeEventNames::from_events([EventName::try_new("TicketCaptured".to_owned())?]),
            true,
        ));
        let event = EmcEvent::SliceFactAdded {
            stream_id: eventcore::StreamId::try_new("slice::capture-ticket".to_owned())?,
            fact: SliceFactEvent::new(fact),
        };

        let serialized = serde_json::to_value(&event)?;
        assert_eq!(
            serialized,
            serde_json::json!({
                "SliceFactAdded": {
                    "stream_id": "slice::capture-ticket",
                    "exported_event_type": "SliceOutcomeAdded",
                    "payload": {
                        "slice": "capture-ticket",
                        "label": "ticket-captured",
                        "events": ["TicketCaptured"],
                        "externally_relevant": true
                    }
                }
            })
        );
        let round_trip: EmcEvent = serde_json::from_value(serialized)?;

        assert_eq!(round_trip, event);

        Ok(())
    }

    #[test]
    fn verification_invariant_sets_are_composed_from_closed_domain_names()
    -> Result<(), Box<dyn Error>> {
        let project_root = QuintInvariantSet::project_root();
        let workflow = QuintInvariantSet::workflow();
        let slice = QuintInvariantSet::slice();

        assert!(
            project_root.contains(QuintInvariantName::MODEL_WORKFLOW_BEHAVIOR_SURFACE_IS_COMPLETE)
        );
        assert!(project_root.contains(
            QuintInvariantName::MODEL_DATA_FLOW_SOURCE_CHAINS_PRESERVE_BIT_ENCODING_SEMANTICS
        ));
        assert!(
            workflow.contains(QuintInvariantName::WORKFLOW_ONLY_EVENTS_MAY_BE_SHARED_ACROSS_SLICES)
        );
        assert!(slice.contains(
            QuintInvariantName::STATE_VIEW_SLICES_REPRESENT_SINGLE_VIEW_PROJECTION_BOUNDARY
        ));
        assert_eq!(
            project_root.as_process_argument().as_ref(),
            "modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelWorkflowBehaviorSurfaceIsComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs"
        );

        Ok(())
    }
}
