Feature: Event model validator rejects structurally incomplete models and broken source chains

  The event-model JSON is the source of truth for business behavior modeling.
  Validation must reject models whose structure or provenance is not mechanically
  inspectable.

  Background:
    Given validation runs through the event-model validator CLI against temporary workflow and slice "*.eventmodel.json" files
    Given a valid state-change event model named "Open repair ticket"

  Rule: Top-level model structure is explicit and deterministic

    Scenario: Model files must contain valid JSON
      Given the event model file contains malformed JSON
      When I validate the event model file
      Then validation fails with "invalid JSON"

    Scenario: Model must be a JSON object
      Given the event model document is not a JSON object
      When I validate the event model
      Then validation fails with "model must be a JSON object"

    Scenario Outline: Required top-level sections are present
      Given the event model is missing the top-level key "<top-level key>"
      When I validate the event model
      Then validation fails with "missing top-level key '<top-level key>'"

      Examples:
        | top-level key |
        | name          |
        | version       |
        | streams       |
        | events        |
        | commands      |
        | read_models   |
        | slices        |

    Scenario: Explicit board data is required
      Given the event model has no "board" section
      When I validate the event model
      Then validation fails with "missing explicit board"

    Scenario: Top-level named definitions are unique within a file
      Given the event model defines two commands named "OpenRepairTicket"
      When I validate the event model
      Then validation fails with "duplicate command name 'OpenRepairTicket'"

    Scenario Outline: Every core top-level named definition list rejects duplicates
      Given the event model defines two <definition kind> named "duplicate_name"
      When I validate the event model
      Then validation fails with "duplicate <definition kind> name 'duplicate_name'"

      Examples:
        | definition kind |
        | stream          |
        | event           |
        | command         |
        | read model      |
        | view            |

    Scenario: Slice files contain exactly one slice
      Given a slice event model file defines two slices
      When I validate the event model
      Then validation fails with "slice file must contain exactly one slice"

    Scenario: Slice files may not be empty
      Given a slice event model file defines zero slices
      When I validate the event model
      Then validation fails with "slice file must contain exactly one slice"

    Scenario: Slice files reject legacy scenarios fields
      Given slice "Submit lesson" declares a legacy scenarios field
      When I validate the event model
      Then validation fails with "slice 'Submit lesson' uses legacy 'scenarios'; use 'acceptance_scenarios' and 'contract_scenarios'"

    Scenario: Slice files accept first-class scenario fields
      Given slice "Submit lesson" declares first-class acceptance and contract scenarios
      When I validate the event model
      Then validation succeeds

    Scenario: First-class scenario fields require Given When Then
      Given slice "Submit lesson" declares a first-class scenario without When
      When I validate the event model
      Then validation fails with "slice 'Submit lesson' scenario 'reader sees lesson' is missing 'when'"

    Scenario: Acceptance scenarios describe only user-facing behavior
      Given slice "Resolve application entry" declares an acceptance scenario that references event "RootOrganizationBootstrapped"
      When I validate the event model
      Then validation fails with "slice 'Resolve application entry' acceptance scenario 'fresh install sees bootstrap setup' references event 'RootOrganizationBootstrapped'; acceptance_scenarios must describe user-facing behavior only"

    Scenario: Slice scenario names are unique across first-class scenario fields
      Given slice "Submit lesson" declares duplicate scenario names across first-class acceptance and contract scenarios
      When I validate the event model
      Then validation fails with "slice 'Submit lesson' has duplicate scenario name 'duplicate scenario' across acceptance_scenarios and contract_scenarios"

    Scenario: Slice scenario names are unique within a slice
      Given slice "Submit lesson" defines two scenarios named "missing evidence"
      When I validate the event model
      Then validation fails with "slice 'Submit lesson' has duplicate scenario name 'missing evidence'"

    Scenario: State-view read models require projector contract scenarios
      Given state-view slice "Resolve application entry" owns read model "application_entry_state" without projector contract scenarios
      When I validate the event model
      Then validation fails with "state_view slice 'Resolve application entry' read model 'application_entry_state' requires a contract_scenarios GWT for the projector"

    Scenario: Slice outcome names are unique within a slice
      Given slice "Submit lesson" defines two outcomes named "submitted"
      When I validate the event model
      Then validation fails with "slice 'Submit lesson' has duplicate outcome label 'submitted'"

  Rule: Streams and events have valid ownership and source provenance

    Scenario: Events reference known streams
      Given event "RepairTicketOpened" references stream "missing_stream"
      When I validate the event model
      Then validation fails with "event 'RepairTicketOpened' references unknown stream 'missing_stream'"

    Scenario: Every locally emitted event is produced by a command
      Given event "RepairTicketOpened" is not produced by any local command
      And event "RepairTicketOpened" is not declared as an observed shared event
      When I validate the event model
      Then validation fails with "event 'RepairTicketOpened' is not produced by any command"

    Scenario: Command-sourced event attributes reference declared command inputs
      Given event "RepairTicketOpened" attribute "customer_name" has source "command.customer_name"
      And the command producing "RepairTicketOpened" does not declare input "customer_name"
      When I validate the event model
      Then validation fails with "event 'RepairTicketOpened' attribute 'customer_name' has invalid source 'command.customer_name'"

    Scenario: External event attributes reference declared external payloads
      Given event "PaymentRecorded" attribute "provider_payment_id" has source "external.payment_webhook.payment_id"
      And the command producing "PaymentRecorded" does not declare external input "payment_webhook"
      When I validate the event model
      Then validation fails with "event 'PaymentRecorded' attribute 'provider_payment_id' has invalid source 'external.payment_webhook.payment_id'"

    Scenario: External event attributes reference declared external payload fields
      Given command "RecordPayment" declares external input schema "payment_webhook" with field "payment_id"
      And event "PaymentRecorded" attribute "provider_status" has source "external.payment_webhook.status"
      When I validate the event model
      Then validation fails with "event 'PaymentRecorded' attribute 'provider_status' references undeclared external input field 'status'"

    Scenario: Event attributes cannot source from read models
      Given event "RepairTicketEscalated" attribute "priority" has source "read_model.repair_ticket_summary.priority"
      When I validate the event model
      Then validation fails with "event 'RepairTicketEscalated' attribute 'priority' has invalid source 'read_model.repair_ticket_summary.priority'"

    Scenario: Generated event attributes use a non-empty generated source kind
      Given event "RepairTicketOpened" attribute "repair_ticket_id" has source "generated."
      When I validate the event model
      Then validation fails with "event 'RepairTicketOpened' attribute 'repair_ticket_id' has invalid source 'generated.'"

  Rule: Read models and displayed fields trace to event facts

    Scenario: Read model fields source known event attributes
      Given read model "repair_queue" field "customer_name" has source "RepairTicketOpened.customer_name"
      And event "RepairTicketOpened" does not define attribute "customer_name"
      When I validate the event model
      Then validation fails with "read model 'repair_queue' field 'customer_name' references unknown event attribute 'RepairTicketOpened.customer_name'"

    Scenario: Read model fields must not source directly from commands
      Given read model "repair_queue" field "customer_name" has source "command.customer_name"
      When I validate the event model
      Then validation fails with "read model 'repair_queue' field 'customer_name' references unknown event attribute 'command.customer_name'"

    Scenario: Derived read model fields declare derivation provenance
      Given read model "manager_visibility" field "visible_report_count" is marked as derived
      And field "visible_report_count" has no derivation source fields or derivation description
      When I validate the event model
      Then validation fails with "derived read model field 'visible_report_count' must declare source fields and derivation"

    Scenario: Derived read model fields require derivation scenario coverage
      Given read model "manager_visibility" field "visible_report_count" is marked as derived
      And field "visible_report_count" declares source fields and a derivation description
      But no scenario demonstrates how "visible_report_count" is derived
      When I validate the event model
      Then validation fails with "derived read model field 'visible_report_count' must have a derivation scenario"

    Scenario: Transitive relationship read models declare source fields, derivation rule, and examples
      Given read model "manager_progress_visibility" claims indirect or transitive report visibility
      And field "visible_report_user_id" does not declare source relationship fields, a transitive derivation rule, and at least one derivation scenario
      When I validate the event model
      Then validation fails with "transitive read model 'manager_progress_visibility' must declare source fields, derivation rule, and scenarios"

    Scenario: Absence/default projection fields declare absence semantics
      Given read model "application_entry_state" field "is_bootstrapped" is marked as defaulted from absence
      And field "is_bootstrapped" does not say which event absence creates the default state
      When I validate the event model
      Then validation fails with "absence/default field 'is_bootstrapped' must declare the event absence it derives from"

    Scenario: Absence/default projection fields require absence scenario coverage
      Given read model "application_entry_state" field "is_bootstrapped" is marked as defaulted from absence
      And field "is_bootstrapped" declares that absence of "RootOrganizationBootstrapped" creates the default state
      But no scenario demonstrates the default state before "RootOrganizationBootstrapped" exists
      When I validate the event model
      Then validation fails with "absence/default field 'is_bootstrapped' must have an absence scenario"

    Scenario: Translation command input provenance references declared external payload fields
      Given translation command "RecordCheckpointResult" input "output_excerpt" has source "external.lesson_checkpoint_result.output_excerpt"
      And external input schema "lesson_checkpoint_result" does not declare field "output_excerpt"
      When I validate the event model
      Then validation fails with "command input 'output_excerpt' references undeclared external input field 'output_excerpt'"
