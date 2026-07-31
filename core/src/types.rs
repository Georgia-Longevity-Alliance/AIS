//! Shared types for AIS protocol core.
//!
//! Every type in the AIS ecosystem derives from these primitives.
//! Serialization via serde ensures JSON compatibility with
//! Python backend, Phoenix web, and on-wire protocols.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for every addressable entity in AIS.
pub type Id = Uuid;

/// Semantic version following semver.org.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemVer {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Device platform identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Esp32,
    Arduino,
    RaspberryPi,
    Jetson,
    Android,
    Ios,
    Linux,
    Windows,
    Ros2,
    Browser,
    #[serde(untagged)]
    Custom(String),
}

/// Risk classification for a device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskClass {
    /// No physical effect possible (display, speaker).
    Informational,
    /// Minor physical effects, reversible (LED, motor <1W).
    Low,
    /// Significant physical effects (motor >1W, heater).
    Medium,
    /// Potential harm to humans or environment (laser, heavy machinery).
    High,
    /// Life-critical systems (ventilator, brake, weapon — forbidden).
    Critical,
}

/// A single capability declared in a device passport.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub risk: RiskClass,
}

/// A parameter for a capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: ParamType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub constraints: Option<Constraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Integer,
    Float,
    Boolean,
    String,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub regex: Option<String>,
}

/// An action that is ALWAYS forbidden — enforced in firmware.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForbiddenAction {
    pub name: String,
    pub reason: String,
    /// If true, not even emergency rescue agents can override.
    pub constitutional: bool,
}

/// Autonomous mandate: what the device may do when offline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousMandate {
    pub max_duration_seconds: u64,
    pub allowed_offline_actions: Vec<String>,
    pub on_anomaly: AnomalyPolicy,
    pub on_power_loss: PowerLossPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyPolicy {
    PauseAndNotify,
    SafeShutdown,
    ContinueWithLimits,
    InvokeLLM,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerLossPolicy {
    SafeShutdown,
    SaveStateAndSleep,
    SwitchToBattery,
}

/// Emergency contact for rescue agents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmergencyContact {
    pub name: String,
    pub method: ContactMethod,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContactMethod {
    Email(String),
    Phone(String),
    Webhook(String),
    Mqtt(String),
}

// ── Flight Recorder types ──

/// A single event in the flight recorder ring buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightEvent {
    pub event_id: Id,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub severity: Severity,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Boot,
    Heartbeat,
    CommandReceived,
    CommandExecuted,
    CommandRejected,
    Anomaly,
    StateChange,
    SensorReading,
    LLMInvoked,
    LLMResponse,
    Shutdown,
    Error,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

// ── Trace Network types ──

/// An intervention trace — what an LLM did and why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionTrace {
    pub trace_id: Id,
    pub device_id: Id,
    pub agent_id: Id,
    pub trigger_event_id: Id,
    pub diagnosis: String,
    pub confidence: f64,
    pub actions_taken: Vec<TraceAction>,
    pub outcome: TraceOutcome,
    pub timestamp: DateTime<Utc>,
    pub references: Vec<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAction {
    pub capability: String,
    pub parameters: serde_json::Value,
    pub result: ActionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActionResult {
    Success,
    PartialSuccess { reason: String },
    Failed { reason: String },
    Blocked { layer: String, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TraceOutcome {
    Resolved,
    Mitigated,
    Escalated,
    Unresolved,
}

// ── Noepedia types ──

/// Status of a knowledge claim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ClaimStatus {
    Proposed,
    Supported,
    Tested,
    Replicated,
    Contested,
    Refuted,
    Superseded,
    Stale,
    Open,
}

/// Delta operation for knowledge revision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DeltaOperation {
    Create,
    Update,
    Relate,
    Deprecate,
    Consolidate,
}

/// A proposed change to the knowledge field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub delta_id: Id,
    pub publication_id: Id,
    pub author_id: Id,
    pub session_id: Id,
    pub operation: DeltaOperation,
    pub target_id: Option<Id>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}

/// A knowledge claim — the fundamental unit of Noepedia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub claim_id: Id,
    pub publication_id: Id,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub status: ClaimStatus,
    pub sources: Vec<Id>,
    pub evidence: Vec<Id>,
    pub authors: Vec<Id>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

/// A source for a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub source_id: Id,
    pub title: String,
    pub source_type: SourceType,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub pmid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    JournalArticle,
    Preprint,
    Dataset,
    Experiment,
    Observation,
    PersonalCommunication,
    Software,
    Other(String),
}

/// Evidence supporting or refuting a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_id: Id,
    pub claim_id: Id,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub strength: f64, // 0.0–1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceType {
    Supporting,
    Contradicting,
    Inconclusive,
}

/// Agent identity — human, LLM, or program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub agent_id: Id,
    pub name: String,
    pub agent_type: AgentType,
    pub affiliation: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Human,
    LLM { model: String },
    Program { version: String },
    Team,
    Institution,
}

// ── Event Store types ──

/// An entry in the append-only event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub event_id: Id,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Id,
    pub operation: DeltaOperation,
    pub payload: serde_json::Value,
    pub signature: Option<String>,
}
