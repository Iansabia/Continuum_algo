// Statistical distributions for shot simulation
//
// Implements:
// - Normal distribution (Box-Muller transform)
// - Bivariate Normal distribution (BVN) for 2D shot modeling with correlation
// - Rayleigh distribution (1D radial distance)

use rand::Rng;
use std::f64::consts::PI;

/// Generate a random sample from a normal distribution using Box-Muller transform
///
/// # Arguments
/// * `mean` - The mean (μ) of the distribution
/// * `std_dev` - The standard deviation (σ) of the distribution
///
/// # Returns
/// A random sample from N(mean, std_dev²)
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::normal_random;
/// let sample = normal_random(0.0, 1.0);  // Standard normal
/// ```
pub fn normal_random(mean: f64, std_dev: f64) -> f64 {
    let mut rng = rand::thread_rng();

    // Box-Muller transform
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();

    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();

    mean + std_dev * z0
}

/// Generate a random sample from a Rayleigh distribution
///
/// Used for 1D radial miss distances.
///
/// # Arguments
/// * `sigma` - Scale parameter (mode of the distribution)
///
/// # Returns
/// A random sample from Rayleigh(σ)
pub fn rayleigh_random(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen();
    sigma * (-2.0 * u.ln()).sqrt()
}

/// Calculate the mean of a Rayleigh distribution
///
/// # Arguments
/// * `sigma` - Scale parameter
///
/// # Returns
/// Mean = σ * √(π/2)
pub fn rayleigh_mean(sigma: f64) -> f64 {
    sigma * (PI / 2.0).sqrt()
}

/// Calculate the variance of a Rayleigh distribution
///
/// # Arguments
/// * `sigma` - Scale parameter
///
/// # Returns
/// Variance = (2 - π/2) * σ²
pub fn rayleigh_variance(sigma: f64) -> f64 {
    (2.0 - PI / 2.0) * sigma * sigma
}

/// Simulate a 1D shot with potential fat-tail event
///
/// # Arguments
/// * `sigma` - Base standard deviation
/// * `fat_tail_prob` - Probability of extreme mishit
/// * `fat_tail_mult` - Multiplier for dispersion
///
/// # Returns
/// Tuple of (miss_distance, is_fat_tail)
pub fn fat_tail_shot(sigma: f64, fat_tail_prob: f64, fat_tail_mult: f64) -> (f64, bool) {
    let mut rng = rand::thread_rng();
    if rng.gen::<f64>() < fat_tail_prob {
        (rayleigh_random(sigma * fat_tail_mult), true)
    } else {
        (rayleigh_random(sigma), false)
    }
}

// ============================================================================
// Bivariate Normal Distribution (BVN) Functions
// ============================================================================
//
// The BVN models 2D shot dispersion with:
// - Systematic bias: (μ_x, μ_y) - average miss in each direction
// - Elliptical spread: (σ_x, σ_y) - precision in lateral vs distance
// - Correlation: ρ - relationship between x and y coordinates

/// Generate a random sample from a Bivariate Normal distribution
///
/// Models golf shots as 2D coordinates (x, y) with systematic bias and
/// elliptical dispersion. Enables detection of directional tendencies
/// (e.g., "player misses 5 ft right on average").
///
/// # Arguments
/// * `mu_x` - Mean lateral position (feet, positive = right of target)
/// * `mu_y` - Mean distance position (feet, positive = long, negative = short)
/// * `sigma_x` - Standard deviation in lateral direction (feet)
/// * `sigma_y` - Standard deviation in distance direction (feet)
/// * `rho` - Correlation coefficient between x and y (-1 ≤ ρ ≤ 1)
///
/// # Returns
/// A random (x, y) coordinate pair in feet from pin
///
/// # Formula
/// (x, y) ~ BVN(μ_x, μ_y, σ_x, σ_y, ρ)
/// - When ρ = 0: x and y are independent
/// - When ρ > 0: positive correlation (diagonal patterns)
/// - When ρ < 0: negative correlation (anti-diagonal patterns)
///
/// Uses Cholesky decomposition for correlated sampling:
/// - Generate independent Z₀, Z₁ ~ N(0,1)
/// - Transform: X = μ_x + σ_x × Z₀
/// - Transform: Y = μ_y + σ_y × (ρ × Z₀ + √(1-ρ²) × Z₁)
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::bvn_random;
///
/// // Player with diagonal miss pattern (positive correlation)
/// let (x, y) = bvn_random(0.0, 0.0, 10.0, 8.0, 0.7);
///
/// // Player with uncorrelated misses
/// let (x, y) = bvn_random(3.0, 0.0, 10.0, 8.0, 0.0);
///
/// println!("Shot landed at ({:.1}, {:.1}) ft from pin", x, y);
/// ```
pub fn bvn_random(mu_x: f64, mu_y: f64, sigma_x: f64, sigma_y: f64, rho: f64) -> (f64, f64) {
    let mut rng = rand::thread_rng();

    // Box-Muller transform for two independent N(0,1) samples
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();

    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
    let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).sin();

    // Cholesky decomposition for correlated sampling
    // Covariance matrix: [[σ_x², ρσ_xσ_y], [ρσ_xσ_y, σ_y²]]
    // Cholesky factor L: [[σ_x, 0], [ρσ_y, σ_y√(1-ρ²)]]
    let x = mu_x + sigma_x * z0;
    let y = mu_y + sigma_y * (rho * z0 + (1.0 - rho * rho).sqrt() * z1);

    (x, y)
}

/// Calculate the Bivariate Normal PDF at a given point
///
/// Computes the probability density for a 2D shot position. Used for
/// numerical integration when calculating P_max with BVN model.
///
/// # Arguments
/// * `x` - Lateral position (feet from pin)
/// * `y` - Distance position (feet from pin)
/// * `mu_x` - Mean lateral position
/// * `mu_y` - Mean distance position
/// * `sigma_x` - Lateral standard deviation
/// * `sigma_y` - Distance standard deviation
/// * `rho` - Correlation coefficient between x and y (-1 ≤ ρ ≤ 1)
///
/// # Returns
/// Probability density at point (x, y)
///
/// # Formula
/// f(x,y) = 1/(2πσ_xσ_y√(1-ρ²)) × exp(-z/(2(1-ρ²)))
/// where z = (x-μ_x)²/σ_x² - 2ρ(x-μ_x)(y-μ_y)/(σ_xσ_y) + (y-μ_y)²/σ_y²
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::bvn_pdf;
///
/// // Probability density at pin (0,0) for player with diagonal pattern
/// let prob = bvn_pdf(0.0, 0.0, 0.0, 0.0, 10.0, 8.0, 0.7);
/// assert!(prob > 0.0);
///
/// // Uncorrelated case (ρ = 0)
/// let prob_uncorr = bvn_pdf(0.0, 0.0, 0.0, 0.0, 10.0, 8.0, 0.0);
/// assert!(prob_uncorr > 0.0);
/// ```
pub fn bvn_pdf(x: f64, y: f64, mu_x: f64, mu_y: f64, sigma_x: f64, sigma_y: f64, rho: f64) -> f64 {
    if sigma_x <= 0.0 || sigma_y <= 0.0 {
        return 0.0;
    }

    // Clamp rho to valid range to prevent numerical issues
    let rho = rho.clamp(-0.9999, 0.9999);

    // Standardized deviations
    let dx = (x - mu_x) / sigma_x;
    let dy = (y - mu_y) / sigma_y;

    // Quadratic form with correlation term
    let z = dx * dx - 2.0 * rho * dx * dy + dy * dy;

    // Compute exponent with correlation adjustment
    let rho_sq = rho * rho;
    let exp_term = (-z / (2.0 * (1.0 - rho_sq))).exp();

    // Normalization constant includes correlation factor
    let norm = 1.0 / (2.0 * PI * sigma_x * sigma_y * (1.0 - rho_sq).sqrt());

    norm * exp_term
}

/// Simulate a 2D shot with potential fat-tail event
///
/// Extends fat-tail logic to bivariate normal distribution. Models extreme
/// mishits where both lateral and distance dispersion increase simultaneously.
///
/// # Arguments
/// * `mu_x` - Mean lateral position
/// * `mu_y` - Mean distance position
/// * `sigma_x` - Base lateral standard deviation
/// * `sigma_y` - Base distance standard deviation
/// * `rho` - Correlation coefficient
/// * `fat_tail_prob` - Probability of extreme mishit (default: 0.02)
/// * `fat_tail_mult` - Multiplier for dispersions (default: 3.0)
///
/// # Returns
/// Tuple of ((x, y), is_fat_tail)
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::fat_tail_shot_bvn;
///
/// let ((x, y), is_extreme) = fat_tail_shot_bvn(0.0, 0.0, 10.0, 8.0, 0.7, 0.02, 3.0);
/// if is_extreme {
///     println!("Extreme mishit at ({:.1}, {:.1})!", x, y);
/// }
/// ```
pub fn fat_tail_shot_bvn(
    mu_x: f64,
    mu_y: f64,
    sigma_x: f64,
    sigma_y: f64,
    rho: f64,
    fat_tail_prob: f64,
    fat_tail_mult: f64,
) -> ((f64, f64), bool) {
    let mut rng = rand::thread_rng();
    let roll: f64 = rng.gen();

    if roll < fat_tail_prob {
        // Fat-tail event: multiply both sigmas (correlation preserved)
        let (x, y) = bvn_random(
            mu_x,
            mu_y,
            sigma_x * fat_tail_mult,
            sigma_y * fat_tail_mult,
            rho,
        );
        ((x, y), true)
    } else {
        // Normal shot
        let (x, y) = bvn_random(mu_x, mu_y, sigma_x, sigma_y, rho);
        ((x, y), false)
    }
}

/// Get the mean of a Bivariate Normal distribution
///
/// # Arguments
/// * `mu_x` - Lateral mean
/// * `mu_y` - Distance mean
///
/// # Returns
/// Mean vector (μ_x, μ_y)
pub fn bvn_mean(mu_x: f64, mu_y: f64) -> (f64, f64) {
    (mu_x, mu_y)
}

/// Get the covariance matrix of a Bivariate Normal distribution
///
/// Constructs the full 2×2 covariance matrix including correlation.
///
/// # Arguments
/// * `sigma_x` - Lateral standard deviation
/// * `sigma_y` - Distance standard deviation
/// * `rho` - Correlation coefficient between x and y
///
/// # Returns
/// 2×2 covariance matrix [[σ_x², ρσ_xσ_y], [ρσ_xσ_y, σ_y²]]
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::bvn_covariance;
///
/// // Positively correlated (diagonal pattern)
/// let cov_diag = bvn_covariance(10.0, 8.0, 0.7);
///
/// // Uncorrelated (circular/elliptical pattern)
/// let cov_uncorr = bvn_covariance(10.0, 8.0, 0.0);
/// ```
pub fn bvn_covariance(sigma_x: f64, sigma_y: f64, rho: f64) -> [[f64; 2]; 2] {
    let var_x = sigma_x * sigma_x;
    let var_y = sigma_y * sigma_y;
    let cov_xy = rho * sigma_x * sigma_y;

    [[var_x, cov_xy], [cov_xy, var_y]]
}

// ============================================================================
// Log-Probability Functions (MCMC Support)
// ============================================================================
//
// These are used internally by MCMC for Bayesian inference

/// Calculate the log-probability of a value under Normal distribution
///
/// Used for prior distribution in Bayesian inference.
///
/// # Arguments
/// * `x` - Value to evaluate
/// * `mean` - Mean of the normal distribution
/// * `std_dev` - Standard deviation
///
/// # Returns
/// log P(x | μ, σ) = -0.5 × ((x - μ) / σ)²
///
/// # Note
/// The normalization constant (1/√(2πσ²)) is omitted because it cancels
/// out in the Metropolis-Hastings acceptance ratio.
pub fn log_normal_pdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev <= 0.0 {
        return f64::NEG_INFINITY;
    }

    let z = (x - mean) / std_dev;
    -0.5 * z * z
}

/// Calculate the log-probability of observing a miss distance under Rayleigh distribution
///
/// Used for MCMC Bayesian inference to evaluate likelihood of observations.
/// Working in log-space prevents numerical underflow when multiplying many probabilities.
///
/// # Arguments
/// * `distance` - Observed miss distance in feet
/// * `sigma` - Scale parameter (skill level)
///
/// # Returns
/// log P(distance | σ) = log(d/σ²) - d²/(2σ²)
///
/// # Note
/// This is kept for MCMC compatibility. The simulator uses BVN for shot generation,
/// but MCMC still works with radial distances for skill estimation.
pub fn log_rayleigh_pdf(distance: f64, sigma: f64) -> f64 {
    if distance <= 0.0 || sigma <= 0.0 {
        return f64::NEG_INFINITY;
    }

    let sigma_sq = sigma * sigma;
    (distance / sigma_sq).ln() - (distance * distance) / (2.0 * sigma_sq)
}

/// Calculate the log-posterior probability for Bayesian skill estimation
///
/// Combines likelihood and prior using Bayes' theorem:
/// log P(σ | D) = log P(D | σ) + log P(σ) + constant
///
/// Where:
/// - P(D | σ) is the likelihood of observations given skill parameter σ
/// - P(σ) is the prior belief about σ (typically centered on handicap-based estimate)
///
/// # Arguments
/// * `sigma` - Proposed skill parameter value
/// * `observations` - Vec of observed miss distances (feet)
/// * `prior_mean` - Prior mean for σ (from handicap)
/// * `prior_std` - Prior uncertainty (wider = less confident in handicap)
/// * `weights` - Optional observation weights for recency weighting (None = uniform weights)
///
/// # Returns
/// log P(σ | observations) ∝ Σ wᵢ × log P(dᵢ | σ) + log P(σ)
///
/// # Note
/// This is kept for MCMC compatibility. Uses Rayleigh likelihood for radial distances.
pub fn log_posterior(
    sigma: f64,
    observations: &[f64],
    prior_mean: f64,
    prior_std: f64,
    weights: Option<&[f64]>,
) -> f64 {
    if sigma <= 0.0 {
        return f64::NEG_INFINITY;
    }

    // Compute log-likelihood: weighted sum of log-probabilities for each observation
    let log_likelihood: f64 = if let Some(ws) = weights {
        // Exponential recency weighting: recent observations matter more
        observations
            .iter()
            .zip(ws.iter())
            .map(|(&distance, &weight)| weight * log_rayleigh_pdf(distance, sigma))
            .sum()
    } else {
        // Uniform weighting (backward compatibility)
        observations
            .iter()
            .map(|&distance| log_rayleigh_pdf(distance, sigma))
            .sum()
    };

    // Add log-prior
    let log_prior = log_normal_pdf(sigma, prior_mean, prior_std);

    log_likelihood + log_prior
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_normal_random_mean() {
        // Test that normal_random produces samples with approximately correct mean
        let samples: Vec<f64> = (0..10000).map(|_| normal_random(5.0, 2.0)).collect();

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        assert_relative_eq!(mean, 5.0, epsilon = 0.1);
    }

    // ========================================================================
    // BVN (Bivariate Normal) Tests
    // ========================================================================

    #[test]
    fn test_bvn_random_mean() {
        // Test that bvn_random produces samples with approximately correct means
        let mu_x = 3.0; // 3 ft right bias
        let mu_y = -2.0; // 2 ft short bias
        let sigma_x = 10.0;
        let sigma_y = 8.0;
        let rho = 0.0; // No correlation

        let samples: Vec<(f64, f64)> = (0..10000)
            .map(|_| bvn_random(mu_x, mu_y, sigma_x, sigma_y, rho))
            .collect();

        let mean_x = samples.iter().map(|(x, _)| x).sum::<f64>() / samples.len() as f64;
        let mean_y = samples.iter().map(|(_, y)| y).sum::<f64>() / samples.len() as f64;

        assert_relative_eq!(mean_x, mu_x, epsilon = 0.2);
        assert_relative_eq!(mean_y, mu_y, epsilon = 0.2);
    }

    #[test]
    fn test_bvn_random_variance() {
        // Test that bvn_random produces samples with approximately correct variances
        let mu_x = 0.0;
        let mu_y = 0.0;
        let sigma_x = 10.0;
        let sigma_y = 8.0;
        let rho = 0.0; // No correlation

        let samples: Vec<(f64, f64)> = (0..10000)
            .map(|_| bvn_random(mu_x, mu_y, sigma_x, sigma_y, rho))
            .collect();

        let mean_x = samples.iter().map(|(x, _)| x).sum::<f64>() / samples.len() as f64;
        let mean_y = samples.iter().map(|(_, y)| y).sum::<f64>() / samples.len() as f64;

        let var_x = samples
            .iter()
            .map(|(x, _)| (x - mean_x).powi(2))
            .sum::<f64>()
            / samples.len() as f64;

        let var_y = samples
            .iter()
            .map(|(_, y)| (y - mean_y).powi(2))
            .sum::<f64>()
            / samples.len() as f64;

        assert_relative_eq!(var_x, sigma_x * sigma_x, epsilon = 5.0);
        assert_relative_eq!(var_y, sigma_y * sigma_y, epsilon = 5.0);
    }

    #[test]
    fn test_bvn_pdf_properties() {
        let mu_x = 0.0;
        let mu_y = 0.0;
        let sigma_x = 10.0;
        let sigma_y = 8.0;
        let rho = 0.0; // No correlation

        // PDF should be maximum at the mean
        let pdf_at_mean = bvn_pdf(mu_x, mu_y, mu_x, mu_y, sigma_x, sigma_y, rho);
        let pdf_away = bvn_pdf(mu_x + 5.0, mu_y + 5.0, mu_x, mu_y, sigma_x, sigma_y, rho);

        assert!(pdf_at_mean > pdf_away);

        // PDF should be positive everywhere
        assert!(bvn_pdf(0.0, 0.0, mu_x, mu_y, sigma_x, sigma_y, rho) > 0.0);
        assert!(bvn_pdf(10.0, 10.0, mu_x, mu_y, sigma_x, sigma_y, rho) > 0.0);

        // PDF should be 0 for invalid sigmas
        assert_eq!(bvn_pdf(0.0, 0.0, mu_x, mu_y, 0.0, sigma_y, rho), 0.0);
        assert_eq!(bvn_pdf(0.0, 0.0, mu_x, mu_y, sigma_x, -1.0, rho), 0.0);
    }

    #[test]
    fn test_bvn_fat_tail_frequency() {
        // Test that fat-tail events occur at approximately the specified rate
        let trials = 10000;
        let rho = 0.0; // No correlation
        let fat_tail_count = (0..trials)
            .map(|_| fat_tail_shot_bvn(0.0, 0.0, 10.0, 8.0, rho, 0.02, 3.0))
            .filter(|(_, is_fat)| *is_fat)
            .count();

        let frequency = fat_tail_count as f64 / trials as f64;
        assert_relative_eq!(frequency, 0.02, epsilon = 0.005);
    }

    #[test]
    fn test_bvn_helper_functions() {
        // Test mean function
        let (mean_x, mean_y) = bvn_mean(3.0, -2.0);
        assert_eq!(mean_x, 3.0);
        assert_eq!(mean_y, -2.0);

        // Test covariance function with no correlation
        let cov = bvn_covariance(10.0, 8.0, 0.0);
        assert_eq!(cov[0][0], 100.0); // σ_x² = 10² = 100
        assert_eq!(cov[1][1], 64.0); // σ_y² = 8² = 64
        assert_eq!(cov[0][1], 0.0); // No correlation
        assert_eq!(cov[1][0], 0.0); // No correlation

        // Test covariance function with positive correlation
        let cov_pos = bvn_covariance(10.0, 8.0, 0.5);
        assert_eq!(cov_pos[0][0], 100.0); // σ_x² = 10² = 100
        assert_eq!(cov_pos[1][1], 64.0); // σ_y² = 8² = 64
        assert_eq!(cov_pos[0][1], 40.0); // ρσ_xσ_y = 0.5 × 10 × 8 = 40
        assert_eq!(cov_pos[1][0], 40.0); // Symmetric
    }

    #[test]
    fn test_bvn_correlation() {
        // Test that bvn_random produces samples with correct correlation
        let mu_x = 0.0;
        let mu_y = 0.0;
        let sigma_x = 10.0;
        let sigma_y = 10.0;
        let rho = 0.8; // Strong positive correlation

        let samples: Vec<(f64, f64)> = (0..10000)
            .map(|_| bvn_random(mu_x, mu_y, sigma_x, sigma_y, rho))
            .collect();

        let mean_x = samples.iter().map(|(x, _)| x).sum::<f64>() / samples.len() as f64;
        let mean_y = samples.iter().map(|(_, y)| y).sum::<f64>() / samples.len() as f64;

        // Compute sample covariance
        let cov_xy: f64 = samples
            .iter()
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>()
            / samples.len() as f64;

        // Compute sample standard deviations
        let var_x: f64 = samples
            .iter()
            .map(|(x, _)| (x - mean_x).powi(2))
            .sum::<f64>()
            / samples.len() as f64;
        let var_y: f64 = samples
            .iter()
            .map(|(_, y)| (y - mean_y).powi(2))
            .sum::<f64>()
            / samples.len() as f64;

        let sample_sigma_x = var_x.sqrt();
        let sample_sigma_y = var_y.sqrt();

        // Compute sample correlation
        let sample_rho = cov_xy / (sample_sigma_x * sample_sigma_y);

        // Sample correlation should be close to 0.8
        assert_relative_eq!(sample_rho, rho, epsilon = 0.05);
    }

    #[test]
    fn test_bvn_pdf_with_correlation() {
        // Test that PDF with correlation is correctly computed
        let mu_x = 0.0;
        let mu_y = 0.0;
        let sigma_x = 10.0;
        let sigma_y = 10.0;

        // At the mean, PDF with any correlation should be higher than PDF away from mean
        let pdf_at_mean_corr = bvn_pdf(mu_x, mu_y, mu_x, mu_y, sigma_x, sigma_y, 0.7);
        let pdf_away_corr = bvn_pdf(5.0, 5.0, mu_x, mu_y, sigma_x, sigma_y, 0.7);

        assert!(pdf_at_mean_corr > pdf_away_corr);

        // For a positively correlated distribution, points along the diagonal (x ≈ y)
        // should have higher probability than points off the diagonal
        let pdf_diagonal = bvn_pdf(5.0, 5.0, mu_x, mu_y, sigma_x, sigma_y, 0.8);
        let pdf_off_diagonal = bvn_pdf(5.0, -5.0, mu_x, mu_y, sigma_x, sigma_y, 0.8);

        assert!(pdf_diagonal > pdf_off_diagonal);
    }

    #[test]
    fn test_log_normal_pdf_properties() {
        let mean = 28.0;
        let std_dev = 5.0;

        // Maximum at the mean
        let log_pdf_at_mean = log_normal_pdf(mean, mean, std_dev);
        let log_pdf_away = log_normal_pdf(mean + std_dev, mean, std_dev);

        assert!(log_pdf_at_mean > log_pdf_away);

        // Should be -∞ for invalid std_dev
        assert_eq!(log_normal_pdf(28.0, 28.0, 0.0), f64::NEG_INFINITY);
        assert_eq!(log_normal_pdf(28.0, 28.0, -1.0), f64::NEG_INFINITY);

        // At mean, should be 0 (since we omit normalization constant)
        assert_eq!(log_normal_pdf(mean, mean, std_dev), 0.0);

        // One std_dev away should be -0.5
        assert_relative_eq!(
            log_normal_pdf(mean + std_dev, mean, std_dev),
            -0.5,
            epsilon = 1e-10
        );
    }
}
