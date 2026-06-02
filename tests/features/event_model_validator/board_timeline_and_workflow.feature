Feature: Event model validator enforces board semantics, timeline usability, and workflow composition

  The event-model browser renders boards and timelines from JSON. The validator
  must reject models whose board/timeline data would be mechanically misleading,
  even if references are syntactically resolvable.

  Background:
    Given validation runs through the event-model validator CLI against temporary workflow and slice "*.eventmodel.json" files

  Rule: Board lanes are canonical

    Scenario: Board lanes must use the canonical lane ids
      Given a slice board defines lane id "external"
      When I validate the event model
      Then validation fails with "board lanes must be exactly ux, actions, and events"

    Scenario: Board lanes must include every canonical lane
      Given a slice board defines lane ids "ux" and "events"
      And the board does not define lane id "actions"
      When I validate the event model
      Then validation fails with "board lanes must include canonical lane 'actions'"

    Scenario: Board lanes may not duplicate canonical lane ids
      Given a slice board defines lane id "actions" twice
      When I validate the event model
      Then validation fails with "board lane id 'actions' is duplicated"

    Scenario: Legacy projection lanes are rejected
      Given a slice board defines lane id "projection"
      When I validate the event model
      Then validation fails with "board lanes must be exactly ux, actions, and events"

    Scenario Outline: Board lane names must match the canonical lane purposes
      Given board lane "<lane id>" is named "<wrong name>"
      When I validate the event model
      Then validation fails with "board lane '<lane id>' must be named '<canonical name>'"

      Examples:
        | lane id | wrong name     | canonical name                  |
        | ux      | Teacher Review | People, Views, and Translations |
        | actions | Teacher Review | Commands and Projections        |
        | events  | Teacher Review | Stored Facts                    |

    Scenario: View elements appear in the UX lane
      Given board element "view-lesson" has kind "view" and lane "actions"
      When I validate the event model
      Then validation fails with "board element 'view-lesson' of kind 'view' must be on lane 'ux'"

    Scenario: Automation elements appear in the UX lane
      Given board element "automation-review" has kind "automation" and lane "events"
      When I validate the event model
      Then validation fails with "board element 'automation-review' of kind 'automation' must be on lane 'ux'"

    Scenario: External event elements appear in the UX lane
      Given board element "external-checkpoint" has kind "external_event" and lane "actions"
      When I validate the event model
      Then validation fails with "board element 'external-checkpoint' of kind 'external_event' must be on lane 'ux'"

    Scenario: Read-model elements appear in the actions lane
      Given board element "rm-lesson-state" has kind "read_model" and lane "ux"
      When I validate the event model
      Then validation fails with "board element 'rm-lesson-state' of kind 'read_model' must be on lane 'actions'"

    Scenario: Command elements appear in the actions lane
      Given board element "cmd-submit" has kind "command" and lane "ux"
      When I validate the event model
      Then validation fails with "board element 'cmd-submit' of kind 'command' must be on lane 'actions'"

    Scenario: Event elements appear in the events lane
      Given board element "event-lesson-accepted" has kind "event" and lane "actions"
      When I validate the event model
      Then validation fails with "board element 'event-lesson-accepted' of kind 'event' must be on lane 'events'"

  Rule: Board elements represent real declared things

    Scenario: Board elements reference known declarations
      Given board element "cmd-submit" has kind "command" and name "MissingCommand"
      When I validate the event model
      Then validation fails with "board element 'cmd-submit' references unknown command 'MissingCommand'"

    Scenario Outline: Every board element kind references an allowed declared model element
      Given board element "missing-element" has kind "<element kind>" and name "MissingDeclaration"
      And no <element kind> named "MissingDeclaration" is declared in the model
      When I validate the event model
      Then validation fails with "board element 'missing-element' references unknown <element kind> 'MissingDeclaration'"

      Examples:
        | element kind    |
        | view            |
        | command         |
        | event           |
        | read_model      |
        | automation      |
        | external_event  |

    Scenario: Automation board elements must be declared automations, not fake intermediates
      Given board element "fake-dependency" has kind "automation"
      And no automation slice or declared automation trigger is named "fake-dependency"
      When I validate the event model
      Then validation fails with "automation board element 'fake-dependency' is not declared by an automation slice"

    Scenario Outline: Undeclared non-event board elements cannot bridge dependencies
      Given board element "fake-dependency" has kind "<element kind>"
      And no declared <element kind> is named "fake-dependency"
      And the element is used only to connect a read model to a command
      When I validate the event model
      Then validation fails with "board element 'fake-dependency' is not declared"

      Examples:
        | element kind    |
        | automation      |
        | external_event  |

    Scenario: External events use explicit external event elements, not automation elements
      Given translation board element "lesson_checkpoint_result" is modeled as kind "automation"
      When I validate the event model
      Then validation fails with "external event 'lesson_checkpoint_result' must not be modeled as automation"

  Rule: External event board connections are explicit boundary crossings

    Scenario: External event to command is a valid translation trigger connection
      Given translation slice "Record checkpoint result" declares external event "lesson_checkpoint_result"
      And board connects external event "lesson_checkpoint_result" to command "RecordCheckpointResult"
      When I validate the event model
      Then validation succeeds

    Scenario: External events cannot update read models directly
      Given board connects external event "lesson_checkpoint_result" to read model "lesson_state"
      When I validate the event model
      Then validation fails with "invalid board connection 'lesson_checkpoint_result' (external_event) -> 'lesson_state' (read_model)"

    Scenario: External event triggers must match translation external_event declarations
      Given board connects external event "unknown_checkpoint_result" to command "RecordCheckpointResult"
      And translation slice "Record checkpoint result" declares external event "lesson_checkpoint_result"
      When I validate the event model
      Then validation fails with "external event board element 'unknown_checkpoint_result' is not declared by translation slice 'Record checkpoint result'"

  Rule: Board connections match causal semantics

    Scenario: Invalid board connection kind pairs are rejected
      Given the board connects command "OpenRepairTicket" to read model "repair_queue"
      When I validate the event model
      Then validation fails with "invalid board connection"

    Scenario: Commands have real incoming triggers
      Given board command element "cmd-open-ticket" has no incoming view, automation, translation, external signal, or explicit workflow trigger
      When I validate the event model
      Then validation fails with "command board element 'cmd-open-ticket' has no incoming trigger"

    Scenario: Command-to-event connections match command produces declarations
      Given board connects command "SubmitLessonForReview" to event "LessonAccepted"
      And command "SubmitLessonForReview" does not produce event "LessonAccepted"
      When I validate the event model
      Then validation fails with "board connects command 'SubmitLessonForReview' to event 'LessonAccepted' that it does not produce"

    Scenario: Event-to-read-model connections match read model field sources
      Given board connects event "LessonAccepted" to read model "lesson_submission_context"
      And read model "lesson_submission_context" has no field sourced from "LessonAccepted"
      When I validate the event model
      Then validation fails with "board connects event 'LessonAccepted' to read model 'lesson_submission_context' but the read model does not project that event"

    Scenario: View-to-command connections match owned controls
      Given board connects view "lesson_screen" to command "SubmitLessonForReview"
      And view "lesson_screen" has no control invoking "SubmitLessonForReview"
      When I validate the event model
      Then validation fails with "board connects view 'lesson_screen' to command 'SubmitLessonForReview' without an owned control"

    Scenario: Read models feeding views have incoming event updates
      Given board read model "repair_queue" feeds view "repair_queue_screen"
      And board read model "repair_queue" has no incoming event update
      When I validate the event model
      Then validation fails with "read_model board element 'repair_queue' has no incoming event/update"

    Scenario: Read models cannot feed commands on the board
      Given board connects read model "lesson_submission_context" to command "SubmitLessonForReview"
      When I validate the event model
      Then validation fails with "invalid board connection 'rm-context' (read_model) -> 'cmd-submit' (command)"

  Rule: Workflow composition references whole reachable slices

    Scenario: Workflow compositions declare explicit steps for browser timelines
      Given workflow "Lesson 01" references slice files
      And workflow "Lesson 01" declares no steps
      When I validate the composed workflow
      Then validation fails with "workflow composition must declare steps"

    Scenario: A valid workflow composition includes entry, main path, branch, async branch, and workflow exit
      Given workflow "Lesson 01" references valid slice files for entry, show lesson, submit lesson, review lesson, and checkpoint translation
      And the workflow has exactly one entry step "entry"
      And main step "show-lesson" is reached from "entry" by navigation
      And main step "submit-lesson" is reached from "show-lesson" by command
      And main step "review-lesson" is reached from "submit-lesson" by event
      And alternate branch "revision" is reached from "review-lesson" by event "LessonRevisionRequested"
      And async branch "record-checkpoint" is reached by external trigger "lesson_checkpoint_result"
      And workflow exit "next-lesson" is reached from "review-lesson" by event "LessonAccepted"
      When I validate the composed workflow
      Then validation succeeds

    Scenario: Workflow slice files must exist
      Given workflow "Lesson 01" references slice file "../slices/missing.eventmodel.json"
      When I validate the composed workflow
      Then validation fails with "missing referenced slice file"

    Scenario: Workflow slice files must each be valid
      Given workflow "Lesson 01" references an invalid slice file
      When I validate the composed workflow
      Then validation fails with "referenced slice file" and "is invalid"

    Scenario: Workflow steps reference referenced slice files
      Given workflow "Lesson 01" has step slice "show-lesson"
      And no referenced slice file has slug "show-lesson"
      When I validate the composed workflow
      Then validation fails with "workflow step 'show-lesson' does not reference a composed slice"

    Scenario: Referenced non-supporting slices appear in workflow steps
      Given workflow "Lesson 01" references slice "submit-lesson"
      And no workflow step references "submit-lesson"
      And the slice is not marked supporting
      When I validate the composed workflow
      Then validation fails with "referenced slice 'submit-lesson' is not used by workflow steps"

    Scenario: Non-entry workflow steps need incoming reachability
      Given workflow step "submit-lesson" has relationship "main"
      And no prior transition targets "submit-lesson"
      And the step has no external trigger or branch classification
      When I validate the composed workflow
      Then validation fails with "workflow step 'submit-lesson' has no incoming transition"

    Scenario: A workflow has exactly one entry step
      Given workflow "Lesson 01" has entry steps "resolve-entry" and "show-lesson"
      When I validate the composed workflow
      Then validation fails with "workflow must declare exactly one entry step"

    Scenario: Workflow step slugs are unique
      Given workflow "Lesson 01" has two steps with slice slug "show-lesson"
      When I validate the composed workflow
      Then validation fails with "workflow step slice 'show-lesson' is duplicated"

    Scenario: Every non-supporting step is reachable from the single entry step
      Given workflow "Lesson 01" has entry step "entry"
      And workflow steps "submit" and "review" transition to each other
      And no path from "entry" reaches "submit" or "review"
      When I validate the composed workflow
      Then validation fails with "workflow step 'submit' is not reachable from entry step 'entry'"

    Scenario: Transitions target known workflow steps or explicit workflow exits
      Given workflow step "entry" transitions to "missing-step"
      And no workflow step has slice slug "missing-step"
      When I validate the composed workflow
      Then validation fails with "transition targets unknown workflow step 'missing-step'"

    Scenario: Branch steps still declare their trigger or incoming transition rationale
      Given workflow step "revision" has relationship "alternate"
      And no transition targets "revision"
      And the step has no explicit external trigger
      When I validate the composed workflow
      Then validation fails with "alternate workflow step 'revision' must declare a trigger or incoming transition"

    Scenario: Composed workflows reject non-canonical lanes in referenced slices
      Given workflow "Lesson 01" references slice file "course-show-lesson.eventmodel.json"
      And that slice file defines board lane id "projection"
      When I validate the composed workflow
      Then validation fails with "referenced slice file" and "board lanes must be exactly ux, actions, and events"

    Scenario: Lifecycle branches are not modeled as required linear happy path
      Given workflow step "record-member-suspension" has relationship "main"
      And the step is triggered by async external event "scim_member_suspended"
      When I validate the composed workflow
      Then validation fails with "async lifecycle step 'record-member-suspension' must be alternate or async_lifecycle"

    Scenario: Workflow entry handles first-arrival lifecycle state before bootstrap
      Given workflow "Organization access" begins with bootstrap state-change step "bootstrap-root-organization"
      And no earlier application-entry state-view step decides whether bootstrap is needed
      When I validate the composed workflow
      Then validation fails with "should model the application entry root bootstrap state view before bootstrap"

    Scenario Outline: Application-entry state views cover important lifecycle and session states
      Given workflow "Organization access" has application-entry state-view step "resolve-application-entry"
      And the entry slice has no scenario for "<entry state>"
      When I validate the composed workflow
      Then validation fails with "application entry slice 'resolve-application-entry' must cover <entry state> state"

      Examples:
        | entry state                              |
        | fresh and uninitialized                  |
        | already initialized and unauthenticated  |
        | already initialized and authenticated    |
        | partially configured                     |
        | fully configured                         |

  Rule: Workflow transitions resolve to modeled sources and targets

    Scenario: Event transitions use events produced or observed by adjacent slices
      Given workflow transition from "submit-lesson" to "review-lesson" uses event "LessonSubmittedForReview"
      And source slice "submit-lesson" does not emit "LessonSubmittedForReview"
      And target slice "review-lesson" does not observe "LessonSubmittedForReview"
      When I validate the composed workflow
      Then validation fails with "transition event 'LessonSubmittedForReview' is not shared by source and target slices"

    Scenario: Event transitions require the source slice to emit or observe the transition event
      Given workflow transition from "submit-lesson" to "review-lesson" uses event "LessonSubmittedForReview"
      And target slice "review-lesson" observes "LessonSubmittedForReview"
      And source slice "submit-lesson" does not emit or observe "LessonSubmittedForReview"
      When I validate the composed workflow
      Then validation fails with "transition event 'LessonSubmittedForReview' is not available from source slice 'submit-lesson'"

    Scenario: Event transitions require the target slice to observe or emit the transition event
      Given workflow transition from "submit-lesson" to "review-lesson" uses event "LessonSubmittedForReview"
      And source slice "submit-lesson" emits "LessonSubmittedForReview"
      And target slice "review-lesson" does not observe or emit "LessonSubmittedForReview"
      When I validate the composed workflow
      Then validation fails with "transition event 'LessonSubmittedForReview' is not accepted by target slice 'review-lesson'"

    Scenario: Command transitions come from controls owned by source views
      Given workflow transition from "show-lesson" to "submit-lesson" uses command "SubmitLessonForReview"
      And source view "lesson_screen" has no control invoking "SubmitLessonForReview"
      When I validate the composed workflow
      Then validation fails with "transition command 'SubmitLessonForReview' is not invoked by source view 'lesson_screen'"

    Scenario: Command transitions target the slice that owns the command
      Given workflow transition from "show-lesson" to "submit-lesson" uses command "SubmitLessonForReview"
      And source view "lesson_screen" has a control invoking "SubmitLessonForReview"
      And target slice "submit-lesson" does not own command "SubmitLessonForReview"
      When I validate the composed workflow
      Then validation fails with "transition command 'SubmitLessonForReview' is not owned by target slice 'submit-lesson'"

    Scenario: Navigation transitions come from controls owned by source views
      Given workflow transition from "entry" to "show-lesson" uses navigation "lesson_screen"
      And source view "course_entry_resolution" has no control navigating to "lesson_screen"
      When I validate the composed workflow
      Then validation fails with "navigation transition to 'lesson_screen' is not owned by source view 'course_entry_resolution'"

    Scenario: Navigation transitions resolve to the target workflow step's entry view
      Given workflow transition from "entry" to "show-lesson" uses navigation "lesson_screen"
      And source view "course_entry_resolution" has a control navigating to "lesson_screen"
      And target step "show-lesson" does not own or expose view "lesson_screen"
      When I validate the composed workflow
      Then validation fails with "navigation target 'lesson_screen' does not resolve to target step 'show-lesson'"

    Scenario: External trigger transitions declare trigger payload contracts
      Given workflow transition to "record-checkpoint" uses external trigger "lesson_checkpoint_result"
      And target translation slice "record-checkpoint" does not declare external event "lesson_checkpoint_result"
      When I validate the composed workflow
      Then validation fails with "external trigger 'lesson_checkpoint_result' is not declared by target slice 'record-checkpoint'"

    Scenario: Workflow exits name the target workflow
      Given workflow transition exits to workflow "course-lesson-02"
      And the transition has no event, navigation, command, or explicit exit reason
      When I validate the composed workflow
      Then validation fails with "workflow exit to 'course-lesson-02' must declare why the exit is reached"

  Rule: Board shapes support human-navigable timelines

    Scenario: Validator rejects disconnected board islands in a main workflow step
      Given board slice "Show Lesson" has two disconnected element graphs
      And one disconnected graph is not marked alternate or supporting
      When I validate the composed workflow
      Then validation fails with "board slice 'Show Lesson' has disconnected main-path elements"

    Scenario: Workflow files compose whole slices without redefining internals
      Given workflow file "Lesson 01" defines its own command "SubmitLessonForReview"
      And workflow file "Lesson 01" also references slice file "course-submit-lesson-for-review.eventmodel.json"
      When I validate the composed workflow
      Then validation fails with "workflow files must not define commands, views, read models, automations, or scenarios"

    Scenario: Workflow steps do not select internal scenarios from a slice
      Given workflow step "submit-lesson" references scenario "missing evidence" inside slice "Submit lesson"
      When I validate the composed workflow
      Then validation fails with "workflow step 'submit-lesson' must compose the whole slice, not scenario 'missing evidence'"
