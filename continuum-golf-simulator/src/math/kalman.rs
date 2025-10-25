// Kalman filter implementation for adaptive player skill tracking
//
// Uses a 1D Kalman filter to estimate player's true skill (σ) based on
// noisy shot measurements. The filter adapts to player's changing performance
// over time while accounting for measurement uncertainty.

use serde::{Deserialize, Serialize};

/// Kalman filter state for tracking player skill
///
/// Maintains the current estimate of a player's skill parameter (σ)
/// along with the uncertainty in that estimate (error covariance).
///
/// # Fields
/// * `estimate` - Current skill estimate (σ in feet)
/// * `error_covariance` - Uncertainty in estimate (P_k)
/// * `process_noise` - Expected skill drift between updates (Q)
/// * `initial_estimate` - Starting σ_0 for reset functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalmanState {
    pub estimate: f64,
    pub error_covariance: f64,
    pub process_noise: f64,
    pub initial_estimate: f64,
    pub measurement_count: usize,
}

impl KalmanState {
    /// Create a new Kalman filter with initial parameters
    ///
    /// # Arguments
    /// * `initial_sigma` - Starting skill estimate (σ_0)
    /// * `process_noise` - Expected skill variation (Q), typically small (~1.0)
    ///
    /// # Returns
    /// New KalmanState with high initial uncertainty
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::kalman::KalmanState;
    ///
    /// // Start with σ = 30ft, low process noise
    /// let mut kalman = KalmanState::new(30.0, 1.0);
    /// assert_eq!(kalman.estimate, 30.0);
    /// ```
    pub fn new(initial_sigma: f64, process_noise: f64) -> Self {
        KalmanState {
            estimate: initial_sigma,
            error_covariance: 1000.0, // High initial uncertainty
            process_noise,
            initial_estimate: initial_sigma,
            measurement_count: 0,
        }
    }

    /// Prediction step: project estimate forward in time
    ///
    /// In our model, we assume skill doesn't change deterministically,
    /// but uncertainty increases due to process noise (Q).
    ///
    /// # Returns
    /// Tuple of (predicted_estimate, predicted_covariance)
    ///
    /// # Update Equations
    /// - σ_predicted = σ_current (no motion model)
    /// - P_predicted = P_current + Q
    pub fn predict(&mut self) -> (f64, f64) {
        // State prediction (skill doesn't change without measurement)
        let predicted_estimate = self.estimate;

        // Covariance prediction (uncertainty increases)
        let predicted_covariance = self.error_covariance + self.process_noise;

        self.error_covariance = predicted_covariance;

        (predicted_estimate, predicted_covariance)
    }

    /// Update step: incorporate new measurement
    ///
    /// Uses a new shot measurement to refine the skill estimate.
    /// The Kalman gain determines how much to trust the measurement vs. the prediction.
    ///
    /// # Arguments
    /// * `measurement` - Observed miss distance (after debiasing for Rayleigh)
    /// * `measurement_noise` - Uncertainty in this measurement (R)
    /// * `batch_size` - Number of shots represented by this measurement (default 1)
    ///
    /// # Update Equations
    /// 1. Kalman gain: K = P / (P + R)
    /// 2. Estimate update: σ_new = σ_old + K * (z - σ_old)
    /// 3. Covariance update: P_new = (1 - K) * P_old
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::kalman::KalmanState;
    ///
    /// let mut kalman = KalmanState::new(30.0, 1.0);
    /// kalman.update(28.0, 50.0, 1);  // Single measurement suggests skill is better
    /// // estimate will move toward 28.0, weighted by Kalman gain
    /// ```
    pub fn update(&mut self, measurement: f64, measurement_noise: f64, batch_size: usize) {
        // Kalman gain: how much to trust the measurement
        let kalman_gain = self.error_covariance / (self.error_covariance + measurement_noise);

        // Update estimate: blend prediction with measurement
        let innovation = measurement - self.estimate;
        self.estimate += kalman_gain * innovation;

        // Update covariance: reduce uncertainty
        self.error_covariance *= 1.0 - kalman_gain;

        // Increment measurement count by batch size
        self.measurement_count += batch_size;
    }

    /// Calculate confidence score from error covariance and measurement count
    ///
    /// Maps error covariance (P) to a confidence percentage (0-100%).
    /// Uses logarithmic scale as P ranges from 50 (high confidence) to 1000 (low confidence).
    /// Additionally penalizes low measurement counts to prevent premature high confidence.
    ///
    /// # Returns
    /// Confidence percentage (0-100)
    ///
    /// # Formula
    /// base_confidence = 100 * (1 - ln(P/50) / ln(1000/50))
    /// measurement_factor = min(1.0, measurement_count / 20.0)
    /// confidence = base_confidence * measurement_factor
    ///
    /// # Interpretation
    /// - Requires at least 20 measurements for full confidence
    /// - P ≈ 50 with 20+ measurements → 100%
    /// - P ≈ 223 with 20+ measurements → 50%
    /// - P ≥ 1000 → 0% regardless of measurements
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::kalman::KalmanState;
    ///
    /// let mut kalman = KalmanState::new(30.0, 1.0);
    /// assert_eq!(kalman.calculate_confidence(), 0.0); // P = 1000
    ///
    /// for _ in 0..20 {
    ///     kalman.update(30.0, 50.0, 1); // Many consistent measurements
    /// }
    /// assert!(kalman.calculate_confidence() > 80.0); // High confidence now
    /// ```
    pub fn calculate_confidence(&self) -> f64 {
        let p = self.error_covariance;
        let min_p = 50.0;
        let max_p = 1000.0;

        // Base confidence from error covariance
        let base_confidence = if p <= min_p {
            100.0
        } else if p >= max_p {
            0.0
        } else {
            // Logarithmic mapping
            let normalized = (p / min_p).ln() / (max_p / min_p).ln();
            100.0 * (1.0 - normalized)
        };

        // Penalize low measurement counts (need ~20 shots for full confidence)
        let measurement_factor = (self.measurement_count as f64 / 20.0).min(1.0);

        base_confidence * measurement_factor
    }

    /// Reset filter to initial state
    ///
    /// Useful when player changes significantly or for debugging.
    pub fn reset(&mut self) {
        self.estimate = self.initial_estimate;
        self.error_covariance = 1000.0;
        self.measurement_count = 0;
    }

    /// Get the current standard error of the estimate
    ///
    /// Returns the square root of the error covariance, representing
    /// the standard deviation of the estimate uncertainty.
    pub fn standard_error(&self) -> f64 {
        self.error_covariance.sqrt()
    }
}

/// Helper function to debias Rayleigh measurements
///
/// Rayleigh-distributed miss distances have mean σ * sqrt(π/2),
/// but we want to estimate σ itself. This function converts
/// a measurement to an unbiased estimate of σ.
///
/// # Arguments
/// * `measured_miss` - Observed miss distance (feet)
///
/// # Returns
/// Unbiased estimate of σ
///
/// # Formula
/// σ_unbiased = measured_miss / sqrt(π/2)
pub fn debias_rayleigh_measurement(measured_miss: f64) -> f64 {
    use std::f64::consts::PI;
    measured_miss / (PI / 2.0).sqrt()
}

/// Calculate weighted average of shot measurements
///
/// When updating with a batch of shots, we weight each measurement
/// by its wager to give higher importance to high-stakes shots.
///
/// # Arguments
/// * `measurements` - Vec of (miss_distance, wager) tuples
///
/// # Returns
/// Weighted average miss distance
///
/// # Formula
/// z_weighted = Σ(miss_i * wager_i) / Σ(wager_i)
pub fn weighted_average_measurement(measurements: &[(f64, f64)]) -> f64 {
    let total_weight: f64 = measurements.iter().map(|(_, w)| w).sum();

    if total_weight == 0.0 {
        return 0.0;
    }

    let weighted_sum: f64 = measurements.iter().map(|(m, w)| m * w).sum();

    weighted_sum / total_weight
}

/// Calculate variance of a batch of measurements
///
/// Used to determine dynamic measurement noise (R) for batch updates.
/// Higher variance means less trustworthy batch.
///
/// # Arguments
/// * `measurements` - Vec of miss distances
///
/// # Returns
/// Sample variance
pub fn measurement_variance(measurements: &[f64]) -> f64 {
    if measurements.len() <= 1 {
        return 100.0; // Default variance for single measurement
    }

    let mean: f64 = measurements.iter().sum::<f64>() / measurements.len() as f64;
    let variance: f64 = measurements
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>()
        / (measurements.len() - 1) as f64;

    variance
}

// ============================================================================
// 4D KALMAN FILTER FOR BVN (BIVARIATE NORMAL)
// ============================================================================

/// 4D Kalman filter state for tracking BVN parameters
///
/// Maintains estimates of:
/// - μ_x: Lateral bias (feet right of target line, positive = right)
/// - μ_y: Distance bias (feet from pin, positive = long)
/// - σ_x: Lateral dispersion (standard deviation)
/// - σ_y: Distance dispersion (standard deviation)
///
/// # State Vector
/// ```text
/// x = [μ_x, μ_y, σ_x, σ_y]^T
/// ```
///
/// # Covariance Matrix (4x4)
/// ```text
/// P = [P_μx_μx   P_μx_μy   P_μx_σx   P_μx_σy  ]
///     [P_μy_μx   P_μy_μy   P_μy_σx   P_μy_σy  ]
///     [P_σx_μx   P_σx_μy   P_σx_σx   P_σx_σy  ]
///     [P_σy_μx   P_σy_μy   P_σy_σx   P_σy_σy  ]
/// ```
///
/// For simplicity, we assume independence: off-diagonal terms ≈ 0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalmanState4D {
    /// State vector: [μ_x, μ_y, σ_x, σ_y]
    pub state: [f64; 4],

    /// Error covariance diagonal (simplified from 4x4 matrix)
    /// [P_μx, P_μy, P_σx, P_σy]
    pub error_covariance: [f64; 4],

    /// Process noise (how much we expect each parameter to drift)
    pub process_noise: [f64; 4],

    /// Initial state for reset
    pub initial_state: [f64; 4],
}

impl KalmanState4D {
    /// Create a new 4D Kalman filter
    ///
    /// # Arguments
    /// * `initial_mu_x` - Initial lateral bias estimate (0.0 = no bias)
    /// * `initial_mu_y` - Initial distance bias estimate (0.0 = no bias)
    /// * `initial_sigma_x` - Initial lateral dispersion estimate
    /// * `initial_sigma_y` - Initial distance dispersion estimate
    /// * `process_noise` - Expected drift in [μ_x, μ_y, σ_x, σ_y]
    ///
    /// # Returns
    /// New KalmanState4D with high initial uncertainty
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::kalman::KalmanState4D;
    ///
    /// // Start with no bias, σ_x=σ_y=30ft (like Rayleigh)
    /// let kalman = KalmanState4D::new(0.0, 0.0, 30.0, 30.0, [0.1, 0.1, 0.5, 0.5]);
    /// assert_eq!(kalman.mu_x(), 0.0);
    /// assert_eq!(kalman.sigma_x(), 30.0);
    /// ```
    pub fn new(
        initial_mu_x: f64,
        initial_mu_y: f64,
        initial_sigma_x: f64,
        initial_sigma_y: f64,
        process_noise: [f64; 4],
    ) -> Self {
        let state = [initial_mu_x, initial_mu_y, initial_sigma_x, initial_sigma_y];

        KalmanState4D {
            state,
            error_covariance: [1000.0, 1000.0, 1000.0, 1000.0], // High initial uncertainty
            process_noise,
            initial_state: state,
        }
    }

    /// Get current μ_x (lateral bias)
    pub fn mu_x(&self) -> f64 {
        self.state[0]
    }

    /// Get current μ_y (distance bias)
    pub fn mu_y(&self) -> f64 {
        self.state[1]
    }

    /// Get current σ_x (lateral dispersion)
    pub fn sigma_x(&self) -> f64 {
        self.state[2]
    }

    /// Get current σ_y (distance dispersion)
    pub fn sigma_y(&self) -> f64 {
        self.state[3]
    }

    /// Prediction step: project state forward
    ///
    /// State doesn't change (no motion model), but uncertainty increases.
    ///
    /// # Update Equations
    /// - x_predicted = x_current
    /// - P_predicted = P_current + Q
    pub fn predict(&mut self) {
        // State stays the same (no deterministic change in skill)
        // Only covariance increases due to process noise
        for i in 0..4 {
            self.error_covariance[i] += self.process_noise[i];
        }
    }

    /// Update step: incorporate (x, y) measurement with σ estimates
    ///
    /// Uses batch statistics to update all four parameters.
    ///
    /// # Arguments
    /// * `measured_x` - Batch mean lateral position (feet right of target line)
    /// * `measured_y` - Batch mean distance position (feet from pin)
    /// * `measured_sigma_x` - Sample std dev of lateral positions from batch
    /// * `measured_sigma_y` - Sample std dev of distance positions from batch
    /// * `measurement_noise` - Measurement uncertainty [R_x, R_y, R_σx, R_σy]
    ///
    /// # Update Strategy
    /// 1. Update μ_x using batch mean x
    /// 2. Update μ_y using batch mean y
    /// 3. Update σ_x using batch sample std dev
    /// 4. Update σ_y using batch sample std dev
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::kalman::KalmanState4D;
    ///
    /// let mut kalman = KalmanState4D::new(0.0, 0.0, 30.0, 30.0, [0.1, 0.1, 0.5, 0.5]);
    ///
    /// // Batch: mean=(5.0, 2.0), std_dev=(20.0, 25.0)
    /// kalman.update_with_batch(5.0, 2.0, 20.0, 25.0, [50.0, 50.0, 100.0, 100.0]);
    ///
    /// // μ should move toward batch mean, σ toward batch std dev
    /// assert!(kalman.mu_x() > 0.0);
    /// assert!(kalman.sigma_x() < 30.0 && kalman.sigma_x() > 15.0);
    /// ```
    pub fn update_with_batch(
        &mut self,
        measured_x: f64,
        measured_y: f64,
        measured_sigma_x: f64,
        measured_sigma_y: f64,
        measurement_noise: [f64; 4],
    ) {
        // Update μ_x using measured batch mean
        let k_mu_x = self.error_covariance[0] / (self.error_covariance[0] + measurement_noise[0]);
        let innovation_mu_x = measured_x - self.state[0];
        self.state[0] += k_mu_x * innovation_mu_x;
        self.error_covariance[0] *= 1.0 - k_mu_x;

        // Update μ_y using measured batch mean
        let k_mu_y = self.error_covariance[1] / (self.error_covariance[1] + measurement_noise[1]);
        let innovation_mu_y = measured_y - self.state[1];
        self.state[1] += k_mu_y * innovation_mu_y;
        self.error_covariance[1] *= 1.0 - k_mu_y;

        // Update σ_x using measured batch std dev
        let k_sigma_x = self.error_covariance[2] / (self.error_covariance[2] + measurement_noise[2]);
        let innovation_sigma_x = measured_sigma_x - self.state[2];
        self.state[2] += k_sigma_x * innovation_sigma_x;
        self.error_covariance[2] *= 1.0 - k_sigma_x;

        // Clamp σ_x to reasonable bounds (minimum 10ft to prevent underestimation)
        self.state[2] = self.state[2].max(10.0).min(200.0);

        // Update σ_y using measured batch std dev
        let k_sigma_y = self.error_covariance[3] / (self.error_covariance[3] + measurement_noise[3]);
        let innovation_sigma_y = measured_sigma_y - self.state[3];
        self.state[3] += k_sigma_y * innovation_sigma_y;
        self.error_covariance[3] *= 1.0 - k_sigma_y;

        // Clamp σ_y to reasonable bounds (minimum 10ft to prevent underestimation)
        self.state[3] = self.state[3].max(10.0).min(200.0);
    }

    /// Legacy update method (deprecated - kept for compatibility)
    ///
    /// **WARNING**: This method uses single-shot residuals to estimate σ,
    /// which severely underestimates true dispersion. Use `update_with_batch()` instead.
    #[deprecated(note = "Use update_with_batch() which properly estimates σ from batch statistics")]
    pub fn update(&mut self, measured_x: f64, measured_y: f64, measurement_noise: [f64; 4]) {
        // Only update bias (μ), not dispersion (σ)
        // Dispersion requires multiple shots to estimate

        // Update μ_x
        let k_mu_x = self.error_covariance[0] / (self.error_covariance[0] + measurement_noise[0]);
        self.state[0] += k_mu_x * (measured_x - self.state[0]);
        self.error_covariance[0] *= 1.0 - k_mu_x;

        // Update μ_y
        let k_mu_y = self.error_covariance[1] / (self.error_covariance[1] + measurement_noise[1]);
        self.state[1] += k_mu_y * (measured_y - self.state[1]);
        self.error_covariance[1] *= 1.0 - k_mu_y;

        // Don't update σ from single shot - too noisy!
    }

    /// Calculate overall confidence from error covariance
    ///
    /// Returns the minimum confidence across all four parameters.
    /// This gives a conservative estimate of overall certainty.
    ///
    /// # Returns
    /// Confidence percentage (0-100)
    pub fn calculate_confidence(&self) -> f64 {
        let min_p = 50.0;
        let max_p = 1000.0;

        let confidences: Vec<f64> = self.error_covariance.iter().map(|&p| {
            if p <= min_p {
                100.0
            } else if p >= max_p {
                0.0
            } else {
                let normalized = (p / min_p).ln() / (max_p / min_p).ln();
                100.0 * (1.0 - normalized)
            }
        }).collect();

        // Return minimum confidence (most conservative)
        confidences.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// Get bias magnitude (Euclidean distance from origin)
    ///
    /// # Returns
    /// Bias magnitude in feet: sqrt(μ_x² + μ_y²)
    pub fn bias_magnitude(&self) -> f64 {
        (self.state[0].powi(2) + self.state[1].powi(2)).sqrt()
    }

    /// Get bias direction in degrees (0° = right, 90° = long, 180° = left, 270° = short)
    ///
    /// # Returns
    /// Angle in degrees (0-360)
    pub fn bias_direction_degrees(&self) -> f64 {
        let angle_rad = self.state[1].atan2(self.state[0]);
        let angle_deg = angle_rad.to_degrees();

        // Convert to 0-360 range
        if angle_deg < 0.0 {
            angle_deg + 360.0
        } else {
            angle_deg
        }
    }

    /// Get precision ratio (σ_x / σ_y)
    ///
    /// Indicates whether player is more precise laterally or in distance.
    ///
    /// # Returns
    /// Ratio > 1.0: worse lateral control than distance
    /// Ratio < 1.0: better lateral control than distance
    /// Ratio ≈ 1.0: equal precision (like Rayleigh)
    pub fn precision_ratio(&self) -> f64 {
        self.state[2] / self.state[3]
    }

    /// Reset filter to initial state
    pub fn reset(&mut self) {
        self.state = self.initial_state;
        self.error_covariance = [1000.0, 1000.0, 1000.0, 1000.0];
    }

    /// Get standard errors of all estimates
    ///
    /// # Returns
    /// [SE_μx, SE_μy, SE_σx, SE_σy]
    pub fn standard_errors(&self) -> [f64; 4] {
        [
            self.error_covariance[0].sqrt(),
            self.error_covariance[1].sqrt(),
            self.error_covariance[2].sqrt(),
            self.error_covariance[3].sqrt(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_kalman_initialization() {
        let kalman = KalmanState::new(30.0, 1.0);

        assert_eq!(kalman.estimate, 30.0);
        assert_eq!(kalman.error_covariance, 1000.0);
        assert_eq!(kalman.process_noise, 1.0);
        assert_eq!(kalman.calculate_confidence(), 0.0);
    }

    #[test]
    fn test_kalman_convergence() {
        let mut kalman = KalmanState::new(30.0, 0.5);
        let true_sigma = 25.0;

        // Simulate 100 consistent measurements
        for _ in 0..100 {
            kalman.predict();
            kalman.update(true_sigma, 50.0, 1);
        }

        // Should converge close to true value
        assert_relative_eq!(kalman.estimate, true_sigma, epsilon = 2.0);

        // Confidence should be high
        assert!(kalman.calculate_confidence() > 70.0);
    }

    #[test]
    fn test_confidence_calculation() {
        let mut kalman = KalmanState::new(30.0, 0.1);

        // Initial: P = 1000 → confidence = 0%
        assert_eq!(kalman.calculate_confidence(), 0.0);

        // After many updates: P decreases → confidence increases
        for _ in 0..100 {
            kalman.update(30.0, 50.0, 1);
        }

        let confidence = kalman.calculate_confidence();
        assert!(confidence > 80.0, "Confidence was: {}", confidence);
    }

    #[test]
    fn test_debias_rayleigh() {
        use std::f64::consts::PI;
        let sigma = 30.0;
        let measured = sigma * (PI / 2.0).sqrt(); // E[Rayleigh(30)] ≈ 37.62
        let unbiased = debias_rayleigh_measurement(measured);

        assert_relative_eq!(unbiased, sigma, epsilon = 0.01);
    }

    #[test]
    fn test_weighted_average() {
        let measurements = vec![
            (10.0, 5.0),  // miss=10, wager=5
            (20.0, 10.0), // miss=20, wager=10
            (30.0, 5.0),  // miss=30, wager=5
        ];

        let avg = weighted_average_measurement(&measurements);

        // Expected: (10*5 + 20*10 + 30*5) / (5+10+5) = 400/20 = 20
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_measurement_variance() {
        let measurements = vec![10.0, 12.0, 14.0, 16.0];
        let variance = measurement_variance(&measurements);

        // Sample variance of [10, 12, 14, 16] = 6.67
        assert_relative_eq!(variance, 6.666, epsilon = 0.01);
    }

    #[test]
    fn test_reset() {
        let mut kalman = KalmanState::new(30.0, 1.0);

        // Make some updates
        for _ in 0..10 {
            kalman.update(25.0, 50.0, 1);
        }

        let modified_estimate = kalman.estimate;
        assert_ne!(modified_estimate, 30.0);

        // Reset should restore initial state
        kalman.reset();
        assert_eq!(kalman.estimate, 30.0);
        assert_eq!(kalman.error_covariance, 1000.0);
    }

    // ========================================================================
    // 4D KALMAN FILTER TESTS
    // ========================================================================

    #[test]
    fn test_kalman_4d_initialization() {
        let kalman = KalmanState4D::new(0.0, 0.0, 30.0, 25.0, [0.1, 0.1, 0.5, 0.5]);

        assert_eq!(kalman.mu_x(), 0.0);
        assert_eq!(kalman.mu_y(), 0.0);
        assert_eq!(kalman.sigma_x(), 30.0);
        assert_eq!(kalman.sigma_y(), 25.0);
        assert_eq!(kalman.calculate_confidence(), 0.0); // High initial uncertainty
    }

    #[test]
    fn test_kalman_4d_bias_convergence() {
        let mut kalman = KalmanState4D::new(0.0, 0.0, 30.0, 30.0, [0.1, 0.1, 0.3, 0.3]);

        // Simulate player who consistently misses 5ft right, 2ft long
        let true_mu_x = 5.0;
        let true_mu_y = 2.0;
        let true_sigma_x = 10.0;
        let true_sigma_y = 10.0;

        // Process in batches of 10 shots
        for _ in 0..10 {
            kalman.predict();

            // Generate batch of 10 shots
            let mut x_coords = Vec::new();
            let mut y_coords = Vec::new();
            for _ in 0..10 {
                let x = true_mu_x + (rand::random::<f64>() - 0.5) * true_sigma_x * 2.0;
                let y = true_mu_y + (rand::random::<f64>() - 0.5) * true_sigma_y * 2.0;
                x_coords.push(x);
                y_coords.push(y);
            }

            // Calculate batch mean
            let mean_x = x_coords.iter().sum::<f64>() / 10.0;
            let mean_y = y_coords.iter().sum::<f64>() / 10.0;

            // Calculate batch std dev
            let n = 10.0;
            let var_x: f64 = x_coords.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>() / (n - 1.0);
            let var_y: f64 = y_coords.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>() / (n - 1.0);
            let sigma_x_measured = var_x.sqrt().max(10.0);
            let sigma_y_measured = var_y.sqrt().max(10.0);

            // Measurement noise (standard error)
            let noise_x = (sigma_x_measured / n.sqrt()).max(1.0);
            let noise_y = (sigma_y_measured / n.sqrt()).max(1.0);
            let noise_sigma_x = (sigma_x_measured / (2.0 * n).sqrt()).max(1.0);
            let noise_sigma_y = (sigma_y_measured / (2.0 * n).sqrt()).max(1.0);

            kalman.update_with_batch(
                mean_x,
                mean_y,
                sigma_x_measured,
                sigma_y_measured,
                [noise_x, noise_y, noise_sigma_x, noise_sigma_y],
            );
        }

        // Should converge to true bias
        assert_relative_eq!(kalman.mu_x(), true_mu_x, epsilon = 2.0);
        assert_relative_eq!(kalman.mu_y(), true_mu_y, epsilon = 2.0);

        // Confidence should increase
        assert!(kalman.calculate_confidence() > 50.0);
    }

    #[test]
    fn test_kalman_4d_dispersion_convergence() {
        let mut kalman = KalmanState4D::new(0.0, 0.0, 30.0, 30.0, [0.1, 0.1, 0.3, 0.3]);

        // Simulate shots with known dispersion (no bias)
        // σ_x = 20, σ_y = 15
        use crate::math::distributions::bvn_random;

        let true_sigma_x = 20.0;
        let true_sigma_y = 15.0;

        // Process in batches of 10 shots
        for _ in 0..20 {
            kalman.predict();

            // Generate batch of 10 shots
            let mut x_coords = Vec::new();
            let mut y_coords = Vec::new();
            for _ in 0..10 {
                let (x, y) = bvn_random(0.0, 0.0, true_sigma_x, true_sigma_y);
                x_coords.push(x);
                y_coords.push(y);
            }

            // Calculate batch mean
            let mean_x = x_coords.iter().sum::<f64>() / 10.0;
            let mean_y = y_coords.iter().sum::<f64>() / 10.0;

            // Calculate batch std dev
            let n = 10.0;
            let var_x: f64 = x_coords.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>() / (n - 1.0);
            let var_y: f64 = y_coords.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>() / (n - 1.0);
            let sigma_x_measured = var_x.sqrt().max(10.0);
            let sigma_y_measured = var_y.sqrt().max(10.0);

            // Measurement noise (standard error)
            let noise_x = (sigma_x_measured / n.sqrt()).max(1.0);
            let noise_y = (sigma_y_measured / n.sqrt()).max(1.0);
            let noise_sigma_x = (sigma_x_measured / (2.0 * n).sqrt()).max(1.0);
            let noise_sigma_y = (sigma_y_measured / (2.0 * n).sqrt()).max(1.0);

            kalman.update_with_batch(
                mean_x,
                mean_y,
                sigma_x_measured,
                sigma_y_measured,
                [noise_x, noise_y, noise_sigma_x, noise_sigma_y],
            );
        }

        // Should converge toward true dispersions
        // With batch statistics, should be closer to true values
        assert!(kalman.sigma_x() > 12.0 && kalman.sigma_x() < 30.0);
        assert!(kalman.sigma_y() > 10.0 && kalman.sigma_y() < 25.0);

        // Bias should stay near zero (no systematic bias in data)
        assert!(kalman.mu_x().abs() < 5.0);
        assert!(kalman.mu_y().abs() < 5.0);
    }

    #[test]
    fn test_kalman_4d_bias_metrics() {
        let kalman = KalmanState4D::new(3.0, 4.0, 30.0, 30.0, [0.1, 0.1, 0.5, 0.5]);

        // Bias magnitude: sqrt(3² + 4²) = 5
        assert_relative_eq!(kalman.bias_magnitude(), 5.0, epsilon = 0.01);

        // Bias direction: atan2(4, 3) ≈ 53.13°
        assert_relative_eq!(kalman.bias_direction_degrees(), 53.13, epsilon = 0.1);
    }

    #[test]
    fn test_kalman_4d_precision_ratio() {
        let kalman_equal = KalmanState4D::new(0.0, 0.0, 20.0, 20.0, [0.1, 0.1, 0.5, 0.5]);
        assert_relative_eq!(kalman_equal.precision_ratio(), 1.0, epsilon = 0.01);

        let kalman_worse_lateral = KalmanState4D::new(0.0, 0.0, 30.0, 10.0, [0.1, 0.1, 0.5, 0.5]);
        assert_relative_eq!(kalman_worse_lateral.precision_ratio(), 3.0, epsilon = 0.01);
    }

    #[test]
    fn test_kalman_4d_reset() {
        let mut kalman = KalmanState4D::new(0.0, 0.0, 30.0, 25.0, [0.1, 0.1, 0.5, 0.5]);

        // Make some updates with batch statistics
        for _ in 0..5 {
            kalman.predict();
            // Simulate batch measurements with consistent bias
            kalman.update_with_batch(5.0, 2.0, 15.0, 12.0, [2.0, 2.0, 3.0, 3.0]);
        }

        assert_ne!(kalman.mu_x(), 0.0);
        assert_ne!(kalman.mu_y(), 0.0);

        // Reset should restore initial state
        kalman.reset();
        assert_eq!(kalman.mu_x(), 0.0);
        assert_eq!(kalman.mu_y(), 0.0);
        assert_eq!(kalman.sigma_x(), 30.0);
        assert_eq!(kalman.sigma_y(), 25.0);
        assert_eq!(kalman.calculate_confidence(), 0.0);
    }

    #[test]
    fn test_kalman_4d_clamping() {
        let mut kalman = KalmanState4D::new(0.0, 0.0, 30.0, 30.0, [0.1, 0.1, 0.5, 0.5]);

        kalman.predict();

        // Try to update with extremely tight dispersion that would push σ too low
        // Simulate batch with very tight clustering (σ ≈ 1.0)
        kalman.update_with_batch(0.0, 0.0, 1.0, 1.0, [0.5, 0.5, 0.5, 0.5]);

        // σ values should be clamped to minimum 10.0
        assert!(kalman.sigma_x() >= 10.0);
        assert!(kalman.sigma_y() >= 10.0);
    }
}
