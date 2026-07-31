//! AIS Core — Autonomous Intelligence Socket protocol library.
//!
//! This crate implements the core protocols for:
//! - **AISocket**: Safe embodied AI — Passport, Body Law, Flight Recorder, Trace Network
//! - **Noepedia**: Hallucination-resistant knowledge — Delta Protocol, Event Store, Validator, Consolidator
//!
//! # Architecture
//!
//! ```text
//! Device → Passport → Body Law (6 layers) → Command Execution
//!                                      ↓ (on anomaly)
//!                              Flight Recorder → LLM → Trace Network
//!                                                         ↓
//!                                                  Noepedia (Event Store)
//! ```
//!
//! # Quick Example
//!
//! ```rust
//! use ais_core::prelude::*;
//!
//! // Create a device passport
//! let mut passport = Passport::new(
//!     "argus_os1_v6",
//!     "Automated centriole tracking microscope",
//!     RiskClass::Medium,
//!     Platform::Jetson,
//! );
//!
//! // Add capabilities and safety constraints
//! passport.add_capability(Capability {
//!     name: "acquire_z_stack".into(),
//!     description: "Acquire Z-stack with 488/561/640nm lasers".into(),
//!     parameters: vec![],
//!     risk: RiskClass::Medium,
//! });
//! passport.forbid("exceed_laser_power", "Eye and embryo safety", true);
//!
//! // Validate the passport
//! passport.validate().expect("Invalid passport");
//!
//! // Set up Body Law with firmware limits
//! let limits = FirmwareLimits {
//!     max_laser_power_mw: Some(10.0),
//!     max_temperature_c: Some(37.0),
//!     ..Default::default()
//! };
//! let body_law = BodyLaw::new(limits);
//!
//! // Flight recorder
//! let mut recorder = FlightRecorder::new(65536);
//!
//! // Trace network
//! let mut traces = TraceNetwork::new();
//! ```

pub mod body_law;
pub mod consolidator;
pub mod delta;
pub mod event_store;
pub mod flight_recorder;
pub mod passport;
pub mod renderer;
pub mod trace;
pub mod types;
pub mod validator;

/// Prelude — commonly used types for convenience.
pub mod prelude {
    pub use crate::body_law::{BodyLaw, Command, DeviceContext, FirmwareLimits};
    pub use crate::consolidator::{ConsolidatedView, Consolidator};
    pub use crate::delta::Delta;
    pub use crate::event_store::EventStore;
    pub use crate::flight_recorder::FlightRecorder;
    pub use crate::passport::Passport;
    pub use crate::renderer::{RenderFormat, Renderer};
    pub use crate::trace::TraceNetwork;
    pub use crate::types::*;
    pub use crate::validator::Validator;
}
