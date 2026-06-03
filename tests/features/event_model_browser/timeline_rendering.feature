Feature: Event model browser renders composed timelines that humans can inspect

  The validator should reject mechanically misleading shape, and the browser
  should render valid composed models without duplicating lanes or hiding branch
  structure.

  Background:
    Given the Vite event-model browser is running against temporary workflow and slice JSON fixtures
    And I open the event-model browser in a browser automation session

  Scenario: Composed board lanes are not repeated per slice
    Given workflow "Lesson 01" references five slice files each with canonical board lanes
    When I load the workflow in the event-model browser
    Then the composed timeline shows one "ux" lane, one "actions" lane, and one "events" lane

  Scenario: Timeline steps render workflow order rather than concatenated slice internals
    Given workflow "Lesson 01" declares steps "entry", "show lesson", "record evidence", "submit", and "review"
    When I load the workflow in the event-model browser
    Then I see a timeline step named "entry"
    And I see a timeline step named "show lesson"
    And I see a timeline step named "record evidence"
    And I see a timeline step named "submit"
    And I see a timeline step named "review"
    And alternate transition cards are outside the section labeled "Main path"

  Scenario: Timeline highlights disconnected supporting or alternate branches distinctly
    Given workflow "Organization access" has an async lifecycle step "record-member-suspension"
    When I load the workflow in the event-model browser
    Then I see a timeline branch card named "record-member-suspension"
    And the branch card is labeled "async lifecycle"
    And the section labeled "Main path" does not list "record-member-suspension" as the next required step

  Scenario Outline: Timeline transition labels explain why a step is reachable
    Given workflow "Lesson 01" has a transition from "<source>" to "<target>" via <transition kind> "<reason>"
    When I load the workflow in the event-model browser
    Then the transition from "<source>" to "<target>" is labeled with "<reason>"
    And the label identifies the transition kind as "<transition kind>"

    Examples:
      | source      | target       | transition kind  | reason                         |
      | entry       | show lesson  | navigation       | lesson_screen                  |
      | show lesson | submit       | command          | SubmitLessonForReview          |
      | submit      | review       | event            | LessonSubmittedForReview       |
      | show lesson | checkpoint   | external trigger | lesson_checkpoint_result       |
      | review      | next lesson  | workflow exit    | LessonAccepted                 |

  Scenario: Timeline renders alternate outcome branches apart from the happy path
    Given workflow "Lesson 01" has accepted outcome "LessonAccepted"
    And workflow "Lesson 01" has alternate outcome "LessonRevisionRequested"
    When I load the workflow in the event-model browser
    Then I see a timeline branch card named "LessonRevisionRequested"
    And the branch card is labeled "alternate outcome"
    And the section labeled "Main path" does not list "LessonRevisionRequested" as the accepted next step

  Scenario: Timeline renders retry branches apart from the happy path
    Given workflow "Lesson 01" has retry transition "RegenerateTeacherReview"
    When I load the workflow in the event-model browser
    Then I see a timeline transition named "RegenerateTeacherReview"
    And the transition is labeled "retry"
    And the transition target is the original review step

  Scenario: Timeline renders error recovery branches apart from the happy path
    Given view "lesson_screen" handles command error "evidence_required" by staying on the screen
    When I load the workflow in the event-model browser
    Then I see an error recovery card named "evidence_required"
    And the recovery card shows source screen "lesson_screen"
    And no event element is rendered for "evidence_required"

  Scenario: Timeline overlays show unreachable or weakly justified steps during review
    Given a raw review fixture for workflow "Lesson 01" has step "review" with no path from the entry step
    And the review fixture includes validator diagnostic "entry reachability" for step "review"
    When I enable review mode from the event-model browser toolbar
    And I load the review fixture in the event-model browser
    Then I see timeline step "review" marked "unreachable"
    And the transition inspector shows missing rule "entry reachability"

  Scenario: Definition views show ownership and back-references
    Given command "SubmitLessonForReview" is owned by slice "Submit lesson for review"
    And view "lesson_screen" has a control that invokes "SubmitLessonForReview"
    When I inspect the command definition in the event-model browser
    Then I see owning slice "Submit lesson for review"
    And I see source control "lesson_screen / Submit for review"
    And I see sections labeled "Produced events", "Read models", "Returned errors", and "Workflow transitions"

  Scenario: Definition views show full source chains for displayed fields
    Given view "lesson_screen" displays field "lesson_title"
    And field "lesson_title" traces through read model "lesson_state" to event "CourseLessonCatalogPublished" and external input "course_lesson_catalog_manifest.lesson_title"
    When I inspect the view definition in the event-model browser
    Then I see the full source chain for "lesson_title"
    And each source-chain hop links to its definition

  Scenario: Definition views show control effects and navigation targets
    Given view "lesson_screen" has control "Submit for review" invoking command "SubmitLessonForReview"
    And view "lesson_screen" has control "Open next lesson" navigating to workflow "course-lesson-02"
    When I inspect the view definition in the event-model browser
    Then I see the command effect for "Submit for review"
    And I see the workflow navigation target for "Open next lesson"
