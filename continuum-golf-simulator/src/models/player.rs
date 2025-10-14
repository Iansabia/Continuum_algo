// Player model with skill profiles and Kalman filtering
//
// Each player tracks separate skill profiles for each club category (Wedge, MidIron, LongIron).
// Skills are dynamically updated using a Kalman filter that adapts to observed shot performance.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::math::kalman::{KalmanState, debias_rayleigh_measurement, weighted_average_measurement, measurement_variance};
use crate::math::integration::trapezoidal_rule;
use crate::models::hole::{Hole, ClubCategory};

/// A player with dynamic skill tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique player identifier
    pub id: String,
    /// Golf handicap (0-30, lower is better)
    pub handicap: u8,
    /// Skill profiles for each club category
    pub skill_profiles: HashMap<ClubCategory, SkillProfile>,
}

/// Skill profile for a specific club category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProfile {
    /// Kalman filter for adaptive skill tracking
    pub kalman_filter: KalmanState,
    /// History of P_max values (for analysis)
    pub p_max_history: Vec<f64>,
    /// Current batch of shots (for batched Kalman updates)
    pub shot_batch: Vec<ShotRecord>,
    /// Maximum batch size before triggering update
    pub batch_size: usize,
}

/// Record of a single shot for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotRecord {
    /// Miss distance in feet
    pub miss_distance: f64,
    /// Wager amount in dollars
    pub wager: f64,
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

            skill_profiles.insert(*category, SkillProfile {
                kalman_filter,
                p_max_history: Vec::new(),
                shot_batch: Vec::new(),
                batch_size: 5, // Default batch size
            });
        }

        Player {
            id,
            handicap,
            skill_profiles,
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

    /// Calculate P_max for a given hole using numerical integration
    ///
    /// P_max is the maximum payout multiplier that maintains the house's RTP.
    ///
    /// # Formula
    /// P_max = RTP / ∫[0, d_max] (1 - d/d_max)^k * PDF(d | σ) dd
    ///
    /// Where PDF(d | σ) is the Rayleigh distribution:
    /// PDF(d) = (d/σ²) * exp(-d²/(2σ²))
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
        let skill = self.get_skill_for_hole(hole);
        let sigma = skill.kalman_filter.estimate;

        // Calculate expected payout using numerical integration
        let d_max = hole.d_max_ft;
        let k = hole.k;

        // Define integrand: payout_function(d) * rayleigh_pdf(d, sigma)
        let integrand = |d: f64| -> f64 {
            if d > d_max {
                return 0.0;
            }

            // Payout function: (1 - d/d_max)^k
            let payout_factor = (1.0 - d / d_max).powf(k);

            // Rayleigh PDF: (d/σ²) * exp(-d²/(2σ²))
            let rayleigh_pdf = (d / (sigma * sigma)) * (-d * d / (2.0 * sigma * sigma)).exp();

            payout_factor * rayleigh_pdf
        };

        // Integrate from 0 to d_max (use higher bound for numerical stability)
        let upper_bound = (d_max * 1.5).max(sigma * 5.0);
        let n_subdivisions = 2000; // High accuracy

        let expected_payout = trapezoidal_rule(integrand, 0.0, upper_bound, n_subdivisions);

        // P_max = RTP / expected_payout
        // Add small epsilon to prevent division by zero
        let epsilon = 1e-10;
        hole.rtp / (expected_payout + epsilon)
    }

    /// Add a shot to the batch for a specific hole
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

        skill.shot_batch.push(ShotRecord {
            miss_distance,
            wager,
        });

        skill.shot_batch.len() >= skill.batch_size
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

        if skill.shot_batch.is_empty() {
            return false;
        }

        let total_wagers: f64 = skill.shot_batch.iter().map(|s| s.wager).sum();
        let avg_wager = total_wagers / skill.shot_batch.len() as f64;

        wager >= 10.0 * avg_wager
    }

    /// Update skill profile using Kalman filter with current batch
    ///
    /// This performs a wager-weighted update of the player's skill estimate.
    ///
    /// # Arguments
    /// * `hole` - The hole that was played
    /// * `p_max` - The P_max value used for these shots
    ///
    /// # Process
    /// 1. Calculate wager-weighted average miss distance
    /// 2. Debias for Rayleigh distribution
    /// 3. Calculate batch variance for measurement noise
    /// 4. Update Kalman filter
    /// 5. Store P_max in history
    /// 6. Clear shot batch
    pub fn update_skill(&mut self, hole: &Hole, p_max: f64) {
        let skill = self.get_skill_for_hole_mut(hole);

        if skill.shot_batch.is_empty() {
            return;
        }

        // Extract miss distances and wagers
        let measurements: Vec<(f64, f64)> = skill.shot_batch.iter()
            .map(|s| (s.miss_distance, s.wager))
            .collect();

        // Calculate wager-weighted average
        let weighted_avg = weighted_average_measurement(&measurements);

        // Debias for Rayleigh distribution
        let unbiased_measurement = debias_rayleigh_measurement(weighted_avg);

        // Calculate batch variance for dynamic measurement noise
        let miss_distances: Vec<f64> = skill.shot_batch.iter()
            .map(|s| s.miss_distance)
            .collect();
        let batch_variance = measurement_variance(&miss_distances);

        // Measurement noise (R) is based on batch variance
        // Higher variance = less trustworthy batch
        let measurement_noise = batch_variance.max(50.0); // Minimum R = 50

        // Kalman filter update
        skill.kalman_filter.predict();
        skill.kalman_filter.update(unbiased_measurement, measurement_noise);

        // Store P_max in history
        skill.p_max_history.push(p_max);

        // Clear batch
        skill.shot_batch.clear();
    }

    /// Get current skill confidence for a hole (0-100%)
    pub fn get_skill_confidence(&self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole(hole);
        skill.kalman_filter.calculate_confidence()
    }

    /// Get current sigma estimate for a hole
    pub fn get_current_sigma(&self, hole: &Hole) -> f64 {
        let skill = self.get_skill_for_hole(hole);
        skill.kalman_filter.estimate
    }

    /// Get number of shots in current batch for a hole
    pub fn get_batch_size(&self, hole: &Hole) -> usize {
        let skill = self.get_skill_for_hole(hole);
        skill.shot_batch.len()
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
}
