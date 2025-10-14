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
}
