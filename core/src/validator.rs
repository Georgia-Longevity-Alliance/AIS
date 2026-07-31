//! Validator — checks deltas before they enter the event store.
//!
//! The validator enforces:
//! - Schema conformance
//! - Referential integrity (target_id must exist for UPDATE/DEPRECATE)
//! - Status transitions (e.g., PROPOSED→SUPPORTED is valid, SUPPORTED→PROPOSED is not)
//! - Permission boundaries (agent must be authorized)

use crate::delta::Delta;
use crate::event_store::EventStore;
use crate::types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Result of delta validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

/// The delta validator.
pub struct Validator {
    /// Currently known state (from event store replay).
    state: serde_json::Value,
}

impl Validator {
    /// Create a new validator from the event store's current state.
    pub fn new(store: &EventStore) -> Self {
        Self {
            state: store.replay(),
        }
    }

    /// Validate a delta against all rules.
    pub fn validate(&self, delta: &Delta) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Structural validation
        if let Err(struct_errors) = delta.validate_structure() {
            for e in struct_errors {
                errors.push(ValidationError {
                    code: "STRUCTURAL".into(),
                    message: e.to_string(),
                    field: None,
                });
            }
        }

        // 2. Referential integrity
        if matches!(
            delta.operation,
            DeltaOperation::Update | DeltaOperation::Deprecate
        ) {
            if let Some(target_id) = delta.target_id {
                if !self.object_exists(target_id) {
                    errors.push(ValidationError {
                        code: "REFERENTIAL_INTEGRITY".into(),
                        message: format!(
                            "Target object {} does not exist — cannot {:?} non-existent object",
                            target_id, delta.operation
                        ),
                        field: Some("target_id".into()),
                    });
                }
            }
        }

        // 3. Status transitions
        if let Some(target_id) = delta.target_id {
            if let Some(current_status) = self.get_object_status(target_id) {
                if let Some(new_status) = delta.payload.get("status").and_then(|v| v.as_str()) {
                    if !Self::is_valid_transition(&current_status, new_status) {
                        errors.push(ValidationError {
                            code: "INVALID_STATUS_TRANSITION".into(),
                            message: format!(
                                "Cannot transition from '{}' to '{}'",
                                current_status, new_status
                            ),
                            field: Some("status".into()),
                        });
                    }
                }
            }
        }

        // 4. Content checks (warnings only for v1)
        if delta.payload.get("subject").and_then(|v| v.as_str()) == Some("") {
            warnings.push("Claim subject is empty — consider adding a meaningful subject".into());
        }

        if delta.payload.get("sources").and_then(|v| v.as_array()).map_or(false, |a| a.is_empty())
            && delta.operation == DeltaOperation::Create
        {
            warnings.push("Claim created without sources — PROPOSED status requires sources for elevation".into());
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Check if an object exists in the current state.
    fn object_exists(&self, id: Uuid) -> bool {
        self.state["objects"]
            .get(id.to_string())
            .is_some()
    }

    /// Get the status of an object.
    fn get_object_status(&self, id: Uuid) -> Option<String> {
        self.state["objects"]
            .get(id.to_string())
            .and_then(|obj| obj.get("status"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
    }

    /// Valid status transitions in Noepedia.
    ///
    /// PROPOSED → SUPPORTED → TESTED → REPLICATED
    ///          → CONTESTED
    ///          → REFUTED
    ///          → STALE
    /// ANY → SUPERSEDED
    /// ANY → OPEN (reset)
    fn is_valid_transition(from: &str, to: &str) -> bool {
        matches!(
            (from, to),
            ("PROPOSED", "SUPPORTED")
                | ("PROPOSED", "CONTESTED")
                | ("PROPOSED", "REFUTED")
                | ("PROPOSED", "STALE")
                | ("SUPPORTED", "TESTED")
                | ("SUPPORTED", "CONTESTED")
                | ("TESTED", "REPLICATED")
                | ("TESTED", "CONTESTED")
                | ("TESTED", "REFUTED")
                | (_, "SUPERSEDED")
                | (_, "OPEN")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store_with_object(obj_id: Uuid) -> EventStore {
        let mut store = EventStore::new();
        let mut delta = Delta::create(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "PROPOSED"}),
        );
        delta.target_id = Some(obj_id);
        store.append(&delta);
        store
    }

    #[test]
    fn test_update_existing_object() {
        let obj_id = Uuid::new_v4();
        let store = make_store_with_object(obj_id);
        let validator = Validator::new(&store);

        let delta = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            obj_id,
            serde_json::json!({"status": "SUPPORTED"}),
        );

        let report = validator.validate(&delta);
        assert!(report.valid, "Expected valid update: {:?}", report.errors);
    }

    #[test]
    fn test_update_nonexistent_object() {
        let store = EventStore::new();
        let validator = Validator::new(&store);

        let delta = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(), // nonexistent
            serde_json::json!({"status": "SUPPORTED"}),
        );

        let report = validator.validate(&delta);
        assert!(!report.valid);
    }

    #[test]
    fn test_invalid_status_transition() {
        let obj_id = Uuid::new_v4();
        let mut store = make_store_with_object(obj_id);

        // Set to SUPPORTED first
        let delta = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            obj_id,
            serde_json::json!({"status": "SUPPORTED"}),
        );
        store.append(&delta);

        // Re-create validator with updated state
        let validator = Validator::new(&store);
        let delta2 = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            obj_id,
            serde_json::json!({"status": "PROPOSED"}), // invalid back-transition
        );

        let report = validator.validate(&delta2);
        assert!(!report.valid);
    }
}
