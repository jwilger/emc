Feature: Event model validator enforces slice architecture boundaries

  A slice is the smallest useful modeled behavior boundary. Only events may be
  shared across slices. Commands, views, controls, read models, automations,
  translations, scenarios, and UI are owned by one slice.

  Background:
    Given validation runs through the event-model validator CLI against temporary workflow and slice "*.eventmodel.json" files

  Rule: State-view slices own views, read models, and projection paths

    Scenario: State-view slices must own at least one view
      Given a slice named "Show repair queue" has type "state_view"
      And the slice owns no views
      When I validate the event model
      Then validation fails with "state_view slice 'Show repair queue' must own at least one view"

    Scenario: State-view read models are updated by observed events before feeding views
      Given state-view slice "Show repair queue" has board connection "repair_queue" to "repair_queue_screen"
      And board read model "repair_queue" has no incoming event update
      When I validate the event model
      Then validation fails with "read_model board element 'repair_queue' has no incoming event/update"

    Scenario: State-view slices do not own state-changing commands
      Given state-view slice "Show repair queue" owns command "OpenRepairTicket"
      When I validate the event model
      Then validation fails with "state_view slice 'Show repair queue' must not own command 'OpenRepairTicket'"

    Scenario: State-view controls may link to commands owned by state-change slices
      Given state-view slice "Show repair queue" has control "Open repair ticket"
      And the control links to command "OpenRepairTicket" owned by state-change slice "Open repair ticket"
      When I validate the composed workflow
      Then validation succeeds

  Rule: State-change slices own commands, emitted event facts, outcomes, and errors

    Scenario: State-change slices emit at least one event
      Given state-change slice "Open repair ticket" owns command "OpenRepairTicket"
      And command "OpenRepairTicket" produces no events
      When I validate the event model
      Then validation fails with "state_change slice 'Open repair ticket' must emit at least one event"

    Scenario: State-change slices do not own source or post-command views
      Given state-change slice "Open repair ticket" owns view "repair_queue_screen"
      When I validate the event model
      Then validation fails with "state_change slice 'Open repair ticket' must not own view 'repair_queue_screen'"

    Scenario Outline: State-change slices do not own non-command UI/projection/automation definitions
      Given state-change slice "Submit lesson" owns a <definition kind> named "<definition name>"
      When I validate the event model
      Then validation fails with "state_change slice 'Submit lesson' must not own <definition kind> '<definition name>'"

      Examples:
        | definition kind | definition name             |
        | read model      | lesson_submission_projection |
        | automation      | Review lesson submission     |
        | translation     | Record checkpoint result     |
        | control         | Submit for review            |
        | wireframe       | lesson_screen                |

    Scenario: Legacy command read-model reads are rejected
      Given command "SubmitLessonForReview" declares legacy read-model dependency "lesson_submission_context"
      When I validate the event model
      Then validation fails with "command 'SubmitLessonForReview' uses legacy read-model reads"

    Scenario: State-change scenarios name stream reads for written events
      Given state-change scenario "submit lesson" records event "LessonSubmittedForReview" in stream "lesson_submission"
      And scenario "submit lesson" does not name stream "lesson_submission" in given_streams
      When I validate the event model
      Then validation fails with "state-change scenario 'submit lesson' writes stream 'lesson_submission' but does not name it in given_streams"

    Scenario: Singleton state changes declare repeat behavior
      Given state-change slice "Bootstrap Root organization" is marked as singleton
      And the slice has no scenario for an already-existing Root organization
      When I validate the event model
      Then validation fails with "singleton state_change slice 'Bootstrap Root organization' must declare already-exists or idempotent behavior"

  Rule: Translation slices represent boundary crossing without owning UI

    Scenario: Translation slices have an external signal or payload contract
      Given translation slice "Record SCIM member provisioning" has no external event or external input schema
      When I validate the event model
      Then validation fails with "translation slice 'Record SCIM member provisioning' must declare an external event or payload contract"

    Scenario: Translation slices do not own screens
      Given translation slice "Activate member from SAML claim" owns view "organization_sign_in_screen"
      When I validate the event model
      Then validation fails with "translation slice 'Activate member from SAML claim' must not own view 'organization_sign_in_screen'"

    Scenario: Board read-model to command dependencies require declared automations
      Given board read model "scim_configuration" influences command "RecordSCIMMember"
      And the board inserts an undeclared automation element between "scim_configuration" and "RecordSCIMMember"
      When I validate the event model
      Then validation fails with "board element between read model 'scim_configuration' and command 'RecordSCIMMember' is not a declared automation"

  Rule: Automation slices represent one coherent reaction

    Scenario: Automation slices declare a trigger
      Given automation slice "Review lesson submission" has no trigger
      When I validate the event model
      Then validation fails with "automation slice 'Review lesson submission' must declare a trigger"

    Scenario: Automation slices issue one command for a single triggered operation
      Given automation slice "Review lesson submission" is triggered by event "LessonSubmittedForReview"
      And one scenario for that trigger issues commands "RecordAcceptedReview" and "NotifyInstructor"
      When I validate the event model
      Then validation fails with "automation slice 'Review lesson submission' must issue one command per operation"

    Scenario: Automations handle every command error they can receive
      Given automation slice "Review lesson submission" issues command "RecordTeacherReview"
      And command "RecordTeacherReview" declares error "review_decision_required"
      And automation slice "Review lesson submission" does not handle "review_decision_required"
      When I validate the event model
      Then validation fails with "automation slice 'Review lesson submission' does not handle command error 'review_decision_required'"

  Rule: Non-event definitions are not shared across slices

    Scenario: Duplicate commands across composed slices are rejected
      Given workflow "Lesson 01" references two slice files that both define command "SubmitLessonForReview"
      When I validate the composed workflow
      Then validation fails with "command 'SubmitLessonForReview' is defined by more than one slice"

    Scenario: Duplicate read models across composed slices are rejected
      Given workflow "Lesson 01" references two slice files that both define read model "lesson_submission_context"
      When I validate the composed workflow
      Then validation fails with "read model 'lesson_submission_context' is defined by more than one slice"

    Scenario: Duplicate views across composed slices are rejected
      Given workflow "Lesson 01" references two slice files that both define view "lesson_screen"
      When I validate the composed workflow
      Then validation fails with "view 'lesson_screen' is defined by more than one slice"

    Scenario: Duplicate controls across composed slices are rejected
      Given workflow "Lesson 01" references two slice files that both define control "Submit for review" on view "lesson_screen"
      When I validate the composed workflow
      Then validation fails with "control 'Submit for review' on view 'lesson_screen' is defined by more than one slice"

    Scenario: Duplicate automations across composed slices are rejected
      Given workflow "Lesson 01" references two slice files that both define automation "Review lesson submission"
      When I validate the composed workflow
      Then validation fails with "automation 'Review lesson submission' is defined by more than one slice"

    Scenario: Duplicate translations across composed slices are rejected
      Given workflow "Lesson 01" references two slice files that both define translation "Record checkpoint result"
      When I validate the composed workflow
      Then validation fails with "translation 'Record checkpoint result' is defined by more than one slice"

    Scenario: Duplicate scenarios across composed slices are rejected unless they belong to the same owned slice
      Given workflow "Lesson 01" references two slice files that both define scenario "teacher accepts progress" for different slices
      When I validate the composed workflow
      Then validation fails with "scenario 'teacher accepts progress' is ambiguously defined across slices"

    Scenario: Duplicate UI wireframes across composed slices are rejected
      Given workflow "Lesson 01" references two slice files that both define a wireframe for view "lesson_screen"
      When I validate the composed workflow
      Then validation fails with "wireframe for view 'lesson_screen' is defined by more than one slice"

    Scenario: Duplicate events are allowed only when their definitions are identical
      Given workflow "Lesson 01" references two slice files that both define event "LessonAccepted"
      And the two "LessonAccepted" definitions have different attributes
      When I validate the composed workflow
      Then validation fails with "event 'LessonAccepted' has conflicting definitions across slices"

    Scenario: Identical duplicate events are allowed across slices
      Given workflow "Lesson 01" references two slice files that both define identical event "LessonAccepted"
      When I validate the composed workflow
      Then validation succeeds

    Scenario: Shared events cannot differ by stream
      Given workflow "Lesson 01" references two slice files that both define event "LessonAccepted"
      And one definition stores the event in stream "course_progress"
      And the other definition stores the event in stream "lesson_progress"
      When I validate the composed workflow
      Then validation fails with "event 'LessonAccepted' has conflicting definitions across slices"

    Scenario: Shared events cannot differ by source provenance
      Given workflow "Lesson 01" references two slice files that both define event "LessonAccepted" attribute "learner_id"
      And one definition sources "learner_id" from "command.learner_id"
      And the other definition sources "learner_id" from "read_model.lesson_review_packet.learner_id"
      When I validate the composed workflow
      Then validation fails with "event 'LessonAccepted' has conflicting definitions across slices"
