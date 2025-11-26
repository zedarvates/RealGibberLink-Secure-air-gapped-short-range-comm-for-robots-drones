//! # Audit Module
//!
//! Comprehensive audit trail system for mission transfer compliance and forensic analysis.
//!
//! This module provides:
//! - Audit event recording and querying
//! - Regulatory compliance validation
//! - Security alert generation
//! - Report generation and scheduling
//! - Retention policy management

pub mod events;
pub mod compliance;

// Re-export main types for convenience
pub use events::{
    AuditSystem,
    AuditEntry,
    SecurityAlert,
    AuditEventType,
    AuditSeverity,
    AuditActor,
    AuditOperation,
    create_audit_entry,
    AuditQuery,
    ActorFilter,
    ReportRequest,
    AuditError,
};

pub use compliance::{
    ComplianceEngine,
    RegulatoryFramework,
    InternalPolicy,
    RegulatoryRequirement,
    RequirementCategory,
    PolicyControl,
    ComplianceRule,
    ComplianceAction,
    CompliancePriority,
};