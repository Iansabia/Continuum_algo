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
    /// kalman.update(28.0, 50.0);  // Measurement suggests skill is better
    /// // estimate will move toward 28.0, weighted by Kalman gain
    /// ```
    pub fn update(&mut self, measurement: f64, measurement_noise: f64) {
        // Kalman gain: how much to trust the measurement
        let kalman_gain = self.error_covariance / (self.error_covariance + measurement_noise);

        // Update estimate: blend prediction with measurement
        let innovation = measurement - self.estimate;
        self.estimate += kalman_gain * innovation;

        // Update covariance: reduce uncertainty
        self.error_covariance *= 1.0 - kalman_gain;
    }

    /// Calculate confidence score from error covariance
    ///
    /// Maps error covariance (P) to a confidence percentage (0-100%).
    /// Uses logarithmic scale as P ranges from 50 (high confidence) to 1000 (low confidence).
    ///
    /// # Returns
    /// Confidence percentage (0-100)
    ///
    /// # Formula
    /// confidence = 100 * (1 - ln(P/50) / ln(1000/50))
    ///
    /// # Interpretation
    /// - 100%: Very confident (P ≈ 50)
    /// - 50%: Moderate confidence (P ≈ 223)
    /// - 0%: No confidence (P ≥ 1000)
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::kalman::KalmanState;
    ///
    /// let mut kalman = KalmanState::new(30.0, 1.0);
    /// assert_eq!(kalman.calculate_confidence(), 0.0); // P = 1000
    ///
    /// for _ in 0..50 {
    ///     kalman.update(30.0, 50.0); // Many consistent measurements
    /// }
    /// assert!(kalman.calculate_confidence() > 80.0); // High confidence now
    /// ```
    pub fn calculate_confidence(&self) -> f64 {
        let p = self.error_covariance;
        let min_p = 50.0;
        let max_p = 1000.0;

        if p <= min_p {
            return 100.0;
        }
        if p >= max_p {
            return 0.0;
        }

        // Logarithmic mapping
        let normalized = (p / min_p).ln() / (max_p / min_p).ln();
        100.0 * (1.0 - normalized)
    }

    /// Reset filter to initial state
    ///
    /// Useful when player changes significantly or for debugging.
    pub fn reset(&mut self) {
        self.estimate = self.initial_estimate;
        self.error_covariance = 1000.0;
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
            kalman.update(true_sigma, 50.0);
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
            kalman.update(30.0, 50.0);
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
            kalman.update(25.0, 50.0);
        }

        let modified_estimate = kalman.estimate;
        assert_ne!(modified_estimate, 30.0);

        // Reset should restore initial state
        kalman.reset();
        assert_eq!(kalman.estimate, 30.0);
        assert_eq!(kalman.error_covariance, 1000.0);
    }
}
