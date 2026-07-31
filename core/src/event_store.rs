//! Event Store — append-only log of all knowledge operations.
//!
//! Every delta, every publication creation, every consolidation
//! is recorded in the event store. The current state is a PROJECTION
//! of this log — the log itself is the source of truth.
//!
//! In production: PostgreSQL with WAL. For embedded/MVP: SQLite.

use crate::delta::Delta;
use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The append-only event store.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventStore {
    entries: Vec<EventLogEntry>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Append a delta to the event log.
    ///
    /// Returns the event_id of the recorded entry.
    pub fn append(&mut self, delta: &Delta) -> Uuid {
        let entry = EventLogEntry {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: delta.author_id,
            operation: delta.operation.clone(),
            payload: serde_json::to_value(delta).unwrap_or_default(),
            signature: delta.signature.clone(),
        };
        let id = entry.event_id;
        self.entries.push(entry);
        id
    }

    /// Append a raw event log entry.
    pub fn append_raw(&mut self, entry: EventLogEntry) -> Uuid {
        let id = entry.event_id;
        self.entries.push(entry);
        id
    }

    /// Get all entries for a publication, ordered by timestamp.
    pub fn get_publication_history(&self, publication_id: Uuid) -> Vec<&EventLogEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Ok(delta) = serde_json::from_value::<Delta>(e.payload.clone()) {
                    delta.publication_id == publication_id
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get entries by actor.
    pub fn get_by_actor(&self, actor_id: Uuid) -> Vec<&EventLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.actor_id == actor_id)
            .collect()
    }

    /// Get entries since a timestamp.
    pub fn get_since(&self, since: chrono::DateTime<Utc>) -> Vec<&EventLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= since)
            .collect()
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Replay all entries and build the current projection.
    /// This is the core of CQRS: event log → current state.
    pub fn replay(&self) -> serde_json::Value {
        let mut state = serde_json::json!({
            "objects": {},
            "relations": [],
            "publications": {}
        });

        for entry in &self.entries {
            if let Ok(delta) = serde_json::from_value::<Delta>(entry.payload.clone()) {
                match delta.operation {
                    DeltaOperation::Create => {
                        if let Some(obj) = state["objects"].as_object_mut() {
                            let id = delta.target_id.unwrap_or(delta.delta_id);
                            obj.insert(id.to_string(), delta.payload.clone());
                        }
                    }
                    DeltaOperation::Update => {
                        if let Some(obj) = state["objects"].as_object_mut() {
                            if let Some(target_id) = delta.target_id {
                                obj.insert(target_id.to_string(), delta.payload.clone());
                            }
                        }
                    }
                    DeltaOperation::Deprecate => {
                        if let Some(obj) = state["objects"].as_object_mut() {
                            if let Some(target_id) = delta.target_id {
                                if let Some(existing) = obj.get_mut(&target_id.to_string()) {
                                    existing["status"] = serde_json::json!("SUPERSEDED");
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_retrieve() {
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let delta = Delta::create(
            pub_id,
            author_id,
            Uuid::new_v4(),
            serde_json::json!({"claim": "test"}),
        );

        let event_id = store.append(&delta);
        assert_eq!(store.len(), 1);

        let history = store.get_publication_history(pub_id);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].event_id, event_id);
    }

    #[test]
    fn test_replay_builds_state() {
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let obj_id = Uuid::new_v4();

        let mut delta = Delta::create(
            pub_id,
            author_id,
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "PROPOSED"}),
        );
        delta.target_id = Some(obj_id);
        store.append(&delta);

        let state = store.replay();
        assert!(state["objects"][obj_id.to_string()].is_object());
    }
}
