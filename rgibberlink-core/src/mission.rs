//! Mission and flight plan transfer system for autonomous drones
//!
//! This module implements secure encrypted flight plan delivery with weather-aware
//! constraints and validation for drone operations. Supports mission header, flight
//! paths, control points, actions, geofencing, energy management, and safety policies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// Unique mission identifier (UUID-like format)
pub type MissionId = [u8; 16];

/// Geographic coordinate in decimal degrees
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeoCoordinate {
    pub latitude: f64,  // -90.0 to 90.0
    pub longitude: f64, // -180.0 to 180.0
    pub altitude_msl: f32, // Meters above mean sea level
}

/// Geographic bounds for zones and corridors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoBounds {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
    pub min_altitude: f32,
    pub max_altitude: f32,
}

/// Mission header with identification and validity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionHeader {
    pub id: MissionId,
    pub name: String,
    pub description: Option<String>,
    pub validity_start: SystemTime,
    pub validity_end: SystemTime,
    pub max_execution_duration: Duration,
    pub issuing_station_fingerprint: [u8; 32],
    pub drone_fingerprint: Option<[u8; 32]>,
    pub priority: MissionPriority,
    pub tags: Vec<String>,
}

/// Mission priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MissionPriority {
    Low,
    Normal,
    High,
    Critical,
    Emergency,
}

/// Waypoint with position, tolerances, and loiter parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: u32,
    pub position: GeoCoordinate,
    pub position_tolerance_m: f32,
    pub altitude_tolerance_m: f32,
    pub loiter_time_seconds: Option<u32>,
    pub loiter_radius_m: Option<f32>,
    pub speed_limit_mps: Option<f32>,
    pub heading_required_degrees: Option<f32>,
    pub heading_tolerance_degrees: f32,
}

/// Flight path segment with speed and altitude constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightPath {
    pub id: u32,
    pub waypoints: Vec<Waypoint>,
    pub max_speed_mps: f32,
    pub min_speed_mps: f32,
    pub climb_rate_max_mps: f32,
    pub descent_rate_max_mps: f32,
    pub max_bank_angle_degrees: Option<f32>,
    pub min_turn_radius_m: Option<f32>,
    pub corridor_bounds: Option<GeoBounds>,
}

/// Control point types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlPoint {
    PatrolArea {
        id: u32,
        bounds: GeoBounds,
        altitude_min: f32,
        altitude_max: f32,
        pattern: PatrolPattern,
        dwell_time_per_pass: u32,
    },
    ObservationBox {
        id: u32,
        target_location: GeoCoordinate,
        observation_radius_m: f32,
        observation_altitude: f32,
        sensor_config: SensorConfiguration,
    },
    Rendezvous {
        id: u32,
        location: GeoCoordinate,
        time_window_start: SystemTime,
        time_window_end: SystemTime,
        partner_id: Option<String>,
    },
    ReturnToBase {
        id: u32,
        home_location: GeoCoordinate,
        abort_conditions: Vec<AbortCondition>,
    },
    EmergencyLanding {
        id: u32,
        landing_zone: GeoCoordinate,
        priority: LandingPriority,
    },
}

/// Patrol patterns for area surveillance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatrolPattern {
    LawnMower,     // Back and forth pattern
    Spiral,        // Spiral inward/outward
    Perimeter,     // Around the edges
    Grid,         // Grid pattern
    Random,       // Random waypoints within bounds
}

/// Sensor configurations for observation tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfiguration {
    pub optical_enabled: bool,
    pub infrared_enabled: bool,
    pub lidar_enabled: bool,
    pub radar_enabled: bool,
    pub resolution_settings: HashMap<String, String>,
    pub exposure_settings: Option<ExposureSettings>,
}

/// Camera exposure settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureSettings {
    pub shutter_speed: f32,
    pub iso: u32,
    pub aperture: f32,
    pub white_balance: String,
}

/// Mission action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionAction {
    RecordVideo {
        duration_seconds: u32,
        quality: VideoQuality,
        target_location: Option<GeoCoordinate>,
    },
    CaptureImage {
        count: u32,
        interval_seconds: Option<u32>,
        target_location: Option<GeoCoordinate>,
    },
    ScanArea {
        bounds: GeoBounds,
        sensor_type: SensorType,
        resolution_m: f32,
    },
    DeployPayload {
        payload_type: String,
        target_location: GeoCoordinate,
        deployment_altitude: f32,
    },
    BeaconSignal {
        frequency_hz: f64,
        modulation_type: String,
        duration_seconds: u32,
    },
    Handoff {
        target_system: String,
        handover_data: Vec<u8>,
    },
    Wait {
        duration_seconds: u32,
        condition: Option<String>,
    },
    Custom {
        action_type: String,
        parameters: HashMap<String, String>,
    },
}

/// Video quality settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoQuality {
    Low,    // 720p
    Medium, // 1080p
    High,   // 4K
    Ultra,  // 8K
}

/// Sensor types for scanning operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorType {
    Optical,
    Infrared,
    Thermal,
    Multispectral,
    Hyperspectral,
    Lidar,
    Radar,
    Combined,
}

/// Task sequence with actions and control points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTask {
    pub id: u32,
    pub label: String,
    pub sequence_order: u32,
    pub control_point: Option<ControlPoint>,
    pub actions: Vec<MissionAction>,
    pub preconditions: Vec<String>,
    pub postconditions: Option<String>,
    pub timeout_seconds: Option<u32>,
}

/// Geofencing zone types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeofenceZone {
    KeepOut {
        bounds: GeoBounds,
        reason: String,
        exception_conditions: Vec<String>,
    },
    KeepIn {
        bounds: GeoBounds,
        reason: String,
    },
    AltitudeFloor {
        altitude_msl: f32,
        bounds: Option<GeoBounds>,
    },
    AltitudeCeiling {
        altitude_msl: f32,
        bounds: Option<GeoBounds>,
    },
    Corridor {
        waypoints: Vec<GeoCoordinate>,
        width_m: f32,
    },
}

/// Energy constraints for mission planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConstraints {
    pub min_soc_start: f32,           // Minimum state of charge to start (0.0-1.0)
    pub reserve_margin_soc: f32,     // Reserve energy margin (0.0-1.0)
    pub expected_consumption_wh: f32, // Expected energy consumption
    pub max_flight_time_minutes: u32,
    pub power_profile: Vec<PowerSegment>,
}

/// Power consumption segments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSegment {
    pub phase_start_minutes: u32,
    pub power_consumption_w: f32,
    pub altitude_m: Option<f32>,
    pub speed_mps: Option<f32>,
}

/// Safety constraints for mission execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConstraints {
    pub max_wind_speed_mps: f32,
    pub max_gust_speed_mps: f32,
    pub min_visibility_m: f32,
    pub max_proximity_to_crowd_m: f32,
    pub emergency_landing_sites: Vec<EmergencyLandingSite>,
    pub fail_safe_procedures: Vec<String>,
}

/// Emergency landing site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyLandingSite {
    pub location: GeoCoordinate,
    pub size_m: f32,
    pub surface_type: String,
    pub accessibility: LandingAccessibility,
}

/// Landing accessibility ratings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LandingAccessibility {
    Excellent,
    Good,
    Fair,
    Poor,
    Dangerous,
}

/// Abort conditions for mission termination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbortCondition {
    LowBattery { threshold_soc: f32 },
    CriticalWeather { weather_type: String, severity: f32 },
    SystemFailure { component: String },
    LostLink { timeout_seconds: u32 },
    GeofenceViolation,
    ManualOverride,
    Emergency { priority: LandingPriority },
}

/// Landing priorities for emergency procedures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LandingPriority {
    Immediate,  // Land now at any cost
    Urgent,     // Land as soon as safe spot found
    Priority,   // Complete current task then land
    Routine,    // Return to base normally
}

/// Authorization scopes for mission permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthorizationScope {
    ExecuteMission,
    Diagnostics,
    Networking,
    Coupling,
    EmergencyOverride,
    FleetManagement,
    Maintenance,
}

/// Time-based limits for authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLimits {
    pub session_max_duration_hours: u32,
    pub mission_max_duration_hours: u32,
    pub authorization_refresh_hours: u32,
    pub emergency_override_minutes: u32,
}

/// Complete mission payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionPayload {
    pub header: MissionHeader,
    pub flight_plan: FlightPlan,
    pub tasks: Vec<MissionTask>,
    pub constraints: MissionConstraints,
    pub policies: MissionPolicies,
    pub crypto: MissionCrypto,
    pub weather_snapshot: Option<WeatherSnapshot>,
    pub formation_config: Option<FormationConfiguration>, // NEW: Formation missions
}

/// Flight plan container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightPlan {
    pub paths: Vec<FlightPath>,
    pub home_location: GeoCoordinate,
    pub takeoff_procedure: Option<String>,
    pub landing_procedure: Option<String>,
    pub contingency_routes: Vec<FlightPath>,
}

/// Mission constraints container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionConstraints {
    pub geofencing: Vec<GeofenceZone>,
    pub energy: EnergyConstraints,
    pub safety: SafetyConstraints,
    pub environmental: EnvironmentalConstraints,
}

/// Environmental constraints for weather-adaptive planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConstraints {
    pub max_temperature_c: f32,
    pub min_temperature_c: f32,
    pub max_humidity_percent: f32,
    pub max_precipitation_mmh: f32,
    pub min_visibility_m: f32,
    pub max_wind_speed_mps: f32,
    pub max_gust_speed_mps: f32,
    pub protected_weather_zones: Vec<WeatherProtectedZone>,
}

/// Weather-protected zones with special handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherProtectedZone {
    pub bounds: GeoBounds,
    pub weather_sensitivity: Vec<String>,
    pub alternative_routes: Vec<FlightPath>,
}

/// Mission policies container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionPolicies {
    pub authorization_scopes: Vec<AuthorizationScope>,
    pub time_limits: TimeLimits,
    pub approval_requirements: Vec<String>,
    pub emergency_procedures: Vec<EmergencyProcedure>,
}

/// Emergency procedures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyProcedure {
    pub trigger: AbortCondition,
    pub procedure: Vec<String>,
    pub contact_info: Option<String>,
}

/// Cryptographic elements for mission integrity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCrypto {
    pub payload_signature: Vec<u8>,
    pub channel_mac_binding: Vec<u8>,
    pub nonce: [u8; 16],
    pub timestamp: SystemTime,
    pub session_key: Option<[u8; 32]>, // For encrypted missions
}

/// Formation configuration for multi-drone coordinated operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationConfiguration {
    pub formation_type: FormationType,
    pub drones: Vec<FormationDrone>,
    pub payload_config: PayloadConfiguration,
    pub synchronization: SynchronizationConfig,
    pub attachment_points: Vec<AttachmentPoint>,
    pub load_distribution: LoadDistribution,
    pub communication_mesh: Vec<MeshLink>,
    pub formation_geofence: Option<GeoBounds>,
}

/// Formation/drone swarm types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormationType {
    Square,           // 4 corners for rectangular objects
    Hexagon,          // 6 drones for larger loads
    Line,             // Linear formation for long objects
    Circle,           // Circular arrangement
    Pyramid,          // Hierarchical load distribution
    Custom(Vec<DronePosition>), // Manually specified positions
}

/// Individual drone position and role in formation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationDrone {
    pub drone_id: String,
    pub role: DroneRole,
    pub position: DronePosition,
    pub synchronization_offset: SynchronizationOffset,
    pub fail_safe_behavior: FailSafeBehavior,
    pub energy_reserve_required: f32, // Extra battery for formation operations
}

/// Drone roles in formation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DroneRole {
    Leader,           // Lead drone with primary control
    Wingman,          // Supporting drones following leader
    Anchor,           // Position-holding drones
    Lift,             // Load-bearing drones
    Scout,            // Forward/reconnaissance
    Communications,   // Signal relay
    Emergency,        // Spare for failover
}

/// 3D position relative to formation center
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DronePosition {
    pub x_offset_m: f32,        // East-West offset
    pub y_offset_m: f32,        // North-South offset
    pub z_offset_m: f32,        // Altitude offset
    pub heading_offset_degrees: f32, // Heading relative to formation
}

/// Synchronization timing offsets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynchronizationOffset {
    pub takeoff_delay_ms: u32,    // Delay before takeoff
    pub target_altitude: f32,     // Formation altitude
    pub speed_sync_enabled: bool, // Coordinate speed changes
    pub position_sync_tolerance_m: f32, // Max position deviation
}

/// Failure handling behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailSafeBehavior {
    HoldPosition,      // Stay in position
    ReturnToHome,      // RTL individually
    FormationRTL,      // Coordinated RTL
    DescendSlowly,     // Emergency descent
    CutPayload,        // Release payload
    WaitForReplacement, // Hold until replacement arrives
}

/// Payload configuration for heavy lift missions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadConfiguration {
    pub payload_type: PayloadType,
    pub weight_kg: f32,
    pub dimensions: PayloadDimensions,
    pub center_of_gravity: CenterOfGravity,
    pub stability_requirements: StabilityRequirements,
    pub release_mechanism: ReleaseMechanism,
    pub lifting_slings: Vec<LiftingSling>,
}

/// Types of payloads for formation lifting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayloadType {
    Container { volume_liters: f32, contents: String },
    Equipment { category: String, fragility: Fragility },
    Vehicle { vehicle_type: String, axle_distance_m: f32 },
    Structural { material: String, structural_integrity: f32 },
    Hazardous { hazard_class: String, containment: String },
}

/// Payload physical dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadDimensions {
    pub length_m: f32,
    pub width_m: f32,
    pub height_m: f32,
    pub volume_m3: Option<f32>,
}

/// Payload center of gravity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CenterOfGravity {
    pub x_offset_m: f32,        // From geometric center
    pub y_offset_m: f32,
    pub z_offset_m: f32,
    pub uncertainty_m: f32,     // measurement uncertainty
}

/// Stability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityRequirements {
    pub max_roll_degrees: f32,
    pub max_pitch_degrees: f32,
    pub max_yaw_rate_degrees_per_sec: f32,
    pub min_bridle_clearance_m: f32, // Minimum ground clearance
    pub wind_stability_factor: f32,  // Resistance to wind deviation
}

/// Release mechanism types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseMechanism {
    ElectromagneticRelease,
    ServoRelease,
    ThermalCutting,
    ManualRelease,
    SequentialRelease, // Release one sling at a time
    EmergencyJettison,
}

/// Grappling hook/lifting sling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiftingSling {
    pub sling_id: String,
    pub drone_assignment: String,     // Which drone carries this sling
    pub attachment_point: GeoCoordinate, // Where it connects to payload
    pub sling_type: SlingType,
    pub length_m: f32,
    pub max_load_kg: f32,
    pub tension_sensor: Option<TensionSpecification>,
}

/// Sling material and design types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlingType {
    Nylon { diameter_mm: f32 },
    Kevlar { diameter_mm: f32 },
    SteelCable { diameter_mm: f32 },
    CarbonFiber { diameter_mm: f32 },
    ElectromagneticHook,
}

/// Load tension monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensionSpecification {
    pub max_tension_kg: f32,
    pub warning_threshold_kg: f32,
    pub sensor_accuracy_kg: f32,
}

/// Attachment points on payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentPoint {
    pub point_id: String,
    pub location: PayloadCoordinate,  // Position on payload
    pub sling_connection: Option<String>, // Which sling connects here
    pub stress_limit_kg: f32,
    pub preferred_drone_angle: f32, // Optimal approach angle
}

/// Payload-relative coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadCoordinate {
    pub x_m: f32,  // From payload center
    pub y_m: f32,
    pub z_m: f32,
}

/// Load distribution across drones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadDistribution {
    pub target_load_per_drone_kg: f32,
    pub max_asymmetry_allowed: f32,      // Max load difference between drones
    pub redistribution_strategy: LoadRedistribution,
    pub dynamic_balancing: bool,         // Real-time load adjustment
    pub critical_load_threshold: f32,    // Emergency threshold
}

/// Load redistribution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadRedistribution {
    ShedLoad,           // Reduce total payload weight
    Redistribute,       // Shift load to stronger drones
    EmergencyDescent,   // Emergency landing
    AbortMission,       // Complete mission abort
}

/// Communication mesh for formation coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshLink {
    pub from_drone: String,
    pub to_drone: String,
    pub link_type: CommunicationType,
    pub max_distance_m: f32,
    pub redundancy_required: bool,
}

/// Types of inter-drone communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationType {
    DirectRadio,       // Direct radio link
    MeshRouting,       // Multi-hop mesh networking
    Ultrasonic,        // Ultrasonic position signaling
    LEDOptical,        // LED optical signaling
    CooperativeGPS,    // GPS position sharing
}

/// Synchronization configuration for formation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynchronizationConfig {
    pub clock_sync_interval_ms: u32,      // NTP-style time synchronization
    pub position_sync_tolerance_m: f32,   // Max position deviation
    pub speed_sync_tolerance_mps: f32,    // Max speed difference
    pub altitude_sync_tolerance_m: f32,   // Max altitude difference
    pub heading_sync_tolerance_deg: f32,  // Max heading difference
    pub takeoff_sequence: Vec<String>,    // Ordered drone takeoff list
    pub landing_sequence: Vec<String>,    // Ordered drone landing list
    pub emergency_sync_timeout_ms: u32,   // Max time for synchronization recovery
}

/// Formation-specific geofencing for coordinated operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationGeofence {
    pub formation_center_bounds: Option<GeoBounds>,  // Where formation center can go
    pub individual_drone_bounds: Option<GeoBounds>,  // Individual drone limits
    pub minimum_clearance_m: f32,                    // Min distance between drones
    pub maximum_spread_m: f32,                       // Max formation diameter
    pub air_corridor_reserved: bool,                 // Reserve airspace for formation
}

/// Fragility classifications for payload handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Fragility {
    Robust,         // Can handle rough handling
    Sensitive,      // Requires careful handling
    Delicate,       // Minimal vibration/shock allowed
    Critical,       // Mission-critical with strict requirements
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSnapshot {
    pub timestamp: SystemTime,
    pub location: GeoCoordinate,
    pub temperature_c: f32,
    pub humidity_percent: f32,
    pub wind_speed_mps: f32,
    pub wind_direction_degrees: f32,
    pub gust_speed_mps: f32,
    pub visibility_m: f32,
    pub precipitation_type: Option<String>,
    pub precipitation_rate_mmh: f32,
    pub pressure_hpa: f32,
    pub cloud_cover_percent: f32,
    pub source: String,
}

impl Default for MissionPayload {
    fn default() -> Self {
        Self {
            header: MissionHeader {
                id: [0u8; 16],
                name: "Default Mission".to_string(),
                description: None,
                validity_start: SystemTime::now(),
                validity_end: SystemTime::now() + Duration::from_secs(3600),
                max_execution_duration: Duration::from_secs(1800),
                issuing_station_fingerprint: [0u8; 32],
                drone_fingerprint: None,
                priority: MissionPriority::Normal,
                tags: Vec::new(),
            },
            flight_plan: FlightPlan {
                paths: Vec::new(),
                home_location: GeoCoordinate {
                    latitude: 0.0,
                    longitude: 0.0,
                    altitude_msl: 0.0,
                },
                takeoff_procedure: None,
                landing_procedure: None,
                contingency_routes: Vec::new(),
            },
            tasks: Vec::new(),
            constraints: MissionConstraints {
                geofencing: Vec::new(),
                energy: EnergyConstraints {
                    min_soc_start: 0.2,
                    reserve_margin_soc: 0.1,
                    expected_consumption_wh: 100.0,
                    max_flight_time_minutes: 30,
                    power_profile: Vec::new(),
                },
                safety: SafetyConstraints {
                    max_wind_speed_mps: 10.0,
                    max_gust_speed_mps: 15.0,
                    min_visibility_m: 500.0,
                    max_proximity_to_crowd_m: 50.0,
                    emergency_landing_sites: Vec::new(),
                    fail_safe_procedures: vec!["RTL".to_string()],
                },
                environmental: EnvironmentalConstraints {
                    max_temperature_c: 40.0,
                    min_temperature_c: -10.0,
                    max_humidity_percent: 90.0,
                    max_precipitation_mmh: 10.0,
                    min_visibility_m: 300.0,
                    max_wind_speed_mps: 8.0,
                    max_gust_speed_mps: 12.0,
                    protected_weather_zones: Vec::new(),
                },
            },
            policies: MissionPolicies {
                authorization_scopes: vec![AuthorizationScope::ExecuteMission],
                time_limits: TimeLimits {
                    session_max_duration_hours: 2,
                    mission_max_duration_hours: 1,
                    authorization_refresh_hours: 6,
                    emergency_override_minutes: 5,
                },
                approval_requirements: Vec::new(),
                emergency_procedures: Vec::new(),
            },
            crypto: MissionCrypto {
                payload_signature: Vec::new(),
                channel_mac_binding: Vec::new(),
                nonce: [0u8; 16],
                timestamp: SystemTime::now(),
                session_key: None,
            },
            weather_snapshot: None,
            formation_config: None, // NEW: No formation by default
        }
    }
}
