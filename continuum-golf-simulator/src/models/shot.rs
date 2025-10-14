// Shot outcome modeling
//
// Models the outcome of a single shot, including:
// - Miss distance (from Rayleigh distribution)
// - Fat-tail events (2% chance of 3× worse dispersion)
// - Payout calculation
// - Metadata for analysis

use serde::{Deserialize, Serialize};
use crate::math::distributions::{rayleigh_random, fat_tail_shot};

/// Result of a single shot attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotOutcome {
    /// Miss distance from target in feet
    pub miss_distance_ft: f64,
    /// Payout multiplier (e.g., 5.0 = 5× return)
    pub multiplier: f64,
    /// Total payout amount in dollars
    pub payout: f64,
    /// Wager amount in dollars
    pub wager: f64,
    /// Which hole was played (1-8)
    pub hole_id: u8,
    /// Whether this was a fat-tail event (extreme mishit)
    pub is_fat_tail: bool,
}

impl ShotOutcome {
    /// Create a new shot outcome
    ///
    /// # Arguments
    /// * `miss_distance_ft` - Miss distance in feet
    /// * `multiplier` - Payout multiplier
    /// * `wager` - Wager amount
    /// * `hole_id` - Hole number (1-8)
    /// * `is_fat_tail` - Whether this was a fat-tail event
    ///
    /// # Returns
    /// New ShotOutcome with calculated payout
    pub fn new(
        miss_distance_ft: f64,
        multiplier: f64,
        wager: f64,
        hole_id: u8,
        is_fat_tail: bool,
    ) -> Self {
        let payout = multiplier * wager;
        ShotOutcome {
            miss_distance_ft,
            multiplier,
            payout,
            wager,
            hole_id,
            is_fat_tail,
        }
    }

    /// Calculate net gain/loss for this shot
    pub fn net_result(&self) -> f64 {
        self.payout - self.wager
    }

    /// Check if this was a winning shot (multiplier > 1.0)
    pub fn is_win(&self) -> bool {
        self.multiplier >= 1.0
    }

    /// Check if this was an ace (landed at center, d=0)
    pub fn is_ace(&self) -> bool {
        self.miss_distance_ft < 0.1 // Within 1 inch
    }
}

/// Simulate a shot with optional fat-tail behavior
///
/// # Arguments
/// * `sigma` - Player's skill parameter (Rayleigh σ)
/// * `fat_tail_prob` - Probability of fat-tail event (typically 0.02 = 2%)
/// * `fat_tail_mult` - Multiplier for fat-tail dispersion (typically 3.0)
///
/// # Returns
/// Tuple of (miss_distance_ft, is_fat_tail)
///
/// # Example
/// ```
/// use continuum_golf_simulator::models::shot::simulate_shot;
///
/// let (miss, is_fat_tail) = simulate_shot(30.0, 0.02, 3.0);
/// assert!(miss >= 0.0);
/// ```
pub fn simulate_shot(sigma: f64, fat_tail_prob: f64, fat_tail_mult: f64) -> (f64, bool) {
    fat_tail_shot(sigma, fat_tail_prob, fat_tail_mult)
}

/// Simulate a standard shot without fat-tail behavior
///
/// # Arguments
/// * `sigma` - Player's skill parameter (Rayleigh σ)
///
/// # Returns
/// Miss distance in feet
///
/// # Example
/// ```
/// use continuum_golf_simulator::models::shot::simulate_standard_shot;
///
/// let miss = simulate_standard_shot(30.0);
/// assert!(miss >= 0.0);
/// ```
pub fn simulate_standard_shot(sigma: f64) -> f64 {
    rayleigh_random(sigma)
}

/// Batch of shot records for skill updates
///
/// Used to accumulate shots before triggering a Kalman filter update
#[derive(Debug, Clone)]
pub struct ShotBatch {
    /// Individual shot records (miss_distance, wager)
    pub shots: Vec<(f64, f64)>,
    /// Maximum batch size before triggering update
    pub max_size: usize,
}

impl ShotBatch {
    /// Create a new shot batch
    ///
    /// # Arguments
    /// * `max_size` - Maximum shots before forced update (typically 5)
    pub fn new(max_size: usize) -> Self {
        ShotBatch {
            shots: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Add a shot to the batch
    pub fn add_shot(&mut self, miss_distance: f64, wager: f64) {
        self.shots.push((miss_distance, wager));
    }

    /// Check if batch is full
    pub fn is_full(&self) -> bool {
        self.shots.len() >= self.max_size
    }

    /// Check if batch contains a high-stakes shot (≥10× average wager)
    ///
    /// High-stakes shots trigger immediate updates
    pub fn has_high_stakes_shot(&self, new_wager: f64) -> bool {
        if self.shots.is_empty() {
            return false;
        }

        let avg_wager: f64 = self.shots.iter().map(|(_, w)| w).sum::<f64>()
            / self.shots.len() as f64;

        new_wager >= 10.0 * avg_wager
    }

    /// Clear all shots from batch
    pub fn clear(&mut self) {
        self.shots.clear();
    }

    /// Get number of shots in batch
    pub fn len(&self) -> usize {
        self.shots.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.shots.is_empty()
    }

    /// Get all shots as a slice
    pub fn get_shots(&self) -> &[(f64, f64)] {
        &self.shots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shot_outcome_creation() {
        let outcome = ShotOutcome::new(10.0, 5.0, 10.0, 1, false);

        assert_eq!(outcome.miss_distance_ft, 10.0);
        assert_eq!(outcome.multiplier, 5.0);
        assert_eq!(outcome.payout, 50.0);
        assert_eq!(outcome.wager, 10.0);
        assert_eq!(outcome.hole_id, 1);
        assert!(!outcome.is_fat_tail);
    }

    #[test]
    fn test_net_result() {
        let winning_shot = ShotOutcome::new(5.0, 8.0, 10.0, 1, false);
        assert_eq!(winning_shot.net_result(), 70.0); // Won $80, wagered $10 = +$70

        let losing_shot = ShotOutcome::new(50.0, 0.0, 10.0, 1, false);
        assert_eq!(losing_shot.net_result(), -10.0); // Won $0, wagered $10 = -$10
    }

    #[test]
    fn test_is_win() {
        let winning_shot = ShotOutcome::new(5.0, 2.5, 10.0, 1, false);
        assert!(winning_shot.is_win());

        let breakeven_shot = ShotOutcome::new(20.0, 1.0, 10.0, 1, false);
        assert!(breakeven_shot.is_win());

        let losing_shot = ShotOutcome::new(50.0, 0.5, 10.0, 1, false);
        assert!(!losing_shot.is_win());
    }

    #[test]
    fn test_is_ace() {
        let ace = ShotOutcome::new(0.05, 10.0, 10.0, 1, false);
        assert!(ace.is_ace());

        let near_ace = ShotOutcome::new(0.2, 9.9, 10.0, 1, false);
        assert!(!near_ace.is_ace());
    }

    #[test]
    fn test_simulate_shot_produces_valid_distances() {
        // Run 100 simulations to ensure all are valid
        for _ in 0..100 {
            let (miss, is_fat_tail) = simulate_shot(30.0, 0.02, 3.0);

            assert!(miss >= 0.0, "Miss distance should be non-negative");
            assert!(miss < 500.0, "Miss distance should be reasonable");

            // is_fat_tail is a boolean, just check it exists
            let _ = is_fat_tail;
        }
    }

    #[test]
    fn test_simulate_standard_shot() {
        // Run 100 simulations
        for _ in 0..100 {
            let miss = simulate_standard_shot(30.0);
            assert!(miss >= 0.0);
            assert!(miss < 500.0);
        }
    }

    #[test]
    fn test_fat_tail_frequency() {
        // Run many simulations and check that ~2% are fat-tail
        let n = 10000;
        let mut fat_tail_count = 0;

        for _ in 0..n {
            let (_, is_fat_tail) = simulate_shot(30.0, 0.02, 3.0);
            if is_fat_tail {
                fat_tail_count += 1;
            }
        }

        let frequency = fat_tail_count as f64 / n as f64;

        // Should be close to 2% (within 1% tolerance)
        assert!(frequency > 0.01 && frequency < 0.03,
            "Fat-tail frequency was {}, expected ~0.02", frequency);
    }

    #[test]
    fn test_shot_batch_creation() {
        let batch = ShotBatch::new(5);
        assert_eq!(batch.max_size, 5);
        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_shot_batch_add_and_full() {
        let mut batch = ShotBatch::new(3);

        assert!(!batch.is_full());

        batch.add_shot(10.0, 5.0);
        batch.add_shot(12.0, 5.0);
        assert!(!batch.is_full());

        batch.add_shot(15.0, 5.0);
        assert!(batch.is_full());
    }

    #[test]
    fn test_high_stakes_detection() {
        let mut batch = ShotBatch::new(5);

        batch.add_shot(10.0, 5.0);
        batch.add_shot(12.0, 5.0);
        batch.add_shot(11.0, 5.0);

        // Average wager is 5.0, so 10× = 50.0
        assert!(!batch.has_high_stakes_shot(40.0));
        assert!(batch.has_high_stakes_shot(50.0));
        assert!(batch.has_high_stakes_shot(100.0));
    }

    #[test]
    fn test_batch_clear() {
        let mut batch = ShotBatch::new(5);

        batch.add_shot(10.0, 5.0);
        batch.add_shot(12.0, 5.0);
        assert_eq!(batch.len(), 2);

        batch.clear();
        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_get_shots() {
        let mut batch = ShotBatch::new(5);

        batch.add_shot(10.0, 5.0);
        batch.add_shot(12.0, 6.0);

        let shots = batch.get_shots();
        assert_eq!(shots.len(), 2);
        assert_eq!(shots[0], (10.0, 5.0));
        assert_eq!(shots[1], (12.0, 6.0));
    }
}
