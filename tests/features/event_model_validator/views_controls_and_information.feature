Feature: Event model validator enforces view information completeness and navigation contracts

  Views are information design, not page decoration. Every meaningful displayed
  datum, user-provided value, control, and navigation target must be traceable and
  classified.

  Background:
    Given validation runs through the event-model validator CLI against temporary workflow and slice "*.eventmodel.json" files
    Given a valid state-view event model named "Show repair queue"

  Rule: View sketches are complete and constrained

    Scenario: Views require information sketches
      Given view "repair_queue_screen" has no wireframe
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' is missing wireframe"

    Scenario: Wireframe tokens map to modeled fields, controls, or actor inputs
      Given view "repair_queue_screen" wireframe references token "decorative_total"
      And "decorative_total" is not a view field, control label, or actor-provided input
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' wireframe token 'decorative_total' does not match a field or control"

    Scenario: Every displayed field appears in the information sketch
      Given view "repair_queue_screen" defines field "customer_name"
      And the wireframe does not reference "customer_name"
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' wireframe does not reference field 'customer_name'"

    Scenario: Every control appears in the information sketch
      Given view "repair_queue_screen" defines control "Open repair ticket"
      And the wireframe does not reference "Open repair ticket"
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' wireframe does not reference control 'Open repair ticket'"

  Rule: Visible fields source only from read models used by the view

    Scenario: Event attributes must declare source provenance
      Given event "RepairTicketOpened" attribute "customer_name" has no source
      When I validate the event model
      Then validation fails with "event 'RepairTicketOpened' attribute 'customer_name' is missing source"

    Scenario: Read model fields must declare source provenance
      Given read model "repair_queue" field "customer_name" has no source
      When I validate the event model
      Then validation fails with "read model 'repair_queue' field 'customer_name' is missing source"

    Scenario: View fields must declare source provenance
      Given view "repair_queue_screen" field "customer_name" has no source
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' field 'customer_name' is missing source"

    Scenario: View fields must source from referenced read model fields
      Given view "repair_queue_screen" uses read model "repair_queue"
      And view field "customer_name" has source "read_model.customer_profile.customer_name"
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' field 'customer_name' must source from a referenced read model field"

    Scenario: View fields must not source directly from events
      Given view field "customer_name" has source "RepairTicketOpened.customer_name"
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' field 'customer_name' must source from a referenced read model field"

    Scenario: Every displayed datum has a full source chain to original provenance
      Given view field "customer_name" sources from read model field "repair_queue.customer_name"
      And read model field "repair_queue.customer_name" sources from event attribute "RepairTicketOpened.customer_name"
      And event attribute "RepairTicketOpened.customer_name" does not source from command input, external input field, generated value, session value, or modeled derivation
      When I validate the event model
      Then validation fails with "view field 'customer_name' source chain stops before original provenance"

    Scenario: Command inputs have reportable source chains
      Given command "SubmitLessonForReview" declares input "reflection_answer"
      And the issuing control provides "reflection_answer" from "user_input.reflection_answer"
      But "user_input.reflection_answer" is not declared as an actor-provided input, session value, generated value, external payload field, or read-model source
      When I validate the event model
      Then validation fails with "command input 'reflection_answer' source chain is incomplete"

    Scenario: Command inputs require modeled provenance when no issuing control supplies them
      Given command "RecordTeacherReview" declares input "review_decision"
      And no view control, automation, translation, external payload, session value, generated value, or read model source supplies "review_decision"
      When I validate the event model
      Then validation fails with "command input 'review_decision' is missing source provenance"

    Scenario: Command inputs sourced from read models trace back to original provenance
      Given command "SubmitLessonForReview" declares input "evidence_summary"
      And input "evidence_summary" is sourced from "read_model.lesson_submission_context.evidence_summary"
      And read model field "lesson_submission_context.evidence_summary" does not trace to an event attribute
      When I validate the event model
      Then validation fails with "command input 'evidence_summary' source chain stops before event provenance"

  Rule: Command controls provide complete inputs and error handling

    Scenario: Controls reference known commands
      Given control "Open repair ticket" references command "MissingCommand"
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' control 'Open repair ticket' references unknown command 'MissingCommand'"

    Scenario: Controls provide every command input
      Given command "OpenRepairTicket" declares input "customer_name"
      And control "Open repair ticket" invokes "OpenRepairTicket"
      And the control does not provide input "customer_name"
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' control 'Open repair ticket' does not provide input 'customer_name' for command 'OpenRepairTicket'"

    Scenario: Control inputs require source provenance
      Given control "Open repair ticket" provides input "customer_name" without source
      When I validate the event model
      Then validation fails with "control 'Open repair ticket' input 'customer_name' is missing source"

    Scenario: Control inputs require descriptions
      Given control "Open repair ticket" provides input "customer_name" without description
      When I validate the event model
      Then validation fails with "control 'Open repair ticket' input 'customer_name' is missing description"

    Scenario: Actor-provided inputs must be visible in the sketch
      Given control input "customer_name" has source "user_input.customer_name"
      And the wireframe does not reference "customer_name"
      When I validate the event model
      Then validation fails with "view 'repair_queue_screen' wireframe does not reference control input 'customer_name'"

    Scenario: Hidden session inputs may stay out of the sketch but still need descriptions
      Given control input "actor_user_id" has source "session.user_id"
      And control input "actor_user_id" has a description
      And the wireframe does not reference "actor_user_id"
      When I validate the event model
      Then validation succeeds

    Scenario: Decision fields must be visible to the actor
      Given control "Open repair ticket" declares decision field "priority"
      And view "repair_queue_screen" does not display field "priority"
      When I validate the event model
      Then validation fails with "control 'Open repair ticket' decision field 'priority' is not visible on the screen"

    Scenario: Views handle every returned error from commands they issue
      Given control "Submit lesson" invokes command "SubmitLessonForReview"
      And command "SubmitLessonForReview" declares error "checkpoint_evidence_required"
      And control "Submit lesson" does not declare handling for "checkpoint_evidence_required"
      When I validate the event model
      Then validation fails with "view 'lesson_screen' control 'Submit lesson' does not handle command error 'checkpoint_evidence_required'"

  Rule: Navigation targets are explicit and resolvable

    Scenario: Navigation controls declare a navigation type
      Given control "Open settings" navigates to "settings_screen"
      And control "Open settings" has no navigation type
      When I validate the event model
      Then validation fails with "navigation target must be classified as modeled_view, local_view_state, external_system, or external_workflow"

    Scenario: Modeled-view navigation targets existing views
      Given control "Open settings" has navigation type "modeled_view"
      And control "Open settings" navigates to "missing_settings_screen"
      And no composed slice defines view "missing_settings_screen"
      When I validate the composed workflow
      Then validation fails with "references unknown modeled view navigation target 'missing_settings_screen'"

    Scenario: Local view-state navigation identifies the owning view state or filter
      Given control "Filter direct reports" has navigation type "local_view_state"
      And the target local state "direct_reports" is not declared by the owning view
      When I validate the event model
      Then validation fails with "local view state navigation target 'direct_reports' is not declared by view 'manager_progress_visibility_screen'"

    Scenario: External workflow navigation names a workflow target
      Given control "Open report detail" has navigation type "external_workflow"
      And the control does not name a workflow target
      When I validate the event model
      Then validation fails with "external workflow navigation must name the target workflow"

    Scenario: External system navigation names the external system and handoff contract
      Given control "Run checkpoint" has navigation type "external_system"
      And the control does not name the external system or returned payload contract
      When I validate the event model
      Then validation fails with "external system navigation must name the external system and returned payload contract"
