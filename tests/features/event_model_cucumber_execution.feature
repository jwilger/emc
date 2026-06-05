# Copyright 2026 John Wilger

Feature: Event model Gherkin suites are executable by dedicated Cucumber runners

  The review-gate specifications are not documentation-only.
  They need a dedicated runner and step definitions instead of being
  accidentally ignored by the existing first-launch setup harness.

  Background:
    Given this runner meta-check feature is outside the event-model review-gate feature directory

  Scenario: Runner meta-check feature is registered without recursive execution
    Given the repository contains feature file "tests/features/event_model_cucumber_execution.feature"
    When I list feature paths configured for the event-model Cucumber meta-check runner
    Then the configured feature paths include "tests/features/event_model_cucumber_execution.feature"
    And listing feature paths does not execute scenarios
    And the configured execution command reports undefined, pending, or skipped steps as failures when it runs the listed features

  Scenario: Review-gate feature runner discovers review-gate feature files
    Given the repository contains feature files under "tests/features/event_model_review_gate"
    When I run the event-model review-gate Cucumber runner
    Then the runner discovers every "*.feature" file under "tests/features/event_model_review_gate"
    And the runner attempts every discovered scenario under "tests/features/event_model_review_gate"
    And the runner reports undefined, pending, or skipped steps as failures
