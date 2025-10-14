//! Venue Economics Simulator
//!
//! Simulates complete venue operations with:
//! - Multiple hitting bays running in parallel
//! - Realistic player population distributions
//! - Time-series profit tracking
//! - Heatmap data for handicap × distance analysis
//! - Payout distribution histograms

use crate::models::{
    hole::HOLE_CONFIGURATIONS,
    player::Player,
};
use crate::simulators::player_session::{run_session, HoleSelection, SessionConfig};
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for venue simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueConfig {
    /// Number of hitting bays in the venue
    pub num_bays: usize,
    /// Operating hours
    pub hours: f64,
    /// Average shots per bay per hour
    pub shots_per_hour: usize,
    /// Player population distribution
    pub player_archetype: PlayerArchetype,
    /// Wager range for players (min, max)
    pub wager_range: (f64, f64),
}

impl Default for VenueConfig {
    fn default() -> Self {
        Self {
            num_bays: 20,
            hours: 8.0,
            shots_per_hour: 100,
            player_archetype: PlayerArchetype::BellCurve { mean: 15, std_dev: 5.0 },
            wager_range: (5.0, 20.0),
        }
    }
}

/// Player population distribution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerArchetype {
    /// Uniform distribution across all handicaps (0-30)
    Uniform,
    /// Normal distribution centered at mean with std_dev
    BellCurve { mean: u8, std_dev: f64 },
    /// Skewed toward beginners (high handicaps)
    SkewedHigh,
    /// Skewed toward experts (low handicaps)
    SkewedLow,
}

/// Results from venue simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueResult {
    /// Total amount wagered across all shots
    pub total_wagered: f64,
    /// Total payouts across all shots
    pub total_payouts: f64,
    /// Net profit for the venue
    pub net_profit: f64,
    /// Hold percentage (profit / wagered)
    pub hold_percentage: f64,
    /// Profit over time: (hour, cumulative_profit) pairs
    pub profit_over_time: Vec<(f64, f64)>,
    /// Heatmap data for visualization
    pub heatmap_data: HeatmapData,
    /// Payout distribution: bins 0x, 1x, 2x, ..., 10x+
    pub payout_distribution: [usize; 11],
    /// Total number of shots simulated
    pub total_shots: usize,
}

/// Heatmap data showing hold percentage by handicap and distance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    /// Handicap bin labels: "0-4", "5-9", etc.
    pub handicap_bins: Vec<String>,
    /// Distance bins (hole distances)
    pub distance_bins: Vec<u16>,
    /// Hold percentages: [handicap_bin][distance_bin] -> hold%
    pub hold_percentages: Vec<Vec<f64>>,
}

/// Generate a pool of players based on archetype
///
/// # Arguments
/// * `archetype` - Distribution strategy for handicaps
/// * `size` - Number of players to generate
///
/// # Returns
/// Vector of players with handicaps drawn from the specified distribution
pub fn generate_player_pool(archetype: &PlayerArchetype, size: usize) -> Vec<Player> {
    let mut rng = rand::thread_rng();
    let mut players = Vec::with_capacity(size);

    for i in 0..size {
        let handicap = match archetype {
            PlayerArchetype::Uniform => {
                rng.gen_range(0..=30)
            }
            PlayerArchetype::BellCurve { mean, std_dev } => {
                let normal = Normal::new(*mean as f64, *std_dev).unwrap();
                let sample = normal.sample(&mut rng);
                sample.round().clamp(0.0, 30.0) as u8
            }
            PlayerArchetype::SkewedHigh => {
                // Beta-like distribution skewed toward high handicaps (20-30)
                let uniform = Uniform::new(0.0, 1.0);
                let u: f64 = uniform.sample(&mut rng);
                let skewed = 1.0 - (1.0 - u) * (1.0 - u); // Skew toward 1
                (skewed * 30.0).round() as u8
            }
            PlayerArchetype::SkewedLow => {
                // Beta-like distribution skewed toward low handicaps (0-10)
                let uniform = Uniform::new(0.0, 1.0);
                let u: f64 = uniform.sample(&mut rng);
                let skewed = u * u; // Skew toward 0
                (skewed * 30.0).round() as u8
            }
        };

        players.push(Player::new(format!("player_{}", i), handicap));
    }

    players
}

/// Run full venue simulation
///
/// # Arguments
/// * `config` - Venue configuration
///
/// # Returns
/// VenueResult with comprehensive analytics
pub fn run_venue_simulation(config: VenueConfig) -> VenueResult {
    let total_shots = (config.num_bays as f64 * config.hours * config.shots_per_hour as f64) as usize;
    let shots_per_bay = (total_shots / config.num_bays) as usize;

    // Generate player pool (one per bay for simplicity)
    let players = generate_player_pool(&config.player_archetype, config.num_bays);

    // Run sessions in parallel for each bay
    let bay_results: Vec<_> = players
        .into_par_iter()
        .map(|mut player| {
            let session_config = SessionConfig {
                num_shots: shots_per_bay,
                wager_min: config.wager_range.0,
                wager_max: config.wager_range.1,
                hole_selection: HoleSelection::Random,
                developer_mode: None,
                ..Default::default()
            };

            let result = run_session(&mut player, session_config);
            (player, result)
        })
        .collect();

    // Aggregate results
    let mut total_wagered = 0.0;
    let mut total_payouts = 0.0;
    let mut all_shots = Vec::new();

    for (_player, session_result) in &bay_results {
        total_wagered += session_result.total_wagered;
        total_payouts += session_result.total_won;
        all_shots.extend(session_result.shots.clone());
    }

    let net_profit = total_wagered - total_payouts;
    let hold_percentage = if total_wagered > 0.0 {
        net_profit / total_wagered
    } else {
        0.0
    };

    // Calculate profit over time (simplified: evenly distributed)
    let mut profit_over_time = Vec::new();
    let profit_per_hour = net_profit / config.hours;
    for hour in 0..=(config.hours as usize) {
        let cumulative = profit_per_hour * hour as f64;
        profit_over_time.push((hour as f64, cumulative));
    }

    // Build heatmap data
    let heatmap_data = build_heatmap(&bay_results);

    // Build payout distribution
    let payout_distribution = build_payout_distribution(&all_shots);

    VenueResult {
        total_wagered,
        total_payouts,
        net_profit,
        hold_percentage,
        profit_over_time,
        heatmap_data,
        payout_distribution,
        total_shots: all_shots.len(),
    }
}

/// Build heatmap data from bay results
fn build_heatmap(bay_results: &[(Player, crate::simulators::player_session::SessionResult)]) -> HeatmapData {
    // Define handicap bins
    let handicap_bins = vec![
        "0-4".to_string(),
        "5-9".to_string(),
        "10-14".to_string(),
        "15-19".to_string(),
        "20-24".to_string(),
        "25-30".to_string(),
    ];

    // Get all hole distances
    let distance_bins: Vec<u16> = HOLE_CONFIGURATIONS.iter().map(|h| h.distance_yds).collect();

    // Initialize hold percentage matrix
    let mut hold_matrix = vec![vec![0.0; distance_bins.len()]; handicap_bins.len()];
    let mut count_matrix = vec![vec![0; distance_bins.len()]; handicap_bins.len()];

    for (player, session_result) in bay_results {
        let handicap_bin = match player.handicap {
            0..=4 => 0,
            5..=9 => 1,
            10..=14 => 2,
            15..=19 => 3,
            20..=24 => 4,
            25..=30 => 5,
            _ => 5,
        };

        for shot in &session_result.shots {
            if let Some(hole_idx) = HOLE_CONFIGURATIONS.iter().position(|h| h.id == shot.hole_id) {
                let profit = shot.wager - shot.payout;
                hold_matrix[handicap_bin][hole_idx] += profit;
                count_matrix[handicap_bin][hole_idx] += 1;
            }
        }
    }

    // Calculate hold percentages
    let hold_percentages: Vec<Vec<f64>> = hold_matrix
        .iter()
        .zip(count_matrix.iter())
        .map(|(holds, counts)| {
            holds
                .iter()
                .zip(counts.iter())
                .map(|(profit, count)| {
                    if *count > 0 {
                        // Hold % = profit / total_wagered_in_bin
                        // Approximate wager as profit / hold_rate (we use average)
                        profit / (*count as f64 * 10.0) // Assume avg wager ~$10
                    } else {
                        0.0
                    }
                })
                .collect()
        })
        .collect();

    HeatmapData {
        handicap_bins,
        distance_bins,
        hold_percentages,
    }
}

/// Build payout distribution histogram
fn build_payout_distribution(shots: &[crate::models::shot::ShotOutcome]) -> [usize; 11] {
    let mut distribution = [0; 11];

    for shot in shots {
        let bin = if shot.multiplier >= 10.0 {
            10
        } else {
            shot.multiplier.floor() as usize
        };
        distribution[bin] += 1;
    }

    distribution
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_venue_config_default() {
        let config = VenueConfig::default();
        assert_eq!(config.num_bays, 20);
        assert_eq!(config.hours, 8.0);
        assert_eq!(config.shots_per_hour, 100);
    }

    #[test]
    fn test_generate_player_pool_uniform() {
        let players = generate_player_pool(&PlayerArchetype::Uniform, 100);
        assert_eq!(players.len(), 100);

        // Should see a range of handicaps
        let mut min_handicap = 30;
        let mut max_handicap = 0;
        for player in &players {
            min_handicap = min_handicap.min(player.handicap);
            max_handicap = max_handicap.max(player.handicap);
        }

        assert!(max_handicap - min_handicap > 10, "Uniform should have wide spread");
    }

    #[test]
    fn test_generate_player_pool_bell_curve() {
        let players = generate_player_pool(
            &PlayerArchetype::BellCurve { mean: 15, std_dev: 3.0 },
            100
        );
        assert_eq!(players.len(), 100);

        // Calculate mean handicap
        let mean: f64 = players.iter().map(|p| p.handicap as f64).sum::<f64>() / 100.0;

        // Should be roughly centered at 15 (within a reasonable tolerance)
        assert!((mean - 15.0).abs() < 3.0, "Mean handicap should be near 15, got {}", mean);
    }

    #[test]
    fn test_generate_player_pool_skewed_high() {
        let players = generate_player_pool(&PlayerArchetype::SkewedHigh, 100);
        assert_eq!(players.len(), 100);

        // Mean should be above 15 (skewed toward high handicaps)
        let mean: f64 = players.iter().map(|p| p.handicap as f64).sum::<f64>() / 100.0;
        assert!(mean > 15.0, "SkewedHigh should have mean > 15, got {}", mean);
    }

    #[test]
    fn test_generate_player_pool_skewed_low() {
        let players = generate_player_pool(&PlayerArchetype::SkewedLow, 100);
        assert_eq!(players.len(), 100);

        // Mean should be below 15 (skewed toward low handicaps)
        let mean: f64 = players.iter().map(|p| p.handicap as f64).sum::<f64>() / 100.0;
        assert!(mean < 15.0, "SkewedLow should have mean < 15, got {}", mean);
    }

    #[test]
    fn test_run_venue_simulation_basic() {
        let config = VenueConfig {
            num_bays: 2,
            hours: 1.0,
            shots_per_hour: 10,
            player_archetype: PlayerArchetype::Uniform,
            wager_range: (5.0, 10.0),
        };

        let result = run_venue_simulation(config);

        assert_eq!(result.total_shots, 20); // 2 bays * 1 hour * 10 shots/hour
        assert!(result.total_wagered > 0.0);
        assert!(result.net_profit != 0.0);
        // Hold percentage can be negative (player wins) or positive (house wins)
        assert!(result.hold_percentage > -1.0 && result.hold_percentage < 1.0);
    }

    #[test]
    fn test_build_payout_distribution() {
        use crate::models::shot::ShotOutcome;

        let shots = vec![
            ShotOutcome {
                miss_distance_ft: 5.0,
                multiplier: 0.0,
                payout: 0.0,
                wager: 10.0,
                hole_id: 1,
                is_fat_tail: false,
            },
            ShotOutcome {
                miss_distance_ft: 2.0,
                multiplier: 5.5,
                payout: 55.0,
                wager: 10.0,
                hole_id: 1,
                is_fat_tail: false,
            },
            ShotOutcome {
                miss_distance_ft: 1.0,
                multiplier: 12.0,
                payout: 120.0,
                wager: 10.0,
                hole_id: 1,
                is_fat_tail: false,
            },
        ];

        let dist = build_payout_distribution(&shots);

        assert_eq!(dist[0], 1); // 0x multiplier
        assert_eq!(dist[5], 1); // 5x multiplier (5.5 floors to 5)
        assert_eq!(dist[10], 1); // 10x+ multiplier (12.0)
    }

    #[test]
    fn test_venue_result_profit_over_time() {
        let config = VenueConfig {
            num_bays: 5,
            hours: 4.0,
            shots_per_hour: 20,
            player_archetype: PlayerArchetype::BellCurve { mean: 15, std_dev: 5.0 },
            wager_range: (5.0, 15.0),
        };

        let result = run_venue_simulation(config);

        // Should have 5 time points (0, 1, 2, 3, 4 hours)
        assert_eq!(result.profit_over_time.len(), 5);

        // First point should be 0
        assert_eq!(result.profit_over_time[0].1, 0.0);

        // Last point should equal net_profit
        assert!((result.profit_over_time[4].1 - result.net_profit).abs() < 0.01);
    }

    #[test]
    fn test_heatmap_structure() {
        let config = VenueConfig {
            num_bays: 3,
            hours: 1.0,
            shots_per_hour: 10,
            player_archetype: PlayerArchetype::Uniform,
            wager_range: (5.0, 10.0),
        };

        let result = run_venue_simulation(config);

        // Should have 6 handicap bins
        assert_eq!(result.heatmap_data.handicap_bins.len(), 6);

        // Should have 8 distance bins (one per hole)
        assert_eq!(result.heatmap_data.distance_bins.len(), 8);

        // Hold matrix should be 6x8
        assert_eq!(result.heatmap_data.hold_percentages.len(), 6);
        for row in &result.heatmap_data.hold_percentages {
            assert_eq!(row.len(), 8);
        }
    }
}