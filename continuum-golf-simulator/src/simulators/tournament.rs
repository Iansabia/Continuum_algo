//! Tournament Simulator
//!
//! Simulates competitive tournaments with:
//! - Multiple game modes (Longest Drive, Closest to Pin)
//! - Flexible payout structures (Winner Takes All, Top 2, Top 3)
//! - House rake management
//! - Leaderboard generation

use crate::models::{
    hole::get_hole_by_id,
    player::Player,
    shot::simulate_shot,
};
use crate::simulators::venue::generate_player_pool;
use crate::simulators::venue::PlayerArchetype;
use serde::{Deserialize, Serialize};

/// Configuration for tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentConfig {
    /// Game mode for the tournament
    pub game_mode: GameMode,
    /// Number of players in the tournament
    pub num_players: usize,
    /// Entry fee per player
    pub entry_fee: f64,
    /// House rake as percentage (0.0 - 1.0)
    pub house_rake_percent: f64,
    /// Prize payout structure
    pub payout_structure: PayoutStructure,
    /// Number of attempts each player gets
    pub attempts_per_player: usize,
}

impl Default for TournamentConfig {
    fn default() -> Self {
        Self {
            game_mode: GameMode::ClosestToPin { hole_id: 4 },
            num_players: 20,
            entry_fee: 50.0,
            house_rake_percent: 0.10,
            payout_structure: PayoutStructure::Top3 {
                first: 0.60,
                second: 0.25,
                third: 0.15,
            },
            attempts_per_player: 5,
        }
    }
}

/// Game mode for tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    /// Longest drive competition (maximize distance)
    LongestDrive,
    /// Closest to pin (minimize miss distance)
    ClosestToPin { hole_id: u8 },
}

/// Prize payout structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayoutStructure {
    /// Winner takes the entire prize pool
    WinnerTakesAll,
    /// Top 2 split the pool
    Top2 { first: f64, second: f64 },
    /// Top 3 split the pool
    Top3 { first: f64, second: f64, third: f64 },
}

/// Results from a tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentResult {
    /// Leaderboard: (player_id, best_score)
    pub leaderboard: Vec<(String, f64)>,
    /// Total entry fees collected
    pub total_pool: f64,
    /// House rake amount
    pub house_rake: f64,
    /// Prize pool after rake
    pub prize_pool: f64,
    /// Prize payouts: (player_id, amount)
    pub payouts: Vec<(String, f64)>,
}

/// Run a tournament simulation
///
/// # Arguments
/// * `config` - Tournament configuration
///
/// # Returns
/// TournamentResult with leaderboard and payouts
pub fn run_tournament(config: TournamentConfig) -> TournamentResult {
    // Generate players
    let players = generate_player_pool(&PlayerArchetype::Uniform, config.num_players);

    // Collect scores
    let mut scores: Vec<(String, f64)> = players
        .iter()
        .map(|player| {
            let best_score = simulate_player_tournament_attempts(player, &config);
            (player.id.clone(), best_score)
        })
        .collect();

    // Sort leaderboard based on game mode
    match config.game_mode {
        GameMode::LongestDrive => {
            // Higher is better
            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        }
        GameMode::ClosestToPin { .. } => {
            // Lower is better
            scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        }
    }

    let leaderboard = scores;

    // Calculate prize pool
    let total_pool = config.entry_fee * config.num_players as f64;
    let house_rake = total_pool * config.house_rake_percent;
    let prize_pool = total_pool - house_rake;

    // Distribute prizes
    let payouts = distribute_prizes(&leaderboard, &config.payout_structure, prize_pool);

    TournamentResult {
        leaderboard,
        total_pool,
        house_rake,
        prize_pool,
        payouts,
    }
}

/// Simulate a player's tournament attempts
fn simulate_player_tournament_attempts(player: &Player, config: &TournamentConfig) -> f64 {
    match config.game_mode {
        GameMode::LongestDrive => {
            // For longest drive, we'll use a simple distance model
            // based on player skill (lower handicap = longer drive)
            let mut best_distance: f64 = 0.0;
            for _ in 0..config.attempts_per_player {
                // Base distance inversely related to handicap
                let base_distance = 250.0 - (player.handicap as f64 * 3.0);
                // Add some randomness
                let variance = 20.0;
                let (random_offset, _) = simulate_shot(variance, 0.02, 3.0);
                let distance = base_distance + random_offset - variance;
                best_distance = best_distance.max(distance);
            }
            best_distance
        }
        GameMode::ClosestToPin { hole_id } => {
            // For closest to pin, use actual shot simulation
            let hole = get_hole_by_id(hole_id).expect("Invalid hole_id");
            let skill_profile = player.get_skill_for_hole(hole);
            let sigma = skill_profile.kalman_filter.estimate;

            let mut best_miss = f64::MAX;
            for _ in 0..config.attempts_per_player {
                let (miss_distance, _) = simulate_shot(sigma, 0.02, 3.0);
                best_miss = best_miss.min(miss_distance);
            }
            best_miss
        }
    }
}

/// Distribute prizes according to payout structure
fn distribute_prizes(
    leaderboard: &[(String, f64)],
    structure: &PayoutStructure,
    prize_pool: f64,
) -> Vec<(String, f64)> {
    let mut payouts = Vec::new();

    match structure {
        PayoutStructure::WinnerTakesAll => {
            if !leaderboard.is_empty() {
                payouts.push((leaderboard[0].0.clone(), prize_pool));
            }
        }
        PayoutStructure::Top2 { first, second } => {
            if leaderboard.len() >= 1 {
                payouts.push((leaderboard[0].0.clone(), prize_pool * first));
            }
            if leaderboard.len() >= 2 {
                payouts.push((leaderboard[1].0.clone(), prize_pool * second));
            }
        }
        PayoutStructure::Top3 {
            first,
            second,
            third,
        } => {
            if leaderboard.len() >= 1 {
                payouts.push((leaderboard[0].0.clone(), prize_pool * first));
            }
            if leaderboard.len() >= 2 {
                payouts.push((leaderboard[1].0.clone(), prize_pool * second));
            }
            if leaderboard.len() >= 3 {
                payouts.push((leaderboard[2].0.clone(), prize_pool * third));
            }
        }
    }

    payouts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tournament_config_default() {
        let config = TournamentConfig::default();
        assert_eq!(config.num_players, 20);
        assert_eq!(config.entry_fee, 50.0);
        assert_eq!(config.house_rake_percent, 0.10);
        assert_eq!(config.attempts_per_player, 5);
    }

    #[test]
    fn test_run_tournament_closest_to_pin() {
        let config = TournamentConfig {
            game_mode: GameMode::ClosestToPin { hole_id: 4 },
            num_players: 10,
            entry_fee: 20.0,
            house_rake_percent: 0.10,
            payout_structure: PayoutStructure::Top3 {
                first: 0.60,
                second: 0.25,
                third: 0.15,
            },
            attempts_per_player: 3,
        };

        let result = run_tournament(config);

        assert_eq!(result.leaderboard.len(), 10);
        assert_eq!(result.total_pool, 200.0); // 10 * $20
        assert_eq!(result.house_rake, 20.0); // 10% of $200
        assert_eq!(result.prize_pool, 180.0); // $200 - $20

        // Check that leaderboard is sorted (lower is better for CTP)
        for i in 0..result.leaderboard.len() - 1 {
            assert!(result.leaderboard[i].1 <= result.leaderboard[i + 1].1,
                "Leaderboard should be sorted ascending for CTP");
        }

        // Check payouts
        assert_eq!(result.payouts.len(), 3);
        let total_paid: f64 = result.payouts.iter().map(|(_, amt)| amt).sum();
        assert!((total_paid - result.prize_pool).abs() < 0.01);
    }

    #[test]
    fn test_run_tournament_longest_drive() {
        let config = TournamentConfig {
            game_mode: GameMode::LongestDrive,
            num_players: 5,
            entry_fee: 10.0,
            house_rake_percent: 0.05,
            payout_structure: PayoutStructure::WinnerTakesAll,
            attempts_per_player: 3,
        };

        let result = run_tournament(config);

        assert_eq!(result.leaderboard.len(), 5);
        assert_eq!(result.total_pool, 50.0);
        assert_eq!(result.house_rake, 2.5);
        assert_eq!(result.prize_pool, 47.5);

        // Check that leaderboard is sorted (higher is better for longest drive)
        for i in 0..result.leaderboard.len() - 1 {
            assert!(result.leaderboard[i].1 >= result.leaderboard[i + 1].1,
                "Leaderboard should be sorted descending for longest drive");
        }

        // Winner takes all
        assert_eq!(result.payouts.len(), 1);
        assert_eq!(result.payouts[0].1, 47.5);
    }

    #[test]
    fn test_distribute_prizes_winner_takes_all() {
        let leaderboard = vec![
            ("player_1".to_string(), 5.0),
            ("player_2".to_string(), 10.0),
            ("player_3".to_string(), 15.0),
        ];

        let payouts = distribute_prizes(
            &leaderboard,
            &PayoutStructure::WinnerTakesAll,
            100.0,
        );

        assert_eq!(payouts.len(), 1);
        assert_eq!(payouts[0].0, "player_1");
        assert_eq!(payouts[0].1, 100.0);
    }

    #[test]
    fn test_distribute_prizes_top2() {
        let leaderboard = vec![
            ("player_1".to_string(), 5.0),
            ("player_2".to_string(), 10.0),
            ("player_3".to_string(), 15.0),
        ];

        let payouts = distribute_prizes(
            &leaderboard,
            &PayoutStructure::Top2 {
                first: 0.70,
                second: 0.30,
            },
            100.0,
        );

        assert_eq!(payouts.len(), 2);
        assert_eq!(payouts[0].0, "player_1");
        assert_eq!(payouts[0].1, 70.0);
        assert_eq!(payouts[1].0, "player_2");
        assert_eq!(payouts[1].1, 30.0);
    }

    #[test]
    fn test_distribute_prizes_top3() {
        let leaderboard = vec![
            ("player_1".to_string(), 5.0),
            ("player_2".to_string(), 10.0),
            ("player_3".to_string(), 15.0),
            ("player_4".to_string(), 20.0),
        ];

        let payouts = distribute_prizes(
            &leaderboard,
            &PayoutStructure::Top3 {
                first: 0.50,
                second: 0.30,
                third: 0.20,
            },
            100.0,
        );

        assert_eq!(payouts.len(), 3);
        assert_eq!(payouts[0].0, "player_1");
        assert_eq!(payouts[0].1, 50.0);
        assert_eq!(payouts[1].0, "player_2");
        assert_eq!(payouts[1].1, 30.0);
        assert_eq!(payouts[2].0, "player_3");
        assert_eq!(payouts[2].1, 20.0);
    }

    #[test]
    fn test_payout_structure_sums_to_one() {
        // Test that default Top3 structure sums to 1.0
        if let PayoutStructure::Top3 { first, second, third } =
            TournamentConfig::default().payout_structure
        {
            assert_eq!(first + second + third, 1.0);
        }
    }

    #[test]
    fn test_tournament_with_few_players() {
        // Test with fewer players than payout positions
        let config = TournamentConfig {
            game_mode: GameMode::ClosestToPin { hole_id: 1 },
            num_players: 2,
            entry_fee: 10.0,
            house_rake_percent: 0.0,
            payout_structure: PayoutStructure::Top3 {
                first: 0.50,
                second: 0.30,
                third: 0.20,
            },
            attempts_per_player: 1,
        };

        let result = run_tournament(config);

        // Should only pay out to 2 players (not 3)
        assert_eq!(result.payouts.len(), 2);
    }
}