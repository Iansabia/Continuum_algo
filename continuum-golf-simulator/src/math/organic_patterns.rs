use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Parameters for a Bivariate Normal Distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BVNParameters {
    pub mu_x: f64,
    pub mu_y: f64,
    pub sigma_x: f64,
    pub sigma_y: f64,
    pub rho: f64,
}

/// An organic pattern representing a player's shot dispersion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganicPattern {
    /// Boundary points defining the pattern shape (in feet)
    pub boundary_points: Vec<(f64, f64)>,
    /// Center offset from target (0, 0)
    pub center_offset: (f64, f64),
    /// Effective sigma calculated from pattern size
    pub effective_sigma: f64,
}

impl OrganicPattern {
    /// Convert organic pattern to BVN parameters
    /// Uses bounding box analysis like PatternDrawingDemo
    pub fn to_bvn_parameters(&self) -> BVNParameters {
        let points = &self.boundary_points;

        if points.is_empty() {
            // Fallback for empty patterns
            return BVNParameters {
                mu_x: 0.0,
                mu_y: 0.0,
                sigma_x: self.effective_sigma,
                sigma_y: self.effective_sigma,
                rho: 0.0,
            };
        }

        // Calculate bounding box
        let xs: Vec<f64> = points.iter().map(|(x, _)| *x).collect();
        let ys: Vec<f64> = points.iter().map(|(_, y)| *y).collect();

        let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        // Pattern dimensions
        let width = max_x - min_x;
        let height = max_y - min_y;

        // BVN parameters: σ = radius/2 (95% containment)
        // width = 2R, so R = width/2, σ = R/2 = width/4
        let sigma_x = width / 4.0;
        let sigma_y = height / 4.0;

        // Bias from center offset
        let mu_x = self.center_offset.0;
        let mu_y = self.center_offset.1;

        // Estimate correlation from point cloud shape
        let rho = self.estimate_correlation(points);

        BVNParameters {
            mu_x,
            mu_y,
            sigma_x,
            sigma_y,
            rho,
        }
    }

    /// Estimate correlation coefficient from point cloud
    /// Calculates sample correlation like PatternDrawingDemo
    fn estimate_correlation(&self, points: &[(f64, f64)]) -> f64 {
        let n = points.len() as f64;

        if n < 2.0 {
            return 0.0;
        }

        // Calculate means
        let mean_x: f64 = points.iter().map(|(x, _)| x).sum::<f64>() / n;
        let mean_y: f64 = points.iter().map(|(_, y)| y).sum::<f64>() / n;

        // Calculate variances
        let var_x: f64 = points.iter()
            .map(|(x, _)| (x - mean_x).powi(2))
            .sum::<f64>() / n;
        let var_y: f64 = points.iter()
            .map(|(_, y)| (y - mean_y).powi(2))
            .sum::<f64>() / n;

        // Calculate covariance
        let cov_xy: f64 = points.iter()
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>() / n;

        let sigma_x = var_x.sqrt();
        let sigma_y = var_y.sqrt();

        let rho = if sigma_x > 1e-6 && sigma_y > 1e-6 {
            cov_xy / (sigma_x * sigma_y)
        } else {
            0.0
        };

        // Clamp to valid range (avoid numerical issues)
        rho.clamp(-0.99, 0.99)
    }
}

/// Generator for organic shot dispersion patterns
#[derive(Debug, Clone)]
pub struct OrganicPatternGenerator {
    /// Random seed for reproducibility
    seed: u64,
    /// Base radius (controls pattern size and thus skill level)
    base_radius: f64,
    /// Irregularity factor (0.0 = perfect circle, 1.0 = very organic)
    irregularity: f64,
    /// Asymmetry factor (0.0 = symmetric, 1.0 = highly asymmetric)
    asymmetry: f64,
    /// Number of harmonics for Fourier series (more = more detail)
    num_harmonics: usize,
}

impl OrganicPatternGenerator {
    /// Create a new pattern generator with specific parameters
    pub fn new(seed: u64, base_radius: f64, irregularity: f64, asymmetry: f64) -> Self {
        Self {
            seed,
            base_radius,
            irregularity,
            asymmetry,
            num_harmonics: 5,
        }
    }

    /// Create a random pattern generator for a target skill level
    ///
    /// # Arguments
    /// * `rng` - Random number generator
    /// * `target_sigma` - Target skill level (sigma in feet)
    pub fn create_random(rng: &mut impl Rng, target_sigma: f64) -> Self {
        // Target skill (sigma) maps to base radius
        // σ = R/2, so R = 2σ
        let base_radius = target_sigma * 2.0;

        // Randomize pattern characteristics for variety
        let irregularity = rng.gen_range(0.2..0.8);  // Never too perfect or too chaotic
        let asymmetry = rng.gen_range(0.0..0.5);     // Slight natural biases

        Self::new(rng.gen(), base_radius, irregularity, asymmetry)
    }

    /// Generate an organic pattern using Fourier series
    pub fn generate(&self) -> OrganicPattern {
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);

        // Generate Fourier series coefficients
        // r(θ) = base_radius + Σ(a_n × cos(n×θ) + b_n × sin(n×θ))
        let mut fourier_coeffs = Vec::new();

        for n in 1..=self.num_harmonics {
            let amplitude = self.irregularity * self.base_radius / (n as f64);
            let a_n = rng.gen_range(-amplitude..amplitude);
            let b_n = rng.gen_range(-amplitude..amplitude);
            fourier_coeffs.push((a_n, b_n));
        }

        // Generate center offset for asymmetry
        let center_x = self.asymmetry * self.base_radius * rng.gen_range(-0.3..0.3);
        let center_y = self.asymmetry * self.base_radius * rng.gen_range(-0.3..0.3);

        // Generate boundary points
        let num_points = 100;
        let mut boundary_points = Vec::new();

        for i in 0..num_points {
            let theta = 2.0 * PI * (i as f64) / (num_points as f64);

            // Base radius with Fourier perturbations
            let mut r = self.base_radius;
            for (n, (a_n, b_n)) in fourier_coeffs.iter().enumerate() {
                let n_val = (n + 1) as f64;
                r += a_n * (n_val * theta).cos() + b_n * (n_val * theta).sin();
            }

            // Ensure radius stays positive
            r = r.max(self.base_radius * 0.3);

            // Convert polar to Cartesian with center offset
            let x = center_x + r * theta.cos();
            let y = center_y + r * theta.sin();

            boundary_points.push((x, y));
        }

        // Calculate effective sigma (should be ≈ base_radius / 2)
        let effective_sigma = self.calculate_effective_sigma();

        OrganicPattern {
            boundary_points,
            center_offset: (center_x, center_y),
            effective_sigma,
        }
    }

    /// Calculate effective sigma from pattern size
    /// Uses the relationship σ = R/2 (95% containment)
    fn calculate_effective_sigma(&self) -> f64 {
        self.base_radius / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_pattern_generation() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let generator = OrganicPatternGenerator::create_random(&mut rng, 25.0);
        let pattern = generator.generate();

        // Should have 100 boundary points
        assert_eq!(pattern.boundary_points.len(), 100);

        // Effective sigma should be around 25 (target sigma)
        assert!((pattern.effective_sigma - 25.0).abs() < 1.0);
    }

    #[test]
    fn test_bvn_parameters() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let generator = OrganicPatternGenerator::create_random(&mut rng, 30.0);
        let pattern = generator.generate();
        let bvn_params = pattern.to_bvn_parameters();

        // Sigma should be approximately 30 (with some variation from bounding box)
        assert!(bvn_params.sigma_x > 15.0 && bvn_params.sigma_x < 50.0);
        assert!(bvn_params.sigma_y > 15.0 && bvn_params.sigma_y < 50.0);

        // Rho should be valid
        assert!(bvn_params.rho >= -0.99 && bvn_params.rho <= 0.99);
    }

    #[test]
    fn test_pattern_variety() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // Generate multiple patterns
        let patterns: Vec<_> = (0..10)
            .map(|_| {
                let gen = OrganicPatternGenerator::create_random(&mut rng, 25.0);
                gen.generate()
            })
            .collect();

        // Check that patterns are different
        for i in 0..patterns.len() {
            for j in (i + 1)..patterns.len() {
                // Compare first boundary point (should be different)
                let p1 = patterns[i].boundary_points[0];
                let p2 = patterns[j].boundary_points[0];
                let dist = ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt();
                assert!(dist > 1.0, "Patterns should be visually distinct");
            }
        }
    }

    #[test]
    fn test_sigma_correlation() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // Test different target sigmas
        let target_sigmas = vec![15.0, 25.0, 35.0, 50.0];

        for target_sigma in target_sigmas {
            let gen = OrganicPatternGenerator::create_random(&mut rng, target_sigma);
            let pattern = gen.generate();
            let bvn_params = pattern.to_bvn_parameters();

            // Average sigma should be close to target
            let avg_sigma = (bvn_params.sigma_x + bvn_params.sigma_y) / 2.0;
            let error = (avg_sigma - target_sigma).abs() / target_sigma;

            // Allow 30% error due to irregularity and bounding box estimation
            assert!(error < 0.3,
                "Sigma mismatch: target={}, actual={}, error={}%",
                target_sigma, avg_sigma, error * 100.0);
        }
    }

    #[test]
    fn test_reproducibility() {
        let seed = 12345;

        let gen1 = OrganicPatternGenerator::new(seed, 50.0, 0.5, 0.3);
        let pattern1 = gen1.generate();

        let gen2 = OrganicPatternGenerator::new(seed, 50.0, 0.5, 0.3);
        let pattern2 = gen2.generate();

        // Same seed should produce identical patterns
        assert_eq!(pattern1.boundary_points.len(), pattern2.boundary_points.len());

        for i in 0..pattern1.boundary_points.len() {
            let (x1, y1) = pattern1.boundary_points[i];
            let (x2, y2) = pattern2.boundary_points[i];
            assert!((x1 - x2).abs() < 1e-10);
            assert!((y1 - y2).abs() < 1e-10);
        }
    }
}
