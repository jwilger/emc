Feature: Event model Gherkin suites are executable by dedicated Cucumber runners

  The validator/browser/review-gate specifications are not documentation-only.
  They need a dedicated Cucumber runner and step definitions instead of being
  accidentally ignored by the existing first-launch setup harness.

  Background:
    Given this runner meta-check feature is outside the event-model validator, browser, and review-gate feature directories

  Scenario: Runner meta-check feature is registered without recursive execution
    Given the repository contains feature file "tests/features/event_model_cucumber_execution.feature"
    When I list feature paths configured for the event-model Cucumber meta-check runner
    Then the configured feature paths include "tests/features/event_model_cucumber_execution.feature"
    And listing feature paths does not execute scenarios
    And the configured execution command reports undefined, pending, or skipped steps as failures when it runs the listed features

  Scenario: Validator feature runner discovers validator feature files
    Given the repository contains feature files under "tests/features/event_model_validator"
    When I run the event-model validator Cucumber runner
    Then the runner discovers every "*.feature" file under "tests/features/event_model_validator"
    And the runner attempts every discovered scenario under "tests/features/event_model_validator"
    And the runner reports undefined, pending, or skipped steps as failures

  Scenario: Browser feature runner discovers browser feature files
    Given the repository contains feature files under "tests/features/event_model_browser"
    When I run the event-model browser Cucumber runner
    Then the runner discovers every "*.feature" file under "tests/features/event_model_browser"
    And the runner attempts every discovered scenario under "tests/features/event_model_browser"
    And the runner reports undefined, pending, or skipped steps as failures

  Scenario: Review-gate feature runner discovers review-gate feature files
    Given the repository contains feature files under "tests/features/event_model_review_gate"
    When I run the event-model review-gate Cucumber runner
    Then the runner discovers every "*.feature" file under "tests/features/event_model_review_gate"
    And the runner attempts every discovered scenario under "tests/features/event_model_review_gate"
    And the runner reports undefined, pending, or skipped steps as failures

  Scenario: Legacy TUI acceptance runner does not own event-model feature suites
    Given the legacy TUI acceptance runner reads "tests/features/first_launch_setup.feature"
    When I list the event-model feature suites
    Then the legacy Rust-native acceptance runner is absent from event-model feature suite configuration
    And the event-model validator features require a TypeScript/Node runner path
    And the event-model browser features require a TypeScript/Node runner path
    And the event-model review-gate features require a TypeScript/Node runner path
    And the event-model Cucumber meta-check feature requires a TypeScript/Node runner path

  Scenario: Retired Rust-native first-launch harness artifacts are absent
    Given the repository retirement check targets the Rust-native first-launch Cucumber harness artifacts
    Then the retired first-launch Rust Cucumber example target "first_launch_setup_acceptance" is absent from "Cargo.toml"
    And the retired first-launch Rust Cucumber example file "examples/first_launch_setup_acceptance.rs" is absent
    And the retired legacy first-launch feature file "tests/features/first_launch_setup.feature" is absent
    And the Cargo dev dependencies used only by that harness are absent: "anyhow", "cucumber", "portable-pty", "tempfile", "tokio", and "vt100"
