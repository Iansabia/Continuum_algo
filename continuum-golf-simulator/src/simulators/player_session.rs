//! Player Session Simulator
//!
//! Simulates individual player gaming sessions with:
//! - Configurable shot counts and wager ranges
//! - Flexible hole selection strategies
//! - Real-time Kalman filter updates for skill tracking
//! - Batch processing and high-stakes shot detection
//! - Developer mode for manual testing

use crate::models::{
    hole::{get_hole_by_id, Hole, HOLE_CONFIGURATIONS},
    player::Player,
    shot::{simulate_shot, ShotOutcome},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a player gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Number of shots to simulate in the session
    pub num_shots: usize,
    /// Minimum wager per shot
    pub wager_min: f64,
    /// Maximum wager per shot
    pub wager_max: f64,
    /// Strategy for selecting which hole to play
    pub hole_selection: HoleSelection,
    /// Optional developer mode settings for testing
    pub developer_mode: Option<DeveloperMode>,
    /// Fat-tail probability (default: 0.02 = 2%)
    pub fat_tail_prob: f64,
    /// Fat-tail multiplier (default: 3.0)
    pub fat_tail_mult: f64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            num_shots: 100,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Random,
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        }
    }
}

/// Strategy for selecting which hole to play
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HoleSelection {
    /// Random selection from all 8 holes
    Random,
    /// Weighted probabilities for each hole
    /// Vec of (hole_id, probability) pairs that must sum to 1.0
    Weighted(Vec<(u8, f64)>),
    /// Always play the same hole
    Fixed(u8),
}

/// Developer mode settings for manual testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperMode {
    /// If set, use this miss distance instead of simulating
    pub manual_miss_distance: Option<f64>,
    /// If true, disable Kalman filter updates (skill stays constant)
    pub disable_kalman: bool,
}

/// Results from a completed player session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResult {
    /// Total amount wagered across all shots
    pub total_wagered: f64,
    /// Total amount won (payouts) across all shots
    pub total_won: f64,
    /// Net gain or loss (total_won - total_wagered)
    pub net_gain_loss: f64,
    /// All shot outcomes in chronological order
    pub shots: Vec<ShotOutcome>,
    /// Final skill profiles after all Kalman updates
    pub final_skill_profiles: HashMap<String, f64>, // ClubCategory -> sigma
    /// Actual house edge for this session
    pub session_house_edge: f64,
    /// Number of Kalman updates performed
    pub num_kalman_updates: usize,
    /// Number of high-stakes shots (triggered immediate updates)
    pub num_high_stakes_shots: usize,
}

impl SessionResult {
    /// Calculate session house edge as percentage
    pub fn house_edge_percent(&self) -> f64 {
        if self.total_wagered > 0.0 {
            (1.0 - self.total_won / self.total_wagered) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate average wager per shot
    pub fn avg_wager(&self) -> f64 {
        if !self.shots.is_empty() {
            self.total_wagered / self.shots.len() as f64
        } else {
            0.0
        }
    }

    /// Calculate win rate (percentage of shots with payout > 0)
    pub fn win_rate(&self) -> f64 {
        if self.shots.is_empty() {
            return 0.0;
        }
        let wins = self.shots.iter().filter(|s| s.payout > 0.0).count();
        (wins as f64 / self.shots.len() as f64) * 100.0
    }
}

/// Run a player gaming session simulation
///
/// # Arguments
/// * `player` - Mutable reference to player (skill will be updated)
/// * `config` - Session configuration parameters
///
/// # Returns
/// SessionResult with all shot outcomes and final statistics
pub fn run_session(player: &mut Player, config: SessionConfig) -> SessionResult {
    let mut rng = rand::thread_rng();
    let mut shots = Vec::with_capacity(config.num_shots);
    let mut total_wagered = 0.0;
    let mut total_won = 0.0;
    let mut num_kalman_updates = 0;
    let mut num_high_stakes_shots = 0;

    for _shot_num in 0..config.num_shots {
        // Select hole based on strategy
        let hole = select_hole(&config.hole_selection, &mut rng);

        // Determine wager for this shot
        let wager = rng.gen_range(config.wager_min..=config.wager_max);

        // Get player's current skill for this hole's category
        let skill_profile = player.get_skill_for_hole(hole);
        let current_sigma = skill_profile.kalman_filter.estimate;

        // Calculate P_max for current skill level
        let p_max = player.calculate_p_max(hole);

        // Simulate or use manual miss distance
        let (miss_distance, is_fat_tail) = if let Some(ref dev_mode) = config.developer_mode {
            if let Some(manual_dist) = dev_mode.manual_miss_distance {
                (manual_dist, false)
            } else {
                simulate_shot(current_sigma, config.fat_tail_prob, config.fat_tail_mult)
            }
        } else {
            simulate_shot(current_sigma, config.fat_tail_prob, config.fat_tail_mult)
        };

        // Calculate payout
        let payout_multiplier = hole.calculate_payout(miss_distance, p_max);
        let payout_amount = payout_multiplier * wager;

        // Create shot outcome
        let outcome = ShotOutcome {
            miss_distance_ft: miss_distance,
            multiplier: payout_multiplier,
            payout: payout_amount,
            wager,
            hole_id: hole.id,
            is_fat_tail,
        };

        total_wagered += wager;
        total_won += payout_amount;
        shots.push(outcome);

        // Add shot to batch (unless Kalman is disabled)
        if config.developer_mode.as_ref().map_or(true, |dm| !dm.disable_kalman) {
            // Check if this is a high-stakes shot
            let is_high_stakes = player.is_high_stakes_shot(hole, wager);

            if is_high_stakes {
                num_high_stakes_shots += 1;
                // Process existing batch first if it has shots
                let skill = player.get_skill_for_hole(hole);
                if !skill.shot_batch.is_empty() {
                    player.update_skill(hole, p_max);
                    num_kalman_updates += 1;
                }
            }

            // Add shot to batch
            let batch_full = player.add_shot_to_batch(hole, miss_distance, wager);

            // Update if batch is full or this is a high-stakes shot
            if batch_full || is_high_stakes {
                player.update_skill(hole, p_max);
                num_kalman_updates += 1;
            }
        }
    }

    // Process any remaining shots in batches at end of session
    if config.developer_mode.as_ref().map_or(true, |dm| !dm.disable_kalman) {
        for hole in HOLE_CONFIGURATIONS.iter() {
            let skill = player.get_skill_for_hole(hole);
            if !skill.shot_batch.is_empty() {
                let p_max = player.calculate_p_max(hole);
                player.update_skill(hole, p_max);
                num_kalman_updates += 1;
            }
        }
    }

    // Collect final skill profiles
    let final_skill_profiles = player
        .skill_profiles
        .iter()
        .map(|(cat, profile)| {
            (format!("{:?}", cat), profile.kalman_filter.estimate)
        })
        .collect();

    let net_gain_loss = total_won - total_wagered;
    let session_house_edge = if total_wagered > 0.0 {
        1.0 - (total_won / total_wagered)
    } else {
        0.0
    };

    SessionResult {
        total_wagered,
        total_won,
        net_gain_loss,
        shots,
        final_skill_profiles,
        session_house_edge,
        num_kalman_updates,
        num_high_stakes_shots,
    }
}

/// Select a hole based on the configured strategy
fn select_hole<'a>(selection: &HoleSelection, rng: &mut impl Rng) -> &'a Hole {
    match selection {
        HoleSelection::Random => {
            let idx = rng.gen_range(0..HOLE_CONFIGURATIONS.len());
            &HOLE_CONFIGURATIONS[idx]
        }
        HoleSelection::Weighted(weights) => {
            let roll: f64 = rng.gen();
            let mut cumulative = 0.0;
            for (hole_id, prob) in weights {
                cumulative += prob;
                if roll < cumulative {
                    return get_hole_by_id(*hole_id).expect("Invalid hole_id in weights");
                }
            }
            // Fallback to last hole if rounding errors occur
            let last_id = weights.last().map(|(id, _)| *id).unwrap_or(1);
            get_hole_by_id(last_id).expect("Invalid hole_id in weights")
        }
        HoleSelection::Fixed(hole_id) => {
            get_hole_by_id(*hole_id).expect("Invalid hole_id in Fixed selection")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.num_shots, 100);
        assert_eq!(config.wager_min, 5.0);
        assert_eq!(config.wager_max, 10.0);
        assert_eq!(config.fat_tail_prob, 0.02);
        assert_eq!(config.fat_tail_mult, 3.0);
    }

    #[test]
    fn test_hole_selection_fixed() {
        let selection = HoleSelection::Fixed(3);
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let hole = select_hole(&selection, &mut rng);
            assert_eq!(hole.id, 3);
        }
    }

    #[test]
    fn test_hole_selection_random() {
        let selection = HoleSelection::Random;
        let mut rng = rand::thread_rng();
        let mut seen_holes = std::collections::HashSet::new();

        // Should see multiple different holes over 100 selections
        for _ in 0..100 {
            let hole = select_hole(&selection, &mut rng);
            seen_holes.insert(hole.id);
        }

        assert!(seen_holes.len() > 1, "Random selection should pick different holes");
    }

    #[test]
    fn test_hole_selection_weighted() {
        // 100% weight on hole 5
        let selection = HoleSelection::Weighted(vec![(5, 1.0)]);
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let hole = select_hole(&selection, &mut rng);
            assert_eq!(hole.id, 5);
        }
    }

    #[test]
    fn test_run_session_basic() {
        let mut player = Player::new("test_player".to_string(), 15);
        let config = SessionConfig {
            num_shots: 10,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: None,
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        assert_eq!(result.shots.len(), 10);
        assert!(result.total_wagered >= 50.0 && result.total_wagered <= 100.0);
        assert_eq!(result.net_gain_loss, result.total_won - result.total_wagered);
        // House edge can be negative in individual sessions (player wins)
        // Typically should be between -5.0 and 1.0 for small sample sizes
        assert!(result.session_house_edge >= -5.0 && result.session_house_edge <= 1.0);
    }

    #[test]
    fn test_run_session_developer_mode_manual_miss() {
        let mut player = Player::new("test_player".to_string(), 15);
        let config = SessionConfig {
            num_shots: 5,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: Some(5.0), // Always miss by 5ft
                disable_kalman: false,
            }),
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        // All shots should have exactly 5ft miss distance
        for shot in &result.shots {
            assert_eq!(shot.miss_distance_ft, 5.0);
        }
    }

    #[test]
    fn test_run_session_developer_mode_disable_kalman() {
        let mut player = Player::new("test_player".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap();
        let initial_sigma = player.get_skill_for_hole(hole).kalman_filter.estimate;

        let config = SessionConfig {
            num_shots: 20,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: None,
                disable_kalman: true, // No updates
            }),
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        assert_eq!(result.num_kalman_updates, 0);

        // Skill should not have changed
        let final_sigma = player.get_skill_for_hole(hole).kalman_filter.estimate;
        assert_eq!(initial_sigma, final_sigma);
    }

    #[test]
    fn test_session_result_calculations() {
        let result = SessionResult {
            total_wagered: 100.0,
            total_won: 88.0,
            net_gain_loss: -12.0,
            shots: vec![
                ShotOutcome {
                    miss_distance_ft: 10.0,
                    multiplier: 2.0,
                    payout: 20.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 30.0,
                    multiplier: 0.0,
                    payout: 0.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 15.0,
                    multiplier: 1.5,
                    payout: 15.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 8.0,
                    multiplier: 2.3,
                    payout: 23.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 25.0,
                    multiplier: 0.0,
                    payout: 0.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 12.0,
                    multiplier: 1.8,
                    payout: 18.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 20.0,
                    multiplier: 0.0,
                    payout: 0.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 9.0,
                    multiplier: 2.1,
                    payout: 21.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 30.0,
                    multiplier: 0.0,
                    payout: 0.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
                ShotOutcome {
                    miss_distance_ft: 11.0,
                    multiplier: 1.9,
                    payout: 19.0,
                    wager: 10.0,
                    hole_id: 1,
                    is_fat_tail: false,
                },
            ],
            final_skill_profiles: HashMap::new(),
            session_house_edge: 0.12,
            num_kalman_updates: 1,
            num_high_stakes_shots: 0,
        };

        assert_eq!(result.house_edge_percent(), 12.0);
        assert_eq!(result.avg_wager(), 10.0);
        // 6 out of 10 shots have payout > 0 (shots 1, 3, 4, 6, 8, 10)
        assert_eq!(result.win_rate(), 60.0);
    }

    #[test]
    fn test_session_kalman_updates_occur() {
        let mut player = Player::new("test_player".to_string(), 20);
        let config = SessionConfig {
            num_shots: 25, // Should trigger multiple batch updates
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(3),
            developer_mode: None,
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        // Should have at least some Kalman updates
        assert!(result.num_kalman_updates > 0,
            "Expected Kalman updates, got {}", result.num_kalman_updates);
    }
}