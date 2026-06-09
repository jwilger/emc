// Copyright 2026 John Wilger

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use nutype::nutype;
use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::effect::ArtifactDigest;

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct ModelName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct ModelDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(
        Debug,
        Clone,
        Eq,
        PartialEq,
        Ord,
        PartialOrd,
        AsRef,
        Display,
        Serialize,
        Deserialize
    )
)]
pub struct WorkflowSlug(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct SliceSlug(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowSliceDetail {
    slug: SliceSlug,
    name: ModelName,
    kind: SliceKindName,
    description: ModelDescription,
    relationship: WorkflowStepRelationshipName,
}

impl WorkflowSliceDetail {
    pub fn new(
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
    ) -> Self {
        Self::new_with_relationship(
            slug,
            name,
            kind,
            description,
            WorkflowStepRelationshipName::Main,
        )
    }

    pub fn new_with_relationship(
        slug: SliceSlug,
        name: ModelName,
        kind: SliceKindName,
        description: ModelDescription,
        relationship: WorkflowStepRelationshipName,
    ) -> Self {
        Self {
            slug,
            name,
            kind,
            description,
            relationship,
        }
    }

    pub fn slug(&self) -> &SliceSlug {
        &self.slug
    }

    pub fn name(&self) -> &ModelName {
        &self.name
    }

    pub fn kind(&self) -> &SliceKindName {
        &self.kind
    }

    pub fn description(&self) -> &ModelDescription {
        &self.description
    }

    pub fn relationship(&self) -> &WorkflowStepRelationshipName {
        &self.relationship
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowSliceDetails {
    details: Vec<WorkflowSliceDetail>,
}

impl WorkflowSliceDetails {
    pub fn from_details(details: impl IntoIterator<Item = WorkflowSliceDetail>) -> Self {
        Self {
            details: details.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowSliceDetail] {
        &self.details
    }

    pub(crate) fn into_inner(self) -> Vec<WorkflowSliceDetail> {
        self.details
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SliceKindName {
    StateView,
    StateChange,
    Translation,
    Automation,
}

impl SliceKindName {
    pub fn try_new(value: String) -> Result<Self, SliceKindNameError> {
        match value.trim() {
            "state_view" => Ok(Self::StateView),
            "state_change" => Ok(Self::StateChange),
            "translation" => Ok(Self::Translation),
            "automation" => Ok(Self::Automation),
            _ => Err(SliceKindNameError::new(value)),
        }
    }
}

impl AsRef<str> for SliceKindName {
    fn as_ref(&self) -> &str {
        match self {
            Self::StateView => "state_view",
            Self::StateChange => "state_change",
            Self::Translation => "translation",
            Self::Automation => "automation",
        }
    }
}

impl Display for SliceKindName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for SliceKindName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for SliceKindName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceKindNameError {
    message: String,
}

impl SliceKindNameError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled slice kind, got '{value}'"),
        }
    }
}

impl Display for SliceKindNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for SliceKindNameError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct LeanModuleName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct QuintModuleName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ModelDigest(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct WorkflowTransitionEndpoint(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowStepRelationshipName {
    Entry,
    Main,
    Branch,
    Alternate,
    AsyncLifecycle,
    Supporting,
}

impl WorkflowStepRelationshipName {
    pub fn try_new(value: String) -> Result<Self, WorkflowStepRelationshipNameError> {
        match value.trim() {
            "entry" => Ok(Self::Entry),
            "main" => Ok(Self::Main),
            "branch" => Ok(Self::Branch),
            "alternate" => Ok(Self::Alternate),
            "async_lifecycle" => Ok(Self::AsyncLifecycle),
            "supporting" => Ok(Self::Supporting),
            _ => Err(WorkflowStepRelationshipNameError::new(value)),
        }
    }
}

impl AsRef<str> for WorkflowStepRelationshipName {
    fn as_ref(&self) -> &str {
        match self {
            Self::Entry => "entry",
            Self::Main => "main",
            Self::Branch => "branch",
            Self::Alternate => "alternate",
            Self::AsyncLifecycle => "async_lifecycle",
            Self::Supporting => "supporting",
        }
    }
}

impl Display for WorkflowStepRelationshipName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowStepRelationshipNameError {
    message: String,
}

impl WorkflowStepRelationshipNameError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled workflow step relationship, got '{value}'"),
        }
    }
}

impl Display for WorkflowStepRelationshipNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowStepRelationshipNameError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct WorkflowTransitionFieldName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct TransitionTriggerName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct PayloadContractName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct TranslationName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct TranslationExternalEventName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct OutcomeLabelName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ScenarioName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ScenarioStepText(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ContractKindName {
    Projector,
    Command,
    Automation,
    Translation,
    Derivation,
    Absence,
    Transitive,
}

impl ContractKindName {
    pub fn try_new(value: String) -> Result<Self, ContractKindNameError> {
        match value.trim() {
            "projector" => Ok(Self::Projector),
            "command" => Ok(Self::Command),
            "automation" => Ok(Self::Automation),
            "translation" => Ok(Self::Translation),
            "derivation" => Ok(Self::Derivation),
            "absence" => Ok(Self::Absence),
            "transitive" => Ok(Self::Transitive),
            _ => Err(ContractKindNameError::new(value)),
        }
    }
}

impl AsRef<str> for ContractKindName {
    fn as_ref(&self) -> &str {
        match self {
            Self::Projector => "projector",
            Self::Command => "command",
            Self::Automation => "automation",
            Self::Translation => "translation",
            Self::Derivation => "derivation",
            Self::Absence => "absence",
            Self::Transitive => "transitive",
        }
    }
}

impl Display for ContractKindName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContractKindNameError {
    message: String,
}

impl ContractKindNameError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled contract kind, got '{value}'"),
        }
    }
}

impl Display for ContractKindNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ContractKindNameError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CoveredDefinitionName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DatumName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DataFlowSource(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum DataFlowSourceKind {
    Original,
    ModeledTarget,
}

impl DataFlowSourceKind {
    pub fn try_new(value: String) -> Result<Self, DataFlowSourceKindError> {
        match value.trim() {
            "original" => Ok(Self::Original),
            "modeled_target" => Ok(Self::ModeledTarget),
            _ => Err(DataFlowSourceKindError::new(value)),
        }
    }
}

impl AsRef<str> for DataFlowSourceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Original => "original",
            Self::ModeledTarget => "modeled_target",
        }
    }
}

impl Display for DataFlowSourceKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DataFlowSourceKindError {
    message: String,
}

impl DataFlowSourceKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled data-flow source kind, got '{value}'"),
        }
    }
}

impl Display for DataFlowSourceKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for DataFlowSourceKindError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TransformationSemantics {
    Identity,
    Projection,
    Derivation,
    Default,
    Absence,
    Transformation,
}

impl TransformationSemantics {
    pub fn try_new(value: String) -> Result<Self, TransformationSemanticsError> {
        match value.trim() {
            "identity" => Ok(Self::Identity),
            "projection" => Ok(Self::Projection),
            "derivation" => Ok(Self::Derivation),
            "default" => Ok(Self::Default),
            "absence" => Ok(Self::Absence),
            "transformation" => Ok(Self::Transformation),
            _ => Err(TransformationSemanticsError::new(value)),
        }
    }
}

impl AsRef<str> for TransformationSemantics {
    fn as_ref(&self) -> &str {
        match self {
            Self::Identity => "identity",
            Self::Projection => "projection",
            Self::Derivation => "derivation",
            Self::Default => "default",
            Self::Absence => "absence",
            Self::Transformation => "transformation",
        }
    }
}

impl Display for TransformationSemantics {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransformationSemanticsError {
    message: String,
}

impl TransformationSemanticsError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected modeled transformation semantics, got '{value}'"),
        }
    }
}

impl Display for TransformationSemanticsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for TransformationSemanticsError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReadModelDerivationRule(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReadModelTransitiveRule(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DataFlowTarget(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BitEncodingSemantics(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BoardLaneId {
    Ux,
    Actions,
    Events,
}

impl BoardLaneId {
    pub const CANONICAL: [Self; 3] = [Self::Ux, Self::Actions, Self::Events];

    pub fn try_new(value: String) -> Result<Self, BoardLaneIdError> {
        match value.trim() {
            "ux" => Ok(Self::Ux),
            "actions" => Ok(Self::Actions),
            "events" => Ok(Self::Events),
            _ => Err(BoardLaneIdError::new(value)),
        }
    }
}

impl AsRef<str> for BoardLaneId {
    fn as_ref(&self) -> &str {
        match self {
            Self::Ux => "ux",
            Self::Actions => "actions",
            Self::Events => "events",
        }
    }
}

impl Display for BoardLaneId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoardLaneIdError {
    message: String,
}

impl BoardLaneIdError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a canonical board lane, got '{value}'"),
        }
    }
}

impl Display for BoardLaneIdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BoardLaneIdError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardElementName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BoardElementKind {
    View,
    Automation,
    ExternalEvent,
    Command,
    ReadModel,
    Event,
}

impl BoardElementKind {
    pub fn try_new(value: String) -> Result<Self, BoardElementKindError> {
        match value.trim() {
            "view" => Ok(Self::View),
            "automation" => Ok(Self::Automation),
            "external_event" => Ok(Self::ExternalEvent),
            "command" => Ok(Self::Command),
            "read_model" => Ok(Self::ReadModel),
            "event" => Ok(Self::Event),
            _ => Err(BoardElementKindError::new(value)),
        }
    }
}

impl AsRef<str> for BoardElementKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::View => "view",
            Self::Automation => "automation",
            Self::ExternalEvent => "external_event",
            Self::Command => "command",
            Self::ReadModel => "read_model",
            Self::Event => "event",
        }
    }
}

impl Display for BoardElementKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoardElementKindError {
    message: String,
}

impl BoardElementKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled board element kind, got '{value}'"),
        }
    }
}

impl Display for BoardElementKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BoardElementKindError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardElementDeclaredName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct BoardConnectionEndpoint(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BoardConnectionEndpointKind {
    View,
    Automation,
    ExternalEvent,
    WorkflowTrigger,
    Command,
    Event,
    ReadModel,
}

impl BoardConnectionEndpointKind {
    pub fn try_new(value: String) -> Result<Self, BoardConnectionEndpointKindError> {
        match value.trim() {
            "view" => Ok(Self::View),
            "automation" => Ok(Self::Automation),
            "external_event" => Ok(Self::ExternalEvent),
            "workflow_trigger" => Ok(Self::WorkflowTrigger),
            "command" => Ok(Self::Command),
            "event" => Ok(Self::Event),
            "read_model" => Ok(Self::ReadModel),
            _ => Err(BoardConnectionEndpointKindError::new(value)),
        }
    }
}

impl AsRef<str> for BoardConnectionEndpointKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::View => "view",
            Self::Automation => "automation",
            Self::ExternalEvent => "external_event",
            Self::WorkflowTrigger => "workflow_trigger",
            Self::Command => "command",
            Self::Event => "event",
            Self::ReadModel => "read_model",
        }
    }
}

impl Display for BoardConnectionEndpointKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoardConnectionEndpointKindError {
    message: String,
}

impl BoardConnectionEndpointKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled board connection endpoint kind, got '{value}'"),
        }
    }
}

impl Display for BoardConnectionEndpointKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for BoardConnectionEndpointKindError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowTransitionKind {
    Command,
    Event,
    Navigation,
    ExternalTrigger,
    Outcome,
    WorkflowExitCommand,
    WorkflowExitEvent,
    WorkflowExitNavigation,
    WorkflowExitExternalTrigger,
    WorkflowExitOutcome,
}

impl WorkflowTransitionKind {
    pub fn try_new(value: String) -> Result<Self, WorkflowTransitionKindError> {
        match value.trim() {
            "command" => Ok(Self::Command),
            "event" => Ok(Self::Event),
            "navigation" => Ok(Self::Navigation),
            "external_trigger" => Ok(Self::ExternalTrigger),
            "outcome" => Ok(Self::Outcome),
            "workflow_exit:command" => Ok(Self::WorkflowExitCommand),
            "workflow_exit:event" => Ok(Self::WorkflowExitEvent),
            "workflow_exit:navigation" => Ok(Self::WorkflowExitNavigation),
            "workflow_exit:external_trigger" => Ok(Self::WorkflowExitExternalTrigger),
            "workflow_exit:outcome" => Ok(Self::WorkflowExitOutcome),
            _ => Err(WorkflowTransitionKindError::new(value)),
        }
    }

    pub fn is_workflow_exit(self) -> bool {
        matches!(
            self,
            Self::WorkflowExitCommand
                | Self::WorkflowExitEvent
                | Self::WorkflowExitNavigation
                | Self::WorkflowExitExternalTrigger
                | Self::WorkflowExitOutcome
        )
    }
}

impl AsRef<str> for WorkflowTransitionKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Command => "command",
            Self::Event => "event",
            Self::Navigation => "navigation",
            Self::ExternalTrigger => "external_trigger",
            Self::Outcome => "outcome",
            Self::WorkflowExitCommand => "workflow_exit:command",
            Self::WorkflowExitEvent => "workflow_exit:event",
            Self::WorkflowExitNavigation => "workflow_exit:navigation",
            Self::WorkflowExitExternalTrigger => "workflow_exit:external_trigger",
            Self::WorkflowExitOutcome => "workflow_exit:outcome",
        }
    }
}

impl Display for WorkflowTransitionKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for WorkflowTransitionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for WorkflowTransitionKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionKindError {
    message: String,
}

impl WorkflowTransitionKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled workflow transition kind, got '{value}'"),
        }
    }
}

impl Display for WorkflowTransitionKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowTransitionKindError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct WorkflowTransitionSourceEvidenceText(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct WorkflowTransitionTargetEvidenceText(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowEntryLifecycleStateName {
    FreshUninitialized,
    InitializedUnauthenticated,
    InitializedAuthenticated,
    PartiallyConfigured,
    FullyConfigured,
}

impl WorkflowEntryLifecycleStateName {
    pub const REQUIRED: [Self; 5] = [
        Self::FreshUninitialized,
        Self::InitializedUnauthenticated,
        Self::InitializedAuthenticated,
        Self::PartiallyConfigured,
        Self::FullyConfigured,
    ];

    pub fn try_new(value: String) -> Result<Self, WorkflowEntryLifecycleStateNameError> {
        match value.trim() {
            "fresh_uninitialized" => Ok(Self::FreshUninitialized),
            "initialized_unauthenticated" => Ok(Self::InitializedUnauthenticated),
            "initialized_authenticated" => Ok(Self::InitializedAuthenticated),
            "partially_configured" => Ok(Self::PartiallyConfigured),
            "fully_configured" => Ok(Self::FullyConfigured),
            _ => Err(WorkflowEntryLifecycleStateNameError::new(value)),
        }
    }
}

impl AsRef<str> for WorkflowEntryLifecycleStateName {
    fn as_ref(&self) -> &str {
        match self {
            Self::FreshUninitialized => "fresh_uninitialized",
            Self::InitializedUnauthenticated => "initialized_unauthenticated",
            Self::InitializedAuthenticated => "initialized_authenticated",
            Self::PartiallyConfigured => "partially_configured",
            Self::FullyConfigured => "fully_configured",
        }
    }
}

impl Display for WorkflowEntryLifecycleStateName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for WorkflowEntryLifecycleStateName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for WorkflowEntryLifecycleStateName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowEntryLifecycleStateNameError {
    message: String,
}

impl WorkflowEntryLifecycleStateNameError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled workflow entry lifecycle state, got '{value}'"),
        }
    }
}

impl Display for WorkflowEntryLifecycleStateNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowEntryLifecycleStateNameError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct WorkflowEntryLifecycleEvidenceText(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionRecord {
    source: WorkflowTransitionEndpoint,
    target: WorkflowTransitionEndpoint,
    kind: WorkflowTransitionKind,
    trigger: TransitionTriggerName,
    rationale: Option<ModelDescription>,
    payload_contract: Option<PayloadContractName>,
}

impl WorkflowTransitionRecord {
    pub fn new(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
            rationale: None,
            payload_contract: None,
        }
    }

    pub fn new_with_rationale(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
        rationale: ModelDescription,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
            rationale: Some(rationale),
            payload_contract: None,
        }
    }

    pub fn new_with_payload_contract(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
        payload_contract: PayloadContractName,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
            rationale: None,
            payload_contract: Some(payload_contract),
        }
    }

    pub fn source(&self) -> &WorkflowTransitionEndpoint {
        &self.source
    }

    pub fn target(&self) -> &WorkflowTransitionEndpoint {
        &self.target
    }

    pub fn kind(&self) -> &WorkflowTransitionKind {
        &self.kind
    }

    pub fn trigger(&self) -> &TransitionTriggerName {
        &self.trigger
    }

    pub fn rationale(&self) -> Option<&ModelDescription> {
        self.rationale.as_ref()
    }

    pub fn payload_contract(&self) -> Option<&PayloadContractName> {
        self.payload_contract.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionRecords {
    records: Vec<WorkflowTransitionRecord>,
}

impl WorkflowTransitionRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowTransitionRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowTransitionRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOutcomeRecord {
    source_slice: WorkflowTransitionEndpoint,
    label: OutcomeLabelName,
    externally_relevant: bool,
}

impl WorkflowOutcomeRecord {
    pub fn new(
        source_slice: WorkflowTransitionEndpoint,
        label: OutcomeLabelName,
        externally_relevant: bool,
    ) -> Self {
        Self {
            source_slice,
            label,
            externally_relevant,
        }
    }

    pub fn source_slice(&self) -> &WorkflowTransitionEndpoint {
        &self.source_slice
    }

    pub fn label(&self) -> &OutcomeLabelName {
        &self.label
    }

    pub fn externally_relevant(&self) -> bool {
        self.externally_relevant
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOutcomeRecords {
    records: Vec<WorkflowOutcomeRecord>,
}

impl WorkflowOutcomeRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowOutcomeRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowOutcomeRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowCommandErrorRecord {
    source_slice: WorkflowTransitionEndpoint,
    command_name: CommandName,
    error_name: CommandErrorName,
}

impl WorkflowCommandErrorRecord {
    pub fn new(
        source_slice: WorkflowTransitionEndpoint,
        command_name: CommandName,
        error_name: CommandErrorName,
    ) -> Self {
        Self {
            source_slice,
            command_name,
            error_name,
        }
    }

    pub fn source_slice(&self) -> &WorkflowTransitionEndpoint {
        &self.source_slice
    }

    pub fn command_name(&self) -> &CommandName {
        &self.command_name
    }

    pub fn error_name(&self) -> &CommandErrorName {
        &self.error_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowCommandErrorRecords {
    records: Vec<WorkflowCommandErrorRecord>,
}

impl WorkflowCommandErrorRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowCommandErrorRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowCommandErrorRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowOwnedDefinitionKind {
    Command,
    Event,
    View,
    Control,
    ReadModel,
    Outcome,
    Error,
    Automation,
    Translation,
    ExternalPayload,
}

impl WorkflowOwnedDefinitionKind {
    pub fn try_new(value: String) -> Result<Self, WorkflowOwnedDefinitionKindError> {
        match value.trim() {
            "command" => Ok(Self::Command),
            "event" => Ok(Self::Event),
            "view" => Ok(Self::View),
            "control" => Ok(Self::Control),
            "read_model" => Ok(Self::ReadModel),
            "outcome" => Ok(Self::Outcome),
            "error" => Ok(Self::Error),
            "automation" => Ok(Self::Automation),
            "translation" => Ok(Self::Translation),
            "external_payload" => Ok(Self::ExternalPayload),
            _ => Err(WorkflowOwnedDefinitionKindError::new(value)),
        }
    }

    pub fn is_view(self) -> bool {
        matches!(self, Self::View)
    }
}

impl AsRef<str> for WorkflowOwnedDefinitionKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Command => "command",
            Self::Event => "event",
            Self::View => "view",
            Self::Control => "control",
            Self::ReadModel => "read_model",
            Self::Outcome => "outcome",
            Self::Error => "error",
            Self::Automation => "automation",
            Self::Translation => "translation",
            Self::ExternalPayload => "external_payload",
        }
    }
}

impl Display for WorkflowOwnedDefinitionKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for WorkflowOwnedDefinitionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for WorkflowOwnedDefinitionKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOwnedDefinitionKindError {
    message: String,
}

impl WorkflowOwnedDefinitionKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled workflow owned-definition kind, got '{value}'"),
        }
    }
}

impl Display for WorkflowOwnedDefinitionKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowOwnedDefinitionKindError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct WorkflowOwnedDefinitionName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowEventParticipation {
    Emitted,
    Observed,
}

impl WorkflowEventParticipation {
    pub fn try_new(value: String) -> Result<Self, WorkflowEventParticipationError> {
        match value.trim() {
            "emitted" => Ok(Self::Emitted),
            "observed" => Ok(Self::Observed),
            _ => Err(WorkflowEventParticipationError::new(value)),
        }
    }
}

impl AsRef<str> for WorkflowEventParticipation {
    fn as_ref(&self) -> &str {
        match self {
            Self::Emitted => "emitted",
            Self::Observed => "observed",
        }
    }
}

impl Display for WorkflowEventParticipation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for WorkflowEventParticipation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for WorkflowEventParticipation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowEventParticipationError {
    message: String,
}

impl WorkflowEventParticipationError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled workflow event participation, got '{value}'"),
        }
    }
}

impl Display for WorkflowEventParticipationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowEventParticipationError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowViewRole {
    Entry,
}

impl WorkflowViewRole {
    pub fn try_new(value: String) -> Result<Self, WorkflowViewRoleError> {
        match value.trim() {
            "entry" => Ok(Self::Entry),
            _ => Err(WorkflowViewRoleError::new(value)),
        }
    }
}

impl AsRef<str> for WorkflowViewRole {
    fn as_ref(&self) -> &str {
        match self {
            Self::Entry => "entry",
        }
    }
}

impl Display for WorkflowViewRole {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for WorkflowViewRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for WorkflowViewRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowViewRoleError {
    message: String,
}

impl WorkflowViewRoleError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled workflow view role, got '{value}'"),
        }
    }
}

impl Display for WorkflowViewRoleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for WorkflowViewRoleError {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOwnedDefinitionRecord {
    source_slice: WorkflowTransitionEndpoint,
    definition_kind: WorkflowOwnedDefinitionKind,
    definition_name: WorkflowOwnedDefinitionName,
    definition_stream: Option<StreamName>,
    source_provenance: Option<ModelDescription>,
    event_participation: Option<WorkflowEventParticipation>,
    view_role: Option<WorkflowViewRole>,
}

impl WorkflowOwnedDefinitionRecord {
    pub fn new(
        source_slice: WorkflowTransitionEndpoint,
        definition_kind: WorkflowOwnedDefinitionKind,
        definition_name: WorkflowOwnedDefinitionName,
    ) -> Self {
        Self {
            source_slice,
            definition_kind,
            definition_name,
            definition_stream: None,
            source_provenance: None,
            event_participation: None,
            view_role: None,
        }
    }

    pub fn new_with_view_role(
        source_slice: WorkflowTransitionEndpoint,
        definition_kind: WorkflowOwnedDefinitionKind,
        definition_name: WorkflowOwnedDefinitionName,
        view_role: WorkflowViewRole,
    ) -> Option<Self> {
        if !definition_kind.is_view() {
            return None;
        }
        Some(Self {
            source_slice,
            definition_kind,
            definition_name,
            definition_stream: None,
            source_provenance: None,
            event_participation: None,
            view_role: Some(view_role),
        })
    }

    pub fn new_with_event_identity(
        source_slice: WorkflowTransitionEndpoint,
        definition_kind: WorkflowOwnedDefinitionKind,
        definition_name: WorkflowOwnedDefinitionName,
        definition_stream: StreamName,
        source_provenance: ModelDescription,
    ) -> Self {
        Self {
            source_slice,
            definition_kind,
            definition_name,
            definition_stream: Some(definition_stream),
            source_provenance: Some(source_provenance),
            event_participation: None,
            view_role: None,
        }
    }

    pub fn new_with_event_identity_and_participation(
        source_slice: WorkflowTransitionEndpoint,
        definition_kind: WorkflowOwnedDefinitionKind,
        definition_name: WorkflowOwnedDefinitionName,
        definition_stream: StreamName,
        source_provenance: ModelDescription,
        event_participation: WorkflowEventParticipation,
    ) -> Self {
        Self {
            source_slice,
            definition_kind,
            definition_name,
            definition_stream: Some(definition_stream),
            source_provenance: Some(source_provenance),
            event_participation: Some(event_participation),
            view_role: None,
        }
    }

    pub fn source_slice(&self) -> &WorkflowTransitionEndpoint {
        &self.source_slice
    }

    pub fn definition_kind(&self) -> &WorkflowOwnedDefinitionKind {
        &self.definition_kind
    }

    pub fn definition_name(&self) -> &WorkflowOwnedDefinitionName {
        &self.definition_name
    }

    pub fn definition_stream(&self) -> Option<&StreamName> {
        self.definition_stream.as_ref()
    }

    pub fn source_provenance(&self) -> Option<&ModelDescription> {
        self.source_provenance.as_ref()
    }

    pub fn event_participation(&self) -> Option<&WorkflowEventParticipation> {
        self.event_participation.as_ref()
    }

    pub fn view_role(&self) -> Option<&WorkflowViewRole> {
        self.view_role.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowOwnedDefinitionRecords {
    records: Vec<WorkflowOwnedDefinitionRecord>,
}

impl WorkflowOwnedDefinitionRecords {
    pub fn from_records(records: impl IntoIterator<Item = WorkflowOwnedDefinitionRecord>) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowOwnedDefinitionRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionEvidenceRecord {
    source: WorkflowTransitionEndpoint,
    target: WorkflowTransitionEndpoint,
    kind: WorkflowTransitionKind,
    trigger: TransitionTriggerName,
    source_evidence: WorkflowTransitionSourceEvidenceText,
    target_evidence: WorkflowTransitionTargetEvidenceText,
}

impl WorkflowTransitionEvidenceRecord {
    pub fn new(
        source: WorkflowTransitionEndpoint,
        target: WorkflowTransitionEndpoint,
        kind: WorkflowTransitionKind,
        trigger: TransitionTriggerName,
        source_evidence: WorkflowTransitionSourceEvidenceText,
        target_evidence: WorkflowTransitionTargetEvidenceText,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            trigger,
            source_evidence,
            target_evidence,
        }
    }

    pub fn source(&self) -> &WorkflowTransitionEndpoint {
        &self.source
    }

    pub fn target(&self) -> &WorkflowTransitionEndpoint {
        &self.target
    }

    pub fn kind(&self) -> &WorkflowTransitionKind {
        &self.kind
    }

    pub fn trigger(&self) -> &TransitionTriggerName {
        &self.trigger
    }

    pub fn source_evidence(&self) -> &WorkflowTransitionSourceEvidenceText {
        &self.source_evidence
    }

    pub fn target_evidence(&self) -> &WorkflowTransitionTargetEvidenceText {
        &self.target_evidence
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowTransitionEvidenceRecords {
    records: Vec<WorkflowTransitionEvidenceRecord>,
}

impl WorkflowTransitionEvidenceRecords {
    pub fn from_records(
        records: impl IntoIterator<Item = WorkflowTransitionEvidenceRecord>,
    ) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowTransitionEvidenceRecord] {
        &self.records
    }
}

impl Default for WorkflowTransitionEvidenceRecords {
    fn default() -> Self {
        Self::from_records([])
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowEntryLifecycleStateRecord {
    state: WorkflowEntryLifecycleStateName,
    step: WorkflowTransitionEndpoint,
    evidence: WorkflowEntryLifecycleEvidenceText,
}

impl WorkflowEntryLifecycleStateRecord {
    pub fn new(
        state: WorkflowEntryLifecycleStateName,
        step: WorkflowTransitionEndpoint,
        evidence: WorkflowEntryLifecycleEvidenceText,
    ) -> Self {
        Self {
            state,
            step,
            evidence,
        }
    }

    pub fn state(&self) -> &WorkflowEntryLifecycleStateName {
        &self.state
    }

    pub fn step(&self) -> &WorkflowTransitionEndpoint {
        &self.step
    }

    pub fn evidence(&self) -> &WorkflowEntryLifecycleEvidenceText {
        &self.evidence
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowEntryLifecycleStateRecords {
    records: Vec<WorkflowEntryLifecycleStateRecord>,
}

impl WorkflowEntryLifecycleStateRecords {
    pub fn from_records(
        records: impl IntoIterator<Item = WorkflowEntryLifecycleStateRecord>,
    ) -> Self {
        Self {
            records: records.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[WorkflowEntryLifecycleStateRecord] {
        &self.records
    }
}

impl Default for WorkflowEntryLifecycleStateRecords {
    fn default() -> Self {
        Self::from_records([])
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkflowModuleData {
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_slice_details: WorkflowSliceDetails,
    workflow_transitions: WorkflowTransitionRecords,
    workflow_outcomes: WorkflowOutcomeRecords,
    workflow_command_errors: WorkflowCommandErrorRecords,
    workflow_owned_definitions: WorkflowOwnedDefinitionRecords,
    workflow_transition_evidences: WorkflowTransitionEvidenceRecords,
    workflow_requires_entry_lifecycle_coverage: bool,
    workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
    digest: ArtifactDigest,
}

impl WorkflowModuleData {
    pub fn new(
        workflow_name: ModelName,
        workflow_description: ModelDescription,
        workflow_slug: WorkflowSlug,
        digest: ArtifactDigest,
    ) -> Self {
        Self {
            workflow_name,
            workflow_description,
            workflow_slug,
            workflow_slice_details: WorkflowSliceDetails::from_details([]),
            workflow_transitions: WorkflowTransitionRecords::from_records([]),
            workflow_outcomes: WorkflowOutcomeRecords::from_records([]),
            workflow_command_errors: WorkflowCommandErrorRecords::from_records([]),
            workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records([]),
            workflow_transition_evidences: WorkflowTransitionEvidenceRecords::from_records([]),
            workflow_requires_entry_lifecycle_coverage: false,
            workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords::from_records([]),
            digest,
        }
    }

    pub fn with_slice_details(mut self, workflow_slice_details: WorkflowSliceDetails) -> Self {
        self.workflow_slice_details = workflow_slice_details;
        self
    }

    pub fn with_transitions(mut self, workflow_transitions: WorkflowTransitionRecords) -> Self {
        self.workflow_transitions = workflow_transitions;
        self
    }

    pub fn with_outcomes(mut self, workflow_outcomes: WorkflowOutcomeRecords) -> Self {
        self.workflow_outcomes = workflow_outcomes;
        self
    }

    pub fn with_command_errors(
        mut self,
        workflow_command_errors: WorkflowCommandErrorRecords,
    ) -> Self {
        self.workflow_command_errors = workflow_command_errors;
        self
    }

    pub fn with_owned_definitions(
        mut self,
        workflow_owned_definitions: WorkflowOwnedDefinitionRecords,
    ) -> Self {
        self.workflow_owned_definitions = workflow_owned_definitions;
        self
    }

    pub fn with_transition_evidences(
        mut self,
        workflow_transition_evidences: WorkflowTransitionEvidenceRecords,
    ) -> Self {
        self.workflow_transition_evidences = workflow_transition_evidences;
        self
    }

    pub fn with_entry_lifecycle_required(
        mut self,
        workflow_requires_entry_lifecycle_coverage: bool,
    ) -> Self {
        self.workflow_requires_entry_lifecycle_coverage =
            workflow_requires_entry_lifecycle_coverage;
        self
    }

    pub fn with_entry_lifecycle_states(
        mut self,
        workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
    ) -> Self {
        self.workflow_entry_lifecycle_states = workflow_entry_lifecycle_states;
        self
    }

    pub fn workflow_name(&self) -> &ModelName {
        &self.workflow_name
    }

    pub fn workflow_description(&self) -> &ModelDescription {
        &self.workflow_description
    }

    pub fn workflow_slug(&self) -> &WorkflowSlug {
        &self.workflow_slug
    }

    pub(crate) fn workflow_slice_details(&self) -> &WorkflowSliceDetails {
        &self.workflow_slice_details
    }

    pub(crate) fn workflow_transitions(&self) -> &WorkflowTransitionRecords {
        &self.workflow_transitions
    }

    pub(crate) fn workflow_outcomes(&self) -> &WorkflowOutcomeRecords {
        &self.workflow_outcomes
    }

    pub(crate) fn workflow_command_errors(&self) -> &WorkflowCommandErrorRecords {
        &self.workflow_command_errors
    }

    pub(crate) fn workflow_owned_definitions(&self) -> &WorkflowOwnedDefinitionRecords {
        &self.workflow_owned_definitions
    }

    pub(crate) fn workflow_transition_evidences(&self) -> &WorkflowTransitionEvidenceRecords {
        &self.workflow_transition_evidences
    }

    pub(crate) fn workflow_requires_entry_lifecycle_coverage(&self) -> bool {
        self.workflow_requires_entry_lifecycle_coverage
    }

    pub(crate) fn workflow_entry_lifecycle_states(&self) -> &WorkflowEntryLifecycleStateRecords {
        &self.workflow_entry_lifecycle_states
    }

    pub fn digest(&self) -> &ArtifactDigest {
        &self.digest
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReviewStatus {
    Clean,
    ChangesRequested,
}

impl ReviewStatus {
    pub fn try_new(value: String) -> Result<Self, ReviewStatusError> {
        match value.trim() {
            "clean" => Ok(Self::Clean),
            "changes_requested" => Ok(Self::ChangesRequested),
            _ => Err(ReviewStatusError::new(value)),
        }
    }
}

impl AsRef<str> for ReviewStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::Clean => "clean",
            Self::ChangesRequested => "changes_requested",
        }
    }
}

impl Display for ReviewStatus {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReviewStatusError {
    message: String,
}

impl ReviewStatusError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled review status, got '{value}'"),
        }
    }
}

impl Display for ReviewStatusError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ReviewStatusError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReviewRuleName {
    LifecycleEntry,
    CanonicalLanes,
    BoardConnections,
    FakeIntermediates,
    SliceOwnership,
    SourceChains,
    WorkflowReachability,
    TransitionResolution,
    NavigationTargets,
    BranchShape,
    OutcomesAndErrors,
    ScenarioCoverage,
    TimelineRendering,
}

impl ReviewRuleName {
    pub const REQUIRED: [Self; 13] = [
        Self::LifecycleEntry,
        Self::CanonicalLanes,
        Self::BoardConnections,
        Self::FakeIntermediates,
        Self::SliceOwnership,
        Self::SourceChains,
        Self::WorkflowReachability,
        Self::TransitionResolution,
        Self::NavigationTargets,
        Self::BranchShape,
        Self::OutcomesAndErrors,
        Self::ScenarioCoverage,
        Self::TimelineRendering,
    ];

    pub fn try_new(value: String) -> Result<Self, ReviewRuleNameError> {
        match value.trim() {
            "lifecycle-entry" => Ok(Self::LifecycleEntry),
            "canonical-lanes" => Ok(Self::CanonicalLanes),
            "board-connections" => Ok(Self::BoardConnections),
            "fake-intermediates" => Ok(Self::FakeIntermediates),
            "slice-ownership" => Ok(Self::SliceOwnership),
            "source-chains" => Ok(Self::SourceChains),
            "workflow-reachability" => Ok(Self::WorkflowReachability),
            "transition-resolution" => Ok(Self::TransitionResolution),
            "navigation-targets" => Ok(Self::NavigationTargets),
            "branch-shape" => Ok(Self::BranchShape),
            "outcomes-and-errors" => Ok(Self::OutcomesAndErrors),
            "scenario-coverage" => Ok(Self::ScenarioCoverage),
            "timeline-rendering" => Ok(Self::TimelineRendering),
            _ => Err(ReviewRuleNameError::new(value)),
        }
    }
}

impl AsRef<str> for ReviewRuleName {
    fn as_ref(&self) -> &str {
        match self {
            Self::LifecycleEntry => "lifecycle-entry",
            Self::CanonicalLanes => "canonical-lanes",
            Self::BoardConnections => "board-connections",
            Self::FakeIntermediates => "fake-intermediates",
            Self::SliceOwnership => "slice-ownership",
            Self::SourceChains => "source-chains",
            Self::WorkflowReachability => "workflow-reachability",
            Self::TransitionResolution => "transition-resolution",
            Self::NavigationTargets => "navigation-targets",
            Self::BranchShape => "branch-shape",
            Self::OutcomesAndErrors => "outcomes-and-errors",
            Self::ScenarioCoverage => "scenario-coverage",
            Self::TimelineRendering => "timeline-rendering",
        }
    }
}

impl Display for ReviewRuleName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

impl Serialize for ReviewRuleName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for ReviewRuleName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(DeserializeError::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReviewRuleNameError {
    message: String,
}

impl ReviewRuleNameError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled review category, got '{value}'"),
        }
    }
}

impl Display for ReviewRuleNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ReviewRuleNameError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct ReviewerId(String);

fn is_utc_millisecond_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 24
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'.'
        && bytes[23] == b'Z'
        && ascii_digits(bytes, 0, 4)
        && ascii_digits(bytes, 5, 2)
        && ascii_digits(bytes, 8, 2)
        && ascii_digits(bytes, 11, 2)
        && ascii_digits(bytes, 14, 2)
        && ascii_digits(bytes, 17, 2)
        && ascii_digits(bytes, 20, 3)
        && numeric_range(bytes, 5, 2, 1, 12)
        && numeric_range(bytes, 8, 2, 1, 31)
        && numeric_range(bytes, 11, 2, 0, 23)
        && numeric_range(bytes, 14, 2, 0, 59)
        && numeric_range(bytes, 17, 2, 0, 59)
}

fn ascii_digits(bytes: &[u8], start: usize, length: usize) -> bool {
    bytes[start..start + length].iter().all(u8::is_ascii_digit)
}

fn numeric_range(bytes: &[u8], start: usize, length: usize, minimum: u32, maximum: u32) -> bool {
    ascii_digits(bytes, start, length)
        && (minimum..=maximum).contains(
            &bytes[start..start + length]
                .iter()
                .fold(0_u32, |value, byte| (value * 10) + u32::from(byte - b'0')),
        )
}

#[nutype(
    sanitize(trim),
    validate(not_empty, predicate = is_utc_millisecond_timestamp),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct ReviewTimestamp(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct CommandName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct CommandErrorName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommandErrorRecoveryKind {
    Retry,
    StayOnScreen,
    Navigation,
    ExplicitRecoveryAction,
}

impl CommandErrorRecoveryKind {
    pub const ALLOWED: [Self; 4] = [
        Self::Retry,
        Self::StayOnScreen,
        Self::Navigation,
        Self::ExplicitRecoveryAction,
    ];

    pub fn try_new(value: String) -> Result<Self, CommandErrorRecoveryKindError> {
        match value.trim() {
            "retry" => Ok(Self::Retry),
            "stay_on_screen" => Ok(Self::StayOnScreen),
            "navigation" => Ok(Self::Navigation),
            "explicit_recovery_action" => Ok(Self::ExplicitRecoveryAction),
            _ => Err(CommandErrorRecoveryKindError::new(value)),
        }
    }
}

impl AsRef<str> for CommandErrorRecoveryKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Retry => "retry",
            Self::StayOnScreen => "stay_on_screen",
            Self::Navigation => "navigation",
            Self::ExplicitRecoveryAction => "explicit_recovery_action",
        }
    }
}

impl Display for CommandErrorRecoveryKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandErrorRecoveryKindError {
    message: String,
}

impl CommandErrorRecoveryKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled command error recovery kind, got '{value}'"),
        }
    }
}

impl Display for CommandErrorRecoveryKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for CommandErrorRecoveryKindError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SingletonRepeatBehavior {
    AlreadyExistsError,
    Idempotent,
}

impl SingletonRepeatBehavior {
    pub const ALLOWED: [Self; 2] = [Self::AlreadyExistsError, Self::Idempotent];

    pub fn try_new(value: String) -> Result<Self, SingletonRepeatBehaviorError> {
        match value.trim() {
            "already_exists_error" => Ok(Self::AlreadyExistsError),
            "idempotent" => Ok(Self::Idempotent),
            _ => Err(SingletonRepeatBehaviorError::new(value)),
        }
    }
}

impl AsRef<str> for SingletonRepeatBehavior {
    fn as_ref(&self) -> &str {
        match self {
            Self::AlreadyExistsError => "already_exists_error",
            Self::Idempotent => "idempotent",
        }
    }
}

impl Display for SingletonRepeatBehavior {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SingletonRepeatBehaviorError {
    message: String,
}

impl SingletonRepeatBehaviorError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled singleton repeat behavior, got '{value}'"),
        }
    }
}

impl Display for SingletonRepeatBehaviorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for SingletonRepeatBehaviorError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct AutomationName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct AutomationTriggerName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct AutomationReactionDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ControlRecoveryBehavior {
    Retry,
    StayOnScreen,
    Navigation,
    ExplicitRecoveryAction,
}

impl ControlRecoveryBehavior {
    pub fn try_new(value: String) -> Result<Self, ControlRecoveryBehaviorError> {
        match value.trim() {
            "retry" => Ok(Self::Retry),
            "stay_on_screen" => Ok(Self::StayOnScreen),
            "navigation" => Ok(Self::Navigation),
            "explicit_recovery_action" => Ok(Self::ExplicitRecoveryAction),
            _ => Err(ControlRecoveryBehaviorError::new(value)),
        }
    }
}

impl AsRef<str> for ControlRecoveryBehavior {
    fn as_ref(&self) -> &str {
        match self {
            Self::Retry => "retry",
            Self::StayOnScreen => "stay_on_screen",
            Self::Navigation => "navigation",
            Self::ExplicitRecoveryAction => "explicit_recovery_action",
        }
    }
}

impl Display for ControlRecoveryBehavior {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ControlRecoveryBehaviorError {
    message: String,
}

impl ControlRecoveryBehaviorError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled control recovery behavior, got '{value}'"),
        }
    }
}

impl Display for ControlRecoveryBehaviorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ControlRecoveryBehaviorError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NavigationTargetType {
    ModeledView,
    LocalViewState,
    ExternalSystem,
    ExternalWorkflow,
}

impl NavigationTargetType {
    pub const ALLOWED: [Self; 4] = [
        Self::ModeledView,
        Self::LocalViewState,
        Self::ExternalSystem,
        Self::ExternalWorkflow,
    ];

    pub fn try_new(value: String) -> Result<Self, NavigationTargetTypeError> {
        match value.trim() {
            "modeled_view" => Ok(Self::ModeledView),
            "local_view_state" => Ok(Self::LocalViewState),
            "external_system" => Ok(Self::ExternalSystem),
            "external_workflow" => Ok(Self::ExternalWorkflow),
            _ => Err(NavigationTargetTypeError::new(value)),
        }
    }
}

impl AsRef<str> for NavigationTargetType {
    fn as_ref(&self) -> &str {
        match self {
            Self::ModeledView => "modeled_view",
            Self::LocalViewState => "local_view_state",
            Self::ExternalSystem => "external_system",
            Self::ExternalWorkflow => "external_workflow",
        }
    }
}

impl Display for NavigationTargetType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NavigationTargetTypeError {
    message: String,
}

impl NavigationTargetTypeError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled navigation target type, got '{value}'"),
        }
    }
}

impl Display for NavigationTargetTypeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for NavigationTargetTypeError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct NavigationTargetName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum CommandInputSourceKind {
    Actor,
    Session,
    Generated,
    ExternalPayload,
    EventStreamState,
    InvocationArgument,
}

impl CommandInputSourceKind {
    pub const ALLOWED: [Self; 6] = [
        Self::Actor,
        Self::Session,
        Self::Generated,
        Self::ExternalPayload,
        Self::EventStreamState,
        Self::InvocationArgument,
    ];

    pub fn try_new(value: String) -> Result<Self, CommandInputSourceKindError> {
        match value.trim() {
            "actor" => Ok(Self::Actor),
            "session" => Ok(Self::Session),
            "generated" => Ok(Self::Generated),
            "external_payload" => Ok(Self::ExternalPayload),
            "event_stream_state" => Ok(Self::EventStreamState),
            "invocation_argument" => Ok(Self::InvocationArgument),
            _ => Err(CommandInputSourceKindError::new(value)),
        }
    }
}

impl AsRef<str> for CommandInputSourceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Actor => "actor",
            Self::Session => "session",
            Self::Generated => "generated",
            Self::ExternalPayload => "external_payload",
            Self::EventStreamState => "event_stream_state",
            Self::InvocationArgument => "invocation_argument",
        }
    }
}

impl Display for CommandInputSourceKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandInputSourceKindError {
    message: String,
}

impl CommandInputSourceKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled command input source kind, got '{value}'"),
        }
    }
}

impl Display for CommandInputSourceKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for CommandInputSourceKindError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct CommandInputSourceDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display, Serialize, Deserialize)
)]
pub struct StreamName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventAttributeName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EventAttributeSourceKind {
    CommandInput,
    ExternalPayload,
    Generated,
    Session,
    Derivation,
}

impl EventAttributeSourceKind {
    pub const ALLOWED: [Self; 5] = [
        Self::CommandInput,
        Self::ExternalPayload,
        Self::Generated,
        Self::Session,
        Self::Derivation,
    ];

    pub fn try_new(value: String) -> Result<Self, EventAttributeSourceKindError> {
        match value.trim() {
            "command_input" => Ok(Self::CommandInput),
            "external_payload" => Ok(Self::ExternalPayload),
            "generated" => Ok(Self::Generated),
            "session" => Ok(Self::Session),
            "derivation" => Ok(Self::Derivation),
            _ => Err(EventAttributeSourceKindError::new(value)),
        }
    }
}

impl AsRef<str> for EventAttributeSourceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::CommandInput => "command_input",
            Self::ExternalPayload => "external_payload",
            Self::Generated => "generated",
            Self::Session => "session",
            Self::Derivation => "derivation",
        }
    }
}

impl Display for EventAttributeSourceKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventAttributeSourceKindError {
    message: String,
}

impl EventAttributeSourceKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled event attribute source kind, got '{value}'"),
        }
    }
}

impl Display for EventAttributeSourceKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for EventAttributeSourceKindError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct GeneratedEventAttributeSourceKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventAttributeSourceName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct EventAttributeSourceField(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ProvenanceDescription(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ReadModelName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReadModelFieldSourceKind {
    EventAttribute,
    Derivation,
    AbsenceDefault,
}

impl ReadModelFieldSourceKind {
    pub const ALLOWED: [Self; 3] = [Self::EventAttribute, Self::Derivation, Self::AbsenceDefault];

    pub fn try_new(value: String) -> Result<Self, ReadModelFieldSourceKindError> {
        match value.trim() {
            "event_attribute" => Ok(Self::EventAttribute),
            "derivation" => Ok(Self::Derivation),
            "absence_default" => Ok(Self::AbsenceDefault),
            _ => Err(ReadModelFieldSourceKindError::new(value)),
        }
    }
}

impl AsRef<str> for ReadModelFieldSourceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::EventAttribute => "event_attribute",
            Self::Derivation => "derivation",
            Self::AbsenceDefault => "absence_default",
        }
    }
}

impl Display for ReadModelFieldSourceKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadModelFieldSourceKindError {
    message: String,
}

impl ReadModelFieldSourceKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled read-model field source kind, got '{value}'"),
        }
    }
}

impl Display for ReadModelFieldSourceKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ReadModelFieldSourceKindError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ViewName(String);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ViewFieldSourceKind {
    ReadModel,
}

impl ViewFieldSourceKind {
    pub const ALLOWED: [Self; 1] = [Self::ReadModel];

    pub fn try_new(value: String) -> Result<Self, ViewFieldSourceKindError> {
        match value.trim() {
            "read_model" => Ok(Self::ReadModel),
            _ => Err(ViewFieldSourceKindError::new(value)),
        }
    }
}

impl AsRef<str> for ViewFieldSourceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::ReadModel => "read_model",
        }
    }
}

impl Display for ViewFieldSourceKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewFieldSourceKindError {
    message: String,
}

impl ViewFieldSourceKindError {
    fn new(value: String) -> Self {
        Self {
            message: format!("expected a modeled view field source kind, got '{value}'"),
        }
    }
}

impl Display for ViewFieldSourceKindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for ViewFieldSourceKindError {}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SketchToken(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SliceName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SourceControlReference(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct DefinitionSectionLabel(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ViewFieldName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct SourceChainHop(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlLabel(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlEffectKind(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, Eq, PartialEq, AsRef, Display)
)]
pub struct ControlEffectTarget(String);
