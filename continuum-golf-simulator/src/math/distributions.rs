// Statistical distributions for shot simulation
//
// Implements:
// - Normal distribution (Box-Muller transform)
// - Rayleigh distribution (miss distance modeling)
// - Fat-tail shot logic (2% chance of 3× worse dispersion)

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
/// The Rayleigh distribution models the miss distance for golf shots.
/// For a 2D radial error with independent normal components (x, y) ~ N(0, σ²),
/// the radial distance d = sqrt(x² + y²) follows Rayleigh(σ).
///
/// # Arguments
/// * `sigma` - Scale parameter (relates to standard deviation of components)
///
/// # Returns
/// A random miss distance in feet
///
/// # Formula
/// d = σ * sqrt(-2 * ln(U)) where U ~ Uniform(0, 1)
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::rayleigh_random;
/// let miss_distance = rayleigh_random(30.0);  // σ = 30 feet
/// ```
pub fn rayleigh_random(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen();

    // Inverse transform sampling for Rayleigh distribution
    sigma * (-2.0 * u.ln()).sqrt()
}

/// Simulate a shot with potential fat-tail event
///
/// Implements the 2% fat-tail logic where shots can have significantly worse
/// dispersion to model extreme mishits (topped shots, shanks, etc.).
///
/// # Arguments
/// * `sigma` - Base skill parameter (miss distance standard deviation)
/// * `fat_tail_prob` - Probability of fat-tail event (default: 0.02)
/// * `fat_tail_mult` - Multiplier for fat-tail dispersion (default: 3.0)
///
/// # Returns
/// Tuple of (miss_distance, is_fat_tail)
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::fat_tail_shot;
/// let (miss, is_extreme) = fat_tail_shot(25.0, 0.02, 3.0);
/// if is_extreme {
///     println!("Extreme mishit! Distance: {:.1}ft", miss);
/// }
/// ```
pub fn fat_tail_shot(sigma: f64, fat_tail_prob: f64, fat_tail_mult: f64) -> (f64, bool) {
    let mut rng = rand::thread_rng();
    let roll: f64 = rng.gen();

    if roll < fat_tail_prob {
        // Fat-tail event: use increased sigma
        let miss_distance = rayleigh_random(sigma * fat_tail_mult);
        (miss_distance, true)
    } else {
        // Normal shot
        let miss_distance = rayleigh_random(sigma);
        (miss_distance, false)
    }
}

/// Calculate the Rayleigh PDF at a given point
///
/// Used for numerical integration when calculating P_max.
///
/// # Arguments
/// * `d` - Miss distance
/// * `sigma` - Scale parameter
///
/// # Returns
/// Probability density at distance d
///
/// # Formula
/// f(d | σ) = (d / σ²) * exp(-d² / 2σ²)
pub fn rayleigh_pdf(d: f64, sigma: f64) -> f64 {
    if d < 0.0 || sigma <= 0.0 {
        return 0.0;
    }

    let sigma_sq = sigma * sigma;
    (d / sigma_sq) * (-(d * d) / (2.0 * sigma_sq)).exp()
}

/// Calculate the expected value (mean) of a Rayleigh distribution
///
/// # Arguments
/// * `sigma` - Scale parameter
///
/// # Returns
/// Expected miss distance
///
/// # Formula
/// E[d] = σ * sqrt(π/2)
pub fn rayleigh_mean(sigma: f64) -> f64 {
    sigma * (PI / 2.0).sqrt()
}

/// Calculate the variance of a Rayleigh distribution
///
/// # Arguments
/// * `sigma` - Scale parameter
///
/// # Returns
/// Variance of miss distance
///
/// # Formula
/// Var[d] = σ² * (4 - π) / 2
pub fn rayleigh_variance(sigma: f64) -> f64 {
    sigma * sigma * (4.0 - PI) / 2.0
}

// ============================================================================
// Bivariate Normal Distribution (BVN) Functions
// ============================================================================
//
// The BVN models 2D shot dispersion with:
// - Systematic bias: (μ_x, μ_y) - average miss in each direction
// - Elliptical spread: (σ_x, σ_y) - precision in lateral vs distance
//
// This replaces the Rayleigh distribution for camera-based (x,y) measurements.

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
///
/// # Returns
/// A random (x, y) coordinate pair in feet from pin
///
/// # Formula
/// (x, y) ~ BVN(μ_x, μ_y, σ_x, σ_y) with independent components:
/// - x ~ N(μ_x, σ_x²)
/// - y ~ N(μ_y, σ_y²)
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::bvn_random;
///
/// // Player with 3 ft right bias, no distance bias
/// // Lateral precision: 10 ft, Distance precision: 8 ft
/// let (x, y) = bvn_random(3.0, 0.0, 10.0, 8.0);
///
/// println!("Shot landed at ({:.1}, {:.1}) ft from pin", x, y);
/// ```
pub fn bvn_random(mu_x: f64, mu_y: f64, sigma_x: f64, sigma_y: f64) -> (f64, f64) {
    let mut rng = rand::thread_rng();

    // Box-Muller transform for two independent N(0,1) samples
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();

    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
    let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).sin();

    // Scale by standard deviations and shift by means
    let x = mu_x + sigma_x * z0;
    let y = mu_y + sigma_y * z1;

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
///
/// # Returns
/// Probability density at point (x, y)
///
/// # Formula
/// f(x,y) = (1 / (2π σ_x σ_y)) × exp(-0.5 × [(x-μ_x)²/σ_x² + (y-μ_y)²/σ_y²])
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::bvn_pdf;
///
/// // Probability density at pin (0,0) for unbiased player
/// let prob = bvn_pdf(0.0, 0.0, 0.0, 0.0, 10.0, 8.0);
/// assert!(prob > 0.0);
/// ```
pub fn bvn_pdf(x: f64, y: f64, mu_x: f64, mu_y: f64, sigma_x: f64, sigma_y: f64) -> f64 {
    if sigma_x <= 0.0 || sigma_y <= 0.0 {
        return 0.0;
    }

    // Standardized deviations
    let dx = (x - mu_x) / sigma_x;
    let dy = (y - mu_y) / sigma_y;

    // Exponent term
    let exp_term = (-0.5 * (dx * dx + dy * dy)).exp();

    // Normalization constant
    let norm = 1.0 / (2.0 * PI * sigma_x * sigma_y);

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
/// let ((x, y), is_extreme) = fat_tail_shot_bvn(0.0, 0.0, 10.0, 8.0, 0.02, 3.0);
/// if is_extreme {
///     println!("Extreme mishit at ({:.1}, {:.1})!", x, y);
/// }
/// ```
pub fn fat_tail_shot_bvn(
    mu_x: f64,
    mu_y: f64,
    sigma_x: f64,
    sigma_y: f64,
    fat_tail_prob: f64,
    fat_tail_mult: f64,
) -> ((f64, f64), bool) {
    let mut rng = rand::thread_rng();
    let roll: f64 = rng.gen();

    if roll < fat_tail_prob {
        // Fat-tail event: multiply both sigmas
        let (x, y) = bvn_random(mu_x, mu_y, sigma_x * fat_tail_mult, sigma_y * fat_tail_mult);
        ((x, y), true)
    } else {
        // Normal shot
        let (x, y) = bvn_random(mu_x, mu_y, sigma_x, sigma_y);
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
/// For independent x and y components (no correlation), the covariance
/// matrix is diagonal.
///
/// # Arguments
/// * `sigma_x` - Lateral standard deviation
/// * `sigma_y` - Distance standard deviation
///
/// # Returns
/// 2×2 covariance matrix [[σ_x², 0], [0, σ_y²]]
///
/// # Note
/// This assumes independence (ρ = 0). Future versions may add correlation.
pub fn bvn_covariance(sigma_x: f64, sigma_y: f64) -> [[f64; 2]; 2] {
    let var_x = sigma_x * sigma_x;
    let var_y = sigma_y * sigma_y;

    [[var_x, 0.0], [0.0, var_y]]
}

// ============================================================================
// Bayesian Inference Functions for MCMC
// ============================================================================
//
// These log-probability functions enable MCMC sampling for skill estimation.
// Using log-probabilities prevents numerical underflow in likelihood calculations.

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
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::log_rayleigh_pdf;
///
/// let sigma = 30.0;
/// let observed_distance = 25.0;
/// let log_likelihood = log_rayleigh_pdf(observed_distance, sigma);
/// ```
pub fn log_rayleigh_pdf(distance: f64, sigma: f64) -> f64 {
    if distance <= 0.0 || sigma <= 0.0 {
        return f64::NEG_INFINITY;
    }

    let sigma_sq = sigma * sigma;
    (distance / sigma_sq).ln() - (distance * distance) / (2.0 * sigma_sq)
}

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
///
/// # Returns
/// log P(σ | observations) ∝ Σ log P(dᵢ | σ) + log P(σ)
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::distributions::log_posterior;
///
/// let observations = vec![25.0, 30.0, 22.0, 35.0];
/// let prior_mean = 28.0;  // From handicap
/// let prior_std = 5.0;    // Prior uncertainty
///
/// let sigma_proposal = 27.5;
/// let log_prob = log_posterior(sigma_proposal, &observations, prior_mean, prior_std);
/// ```
pub fn log_posterior(
    sigma: f64,
    observations: &[f64],
    prior_mean: f64,
    prior_std: f64,
) -> f64 {
    if sigma <= 0.0 {
        return f64::NEG_INFINITY;
    }

    // Compute log-likelihood: sum of log-probabilities for each observation
    let log_likelihood: f64 = observations
        .iter()
        .map(|&distance| log_rayleigh_pdf(distance, sigma))
        .sum();

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
        let samples: Vec<f64> = (0..10000)
            .map(|_| normal_random(5.0, 2.0))
            .collect();

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        assert_relative_eq!(mean, 5.0, epsilon = 0.1);
    }

    #[test]
    fn test_rayleigh_random_mean() {
        // Test that rayleigh_random produces samples with approximately correct mean
        let sigma = 30.0;
        let samples: Vec<f64> = (0..10000)
            .map(|_| rayleigh_random(sigma))
            .collect();

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let expected_mean = rayleigh_mean(sigma);

        assert_relative_eq!(mean, expected_mean, epsilon = 1.0);
    }

    #[test]
    fn test_fat_tail_frequency() {
        // Test that fat-tail events occur at approximately the specified rate
        let trials = 10000;
        let fat_tail_count = (0..trials)
            .map(|_| fat_tail_shot(25.0, 0.02, 3.0))
            .filter(|(_, is_fat)| *is_fat)
            .count();

        let frequency = fat_tail_count as f64 / trials as f64;
        assert_relative_eq!(frequency, 0.02, epsilon = 0.005);
    }

    #[test]
    fn test_rayleigh_pdf_properties() {
        let sigma = 30.0;

        // PDF should be 0 at d=0
        assert_eq!(rayleigh_pdf(0.0, sigma), 0.0);

        // PDF should be positive for d > 0
        assert!(rayleigh_pdf(10.0, sigma) > 0.0);

        // PDF should be 0 for negative d
        assert_eq!(rayleigh_pdf(-5.0, sigma), 0.0);
    }

    #[test]
    fn test_rayleigh_mean_formula() {
        let sigma = 25.0;
        let expected = sigma * (PI / 2.0).sqrt();
        assert_relative_eq!(rayleigh_mean(sigma), expected, epsilon = 1e-10);
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

        let samples: Vec<(f64, f64)> = (0..10000)
            .map(|_| bvn_random(mu_x, mu_y, sigma_x, sigma_y))
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

        let samples: Vec<(f64, f64)> = (0..10000)
            .map(|_| bvn_random(mu_x, mu_y, sigma_x, sigma_y))
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
    fn test_bvn_symmetry_reduces_to_rayleigh() {
        // CRITICAL TEST: When μ_x = μ_y = 0 and σ_x = σ_y = σ,
        // the radial distance sqrt(x² + y²) should follow Rayleigh(σ)

        let sigma = 30.0;

        // Generate BVN samples with no bias and equal dispersions
        let bvn_samples: Vec<(f64, f64)> = (0..10000)
            .map(|_| bvn_random(0.0, 0.0, sigma, sigma))
            .collect();

        // Convert to radial distances
        let bvn_radial: Vec<f64> = bvn_samples
            .iter()
            .map(|(x, y)| (x * x + y * y).sqrt())
            .collect();

        // Generate direct Rayleigh samples
        let rayleigh_samples: Vec<f64> = (0..10000).map(|_| rayleigh_random(sigma)).collect();

        // Compare means
        let bvn_mean = bvn_radial.iter().sum::<f64>() / bvn_radial.len() as f64;
        let rayleigh_mean_val = rayleigh_samples.iter().sum::<f64>() / rayleigh_samples.len() as f64;

        assert_relative_eq!(bvn_mean, rayleigh_mean_val, epsilon = 1.0);

        // Compare to theoretical Rayleigh mean
        let expected_mean = rayleigh_mean(sigma);
        assert_relative_eq!(bvn_mean, expected_mean, epsilon = 1.5);
    }

    #[test]
    fn test_bvn_pdf_properties() {
        let mu_x = 0.0;
        let mu_y = 0.0;
        let sigma_x = 10.0;
        let sigma_y = 8.0;

        // PDF should be maximum at the mean
        let pdf_at_mean = bvn_pdf(mu_x, mu_y, mu_x, mu_y, sigma_x, sigma_y);
        let pdf_away = bvn_pdf(mu_x + 5.0, mu_y + 5.0, mu_x, mu_y, sigma_x, sigma_y);

        assert!(pdf_at_mean > pdf_away);

        // PDF should be positive everywhere
        assert!(bvn_pdf(0.0, 0.0, mu_x, mu_y, sigma_x, sigma_y) > 0.0);
        assert!(bvn_pdf(10.0, 10.0, mu_x, mu_y, sigma_x, sigma_y) > 0.0);

        // PDF should be 0 for invalid sigmas
        assert_eq!(bvn_pdf(0.0, 0.0, mu_x, mu_y, 0.0, sigma_y), 0.0);
        assert_eq!(bvn_pdf(0.0, 0.0, mu_x, mu_y, sigma_x, -1.0), 0.0);
    }

    #[test]
    fn test_bvn_fat_tail_frequency() {
        // Test that fat-tail events occur at approximately the specified rate
        let trials = 10000;
        let fat_tail_count = (0..trials)
            .map(|_| fat_tail_shot_bvn(0.0, 0.0, 10.0, 8.0, 0.02, 3.0))
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

        // Test covariance function
        let cov = bvn_covariance(10.0, 8.0);
        assert_eq!(cov[0][0], 100.0); // σ_x² = 10² = 100
        assert_eq!(cov[1][1], 64.0); // σ_y² = 8² = 64
        assert_eq!(cov[0][1], 0.0); // No correlation
        assert_eq!(cov[1][0], 0.0); // No correlation
    }

    // ========================================================================
    // Bayesian Inference Tests
    // ========================================================================

    #[test]
    fn test_log_rayleigh_pdf_properties() {
        let sigma = 30.0;

        // Log-PDF should be -∞ for invalid inputs
        assert_eq!(log_rayleigh_pdf(0.0, sigma), f64::NEG_INFINITY);
        assert_eq!(log_rayleigh_pdf(-5.0, sigma), f64::NEG_INFINITY);
        assert_eq!(log_rayleigh_pdf(10.0, 0.0), f64::NEG_INFINITY);
        assert_eq!(log_rayleigh_pdf(10.0, -5.0), f64::NEG_INFINITY);

        // Log-PDF should be finite for valid inputs
        let log_pdf = log_rayleigh_pdf(25.0, sigma);
        assert!(log_pdf.is_finite());
        assert!(log_pdf < 0.0); // Log of probability should be negative
    }

    #[test]
    fn test_log_rayleigh_matches_pdf() {
        // Verify that log_rayleigh_pdf = log(rayleigh_pdf)
        let sigma = 30.0;
        let distance = 25.0;

        let pdf = rayleigh_pdf(distance, sigma);
        let log_pdf = log_rayleigh_pdf(distance, sigma);

        assert_relative_eq!(log_pdf, pdf.ln(), epsilon = 1e-10);
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

    #[test]
    fn test_log_posterior_combines_likelihood_and_prior() {
        let observations = vec![25.0, 30.0, 22.0, 35.0];
        let prior_mean = 28.0;
        let prior_std = 5.0;
        let sigma = 27.5;

        // Compute posterior
        let posterior = log_posterior(sigma, &observations, prior_mean, prior_std);

        // Manually compute likelihood and prior
        let likelihood: f64 = observations
            .iter()
            .map(|&d| log_rayleigh_pdf(d, sigma))
            .sum();
        let prior = log_normal_pdf(sigma, prior_mean, prior_std);

        // Posterior should equal likelihood + prior
        assert_relative_eq!(posterior, likelihood + prior, epsilon = 1e-10);
    }

    #[test]
    fn test_log_posterior_rejects_invalid_sigma() {
        let observations = vec![25.0, 30.0];
        let prior_mean = 28.0;
        let prior_std = 5.0;

        // Negative sigma should give -∞
        assert_eq!(
            log_posterior(-5.0, &observations, prior_mean, prior_std),
            f64::NEG_INFINITY
        );

        // Zero sigma should give -∞
        assert_eq!(
            log_posterior(0.0, &observations, prior_mean, prior_std),
            f64::NEG_INFINITY
        );
    }

    #[test]
    fn test_log_posterior_prefers_sigma_near_observations() {
        let observations = vec![30.0, 32.0, 28.0, 31.0]; // Cluster around 30
        let prior_mean = 25.0; // Prior centered elsewhere
        let prior_std = 10.0; // Weak prior

        // Sigma=30 (near observations) should have higher posterior than sigma=25
        let posterior_30 = log_posterior(30.0, &observations, prior_mean, prior_std);
        let posterior_25 = log_posterior(25.0, &observations, prior_mean, prior_std);

        assert!(posterior_30 > posterior_25);
    }

    #[test]
    fn test_log_posterior_numerical_stability() {
        // Test with many observations (should not overflow/underflow)
        let observations: Vec<f64> = (0..1000).map(|_| 25.0).collect();
        let prior_mean = 28.0;
        let prior_std = 5.0;
        let sigma = 27.5;

        let posterior = log_posterior(sigma, &observations, prior_mean, prior_std);

        // Should be finite (not overflow to ±∞)
        assert!(posterior.is_finite());

        // Should be very negative (many observations = strong evidence)
        assert!(posterior < -100.0);
    }
}
