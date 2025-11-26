//! Laser safety monitoring and power management

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

use super::types::{PowerProfile, BatteryState, PowerManagementConfig, PowerStatistics, PowerBudget, LaserType};
use super::error::LaserError;

#[derive(Debug)]
pub struct SafetyMonitor {
    last_activity: Instant,
    total_energy_joules: f64,
    eye_safety_violations: u32,
}

impl SafetyMonitor {
    pub fn new() -> Self {
        Self {
            last_activity: Instant::now(),
            total_energy_joules: 0.0,
            eye_safety_violations: 0,
        }
    }

    pub fn record_energy_usage(&mut self, power_mw: f32, duration_ms: u64) {
        let energy = power_mw as f64 * duration_ms as f64 / 1000.0; // Convert to Joules
        self.total_energy_joules += energy;
        self.last_activity = Instant::now();
    }

    pub fn record_safety_violation(&mut self) {
        self.eye_safety_violations += 1;
    }

    pub fn get_stats(&self) -> (f64, u32, std::time::Duration) {
        (self.total_energy_joules, self.eye_safety_violations, self.last_activity.elapsed())
    }

    pub fn reset(&mut self) {
        self.total_energy_joules = 0.0;
        self.eye_safety_violations = 0;
        self.last_activity = Instant::now();
    }
}

pub struct PowerManager {
    safety_monitor: Arc<Mutex<SafetyMonitor>>,
    current_profile: Arc<Mutex<PowerProfile>>,
    laser_type: LaserType,
    adaptive_mode: bool,
}

impl PowerManager {
    pub fn new(laser_type: LaserType) -> Self {
        Self {
            safety_monitor: Arc::new(Mutex::new(SafetyMonitor::new())),
            current_profile: Arc::new(Mutex::new(PowerProfile::default())),
            laser_type,
            adaptive_mode: false,
        }
    }

    pub async fn check_safety(&self) -> Result<(), LaserError> {
        let monitor = self.safety_monitor.lock().await;
        let profile = self.current_profile.lock().await;

        // Check eye safety limits based on current profile
        let safe_limit = profile.safe_power_limit(&self.laser_type);
        if profile.optimal_power_mw > safe_limit {
            return Err(LaserError::SafetyViolation);
        }

        // Check total energy usage
        if monitor.total_energy_joules > 1000.0 { // 1kJ limit
            return Err(LaserError::SafetyViolation);
        }

        Ok(())
    }

    pub async fn validate_power_level(&self, power_mw: f32) -> Result<(), LaserError> {
        // Safety check
        if power_mw > 1.0 || power_mw < 0.0 {
            return Err(LaserError::SafetyViolation);
        }

        // Get effective power limit from current profile
        let effective_limit = self.get_effective_power_limit().await;
        if power_mw > effective_limit {
            return Err(LaserError::SafetyViolation);
        }

        Ok(())
    }

    pub async fn record_power_usage(&self, power_mw: f32, duration_ms: u64) {
        let mut monitor = self.safety_monitor.lock().await;
        monitor.record_energy_usage(power_mw, duration_ms);
    }

    pub async fn is_power_safe(&self) -> bool {
        let monitor = self.safety_monitor.lock().await;
        let profile = self.current_profile.lock().await;

        // Check energy limits
        if monitor.total_energy_joules > 1000.0 {
            return false;
        }

        // Check power limits
        let safe_limit = profile.safe_power_limit(&self.laser_type);
        if profile.optimal_power_mw > safe_limit {
            return false;
        }

        true
    }

    pub async fn get_current_power_consumption(&self) -> f32 {
        let profile = self.current_profile.lock().await;
        profile.optimal_power_mw
    }

    pub async fn get_power_efficiency(&self) -> f32 {
        let monitor = self.safety_monitor.lock().await;
        let uptime_seconds = monitor.last_activity.elapsed().as_secs_f32();

        if uptime_seconds > 0.0 {
            // Efficiency as energy per second (lower is better)
            (monitor.total_energy_joules as f32) / uptime_seconds
        } else {
            0.0
        }
    }

    pub async fn get_effective_power_limit(&self) -> f32 {
        let profile = self.current_profile.lock().await;
        let safe_limit = profile.safe_power_limit(&self.laser_type);
        profile.max_power_mw.min(safe_limit)
    }

    pub async fn set_power_profile(&self, profile: PowerProfile) -> Result<(), LaserError> {
        // Validate profile against laser type safety limits
        let safe_limit = profile.safe_power_limit(&self.laser_type);
        if profile.optimal_power_mw > safe_limit {
            return Err(LaserError::SafetyViolation);
        }

        *self.current_profile.lock().await = profile;
        Ok(())
    }

    pub async fn get_current_power_profile(&self) -> PowerProfile {
        self.current_profile.lock().await.clone()
    }

    pub async fn optimize_power_usage(&mut self, battery_state: Option<&BatteryState>) -> Result<(), LaserError> {
        let mut profile = self.current_profile.lock().await;

        if let Some(battery) = battery_state {
            // Adaptive power scaling based on battery level
            let power_multiplier = if battery.capacity_percent < 20.0 {
                // Emergency power mode - reduce power significantly
                0.3
            } else if battery.capacity_percent < 50.0 {
                // Power saving mode
                0.6
            } else if battery.estimated_runtime_hours < 2.0 {
                // Low runtime - moderate power reduction
                0.8
            } else {
                // Normal operation
                1.0
            };

            // Apply temperature compensation
            let temp_multiplier = if battery.temperature_celsius > 40.0 {
                // High temperature - reduce power to prevent overheating
                0.7
            } else if battery.temperature_celsius < 0.0 {
                // Cold temperature - may need more power for efficiency
                1.1
            } else {
                1.0
            };

            let total_multiplier = power_multiplier * temp_multiplier;
            profile.optimal_power_mw *= total_multiplier;
            profile.optimal_power_mw = profile.optimal_power_mw.min(profile.max_power_mw);
            profile.optimal_power_mw = profile.optimal_power_mw.max(profile.min_power_mw);
        }

        Ok(())
    }

    pub async fn calculate_optimal_duty_cycle(&self, data_rate_bps: u32, required_power_mw: f32) -> f32 {
        // Duty cycle optimization for pulsed operation
        // Higher data rates may require higher duty cycles
        let base_duty_cycle = (data_rate_bps as f32 / 1_000_000.0).min(1.0); // Max 100% at 1Mbps

        // Adjust based on power requirements
        let power_factor = (required_power_mw / self.get_effective_power_limit().await).min(1.0);

        (base_duty_cycle * power_factor).max(0.1).min(1.0) // Keep between 10% and 100%
    }

    pub async fn predict_battery_drain(&self, operation_duration_seconds: f32) -> f32 {
        let current_power = self.get_current_power_consumption().await;
        let energy_consumed_joules = current_power as f32 * operation_duration_seconds / 1000.0;

        // Convert to battery percentage (simplified model)
        // Assuming 3000mAh battery at 3.7V = ~11.1Wh = 40,000J
        const BATTERY_CAPACITY_JOULES: f32 = 40_000.0;
        (energy_consumed_joules / BATTERY_CAPACITY_JOULES) * 100.0
    }

    pub async fn get_power_recommendations(&self, battery_state: Option<&BatteryState>) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(battery) = battery_state {
            if battery.capacity_percent < 15.0 {
                recommendations.push("Battery critically low. Switching to emergency power mode.".to_string());
            } else if battery.capacity_percent < 30.0 {
                recommendations.push("Battery low. Consider reducing transmission power.".to_string());
            }

            if battery.estimated_runtime_hours < 1.0 {
                recommendations.push("Estimated runtime very low. Enable burst mode for power saving.".to_string());
            }

            if battery.temperature_celsius > 45.0 {
                recommendations.push("High battery temperature detected. Reducing power to prevent damage.".to_string());
            }
        }

        let efficiency = self.get_power_efficiency().await;
        if efficiency < 0.5 {
            recommendations.push("Low power efficiency detected. Consider duty cycle optimization.".to_string());
        }

        recommendations
    }

    pub async fn calculate_power_budget(&self, operation: &str, duration_seconds: f32) -> PowerBudget {
        let current_power = self.get_current_power_consumption().await;
        let energy_required = current_power as f64 * duration_seconds as f64 / 1000.0; // Joules

        let battery_capacity = 40_000.0; // 40kJ typical battery capacity
        let available_energy = battery_capacity * 0.8; // 80% usable capacity

        let can_complete = energy_required <= available_energy;
        let estimated_drain_percent = (energy_required / battery_capacity * 100.0) as f32;

        PowerBudget {
            operation: operation.to_string(),
            energy_required_joules: energy_required,
            estimated_duration_seconds: duration_seconds,
            can_complete_operation: can_complete,
            estimated_battery_drain_percent: estimated_drain_percent,
            recommended_power_level_mw: if can_complete {
                current_power
            } else {
                (available_energy / duration_seconds as f64 * 1000.0) as f32
            },
        }
    }

    pub async fn emergency_shutdown(&self) -> Result<(), LaserError> {
        // Log emergency shutdown
        let mut monitor = self.safety_monitor.lock().await;
        monitor.record_safety_violation();

        Ok(())
    }

    pub async fn monitor_power_safety(&self) -> Result<(), LaserError> {
        if !self.is_power_safe().await {
            // Log safety violation
            let mut monitor = self.safety_monitor.lock().await;
            monitor.record_safety_violation();

            // Emergency shutdown if violations exceed threshold
            if monitor.eye_safety_violations > 3 {
                return self.emergency_shutdown().await;
            }

            // Reduce power to safe levels
            let profile = self.current_profile.lock().await;
            let safe_limit = profile.safe_power_limit(&self.laser_type);

            if profile.optimal_power_mw > safe_limit {
                // Would adjust power profile here
                // For now, just return error
                return Err(LaserError::SafetyViolation);
            }
        }

        Ok(())
    }

    pub async fn reset_energy_monitoring(&self) {
        let mut monitor = self.safety_monitor.lock().await;
        monitor.reset();
    }

    pub async fn get_safety_stats(&self) -> (f64, u32, std::time::Duration) {
        let monitor = self.safety_monitor.lock().await;
        monitor.get_stats()
    }

    pub fn enable_adaptive_mode(&mut self) {
        self.adaptive_mode = true;
    }

    pub fn disable_adaptive_mode(&mut self) {
        self.adaptive_mode = false;
    }

    pub fn is_adaptive_mode(&self) -> bool {
        self.adaptive_mode
    }
}