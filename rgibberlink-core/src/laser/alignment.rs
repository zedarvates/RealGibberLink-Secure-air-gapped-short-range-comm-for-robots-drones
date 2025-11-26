//! Laser beam alignment and tracking

use std::collections::VecDeque;
use tokio::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::AlignmentStatus;
use super::error::LaserError;

#[derive(Debug, Clone)]
pub struct AlignmentStatus {
    pub is_aligned: bool,
    pub beam_position_x: f32,
    pub beam_position_y: f32,
    pub signal_strength: f32,
    pub last_update: Instant,
}

/// Simple Kalman filter for position tracking and prediction
#[derive(Debug)]
struct KalmanFilter {
    // State vector: [x, y, vx, vy] (position and velocity)
    state: [f32; 4],
    // State covariance matrix (simplified as diagonal)
    covariance: [f32; 4],
    // Process noise
    process_noise: f32,
    // Measurement noise
    measurement_noise: f32,
}

impl KalmanFilter {
    fn new() -> Self {
        Self {
            state: [0.0; 4],
            covariance: [1.0; 4], // Initial uncertainty
            process_noise: 0.1,
            measurement_noise: 0.5,
        }
    }

    /// Predict next state
    fn predict(&mut self, dt: f32) {
        // State transition: position += velocity * dt
        self.state[0] += self.state[2] * dt; // x += vx * dt
        self.state[1] += self.state[3] * dt; // y += vy * dt

        // Update covariance with process noise
        for i in 0..4 {
            self.covariance[i] += self.process_noise;
        }
    }

    /// Update with measurement
    fn update(&mut self, measurement: (f32, f32)) {
        // Kalman gain (simplified)
        let kx = self.covariance[0] / (self.covariance[0] + self.measurement_noise);
        let ky = self.covariance[1] / (self.covariance[1] + self.measurement_noise);

        // Update state
        let innovation_x = measurement.0 - self.state[0];
        let innovation_y = measurement.1 - self.state[1];

        self.state[0] += kx * innovation_x;
        self.state[1] += ky * innovation_y;

        // Update covariance
        self.covariance[0] *= 1.0 - kx;
        self.covariance[1] *= 1.0 - ky;
    }

    /// Get predicted position
    fn predict_position(&self, dt: f32) -> (f32, f32) {
        (
            self.state[0] + self.state[2] * dt,
            self.state[1] + self.state[3] * dt,
        )
    }
}

#[derive(Debug)]
pub struct AlignmentTracker {
    target_position: (f32, f32),
    current_position: (f32, f32),
    tolerance_px: f32,
    last_alignment_check: Instant,
    alignment_attempts: u32,
    // Enhanced tracking for optimization
    position_history: VecDeque<((f32, f32), Instant)>,
    velocity_estimate: (f32, f32), // pixels per second
    prediction_enabled: bool,
    kalman_filter: Option<KalmanFilter>,
}

impl AlignmentTracker {
    pub fn new(tolerance_px: f32) -> Self {
        Self {
            target_position: (0.0, 0.0),
            current_position: (0.0, 0.0),
            tolerance_px,
            last_alignment_check: Instant::now(),
            alignment_attempts: 0,
            position_history: VecDeque::with_capacity(20),
            velocity_estimate: (0.0, 0.0),
            prediction_enabled: true,
            kalman_filter: Some(KalmanFilter::new()),
        }
    }

    pub fn get_alignment_status(&self, signal_strength: f32) -> AlignmentStatus {
        let distance = ((self.target_position.0 - self.current_position.0).powi(2)
                       + (self.target_position.1 - self.current_position.1).powi(2)).sqrt();

        AlignmentStatus {
            is_aligned: distance <= self.tolerance_px,
            beam_position_x: self.current_position.0,
            beam_position_y: self.current_position.1,
            signal_strength,
            last_update: self.last_alignment_check,
        }
    }

    pub fn set_target_position(&mut self, x: f32, y: f32) {
        self.target_position = (x, y);
        self.last_alignment_check = Instant::now();
    }

    pub async fn auto_align(&mut self, max_attempts: u32) -> Result<(), LaserError> {
        for attempt in 0..max_attempts {
            self.alignment_attempts = attempt + 1;

            // Measure current position (would use camera feedback)
            let current_pos = self.detect_beam_position().await?;
            let measurement_time = Instant::now();

            // Update position history
            self.position_history.push_back((current_pos, measurement_time));
            if self.position_history.len() > 20 {
                self.position_history.pop_front();
            }

            // Update Kalman filter if enabled
            if let Some(kalman) = &mut self.kalman_filter {
                // Update with measurement
                kalman.update(current_pos);

                // Use filtered position
                self.current_position = (kalman.state[0], kalman.state[1]);
            } else {
                self.current_position = current_pos;
            }

            // Estimate velocity from recent measurements (after position update)
            if self.position_history.len() >= 2 {
                self.update_velocity_estimate().await;
            }

            self.last_alignment_check = measurement_time;

            let distance = ((self.target_position.0 - self.current_position.0).powi(2)
                           + (self.target_position.1 - self.current_position.1).powi(2)).sqrt();

            if distance <= self.tolerance_px {
                return Ok(());
            }

            // Predictive adjustment using velocity and Kalman prediction
            let adjustment = if self.prediction_enabled && self.position_history.len() >= 3 {
                self.calculate_predictive_adjustment().await
            } else {
                // Simple proportional adjustment
                (self.target_position.0 - self.current_position.0,
                 self.target_position.1 - self.current_position.1)
            };

            // Adjust beam position (would control beam steering)
            self.adjust_beam_position(adjustment.0, adjustment.1).await?;

            // Small delay for stabilization
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        Err(LaserError::AlignmentLost)
    }

    /// Update velocity estimate from position history
    async fn update_velocity_estimate(&mut self) {
        if self.position_history.len() < 2 {
            return;
        }

        // Calculate velocity from recent measurements
        let _len = self.position_history.len();
        let recent_positions: Vec<&((f32, f32), Instant)> = self.position_history.iter().rev().take(3).collect();

        if recent_positions.len() >= 2 {
            let (pos1, time1) = recent_positions[0];
            let (pos2, time2) = recent_positions[1];

            let dt = time1.duration_since(*time2).as_secs_f32();
            if dt > 0.0 {
                let vx = (pos1.0 - pos2.0) / dt;
                let vy = (pos1.1 - pos2.1) / dt;

                // Smooth velocity estimate
                self.velocity_estimate.0 = 0.7 * self.velocity_estimate.0 + 0.3 * vx;
                self.velocity_estimate.1 = 0.7 * self.velocity_estimate.1 + 0.3 * vy;
            }
        }
    }

    /// Calculate predictive adjustment using velocity and Kalman prediction
    async fn calculate_predictive_adjustment(&self) -> (f32, f32) {
        let dt = 0.1; // Look ahead 100ms

        // Use Kalman prediction if available
        if let Some(kalman) = &self.kalman_filter {
            let predicted_pos = kalman.predict_position(dt);
            return (self.target_position.0 - predicted_pos.0,
                    self.target_position.1 - predicted_pos.1);
        }

        // Fallback to velocity-based prediction
        let predicted_x = self.current_position.0 + self.velocity_estimate.0 * dt;
        let predicted_y = self.current_position.1 + self.velocity_estimate.1 * dt;

        (self.target_position.0 - predicted_x,
         self.target_position.1 - predicted_y)
    }

    /// Detect beam position using camera feedback
    async fn detect_beam_position(&self) -> Result<(f32, f32), LaserError> {
        // Would analyze camera frame to detect laser spot
        // For now, return mock position
        Ok((0.0, 0.0))
    }

    /// Adjust beam position (beam steering)
    async fn adjust_beam_position(&self, _delta_x: f32, _delta_y: f32) -> Result<(), LaserError> {
        #[cfg(target_os = "android")]
        {
            let result = unsafe { super::hardware::laser_set_alignment(_delta_x, _delta_y) };
            if result != 0 {
                return Err(LaserError::AlignmentLost);
            }
        }

        #[cfg(not(target_os = "android"))] {
            // Mock implementation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }
}

pub struct AlignmentManager {
    tracker: Arc<Mutex<AlignmentTracker>>,
}

impl AlignmentManager {
    pub fn new(tolerance_px: f32) -> Self {
        Self {
            tracker: Arc::new(Mutex::new(AlignmentTracker::new(tolerance_px))),
        }
    }

    pub async fn get_alignment_status(&self, signal_strength: f32) -> AlignmentStatus {
        let tracker = self.tracker.lock().await;
        tracker.get_alignment_status(signal_strength)
    }

    pub async fn set_alignment_target(&self, x: f32, y: f32) -> Result<(), LaserError> {
        let mut tracker = self.tracker.lock().await;
        tracker.set_target_position(x, y);
        Ok(())
    }

    pub async fn perform_auto_alignment(&self, max_attempts: u32) -> Result<(), LaserError> {
        let mut tracker = self.tracker.lock().await;
        tracker.auto_align(max_attempts).await
    }

    pub async fn update_kalman_prediction(&self) {
        let mut tracker = self.tracker.lock().await;
        if let Some(kalman) = &mut tracker.kalman_filter {
            // Predict next position (50ms ahead)
            kalman.predict(0.05);
        }
    }
}