// Shot outcome modeling
//
// Models the outcome of a single shot, including:
// - Miss distance (from Rayleigh distribution)
// - Fat-tail events (2% chance of 3× worse dispersion)
// - Payout calculation
// - Metadata for analysis

use serde::{Deserialize, Serialize};
// Removed unused Rayleigh imports - now using BVN for all shot generation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotOutcome {
    pub miss_distance_ft: f64,

    pub multiplier: f64,

    pub payout: f64,

    pub wager: f64,

    pub hole_id: u8,

    pub is_fat_tail: bool,

    pub x_ft: Option<f64>,

    pub y_ft: Option<f64>,

    pub p_max: f64,
}

impl ShotOutcome {
    /// Create a new shot outcome (legacy 1D Rayleigh interface)
    ///
    /// # Arguments
    /// * `miss_distance_ft` - Miss distance in feet
    /// * `multiplier` - Payout multiplier
    /// * `wager` - Wager amount
    /// * `hole_id` - Hole number (1-8)
    /// * `is_fat_tail` - Whether this was a fat-tail event
    /// * `p_max` - P_max value at the time of this shot
    ///
    /// # Returns
    /// New ShotOutcome with calculated payout (no x,y coordinates)
    pub fn new(
        miss_distance_ft: f64,
        multiplier: f64,
        wager: f64,
        hole_id: u8,
        is_fat_tail: bool,
        p_max: f64,
    ) -> Self {
        let payout = multiplier * wager;
        ShotOutcome {
            miss_distance_ft,
            multiplier,
            payout,
            wager,
            hole_id,
            is_fat_tail,
            x_ft: None,
            y_ft: None,
            p_max,
        }
    }

    /// Create a new shot outcome with 2D coordinates (BVN interface)
    ///
    /// # Arguments
    /// * `x_ft` - Lateral position (feet right of target line)
    /// * `y_ft` - Distance position (feet from pin, positive = long)
    /// * `multiplier` - Payout multiplier
    /// * `wager` - Wager amount
    /// * `hole_id` - Hole number (1-8)
    /// * `is_fat_tail` - Whether this was a fat-tail event
    /// * `p_max` - P_max value at the time of this shot
    ///
    /// # Returns
    /// New ShotOutcome with calculated payout and (x,y) coordinates
    pub fn new_bvn(
        x_ft: f64,
        y_ft: f64,
        multiplier: f64,
        wager: f64,
        hole_id: u8,
        is_fat_tail: bool,
        p_max: f64,
    ) -> Self {
        let miss_distance_ft = (x_ft * x_ft + y_ft * y_ft).sqrt();
        let payout = multiplier * wager;
        ShotOutcome {
            miss_distance_ft,
            multiplier,
            payout,
            wager,
            hole_id,
            is_fat_tail,
            x_ft: Some(x_ft),
            y_ft: Some(y_ft),
            p_max,
        }
    }

    pub fn net_result(&self) -> f64 {
        self.payout - self.wager
    }

    pub fn is_win(&self) -> bool {
        self.multiplier >= 1.0
    }

    pub fn is_ace(&self) -> bool {
        self.miss_distance_ft < 0.1 // Within 1 inch
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotRecord {
    pub miss_distance: f64,

    pub wager: f64,

    pub x_ft: Option<f64>,

    pub y_ft: Option<f64>,
}

impl ShotRecord {
    pub fn new_1d(miss_distance: f64, wager: f64) -> Self {
        ShotRecord {
            miss_distance,
            wager,
            x_ft: None,
            y_ft: None,
        }
    }

    pub fn new_2d(x_ft: f64, y_ft: f64, wager: f64) -> Self {
        let miss_distance = (x_ft * x_ft + y_ft * y_ft).sqrt();
        ShotRecord {
            miss_distance,
            wager,
            x_ft: Some(x_ft),
            y_ft: Some(y_ft),
        }
    }

    pub fn has_coordinates(&self) -> bool {
        self.x_ft.is_some() && self.y_ft.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotBatch {
    pub shots: Vec<ShotRecord>,

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

    pub fn add_shot(&mut self, miss_distance: f64, wager: f64) {
        self.shots.push(ShotRecord::new_1d(miss_distance, wager));
    }

    pub fn add_shot_2d(&mut self, x_ft: f64, y_ft: f64, wager: f64) {
        self.shots.push(ShotRecord::new_2d(x_ft, y_ft, wager));
    }

    pub fn is_full(&self) -> bool {
        self.shots.len() >= self.max_size
    }

    pub fn has_2d_shots(&self) -> bool {
        self.shots.iter().any(|s| s.has_coordinates())
    }

    pub fn has_high_stakes_shot(&self, new_wager: f64) -> bool {
        if self.shots.is_empty() {
            return false;
        }

        let avg_wager: f64 =
            self.shots.iter().map(|s| s.wager).sum::<f64>() / self.shots.len() as f64;

        new_wager >= 10.0 * avg_wager
    }

    pub fn clear(&mut self) {
        self.shots.clear();
    }

    pub fn len(&self) -> usize {
        self.shots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.shots.is_empty()
    }

    pub fn get_shots(&self) -> &[ShotRecord] {
        &self.shots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shot_outcome_creation() {
        let outcome = ShotOutcome::new(10.0, 5.0, 10.0, 1, false, 10.0);

        assert_eq!(outcome.miss_distance_ft, 10.0);
        assert_eq!(outcome.multiplier, 5.0);
        assert_eq!(outcome.payout, 50.0);
        assert_eq!(outcome.wager, 10.0);
        assert_eq!(outcome.hole_id, 1);
        assert!(!outcome.is_fat_tail);
    }

    #[test]
    fn test_net_result() {
        let winning_shot = ShotOutcome::new(5.0, 8.0, 10.0, 1, false, 10.0);
        assert_eq!(winning_shot.net_result(), 70.0); // Won $80, wagered $10 = +$70

        let losing_shot = ShotOutcome::new(50.0, 0.0, 10.0, 1, false, 10.0);
        assert_eq!(losing_shot.net_result(), -10.0); // Won $0, wagered $10 = -$10
    }

    #[test]
    fn test_is_win() {
        let winning_shot = ShotOutcome::new(5.0, 2.5, 10.0, 1, false, 10.0);
        assert!(winning_shot.is_win());

        let breakeven_shot = ShotOutcome::new(20.0, 1.0, 10.0, 1, false, 10.0);
        assert!(breakeven_shot.is_win());

        let losing_shot = ShotOutcome::new(50.0, 0.5, 10.0, 1, false, 10.0);
        assert!(!losing_shot.is_win());
    }

    #[test]
    fn test_is_ace() {
        let ace = ShotOutcome::new(0.05, 10.0, 10.0, 1, false, 10.0);
        assert!(ace.is_ace());

        let near_ace = ShotOutcome::new(0.2, 9.9, 10.0, 1, false, 10.0);
        assert!(!near_ace.is_ace());
    }

    #[test]
    fn test_simulate_shot_produces_valid_distances() {
        use crate::math::distributions::fat_tail_shot_bvn;
        // Run 100 simulations to ensure all are valid
        for _ in 0..100 {
            let ((x, y), is_fat_tail) = fat_tail_shot_bvn(0.0, 0.0, 30.0, 30.0, 0.0, 0.02, 3.0);
            let miss = (x * x + y * y).sqrt();

            assert!(miss >= 0.0, "Miss distance should be non-negative");
            assert!(miss < 500.0, "Miss distance should be reasonable");

            // is_fat_tail is a boolean, just check it exists
            let _ = is_fat_tail;
        }
    }

    #[test]
    fn test_simulate_standard_shot() {
        use crate::math::distributions::bvn_random;
        // Run 100 simulations
        for _ in 0..100 {
            let (x, y) = bvn_random(0.0, 0.0, 30.0, 30.0, 0.0);
            let miss = (x * x + y * y).sqrt();
            assert!(miss >= 0.0);
            assert!(miss < 500.0);
        }
    }

    #[test]
    fn test_fat_tail_frequency() {
        use crate::math::distributions::fat_tail_shot_bvn;
        // Run many simulations and check that ~2% are fat-tail
        let n = 10000;
        let mut fat_tail_count = 0;

        for _ in 0..n {
            let (_, is_fat_tail) = fat_tail_shot_bvn(0.0, 0.0, 30.0, 30.0, 0.0, 0.02, 3.0);
            if is_fat_tail {
                fat_tail_count += 1;
            }
        }

        let frequency = fat_tail_count as f64 / n as f64;

        // Should be close to 2% (within 1% tolerance)
        assert!(
            frequency > 0.01 && frequency < 0.03,
            "Fat-tail frequency was {}, expected ~0.02",
            frequency
        );
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
        assert_eq!(shots[0].miss_distance, 10.0);
        assert_eq!(shots[0].wager, 5.0);
        assert_eq!(shots[1].miss_distance, 12.0);
        assert_eq!(shots[1].wager, 6.0);
    }
}
