//! # Security Module
//!
//! Comprehensive security system for RealGibberLink with authentication, authorization,
//! and cryptographic operations.
//!
//! ## Modules
//! - `auth`: Authentication and PIN management
//! - `permissions`: Access control and authorization
//! - `policy`: Security policies and enforcement
//! - `hardware`: HSM and hardware security

pub mod auth;
pub mod permissions;
pub mod policy;
pub mod hardware;

// Re-export main types for backward compatibility
pub use auth::{Authenticator, PinManager};
pub use permissions::{PermissionManager, AccessControl};
pub use policy::{SecurityPolicy, PolicyEngine};
pub use hardware::{HardwareSecurityManager, HsmManager};

// Re-export core security types for easy access
pub use crate::security::{SecurityConfig, SecurityLevel, SecurityManager, SecurityError};