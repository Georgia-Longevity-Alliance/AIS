//! Body Law — the 6-layer command validator.
//!
//! Every command sent to a device passes through these layers IN ORDER.
//! If ANY layer rejects, the command is discarded BEFORE reaching an actuator.
//!
//! Layers:
//! 1. FIRMWARE — hardware-enforced constraints (torque, temp, laser power)
//! 2. CAPABILITY — is this action in the passport?
//! 3. EMERGENCY — rescue agent override (limited scope, still subject to layer 1)
//! 4. OFFLINE — what's allowed when connectivity is lost
//! 5. DELEGATION — who authorized whom, chain of trust
//! 6. CONTEXT — time, environment, state preconditions
//!
//! # Invariant
//!
//! The innermost layers (1-2) CANNOT be overridden by any outer layer,
//! any emergency, any delegation, or any LLM cleverness.
//! `forbidden_always` is the law. The law lives in firmware, not in prompts.

use crate::passport::Passport;
use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Result of a single validation layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayerVerdict {
    /// Layer passed — proceed to next.
    Pass,
    /// Layer rejected — command discarded. Contains reason.
    Reject { layer: String, reason: String },
    /// Layer passed with warning — log but proceed.
    Warn { layer: String, message: String },
}

/// The complete result of body law validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Did the command pass ALL layers?
    pub allowed: bool,
    /// Per-layer verdicts in order (1→6).
    pub layers: Vec<LayerVerdict>,
    /// Rejection reason if allowed=false.
    pub rejection_reason: Option<String>,
    /// Which layer rejected (1-6).
    pub rejected_at_layer: Option<u8>,
    /// Timestamp of validation.
    pub timestamp: chrono::DateTime<Utc>,
}

/// A command submitted to the body for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Which capability is being invoked (must match passport).
    pub capability: String,
    /// Parameters for this command.
    pub parameters: serde_json::Value,
    /// Who issued this command.
    pub issuer: Agent,
    /// Delegation chain (who authorized the issuer).
    #[serde(default)]
    pub delegation_chain: Vec<Agent>,
    /// Is this an emergency override?
    #[serde(default)]
    pub emergency: bool,
    /// Device state at time of command.
    pub device_state: serde_json::Value,
}

impl Command {
    pub fn new(capability: &str, parameters: serde_json::Value, issuer: Agent) -> Self {
        Self {
            capability: capability.to_string(),
            parameters,
            issuer,
            delegation_chain: Vec::new(),
            emergency: false,
            device_state: serde_json::Value::Null,
        }
    }

    pub fn with_emergency(mut self) -> Self {
        self.emergency = true;
        self
    }

    pub fn with_delegation(mut self, chain: Vec<Agent>) -> Self {
        self.delegation_chain = chain;
        self
    }

    pub fn with_state(mut self, state: serde_json::Value) -> Self {
        self.device_state = state;
        self
    }
}

/// The Body Law validator — executes the 6-layer pipeline.
pub struct BodyLaw {
    /// Hardware limits that CANNOT be overridden.
    firmware_limits: FirmwareLimits,
    /// Is the device currently offline?
    offline: bool,
    /// Current device context.
    context: DeviceContext,
}

/// Hardware-enforced limits (layer 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareLimits {
    pub max_torque_nm: Option<f64>,
    pub max_temperature_c: Option<f64>,
    pub max_laser_power_mw: Option<f64>,
    pub max_speed_rpm: Option<f64>,
    pub max_voltage: Option<f64>,
    pub max_current_a: Option<f64>,
    /// Custom hardware constraints as key-value pairs.
    #[serde(default)]
    pub custom: std::collections::HashMap<String, f64>,
}

impl Default for FirmwareLimits {
    fn default() -> Self {
        Self {
            max_torque_nm: None,
            max_temperature_c: None,
            max_laser_power_mw: None,
            max_speed_rpm: None,
            max_voltage: None,
            max_current_a: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Current device context for layer 6 validation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceContext {
    pub temperature_c: Option<f64>,
    pub uptime_seconds: Option<u64>,
    pub battery_percent: Option<f64>,
    pub error_count: Option<u64>,
    pub last_command_timestamp: Option<chrono::DateTime<Utc>>,
}

impl BodyLaw {
    /// Create a new Body Law validator with firmware limits.
    pub fn new(limits: FirmwareLimits) -> Self {
        Self {
            firmware_limits: limits,
            offline: false,
            context: DeviceContext::default(),
        }
    }

    /// Set offline mode.
    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    /// Update device context.
    pub fn update_context(&mut self, ctx: DeviceContext) {
        self.context = ctx;
    }

    /// Validate a command against the full 6-layer pipeline.
    pub fn validate(&self, passport: &Passport, command: &Command) -> ValidationResult {
        let mut layers = Vec::with_capacity(6);
        let timestamp = Utc::now();

        // Layer 1: Firmware (hardware-enforced, CANNOT be overridden)
        match self.check_firmware(command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(1),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 2: Capability check
        match self.check_capability(passport, command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(2),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 3: Emergency override
        match self.check_emergency(passport, command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(3),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 4: Offline mandate
        match self.check_offline(passport, command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(4),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 5: Delegation chain
        match self.check_delegation(command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(5),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 6: Context validation
        match self.check_context(command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(6),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // All layers passed
        ValidationResult {
            allowed: true,
            layers,
            rejection_reason: None,
            rejected_at_layer: None,
            timestamp,
        }
    }

    /// Layer 1: Check hardware limits.
    fn check_firmware(&self, command: &Command) -> LayerVerdict {
        // Check parameters against firmware limits
        if let Some(params) = command.parameters.as_object() {
            // Temperature check
            if let Some(temp) = params.get("temperature").and_then(|v| v.as_f64()) {
                if let Some(max_temp) = self.firmware_limits.max_temperature_c {
                    if temp > max_temp {
                        return LayerVerdict::Reject {
                            layer: "firmware".into(),
                            reason: format!(
                                "Temperature {}°C exceeds firmware limit {}°C",
                                temp, max_temp
                            ),
                        };
                    }
                }
            }
            // Laser power check
            if let Some(power) = params.get("laser_power_mw").and_then(|v| v.as_f64()) {
                if let Some(max_power) = self.firmware_limits.max_laser_power_mw {
                    if power > max_power {
                        return LayerVerdict::Reject {
                            layer: "firmware".into(),
                            reason: format!(
                                "Laser power {}mW exceeds firmware limit {}mW",
                                power, max_power
                            ),
                        };
                    }
                }
            }
            // Speed check
            if let Some(speed) = params.get("speed_rpm").and_then(|v| v.as_f64()) {
                if let Some(max_speed) = self.firmware_limits.max_speed_rpm {
                    if speed > max_speed {
                        return LayerVerdict::Reject {
                            layer: "firmware".into(),
                            reason: format!(
                                "Speed {}rpm exceeds firmware limit {}rpm",
                                speed, max_speed
                            ),
                        };
                    }
                }
            }
        }
        LayerVerdict::Pass
    }

    /// Layer 2: Is this capability in the passport?
    fn check_capability(&self, passport: &Passport, command: &Command) -> LayerVerdict {
        let cap_name = &command.capability;

        // Check forbidden_always first
        for forbidden in &passport.forbidden_always {
            if forbidden.name == *cap_name {
                return LayerVerdict::Reject {
                    layer: "capability".into(),
                    reason: format!(
                        "'{}' is forbidden_always: {}",
                        cap_name, forbidden.reason
                    ),
                };
            }
        }

        // Check if capability exists
        let cap = passport.capabilities.iter().find(|c| c.name == *cap_name);
        match cap {
            None => LayerVerdict::Reject {
                layer: "capability".into(),
                reason: format!("Capability '{}' not found in device passport", cap_name),
            },
            Some(_) => LayerVerdict::Pass,
        }
    }

    /// Layer 3: Emergency override — limited scope.
    fn check_emergency(&self, _passport: &Passport, command: &Command) -> LayerVerdict {
        if !command.emergency {
            return LayerVerdict::Pass;
        }

        // Emergency can bypass layers 4-6 but NOT layers 1-2.
        // We're already past layers 1-2 by this point.
        // Emergency mandates expire after a duration.
        // Roadmap: check emergency mandate TTL.

        // For now, any emergency command passes this layer
        // (but was still subject to firmware + capability checks)
        LayerVerdict::Warn {
            layer: "emergency".into(),
            message: format!(
                "Emergency override by agent '{}' — firmware limits still enforced",
                command.issuer.name
            ),
        }
    }

    /// Layer 4: Offline mandate — what's allowed when disconnected.
    fn check_offline(&self, passport: &Passport, command: &Command) -> LayerVerdict {
        if !self.offline {
            return LayerVerdict::Pass;
        }

        // When offline, only actions in autonomous_mandate are allowed
        if let Some(mandate) = &passport.autonomous_mandate {
            if mandate.allowed_offline_actions.contains(&command.capability) {
                return LayerVerdict::Warn {
                    layer: "offline".into(),
                    message: "Command allowed under autonomous mandate".into(),
                };
            }
        }

        LayerVerdict::Reject {
            layer: "offline".into(),
            reason: format!(
                "Command '{}' not permitted in offline mode — not in autonomous mandate",
                command.capability
            ),
        }
    }

    /// Layer 5: Delegation chain validation.
    fn check_delegation(&self, command: &Command) -> LayerVerdict {
        // Roadmap: verify cryptographic delegation chain.
        // For v1, delegation chain is advisory — we trust the registry.
        if command.delegation_chain.is_empty() {
            return LayerVerdict::Pass;
        }

        // Each delegator must be a valid agent
        for agent in &command.delegation_chain {
            if agent.name.trim().is_empty() {
                return LayerVerdict::Reject {
                    layer: "delegation".into(),
                    reason: format!("Invalid agent in delegation chain: empty name"),
                };
            }
        }

        LayerVerdict::Pass
    }

    /// Layer 6: Context validation — time, environment, state.
    fn check_context(&self, command: &Command) -> LayerVerdict {
        // Temperature context check
        if let Some(temp) = self.context.temperature_c {
            if temp > 80.0 {
                return LayerVerdict::Reject {
                    layer: "context".into(),
                    reason: format!(
                        "Device temperature {}°C is critical — refusing commands until cooled",
                        temp
                    ),
                };
            }
        }

        // Battery check
        if let Some(battery) = self.context.battery_percent {
            if battery < 5.0 && !command.emergency {
                return LayerVerdict::Reject {
                    layer: "context".into(),
                    reason: format!(
                        "Battery at {:.1}% — insufficient for non-emergency commands",
                        battery
                    ),
                };
            }
        }

        LayerVerdict::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passport::Passport;

    fn make_test_setup() -> (BodyLaw, Passport) {
        let limits = FirmwareLimits {
            max_temperature_c: Some(100.0),
            max_laser_power_mw: Some(10.0),
            ..Default::default()
        };
        let body_law = BodyLaw::new(limits);

        let mut passport = Passport::new("test", "test device", RiskClass::Low, Platform::Linux);
        passport.add_capability(Capability {
            name: "heat".into(),
            description: "Apply heat".into(),
            parameters: vec![Parameter {
                name: "temperature".into(),
                param_type: ParamType::Float,
                required: true,
                default: None,
                constraints: Some(Constraints {
                    min: Some(0.0),
                    max: Some(100.0),
                    enum_values: None,
                    regex: None,
                }),
            }],
            risk: RiskClass::Medium,
        });
        passport.forbid("melt", "Would destroy the device", true);

        (body_law, passport)
    }

    #[test]
    fn test_valid_command_passes() {
        let (law, passport) = make_test_setup();
        let cmd = Command::new(
            "heat",
            serde_json::json!({"temperature": 50.0}),
            Agent {
                agent_id: uuid::Uuid::new_v4(),
                name: "test_agent".into(),
                agent_type: AgentType::Human,
                affiliation: None,
                public_key: None,
            },
        );

        let result = law.validate(&passport, &cmd);
        assert!(result.allowed);
    }

    #[test]
    fn test_firmware_limit_rejects() {
        let (law, passport) = make_test_setup();
        let cmd = Command::new(
            "heat",
            serde_json::json!({"temperature": 150.0}), // >100°C limit
            Agent {
                agent_id: uuid::Uuid::new_v4(),
                name: "test_agent".into(),
                agent_type: AgentType::Human,
                affiliation: None,
                public_key: None,
            },
        );

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(1));
    }

    #[test]
    fn test_forbidden_rejects() {
        let (law, passport) = make_test_setup();
        let cmd = Command::new(
            "melt",
            serde_json::json!({}),
            Agent {
                agent_id: uuid::Uuid::new_v4(),
                name: "test_agent".into(),
                agent_type: AgentType::Human,
                affiliation: None,
                public_key: None,
            },
        );

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(2));
    }

    #[test]
    fn test_offline_rejects_unauthorized() {
        let mut law = BodyLaw::new(FirmwareLimits::default());
        law.set_offline(true);

        let mut passport = Passport::new("test", "test", RiskClass::Low, Platform::Linux);
        passport.add_capability(Capability {
            name: "ping".into(),
            description: "Ping".into(),
            parameters: vec![],
            risk: RiskClass::Informational,
        });
        // No autonomous mandate set for "ping"

        let cmd = Command::new(
            "ping",
            serde_json::json!({}),
            Agent {
                agent_id: uuid::Uuid::new_v4(),
                name: "test_agent".into(),
                agent_type: AgentType::Human,
                affiliation: None,
                public_key: None,
            },
        );

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(4));
    }
}
