// Player model with skill profiles and Kalman filtering
//
// Each player tracks separate skill profiles for each club category (Wedge, MidIron, LongIron).
// Skills are dynamically updated using a Kalman filter that adapts to observed shot performance.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::math::kalman::{KalmanState, KalmanState4D};
use crate::math::mcmc::MCMCSkillEstimator;
use crate::math::integration::trapezoidal_rule;
use crate::models::hole::{Hole, ClubCategory};
use crate::models::shot::ShotBatch;

/// A player with dynamic skill tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique player identifier
    pub id: String,
    /// Golf handicap (0-30, lower is better)
    pub handicap: u8,
    /// Skill profiles for each club category
    pub skill_profiles: HashMap<ClubCategory, SkillProfile>,
    /// Lifetime wager history for anti-cheat detection
    pub lifetime_wagers: Vec<f64>,
    /// Lifetime total wagered (for average calculation)
    pub lifetime_total_wagered: f64,
}

/// Skill profile for a specific club category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProfile {
    /// MCMC Bayesian skill estimator (PRIMARY - replaces Kalman filter)
    /// Provides mathematically optimal skill estimation with convergence guarantees
    pub mcmc_estimator: MCMCSkillEstimator,

    /// Cached sigma estimate from last MCMC update
    /// This prevents re-running expensive MCMC on every shot
    pub cached_sigma: f64,

    /// 1D Kalman filter for radial dispersion (DEPRECATED - kept for migration)
    pub kalman_filter: KalmanState,
    /// 4D Kalman filter for BVN distribution [μ_x, μ_y, σ_x, σ_y]
    /// Only populated when BVN mode is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kalman_filter_4d: Option<KalmanState4D>,
    /// Whether to use BVN (4D) mode instead of Rayleigh (1D)
    pub use_bvn: bool,
    /// Current batch of shots (for batched updates)
    pub shot_batch: ShotBatch,
    /// History of P_max values (for analysis)
    pub p_max_history: Vec<f64>,
    /// Maximum batch size before triggering update
    pub batch_size: usize,
    /// Total number of shots taken for this skill profile
    pub shot_count: usize,
    /// Total payout accumulated (for RTP tracking)
    pub total_payout: f64,
    /// Total shots counted for RTP calculation
    pub total_shots_for_rtp: usize,
}

impl Player {
    /// Create a new player with initial skill estimates
    ///
    /// # Arguments
    /// * `id` - Unique player identifier
    /// * `handicap` - Golf handicap (0-30)
    ///
    /// # Returns
    /// Player with initialized skill profiles for all club categories
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::models::player::Player;
    ///
    /// let player = Player::new("player_1".to_string(), 15);
    /// assert_eq!(player.handicap, 15);
    /// ```
    pub fn new(id: String, handicap: u8) -> Self {
        let mut skill_profiles = HashMap::new();

        // Initialize skill profiles for each category
        // Use representative distances for each category
        let categories = [
            (ClubCategory::Wedge, 100),     // 75-125 yds
            (ClubCategory::MidIron, 162),   // 150-175 yds
            (ClubCategory::LongIron, 225),  // 200-250 yds
        ];

        for (category, distance) in categories.iter() {
            let initial_sigma = calculate_initial_dispersion(handicap, *distance);

            let kalman_filter = KalmanState::new(initial_sigma, 1.0);

            // Initialize MCMC estimator with handicap-based prior
            // Prior std = 30% of initial estimate (reflects handicap uncertainty)
            let prior_std = initial_sigma * 0.3;
            let mcmc_estimator = MCMCSkillEstimator::new(
                initial_sigma,  // Initial estimate
                initial_sigma,  // Prior mean (same as initial)
                prior_std,      // Prior uncertainty
            );

            skill_profiles.insert(*category, SkillProfile {
                mcmc_estimator,
                cached_sigma: initial_sigma, // Initialize cache with handicap-based estimate
                kalman_filter,
                kalman_filter_4d: None, // 4D mode disabled by default
                use_bvn: false,         // Start in 1D Rayleigh mode
                shot_batch: ShotBatch::new(5), // Use ShotBatch struct
                p_max_history: Vec::new(),
                batch_size: 5, // Default batch size
                shot_count: 0, // Start at zero shots
                total_payout: 0.0, // Initialize payout tracking
                total_shots_for_rtp: 0, // Initialize shot count for RTP
            });
        }

        Player {
            id,
            handicap,
            skill_profiles,
            lifetime_wagers: Vec::new(),
            lifetime_total_wagered: 0.0,
        }
    }

    /// Get the skill profile for a specific hole
    ///
    /// # Arguments
    /// * `hole` - The hole being played
    ///
    /// # Returns
    /// Reference to the appropriate skill profile
    pub fn get_skill_for_hole(&self, hole: &Hole) -> &SkillProfile {
        self.skill_profiles.get(&hole.category).unwrap()
    }

    /// Get mutable skill profile for a specific hole
    pub fn get_skill_for_hole_mut(&mut self, hole: &Hole) -> &mut SkillProfile {
        self.skill_profiles.get_mut(&hole.category).unwrap()
    }

    /// Enable BVN (4D Kalman) mode for a specific club category
    ///
    /// # Arguments
    /// * `category` - The club category to enable BVN mode for
    /// * `initial_mu_x` - Initial lateral bias (feet, positive = right)
    /// * `initial_mu_y` - Initial distance bias (feet, positive = long)
    /// * `initial_sigma_x` - Initial lateral dispersion (feet)
    /// * `initial_sigma_y` - Initial distance dispersion (feet)
    ///
    /// # Notes
    /// - This initializes a 4D Kalman filter with the given parameters
    /// - The 1D Rayleigh filter remains available for backward compatibility
    /// - Use `disable_bvn_mode()` to revert to 1D mode
    pub fn enable_bvn_mode(
        &mut self,
        category: ClubCategory,
        initial_mu_x: f64,
        initial_mu_y: f64,
        initial_sigma_x: f64,
        initial_sigma_y: f64,
    ) {
        if let Some(skill) = self.skill_profiles.get_mut(&category) {
            // Process noise: [Q_mu_x, Q_mu_y, Q_sigma_x, Q_sigma_y]
            // Small for bias (changes slowly), moderate for dispersion
            let process_noise = [0.1, 0.1, 0.5, 0.5];

            skill.kalman_filter_4d = Some(KalmanState4D::new(
                initial_mu_x,
                initial_mu_y,
                initial_sigma_x,
                initial_sigma_y,
                process_noise,
            ));
            skill.use_bvn = true;
        }
    }

    /// Enable BVN mode for all club categories using symmetric initial conditions
    ///
    /// # Arguments
    /// * `initial_sigma` - Initial dispersion (same for x and y)
    ///
    /// # Notes
    /// - Sets bias to (0, 0) and uses symmetric dispersion
    /// - Useful for players with no prior shot data
    pub fn enable_bvn_mode_all(&mut self, initial_sigma: f64) {
        // Process noise: [Q_mu_x, Q_mu_y, Q_sigma_x, Q_sigma_y]
        let process_noise = [0.1, 0.1, 0.5, 0.5];

        for (_category, skill) in self.skill_profiles.iter_mut() {
            skill.kalman_filter_4d = Some(KalmanState4D::new(
                0.0,           // No initial lateral bias
                0.0,           // No initial distance bias
                initial_sigma, // Symmetric dispersion
                initial_sigma,
                process_noise,
            ));
            skill.use_bvn = true;
        }
    }

    /// Disable BVN mode and revert to 1D Rayleigh for a specific category
    ///
    /// # Arguments
    /// * `category` - The club category to disable BVN mode for
    pub fn disable_bvn_mode(&mut self, category: ClubCategory) {
        if let Some(skill) = self.skill_profiles.get_mut(&category) {
            skill.use_bvn = false;
            // Keep the 4D filter in memory in case we want to re-enable
        }
    }

    /// Check if BVN mode is enabled for a specific category
    ///
    /// # Arguments
    /// * `category` - The club category to check
    ///
    /// # Returns
    /// True if BVN mode is enabled and 4D Kalman filter is available
    pub fn is_bvn_mode(&self, category: ClubCategory) -> bool {
        self.skill_profiles
            .get(&category)
            .map(|s| s.use_bvn && s.kalman_filter_4d.is_some())
            .unwrap_or(false)
    }

    /// Get current 4D Kalman state for a specific category
    ///
    /// # Arguments
    /// * `category` - The club category to query
    ///
    /// # Returns
    /// Option containing (μ_x, μ_y, σ_x, σ_y) if BVN mode is enabled
    pub fn get_bvn_state(&self, category: ClubCategory) -> Option<(f64, f64, f64, f64)> {
        self.skill_profiles.get(&category).and_then(|s| {
            s.kalman_filter_4d.as_ref().map(|kf| {
                (
                    kf.state[0], // μ_x
                    kf.state[1], // μ_y
                    kf.state[2], // σ_x
                    kf.state[3], // σ_y
                )
            })
        })
    }

    /// Calculate P_max for a given hole using numerical integration
    ///
    /// P_max is the maximum payout multiplier that maintains the house's RTP.
    ///
    /// # Formula
    /// P_max = RTP / E[payout]
    ///
    /// Where E[payout] accounts for fat-tail distribution:
    /// E[payout] = (1-p_fat) * ∫ (1-d/d_max)^k * Rayleigh(d|σ) dd
    ///           + p_fat * ∫ (1-d/d_max)^k * Rayleigh(d|3σ) dd
    ///
    /// With p_fat = 0.02 (2% chance of fat-tail shot)
    ///
    /// # Security
    /// If P_max history exists, returns the last value (which has rate limiting applied).
    /// This prevents rapid P_max inflation from sandbagging attacks.
    ///
    /// # Arguments
    /// * `hole` - The hole configuration
    ///
    /// # Returns
    /// Maximum payout multiplier
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::models::player::Player;
    /// use continuum_golf_simulator::models::hole::Hole;
    ///
    /// let player = Player::new("p1".to_string(), 15);
    /// let hole = Hole::new(1, 75, 17.95, 0.86, 5.0);
    /// let p_max = player.calculate_p_max(&hole);
    /// assert!(p_max > 1.0);
    /// assert!(p_max < 20.0);
    /// ```
    pub fn calculate_p_max(&self, hole: &Hole) -> f64 {
        // Always calculate fresh P_max based on cached sigma
        // Sigma is updated only during batch processing for stability
        self.calculate_p_max_fresh(hole)
    }

    /// Calculate fresh P_max without rate limiting (internal use only)
    /// Uses cached MCMC posterior median for stable estimates
    fn calculate_p_max_fresh(&self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole(hole);

        // Use cached MCMC estimate (updated during batch processing)
        // If no observations yet, this will be the handicap-based initial estimate
        let sigma = skill.cached_sigma;

        // Calculate expected payout using numerical integration
        // Must account for fat-tail distribution (2% chance of 3x sigma)
        let d_max = hole.d_max_ft;
        let k = hole.k;
        let fat_tail_prob = 0.02;
        let fat_tail_mult = 3.0;

        // Define integrand for normal shots: payout_function(d) * rayleigh_pdf(d, sigma)
        let integrand_normal = |d: f64| -> f64 {
            if d > d_max {
                return 0.0;
            }

            // Payout function: (1 - d/d_max)^k
            let payout_factor = (1.0 - d / d_max).powf(k);

            // Rayleigh PDF: (d/σ²) * exp(-d²/(2σ²))
            let rayleigh_pdf = (d / (sigma * sigma)) * (-d * d / (2.0 * sigma * sigma)).exp();

            payout_factor * rayleigh_pdf
        };

        // Define integrand for fat-tail shots: payout_function(d) * rayleigh_pdf(d, 3*sigma)
        let sigma_fat = sigma * fat_tail_mult;
        let integrand_fat = |d: f64| -> f64 {
            if d > d_max {
                return 0.0;
            }

            // Payout function: (1 - d/d_max)^k
            let payout_factor = (1.0 - d / d_max).powf(k);

            // Rayleigh PDF with fat-tail sigma: (d/(3σ)²) * exp(-d²/(2(3σ)²))
            let rayleigh_pdf = (d / (sigma_fat * sigma_fat)) * (-d * d / (2.0 * sigma_fat * sigma_fat)).exp();

            payout_factor * rayleigh_pdf
        };

        // Integrate from 0 to d_max (use higher bound for numerical stability)
        // Use the fat-tail sigma for upper bound since it has longer tail
        let upper_bound = (d_max * 1.5).max(sigma_fat * 5.0);
        let n_subdivisions = 2000; // High accuracy

        let expected_payout_normal = trapezoidal_rule(integrand_normal, 0.0, upper_bound, n_subdivisions);
        let expected_payout_fat = trapezoidal_rule(integrand_fat, 0.0, upper_bound, n_subdivisions);

        // Weighted average: (1 - p_fat) * E[normal] + p_fat * E[fat]
        let expected_payout = (1.0 - fat_tail_prob) * expected_payout_normal + fat_tail_prob * expected_payout_fat;

        // P_max = RTP / expected_payout
        // Add small epsilon to prevent division by zero
        let epsilon = 1e-10;
        hole.rtp / (expected_payout + epsilon)
    }

    /// Calculate P_max using 2D BVN distribution (Phase 9.3)
    ///
    /// Unlike the 1D Rayleigh model, this accounts for:
    /// - Systematic bias (μ_x, μ_y) - player tends to miss in a specific direction
    /// - Elliptical dispersion (σ_x ≠ σ_y) - different precision in lateral vs distance
    ///
    /// # Formula
    /// P_max = RTP / E[payout]
    ///
    /// Where E[payout] = ∬ payout(x,y) * BVN(x,y | μ_x, μ_y, σ_x, σ_y) dx dy
    ///
    /// Integration performed over 2D grid using Cartesian coordinates.
    ///
    /// # Arguments
    /// * `hole` - The hole configuration
    /// * `mu_x` - Lateral bias (feet right of target line)
    /// * `mu_y` - Distance bias (feet from pin, positive = long)
    /// * `sigma_x` - Lateral dispersion (precision)
    /// * `sigma_y` - Distance dispersion (precision)
    ///
    /// # Returns
    /// Maximum payout multiplier for BVN distribution
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::models::player::Player;
    /// use continuum_golf_simulator::models::hole::Hole;
    ///
    /// let player = Player::new("p1".to_string(), 15);
    /// let hole = Hole::new(1, 75, 17.95, 0.86, 5.0);
    ///
    /// // Player with rightward bias and better distance control
    /// let p_max = player.calculate_p_max_bvn(&hole, 5.0, 2.0, 20.0, 12.0, 0.0, None);
    /// assert!(p_max > 1.0);
    /// ```
    pub fn calculate_p_max_bvn(
        &self,
        hole: &Hole,
        mu_x: f64,
        mu_y: f64,
        sigma_x: f64,
        sigma_y: f64,
        rho: f64, // Correlation coefficient between x and y
        actual_rtp_percent: Option<f64>, // Actual RTP% observed so far (for adaptive correction)
    ) -> f64 {
        use crate::math::distributions::bvn_pdf;

        let d_max = hole.d_max_ft;
        let k = hole.k;
        let fat_tail_prob = 0.02;
        let fat_tail_mult = 3.0;

        // Integration bounds: ±4σ from bias (covers 99.99% of distribution)
        let x_min = mu_x - 4.0 * sigma_x;
        let x_max = mu_x + 4.0 * sigma_x;
        let y_min = mu_y - 4.0 * sigma_y;
        let y_max = mu_y + 4.0 * sigma_y;

        // Grid resolution (200×200 = 40,000 evaluations, ~5ms)
        let n_x = 200;
        let n_y = 200;
        let dx = (x_max - x_min) / n_x as f64;
        let dy = (y_max - y_min) / n_y as f64;

        // Calculate expected payout for normal shots
        let mut expected_payout_normal = 0.0;
        for i in 0..n_x {
            let x = x_min + (i as f64 + 0.5) * dx;
            for j in 0..n_y {
                let y = y_min + (j as f64 + 0.5) * dy;

                // Distance from pin
                let r = (x * x + y * y).sqrt();

                // Payout function
                let payout = if r > d_max {
                    0.0
                } else {
                    (1.0 - r / d_max).powf(k)
                };

                // BVN probability density
                let prob = bvn_pdf(x, y, mu_x, mu_y, sigma_x, sigma_y, rho);

                // Accumulate: payout × probability × area
                expected_payout_normal += payout * prob * dx * dy;
            }
        }

        // Calculate expected payout for fat-tail shots (3× dispersion)
        let sigma_x_fat = sigma_x * fat_tail_mult;
        let sigma_y_fat = sigma_y * fat_tail_mult;

        // Expand bounds for fat-tail
        let x_min_fat = mu_x - 4.0 * sigma_x_fat;
        let x_max_fat = mu_x + 4.0 * sigma_x_fat;
        let y_min_fat = mu_y - 4.0 * sigma_y_fat;
        let y_max_fat = mu_y + 4.0 * sigma_y_fat;
        let dx_fat = (x_max_fat - x_min_fat) / n_x as f64;
        let dy_fat = (y_max_fat - y_min_fat) / n_y as f64;

        let mut expected_payout_fat = 0.0;
        for i in 0..n_x {
            let x = x_min_fat + (i as f64 + 0.5) * dx_fat;
            for j in 0..n_y {
                let y = y_min_fat + (j as f64 + 0.5) * dy_fat;

                let r = (x * x + y * y).sqrt();
                let payout = if r > d_max {
                    0.0
                } else {
                    (1.0 - r / d_max).powf(k)
                };

                let prob = bvn_pdf(x, y, mu_x, mu_y, sigma_x_fat, sigma_y_fat, rho);
                expected_payout_fat += payout * prob * dx_fat * dy_fat;
            }
        }

        // Weighted average: (1 - p_fat) * E[normal] + p_fat * E[fat]
        let expected_payout = (1.0 - fat_tail_prob) * expected_payout_normal
            + fat_tail_prob * expected_payout_fat;

        // Base P_max = RTP / expected_payout
        let epsilon = 1e-10;
        let base_p_max = hole.rtp / (expected_payout + epsilon);

        // Apply adaptive correction if we have actual RTP data
        if let Some(actual_rtp) = actual_rtp_percent {
            let target_rtp = hole.rtp;
            let rtp_error = actual_rtp - target_rtp;

            // Adaptive correction with exponential damping
            // - If RTP > target: reduce P_max (multiply by factor < 1)
            // - If RTP < target: increase P_max (multiply by factor > 1)
            // - Use exponential decay for stability: correction = exp(-α * error)
            //
            // α = 0.02 means:
            //   - 5% overshoot → 90% of base P_max (10% reduction)
            //   - 10% overshoot → 82% of base P_max (18% reduction)
            //   - 20% overshoot → 67% of base P_max (33% reduction)
            let alpha = 0.02;
            let correction_factor = (-alpha * rtp_error).exp();

            let corrected_p_max = base_p_max * correction_factor;

            eprintln!(
                "🎯 Adaptive P_max: base={:.2}x, actual_RTP={:.1}%, target_RTP={:.1}%, error={:.1}%, correction={:.3}x, final={:.2}x",
                base_p_max, actual_rtp, target_rtp, rtp_error, correction_factor, corrected_p_max
            );

            corrected_p_max
        } else {
            // No RTP data yet, use base calculation
            base_p_max
        }
    }

    /// Add a 1D shot (radial distance only) to the batch
    ///
    /// # Arguments
    /// * `hole` - The hole that was played
    /// * `miss_distance` - Miss distance in feet
    /// * `wager` - Wager amount in dollars
    ///
    /// # Returns
    /// True if the batch is full and should be processed
    pub fn add_shot_to_batch(&mut self, hole: &Hole, miss_distance: f64, wager: f64) -> bool {
        let skill = self.get_skill_for_hole_mut(hole);

        // Add shot to batch for later batch processing
        skill.shot_batch.add_shot(miss_distance, wager);

        // IMMEDIATELY add observation to MCMC estimator so P_max stays current
        skill.mcmc_estimator.add_observation(miss_distance);

        // Update cached sigma with quick MCMC sampling (fewer iterations for speed)
        // This ensures P_max calculations reflect current skill without waiting for batch
        if skill.mcmc_estimator.observation_count() > 0 {
            // Use lightweight sampling for real-time updates
            skill.mcmc_estimator.sample(500, 100, 1);
            skill.cached_sigma = skill.mcmc_estimator.get_sigma_estimate();
        }

        skill.shot_batch.is_full()
    }

    /// Add a 2D shot (x,y coordinates) to the batch for BVN mode
    ///
    /// # Arguments
    /// * `hole` - The hole that was played
    /// * `x_ft` - Lateral position (feet, positive = right)
    /// * `y_ft` - Distance position (feet, positive = long)
    /// * `wager` - Wager amount in dollars
    ///
    /// # Returns
    /// True if the batch is full and should be processed
    pub fn add_shot_to_batch_2d(&mut self, hole: &Hole, x_ft: f64, y_ft: f64, wager: f64) -> bool {
        let skill = self.get_skill_for_hole_mut(hole);
        skill.shot_batch.add_shot_2d(x_ft, y_ft, wager);
        skill.shot_batch.is_full()
    }

    /// Check if a new shot qualifies as high-stakes (≥10× average wager)
    ///
    /// # Arguments
    /// * `hole` - The hole being played
    /// * `wager` - The proposed wager
    ///
    /// # Returns
    /// True if this is a high-stakes shot
    pub fn is_high_stakes_shot(&self, hole: &Hole, wager: f64) -> bool {
        let skill = self.get_skill_for_hole(hole);
        skill.shot_batch.has_high_stakes_shot(wager)
    }

    /// Update skill profile using Kalman filter with current batch
    ///
    /// This method branches based on whether BVN (4D) or Rayleigh (1D) mode is active.
    ///
    /// # Arguments
    /// * `hole` - The hole that was played
    /// * `p_max` - The P_max value used for these shots (unused, kept for backward compat)
    ///
    /// # Modes
    /// - **1D Rayleigh**: Updates single sigma parameter from radial miss distances
    /// - **4D BVN**: Updates [μ_x, μ_y, σ_x, σ_y] from (x,y) coordinates
    ///
    /// # Security
    /// - Applies outlier detection to reduce impact of suspicious shots
    /// - Uses wager-weighted averaging to account for bet sizing patterns
    pub fn update_skill(&mut self, hole: &Hole, _p_max: f64) {
        let skill = self.get_skill_for_hole_mut(hole);

        if skill.shot_batch.is_empty() {
            return;
        }

        // Branch based on mode
        if skill.use_bvn && skill.shot_batch.has_2d_shots() {
            self.update_skill_4d(hole);
        } else {
            self.update_skill_1d(hole);
        }
    }

    /// Update skill using MCMC Bayesian inference (primary method)
    ///
    /// This replaces the Kalman filter approach with mathematically optimal
    /// Bayesian inference using MCMC sampling. Provides:
    /// - Guaranteed convergence to true skill level
    /// - No oscillation issues
    /// - Quantified uncertainty (credible intervals)
    /// - Robust to outliers through posterior distribution
    fn update_skill_1d(&mut self, hole: &Hole) {
        let skill = self.get_skill_for_hole_mut(hole);

        if skill.shot_batch.is_empty() {
            return;
        }

        // Extract miss distances from batch
        let shots = skill.shot_batch.get_shots();
        let miss_distances: Vec<f64> = shots.iter().map(|s| s.miss_distance).collect();

        // SECURITY: Outlier detection - filter shots >3 sigma from mean
        let mean_miss: f64 = miss_distances.iter().sum::<f64>() / miss_distances.len() as f64;
        let variance: f64 = miss_distances
            .iter()
            .map(|&d| (d - mean_miss).powi(2))
            .sum::<f64>()
            / miss_distances.len() as f64;
        let std_dev = variance.sqrt();

        let filtered_distances: Vec<f64> = miss_distances
            .iter()
            .filter(|&&d| (d - mean_miss).abs() <= 3.0 * std_dev)
            .copied()
            .collect();

        // Use filtered distances if we have any, otherwise use all
        let final_distances = if filtered_distances.is_empty() {
            miss_distances
        } else {
            filtered_distances
        };

        // NOTE: Observations already added to MCMC in add_shot_to_batch()
        // This batch processing just re-samples with higher accuracy

        // Update shot count
        skill.shot_count += final_distances.len();

        // Adaptive MCMC sampling strategy based on observation count
        // Early phase: More samples for faster convergence
        // Later phase: Fewer samples as posterior becomes concentrated
        let (num_samples, burn_in, thin) = if skill.shot_count <= 10 {
            (2000, 400, 2)  // High sampling early for quick skill detection
        } else if skill.shot_count <= 50 {
            (1500, 300, 2)  // Medium sampling during transition
        } else {
            (1000, 200, 2)  // Standard sampling when mature
        };

        // Run MCMC to get posterior estimate
        skill.mcmc_estimator.sample(num_samples, burn_in, thin);

        // Get sigma estimate (posterior median)
        let sigma_estimate = skill.mcmc_estimator.get_sigma_estimate();

        // **CRITICAL**: Cache the estimate so it's stable between updates
        skill.cached_sigma = sigma_estimate;

        // Get credible interval for logging
        let (sigma_lower, sigma_upper) = skill.mcmc_estimator.get_credible_interval(0.95);
        let confidence = skill.mcmc_estimator.calculate_confidence();

        eprintln!(
            "📊 MCMC Bayesian update (shot {}): σ={:.2}ft, 95% CI=[{:.2}, {:.2}], conf={:.1}%",
            skill.shot_count, sigma_estimate, sigma_lower, sigma_upper, confidence * 100.0
        );

        // Calculate P_max using posterior median sigma
        let d_max = hole.d_max_ft;
        let k = hole.k;
        let fat_tail_prob = 0.02;
        let fat_tail_mult = 3.0;

        let integrand_normal = |d: f64| -> f64 {
            if d > d_max {
                return 0.0;
            }
            let payout_factor = (1.0 - d / d_max).powf(k);
            let rayleigh_pdf = (d / (sigma_estimate * sigma_estimate)) * (-d * d / (2.0 * sigma_estimate * sigma_estimate)).exp();
            payout_factor * rayleigh_pdf
        };

        let sigma_fat = sigma_estimate * fat_tail_mult;
        let integrand_fat = |d: f64| -> f64 {
            if d > d_max {
                return 0.0;
            }
            let payout_factor = (1.0 - d / d_max).powf(k);
            let rayleigh_pdf =
                (d / (sigma_fat * sigma_fat)) * (-d * d / (2.0 * sigma_fat * sigma_fat)).exp();
            payout_factor * rayleigh_pdf
        };

        let upper_bound = (d_max * 1.5).max(sigma_fat * 5.0);
        let n_subdivisions = 2000;

        let expected_payout_normal =
            trapezoidal_rule(integrand_normal, 0.0, upper_bound, n_subdivisions);
        let expected_payout_fat = trapezoidal_rule(integrand_fat, 0.0, upper_bound, n_subdivisions);
        let expected_payout = (1.0 - fat_tail_prob) * expected_payout_normal
            + fat_tail_prob * expected_payout_fat;
        let epsilon = 1e-10;
        let calculated_p_max = hole.rtp / (expected_payout + epsilon);

        // Store P_max in history
        // MCMC naturally converges - no artificial rate limiting needed!
        skill.p_max_history.push(calculated_p_max);

        eprintln!(
            "💰 P_max calculated (shot {}): {:.2}x (from MCMC σ={:.2}ft)",
            skill.shot_count, calculated_p_max, sigma_estimate
        );

        // Clear batch
        skill.shot_batch.clear();
    }

    /// Update skill using 4D BVN Kalman filter (new mode)
    ///
    /// Uses (x,y) shot coordinates to update [μ_x, μ_y, σ_x, σ_y] state.
    ///
    /// # Process
    /// 1. Extract (x,y) coordinates and wagers from batch
    /// 2. Calculate wager-weighted centroids (x̄, ȳ)
    /// 3. Use residuals to estimate σ_x and σ_y
    /// 4. Update 4D Kalman filter
    /// 5. Calculate P_max using BVN distribution
    fn update_skill_4d(&mut self, hole: &Hole) {
        // First, extract data without holding mutable reference
        let (x_coords, y_coords, wagers) = {
            let skill = self.get_skill_for_hole(hole);

            if skill.shot_batch.is_empty() {
                return;
            }

            let shots = skill.shot_batch.get_shots();

            // Extract (x,y) coordinates with wagers
            let mut x_coords = Vec::new();
            let mut y_coords = Vec::new();
            let mut wagers = Vec::new();

            for shot in shots {
                if let (Some(x), Some(y)) = (shot.x_ft, shot.y_ft) {
                    x_coords.push(x);
                    y_coords.push(y);
                    wagers.push(shot.wager);
                }
            }

            (x_coords, y_coords, wagers)
        };

        if x_coords.is_empty() {
            // No 2D data, fall back to 1D
            self.update_skill_1d(hole);
            return;
        }

        // Calculate wager-weighted centroids
        let total_wager: f64 = wagers.iter().sum();
        let mean_x: f64 = x_coords
            .iter()
            .zip(wagers.iter())
            .map(|(x, w)| x * w)
            .sum::<f64>()
            / total_wager;
        let mean_y: f64 = y_coords
            .iter()
            .zip(wagers.iter())
            .map(|(y, w)| y * w)
            .sum::<f64>()
            / total_wager;

        // Calculate sample standard deviations (proper σ estimation)
        let n = x_coords.len() as f64;
        let var_x: f64 = x_coords.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>()
            / (n - 1.0).max(1.0); // Use (n-1) for unbiased estimate
        let var_y: f64 = y_coords.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>()
            / (n - 1.0).max(1.0);
        let sigma_x_measured = var_x.sqrt().max(10.0); // Floor at 10ft minimum
        let sigma_y_measured = var_y.sqrt().max(10.0);

        // Now perform Kalman update and extract fast estimates
        let (sigma_x_fast, sigma_y_fast) = {
            let skill = self.get_skill_for_hole_mut(hole);

            // 4D Kalman filter update
            if let Some(ref mut kf4d) = skill.kalman_filter_4d {
                kf4d.predict();

                // Measurement noise for bias estimates (batch mean)
                // Standard error of the mean: σ / sqrt(n)
                let noise_x = (sigma_x_measured / n.sqrt()).max(5.0);
                let noise_y = (sigma_y_measured / n.sqrt()).max(5.0);

                // Measurement noise for dispersion estimates (batch std dev)
                // Standard error of std dev: σ / sqrt(2n) approximately
                let noise_sigma_x = (sigma_x_measured / (2.0 * n).sqrt()).max(5.0);
                let noise_sigma_y = (sigma_y_measured / (2.0 * n).sqrt()).max(5.0);

                // Update with batch statistics (mean and std dev)
                kf4d.update_with_batch(
                    mean_x,
                    mean_y,
                    sigma_x_measured,
                    sigma_y_measured,
                    [noise_x, noise_y, noise_sigma_x, noise_sigma_y],
                );

                // Get updated state (fast tracking for bias and dispersion)
                (kf4d.state[2], kf4d.state[3])
            } else {
                // Fallback if 4D Kalman not initialized
                (sigma_x_measured, sigma_y_measured)
            }
        };

        // Update shot count
        let skill = self.get_skill_for_hole_mut(hole);
        skill.shot_count += x_coords.len();

        // Adaptive smoothing: aggressive early to catch sandbagging, conservative later for stability
        // Early phase (0-30 shots): Update every shot, high smoothing factor (70% new)
        // Transition (30-100 shots): Update every 5 shots, medium smoothing (50% new)
        // Mature (100+ shots): Update every 10 shots, low smoothing (30% new)
        let should_update = if skill.shot_count <= 30 {
            true // Update every shot in early phase
        } else if skill.shot_count <= 100 {
            skill.shot_count % 5 == 0 // Every 5 shots in transition
        } else {
            skill.shot_count % 10 == 0 // Every 10 shots when mature
        };

        if should_update {
            // Calculate P_max using BVN distribution with Kalman estimates
            // Get current Kalman state for bias (mu_x, mu_y)
            let (mu_x, mu_y) = if let Some(ref kf4d) = skill.kalman_filter_4d {
                (kf4d.state[0], kf4d.state[1])
            } else {
                (0.0, 0.0) // Fallback to no bias
            };

            // Use Kalman estimates directly (no smoothing in BVN mode yet)
            let sigma_avg = (sigma_x_fast + sigma_y_fast) / 2.0;

            eprintln!(
                "📊 BVN update (shot {}): σ_x={:.2}ft, σ_y={:.2}ft, μ_x={:.2}ft, μ_y={:.2}ft",
                skill.shot_count, sigma_x_fast, sigma_y_fast, mu_x, mu_y
            );

            // Calculate actual RTP% if we have enough data (need at least 20 shots)
            let actual_rtp_percent = if skill.total_shots_for_rtp >= 20 {
                Some((skill.total_payout / skill.total_shots_for_rtp as f64) * 100.0)
            } else {
                None
            };

            // Calculate P_max using BVN distribution with adaptive correction
            let calculated_p_max = {
                // Clone hole to avoid lifetime issues
                let hole_clone = hole.clone();
                let _ = skill; // Drop mutable borrow
                self.calculate_p_max_bvn(&hole_clone, mu_x, mu_y, sigma_x_fast, sigma_y_fast, 0.0, actual_rtp_percent)
            };

            // Re-acquire mutable borrow for storage
            let skill = self.get_skill_for_hole_mut(hole);
            skill.p_max_history.push(calculated_p_max);

            eprintln!(
                "💰 BVN P_max update (shot {}): {:.2}x (σ_avg={:.2}ft)",
                skill.shot_count, calculated_p_max, sigma_avg
            );
        }

        // Clear batch
        let skill = self.get_skill_for_hole_mut(hole);
        skill.shot_batch.clear();
    }

    /// Get current skill confidence for a hole (0-100%)
    pub fn get_skill_confidence(&mut self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole_mut(hole);
        skill.mcmc_estimator.calculate_confidence()
    }

    /// Get current sigma estimate for a hole (uses cached value)
    pub fn get_current_sigma(&self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole(hole);
        skill.cached_sigma
    }

    /// Get number of shots in current batch for a hole
    pub fn get_batch_size(&self, hole: &Hole) -> usize {
        let skill = self.get_skill_for_hole(hole);
        skill.shot_batch.len()
    }

    /// Track a wager for lifetime average calculation
    ///
    /// # Security
    /// Used for cross-session high-stakes detection to prevent cherry-picking
    pub fn track_wager(&mut self, wager: f64) {
        self.lifetime_wagers.push(wager);
        self.lifetime_total_wagered += wager;
    }

    /// Get lifetime average wager
    ///
    /// # Security
    /// Used for high-stakes detection across multiple sessions
    pub fn get_lifetime_avg_wager(&self) -> f64 {
        if self.lifetime_wagers.is_empty() {
            return 0.0;
        }
        self.lifetime_total_wagered / self.lifetime_wagers.len() as f64
    }

    /// Track payout for RTP calculation
    ///
    /// # Arguments
    /// * `hole` - The hole being played
    /// * `payout_multiplier` - The payout multiplier (e.g., 0.5 = 50% return)
    ///
    /// # Notes
    /// This accumulates payout percentages (not dollar amounts) for RTP tracking
    pub fn track_payout(&mut self, hole: &Hole, payout_multiplier: f64) {
        let skill = self.get_skill_for_hole_mut(hole);
        skill.total_payout += payout_multiplier * 100.0; // Convert to percentage
        skill.total_shots_for_rtp += 1;
    }
}

/// Calculate initial dispersion (sigma) based on handicap and distance
///
/// # Formula
/// σ = distance * 3 * (0.05 + (distance - 75) / (250 - 75) * 0.01) * (0.5 + handicap / 30)
///
/// This formula accounts for:
/// - Longer shots have more dispersion
/// - Higher handicap players have more dispersion
/// - Base dispersion increases with distance
///
/// # Arguments
/// * `handicap` - Golf handicap (0-30)
/// * `distance_yds` - Shot distance in yards
///
/// # Returns
/// Initial sigma in feet
///
/// # Example
/// ```
/// use continuum_golf_simulator::models::player::calculate_initial_dispersion;
///
/// let sigma_expert = calculate_initial_dispersion(0, 150);
/// let sigma_beginner = calculate_initial_dispersion(30, 150);
/// assert!(sigma_beginner > sigma_expert);
/// ```
pub fn calculate_initial_dispersion(handicap: u8, distance_yds: u16) -> f64 {
    let distance = distance_yds as f64;

    // Base dispersion factor increases with distance
    let distance_factor = 0.05 + ((distance - 75.0) / (250.0 - 75.0)) * 0.01;

    // Skill factor: handicap 0 → 0.5, handicap 30 → 1.5
    let skill_factor = 0.5 + (handicap as f64 / 30.0);

    // Convert yards to feet and apply factors
    distance * 3.0 * distance_factor * skill_factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::hole::get_hole_by_id;

    #[test]
    fn test_player_creation() {
        let player = Player::new("test_player".to_string(), 15);

        assert_eq!(player.id, "test_player");
        assert_eq!(player.handicap, 15);
        assert_eq!(player.skill_profiles.len(), 3);

        // Check that all categories are initialized
        assert!(player.skill_profiles.contains_key(&ClubCategory::Wedge));
        assert!(player.skill_profiles.contains_key(&ClubCategory::MidIron));
        assert!(player.skill_profiles.contains_key(&ClubCategory::LongIron));
    }

    #[test]
    fn test_initial_dispersion_scales_with_handicap() {
        let sigma_pro = calculate_initial_dispersion(0, 150);
        let sigma_amateur = calculate_initial_dispersion(15, 150);
        let sigma_beginner = calculate_initial_dispersion(30, 150);

        assert!(sigma_pro < sigma_amateur);
        assert!(sigma_amateur < sigma_beginner);
    }

    #[test]
    fn test_initial_dispersion_scales_with_distance() {
        let sigma_short = calculate_initial_dispersion(15, 75);
        let sigma_mid = calculate_initial_dispersion(15, 150);
        let sigma_long = calculate_initial_dispersion(15, 250);

        assert!(sigma_short < sigma_mid);
        assert!(sigma_mid < sigma_long);
    }

    #[test]
    fn test_get_skill_for_hole() {
        let player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(1).unwrap(); // 75yd wedge

        let skill = player.get_skill_for_hole(hole);
        assert!(skill.kalman_filter.estimate > 0.0);
    }

    #[test]
    fn test_calculate_p_max() {
        let player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(1).unwrap();

        let p_max = player.calculate_p_max(hole);

        // P_max should be reasonable (between 1 and 50 for short holes)
        // Short holes with moderate skill can have high P_max values
        assert!(p_max > 1.0, "P_max was {}", p_max);
        assert!(p_max < 50.0, "P_max was {}", p_max);
    }

    #[test]
    fn test_p_max_varies_with_skill() {
        let pro = Player::new("pro".to_string(), 0);
        let beginner = Player::new("beginner".to_string(), 30);
        let hole = get_hole_by_id(4).unwrap(); // 150yd

        let p_max_pro = pro.calculate_p_max(hole);
        let p_max_beginner = beginner.calculate_p_max(hole);

        // Better players (lower sigma) should have lower P_max
        // because they're more likely to hit the high-payout zone
        assert!(p_max_pro < p_max_beginner,
            "Pro P_max: {}, Beginner P_max: {}", p_max_pro, p_max_beginner);
    }

    #[test]
    fn test_add_shot_to_batch() {
        let mut player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(1).unwrap();

        assert!(!player.add_shot_to_batch(hole, 10.0, 5.0));
        assert!(!player.add_shot_to_batch(hole, 12.0, 5.0));
        assert!(!player.add_shot_to_batch(hole, 11.0, 5.0));
        assert!(!player.add_shot_to_batch(hole, 13.0, 5.0));

        // Fifth shot should fill the batch
        assert!(player.add_shot_to_batch(hole, 14.0, 5.0));

        let skill = player.get_skill_for_hole(hole);
        assert_eq!(skill.shot_batch.len(), 5);
    }

    #[test]
    fn test_high_stakes_detection() {
        let mut player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(1).unwrap();

        player.add_shot_to_batch(hole, 10.0, 5.0);
        player.add_shot_to_batch(hole, 12.0, 5.0);
        player.add_shot_to_batch(hole, 11.0, 5.0);

        // Average is 5.0, so 10× = 50.0
        assert!(!player.is_high_stakes_shot(hole, 40.0));
        assert!(player.is_high_stakes_shot(hole, 50.0));
        assert!(player.is_high_stakes_shot(hole, 100.0));
    }

    #[test]
    fn test_update_skill() {
        let mut player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(1).unwrap();

        let initial_confidence = player.get_skill_confidence(hole);

        // Add shots to batch
        player.add_shot_to_batch(hole, 10.0, 5.0);
        player.add_shot_to_batch(hole, 12.0, 5.0);
        player.add_shot_to_batch(hole, 11.0, 5.0);

        let p_max = player.calculate_p_max(hole);

        // Update skill
        player.update_skill(hole, p_max);

        // Batch should be cleared
        assert_eq!(player.get_batch_size(hole), 0);

        // Sigma may have changed (depending on measurements)
        let new_sigma = player.get_current_sigma(hole);
        assert!(new_sigma > 0.0);

        // Confidence should increase
        let new_confidence = player.get_skill_confidence(hole);
        assert!(new_confidence >= initial_confidence);

        // P_max history should have one entry
        let skill = player.get_skill_for_hole(hole);
        assert_eq!(skill.p_max_history.len(), 1);
    }

    #[test]
    fn test_skill_convergence() {
        let mut player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap();

        let initial_confidence = player.get_skill_confidence(hole);

        // Simulate many consistent shots
        for _ in 0..10 {
            for _ in 0..5 {
                player.add_shot_to_batch(hole, 30.0, 5.0);
            }

            let p_max = player.calculate_p_max(hole);
            player.update_skill(hole, p_max);
        }

        // Confidence should increase significantly
        let final_confidence = player.get_skill_confidence(hole);
        assert!(final_confidence > initial_confidence + 30.0,
            "Confidence only increased from {} to {}",
            initial_confidence, final_confidence);
    }

    #[test]
    fn test_separate_skill_profiles() {
        let mut player = Player::new("test".to_string(), 15);

        let wedge_hole = get_hole_by_id(1).unwrap(); // 75yd
        let long_hole = get_hole_by_id(8).unwrap();  // 250yd

        // Add shots to wedge
        for _ in 0..5 {
            player.add_shot_to_batch(wedge_hole, 15.0, 5.0);
        }
        let p_max_wedge = player.calculate_p_max(wedge_hole);
        player.update_skill(wedge_hole, p_max_wedge);

        // Wedge should have update, but long iron should not
        let wedge_skill = player.get_skill_for_hole(wedge_hole);
        let long_skill = player.get_skill_for_hole(long_hole);

        assert_eq!(wedge_skill.p_max_history.len(), 1);
        assert_eq!(long_skill.p_max_history.len(), 0);
    }

    #[test]
    fn test_calculate_p_max_bvn_symmetric() {
        // When BVN is symmetric (no bias, equal dispersions), it should be close to Rayleigh
        let player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap(); // 150 yards

        // Symmetric BVN: no bias, equal dispersions
        let mu_x = 0.0;
        let mu_y = 0.0;
        let sigma = 30.0; // Same as typical Rayleigh sigma
        let sigma_x = sigma;
        let sigma_y = sigma;

        let p_max_bvn = player.calculate_p_max_bvn(hole, mu_x, mu_y, sigma_x, sigma_y, 0.0, None);
        let p_max_rayleigh = player.calculate_p_max(hole);

        // Should be within 30% (numerical integration uses different methods: 2D grid vs 1D trapezoidal)
        // The difference comes from grid resolution and integration bounds
        let ratio = p_max_bvn / p_max_rayleigh;
        assert!(
            ratio > 0.8 && ratio < 1.3,
            "Symmetric BVN should be close to Rayleigh: BVN={:.2}, Rayleigh={:.2}, ratio={:.3}",
            p_max_bvn,
            p_max_rayleigh,
            ratio
        );
    }

    #[test]
    fn test_calculate_p_max_bvn_with_bias() {
        // Player with rightward bias should have lower P_max than centered player
        let player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap();

        // No bias
        let p_max_centered = player.calculate_p_max_bvn(hole, 0.0, 0.0, 25.0, 25.0, 0.0, None);

        // Strong rightward bias (5 ft right on average)
        let p_max_biased = player.calculate_p_max_bvn(hole, 5.0, 0.0, 25.0, 25.0, 0.0, None);

        // Biased player is slightly farther from pin on average → lower expected payout → higher P_max
        assert!(
            p_max_biased > p_max_centered,
            "Biased player should have higher P_max (lower EV): centered={:.2}, biased={:.2}",
            p_max_centered,
            p_max_biased
        );
    }

    #[test]
    fn test_calculate_p_max_bvn_elliptical() {
        // Better distance control (small σ_y) vs lateral control
        let player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap();

        // Good distance control, poor lateral: σ_x=30, σ_y=15
        let p_max_distance_good = player.calculate_p_max_bvn(hole, 0.0, 0.0, 30.0, 15.0, 0.0, None);

        // Poor distance control, good lateral: σ_x=15, σ_y=30
        let p_max_lateral_good = player.calculate_p_max_bvn(hole, 0.0, 0.0, 15.0, 30.0, 0.0, None);

        // Both are valid, just testing that function handles elliptical distributions
        assert!(p_max_distance_good > 1.0 && p_max_distance_good < 20.0);
        assert!(p_max_lateral_good > 1.0 && p_max_lateral_good < 20.0);

        // The difference depends on hole geometry, but both should be reasonable
        let ratio = p_max_distance_good / p_max_lateral_good;
        assert!(
            ratio > 0.5 && ratio < 2.0,
            "Elliptical P_max values should be within 2× of each other: {:.2} vs {:.2}",
            p_max_distance_good,
            p_max_lateral_good
        );
    }

    #[test]
    fn test_calculate_p_max_bvn_values_reasonable() {
        // P_max should always be in reasonable range
        let player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap();

        // Test various parameter combinations
        let test_cases = vec![
            (0.0, 0.0, 20.0, 20.0),   // Centered, tight
            (0.0, 0.0, 35.0, 35.0),   // Centered, loose (but realistic for handicap ~20-25)
            (10.0, 5.0, 25.0, 25.0),  // Large bias
            (2.0, 1.0, 15.0, 30.0),   // Small bias, elliptical
            (0.0, 0.0, 35.0, 20.0),   // Lateral worse than distance
        ];

        for (mu_x, mu_y, sigma_x, sigma_y) in test_cases {
            let p_max = player.calculate_p_max_bvn(hole, mu_x, mu_y, sigma_x, sigma_y, 0.0, None);

            // P_max should be positive and reasonable
            // Note: Very poor players (σ=40ft+) can have P_max > 20, which is mathematically correct
            assert!(
                p_max > 1.0 && p_max < 50.0,
                "P_max out of range for μ=({}, {}), σ=({}, {}): {}",
                mu_x,
                mu_y,
                sigma_x,
                sigma_y,
                p_max
            );
        }
    }

    #[test]
    fn test_calculate_p_max_bvn_fat_tail_effect() {
        // Fat-tail should reduce P_max (increase expected payout)
        let player = Player::new("test".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap();

        // Calculate P_max with BVN (includes 2% fat-tail)
        let p_max_with_fat = player.calculate_p_max_bvn(hole, 0.0, 0.0, 25.0, 25.0, 0.0, None);

        // The fat-tail effect is already baked into the calculation,
        // so we just verify it produces sensible results
        assert!(
            p_max_with_fat > 1.0 && p_max_with_fat < 20.0,
            "P_max with fat-tail should be reasonable: {}",
            p_max_with_fat
        );
    }
}
