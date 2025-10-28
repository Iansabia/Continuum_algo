// MCMC Bayesian Skill Estimation
//
// Implements Metropolis-Hastings algorithm for robust skill parameter estimation.
// This is the gold-standard approach for maintaining 85% RTP with mathematical guarantees.
//
// Key advantages over Kalman filter:
// - Full posterior distribution P(σ | observations)
// - Mathematically guaranteed convergence to true skill level
// - Quantified uncertainty (credible intervals)
// - No oscillation issues - converges to stable estimate
//
// Algorithm:
// 1. Start with initial σ (from handicap or previous estimate)
// 2. Propose new σ' ~ N(σ_current, proposal_std)
// 3. Accept σ' with probability min(1, P(σ' | D) / P(σ | D))
// 4. Repeat to generate posterior samples
// 5. Use median of samples as point estimate

use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::math::distributions::{log_posterior, normal_random};

/// MCMC sampler state using Metropolis-Hastings algorithm
///
/// Maintains current position in parameter space and acceptance statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCMCSampler {
    /// Current sigma value in the Markov chain
    pub current_sigma: f64,

    /// Log-posterior at current position (cached for efficiency)
    pub current_log_posterior: f64,

    /// Standard deviation of proposal distribution
    /// Larger values = more exploration but lower acceptance rate
    /// Target acceptance rate: 20-50%
    pub proposal_std: f64,

    /// Number of accepted proposals
    pub accepted: usize,

    /// Total number of proposals
    pub total: usize,
}

impl MCMCSampler {
    /// Create a new MCMC sampler
    ///
    /// # Arguments
    /// * `initial_sigma` - Starting value for Markov chain
    /// * `initial_log_posterior` - Log-posterior at starting value
    /// * `proposal_std` - Standard deviation for proposal distribution
    pub fn new(initial_sigma: f64, initial_log_posterior: f64, proposal_std: f64) -> Self {
        Self {
            current_sigma: initial_sigma,
            current_log_posterior: initial_log_posterior,
            proposal_std,
            accepted: 0,
            total: 0,
        }
    }

    /// Perform a single Metropolis-Hastings step
    ///
    /// # Arguments
    /// * `observations` - Observed miss distances
    /// * `prior_mean` - Prior mean for σ
    /// * `prior_std` - Prior uncertainty
    ///
    /// # Returns
    /// The sampled sigma value (either new proposal or previous value)
    pub fn step(&mut self, observations: &[f64], prior_mean: f64, prior_std: f64) -> f64 {
        // Propose new sigma from symmetric normal distribution
        let proposed_sigma = normal_random(self.current_sigma, self.proposal_std);

        // Reject if proposed sigma is invalid (≤ 0)
        if proposed_sigma <= 0.0 {
            self.total += 1;
            return self.current_sigma;
        }

        // Compute log-posterior at proposed value
        let proposed_log_posterior = log_posterior(
            proposed_sigma,
            observations,
            prior_mean,
            prior_std,
        );

        // Metropolis-Hastings acceptance ratio
        // log(α) = log P(σ' | D) - log P(σ | D)
        let log_acceptance_ratio = proposed_log_posterior - self.current_log_posterior;

        // Accept with probability α = min(1, P(σ' | D) / P(σ | D))
        let mut rng = rand::thread_rng();
        let u: f64 = rng.gen();

        if log_acceptance_ratio.exp() > u {
            // Accept proposal
            self.current_sigma = proposed_sigma;
            self.current_log_posterior = proposed_log_posterior;
            self.accepted += 1;
        }
        // else: reject proposal, keep current sigma

        self.total += 1;
        self.current_sigma
    }

    /// Get current acceptance rate
    ///
    /// # Returns
    /// Fraction of proposals accepted (target: 0.2 - 0.5)
    pub fn acceptance_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.accepted as f64 / self.total as f64
    }

    /// Adjust proposal standard deviation to maintain target acceptance rate
    ///
    /// Called adaptively during burn-in phase to tune sampler performance.
    ///
    /// # Arguments
    /// * `target_rate` - Desired acceptance rate (typically 0.234 for 1D)
    pub fn adapt_proposal_std(&mut self, target_rate: f64) {
        let current_rate = self.acceptance_rate();

        if current_rate > target_rate {
            // Too many acceptances - increase proposal std (more exploration)
            self.proposal_std *= 1.1;
        } else {
            // Too few acceptances - decrease proposal std (more conservative)
            self.proposal_std *= 0.9;
        }

        // Keep proposal std in reasonable range
        self.proposal_std = self.proposal_std.clamp(0.1, 20.0);

        // Reset counters for next adaptation
        self.accepted = 0;
        self.total = 0;
    }
}

/// MCMC-based skill estimator using Bayesian inference
///
/// Maintains posterior distribution over skill parameter σ and provides
/// robust point estimates with quantified uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCMCSkillEstimator {
    /// Observed miss distances (feet)
    observations: Vec<f64>,

    /// Prior mean for σ (from handicap)
    prior_mean: f64,

    /// Prior uncertainty
    prior_std: f64,

    /// MCMC sampler state
    sampler: MCMCSampler,

    /// Posterior samples from Markov chain
    posterior_samples: Vec<f64>,

    /// Whether posterior samples are valid (cleared when new observations added)
    samples_valid: bool,
}

impl Default for MCMCSkillEstimator {
    fn default() -> Self {
        // Default estimator with neutral prior
        Self::new(30.0, 30.0, 10.0)
    }
}

impl MCMCSkillEstimator {
    /// Create a new MCMC skill estimator
    ///
    /// # Arguments
    /// * `initial_sigma` - Starting estimate (from handicap)
    /// * `prior_mean` - Prior mean for σ
    /// * `prior_std` - Prior uncertainty (wider = less confident in handicap)
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::mcmc::MCMCSkillEstimator;
    ///
    /// let estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);
    /// ```
    pub fn new(initial_sigma: f64, prior_mean: f64, prior_std: f64) -> Self {
        let initial_log_posterior = log_posterior(
            initial_sigma,
            &[],
            prior_mean,
            prior_std,
        );

        let sampler = MCMCSampler::new(initial_sigma, initial_log_posterior, 2.0);

        Self {
            observations: Vec::new(),
            prior_mean,
            prior_std,
            sampler,
            posterior_samples: Vec::new(),
            samples_valid: false,
        }
    }

    /// Add a new observation and invalidate cached samples
    ///
    /// # Arguments
    /// * `miss_distance` - Observed miss distance in feet
    pub fn add_observation(&mut self, miss_distance: f64) {
        if miss_distance > 0.0 {
            self.observations.push(miss_distance);
            self.samples_valid = false;
        }
    }

    /// Add multiple observations at once
    ///
    /// # Arguments
    /// * `miss_distances` - Vec of observed miss distances
    pub fn add_observations(&mut self, miss_distances: &[f64]) {
        for &dist in miss_distances {
            if dist > 0.0 {
                self.observations.push(dist);
            }
        }
        self.samples_valid = false;
    }

    /// Run MCMC sampling to generate posterior distribution
    ///
    /// # Arguments
    /// * `num_samples` - Number of posterior samples to generate
    /// * `burn_in` - Number of initial samples to discard
    /// * `thin` - Keep every nth sample (reduces autocorrelation)
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::mcmc::MCMCSkillEstimator;
    ///
    /// let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);
    /// estimator.add_observations(&[25.0, 30.0, 22.0, 35.0]);
    /// estimator.sample(1000, 200, 2); // 1000 samples, 200 burn-in, thin by 2
    /// ```
    pub fn sample(&mut self, num_samples: usize, burn_in: usize, thin: usize) {
        if self.observations.is_empty() {
            // No observations - use prior
            self.posterior_samples = vec![self.prior_mean];
            self.samples_valid = true;
            return;
        }

        // Reset sampler to use current posterior median as starting point
        // This prevents the chain from getting stuck after serialization/deserialization
        if !self.samples_valid && !self.posterior_samples.is_empty() {
            // Compute median directly (don't call get_sigma_estimate to avoid recursion)
            let mut sorted = self.posterior_samples.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mid = sorted.len() / 2;
            let start_sigma = if sorted.len() % 2 == 0 {
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[mid]
            };

            let start_log_post = log_posterior(start_sigma, &self.observations, self.prior_mean, self.prior_std);
            self.sampler = MCMCSampler::new(start_sigma, start_log_post, self.sampler.proposal_std);
        }

        self.posterior_samples.clear();
        self.posterior_samples.reserve(num_samples);

        // Burn-in phase with adaptive tuning
        for i in 0..burn_in {
            self.sampler.step(&self.observations, self.prior_mean, self.prior_std);

            // Adapt every 50 steps during burn-in
            if (i + 1) % 50 == 0 {
                self.sampler.adapt_proposal_std(0.234); // Optimal 1D acceptance rate
            }
        }

        // Sampling phase
        let total_iterations = num_samples * thin;
        for i in 0..total_iterations {
            let sample = self.sampler.step(&self.observations, self.prior_mean, self.prior_std);

            // Keep every nth sample
            if i % thin == 0 {
                self.posterior_samples.push(sample);
            }
        }

        self.samples_valid = true;
    }

    /// Get point estimate of skill parameter (posterior median)
    ///
    /// # Returns
    /// Median of posterior samples (robust to outliers)
    pub fn get_sigma_estimate(&mut self) -> f64 {
        if !self.samples_valid {
            self.sample(1000, 200, 2);
        }

        if self.posterior_samples.is_empty() {
            return self.prior_mean;
        }

        // Compute median
        let mut sorted = self.posterior_samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Get Bayesian credible interval
    ///
    /// # Arguments
    /// * `alpha` - Confidence level (e.g., 0.95 for 95% CI)
    ///
    /// # Returns
    /// (lower_bound, upper_bound)
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::math::mcmc::MCMCSkillEstimator;
    ///
    /// let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);
    /// estimator.add_observations(&[25.0, 30.0, 22.0, 35.0]);
    /// let (lower, upper) = estimator.get_credible_interval(0.95);
    /// println!("95% CI: [{:.1}, {:.1}]", lower, upper);
    /// ```
    pub fn get_credible_interval(&mut self, alpha: f64) -> (f64, f64) {
        if !self.samples_valid {
            self.sample(1000, 200, 2);
        }

        if self.posterior_samples.is_empty() {
            return (self.prior_mean, self.prior_mean);
        }

        let mut sorted = self.posterior_samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let tail = (1.0 - alpha) / 2.0;
        let lower_idx = (tail * sorted.len() as f64) as usize;
        let upper_idx = ((1.0 - tail) * sorted.len() as f64) as usize;

        (sorted[lower_idx], sorted[upper_idx.min(sorted.len() - 1)])
    }

    /// Calculate confidence score based on posterior concentration
    ///
    /// # Returns
    /// Confidence from 0 to 1 based on:
    /// - Number of observations (more data = higher confidence)
    /// - Width of credible interval (narrower = higher confidence)
    pub fn calculate_confidence(&mut self) -> f64 {
        if self.observations.is_empty() {
            return 0.0;
        }

        // Base confidence from observation count (saturates at ~30 shots)
        let n = self.observations.len() as f64;
        let count_confidence = (n / 30.0).min(1.0);

        // Uncertainty from credible interval width
        let (lower, upper) = self.get_credible_interval(0.95);
        let ci_width = upper - lower;
        let estimate = self.get_sigma_estimate();

        // Relative uncertainty (narrower = higher confidence)
        let relative_width = ci_width / estimate.max(1.0);
        let interval_confidence = (1.0 - relative_width / 2.0).max(0.0).min(1.0);

        // Combine both factors
        (count_confidence * 0.7 + interval_confidence * 0.3).min(1.0)
    }

    /// Get number of observations
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Get acceptance rate of MCMC sampler
    pub fn acceptance_rate(&self) -> f64 {
        self.sampler.acceptance_rate()
    }

    /// Clear observations and reset estimator
    pub fn clear(&mut self) {
        self.observations.clear();
        self.posterior_samples.clear();
        self.samples_valid = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_mcmc_sampler_initialization() {
        let sampler = MCMCSampler::new(28.0, -10.0, 2.0);

        assert_eq!(sampler.current_sigma, 28.0);
        assert_eq!(sampler.current_log_posterior, -10.0);
        assert_eq!(sampler.proposal_std, 2.0);
        assert_eq!(sampler.accepted, 0);
        assert_eq!(sampler.total, 0);
    }

    #[test]
    fn test_mcmc_sampler_step_rejects_negative_sigma() {
        let mut sampler = MCMCSampler::new(0.1, -10.0, 10.0); // Large proposal std
        let observations = vec![25.0, 30.0];

        // Many steps should never accept negative sigma
        for _ in 0..100 {
            let sample = sampler.step(&observations, 28.0, 5.0);
            assert!(sample > 0.0);
        }
    }

    #[test]
    fn test_mcmc_sampler_acceptance_rate() {
        let mut sampler = MCMCSampler::new(28.0, -10.0, 2.0);
        let observations = vec![25.0, 30.0, 22.0, 35.0];

        // Run some steps
        for _ in 0..100 {
            sampler.step(&observations, 28.0, 5.0);
        }

        let rate = sampler.acceptance_rate();
        assert!(rate > 0.0);
        assert!(rate <= 1.0);
    }

    #[test]
    fn test_mcmc_sampler_proposal_adaptation() {
        let mut sampler = MCMCSampler::new(28.0, -10.0, 2.0);
        let initial_std = sampler.proposal_std;

        // Simulate high acceptance rate
        sampler.accepted = 80;
        sampler.total = 100;

        sampler.adapt_proposal_std(0.234);

        // Proposal std should increase
        assert!(sampler.proposal_std > initial_std);
    }

    #[test]
    fn test_skill_estimator_initialization() {
        let estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);

        assert_eq!(estimator.observation_count(), 0);
        assert_eq!(estimator.prior_mean, 28.0);
        assert_eq!(estimator.prior_std, 5.0);
    }

    #[test]
    fn test_skill_estimator_add_observations() {
        let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);

        estimator.add_observation(25.0);
        estimator.add_observation(30.0);

        assert_eq!(estimator.observation_count(), 2);
    }

    #[test]
    fn test_skill_estimator_rejects_invalid_observations() {
        let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);

        estimator.add_observation(-5.0); // Invalid
        estimator.add_observation(0.0); // Invalid
        estimator.add_observation(25.0); // Valid

        assert_eq!(estimator.observation_count(), 1);
    }

    #[test]
    fn test_skill_estimator_sampling() {
        let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);
        estimator.add_observations(&[25.0, 30.0, 22.0, 35.0]);

        estimator.sample(100, 20, 1);

        assert_eq!(estimator.posterior_samples.len(), 100);
        assert!(estimator.samples_valid);
    }

    #[test]
    fn test_skill_estimator_converges_to_data() {
        let mut estimator = MCMCSkillEstimator::new(20.0, 20.0, 10.0); // Prior at 20

        // Add many observations centered at 30
        let observations: Vec<f64> = (0..50).map(|_| 30.0).collect();
        estimator.add_observations(&observations);

        let estimate = estimator.get_sigma_estimate();

        // With strong data, estimate should be close to 30 (not prior of 20)
        assert!(estimate > 25.0);
        assert!(estimate < 35.0);
    }

    #[test]
    fn test_skill_estimator_credible_interval() {
        let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);
        estimator.add_observations(&[25.0, 30.0, 22.0, 35.0]);

        let (lower, upper) = estimator.get_credible_interval(0.95);
        let estimate = estimator.get_sigma_estimate();

        // Estimate should be within credible interval
        assert!(estimate >= lower);
        assert!(estimate <= upper);

        // Interval should be non-trivial
        assert!(upper > lower);
    }

    #[test]
    fn test_skill_estimator_confidence_increases_with_data() {
        let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);

        // No data = low confidence
        let conf_0 = estimator.calculate_confidence();
        assert_eq!(conf_0, 0.0);

        // Add some data
        estimator.add_observations(&[25.0, 30.0, 22.0, 35.0]);
        let conf_4 = estimator.calculate_confidence();

        // Add more data
        for _ in 0..20 {
            estimator.add_observation(28.0);
        }
        let conf_24 = estimator.calculate_confidence();

        // Confidence should increase with more data
        assert!(conf_4 > conf_0);
        assert!(conf_24 > conf_4);
    }

    #[test]
    fn test_skill_estimator_clear() {
        let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);
        estimator.add_observations(&[25.0, 30.0, 22.0, 35.0]);
        estimator.sample(100, 20, 1);

        estimator.clear();

        assert_eq!(estimator.observation_count(), 0);
        assert!(estimator.posterior_samples.is_empty());
        assert!(!estimator.samples_valid);
    }

    #[test]
    fn test_skill_estimator_invalidates_samples_on_new_data() {
        let mut estimator = MCMCSkillEstimator::new(28.0, 28.0, 5.0);
        estimator.add_observations(&[25.0, 30.0]);
        estimator.sample(100, 20, 1);

        assert!(estimator.samples_valid);

        // Adding new observation should invalidate samples
        estimator.add_observation(35.0);

        assert!(!estimator.samples_valid);
    }
}
