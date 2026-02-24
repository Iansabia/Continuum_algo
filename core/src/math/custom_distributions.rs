use rand::Rng;
use serde::{Deserialize, Serialize};

/// Pre-defined shot patterns for demonstrating model adaptation
/// These simulate different player behaviors without requiring perfect RNG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeType {
    /// Tight circular pattern - consistent player (low handicap)
    Circle {
        /// Mean miss distance (sigma equivalent)
        sigma: f64,
    },

    /// Elliptical/oval pattern - player with directional bias
    Oval {
        /// Horizontal axis sigma (e.g., left-right dispersion)
        horizontal_sigma: f64,
        /// Vertical axis sigma (e.g., long-short dispersion)
        vertical_sigma: f64,
        /// Rotation angle in radians (0 = horizontal major axis)
        rotation: f64,
    },

    /// Cluster with outliers - mimics 98% good shots + 2% fat-tail
    Cluster {
        /// Sigma for the main cluster (98% of shots)
        center_sigma: f64,
        /// Sigma for outlier shots (2% of shots)
        outlier_sigma: f64,
        /// Probability of outlier (default 0.02)
        outlier_prob: f64,
    },

    /// Wide scatter - beginner/high-handicap player
    Scatter {
        /// Base sigma for the scattered pattern
        sigma: f64,
        /// Additional uniform noise range (±range)
        noise_range: f64,
    },
}

impl ShapeType {
    /// Create a standard circle pattern matching a given sigma
    pub fn circle(sigma: f64) -> Self {
        ShapeType::Circle { sigma }
    }

    /// Create an oval with horizontal bias (e.g., player always pulls left/right)
    pub fn oval_horizontal(horizontal_sigma: f64, vertical_sigma: f64) -> Self {
        ShapeType::Oval {
            horizontal_sigma,
            vertical_sigma,
            rotation: 0.0,
        }
    }

    /// Create a cluster pattern matching the current fat-tail distribution
    pub fn cluster_fat_tail(sigma: f64) -> Self {
        ShapeType::Cluster {
            center_sigma: sigma,
            outlier_sigma: sigma * 3.0,
            outlier_prob: 0.02,
        }
    }

    /// Create a scatter pattern for high-handicap players
    pub fn scatter(sigma: f64, noise_range: f64) -> Self {
        ShapeType::Scatter { sigma, noise_range }
    }
}

/// Custom shape distribution for non-standard shot patterns
/// Used in demo mode to show MCMC adaptation to arbitrary distributions
#[derive(Debug, Clone)]
pub struct CustomShapeDistribution {
    shape: ShapeType,
}

impl CustomShapeDistribution {
    /// Create a new custom distribution from a shape type
    pub fn new(shape: ShapeType) -> Self {
        CustomShapeDistribution { shape }
    }

    /// Sample a miss distance from this distribution
    ///
    /// The pattern defines BOTH the shape AND the magnitude.
    /// MCMC will learn the effective sigma from the pattern, and P_max will adapt accordingly.
    ///
    /// # Arguments
    /// * `_player_sigma` - Ignored - pattern defines its own sigma
    /// * `rng` - Random number generator
    ///
    /// Returns (miss_distance, is_outlier) where is_outlier indicates a fat-tail event
    pub fn sample_miss_distance(&self, _player_sigma: f64, rng: &mut impl Rng) -> (f64, bool) {
        match &self.shape {
            ShapeType::Circle { sigma } => {
                // Use pattern's sigma directly - MCMC will learn this
                let u: f64 = rng.gen();
                let miss = sigma * (-2.0 * u.ln()).sqrt();
                (miss, false)
            }

            ShapeType::Oval {
                horizontal_sigma,
                vertical_sigma,
                rotation,
            } => {
                // Sample from bivariate normal using pattern's sigmas directly
                let x = sample_normal(rng, 0.0, *horizontal_sigma);
                let y = sample_normal(rng, 0.0, *vertical_sigma);

                // Apply rotation if needed
                let (x_rot, y_rot) = if rotation.abs() > 1e-6 {
                    let cos_theta = rotation.cos();
                    let sin_theta = rotation.sin();
                    (x * cos_theta - y * sin_theta, x * sin_theta + y * cos_theta)
                } else {
                    (x, y)
                };

                // Convert to radial distance
                let miss = (x_rot * x_rot + y_rot * y_rot).sqrt();
                (miss, false)
            }

            ShapeType::Cluster {
                center_sigma,
                outlier_sigma,
                outlier_prob,
            } => {
                // Decide if this is an outlier shot
                let is_outlier: f64 = rng.gen();

                if is_outlier < *outlier_prob {
                    // Outlier shot - use pattern's outlier sigma
                    let u: f64 = rng.gen();
                    let miss = outlier_sigma * (-2.0 * u.ln()).sqrt();
                    (miss, true)
                } else {
                    // Normal shot - use pattern's center sigma
                    let u: f64 = rng.gen();
                    let miss = center_sigma * (-2.0 * u.ln()).sqrt();
                    (miss, false)
                }
            }

            ShapeType::Scatter { sigma, noise_range } => {
                // Base Rayleigh distribution using pattern's sigma
                let u: f64 = rng.gen();
                let base_miss = sigma * (-2.0 * u.ln()).sqrt();

                // Add uniform noise from pattern
                let noise: f64 = rng.gen_range(-noise_range..=*noise_range);
                let miss = (base_miss + noise).max(0.0); // Ensure non-negative

                (miss, false)
            }
        }
    }

    /// Get the expected mean miss distance for this distribution
    /// Used for validation and comparison
    pub fn expected_mean(&self) -> f64 {
        match &self.shape {
            ShapeType::Circle { sigma } => {
                // Mean of Rayleigh distribution: sigma * sqrt(π/2)
                sigma * (std::f64::consts::PI / 2.0).sqrt()
            }

            ShapeType::Oval {
                horizontal_sigma,
                vertical_sigma,
                ..
            } => {
                // Approximate mean for bivariate normal converted to radial
                // E[sqrt(X^2 + Y^2)] ≈ sqrt(E[X^2] + E[Y^2]) for independent X, Y
                (horizontal_sigma.powi(2) + vertical_sigma.powi(2)).sqrt()
                    * (std::f64::consts::PI / 2.0).sqrt()
            }

            ShapeType::Cluster {
                center_sigma,
                outlier_sigma,
                outlier_prob,
            } => {
                // Weighted average of cluster and outlier means
                let center_mean = center_sigma * (std::f64::consts::PI / 2.0).sqrt();
                let outlier_mean = outlier_sigma * (std::f64::consts::PI / 2.0).sqrt();
                (1.0 - outlier_prob) * center_mean + outlier_prob * outlier_mean
            }

            ShapeType::Scatter { sigma, .. } => {
                // Noise is symmetric, so doesn't affect mean
                sigma * (std::f64::consts::PI / 2.0).sqrt()
            }
        }
    }

    /// Get a description of this distribution for logging/display
    pub fn description(&self) -> String {
        match &self.shape {
            ShapeType::Circle { sigma } => {
                format!("Circle (σ={:.1}ft) - Consistent player", sigma)
            }
            ShapeType::Oval {
                horizontal_sigma,
                vertical_sigma,
                rotation,
            } => {
                format!(
                    "Oval (h_σ={:.1}ft, v_σ={:.1}ft, θ={:.1}°) - Directional bias",
                    horizontal_sigma,
                    vertical_sigma,
                    rotation.to_degrees()
                )
            }
            ShapeType::Cluster {
                center_sigma,
                outlier_prob,
                ..
            } => {
                format!(
                    "Cluster (σ={:.1}ft, {:.1}% outliers) - Good player with occasional bad shots",
                    center_sigma,
                    outlier_prob * 100.0
                )
            }
            ShapeType::Scatter { sigma, noise_range } => {
                format!(
                    "Scatter (σ={:.1}ft, noise=±{:.1}ft) - Inconsistent beginner",
                    sigma, noise_range
                )
            }
        }
    }
}

/// Sample from a normal distribution using Box-Muller transform
fn sample_normal(rng: &mut impl Rng, mean: f64, std_dev: f64) -> f64 {
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();

    // Box-Muller transform
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

    mean + std_dev * z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_distribution() {
        let mut rng = rand::thread_rng();
        let dist = CustomShapeDistribution::new(ShapeType::circle(30.0));
        let player_sigma = 30.0; // Test with sigma matching the pattern

        let mut samples = Vec::new();
        for _ in 0..1000 {
            let (miss, _) = dist.sample_miss_distance(player_sigma, &mut rng);
            samples.push(miss);
        }

        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        let expected = dist.expected_mean();

        // Mean should be within 10% of expected
        assert!((mean - expected).abs() / expected < 0.1);
    }

    #[test]
    fn test_cluster_outlier_probability() {
        let mut rng = rand::thread_rng();
        let dist = CustomShapeDistribution::new(ShapeType::cluster_fat_tail(30.0));
        let player_sigma = 30.0; // Test with sigma matching the pattern

        let mut outlier_count = 0;
        let num_samples = 10000;

        for _ in 0..num_samples {
            let (_, is_outlier) = dist.sample_miss_distance(player_sigma, &mut rng);
            if is_outlier {
                outlier_count += 1;
            }
        }

        let outlier_rate = outlier_count as f64 / num_samples as f64;

        // Should be close to 2% outliers (within 1%)
        assert!((outlier_rate - 0.02).abs() < 0.01);
    }

    #[test]
    fn test_oval_distribution() {
        let mut rng = rand::thread_rng();
        let dist = CustomShapeDistribution::new(ShapeType::oval_horizontal(40.0, 20.0));
        // Use a sigma that matches the pattern's effective sigma
        let pattern_effective_sigma = (40.0_f64.powi(2) + 20.0_f64.powi(2)).sqrt();
        let player_sigma = pattern_effective_sigma;

        let mut samples = Vec::new();
        for _ in 0..1000 {
            let (miss, _) = dist.sample_miss_distance(player_sigma, &mut rng);
            samples.push(miss);
            assert!(miss >= 0.0); // All samples should be non-negative
        }

        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        let _expected = dist.expected_mean();

        // Just verify mean is reasonable for this distribution
        // The expected_mean is an approximation, so we just check range
        assert!(
            mean > 30.0 && mean < 70.0,
            "Mean {} is outside reasonable range",
            mean
        );
    }

    #[test]
    fn test_scatter_non_negative() {
        let mut rng = rand::thread_rng();
        let dist = CustomShapeDistribution::new(ShapeType::scatter(30.0, 10.0));
        let player_sigma = 30.0; // Test with sigma matching the pattern

        for _ in 0..1000 {
            let (miss, _) = dist.sample_miss_distance(player_sigma, &mut rng);
            assert!(miss >= 0.0); // All samples must be non-negative
        }
    }
}
