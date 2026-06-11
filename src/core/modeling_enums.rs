// Copyright 2026 John Wilger

pub(crate) struct ModelingEnum {
    name: &'static str,
    values: &'static [&'static str],
}

impl ModelingEnum {
    pub(crate) const fn new(name: &'static str, values: &'static [&'static str]) -> Self {
        Self { name, values }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn values(&self) -> &'static [&'static str] {
        self.values
    }
}

pub(crate) const WORKFLOW_ENTRY_LIFECYCLE_STATES: &[&str] = &[
    "fresh_uninitialized",
    "initialized_unauthenticated",
    "initialized_authenticated",
    "partially_configured",
    "fully_configured",
];

pub(crate) const MODELING_ENUMS: &[ModelingEnum] = &[
    ModelingEnum::new(
        "workflow_entry_lifecycle_state",
        WORKFLOW_ENTRY_LIFECYCLE_STATES,
    ),
    ModelingEnum::new("workflow_view_role", &["entry"]),
    ModelingEnum::new(
        "slice_type",
        &["state_view", "state_change", "translation", "automation"],
    ),
    ModelingEnum::new(
        "workflow_transition_kind",
        &[
            "command",
            "event",
            "navigation",
            "external_trigger",
            "outcome",
        ],
    ),
    ModelingEnum::new(
        "definition_kind",
        &[
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
        ],
    ),
    ModelingEnum::new("event_participation", &["emitted", "observed"]),
    ModelingEnum::new(
        "command_input_source_kind",
        &[
            "actor",
            "event_stream_state",
            "external_payload",
            "generated",
            "session",
            "invocation_argument",
        ],
    ),
    ModelingEnum::new(
        "recovery_behavior",
        &[
            "retry",
            "stay_on_screen",
            "navigation",
            "explicit_recovery_action",
        ],
    ),
    ModelingEnum::new(
        "navigation_type",
        &[
            "modeled_view",
            "local_view_state",
            "external_system",
            "external_workflow",
        ],
    ),
    ModelingEnum::new("scenario_kind", &["acceptance", "contract"]),
    ModelingEnum::new(
        "contract_kind",
        &[
            "command",
            "event",
            "read_model",
            "view",
            "automation",
            "translation",
        ],
    ),
];

pub(crate) fn accepted_values(values: &[&str]) -> String {
    values.join(", ")
}
