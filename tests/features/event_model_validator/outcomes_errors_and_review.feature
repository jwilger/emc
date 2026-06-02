Feature: Event model validator enforces outcomes, errors, scenarios, and review readiness

  Business outcomes, command errors, and scenarios determine how workflows branch
  and recover. They must be mechanically declared instead of hidden in prose.

  Background:
    Given validation runs through the event-model validator CLI against temporary workflow and slice "*.eventmodel.json" files

  Rule: Slice outcomes are unique and event-set backed

    Scenario: Externally relevant outcomes have unique labels
      Given slice "Activate member" declares two outcomes labeled "activated"
      When I validate the event model
      Then validation fails with "slice 'Activate member' has duplicate outcome label 'activated'"

    Scenario: Different outcomes cannot use the same event set
      Given slice "Activate member" declares outcome "activated" with event set "OrganizationMemberActivated"
      And the same slice declares outcome "already_active" with event set "OrganizationMemberActivated"
      When I validate the event model
      Then validation fails with "outcomes 'activated' and 'already_active' use the same event set"

    Scenario: Different outcomes cannot use the same multi-event set in a different order
      Given slice "Activate member" declares outcome "activated" with event set "OrganizationMemberActivated, OrganizationMemberAssignedRole"
      And the same slice declares outcome "activated_with_role" with event set "OrganizationMemberAssignedRole, OrganizationMemberActivated"
      When I validate the event model
      Then validation fails with "outcomes 'activated' and 'activated_with_role' use the same event set"

    Scenario: Outcomes must declare at least one event in their event set
      Given slice "Activate member" declares outcome "activated" with an empty event set
      When I validate the event model
      Then validation fails with "outcome 'activated' must declare at least one event"

    Scenario: Outcome event sets reference events emitted or observed by the slice
      Given slice "Activate member" declares outcome "activated" with event "MissingEvent"
      When I validate the event model
      Then validation fails with "outcome 'activated' references unknown event 'MissingEvent'"

    Scenario: Outcome event sets cannot reference unrelated known events
      Given event "OrganizationMemberSuspended" is declared by another slice
      And slice "Activate member" declares outcome "activated" with event "OrganizationMemberSuspended"
      But slice "Activate member" neither emits nor observes "OrganizationMemberSuspended"
      When I validate the event model
      Then validation fails with "outcome 'activated' references event 'OrganizationMemberSuspended' that is not emitted or observed by slice 'Activate member'"

  Rule: Workflow compositions exhaustively handle slice outcomes

    Scenario: Workflow handles every externally relevant outcome
      Given state-change slice "Activate member" exposes outcomes "activated" and "not_authorized"
      And workflow "Organization access" handles only outcome "activated"
      When I validate the composed workflow
      Then validation fails with "workflow 'Organization access' does not handle outcome 'not_authorized' from slice 'Activate member'"

    Scenario: Workflow transitions cannot branch on command-local errors as outcomes
      Given command "ActivateMember" declares error "member_suspended"
      And workflow "Organization access" has a transition from slice "Activate member" using error "member_suspended"
      When I validate the composed workflow
      Then validation fails with "workflow transition cannot use command-local error 'member_suspended' as a business outcome"

  Rule: Command-local errors are declared and handled

    Scenario: Scenario error references must be declared by the command
      Given state-change slice "Submit lesson" scenario expects "error reflection_required is returned"
      And command "SubmitLesson" does not declare error "reflection_required"
      When I validate the event model
      Then validation fails with "scenario references undeclared command error 'reflection_required'"

    Scenario: Declared command errors require scenario coverage
      Given command "SubmitLesson" declares error "reflection_required"
      And state-change slice "Submit lesson" has no scenario where "reflection_required" is returned
      When I validate the event model
      Then validation fails with "command error 'reflection_required' must be covered by a state-change scenario"

    Scenario: Views handle every error returned by commands they issue
      Given view "lesson_screen" has control "Submit for review" invoking command "SubmitLesson"
      And command "SubmitLesson" declares errors "evidence_required" and "reflection_required"
      And the control handles only "evidence_required"
      When I validate the event model
      Then validation fails with "control 'Submit for review' does not handle command error 'reflection_required'"

    Scenario: Automations handle every error returned by commands they issue
      Given automation slice "Review lesson" issues command "RecordTeacherReview"
      And command "RecordTeacherReview" declares error "review_decision_required"
      And the automation has no error handling for "review_decision_required"
      When I validate the event model
      Then validation fails with "automation slice 'Review lesson' does not handle command error 'review_decision_required'"

    Scenario: Error handling describes recovery, not only display text
      Given control "Submit for review" handles error "evidence_required"
      And the handling has no recovery action, navigation target, retry, or stay-on-screen behavior
      When I validate the event model
      Then validation fails with "error handling for 'evidence_required' must describe recovery behavior"

  Rule: Scenarios cover required behavior for each slice type

    Scenario: State-change slices include concrete Given When Then scenarios
      Given state-change slice "Submit lesson" has a scenario missing "when"
      When I validate the event model
      Then validation fails with "slice 'Submit lesson' scenario is missing 'when'"

    Scenario: State-view slices include empty and partial reachable states
      Given state-view slice "Show lesson" displays optional checkpoint evidence
      And no scenario covers the state before checkpoint evidence exists
      When I validate the event model
      Then validation fails with "state_view slice 'Show lesson' must cover the empty or partial state before 'LessonCheckpointEvidenceRecorded'"

    Scenario: Translation slices include one scenario for each external payload variant
      Given translation slice "Record SCIM member lifecycle" accepts external events "member_provisioned" and "member_suspended"
      And the slice has no scenario for "member_suspended"
      When I validate the event model
      Then validation fails with "translation slice 'Record SCIM member lifecycle' lacks scenario for external event 'member_suspended'"

    Scenario: Automation slices include one scenario for each trigger event
      Given automation slice "Review lesson" triggers on events "LessonSubmittedForReview" and "LessonReviewRetried"
      And the slice has no scenario for "LessonReviewRetried"
      When I validate the event model
      Then validation fails with "automation slice 'Review lesson' lacks scenario for trigger 'LessonReviewRetried'"

  Rule: Complete valid models remain accepted

    Scenario: A minimal composed workflow with owned slices and reachable transitions is valid
      Given a workflow composition with one entry state-view slice, one state-change slice, and one result state-view slice
      And every workflow step is reachable from the entry step by explicit transition
      And every slice file contains exactly one slice
      And only identical event definitions are shared across slice files
      When I validate the composed workflow
      Then validation succeeds

    Scenario: A minimal composed workflow with complete information chains is valid
      Given a workflow composition with one state-view slice that displays a field from a read model
      And the displayed field traces to a read model field, event attribute, and command input source
      And every actor-provided command input is visible and described
      And every hidden session input is described
      When I validate the composed workflow
      Then validation succeeds

    Scenario: A minimal composed workflow with handled errors and canonical boards is valid
      Given a workflow composition with a view issuing a command that declares one returned error
      And the issuing view handles that returned error with stay-on-screen recovery
      And every board uses canonical lanes and slice-appropriate causal connections
      When I validate the composed workflow
      Then validation succeeds
