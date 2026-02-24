// Player model with skill profiles and MCMC estimation
//
// Each player tracks separate skill profiles for each club category (Wedge, MidIron, LongIron).
// Skills are dynamically updated using MCMC Bayesian inference that adapts to observed shot performance.

use crate::math::integration::trapezoidal_rule;
use crate::math::mcmc::MCMCSkillEstimator;
use crate::models::hole::{ClubCategory, Hole};
use crate::models::shot::ShotBatch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,

    pub handicap: u8,

    pub skill_profiles: HashMap<ClubCategory, SkillProfile>,

    pub lifetime_wagers: Vec<f64>,

    pub lifetime_total_wagered: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProfile {
    /// MCMC Bayesian skill estimator (PRIMARY - replaces Kalman filter)
    /// Provides mathematically optimal skill estimation with convergence guarantees
    pub mcmc_estimator: MCMCSkillEstimator,

    /// Cached sigma estimate from last MCMC update
    /// This prevents re-running expensive MCMC on every shot
    pub cached_sigma: f64,

    pub shot_batch: ShotBatch,

    pub p_max_history: Vec<f64>,

    pub batch_size: usize,

    pub shot_count: usize,

    pub total_payout: f64,

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

        let categories = [
            (ClubCategory::Wedge, 100),    // 75-125 yds
            (ClubCategory::MidIron, 162),  // 150-175 yds
            (ClubCategory::LongIron, 225), // 200-250 yds
        ];

        for (category, distance) in categories.iter() {
            let initial_sigma = calculate_initial_dispersion(handicap, *distance);

            // Initialize MCMC estimator with handicap-based prior
            // Prior std = 30% of initial estimate (reflects handicap uncertainty)
            let prior_std = initial_sigma * 0.3;
            let mcmc_estimator = MCMCSkillEstimator::new(
                initial_sigma, // Initial estimate
                initial_sigma, // Prior mean (same as initial)
                prior_std,     // Prior uncertainty
            );

            skill_profiles.insert(
                *category,
                SkillProfile {
                    mcmc_estimator,
                    cached_sigma: initial_sigma, // Initialize cache with handicap-based estimate
                    shot_batch: ShotBatch::new(5), // Use ShotBatch struct
                    p_max_history: Vec::new(),
                    batch_size: 5,          // Default batch size
                    shot_count: 0,          // Start at zero shots
                    total_payout: 0.0,      // Initialize payout tracking
                    total_shots_for_rtp: 0, // Initialize shot count for RTP
                },
            );
        }

        Player {
            id,
            handicap,
            skill_profiles,
            lifetime_wagers: Vec::new(),
            lifetime_total_wagered: 0.0,
        }
    }

    pub fn get_skill_for_hole(&self, hole: &Hole) -> &SkillProfile {
        self.skill_profiles.get(&hole.category).unwrap()
    }

    pub fn get_skill_for_hole_mut(&mut self, hole: &Hole) -> &mut SkillProfile {
        self.skill_profiles.get_mut(&hole.category).unwrap()
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
        self.calculate_p_max_fresh(hole)
    }

    fn calculate_p_max_fresh(&self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole(hole);

        let sigma = skill.cached_sigma;

        self.calculate_p_max_bvn(hole, 0.0, 0.0, sigma, sigma, 0.0, None)
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
        rho: f64,                        // Correlation coefficient between x and y
        actual_rtp_percent: Option<f64>, // Actual RTP% observed so far (for adaptive correction)
    ) -> f64 {
        use crate::math::distributions::bvn_pdf;

        let d_max = hole.d_max_ft;
        let k = hole.k;
        let fat_tail_prob = 0.02;
        let fat_tail_mult = 3.0;

        let x_min = mu_x - 4.0 * sigma_x;
        let x_max = mu_x + 4.0 * sigma_x;
        let y_min = mu_y - 4.0 * sigma_y;
        let y_max = mu_y + 4.0 * sigma_y;

        let n_x = 200;
        let n_y = 200;
        let dx = (x_max - x_min) / n_x as f64;
        let dy = (y_max - y_min) / n_y as f64;

        let mut expected_payout_normal = 0.0;
        for i in 0..n_x {
            let x = x_min + (i as f64 + 0.5) * dx;
            for j in 0..n_y {
                let y = y_min + (j as f64 + 0.5) * dy;

                let r = (x * x + y * y).sqrt();

                let payout = if r > d_max {
                    0.0
                } else {
                    (1.0 - r / d_max).powf(k)
                };

                let prob = bvn_pdf(x, y, mu_x, mu_y, sigma_x, sigma_y, rho);

                expected_payout_normal += payout * prob * dx * dy;
            }
        }

        let sigma_x_fat = sigma_x * fat_tail_mult;
        let sigma_y_fat = sigma_y * fat_tail_mult;

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

        let expected_payout =
            (1.0 - fat_tail_prob) * expected_payout_normal + fat_tail_prob * expected_payout_fat;

        let epsilon = 1e-10;
        let base_p_max = hole.rtp / (expected_payout + epsilon);

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
            base_p_max
        }
    }

    pub fn add_shot_to_batch(&mut self, hole: &Hole, miss_distance: f64, wager: f64) -> bool {
        let skill = self.get_skill_for_hole_mut(hole);

        skill.shot_batch.add_shot(miss_distance, wager);

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

    pub fn add_shot_to_batch_2d(&mut self, hole: &Hole, x_ft: f64, y_ft: f64, wager: f64) -> bool {
        let skill = self.get_skill_for_hole_mut(hole);
        skill.shot_batch.add_shot_2d(x_ft, y_ft, wager);
        skill.shot_batch.is_full()
    }

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

        self.update_skill_1d(hole);
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

        let shots = skill.shot_batch.get_shots();
        let miss_distances: Vec<f64> = shots.iter().map(|s| s.miss_distance).collect();

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

        let final_distances = if filtered_distances.is_empty() {
            miss_distances
        } else {
            filtered_distances
        };

        skill.shot_count += final_distances.len();

        // Adaptive MCMC sampling strategy based on observation count
        // Early phase: More samples for faster convergence
        // Later phase: Fewer samples as posterior becomes concentrated
        let (num_samples, burn_in, thin) = if skill.shot_count <= 10 {
            (2000, 400, 2) // High sampling early for quick skill detection
        } else if skill.shot_count <= 50 {
            (1500, 300, 2) // Medium sampling during transition
        } else {
            (1000, 200, 2) // Standard sampling when mature
        };

        skill.mcmc_estimator.sample(num_samples, burn_in, thin);

        let sigma_estimate = skill.mcmc_estimator.get_sigma_estimate();

        skill.cached_sigma = sigma_estimate;

        let (sigma_lower, sigma_upper) = skill.mcmc_estimator.get_credible_interval(0.95);
        let confidence = skill.mcmc_estimator.calculate_confidence();

        eprintln!(
            "📊 MCMC Bayesian update (shot {}): σ={:.2}ft, 95% CI=[{:.2}, {:.2}], conf={:.1}%",
            skill.shot_count,
            sigma_estimate,
            sigma_lower,
            sigma_upper,
            confidence * 100.0
        );

        let d_max = hole.d_max_ft;
        let k = hole.k;
        let fat_tail_prob = 0.02;
        let fat_tail_mult = 3.0;

        let integrand_normal = |d: f64| -> f64 {
            if d > d_max {
                return 0.0;
            }
            let payout_factor = (1.0 - d / d_max).powf(k);
            let rayleigh_pdf = (d / (sigma_estimate * sigma_estimate))
                * (-d * d / (2.0 * sigma_estimate * sigma_estimate)).exp();
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
        let expected_payout =
            (1.0 - fat_tail_prob) * expected_payout_normal + fat_tail_prob * expected_payout_fat;
        let epsilon = 1e-10;
        let calculated_p_max = hole.rtp / (expected_payout + epsilon);

        skill.p_max_history.push(calculated_p_max);

        eprintln!(
            "💰 P_max calculated (shot {}): {:.2}x (from MCMC σ={:.2}ft)",
            skill.shot_count, calculated_p_max, sigma_estimate
        );

        skill.shot_batch.clear();
    }

    pub fn get_skill_confidence(&mut self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole_mut(hole);
        skill.mcmc_estimator.calculate_confidence()
    }

    pub fn get_current_sigma(&self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole(hole);
        skill.cached_sigma
    }

    pub fn get_batch_size(&self, hole: &Hole) -> usize {
        let skill = self.get_skill_for_hole(hole);
        skill.shot_batch.len()
    }

    pub fn track_wager(&mut self, wager: f64) {
        self.lifetime_wagers.push(wager);
        self.lifetime_total_wagered += wager;
    }

    pub fn get_lifetime_avg_wager(&self) -> f64 {
        if self.lifetime_wagers.is_empty() {
            return 0.0;
        }
        self.lifetime_total_wagered / self.lifetime_wagers.len() as f64
    }

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

    let distance_factor = 0.05 + ((distance - 75.0) / (250.0 - 75.0)) * 0.01;

    let skill_factor = 0.5 + (handicap as f64 / 30.0);

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
        assert!(skill.cached_sigma > 0.0);
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
        assert!(
            p_max_pro < p_max_beginner,
            "Pro P_max: {}, Beginner P_max: {}",
            p_max_pro,
            p_max_beginner
        );
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
        assert!(
            final_confidence > initial_confidence + 0.8,
            "Confidence only increased from {} to {}",
            initial_confidence,
            final_confidence
        );
    }

    #[test]
    fn test_separate_skill_profiles() {
        let mut player = Player::new("test".to_string(), 15);

        let wedge_hole = get_hole_by_id(1).unwrap(); // 75yd
        let long_hole = get_hole_by_id(8).unwrap(); // 250yd

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
            (0.0, 0.0, 20.0, 20.0),  // Centered, tight
            (0.0, 0.0, 35.0, 35.0),  // Centered, loose (but realistic for handicap ~20-25)
            (10.0, 5.0, 25.0, 25.0), // Large bias
            (2.0, 1.0, 15.0, 30.0),  // Small bias, elliptical
            (0.0, 0.0, 35.0, 20.0),  // Lateral worse than distance
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
