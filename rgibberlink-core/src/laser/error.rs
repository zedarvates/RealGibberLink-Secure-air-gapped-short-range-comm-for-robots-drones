//! Laser error types and definitions

use thiserror::Error;

#[derive(Debug, Clone, thiserror::Error)]
pub enum LaserError {
    #[error("Hardware not available")]
    HardwareUnavailable,
    #[error("Invalid modulation scheme")]
    InvalidModulation,
    #[error("Transmission failed")]
    TransmissionFailed,
    #[error("Reception failed")]
    ReceptionFailed,
    #[error("Safety violation")]
    SafetyViolation,
    #[error("Alignment lost")]
    AlignmentLost,
    #[error("Data corruption")]
    DataCorruption,
    #[error("Timeout")]
    Timeout,
    #[error("Visual engine error: {0}")]
    VisualError(#[from] crate::visual::VisualError),
}