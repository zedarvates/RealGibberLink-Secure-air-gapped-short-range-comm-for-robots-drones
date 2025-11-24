//! Drone and station interface schemas for mission transfer operations
//!
//! This module defines the operational interfaces between drones, mission stations,
//! and human operators, providing schema validation and operational state management.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use crate::mission::{MissionPayload, MissionId, GeoCoordinate, MissionPriority};
use crate::mission_transfer::{MissionTransferError, EncryptedMissionPayload};
use crate::weather::{WeatherManager, WeatherData, ConstraintValidationResult};
use crate::security::{SecurityManager, AuthorizationScope, PermissionGrant};

/// Drone operational states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DroneOperationalState {
    Idle,
    PreFlightChecks,
    ReadyForMission,
    MissionLoading,
    MissionValidation,
    MissionExecution,
    MissionPaused,
    MissionAbort,
    PostMission,
    MaintenanceRequired,
    Error(String),
}

/// Mission station operational states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StationOperationalState {
    Idle,
    MissionPreparation,
    MissionValidation,
    WaitingForDrone,
    TransferInProgress,
    TransferComplete,
    MonitoringMission,
    EmergencyResponse,
    SystemMaintenance,
    Error(String),
}

/// Drone interface schema
#[derive(Debug, Clone)]
pub struct DroneInterface {
    pub drone_id: String,
    pub model: String,
    pub capabilities: DroneCapabilities,
    pub current_state: DroneOperationalState,
    pub location: Option<GeoCoordinate>,
    pub battery_soc: f32, // State of charge (0.0-1.0)
    pub communication_status: CommunicationStatus,
    pub active_mission: Option<MissionId>,
    pub last_update: SystemTime,
}

/// Station interface schema
#[derive(Debug, Clone)]
pub struct StationInterface {
    pub station_id: String,
    pub location: GeoCoordinate,
    pub capabilities: StationCapabilities,
    pub current_state: StationOperationalState,
    pub active_sessions: HashMap<String, SessionInfo>,
    pub connected_drones: Vec<String>,
    pub weather_manager: WeatherManager,
    pub security_manager: SecurityManager,
    pub mission_inventory: HashMap<MissionId, MissionInventoryItem>,
    pub last_update: SystemTime,
}

/// Drone capabilities specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneCapabilities {
    pub max_payload_kg: f32,
    pub max_flight_time_minutes: u32,
    pub max_range_km: f32,
    pub max_altitude_m: f32,
    pub supported_sensors: Vec<SensorCapability>,
    pub communication_channels: Vec<CommunicationChannel>,
    pub weather_limits: WeatherLimits,
    pub emergency_features: Vec<String>,
}

/// Station capabilities specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationCapabilities {
    pub max_concurrent_transfers: u32,
    pub supported_drone_models: Vec<String>,
    pub weather_integration: bool,
    pub emergency_override: bool,
    pub fleet_management: bool,
    pub offline_capability: bool,
    pub audit_logging: bool,
}

/// Sensor capability specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCapability {
    pub sensor_type: String,
    pub resolution: String,
    pub weather_tolerance: String,
    pub power_consumption_w: f32,
    pub operational_range_m: f32,
}

/// Communication channel specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationChannel {
    GibberLinkShortRange,
    GibberLinkLongRange,
    Cellular4G,
    Cellular5G,
    Satellite,
    WiFiDirect,
}

/// Weather limits specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherLimits {
    pub max_wind_speed_mps: f32,
    pub max_gust_speed_mps: f32,
    pub min_visibility_m: f32,
    pub max_temperature_c: f32,
    pub min_temperature_c: f32,
    pub max_precipitation_mmh: f32,
}

/// Communication status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStatus {
    pub signal_strength: f32, // 0.0 to 1.0
    pub channel_type: String,
    pub last_contact: SystemTime,
    pub data_rate_bps: u32,
    pub error_rate: f32,
}

/// Active session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub drone_id: String,
    pub operator_id: Option<String>,
    pub mission_id: Option<MissionId>,
    pub start_time: SystemTime,
    pub state: String,
    pub weather_conditions: Option<WeatherData>,
}

/// Mission inventory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionInventoryItem {
    pub mission: MissionPayload,
    pub creator_id: String,
    pub approval_status: ApprovalStatus,
    pub weather_validation: Option<ConstraintValidationResult>,
    pub created_time: SystemTime,
    pub expires_time: Option<SystemTime>,
}

/// Mission approval status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalStatus {
    Draft,
    PendingReview,
    Approved,
    Rejected,
    Expired,
}

/// Operator validation interface
#[derive(Debug, Clone)]
pub struct HumanOperatorInterface {
    pub operator_id: String,
    pub clearance_level: SecurityClearance,
    pub authorized_scopes: Vec<AuthorizationScope>,
    pub active_sessions: Vec<String>,
    pub validation_history: Vec<OperatorValidationRecord>,
    pub current_location: Option<GeoCoordinate>,
}

/// Security clearance levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum SecurityClearance {
    Basic,
    Standard,
    Advanced,
    Critical,
    Emergency,
}

/// Operator validation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorValidationRecord {
    pub mission_id: MissionId,
    pub timestamp: SystemTime,
    pub approved_scopes: Vec<AuthorizationScope>,
    pub risk_assessment: f32, // 0.0 to 1.0
    pub validation_reason: String,
    pub operator_id: String,
}

/// Operational result schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalResult {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub timestamp: SystemTime,
    pub success: bool,
    pub duration_ms: u64,
    pub result_data: ResultData,
    pub error_details: Option<String>,
}

/// Types of operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    MissionTransfer,
    WeatherValidation,
    SecurityCheck,
    DroneCommand,
    SystemHealthCheck,
    EmergencyOverride,
}

/// Result data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultData {
    MissionTransfer { mission_id: MissionId, transfer_size_kb: f32 },
    WeatherValidation { violations_count: u32, risk_score: f32 },
    SecurityCheck { scopes_validated: Vec<String> },
    DroneCommand { command_type: String, parameters: HashMap<String, String> },
    SystemHealthCheck { components_checked: Vec<String>, issues_found: u32 },
    EmergencyOverride { override_reason: String, original_state: String },
}

/// Fleet management interface
pub struct FleetManager {
    pub station_interfaces: HashMap<String, StationInterface>,
    pub drone_fleet: HashMap<String, DroneInterface>,
    pub active_missions: HashMap<MissionId, MissionAssignment>,
    pub mission_queue: Vec<MissionQueueItem>,
    pub global_weather_manager: WeatherManager,
    pub security_policies: FleetSecurityPolicies,
}

/// Mission assignment for fleet management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionAssignment {
    pub mission_id: MissionId,
    pub assigned_drone: String,
    pub assigned_station: String,
    pub operator_id: Option<String>,
    pub assignment_time: SystemTime,
    pub expected_completion: SystemTime,
    pub status: AssignmentStatus,
    pub progress_percent: f32,
}

/// Mission assignment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssignmentStatus {
    Scheduled,
    InProgress,
    Paused,
    Completed,
    Failed,
    Aborted,
}

/// Queued mission item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionQueueItem {
    pub mission_id: MissionId,
    pub priority: MissionPriority,
    pub requested_station: Option<String>,
    pub weather_constraints: Vec<String>,
    pub time_window: TimeWindow,
    pub required_clearance: SecurityClearance,
}

/// Time window specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub max_duration: Duration,
    pub weather_acceptable: bool,
}

/// Fleet security policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSecurityPolicies {
    pub max_simultaneous_missions: u32,
    pub max_drones_per_station: u32,
    pub automatic_weather_aborts: bool,
    pub emergency_override_required: bool,
    pub audit_all_operations: bool,
    pub log_security_events: bool,
    pub session_timeout_minutes: u32,
}

impl DroneInterface {
    /// Create new drone interface
    pub fn new(drone_id: String, model: String, capabilities: DroneCapabilities) -> Self {
        Self {
            drone_id,
            model,
            capabilities,
            current_state: DroneOperationalState::Idle,
            location: None,
            battery_soc: 0.0,
            communication_status: CommunicationStatus {
                signal_strength: 0.0,
                channel_type: "none".to_string(),
                last_contact: SystemTime::now(),
                data_rate_bps: 0,
                error_rate: 0.0,
            },
            active_mission: None,
            last_update: SystemTime::now(),
        }
    }

    /// Update drone operational state
    pub fn update_state(&mut self, new_state: DroneOperationalState) {
        println!("Drone {} state change: {:?} -> {:?}", self.drone_id, self.current_state, new_state);
        self.current_state = new_state;
        self.last_update = SystemTime::now();
    }

    /// Check if drone is ready for mission assignment
    pub fn is_ready_for_mission(&self) -> bool {
        matches!(self.current_state, DroneOperationalState::Idle | DroneOperationalState::ReadyForMission) &&
        self.battery_soc > 0.2 && // At least 20% battery
        self.communication_status.signal_strength > 0.5 // Good signal
    }

    /// Validate mission compatibility with drone capabilities
    pub fn validate_mission_compatibility(&self, mission: &MissionPayload) -> Result<(), String> {
        // Check battery requirements
        let required_energy = mission.constraints.energy.expected_consumption_wh;
        let available_energy = self.capabilities.weather_limits.max_temperature_c as f32; // Placeholder calculation

        if required_energy > available_energy {
            return Err("Mission requires more energy than drone battery capacity".to_string());
        }

        // Check altitude limits
        for path in &mission.flight_plan.paths {
            for waypoint in &path.waypoints {
                if waypoint.position.altitude_msl > self.capabilities.max_altitude_m as f32 {
                    return Err(format!("Waypoint altitude {} exceeds drone limit {}", waypoint.position.altitude_msl, self.capabilities.max_altitude_m));
                }
            }
        }

        // Check payload requirements
        let total_payload = mission.tasks.iter()
            .map(|task| {
                task.actions.iter()
                    .map(|action| match action {
                        crate::mission::MissionAction::DeployPayload { payload_type, .. } => {
                            // Estimate payload weight (placeholder)
                            1.0 // kg
                        },
                        _ => 0.0,
                    })
                    .sum::<f32>()
            })
            .sum::<f32>();

        if total_payload > self.capabilities.max_payload_kg {
            return Err(format!("Total payload {}kg exceeds drone capacity {}kg", total_payload, self.capabilities.max_payload_kg));
        }

        Ok(())
    }
}

impl StationInterface {
    /// Create new station interface
    pub fn new(station_id: String, location: GeoCoordinate, capabilities: StationCapabilities) -> Self {
        Self {
            station_id,
            location,
            capabilities,
            current_state: StationOperationalState::Idle,
            active_sessions: HashMap::new(),
            connected_drones: Vec::new(),
            weather_manager: WeatherManager::new(100), // 100 weather history entries
            security_manager: SecurityManager::new(Default::default()),
            mission_inventory: HashMap::new(),
            last_update: SystemTime::now(),
        }
    }

    /// Prepare mission for a specific drone
    pub async fn prepare_mission_for_drone(&mut self, mission: MissionPayload, drone: &DroneInterface) -> Result<EncryptedMissionPayload, MissionTransferError> {
        // Validate drone compatibility
        drone.validate_mission_compatibility(&mission)
            .map_err(|e| MissionTransferError::MissionIntegrityError(e))?;

        // Update weather data if available
        if let Some(weather) = self.get_current_weather().await {
            self.weather_manager.update_weather(weather)
                .map_err(|_| MissionTransferError::WeatherValidationError)?;
        }

        // Validate constraints against current weather
        let weather_ok = self.weather_manager.validate_mission_constraints(
            &mission,
            &crate::weather::DroneSpecifications {
                max_wind_speed_mps: drone.capabilities.weather_limits.max_wind_speed_mps,
                max_speed_mps: 15.0, // Default max speed
                abort_gust_threshold_mps: drone.capabilities.weather_limits.max_gust_speed_mps,
                power_wind_coefficient: 5.0,
                mass_kg: 2.5,
                battery_capacity_wh: 100.0,
                sensor_types: drone.capabilities.supported_sensors.iter().map(|s| s.sensor_type.clone()).collect(),
            }
        );

        if let Ok(validation) = weather_ok {
            if !validation.is_valid && validation.risk_assessment.abort_recommended {
                return Err(MissionTransferError::MissionIntegrityError("Weather conditions unsafe for mission".to_string()));
            }
        }

        // Create encrypted payload for transfer
        // Note: This would integrate with the MissionStation from mission_transfer.rs

        // Placeholder for encrypted payload creation
        Ok(EncryptedMissionPayload {
            mission_id: mission.header.id,
            encrypted_data: vec![1, 2, 3], // Placeholder encrypted data
            signature: vec![4, 5, 6],
            session_nonce: [7; 16],
            validity_timestamp: SystemTime::now() + Duration::from_secs(300),
            weather_fingerprint: [8; 32],
        })
    }

    /// Add drone to connected fleet
    pub fn connect_drone(&mut self, drone_id: String) {
        if !self.connected_drones.contains(&drone_id) {
            self.connected_drones.push(drone_id);
            println!("Drone connected to station {}", self.station_id);
        }
    }

    /// Remove drone from connected fleet
    pub fn disconnect_drone(&mut self, drone_id: &str) {
        self.connected_drones.retain(|id| id != drone_id);
        println!("Drone {} disconnected from station {}", drone_id, self.station_id);
    }

    /// Get current weather for station location
    pub async fn get_current_weather(&self) -> Option<WeatherData> {
        if let Some(weather) = self.weather_manager.get_current_weather() {
            // Check if weather is recent (within 10 minutes)
            let age = weather.timestamp.elapsed().unwrap_or(Duration::from_secs(0));
            if age < Duration::from_secs(600) {
                return Some(weather.clone());
            }
        }
        None
    }
}

impl HumanOperatorInterface {
    /// Create new operator interface
    pub fn new(operator_id: String, clearance_level: SecurityClearance) -> Self {
        Self {
            operator_id,
            clearance_level,
            authorized_scopes: Vec::new(),
            active_sessions: Vec::new(),
            validation_history: Vec::new(),
            current_location: None,
        }
    }

    /// Check if operator has required clearance for mission
    pub fn has_clearance_for_mission(&self, mission: &MissionPayload, required_scopes: &[AuthorizationScope]) -> bool {
        // Check clearance level matches mission priority
        let required_clearance = match mission.header.priority {
            MissionPriority::Low | MissionPriority::Normal => SecurityClearance::Basic,
            MissionPriority::High => SecurityClearance::Standard,
            MissionPriority::Critical => SecurityClearance::Advanced,
            MissionPriority::Emergency => SecurityClearance::Emergency,
        };

        if self.clearance_level < required_clearance {
            return false;
        }

        // Check authorized scopes
        for scope in required_scopes {
            if !self.authorized_scopes.contains(scope) {
                return false;
            }
        }

        true
    }

    /// Record validation action for audit trail
    pub fn record_validation(&mut self, record: OperatorValidationRecord) {
        self.validation_history.push(record);
        // Keep only last 1000 records
        if self.validation_history.len() > 1000 {
            self.validation_history.remove(0);
        }
    }
}

impl FleetManager {
    /// Create new fleet manager
    pub fn new() -> Self {
        Self {
            station_interfaces: HashMap::new(),
            drone_fleet: HashMap::new(),
            active_missions: HashMap::new(),
            mission_queue: Vec::new(),
            global_weather_manager: WeatherManager::new(500), // Larger weather history
            security_policies: FleetSecurityPolicies {
                max_simultaneous_missions: 10,
                max_drones_per_station: 5,
                automatic_weather_aborts: true,
                emergency_override_required: true,
                audit_all_operations: true,
                log_security_events: true,
                session_timeout_minutes: 60,
            },
        }
    }

    /// Assign mission to optimal drone and station
    pub fn assign_mission(&mut self, mission: MissionPayload) -> Result<String, String> {
        // Find suitable station
        let suitable_station = self.find_suitable_station(&mission)?;
        let station = self.station_interfaces.get_mut(&suitable_station)
            .ok_or("Selected station not found")?;

        // Find suitable drone connected to that station
        let suitable_drone = self.find_suitable_drone(&mission, &suitable_station)?;
        let drone = self.drone_fleet.get(&suitable_drone)
            .ok_or("Selected drone not found")?;

        // Validate mission against drone capabilities and weather
        station.prepare_mission_for_drone(mission.clone(), drone)
            .map_err(|e| format!("Mission preparation failed: {:?}", e))?;

        // Create assignment
        let assignment = MissionAssignment {
            mission_id: mission.header.id,
            assigned_drone: suitable_drone.clone(),
            assigned_station: suitable_station.clone(),
            operator_id: None,
            assignment_time: SystemTime::now(),
            expected_completion: SystemTime::now() + mission.header.max_execution_duration,
            status: AssignmentStatus::Scheduled,
            progress_percent: 0.0,
        };

        self.active_missions.insert(mission.header.id, assignment);

        // Update station mission inventory
        station.mission_inventory.insert(mission.header.id, MissionInventoryItem {
            mission,
            creator_id: "fleet_manager".to_string(),
            approval_status: ApprovalStatus::Approved,
            weather_validation: None, // Would be filled from actual validation
            created_time: SystemTime::now(),
            expires_time: Some(SystemTime::now() + Duration::from_secs(3600)), // 1 hour
        });

        Ok(format!("Mission assigned: Station={}, Drone={}", suitable_station, suitable_drone))
    }

    /// Find suitable station for mission
    fn find_suitable_station(&self, mission: &MissionPayload) -> Result<String, String> {
        // Simple selection logic - find station with fewest active missions
        let mut best_station = None;
        let mut min_load = u32::MAX;

        for (station_id, station) in &self.station_interfaces {
            let current_load = station.active_sessions.len() as u32;
            if current_load < min_load && current_load < self.security_policies.max_drones_per_station {
                min_load = current_load;
                best_station = Some(station_id.clone());
            }
        }

        best_station.ok_or("No suitable station found".to_string())
    }

    /// Find suitable drone for mission at specific station
    fn find_suitable_drone(&self, mission: &MissionPayload, station_id: &str) -> Result<String, String> {
        let station = self.station_interfaces.get(station_id)
            .ok_or("Station not found")?;

        // Find connected, ready drones
        for drone_id in &station.connected_drones {
            if let Some(drone) = self.drone_fleet.get(drone_id) {
                if drone.is_ready_for_mission() {
                    // Check capability compatibility
                    if drone.validate_mission_compatibility(mission).is_ok() {
                        return Ok(drone_id.clone());
                    }
                }
            }
        }

        Err("No suitable drone found".to_string())
    }

    /// Monitor active missions and handle failures
    pub async fn monitor_missions(&mut self) -> Vec<String> {
        let mut events = Vec::new();

        // Check for mission timeouts
        let mut completed_missions = Vec::new();
        for (mission_id, assignment) in &self.active_missions {
            if assignment.status == AssignmentStatus::InProgress {
                let elapsed = assignment.assignment_time.elapsed().unwrap_or(Duration::from_secs(0));
                if elapsed > assignment.expected_completion.duration_since(assignment.assignment_time).unwrap_or(Duration::from_secs(3600)) {
                    events.push(format!("Mission {} timed out", mission_id));
                    // Would trigger abort procedure
                }
            } else if matches!(assignment.status, AssignmentStatus::Completed | AssignmentStatus::Failed | AssignmentStatus::Aborted) {
                completed_missions.push(*mission_id);
            }
        }

        // Clean up completed missions
        for mission_id in completed_missions {
            self.active_missions.remove(&mission_id);
        }

        events
    }
}

/// Safety and compliance validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyValidationResult {
    pub mission_id: MissionId,
    pub is_safe: bool,
    pub safety_checks: Vec<SafetyCheck>,
    pub compliance_score: f32, // 0.0 to 1.0
    pub recommended_actions: Vec<String>,
    pub risk_assessment: String,
}

/// Individual safety check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheck {
    pub check_type: String,
    pub passed: bool,
    pub severity: ViolationSeverity,
    pub message: String,
    pub mitigation_steps: Vec<String>,
}

/// Violation severity levels (re-export from weather module for convenience)
pub use crate::weather::ViolationSeverity;

/// Automated safety validation for mission authorization
pub fn validate_mission_safety(mission: &MissionPayload, weather: &WeatherData, drone_specs: &DroneCapabilities) -> SafetyValidationResult {
    let mut safety_checks = Vec::new();
    let mut issues = Vec::new();

    // Altitude safety checks
    for path in &mission.flight_plan.paths {
        for waypoint in &path.waypoints {
            if waypoint.position.altitude_msl > drone_specs.max_altitude_m as f32 {
                safety_checks.push(SafetyCheck {
                    check_type: "altitude_limit".to_string(),
                    passed: false,
                    severity: ViolationSeverity::Critical,
                    message: format!("Waypoint altitude {}m exceeds drone limit {}m", waypoint.position.altitude_msl, drone_specs.max_altitude_m),
                    mitigation_steps: vec!["Reduce waypoint altitudes".to_string(), "Split mission into lower-altitude segments".to_string()],
                });
                issues.push("altitude_limit".to_string());
            }
        }
    }

    // Weather-related safety checks
    if weather.wind_speed_mps > drone_specs.weather_limits.max_wind_speed_mps {
        safety_checks.push(SafetyCheck {
            check_type: "wind_conditions".to_string(),
            passed: false,
            severity: ViolationSeverity::Abort,
            message: format!("Wind speed {} m/s exceeds safety limit {} m/s", weather.wind_speed_mps, drone_specs.weather_limits.max_wind_speed_mps),
            mitigation_steps: vec!["Delay mission until wind conditions improve".to_string(), "Reduce operational speed".to_string()],
        });
        issues.push("wind_conditions".to_string());
    }

    if weather.visibility_meters < drone_specs.weather_limits.min_visibility_m {
        safety_checks.push(SafetyCheck {
            check_type: "visibility_conditions".to_string(),
            passed: false,
            severity: ViolationSeverity::Critical,
            message: format!("Visibility {}m below minimum {}m", weather.visibility_meters, drone_specs.weather_limits.min_visibility_m),
            mitigation_steps: vec!["Enable instrument flight if available".to_string(), "Switch to LiDAR navigation".to_string()],
        });
        issues.push("visibility_conditions".to_string());
    }

    // Energy reserve checks
    let safe_reserve = mission.constraints.energy.reserve_margin_soc > 0.15; // Minimum 15% reserve
    if !safe_reserve {
        safety_checks.push(SafetyCheck {
            check_type: "energy_reserve".to_string(),
            passed: false,
            severity: ViolationSeverity::Warning,
            message: format!("Energy reserve {}% below recommended minimum 15%", mission.constraints.energy.reserve_margin_soc * 100.0),
            mitigation_steps: vec!["Increase battery reserve margin".to_string(), "Shorten mission duration".to_string()],
        });
        issues.push("energy_reserve".to_string());
    }

    // Calculate compliance score
    let total_checks = safety_checks.len() + 1; // +1 for energy check already performed
    let passed_checks = safety_checks.iter().filter(|c| c.severity == ViolationSeverity::Warning).count() + 1;
    let compliance_score = passed_checks as f32 / total_checks as f32;

    let is_safe = issues.is_empty() || issues.iter().all(|issue| issue == "energy_reserve");

    SafetyValidationResult {
        mission_id: mission.header.id,
        is_safe,
        safety_checks,
        compliance_score,
        recommended_actions: vec![
            "Review all safety check failures".to_string(),
            "Apply mitigation steps for any violations".to_string(),
            "Consider mission abort if critical violations exist".to_string(),
        ],
        risk_assessment: if is_safe { "Low Risk".to_string() } else { "High Risk - Manual Review Required".to_string() },
    }
}
