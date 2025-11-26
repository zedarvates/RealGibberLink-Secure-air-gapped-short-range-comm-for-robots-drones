//! # Laser Module
//!
//! Modular laser communication system with safety monitoring and power management.
//!
//! ## Modules
//! - `control`: Core laser operations and modulation
//! - `safety`: Safety monitoring and compliance
//! - `power`: Power management and efficiency
//! - `alignment`: Beam alignment and tracking

pub mod control;
pub mod safety;
pub mod power;
pub mod alignment;

// Re-export main types for backward compatibility
pub use control::{LaserEngine, LaserConfig, LaserError, LaserType, ModulationScheme};
pub use safety::SafetyMonitor;
pub use power::{PowerManagement, PowerProfile};
pub use alignment::BeamAlignment;