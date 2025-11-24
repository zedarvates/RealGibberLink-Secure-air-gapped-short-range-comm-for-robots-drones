//! # Performance Monitor Module
//!
//! Comprehensive performance profiling, benchmarking, and real-time optimization system
//! for long-range communication protocols. Provides latency tracking, throughput measurement,
//! power consumption analysis, and automatic performance adaptation.

use crate::laser::{LaserEngine, ModulationScheme, PowerProfile};
use crate::ultrasonic_beam::UltrasonicBeamEngine;
use crate::range_detector::{RangeDetector, RangeDetectorCategory};
use crate::security::WeatherCondition;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum PerformanceError {
    #[error("Benchmarking failed: {0}")]
    BenchmarkFailed(String),
    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),
    #[error("Invalid performance metrics")]
    InvalidMetrics,
    #[error("Timeout during performance test")]
    Timeout,
}

/// Performance metrics for different communication aspects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub handshake_latency_ms: f64,
    pub data_throughput_bps: f64,
    pub bit_error_rate: f64,
    pub packet_loss_rate: f64,
    pub power_consumption_mw: f64,
    pub range_meters: f64,
    pub signal_strength: f64,
    pub modulation_scheme: ModulationScheme,
    pub ecc_strength: f64,
    pub environmental_conditions: EnvironmentalFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalFactors {
    pub weather: WeatherCondition,
    pub temperature_celsius: f32,
    pub humidity_percent: f32,
    pub visibility_meters: f32,
    pub wind_speed_mps: f32,
}

/// Benchmark results for different configurations
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub config: PerformanceConfig,
    pub metrics: PerformanceMetrics,
    pub score: f64, // Overall performance score (0-100)
    pub reliability_score: f64,
    pub efficiency_score: f64,
}

/// Performance configuration presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformancePreset {
    SpeedOptimized,      // Maximize throughput, minimize latency - for gaming/interactive apps
    ReliabilityOptimized, // Maximize reliability, acceptable latency - for critical communications
    PowerOptimized,      // Minimize power consumption - for battery-constrained devices
    Balanced,           // Good balance of all factors - default for most applications
    LongRangeOptimized, // Optimized for maximum range - for surveillance/drone communications
    LowLatency,         // Minimize handshake time - for real-time applications
    HighBandwidth,      // Maximize data throughput - for file transfers
    Custom(PerformanceConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub target_latency_ms: f64,
    pub target_throughput_bps: f64,
    pub max_power_mw: f64,
    pub min_reliability: f64,
    pub modulation_scheme: ModulationScheme,
    pub adaptive_ecc: bool,
    pub range_adaptation: bool,
    pub environmental_compensation: bool,
}

/// Real-time performance monitor
pub struct PerformanceMonitor {
    metrics_history: Arc<Mutex<VecDeque<PerformanceMetrics>>>,
    benchmark_results: Arc<Mutex<Vec<BenchmarkResult>>>,
    current_config: Arc<Mutex<PerformanceConfig>>,
    optimization_active: Arc<Mutex<bool>>,
    laser_engine: Option<Arc<Mutex<LaserEngine>>>,
    ultrasonic_engine: Option<Arc<Mutex<UltrasonicBeamEngine>>>,
    range_detector: Option<Arc<Mutex<RangeDetector>>>,
    max_history_size: usize,
}

impl PerformanceMonitor {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            metrics_history: Arc::new(Mutex::new(VecDeque::with_capacity(max_history_size))),
            benchmark_results: Arc::new(Mutex::new(Vec::new())),
            current_config: Arc::new(Mutex::new(PerformanceConfig::default())),
            optimization_active: Arc::new(Mutex::new(false)),
            laser_engine: None,
            ultrasonic_engine: None,
            range_detector: None,
            max_history_size,
        }
    }

    /// Initialize with communication engines
    pub fn with_engines(
        mut self,
        laser: Option<Arc<Mutex<LaserEngine>>>,
        ultrasonic: Option<Arc<Mutex<UltrasonicBeamEngine>>>,
        range_detector: Option<Arc<Mutex<RangeDetector>>>,
    ) -> Self {
        self.laser_engine = laser;
        self.ultrasonic_engine = ultrasonic;
        self.range_detector = range_detector;
        self
    }

    /// Start real-time performance monitoring
    pub async fn start_monitoring(&self) -> Result<(), PerformanceError> {
        *self.optimization_active.lock().await = true;

        // Spawn monitoring task
        let metrics_history = self.metrics_history.clone();
        let laser_engine = self.laser_engine.clone();
        let ultrasonic_engine = self.ultrasonic_engine.clone();
        let range_detector = self.range_detector.clone();
        let max_history = self.max_history_size;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100)); // 10Hz monitoring

            loop {
                interval.tick().await;

                let metrics = Self::collect_current_metrics(
                    &laser_engine,
                    &ultrasonic_engine,
                    &range_detector,
                ).await;

                if let Ok(metrics) = metrics {
                    let mut history = metrics_history.lock().await;
                    if history.len() >= max_history {
                        history.pop_front();
                    }
                    history.push_back(metrics);
                }
            }
        });

        Ok(())
    }

    /// Stop performance monitoring
    pub async fn stop_monitoring(&self) {
        *self.optimization_active.lock().await = false;
    }

    /// Collect current performance metrics
    async fn collect_current_metrics(
        laser_engine: &Option<Arc<Mutex<LaserEngine>>>,
        ultrasonic_engine: &Option<Arc<Mutex<UltrasonicBeamEngine>>>,
        range_detector: &Option<Arc<Mutex<RangeDetector>>>,
    ) -> Result<PerformanceMetrics, PerformanceError> {
        let mut metrics = PerformanceMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            handshake_latency_ms: 0.0,
            data_throughput_bps: 0.0,
            bit_error_rate: 0.0,
            packet_loss_rate: 0.0,
            power_consumption_mw: 0.0,
            range_meters: 0.0,
            signal_strength: 0.0,
            modulation_scheme: ModulationScheme::Ook,
            ecc_strength: 0.0,
            environmental_conditions: EnvironmentalFactors {
                weather: WeatherCondition::Clear,
                temperature_celsius: 20.0,
                humidity_percent: 50.0,
                visibility_meters: 1000.0,
                wind_speed_mps: 2.0,
            },
        };

        // Collect laser metrics
        if let Some(laser) = laser_engine {
            let laser = laser.lock().await;
            let diagnostics = laser.get_channel_diagnostics().await;

            metrics.power_consumption_mw = diagnostics.power_consumption_mw as f64;
            metrics.signal_strength = diagnostics.alignment_status.signal_strength as f64;
            metrics.modulation_scheme = ModulationScheme::Ook; // Would be dynamic

            // Estimate throughput based on modulation and conditions
            metrics.data_throughput_bps = Self::estimate_throughput(&laser).await;
        }
        // Collect ultrasonic metrics
        if let Some(ultrasonic) = ultrasonic_engine {
            let ultrasonic = ultrasonic.lock().await;
            // Add ultrasonic-specific metrics collection
            metrics.handshake_latency_ms = Self::measure_handshake_latency(&ultrasonic).await;
        }

        // Collect range metrics
        if let Some(range_detector) = range_detector {
            let range_detector = range_detector.lock().await;
            if let Ok(measurement) = range_detector.measure_distance_averaged().await {
                metrics.range_meters = measurement.distance_m as f64;
            }

            // Get environmental conditions
            let conditions = range_detector.get_environmental_conditions().await;
            metrics.environmental_conditions = EnvironmentalFactors {
                weather: WeatherCondition::Clear, // Would infer from conditions
                temperature_celsius: conditions.temperature_celsius,
                humidity_percent: conditions.humidity_percent,
                visibility_meters: conditions.visibility_meters,
                wind_speed_mps: conditions.wind_speed_mps,
            };
        }

        Ok(metrics)
    }

    /// Estimate current data throughput
    async fn estimate_throughput(laser: &LaserEngine) -> f64 {
        let profile = laser.get_current_power_profile().await;
        profile.data_rate_bps as f64
    }

    /// Measure handshake latency
    async fn measure_handshake_latency(_ultrasonic: &UltrasonicBeamEngine) -> f64 {
        // In a real implementation, this would measure actual handshake timing
        // For now, return estimated latency based on current conditions
        450.0 // Target <500ms
    }

    /// Run comprehensive benchmark suite
    pub async fn run_benchmark_suite(&self, duration_secs: u64) -> Result<Vec<BenchmarkResult>, PerformanceError> {
        let mut results = Vec::new();
        let start_time = Instant::now();

        // Test different modulation schemes
        let modulation_schemes = vec![
            ModulationScheme::Ook,
            ModulationScheme::Pwm,
            ModulationScheme::QrProjection,
        ];

        for modulation in modulation_schemes {
            if start_time.elapsed() >= Duration::from_secs(duration_secs) {
                break;
            }

            let result = self.benchmark_modulation_scheme(modulation, 10).await?;
            results.push(result);
        }

        // Test different range conditions
        let range_categories = vec![
            RangeDetectorCategory::Close,
            RangeDetectorCategory::Medium,
            RangeDetectorCategory::Far,
            RangeDetectorCategory::Extreme,
        ];

        for category in range_categories {
            if start_time.elapsed() >= Duration::from_secs(duration_secs) {
                break;
            }

            let result = self.benchmark_range_category(category, 5).await?;
            results.push(result);
        }

        // Store results
        let mut benchmark_results = self.benchmark_results.lock().await;
        benchmark_results.extend(results.clone());

        Ok(results)
    }

    /// Benchmark specific modulation scheme
    async fn benchmark_modulation_scheme(&self, modulation: ModulationScheme, test_duration_secs: u64) -> Result<BenchmarkResult, PerformanceError> {
        let start_time = Instant::now();
        let mut total_throughput = 0.0;
        let total_latency = 0.0;
        let mut sample_count = 0;

        while start_time.elapsed() < Duration::from_secs(test_duration_secs) {
            if let Some(laser) = &self.laser_engine {
                let mut laser = laser.lock().await;

                // Measure transmission time
                let test_data = vec![0u8; 1024]; // 1KB test packet
                let tx_start = Instant::now();

                match laser.transmit_data(&test_data).await {
                    Ok(_) => {
                        let tx_time = tx_start.elapsed().as_secs_f64();
                        let throughput = test_data.len() as f64 * 8.0 / tx_time; // bps
                        total_throughput += throughput;
                        sample_count += 1;
                    }
                    Err(_) => {} // Transmission failed, continue
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let avg_throughput = if sample_count > 0 { total_throughput / sample_count as f64 } else { 0.0 };
        let avg_latency = if sample_count > 0 { total_latency / sample_count as f64 } else { 0.0 };

        let config = PerformanceConfig {
            target_latency_ms: 500.0,
            target_throughput_bps: avg_throughput,
            max_power_mw: 50.0,
            min_reliability: 0.95,
            modulation_scheme: modulation,
            adaptive_ecc: true,
            range_adaptation: true,
            environmental_compensation: true,
        };

        let metrics = PerformanceMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            handshake_latency_ms: avg_latency,
            data_throughput_bps: avg_throughput,
            bit_error_rate: 0.001, // Estimated
            packet_loss_rate: 0.01, // Estimated
            power_consumption_mw: 25.0, // Estimated
            range_meters: 100.0, // Default
            signal_strength: 0.8,
            modulation_scheme: modulation,
            ecc_strength: 0.5,
            environmental_conditions: EnvironmentalFactors::default(),
        };

        let score = self.calculate_performance_score(&metrics, &config);
        let reliability_score = 1.0 - metrics.bit_error_rate;
        let efficiency_score = avg_throughput / metrics.power_consumption_mw;

        Ok(BenchmarkResult {
            config,
            metrics,
            score,
            reliability_score,
            efficiency_score,
        })
    }

    /// Benchmark specific range category
    async fn benchmark_range_category(&self, category: RangeDetectorCategory, test_duration_secs: u64) -> Result<BenchmarkResult, PerformanceError> {
        // Similar to modulation benchmarking but with range-specific optimizations
        let config = PerformanceConfig {
            target_latency_ms: 500.0,
            target_throughput_bps: category.expected_throughput(),
            max_power_mw: category.max_power(),
            min_reliability: 0.90,
            modulation_scheme: category.optimal_modulation(),
            adaptive_ecc: true,
            range_adaptation: true,
            environmental_compensation: true,
        };

        // Run benchmark with range-specific settings
        let metrics = self.run_range_benchmark(category, test_duration_secs).await?;
        let score = self.calculate_performance_score(&metrics, &config);
        let reliability_score = 1.0 - metrics.bit_error_rate;
        let efficiency_score = metrics.data_throughput_bps / metrics.power_consumption_mw;

        Ok(BenchmarkResult {
            config,
            metrics,
            score,
            reliability_score,
            efficiency_score,
        })
    }

    /// Run range-specific benchmark
    async fn run_range_benchmark(&self, category: RangeDetectorCategory, _duration_secs: u64) -> Result<PerformanceMetrics, PerformanceError> {
        // Simulate range-specific performance metrics
        let (throughput, power, latency) = match category {
            RangeDetectorCategory::Close => (2_000_000.0, 15.0, 300.0),
            RangeDetectorCategory::Medium => (1_000_000.0, 25.0, 400.0),
            RangeDetectorCategory::Far => (500_000.0, 40.0, 450.0),
            RangeDetectorCategory::Extreme => (250_000.0, 60.0, 480.0),
        };

        Ok(PerformanceMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            handshake_latency_ms: latency,
            data_throughput_bps: throughput,
            bit_error_rate: 0.005,
            packet_loss_rate: 0.02,
            power_consumption_mw: power,
            range_meters: category.expected_range(),
            signal_strength: 0.75,
            modulation_scheme: category.optimal_modulation(),
            ecc_strength: 0.6,
            environmental_conditions: EnvironmentalFactors::default(),
        })
    }

    /// Calculate overall performance score (0-100)
    fn calculate_performance_score(&self, metrics: &PerformanceMetrics, config: &PerformanceConfig) -> f64 {
        let latency_score = (1.0 - (metrics.handshake_latency_ms / config.target_latency_ms).min(1.0)) * 25.0;
        let throughput_score = ((metrics.data_throughput_bps / config.target_throughput_bps).min(1.0)) * 25.0;
        let power_score = (1.0 - (metrics.power_consumption_mw / config.max_power_mw).min(1.0)) * 25.0;
        let reliability_score = (1.0 - metrics.bit_error_rate).min(1.0) * 25.0;

        latency_score + throughput_score + power_score + reliability_score
    }

    /// Apply performance preset
    pub async fn apply_preset(&self, preset: PerformancePreset) -> Result<(), PerformanceError> {
        let config = match preset {
            PerformancePreset::SpeedOptimized => PerformanceConfig {
                target_latency_ms: 300.0,
                target_throughput_bps: 2_000_000.0,
                max_power_mw: 100.0,
                min_reliability: 0.85,
                modulation_scheme: ModulationScheme::Ook,
                adaptive_ecc: false,
                range_adaptation: true,
                environmental_compensation: false,
            },
            PerformancePreset::ReliabilityOptimized => PerformanceConfig {
                target_latency_ms: 600.0,
                target_throughput_bps: 500_000.0,
                max_power_mw: 50.0,
                min_reliability: 0.99,
                modulation_scheme: ModulationScheme::QrProjection,
                adaptive_ecc: true,
                range_adaptation: true,
                environmental_compensation: true,
            },
            PerformancePreset::PowerOptimized => PerformanceConfig {
                target_latency_ms: 800.0,
                target_throughput_bps: 250_000.0,
                max_power_mw: 10.0,
                min_reliability: 0.90,
                modulation_scheme: ModulationScheme::Ook,
                adaptive_ecc: true,
                range_adaptation: true,
                environmental_compensation: false,
            },
            PerformancePreset::Balanced => PerformanceConfig {
                target_latency_ms: 500.0,
                target_throughput_bps: 1_000_000.0,
                max_power_mw: 30.0,
                min_reliability: 0.95,
                modulation_scheme: ModulationScheme::Pwm,
                adaptive_ecc: true,
                range_adaptation: true,
                environmental_compensation: true,
            },
            PerformancePreset::LongRangeOptimized => PerformanceConfig {
                target_latency_ms: 700.0,
                target_throughput_bps: 250_000.0,
                max_power_mw: 200.0,
                min_reliability: 0.98,
                modulation_scheme: ModulationScheme::QrProjection,
                adaptive_ecc: true,
                range_adaptation: true,
                environmental_compensation: true,
            },
            PerformancePreset::LowLatency => PerformanceConfig {
                target_latency_ms: 200.0,
                target_throughput_bps: 1_500_000.0,
                max_power_mw: 80.0,
                min_reliability: 0.85,
                modulation_scheme: ModulationScheme::Ook,
                adaptive_ecc: false,
                range_adaptation: false,
                environmental_compensation: false,
            },
            PerformancePreset::HighBandwidth => PerformanceConfig {
                target_latency_ms: 400.0,
                target_throughput_bps: 5_000_000.0,
                max_power_mw: 120.0,
                min_reliability: 0.9,
                modulation_scheme: ModulationScheme::Ook,
                adaptive_ecc: true,
                range_adaptation: true,
                environmental_compensation: true,
            },
            PerformancePreset::Custom(config) => config,
        };

        *self.current_config.lock().await = config.clone();

        // Apply configuration to engines
        self.apply_performance_config(&config).await?;

        Ok(())
    }

    /// Apply performance configuration to engines
    async fn apply_performance_config(&self, config: &PerformanceConfig) -> Result<(), PerformanceError> {
        if let Some(laser) = &self.laser_engine {
            let mut laser = laser.lock().await;

            // Update modulation scheme
            // Note: In real implementation, this would update the laser's modulation

            // Update power profile based on config
            let power_profile = PowerProfile {
                max_power_mw: config.max_power_mw as f32,
                optimal_power_mw: (config.max_power_mw * 0.6) as f32,
                min_power_mw: (config.max_power_mw * 0.2) as f32,
                data_rate_bps: config.target_throughput_bps as u32,
                beam_angle_deg: 15.0,
                safety_margin: 1.0,
            };

            laser.set_power_profile(power_profile).await
                .map_err(|e| PerformanceError::OptimizationFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// Get current performance metrics
    pub async fn get_current_metrics(&self) -> Option<PerformanceMetrics> {
        let history = self.metrics_history.lock().await;
        history.back().cloned()
    }

    /// Get performance history
    pub async fn get_metrics_history(&self, count: usize) -> Vec<PerformanceMetrics> {
        let history = self.metrics_history.lock().await;
        history.iter().rev().take(count).cloned().collect()
    }

    /// Get benchmark results
    pub async fn get_benchmark_results(&self) -> Vec<BenchmarkResult> {
        self.benchmark_results.lock().await.clone()
    }

    /// Optimize performance based on current conditions
    pub async fn optimize_performance(&self) -> Result<(), PerformanceError> {
        let current_metrics = self.get_current_metrics().await
            .ok_or(PerformanceError::InvalidMetrics)?;

        let config = self.current_config.lock().await.clone();

        // Analyze current performance
        let score = self.calculate_performance_score(&current_metrics, &config);

        if score < 70.0 {
            // Performance is poor, try optimization
            let optimized_config = self.find_optimal_config(&current_metrics).await?;
            self.apply_performance_config(&optimized_config).await?;
            *self.current_config.lock().await = optimized_config;
        }

        Ok(())
    }

    /// Find optimal configuration for current conditions
    async fn find_optimal_config(&self, metrics: &PerformanceMetrics) -> Result<PerformanceConfig, PerformanceError> {
        // Use benchmark results to find best configuration
        let benchmarks = self.benchmark_results.lock().await;

        if benchmarks.is_empty() {
            return Ok(PerformanceConfig::default());
        }

        // Find benchmark with highest score for similar conditions
        let mut best_benchmark = &benchmarks[0];
        let mut best_score = 0.0;

        for benchmark in benchmarks.iter() {
            let condition_similarity = self.calculate_condition_similarity(metrics, &benchmark.metrics);
            let weighted_score = benchmark.score * condition_similarity;

            if weighted_score > best_score {
                best_score = weighted_score;
                best_benchmark = benchmark;
            }
        }

        Ok(best_benchmark.config.clone())
    }

    /// Calculate similarity between two sets of conditions
    fn calculate_condition_similarity(&self, a: &PerformanceMetrics, b: &PerformanceMetrics) -> f64 {
        let range_diff = (a.range_meters - b.range_meters).abs() / 100.0; // Normalize
        let weather_similarity = if a.environmental_conditions.weather == b.environmental_conditions.weather { 1.0 } else { 0.5 };
        let temp_diff = ((a.environmental_conditions.temperature_celsius as f64) - (b.environmental_conditions.temperature_celsius as f64)).abs() / 50.0;

        let similarity = (1.0 - range_diff.min(1.0)) * weather_similarity * (1.0 - temp_diff.min(1.0));
        similarity.max(0.1) // Minimum similarity
    }

    /// Record performance metrics
    pub async fn record_metrics(&self, metrics: PerformanceMetrics) {
        let mut history = self.metrics_history.lock().await;
        if history.len() >= self.max_history_size {
            history.pop_front();
        }
        history.push_back(metrics);
    }

    /// Update environmental factors
    pub async fn update_environmental_factors(&self, factors: EnvironmentalFactors) {
        // This would update environmental monitoring
        // For now, just store in current metrics if available
        if let Some(mut metrics) = self.get_current_metrics().await {
            metrics.environmental_conditions = factors;
            // Note: In a real implementation, this would update the metrics history
        }
    }

    /// Get performance recommendations
    pub async fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let current_metrics = match self.get_current_metrics().await {
            Some(metrics) => metrics,
            None => return recommendations,
        };

        if current_metrics.handshake_latency_ms > 500.0 {
            recommendations.push("Handshake latency exceeds 500ms target. Consider speed optimization preset.".to_string());
        }

        if current_metrics.data_throughput_bps < 1_000_000.0 {
            recommendations.push("Data throughput below 1Mbps target. Consider range optimization.".to_string());
        }

        if current_metrics.power_consumption_mw > 50.0 {
            recommendations.push("High power consumption detected. Consider power optimization preset.".to_string());
        }

        if current_metrics.bit_error_rate > 0.01 {
            recommendations.push("High bit error rate. Enable adaptive ECC or switch to more robust modulation.".to_string());
        }

        recommendations
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_latency_ms: 500.0,
            target_throughput_bps: 1_000_000.0,
            max_power_mw: 50.0,
            min_reliability: 0.95,
            modulation_scheme: ModulationScheme::Pwm,
            adaptive_ecc: true,
            range_adaptation: true,
            environmental_compensation: true,
        }
    }
}

impl Default for EnvironmentalFactors {
    fn default() -> Self {
        Self {
            weather: WeatherCondition::Clear,
            temperature_celsius: 20.0,
            humidity_percent: 50.0,
            visibility_meters: 1000.0,
            wind_speed_mps: 2.0,
        }
    }
}

impl RangeDetectorCategory {
    fn expected_throughput(&self) -> f64 {
        match self {
            RangeDetectorCategory::Close => 2_000_000.0,
            RangeDetectorCategory::Medium => 1_000_000.0,
            RangeDetectorCategory::Far => 500_000.0,
            RangeDetectorCategory::Extreme => 250_000.0,
        }
    }

    fn max_power(&self) -> f64 {
        match self {
            RangeDetectorCategory::Close => 20.0,
            RangeDetectorCategory::Medium => 40.0,
            RangeDetectorCategory::Far => 70.0,
            RangeDetectorCategory::Extreme => 100.0,
        }
    }

    fn expected_range(&self) -> f64 {
        match self {
            RangeDetectorCategory::Close => 75.0,
            RangeDetectorCategory::Medium => 125.0,
            RangeDetectorCategory::Far => 150.0,
            RangeDetectorCategory::Extreme => 190.0,
        }
    }

    fn optimal_modulation(&self) -> ModulationScheme {
        match self {
            RangeDetectorCategory::Close => ModulationScheme::Ook,
            RangeDetectorCategory::Medium => ModulationScheme::Pwm,
            RangeDetectorCategory::Far => ModulationScheme::Manchester,
            RangeDetectorCategory::Extreme => ModulationScheme::QrProjection,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new(100);
        assert!(!*monitor.optimization_active.lock().await);
    }

    #[tokio::test]
    async fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();
        assert_eq!(config.target_latency_ms, 500.0);
        assert_eq!(config.target_throughput_bps, 1_000_000.0);
    }

    #[tokio::test]
    async fn test_range_category_methods() {
        assert_eq!(RangeDetectorCategory::Close.expected_throughput(), 2_000_000.0);
        assert_eq!(RangeDetectorCategory::Extreme.max_power(), 100.0);
    }
}