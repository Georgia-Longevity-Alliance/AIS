//! Delta Protocol — legal operations for knowledge revision.
//!
//! Knowledge in Noepedia changes through deltas — proposed changes
//! that are validated before becoming part of the consolidated view.
//! A delta is a SIGNED proposal from an agent, not a silent rewrite.

use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A delta — a proposed change to a knowledge publication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub delta_id: Uuid,
    pub publication_id: Uuid,
    pub author_id: Uuid,
    pub session_id: Uuid,
    pub operation: DeltaOperation,
    pub target_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<Utc>,
    pub signature: Option<String>,
}

impl Delta {
    /// Create a new CREATE delta.
    pub fn create(
        publication_id: Uuid,
        author_id: Uuid,
        session_id: Uuid,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            delta_id: Uuid::new_v4(),
            publication_id,
            author_id,
            session_id,
            operation: DeltaOperation::Create,
            target_id: None,
            payload,
            timestamp: Utc::now(),
            signature: None,
        }
    }

    /// Create an UPDATE delta for an existing object.
    pub fn update(
        publication_id: Uuid,
        author_id: Uuid,
        session_id: Uuid,
        target_id: Uuid,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            delta_id: Uuid::new_v4(),
            publication_id,
            author_id,
            session_id,
            operation: DeltaOperation::Update,
            target_id: Some(target_id),
            payload,
            timestamp: Utc::now(),
            signature: None,
        }
    }

    /// Create a RELATE delta linking two objects.
    pub fn relate(
        publication_id: Uuid,
        author_id: Uuid,
        session_id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
        relation: &str,
    ) -> Self {
        Self {
            delta_id: Uuid::new_v4(),
            publication_id,
            author_id,
            session_id,
            operation: DeltaOperation::Relate,
            target_id: Some(source_id),
            payload: serde_json::json!({
                "relation": relation,
                "target": target_id
            }),
            timestamp: Utc::now(),
            signature: None,
        }
    }

    /// Create a DEPRECATE delta marking an object as superseded.
    pub fn deprecate(
        publication_id: Uuid,
        author_id: Uuid,
        session_id: Uuid,
        target_id: Uuid,
        reason: &str,
        superseded_by: Option<Uuid>,
    ) -> Self {
        let mut payload = serde_json::json!({"reason": reason});
        if let Some(by) = superseded_by {
            payload["superseded_by"] = serde_json::json!(by);
        }
        Self {
            delta_id: Uuid::new_v4(),
            publication_id,
            author_id,
            session_id,
            operation: DeltaOperation::Deprecate,
            target_id: Some(target_id),
            payload,
            timestamp: Utc::now(),
            signature: None,
        }
    }

    /// Create a CONSOLIDATE delta merging multiple proposals.
    pub fn consolidate(
        publication_id: Uuid,
        author_id: Uuid,
        session_id: Uuid,
        merged_deltas: Vec<Uuid>,
        consolidated_payload: serde_json::Value,
    ) -> Self {
        Self {
            delta_id: Uuid::new_v4(),
            publication_id,
            author_id,
            session_id,
            operation: DeltaOperation::Consolidate,
            target_id: None,
            payload: serde_json::json!({
                "merged_deltas": merged_deltas,
                "result": consolidated_payload
            }),
            timestamp: Utc::now(),
            signature: None,
        }
    }

    /// Validate delta structure (not content — that's the Validator's job).
    pub fn validate_structure(&self) -> Result<(), Vec<DeltaError>> {
        let mut errors = Vec::new();

        // UPDATE, DEPRECATE require target_id
        if matches!(
            self.operation,
            DeltaOperation::Update | DeltaOperation::Deprecate
        ) && self.target_id.is_none()
        {
            errors.push(DeltaError::MissingTargetId(self.operation.clone()));
        }

        if self.publication_id.is_nil() {
            errors.push(DeltaError::NilPublicationId);
        }

        if self.author_id.is_nil() {
            errors.push(DeltaError::NilAuthorId);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaError {
    MissingTargetId(DeltaOperation),
    NilPublicationId,
    NilAuthorId,
}

impl std::fmt::Display for DeltaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTargetId(op) => {
                write!(f, "Operation {:?} requires a target_id", op)
            }
            Self::NilPublicationId => write!(f, "Publication ID must not be nil"),
            Self::NilAuthorId => write!(f, "Author ID must not be nil"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_delta() {
        let delta = Delta::create(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "subject": "test"}),
        );
        assert!(delta.validate_structure().is_ok());
        assert_eq!(delta.operation, DeltaOperation::Create);
    }

    #[test]
    fn test_update_requires_target() {
        let mut delta = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(), // target_id provided
            serde_json::json!({}),
        );
        assert!(delta.validate_structure().is_ok());

        delta.target_id = None;
        assert!(delta.validate_structure().is_err());
    }
}
