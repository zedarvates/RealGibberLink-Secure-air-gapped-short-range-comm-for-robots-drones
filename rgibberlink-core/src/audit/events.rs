//! # Audit Events Module
//!
//! Individual audit entries, event types, and audit system implementation.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use crate::mission::{MissionId, MissionPriority};
use crate::weather::{RiskLevel, ViolationSeverity};
use super::compliance::{ComplianceEngine, SecurityAlert};

/// Comprehensive audit system for drone mission operations
pub struct AuditSystem {
    audit_store: Vec<AuditEntry>,
    max_entries: usize,
    retention_policy: RetentionPolicy,
    compliance_engine: ComplianceEngine,
    report_generator: AuditReportGenerator,
    alerts: Vec<SecurityAlert>,
}

/// Individual audit entry with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub entry_id: String,
    pub timestamp: SystemTime,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub actor: AuditActor,
    pub target: Option<AuditTarget>,
    pub operation: AuditOperation,
    pub result: OperationResult,
    pub context: AuditContext,
    pub compliance_flags: Vec<ComplianceFlag>,
    pub security_metadata: SecurityMetadata,
    pub evidence: Vec<EvidenceArtifact>,
}

/// Types of auditable events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    MissionTransfer,
    SecurityAuthentication,
    AuthorizationCheck,
    WeatherValidation,
    DroneCommand,
    StationOperation,
    PolicyViolation,
    EmergencyAction,
    SystemHealthEvent,
    ComplianceAudit,
}

/// Audit severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AuditSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

/// Actor performing the audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditActor {
    HumanOperator {
        operator_id: String,
        clearance_level: String,
        department: Option<String>,
    },
    Drone {
        drone_id: String,
        model: String,
        firmware_version: String,
    },
    Station {
        station_id: String,
        location: String,
        software_version: String,
    },
    System {
        component: String,
        version: String,
        subsystem: String,
    },
    External {
        source_ip: Option<String>,
        user_agent: Option<String>,
        api_key_prefix: Option<String>,
    },
}

/// Target of the audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditTarget {
    Mission { mission_id: MissionId, priority: MissionPriority },
    Drone { drone_id: String, current_state: String },
    Station { station_id: String, current_load: u32 },
    Session { session_id: String, participants: Vec<String> },
    Policy { policy_id: String, domain: String },
    SystemComponent { component_name: String, state: String },
}

/// Description of the operation performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOperation {
    pub operation_type: String,
    pub operation_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub execution_context: OperationContext,
    pub expected_duration: Option<Duration>,
    pub resource_consumption: ResourceConsumption,
}

/// Operation execution context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationContext {
    pub security_level: String,
    pub environmental_conditions: String,
    pub system_load: f32, // 0.0 to 1.0
    pub network_quality: String,
    pub concurrent_operations: u32,
}

/// Resource consumption metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceConsumption {
    pub cpu_seconds: f32,
    pub memory_mb: f32,
    pub network_bytes: u64,
    pub storage_bytes: u64,
    pub energy_consumption_wh: f32,
}

/// Operation execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub performance_metrics: PerformanceMetrics,
    pub side_effects: Vec<String>,
}

/// Performance metrics for operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub response_time_ms: u64,
    pub throughput_items_per_sec: f32,
    pub efficiency_score: f32, // 0.0 to 1.0
    pub resource_utilization: f32, // 0.0 to 1.0
}

/// Comprehensive audit context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditContext {
    pub correlation_id: String,
    pub parent_operation_id: Option<String>,
    pub workflow_step: Option<u32>,
    pub geographic_location: Option<GeographicContext>,
    pub temporal_context: TemporalContext,
    pub business_context: BusinessContext,
    pub risk_context: RiskContext,
}

/// Geographic operation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicContext {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f32,
    pub jurisdiction: String,
    pub restricted_zone: bool,
}

/// Temporal operation context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalContext {
    pub business_hours: bool,
    pub critical_period: bool,
    pub weather_time_sensitive: bool,
    pub mission_time_pressure: Option<String>,
}

/// Business operation context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BusinessContext {
    pub operation_priority: String,
    pub regulatory_requirement: bool,
    pub commercial_impact: Option<String>,
    pub contractual_obligation: Option<String>,
}

/// Risk operation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskContext {
    pub risk_level: RiskLevel,
    pub threat_vectors: Vec<String>,
    pub mitigation_applied: Vec<String>,
    pub residual_risk: f32, // 0.0 to 1.0
}

impl Default for RiskContext {
    fn default() -> Self {
        Self {
            risk_level: RiskLevel::Low,
            threat_vectors: Vec::new(),
            mitigation_applied: Vec::new(),
            residual_risk: 0.0,
        }
    }
}

/// Compliance validation flags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplianceFlag {
    Compliant,
    Warning { message: String },
    Violation { severity: ViolationSeverity, code: String, message: String },
    Exemption { justification: String, approver: String },
}

/// Security metadata for forensic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetadata {
    pub authentication_method: String,
    pub authorization_checks: Vec<String>,
    pub cryptographic_operations: Vec<String>,
    pub security_controls_applied: Vec<String>,
    pub suspicious_indicators: Vec<String>,
    pub integrity_hash: String,
}

/// Forensic evidence artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceArtifact {
    LogEntry { source: String, level: String, message: String },
    DataBlob { content_type: String, size_bytes: usize, hash: String },
    NetworkTrace { protocol: String, source_ip: String, destination_ip: String, size_bytes: usize },
    SensorReading { sensor_type: String, value: f64, unit: String, timestamp: SystemTime },
    WeatherObservation { condition: String, severity: f32, location: String },
    PolicyReference { policy_id: String, section: String, requirement: String },
}

/// Audit retention policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_age_days: u32,
    pub max_entries: usize,
    pub compression_enabled: bool,
    pub archival_strategy: ArchivalStrategy,
    pub prioritized_events: Vec<AuditEventType>,
}

/// Archival strategies for audit data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchivalStrategy {
    None,
    CompressAfter(u32), // Days
    MoveToColdStorage(u32), // Days
    DeleteAfter(u32), // Days
}

/// Security alerts generated from audit analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub alert_id: String,
    pub timestamp: SystemTime,
    pub severity: AuditSeverity,
    pub alert_type: AlertType,
    pub title: String,
    pub description: String,
    pub affected_systems: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub evidence: Vec<EvidenceArtifact>,
    pub status: AlertStatus,
}

/// Types of security alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    PolicyViolation,
    UnauthorizedAccess,
    SuspiciousActivity,
    SystemCompromise,
    ConfigurationError,
    PerformanceAnomaly,
    ComplianceDeviation,
    EmergencyCondition,
}

/// Alert status and resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Investigating,
    Mitigated,
    Resolved,
    FalsePositive,
    Escalated,
}

impl AuditSystem {
    /// Create new audit system
    pub fn new(max_entries: usize) -> Self {
        Self {
            audit_store: Vec::new(),
            max_entries,
            retention_policy: RetentionPolicy {
                max_age_days: 365,
                max_entries: max_entries,
                compression_enabled: true,
                archival_strategy: ArchivalStrategy::CompressAfter(90),
                prioritized_events: vec![
                    AuditEventType::MissionTransfer,
                    AuditEventType::SecurityAuthentication,
                    AuditEventType::EmergencyAction,
                    AuditEventType::PolicyViolation,
                ],
            },
            compliance_engine: ComplianceEngine::new(),
            report_generator: AuditReportGenerator::new(),
            alerts: Vec::new(),
        }
    }

    /// Record audit event
    pub fn record_event(&mut self, entry: AuditEntry) -> Result<String, AuditError> {
        // Generate unique entry ID if not provided
        let entry_id = if entry.entry_id.is_empty() {
            format!("audit_{}", SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis())
        } else {
            entry.entry_id.clone()
        };

        let mut entry = entry;
        entry.entry_id = entry_id.clone();

        // Add timestamp if not set
        if entry.timestamp == SystemTime::UNIX_EPOCH {
            entry.timestamp = SystemTime::now();
        }

        // Check compliance and generate alerts first (before moving entry)
        self.compliance_engine.check_compliance(&entry, &mut self.alerts)?;

        // Store the entry
        self.audit_store.push(entry.clone());

        // Maintain size limits
        if self.audit_store.len() > self.max_entries {
            // Remove oldest entries, but keep prioritized events longer
            self.enforce_retention_policy();
        }

        Ok(entry_id)
    }

    /// Query audit trail with filters
    pub fn query_audit(&self, query: AuditQuery) -> Vec<AuditEntry> {
        self.audit_store.iter()
            .filter(|entry| self.matches_query(entry, &query))
            .cloned()
            .collect()
    }

    /// Generate audit report
    pub fn generate_report(&mut self, request: ReportRequest) -> Result<String, AuditError> {
        self.report_generator.generate_report(request, &self.audit_store)
    }

    /// Get active security alerts
    pub fn get_active_alerts(&self) -> Vec<&SecurityAlert> {
        self.alerts.iter()
            .filter(|alert| matches!(alert.status, AlertStatus::Active | AlertStatus::Investigating))
            .collect()
    }

    /// Update alert status
    pub fn update_alert_status(&mut self, alert_id: &str, new_status: AlertStatus) -> Result<(), AuditError> {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.alert_id == alert_id) {
            alert.status = new_status;
            Ok(())
        } else {
            Err(AuditError::AlertNotFound)
        }
    }

    /// Enforce retention policy
    fn enforce_retention_policy(&mut self) {
        let max_age = Duration::from_secs(self.retention_policy.max_age_days as u64 * 86400);
        let cutoff_time = SystemTime::now() - max_age;

        // Remove old entries, but preserve high-severity events longer
        self.audit_store.retain(|entry| {
            let is_recent = entry.timestamp > cutoff_time;
            let is_high_severity = matches!(entry.severity, AuditSeverity::High | AuditSeverity::Critical);
            let is_prioritized = self.retention_policy.prioritized_events.contains(&entry.event_type);

            is_recent || (is_high_severity && is_prioritized)
        });

        // If still over limit, remove oldest entries regardless
        if self.audit_store.len() > self.max_entries {
            self.audit_store.sort_by_key(|e| e.timestamp);
            self.audit_store.truncate(self.max_entries);
        }
    }

    /// Check if entry matches query filter
    fn matches_query(&self, entry: &AuditEntry, query: &AuditQuery) -> bool {
        // Time range filter
        if let Some(start_time) = query.start_time {
            if entry.timestamp < start_time {
                return false;
            }
        }
        if let Some(end_time) = query.end_time {
            if entry.timestamp > end_time {
                return false;
            }
        }

        // Event type filter
        if !query.event_types.is_empty() && !query.event_types.contains(&entry.event_type) {
            return false;
        }

        // Severity filter
        if let Some(min_severity) = &query.min_severity {
            if entry.severity < *min_severity {
                return false;
            }
        }

        // Actor filter
        if let Some(actor_filter) = &query.actor_filter {
            if !self.actor_matches(entry, actor_filter) {
                return false;
            }
        }

        // Compliance flags filter
        if !query.compliance_flags.is_empty() {
            if !entry.compliance_flags.iter().any(|flag| query.compliance_flags.contains(flag)) {
                return false;
            }
        }

        true
    }

    /// Check if actor matches filter
    fn actor_matches(&self, entry: &AuditEntry, filter: &ActorFilter) -> bool {
        match (&entry.actor, filter) {
            (AuditActor::HumanOperator { operator_id, .. }, ActorFilter::OperatorId(id)) => operator_id == id,
            (AuditActor::Drone { drone_id, .. }, ActorFilter::DroneId(id)) => drone_id == id,
            (AuditActor::Station { station_id, .. }, ActorFilter::StationId(id)) => station_id == id,
            (AuditActor::System { component, .. }, ActorFilter::SystemComponent(name)) => component == name,
            _ => false,
        }
    }
}

/// Audit query specification
#[derive(Debug, Clone)]
pub struct AuditQuery {
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub event_types: Vec<AuditEventType>,
    pub min_severity: Option<AuditSeverity>,
    pub actor_filter: Option<ActorFilter>,
    pub compliance_flags: Vec<ComplianceFlag>,
    pub limit: Option<usize>,
}

/// Actor filter options
#[derive(Debug, Clone)]
pub enum ActorFilter {
    OperatorId(String),
    DroneId(String),
    StationId(String),
    SystemComponent(String),
}

/// Report generation request
#[derive(Debug, Clone)]
pub struct ReportRequest {
    pub template_id: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub time_range: Option<(SystemTime, SystemTime)>,
    pub filters: Vec<ReportFilter>,
}

/// Audit system errors
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Invalid audit entry: {0}")]
    InvalidEntry(String),
    #[error("Storage limit exceeded")]
    StorageLimitExceeded,
    #[error("Report generation failed: {0}")]
    ReportGenerationError(String),
    #[error("Alert not found")]
    AlertNotFound,
    #[error("Query execution failed")]
    QueryError,
}

/// Quick audit entry creation helper
pub fn create_audit_entry(
    event_type: AuditEventType,
    severity: AuditSeverity,
    actor: AuditActor,
    operation: AuditOperation,
    result: OperationResult,
    context: AuditContext
) -> AuditEntry {
    AuditEntry {
        entry_id: String::new(), // Will be filled by audit system
        timestamp: SystemTime::now(),
        event_type,
        severity,
        actor,
        target: None,
        operation,
        result,
        context,
        compliance_flags: Vec::new(),
        security_metadata: SecurityMetadata {
            authentication_method: "default".to_string(),
            authorization_checks: Vec::new(),
            cryptographic_operations: Vec::new(),
            security_controls_applied: Vec::new(),
            suspicious_indicators: Vec::new(),
            integrity_hash: "placeholder".to_string(),
        },
        evidence: Vec::new(),
    }
}

/// Report template specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub sections: Vec<ReportSection>,
    pub filters: Vec<ReportFilter>,
    pub format: ReportFormat,
}

/// Report section definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub section_id: String,
    pub title: String,
    pub data_sources: Vec<String>,
    pub aggregation_rules: Vec<String>,
    pub visualization_hints: Vec<String>,
}

/// Report filter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilter {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
    pub description: String,
}

/// Report format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    PDF,
    HTML,
    JSON,
    CSV,
    XML,
}

/// Scheduled report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReport {
    pub schedule_id: String,
    pub template_id: String,
    pub frequency: ScheduleFrequency,
    pub last_run: Option<SystemTime>,
    pub next_run: SystemTime,
    pub recipients: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Report execution frequencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    Custom(String),
}

/// Generated report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedReport {
    pub report_id: String,
    pub template_id: String,
    pub generated_at: SystemTime,
    pub parameters_used: HashMap<String, serde_json::Value>,
    pub file_path: String,
    pub file_hash: String,
    pub generation_duration_ms: u64,
    pub status: ReportStatus,
}

/// Report generation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportStatus {
    Success,
    Warning { messages: Vec<String> },
    Error { error_message: String },
}

/// Audit report generator
pub struct AuditReportGenerator {
    report_templates: HashMap<String, ReportTemplate>,
    scheduled_reports: Vec<ScheduledReport>,
    generated_reports: Vec<GeneratedReport>,
}

impl AuditReportGenerator {
    /// Create new report generator
    pub fn new() -> Self {
        Self {
            report_templates: HashMap::new(),
            scheduled_reports: Vec::new(),
            generated_reports: Vec::new(),
        }
    }

    /// Add report template
    pub fn add_template(&mut self, template: ReportTemplate) {
        self.report_templates.insert(template.template_id.clone(), template);
    }

    /// Generate audit report
    pub fn generate_report(&mut self, request: ReportRequest, audit_entries: &[AuditEntry]) -> Result<String, AuditError> {
        let template = self.report_templates.get(&request.template_id)
            .ok_or(AuditError::ReportGenerationError("Template not found".to_string()))?;

        // Filter audit entries based on request
        let filtered_entries: Vec<&AuditEntry> = audit_entries.iter()
            .filter(|entry| {
                // Apply time range filter
                if let Some((start, end)) = request.time_range {
                    if entry.timestamp < start || entry.timestamp > end {
                        return false;
                    }
                }
                // Apply custom filters
                self.apply_filters(entry, &request.filters)
            })
            .collect();

        // Generate report based on format
        let _report_content = match template.format {
            ReportFormat::JSON => self.generate_json_report(template, &filtered_entries)?,
            ReportFormat::CSV => self.generate_csv_report(template, &filtered_entries)?,
            _ => return Err(AuditError::ReportGenerationError("Unsupported format".to_string())),
        };

        // Create report metadata
        let report = GeneratedReport {
            report_id: format!("report_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()),
            template_id: request.template_id.clone(),
            generated_at: SystemTime::now(),
            parameters_used: request.parameters.clone(),
            file_path: format!("/audit_reports/{}_{}.json", request.template_id, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()),
            file_hash: "placeholder_hash".to_string(), // In production, would compute actual hash
            generation_duration_ms: 1000, // Placeholder
            status: ReportStatus::Success,
        };

        self.generated_reports.push(report.clone());

        Ok(report.report_id)
    }

    /// Generate JSON audit report
    fn generate_json_report(&self, template: &ReportTemplate, entries: &[&AuditEntry]) -> Result<String, AuditError> {
        let report_data = serde_json::json!({
            "template_id": template.template_id,
            "generated_at": SystemTime::now(),
            "total_entries": entries.len(),
            "entries": entries
        });

        serde_json::to_string_pretty(&report_data)
            .map_err(|e| AuditError::ReportGenerationError(e.to_string()))
    }

    /// Generate CSV audit report
    fn generate_csv_report(&self, _template: &ReportTemplate, entries: &[&AuditEntry]) -> Result<String, AuditError> {
        let mut csv = String::from("timestamp,event_type,severity,actor,result\n");

        for entry in entries {
            let actor_str = match &entry.actor {
                AuditActor::HumanOperator { operator_id, .. } => format!("operator:{}", operator_id),
                AuditActor::Drone { drone_id, .. } => format!("drone:{}", drone_id),
                AuditActor::Station { station_id, .. } => format!("station:{}", station_id),
                _ => "system".to_string(),
            };

            csv.push_str(&format!("{},{:?},{:?},{},{}\n",
                entry.timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
                entry.event_type,
                entry.severity,
                actor_str,
                entry.result.success
            ));
        }

        Ok(csv)
    }

    /// Apply report filters to audit entry
    fn apply_filters(&self, entry: &AuditEntry, filters: &[ReportFilter]) -> bool {
        for filter in filters {
            if !self.evaluate_filter(entry, filter) {
                return false;
            }
        }
        true
    }

    /// Evaluate individual filter condition
    fn evaluate_filter(&self, entry: &AuditEntry, filter: &ReportFilter) -> bool {
        match (filter.field.as_str(), filter.operator.as_str()) {
            ("severity", "gte") => {
                if let Some(severity_val) = filter.value.as_str() {
                    let filter_severity = match severity_val {
                        "Low" => AuditSeverity::Low,
                        "Medium" => AuditSeverity::Medium,
                        "High" => AuditSeverity::High,
                        "Critical" => AuditSeverity::Critical,
                        _ => return false,
                    };
                    return entry.severity >= filter_severity;
                }
            }
            ("event_type", "eq") => {
                if let Some(event_str) = filter.value.as_str() {
                    // Simplified event type matching
                    return format!("{:?}", entry.event_type) == event_str;
                }
            }
            _ => {} // Unsupported filter combinations return true
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_system_creation() {
        let audit_system = AuditSystem::new(1000);
        assert_eq!(audit_system.audit_store.len(), 0);
        assert_eq!(audit_system.max_entries, 1000);
    }

    #[tokio::test]
    async fn test_audit_entry_recording() {
        let mut audit_system = AuditSystem::new(1000);

        let entry = create_audit_entry(
            AuditEventType::MissionTransfer,
            AuditSeverity::High,
            AuditActor::System {
                component: "test_component".to_string(),
                version: "1.0".to_string(),
                subsystem: "test".to_string(),
            },
            AuditOperation {
                operation_type: "transfer".to_string(),
                operation_name: "mission_transfer".to_string(),
                parameters: std::collections::HashMap::new(),
                execution_context: OperationContext::default(),
                expected_duration: Some(Duration::from_secs(30)),
                resource_consumption: ResourceConsumption::default(),
            },
            OperationResult {
                success: true,
                error_code: None,
                error_message: None,
                duration_ms: 1500,
                performance_metrics: PerformanceMetrics::default(),
                side_effects: vec![],
            },
            AuditContext::default(),
        );

        let result = audit_system.record_event(entry);
        assert!(result.is_ok());
        assert_eq!(audit_system.audit_store.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_query() {
        let mut audit_system = AuditSystem::new(1000);

        // Add a test entry
        let entry = create_audit_entry(
            AuditEventType::SecurityAuthentication,
            AuditSeverity::Medium,
            AuditActor::HumanOperator {
                operator_id: "test_operator".to_string(),
                clearance_level: "standard".to_string(),
                department: Some("operations".to_string()),
            },
            AuditOperation {
                operation_type: "auth".to_string(),
                operation_name: "pin_auth".to_string(),
                parameters: std::collections::HashMap::new(),
                execution_context: OperationContext::default(),
                expected_duration: Some(Duration::from_millis(500)),
                resource_consumption: ResourceConsumption::default(),
            },
            OperationResult {
                success: true,
                error_code: None,
                error_message: None,
                duration_ms: 200,
                performance_metrics: PerformanceMetrics::default(),
                side_effects: vec![],
            },
            AuditContext::default(),
        );

        audit_system.record_event(entry).unwrap();

        // Query for security authentication events
        let query = AuditQuery {
            start_time: None,
            end_time: None,
            event_types: vec![AuditEventType::SecurityAuthentication],
            min_severity: None,
            actor_filter: None,
            compliance_flags: vec![],
            limit: None,
        };

        let results = audit_system.query_audit(query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, AuditEventType::SecurityAuthentication);
    }

    #[tokio::test]
    async fn test_report_generator() {
        let mut report_generator = AuditReportGenerator::new();

        // Add a simple template
        let template = ReportTemplate {
            template_id: "test_template".to_string(),
            name: "Test Report".to_string(),
            description: "Test audit report".to_string(),
            sections: vec![],
            filters: vec![],
            format: ReportFormat::JSON,
        };

        report_generator.add_template(template);

        // Test report generation with empty audit data
        let request = ReportRequest {
            template_id: "test_template".to_string(),
            parameters: std::collections::HashMap::new(),
            time_range: None,
            filters: vec![],
        };

        let result = report_generator.generate_report(request, &[]);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_alerts() {
        let mut audit_system = AuditSystem::new(1000);

        // Initially no alerts
        assert_eq!(audit_system.get_active_alerts().len(), 0);

        // Create a critical audit entry that should trigger compliance rules
        let entry = create_audit_entry(
            AuditEventType::EmergencyAction,
            AuditSeverity::Critical,
            AuditActor::System {
                component: "emergency_system".to_string(),
                version: "1.0".to_string(),
                subsystem: "safety".to_string(),
            },
            AuditOperation {
                operation_type: "emergency".to_string(),
                operation_name: "emergency_shutdown".to_string(),
                parameters: std::collections::HashMap::new(),
                execution_context: OperationContext::default(),
                expected_duration: Some(Duration::from_millis(100)),
                resource_consumption: ResourceConsumption::default(),
            },
            OperationResult {
                success: true,
                error_code: None,
                error_message: None,
                duration_ms: 50,
                performance_metrics: PerformanceMetrics::default(),
                side_effects: vec!["system_shutdown".to_string()],
            },
            AuditContext::default(),
        );

        audit_system.record_event(entry).unwrap();

        // Should have generated alerts due to critical severity
        let active_alerts = audit_system.get_active_alerts();
        assert!(!active_alerts.is_empty());
    }
}