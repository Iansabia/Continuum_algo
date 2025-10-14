// Numerical integration methods for P_max calculation
//
// Implements trapezoidal rule and adaptive integration for computing
// expected payout integrals needed for dynamic odds calculation.

/// Integrate a function using the trapezoidal rule
///
/// The trapezoidal rule approximates the definite integral by dividing
/// the interval into n trapezoids and summing their areas.
///
/// # Arguments
/// * `f` - Function to integrate
/// * `a` - Lower bound of integration
/// * `b` - Upper bound of integration
/// * `n` - Number of subdivisions (more = more accurate)
///
/// # Returns
/// Approximate value of ∫[a,b] f(x) dx
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::integration::trapezoidal_rule;
///
/// // Integrate x² from 0 to 1 (should be ~0.333)
/// let result = trapezoidal_rule(|x| x * x, 0.0, 1.0, 1000);
/// assert!((result - 0.333).abs() < 0.001);
/// ```
pub fn trapezoidal_rule<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    if n == 0 {
        return 0.0;
    }

    let h = (b - a) / n as f64;
    let mut sum = 0.5 * (f(a) + f(b));

    for i in 1..n {
        let x = a + i as f64 * h;
        sum += f(x);
    }

    h * sum
}

/// Adaptive integration using recursive subdivision
///
/// Automatically refines the mesh in regions where the function varies rapidly.
/// Stops when the estimated error is below the tolerance.
///
/// # Arguments
/// * `f` - Function to integrate
/// * `a` - Lower bound
/// * `b` - Upper bound
/// * `tol` - Error tolerance
/// * `max_depth` - Maximum recursion depth (prevents infinite recursion)
///
/// # Returns
/// Approximate integral with error < tol
///
/// # Example
/// ```
/// use continuum_golf_simulator::math::integration::adaptive_integration;
///
/// // Integrate sin(x) from 0 to π (should be 2.0)
/// let result = adaptive_integration(|x: f64| x.sin(), 0.0, std::f64::consts::PI, 1e-6, 10);
/// assert!((result - 2.0).abs() < 1e-5);
/// ```
pub fn adaptive_integration<F>(f: F, a: f64, b: f64, tol: f64, max_depth: usize) -> f64
where
    F: Fn(f64) -> f64 + Copy,
{
    adaptive_integration_recursive(f, a, b, tol, max_depth, 0)
}

fn adaptive_integration_recursive<F>(
    f: F,
    a: f64,
    b: f64,
    tol: f64,
    max_depth: usize,
    depth: usize,
) -> f64
where
    F: Fn(f64) -> f64 + Copy,
{
    // Base case: if max depth reached, use trapezoidal rule
    if depth >= max_depth {
        return trapezoidal_rule(f, a, b, 10);
    }

    let mid = (a + b) / 2.0;

    // Compute integral over [a, b] with coarse resolution
    let whole = trapezoidal_rule(f, a, b, 10);

    // Compute integral as sum of [a, mid] + [mid, b]
    let left = trapezoidal_rule(f, a, mid, 10);
    let right = trapezoidal_rule(f, mid, b, 10);
    let sum = left + right;

    // Estimate error
    let error = (sum - whole).abs();

    if error < tol {
        // Error is acceptable, return the better estimate
        sum
    } else {
        // Error too large, subdivide
        adaptive_integration_recursive(f, a, mid, tol / 2.0, max_depth, depth + 1)
            + adaptive_integration_recursive(f, mid, b, tol / 2.0, max_depth, depth + 1)
    }
}

/// Simpson's rule for numerical integration
///
/// More accurate than trapezoidal rule for smooth functions.
/// Uses parabolic approximation instead of linear.
///
/// # Arguments
/// * `f` - Function to integrate
/// * `a` - Lower bound
/// * `b` - Upper bound
/// * `n` - Number of subdivisions (must be even)
///
/// # Returns
/// Approximate integral
///
/// # Panics
/// Panics if n is odd
pub fn simpsons_rule<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    assert!(n % 2 == 0, "n must be even for Simpson's rule");
    assert!(n > 0, "n must be positive");

    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);

    for i in 1..n {
        let x = a + i as f64 * h;
        let coefficient = if i % 2 == 0 { 2.0 } else { 4.0 };
        sum += coefficient * f(x);
    }

    (h / 3.0) * sum
}

/// Integrate the payout function for P_max calculation
///
/// Computes: ∫[0, d_max] (1 - d/d_max)^k * PDF(d | σ) dd
///
/// This integral represents the expected payout multiplier fraction
/// that must be scaled to achieve target RTP.
///
/// # Arguments
/// * `d_max` - Maximum scoring radius (feet)
/// * `k` - Steepness parameter
/// * `sigma` - Player skill parameter
/// * `pdf_fn` - Probability density function for miss distance
/// * `n` - Number of integration points
///
/// # Returns
/// Integral value (between 0 and 1)
pub fn integrate_payout_function<F>(
    d_max: f64,
    k: f64,
    sigma: f64,
    pdf_fn: F,
    n: usize,
) -> f64
where
    F: Fn(f64, f64) -> f64,
{
    let integrand = |d: f64| {
        if d > d_max {
            0.0
        } else {
            let payout_fraction = (1.0 - d / d_max).powf(k);
            payout_fraction * pdf_fn(d, sigma)
        }
    };

    trapezoidal_rule(integrand, 0.0, d_max, n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    #[test]
    fn test_trapezoidal_rule_polynomial() {
        // Integrate x² from 0 to 1 = 1/3
        let result = trapezoidal_rule(|x| x * x, 0.0, 1.0, 1000);
        assert_relative_eq!(result, 1.0 / 3.0, epsilon = 0.001);
    }

    #[test]
    fn test_trapezoidal_rule_sine() {
        // Integrate sin(x) from 0 to π = 2
        let result = trapezoidal_rule(|x| x.sin(), 0.0, PI, 1000);
        assert_relative_eq!(result, 2.0, epsilon = 0.001);
    }

    #[test]
    fn test_simpsons_rule_polynomial() {
        // Simpson's rule should be exact for polynomials up to degree 3
        // Integrate x³ from 0 to 1 = 1/4
        let result = simpsons_rule(|x| x * x * x, 0.0, 1.0, 100);
        assert_relative_eq!(result, 0.25, epsilon = 1e-10);
    }

    #[test]
    fn test_adaptive_integration() {
        // Test with a simple polynomial
        let f = |x: f64| x * x;
        let result = adaptive_integration(f, 0.0, 1.0, 1e-6, 15);

        // Analytical result: ∫ x² dx from 0 to 1 = 1/3
        let expected = 1.0 / 3.0;
        assert_relative_eq!(result, expected, epsilon = 0.001);
    }

    #[test]
    fn test_integrate_payout_simple() {
        // Test with uniform PDF (not realistic but easy to verify)
        let d_max = 100.0;
        let k = 5.0;

        // Uniform PDF: f(d) = 1/d_max for d in [0, d_max]
        let uniform_pdf = |_d: f64, _sigma: f64| 1.0 / d_max;

        let result = integrate_payout_function(d_max, k, 30.0, uniform_pdf, 1000);

        // For uniform PDF, integral = (1/d_max) * ∫[0,d_max] (1-d/d_max)^k dd
        // Let u = 1 - d/d_max, then du = -1/d_max dd
        // = ∫[1,0] u^k (-d_max) du = d_max * ∫[0,1] u^k du = d_max / (k+1)
        // So integral = (1/d_max) * d_max/(k+1) = 1/(k+1)
        let expected = 1.0 / (k + 1.0);

        assert_relative_eq!(result, expected, epsilon = 0.01);
    }

    #[test]
    #[should_panic(expected = "n must be even")]
    fn test_simpsons_rule_odd_n() {
        simpsons_rule(|x| x, 0.0, 1.0, 99);
    }
}
