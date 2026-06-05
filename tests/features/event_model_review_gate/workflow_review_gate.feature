# Copyright 2026 John Wilger

Feature: Event model review gate prevents weak reviews from advancing workflow modeling

  Workflow modeling may proceed only after explicit clean review. Empty output,
  validator-only success, or unresolved findings are not clean review.

  Background:
    Given the review gate stores review records using path template "reviews/{workflow_slug}.review.json"
    And each review record contains fields:
      | field                |
      | workflow_slug        |
      | model_content_digest |
      | reviewer_id          |
      | status               |
      | category_results     |
      | mandatory_findings   |
      | reviewed_at          |
    And a clean review record has status "clean"
    And a dirty review record has status "changes_requested"
    And the reviewed model content digest must match the current workflow and slice files
    Given the required clean-review category markers are:
      | category marker       |
      | lifecycle-entry       |
      | canonical-lanes       |
      | board-connections     |
      | fake-intermediates    |
      | slice-ownership       |
      | source-chains         |
      | workflow-reachability |
      | transition-resolution |
      | navigation-targets    |
      | branch-shape          |
      | outcomes-and-errors   |
      | scenario-coverage     |
      | timeline-rendering    |

  Scenario: Clean review records must match the current model digest
    Given workflow "Lesson 03" has current model digest "abc123"
    And review record "reviews/course-lesson-03.review.json" has model content digest "def456"
    And the review record has status "clean"
    When I evaluate whether workflow "Lesson 03" may advance
    Then modeling is blocked with "clean review is stale for current model digest"

  Scenario: Clean review records use category result markers
    Given workflow "Lesson 03" has a current clean review record
    And the review record omits category result "navigation-targets"
    When I evaluate whether workflow "Lesson 03" may advance
    Then modeling is blocked with "clean review is missing category 'navigation-targets'"

  Scenario: Mandatory findings are associated with the model digest that produced them
    Given workflow "Lesson 03" has review status "changes_requested"
    And mandatory finding "bad board lane" is associated with model digest "abc123"
    And the current model digest is "abc123"
    When I evaluate whether workflow "Lesson 03" may advance
    Then modeling is blocked with "mandatory review findings remain for current model digest"

  Scenario: Empty review output is not a clean review
    Given no structured review record exists for workflow "Lesson 03"
    When I evaluate whether workflow "Lesson 03" may advance
    Then the workflow review is treated as failed

  Scenario: Validator success alone is not a clean review
    Given workflow "Lesson 03" passes mechanical validation
    And no structured clean review record exists for workflow "Lesson 03"
    When I evaluate whether workflow "Lesson 03" may advance
    Then modeling is blocked with "workflow review is not clean"

  Scenario: Bare clean markers are rejected when required review categories are absent
    Given a structured review record for workflow "Lesson 03" has status "clean"
    And the review record does not include all required clean-review category markers
    When I evaluate whether workflow "Lesson 03" may advance
    Then the workflow review is treated as failed

  Scenario: Required review categories must be clean
    Given a structured review record for workflow "Lesson 03" has status "clean"
    And the review record includes every required clean-review category marker
    But category result "source-chains" is "changes_requested"
    When I evaluate whether workflow "Lesson 03" may advance
    Then modeling is blocked with "review category 'source-chains' is not clean"

  Scenario: A workflow cannot advance while mandatory review findings remain
    Given workflow "Lesson 03" has mandatory review findings
    When I request modeling for workflow "Lesson 04"
    Then modeling is blocked with "previous workflow review is not clean"

  Scenario: Review findings require another review after correction
    Given workflow "Lesson 03" had mandatory review findings
    And the workflow and slice model files changed to address those findings
    And the current model digest changed after the model file corrections
    But no follow-up structured clean review record exists for the new model digest
    When I evaluate whether workflow "Lesson 03" may advance
    Then modeling is blocked with "corrected workflow requires clean follow-up review"
