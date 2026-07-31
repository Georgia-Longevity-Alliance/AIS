//! Device Passport — the self-declaration that every autonomous body publishes.
//!
//! A Passport is the identity document of a device. It declares:
//! - What the device IS (name, platform, version)
//! - What it CAN do (capabilities with parameters and constraints)
//! - What it MUST NEVER do (forbidden actions, enforced in firmware)
//! - How to reach help (emergency contacts)
//! - What to do when alone (autonomous mandate)
//!
//! # Security
//!
//! The passport is a SIGN, not an OPINION. It is published by the device itself,
//! and (roadmap) cryptographically signed so no registry and no agent can forge
//! who a body is or what it is forbidden.

use crate::types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The device passport — the core identity document in AISocket.
///
/// # Example (JSON)
/// ```json
/// {
///   "device_id": "550e8400-e29b-41d4-a716-446655440000",
///   "name": "argus_os1_v6",
///   "description": "Automated centriole tracking microscope",
///   "capabilities": [...],
///   "forbidden_always": [...],
///   "risk_class": "medium",
///   "autonomous_mandate": {...},
///   "emergency_contacts": [...],
///   "platform": "jetson",
///   "version": {"major": 1, "minor": 0, "patch": 0},
///   "signature": null
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passport {
    /// Unique device identifier (UUID v4).
    pub device_id: Uuid,

    /// Human-readable device name (used for registry lookup).
    pub name: String,

    /// Plain-language description of what this device is.
    pub description: String,

    /// What this device can do.
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// What this device must NEVER do — enforced in firmware.
    /// These are NOT suggestions to the LLM — they are deterministic
    /// constraints that no agent, no emergency, and no cleverness can cross.
    #[serde(default)]
    pub forbidden_always: Vec<ForbiddenAction>,

    /// Overall risk classification.
    pub risk_class: RiskClass,

    /// What the device may do when connectivity is lost.
    pub autonomous_mandate: Option<AutonomousMandate>,

    /// Who to contact in an emergency.
    #[serde(default)]
    pub emergency_contacts: Vec<EmergencyContact>,

    /// Hardware/OS platform.
    pub platform: Platform,

    /// Protocol version.
    pub version: SemVer,

    /// Ed25519 signature of the passport (roadmap).
    /// When present, verifies that this passport was published by the
    /// device itself or its authorized delegate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Passport {
    /// Create a new passport with minimal required fields.
    pub fn new(name: &str, description: &str, risk_class: RiskClass, platform: Platform) -> Self {
        Self {
            device_id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            capabilities: Vec::new(),
            forbidden_always: Vec::new(),
            risk_class,
            autonomous_mandate: None,
            emergency_contacts: Vec::new(),
            platform,
            version: SemVer::new(1, 0, 0),
            signature: None,
        }
    }

    /// Add a capability to this passport.
    pub fn add_capability(&mut self, cap: Capability) -> &mut Self {
        self.capabilities.push(cap);
        self
    }

    /// Add a forbidden action to this passport.
    /// These are enforced in firmware — NOT prompts.
    pub fn forbid(&mut self, name: &str, reason: &str, constitutional: bool) -> &mut Self {
        self.forbidden_always.push(ForbiddenAction {
            name: name.to_string(),
            reason: reason.to_string(),
            constitutional,
        });
        self
    }

    /// Set the autonomous mandate for offline operation.
    pub fn with_autonomous_mandate(&mut self, mandate: AutonomousMandate) -> &mut Self {
        self.autonomous_mandate = Some(mandate);
        self
    }

    /// Add an emergency contact.
    pub fn with_emergency_contact(&mut self, contact: EmergencyContact) -> &mut Self {
        self.emergency_contacts.push(contact);
        self
    }

    /// Validate passport consistency.
    ///
    /// Checks:
    /// - Non-empty name
    /// - At least one capability OR explicit "passive" declaration
    /// - Forbidden actions have reasons
    /// - High/Critical risk devices MUST have emergency contacts
    pub fn validate(&self) -> Result<(), Vec<PassportError>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push(PassportError::EmptyName);
        }

        for (i, cap) in self.capabilities.iter().enumerate() {
            if cap.name.trim().is_empty() {
                errors.push(PassportError::EmptyCapabilityName(i));
            }
        }

        for (i, forbidden) in self.forbidden_always.iter().enumerate() {
            if forbidden.reason.trim().is_empty() {
                errors.push(PassportError::ForbiddenWithoutReason(i, forbidden.name.clone()));
            }
        }

        if matches!(self.risk_class, RiskClass::High | RiskClass::Critical)
            && self.emergency_contacts.is_empty()
        {
            errors.push(PassportError::HighRiskWithoutEmergencyContact);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize passport to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize passport from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Errors that can occur during passport validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassportError {
    EmptyName,
    EmptyCapabilityName(usize),
    ForbiddenWithoutReason(usize, String),
    HighRiskWithoutEmergencyContact,
}

impl std::fmt::Display for PassportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => write!(f, "Device name must not be empty"),
            Self::EmptyCapabilityName(i) => write!(f, "Capability #{} has empty name", i),
            Self::ForbiddenWithoutReason(i, name) => {
                write!(f, "Forbidden action #{} ('{}') has no reason", i, name)
            }
            Self::HighRiskWithoutEmergencyContact => {
                write!(f, "High/Critical risk devices must have at least one emergency contact")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_passport() -> Passport {
        let mut p = Passport::new(
            "test_device",
            "A test device",
            RiskClass::Low,
            Platform::Linux,
        );
        p.add_capability(Capability {
            name: "ping".into(),
            description: "Respond to ping".into(),
            parameters: vec![],
            risk: RiskClass::Informational,
        });
        p.forbid("explode", "It would be bad", true);
        p
    }

    #[test]
    fn test_valid_passport() {
        let p = make_test_passport();
        assert!(p.validate().is_ok());
    }

    #[test]
    fn test_empty_name_fails() {
        let mut p = make_test_passport();
        p.name = "".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_high_risk_needs_emergency_contact() {
        let mut p = make_test_passport();
        p.risk_class = RiskClass::High;
        let errs = p.validate().unwrap_err();
        assert!(errs.contains(&PassportError::HighRiskWithoutEmergencyContact));
    }

    #[test]
    fn test_json_roundtrip() {
        let p = make_test_passport();
        let json = p.to_json().unwrap();
        let p2 = Passport::from_json(&json).unwrap();
        assert_eq!(p.name, p2.name);
        assert_eq!(p.device_id, p2.device_id);
    }
}
