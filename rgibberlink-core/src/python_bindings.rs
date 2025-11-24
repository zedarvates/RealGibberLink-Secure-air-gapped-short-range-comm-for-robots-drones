use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::collections::HashMap;
use crate::crypto::{CryptoEngine, CryptoError};
use crate::visual::{VisualEngine, VisualError, VisualPayload};
use crate::audio::AudioEngine;
use crate::protocol::{ProtocolEngine, ProtocolError, ProtocolState};
use crate::RgibberLink;
use crate::weather::{WeatherManager, WeatherData, WeatherImpact, WindImpact, ConstraintValidationResult, ConstraintViolation, WeatherAdaptation, RiskAssessment, WeatherSource, DroneSpecifications};
use crate::mission::{MissionPayload, MissionHeader, MissionTask, GeoCoordinate};
use crate::audit::{AuditSystem, AuditEntry, SecurityAlert, AuditEventType, AuditSeverity, AuditActor, AuditOperation, create_audit_entry};

/// Python wrapper for CryptoEngine
#[pyclass]
pub struct PyCryptoEngine {
    inner: CryptoEngine,
}

#[pymethods]
impl PyCryptoEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: CryptoEngine::new(),
        }
    }

    fn public_key(&self) -> Vec<u8> {
        self.inner.public_key().to_vec()
    }

    fn derive_shared_secret(&mut self, peer_public_key: Vec<u8>) -> PyResult<[u8; 32]> {
        self.inner.derive_shared_secret(&peer_public_key)
            .map_err(|e| PyRuntimeError::new_err(format!("Crypto error: {}", e)))
    }

    #[staticmethod]
    fn encrypt_data(key: Vec<u8>, data: Vec<u8>) -> PyResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(PyRuntimeError::new_err("Key must be 32 bytes"));
        }
        let key_array: [u8; 32] = key.try_into().map_err(|_| PyRuntimeError::new_err("Invalid key length"))?;
        CryptoEngine::encrypt_data(&key_array, &data)
            .map_err(|e| PyRuntimeError::new_err(format!("Encryption error: {}", e)))
    }

    #[staticmethod]
    fn decrypt_data(key: Vec<u8>, encrypted_data: Vec<u8>) -> PyResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(PyRuntimeError::new_err("Key must be 32 bytes"));
        }
        let key_array: [u8; 32] = key.try_into().map_err(|_| PyRuntimeError::new_err("Invalid key length"))?;
        CryptoEngine::decrypt_data(&key_array, &encrypted_data)
            .map_err(|e| PyRuntimeError::new_err(format!("Decryption error: {}", e)))
    }

    #[staticmethod]
    fn generate_nonce() -> [u8; 16] {
        CryptoEngine::generate_nonce()
    }
}

/// Python wrapper for VisualEngine
#[pyclass]
pub struct PyVisualEngine {
    inner: VisualEngine,
}

#[pymethods]
impl PyVisualEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: VisualEngine::new(),
        }
    }

    fn encode_payload(&self, payload: &PyVisualPayload) -> PyResult<String> {
        self.inner.encode_payload(&payload.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Visual error: {}", e)))
    }

    fn decode_payload(&self, qr_data: Vec<u8>) -> PyResult<PyVisualPayload> {
        let payload = self.inner.decode_payload(&qr_data)
            .map_err(|e| PyRuntimeError::new_err(format!("Visual error: {}", e)))?;
        Ok(PyVisualPayload { inner: payload })
    }
}

/// Python wrapper for VisualPayload
#[pyclass]
#[derive(Clone)]
pub struct PyVisualPayload {
    inner: VisualPayload,
}

#[pymethods]
impl PyVisualPayload {
    #[new]
    fn new(session_id: [u8; 16], public_key: Vec<u8>, nonce: [u8; 16], signature: Vec<u8>) -> Self {
        Self {
            inner: VisualPayload {
                session_id,
                public_key,
                nonce,
                signature,
            },
        }
    }

    #[getter]
    fn session_id(&self) -> [u8; 16] {
        self.inner.session_id
    }

    #[getter]
    fn public_key(&self) -> Vec<u8> {
        self.inner.public_key.clone()
    }

    #[getter]
    fn nonce(&self) -> [u8; 16] {
        self.inner.nonce
    }

    #[getter]
    fn signature(&self) -> Vec<u8> {
        self.inner.signature.clone()
    }
}

/// Python wrapper for AudioEngine
#[pyclass]
pub struct PyAudioEngine {
    inner: AudioEngine,
}

#[pymethods]
impl PyAudioEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: AudioEngine::new(),
        }
    }

    fn send_data(&self, py: Python, data: Vec<u8>) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, return Ok since audio engine is not fully implemented
            Ok(())
        })
    }

    fn receive_data(&self, py: Python) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return empty data since audio engine is not fully implemented
            Ok(vec![])
        })
    }

    fn is_receiving(&self, py: Python) -> bool {
        py.allow_threads(|| false)
    }
}

/// Python wrapper for ProtocolEngine
#[pyclass]
pub struct PyProtocolEngine {
    inner: ProtocolEngine,
}

#[pymethods]
impl PyProtocolEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: ProtocolEngine::new(),
        }
    }

    fn initiate_handshake(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate handshake initiation
            Ok(())
        })
    }

    fn receive_nonce(&self, py: Python, nonce: Vec<u8>) -> PyResult<String> {
        py.allow_threads(|| {
            // For now, return a mock QR code
            Ok("<svg>Mock QR Code</svg>".to_string())
        })
    }

    fn process_qr_payload(&self, py: Python, qr_data: Vec<u8>) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate QR processing
            Ok(())
        })
    }

    fn receive_ack(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate ACK reception
            Ok(())
        })
    }

    fn get_state(&self, py: Python) -> PyResult<String> {
        py.allow_threads(|| {
            Ok("idle".to_string())
        })
    }

    fn encrypt_message(&self, py: Python, data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no encryption)
            Ok(data)
        })
    }

    fn decrypt_message(&self, py: Python, encrypted_data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no decryption)
            Ok(encrypted_data)
        })
    }
}

/// Python wrapper for RgibberLink
#[pyclass]
pub struct PyRgibberLink {
    inner: RgibberLink,
}

#[pymethods]
impl PyRgibberLink {
    #[new]
    fn new() -> Self {
        Self {
            inner: RgibberLink::new(),
        }
    }

    fn initiate_handshake(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate handshake initiation
            Ok(())
        })
    }

    fn receive_nonce(&self, py: Python, nonce: Vec<u8>) -> PyResult<String> {
        py.allow_threads(|| {
            // For now, return a mock QR code
            Ok("<svg>Mock QR Code</svg>".to_string())
        })
    }

    fn process_qr_payload(&self, py: Python, qr_data: Vec<u8>) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate QR processing
            Ok(())
        })
    }

    fn receive_ack(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate ACK reception
            Ok(())
        })
    }

    fn get_state(&self, py: Python) -> PyResult<String> {
        py.allow_threads(|| {
            Ok("idle".to_string())
        })
    }

    fn encrypt_message(&self, py: Python, data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no encryption)
            Ok(data)
        })
    }

    fn decrypt_message(&self, py: Python, encrypted_data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no decryption)
            Ok(encrypted_data)
        })
    }
}

/// Python wrapper for WeatherManager
#[pyclass]
pub struct PyWeatherManager {
    inner: WeatherManager,
}

#[pymethods]
impl PyWeatherManager {
    #[new]
    fn new(max_stations: usize) -> Self {
        Self {
            inner: WeatherManager::new(max_stations),
        }
    }

    fn update_weather(&mut self, weather_data: PyWeatherData) -> PyResult<()> {
        self.inner.update_weather(weather_data.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Weather error: {}", e)))
    }

    fn assess_weather_impact(&self, py: Python, mission: &PyMissionPayload, drone_specs: &PyDroneSpecifications) -> PyResult<PyWeatherImpact> {
        py.allow_threads(|| {
            let impact = self.inner.assess_weather_impact(&mission.inner, &drone_specs.inner)
                .map_err(|e| PyRuntimeError::new_err(format!("Weather assessment error: {}", e)))?;
            Ok(PyWeatherImpact { inner: impact })
        })
    }

    fn validate_mission_constraints(&self, py: Python, mission: &PyMissionPayload, drone_specs: &PyDroneSpecifications) -> PyResult<PyValidationResult> {
        py.allow_threads(|| {
            let result = self.inner.validate_mission_constraints(&mission.inner, &drone_specs.inner)
                .map_err(|e| PyRuntimeError::new_err(format!("Validation error: {}", e)))?;
            Ok(PyValidationResult { inner: result })
        })
    }
}

/// Python wrapper for WeatherData
#[pyclass]
#[derive(Clone)]
pub struct PyWeatherData {
    inner: WeatherData,
}

#[pymethods]
impl PyWeatherData {
    #[new]
    fn new(timestamp: f64, location: PyGeoCoordinate, temperature_celsius: f32, humidity_percent: f32,
           wind_speed_mps: f32, wind_direction_degrees: f32, gust_speed_mps: f32, visibility_meters: f32,
           precipitation_rate_mmh: f32, pressure_hpa: f32, cloud_cover_percent: f32, lightning_probability: f32) -> Self {
        Self {
            inner: WeatherData {
                timestamp: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs_f64(timestamp),
                location: location.inner,
                temperature_celsius,
                humidity_percent,
                wind_speed_mps,
                wind_direction_degrees,
                gust_speed_mps,
                visibility_meters,
                precipitation_type: None, // Not provided in constructor
                precipitation_rate_mmh,
                pressure_hpa,
                cloud_cover_percent,
                lightning_probability,
                source: WeatherSource::WeatherAPI, // Default
                forecast_horizon_hours: Some(6), // Default
            },
        }
    }
}

/// Python wrapper for GeoCoordinate
#[pyclass]
#[derive(Clone)]
pub struct PyGeoCoordinate {
    inner: GeoCoordinate,
}

#[pymethods]
impl PyGeoCoordinate {
    #[new]
    fn new(latitude: f64, longitude: f64, altitude_msl: f32) -> Self {
        Self {
            inner: GeoCoordinate {
                latitude,
                longitude,
                altitude_msl,
            },
        }
    }
}

/// Python wrapper for WeatherImpact
#[pyclass]
#[derive(Clone)]
pub struct PyWeatherImpact {
    inner: WeatherImpact,
}

#[pymethods]
impl PyWeatherImpact {
    #[getter]
    fn overall_risk_score(&self) -> f32 {
        self.inner.overall_risk_score
    }

    #[getter]
    fn wind_impact(&self) -> PyWindImpact {
        PyWindImpact { inner: self.inner.wind_impact.clone() }
    }

    #[getter]
    fn recommended_actions(&self) -> Vec<String> {
        self.inner.recommended_actions.clone()
    }
}

/// Python wrapper for WindImpact
#[pyclass]
#[derive(Clone)]
pub struct PyWindImpact {
    inner: WindImpact,
}

#[pymethods]
impl PyWindImpact {
    #[getter]
    fn track_deviation_degrees(&self) -> f32 {
        self.inner.track_deviation_degrees
    }

    #[getter]
    fn increased_power_draw_w(&self) -> f32 {
        self.inner.increased_power_draw_w
    }

    #[getter]
    fn reduced_endurance_percent(&self) -> f32 {
        self.inner.reduced_endurance_percent
    }

    #[getter]
    fn abort_threshold_exceeded(&self) -> bool {
        self.inner.abort_threshold_exceeded
    }
}

/// Python wrapper for ConstraintValidationResult
#[pyclass]
#[derive(Clone)]
pub struct PyValidationResult {
    inner: ConstraintValidationResult,
}

#[pymethods]
impl PyValidationResult {
    #[getter]
    fn is_valid(&self) -> bool {
        self.inner.is_valid
    }

    #[getter]
    fn violations(&self) -> Vec<PyConstraintViolation> {
        self.inner.violations.iter().map(|v| PyConstraintViolation { inner: v.clone() }).collect()
    }

    #[getter]
    fn weather_adaptations(&self) -> Vec<PyWeatherAdaptation> {
        self.inner.weather_adaptations.iter().map(|a| PyWeatherAdaptation { inner: a.clone() }).collect()
    }

    #[getter]
    fn risk_assessment(&self) -> PyRiskAssessment {
        PyRiskAssessment { inner: self.inner.risk_assessment.clone() }
    }
}

/// Python wrapper for ConstraintViolation
#[pyclass]
#[derive(Clone)]
pub struct PyConstraintViolation {
    inner: ConstraintViolation,
}

#[pymethods]
impl PyConstraintViolation {
    #[getter]
    fn constraint_type(&self) -> String {
        format!("{:?}", self.inner.constraint_type)
    }

    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }
}

/// Python wrapper for WeatherAdaptation
#[pyclass]
#[derive(Clone)]
pub struct PyWeatherAdaptation {
    inner: WeatherAdaptation,
}

#[pymethods]
impl PyWeatherAdaptation {
    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }
}

/// Python wrapper for RiskAssessment
#[pyclass]
#[derive(Clone)]
pub struct PyRiskAssessment {
    inner: RiskAssessment,
}

#[pymethods]
impl PyRiskAssessment {
    #[getter]
    fn overall_risk_level(&self) -> String {
        format!("{:?}", self.inner.overall_risk_level)
    }

    #[getter]
    fn confidence_score(&self) -> f32 {
        self.inner.confidence_score
    }
}

/// Python wrapper for MissionPayload
#[pyclass]
#[derive(Clone)]
pub struct PyMissionPayload {
    inner: MissionPayload,
}

#[pymethods]
impl PyMissionPayload {
    #[new]
    fn new(name: String, mission_id: [u8; 16]) -> Self {
        let mut mission = MissionPayload::default();
        mission.header.id = mission_id;
        mission.header.name = name;
        Self { inner: mission }
    }

    #[getter]
    fn header(&self) -> PyMissionHeader {
        PyMissionHeader { inner: self.inner.header.clone() }
    }

    #[getter]
    fn tasks(&self) -> Vec<PyMissionTask> {
        self.inner.tasks.iter().map(|t| PyMissionTask { inner: t.clone() }).collect()
    }
}

/// Python wrapper for MissionHeader
#[pyclass]
#[derive(Clone)]
pub struct PyMissionHeader {
    inner: MissionHeader,
}

#[pymethods]
impl PyMissionHeader {
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn priority(&self) -> String {
        format!("{:?}", self.inner.priority)
    }
}

/// Python wrapper for MissionTask
#[pyclass]
#[derive(Clone)]
pub struct PyMissionTask {
    inner: MissionTask,
}

#[pymethods]
impl PyMissionTask {
    #[getter]
    fn label(&self) -> String {
        self.inner.label.clone()
    }

    #[getter]
    fn sequence_order(&self) -> u32 {
        self.inner.sequence_order
    }
}

/// Python wrapper for DroneSpecifications
#[pyclass]
#[derive(Clone)]
pub struct PyDroneSpecifications {
    inner: DroneSpecifications,
}

#[pymethods]
impl PyDroneSpecifications {
    #[new]
    fn new(max_wind_speed_mps: f32, max_speed_mps: f32, abort_gust_threshold_mps: f32, power_wind_coefficient: f32, mass_kg: f32, battery_capacity_wh: f32, sensor_count: usize) -> Self {
        Self {
            inner: DroneSpecifications {
                max_wind_speed_mps,
                max_speed_mps,
                abort_gust_threshold_mps,
                power_wind_coefficient,
                mass_kg,
                battery_capacity_wh,
                sensor_types: vec!["sensor".to_string(); sensor_count], // Placeholder
            },
        }
    }
}

/// Python wrapper for AuditSystem
#[pyclass]
pub struct PyAuditSystem {
    inner: AuditSystem,
}

#[pymethods]
impl PyAuditSystem {
    #[new]
    fn new(max_entries: usize) -> Self {
        Self {
            inner: AuditSystem::new(max_entries),
        }
    }

    fn record_event(&mut self, py: Python, event: PyAuditEntry) -> PyResult<String> {
        py.allow_threads(|| {
            self.inner.record_event(event.inner)
                .map_err(|e| PyRuntimeError::new_err(format!("Audit error: {}", e)))
        })
    }

    fn get_active_alerts(&self) -> Vec<PySecurityAlert> {
        self.inner.get_active_alerts().iter().map(|a| PySecurityAlert { inner: (*a).clone() }).collect()
    }
}

/// Python wrapper for AuditEntry
#[pyclass]
#[derive(Clone)]
pub struct PyAuditEntry {
    inner: AuditEntry,
}

#[pymethods]
impl PyAuditEntry {
    #[new]
    fn new(event_type: String, severity: String, actor: String, operation: String, success: bool) -> Self {
        // Simplified constructor - would need full implementation
        let audit_entry = create_audit_entry(
            match event_type.as_str() {
                "MissionTransfer" => AuditEventType::MissionTransfer,
                _ => AuditEventType::MissionTransfer,
            },
            match severity.as_str() {
                "High" => AuditSeverity::High,
                _ => AuditSeverity::Medium,
            },
            match actor.as_str() {
                "Operator" => AuditActor::HumanOperator {
                    operator_id: "operator_1".to_string(),
                    clearance_level: "standard".to_string(),
                    department: None,
                },
                _ => AuditActor::System {
                    component: "unknown".to_string(),
                    version: "1.0".to_string(),
                    subsystem: "mission".to_string(),
                },
            },
            AuditOperation {
                operation_type: "mission".to_string(),
                operation_name: operation,
                parameters: HashMap::new(),
                execution_context: crate::audit::OperationContext::default(),
                expected_duration: None,
                resource_consumption: crate::audit::ResourceConsumption::default(),
            },
            crate::audit::OperationResult {
                success,
                error_code: None,
                error_message: None,
                duration_ms: 100,
                performance_metrics: crate::audit::PerformanceMetrics::default(),
                side_effects: vec![],
            },
            crate::audit::AuditContext::default(),
        );

        Self { inner: audit_entry }
    }
}

/// Python wrapper for SecurityAlert
#[pyclass]
#[derive(Clone)]
pub struct PySecurityAlert {
    inner: SecurityAlert,
}

#[pymethods]
impl PySecurityAlert {
    #[getter]
    fn severity(&self) -> String {
        "High".to_string()
    }

    #[getter]
    fn title(&self) -> String {
        self.inner.title.clone()
    }
}

/// Main Python module
#[pymodule]
fn gibberlink_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCryptoEngine>()?;
    m.add_class::<PyVisualEngine>()?;
    m.add_class::<PyVisualPayload>()?;
    m.add_class::<PyAudioEngine>()?;
    m.add_class::<PyProtocolEngine>()?;
    m.add_class::<PyRgibberLink>()?;
    m.add_class::<PyWeatherManager>()?;
    m.add_class::<PyWeatherData>()?;
    m.add_class::<PyGeoCoordinate>()?;
    m.add_class::<PyWeatherImpact>()?;
    m.add_class::<PyWindImpact>()?;
    m.add_class::<PyValidationResult>()?;
    m.add_class::<PyConstraintViolation>()?;
    m.add_class::<PyWeatherAdaptation>()?;
    m.add_class::<PyRiskAssessment>()?;
    m.add_class::<PyMissionPayload>()?;
    m.add_class::<PyMissionHeader>()?;
    m.add_class::<PyMissionTask>()?;
    m.add_class::<PyDroneSpecifications>()?;
    m.add_class::<PyAuditSystem>()?;
    m.add_class::<PyAuditEntry>()?;
    m.add_class::<PySecurityAlert>()?;
    Ok(())
}
