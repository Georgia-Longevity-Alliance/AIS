//! Flight Recorder — ring buffer for device events.
//!
//! Every AISocket body keeps a trace of its recent life.
//! On ESP32: a ring buffer of 256 events. On an android: 65536.
//! Without a flight recorder the emergency doctor is blind;
//! with one, a rare failure becomes a readable story.

use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The flight recorder — a fixed-size ring buffer of events.
///
/// When the buffer is full, the oldest events are overwritten.
/// This ensures bounded memory usage on constrained devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightRecorder {
    /// The ring buffer.
    buffer: Vec<FlightEvent>,
    /// Maximum number of events to store.
    capacity: usize,
    /// Write position in the ring buffer.
    head: usize,
    /// Total events written (may exceed capacity).
    total_events: u64,
}

impl FlightRecorder {
    /// Create a new flight recorder with the given capacity.
    ///
    /// ```ignore
    /// let fr = FlightRecorder::new(256);  // ESP32
    /// let fr = FlightRecorder::new(65536); // Jetson
    /// ```
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be positive");
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            head: 0,
            total_events: 0,
        }
    }

    /// Record an event. If buffer is full, oldest event is overwritten.
    pub fn record(&mut self, event_type: EventType, severity: Severity, payload: serde_json::Value) {
        let event = FlightEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            severity,
            payload,
        };

        if self.buffer.len() < self.capacity {
            self.buffer.push(event);
        } else {
            self.buffer[self.head] = event;
        }
        self.head = (self.head + 1) % self.capacity;
        self.total_events += 1;
    }

    /// Record an anomaly (shortcut).
    pub fn record_anomaly(&mut self, description: &str, severity: Severity) {
        self.record(
            EventType::Anomaly,
            severity,
            serde_json::json!({"description": description}),
        );
    }

    /// Record a command rejection — critical for debugging why a command failed.
    pub fn record_rejection(&mut self, command: &str, layer: u8, reason: &str) {
        self.record(
            EventType::CommandRejected,
            Severity::Warning,
            serde_json::json!({
                "command": command,
                "rejected_at_layer": layer,
                "reason": reason
            }),
        );
    }

    /// Read the most recent N events (0 = all).
    pub fn read(&self, n: usize) -> Vec<&FlightEvent> {
        let len = self.buffer.len();
        let count = if n == 0 || n > len { len } else { n };

        // Events are in insertion order with wrap-around at head
        if len < self.capacity {
            // Buffer not yet full — simple tail slice
            let start = len.saturating_sub(count);
            self.buffer[start..].iter().collect()
        } else {
            // Buffer full — start from (head - count) % capacity
            let mut result = Vec::with_capacity(count);
            let start = (self.head + self.capacity - count) % self.capacity;
            for i in 0..count {
                let idx = (start + i) % self.capacity;
                result.push(&self.buffer[idx]);
            }
            result
        }
    }

    /// Read events since a given timestamp.
    pub fn read_since(&self, since: chrono::DateTime<Utc>) -> Vec<&FlightEvent> {
        self.buffer.iter().filter(|e| e.timestamp >= since).collect()
    }

    /// Read all events of a specific type.
    pub fn read_by_type(&self, event_type: &EventType) -> Vec<&FlightEvent> {
        self.buffer
            .iter()
            .filter(|e| e.event_type == *event_type)
            .collect()
    }

    /// Read all events of a minimum severity.
    pub fn read_by_severity(&self, min_severity: Severity) -> Vec<&FlightEvent> {
        self.buffer
            .iter()
            .filter(|e| e.severity >= min_severity)
            .collect()
    }

    /// Number of events currently stored.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Is the buffer empty?
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Total events ever recorded (may exceed capacity).
    pub fn total_events(&self) -> u64 {
        self.total_events
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.head = 0;
        self.total_events = 0;
    }

    /// Export events as JSON string (e.g., for LLM diagnostic context).
    pub fn to_llm_context(&self, max_events: usize) -> String {
        let events = self.read(max_events);
        let mut ctx = String::from("=== Flight Recorder (last ");
        ctx.push_str(&events.len().to_string());
        ctx.push_str(" events) ===\n\n");

        for event in events {
            ctx.push_str(&format!(
                "[{}] {:?} ({:?}): {}\n",
                event.timestamp.format("%H:%M:%S%.3f"),
                event.event_type,
                event.severity,
                event.payload
            ));
        }
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_read() {
        let mut fr = FlightRecorder::new(10);
        fr.record_anomaly("test anomaly", Severity::Warning);
        fr.record_anomaly("critical failure", Severity::Critical);

        assert_eq!(fr.len(), 2);
        let events = fr.read(0);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_wraparound() {
        let mut fr = FlightRecorder::new(3);
        fr.record_anomaly("1", Severity::Info);
        fr.record_anomaly("2", Severity::Info);
        fr.record_anomaly("3", Severity::Info);
        fr.record_anomaly("4", Severity::Info); // overwrites "1"

        assert_eq!(fr.len(), 3);
        assert_eq!(fr.total_events(), 4);
        let events = fr.read(0);
        // Should contain 2, 3, 4
        let descs: Vec<String> = events
            .iter()
            .map(|e| e.payload["description"].as_str().unwrap().to_string())
            .collect();
        assert!(!descs.contains(&"1".to_string()));
    }

    #[test]
    fn test_read_by_severity() {
        let mut fr = FlightRecorder::new(10);
        fr.record_anomaly("info event", Severity::Info);
        fr.record_anomaly("warning event", Severity::Warning);
        fr.record_anomaly("error event", Severity::Error);

        let critical = fr.read_by_severity(Severity::Error);
        assert_eq!(critical.len(), 1);
    }
}
