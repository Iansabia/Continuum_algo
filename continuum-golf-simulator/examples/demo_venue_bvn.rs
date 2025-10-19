//! Venue Economics Simulation with BVN (Bivariate Normal) Mode
//!
//! This demo shows how enabling BVN mode affects:
//! - Venue hold percentage and profitability
//! - Player win rates and outcomes
//! - Comparison between 1D Rayleigh and 2D BVN modes
//!
//! Run: cargo run --example demo_venue_bvn

use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::simulators::player_session::{
    run_session, HoleSelection, SessionConfig,
};
use rand::Rng;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Venue Economics: 1D Rayleigh vs 2D BVN Comparison       ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    println!("This simulation compares venue profitability and player");
    println!("outcomes between traditional 1D Rayleigh mode and new 2D BVN mode.\n");

    // Configuration
    let num_players = 10;
    let shots_per_player = 100;
    let wager_min = 5.0;
    let wager_max = 20.0;

    println!("Simulation Parameters:");
    println!("  Players: {}", num_players);
    println!("  Shots per player: {}", shots_per_player);
    println!("  Wager range: ${:.2} - ${:.2}\n", wager_min, wager_max);

    println!("═══════════════════════════════════════════════════════════\n");

    // Run Scenario 1: Traditional 1D Rayleigh Mode (default)
    println!("📊 SCENARIO 1: Traditional 1D Rayleigh Mode\n");
    let results_1d = run_venue_with_mode(
        num_players,
        shots_per_player,
        wager_min,
        wager_max,
        false, // Don't enable BVN
    );
    print_results("1D Rayleigh", &results_1d);

    println!("\n═══════════════════════════════════════════════════════════\n");

    // Run Scenario 2: BVN Mode with systematic bias
    println!("📊 SCENARIO 2: BVN Mode (with systematic player bias)\n");
    let results_bvn = run_venue_with_mode(
        num_players,
        shots_per_player,
        wager_min,
        wager_max,
        true, // Enable BVN
    );
    print_results("2D BVN", &results_bvn);

    println!("\n═══════════════════════════════════════════════════════════\n");

    // Comparison
    println!("📈 COMPARATIVE ANALYSIS\n");

    println!("Venue Profitability:");
    let profit_diff = results_bvn.venue_profit - results_1d.venue_profit;
    let profit_diff_pct = (profit_diff / results_1d.venue_profit.abs()) * 100.0;
    println!("  1D Rayleigh: ${:.2}", results_1d.venue_profit);
    println!("  2D BVN:      ${:.2}", results_bvn.venue_profit);
    println!("  Difference:  ${:+.2} ({:+.1}%)\n", profit_diff, profit_diff_pct);

    println!("Hold Percentage:");
    let hold_diff = results_bvn.hold_percentage - results_1d.hold_percentage;
    println!("  1D Rayleigh: {:.2}%", results_1d.hold_percentage * 100.0);
    println!("  2D BVN:      {:.2}%", results_bvn.hold_percentage * 100.0);
    println!("  Difference:  {:+.2}%\n", hold_diff * 100.0);

    println!("Player Outcomes:");
    let player_diff = results_bvn.avg_player_profit - results_1d.avg_player_profit;
    println!("  1D Rayleigh: ${:.2} avg profit/loss per player", results_1d.avg_player_profit);
    println!("  2D BVN:      ${:.2} avg profit/loss per player", results_bvn.avg_player_profit);
    println!("  Difference:  ${:+.2}\n", player_diff);

    println!("Total Volume:");
    println!("  1D Rayleigh: ${:.2} wagered", results_1d.total_wagered);
    println!("  2D BVN:      ${:.2} wagered", results_bvn.total_wagered);

    println!("\n═══════════════════════════════════════════════════════════\n");

    println!("Key Insights:\n");
    println!("1. BVN mode accounts for systematic player bias (μ_x, μ_y)");
    println!("   which Rayleigh mode cannot detect.");
    println!();
    println!("2. Players with lateral/distance bias get more accurate");
    println!("   P_max calculations, leading to fairer payouts.");
    println!();
    println!("3. Venue hold percentage may vary as BVN adapts to");
    println!("   elliptical shot patterns (σ_x ≠ σ_y).");
    println!();
    println!("4. Long-term convergence: BVN improves as Kalman filter");
    println!("   learns each player's true 4D state [μ_x, μ_y, σ_x, σ_y].");

    println!("\n═══════════════════════════════════════════════════════════\n");
}

#[derive(Debug)]
struct VenueResults {
    total_wagered: f64,
    total_payouts: f64,
    venue_profit: f64,
    hold_percentage: f64,
    avg_player_profit: f64,
    total_shots: usize,
    num_kalman_updates: usize,
}

fn run_venue_with_mode(
    num_players: usize,
    shots_per_player: usize,
    wager_min: f64,
    wager_max: f64,
    enable_bvn: bool,
) -> VenueResults {
    let mut total_wagered = 0.0;
    let mut total_payouts = 0.0;
    let mut total_shots = 0;
    let mut total_kalman_updates = 0;
    let mut player_profits = Vec::new();

    let mut rng = rand::thread_rng();

    // Generate player population (handicaps 10-20)
    for player_num in 0..num_players {
        let handicap = rng.gen_range(10..=20);
        let mut player = Player::new(format!("player_{}", player_num), handicap);

        // Enable BVN mode if requested
        if enable_bvn {
            // Start with neutral initial estimate
            // The 4D Kalman filter will learn each player's true
            // bias [μ_x, μ_y] and dispersion [σ_x, σ_y] over time
            player.enable_bvn_mode_all(30.0);
        }

        // Run player session
        let config = SessionConfig {
            num_shots: shots_per_player,
            wager_min,
            wager_max,
            hole_selection: HoleSelection::Random,
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };

        let result = run_session(&mut player, config);

        total_wagered += result.total_wagered;
        total_payouts += result.total_won;
        total_shots += result.shots.len();
        total_kalman_updates += result.num_kalman_updates;
        player_profits.push(result.net_gain_loss);
    }

    let venue_profit = total_wagered - total_payouts;
    let hold_percentage = if total_wagered > 0.0 {
        venue_profit / total_wagered
    } else {
        0.0
    };
    let avg_player_profit = player_profits.iter().sum::<f64>() / num_players as f64;

    VenueResults {
        total_wagered,
        total_payouts,
        venue_profit,
        hold_percentage,
        avg_player_profit,
        total_shots,
        num_kalman_updates: total_kalman_updates,
    }
}

fn print_results(mode_name: &str, results: &VenueResults) {
    println!("Mode: {}\n", mode_name);
    println!("  Total Shots:       {}", results.total_shots);
    println!("  Total Wagered:     ${:.2}", results.total_wagered);
    println!("  Total Payouts:     ${:.2}", results.total_payouts);
    println!();
    println!("  Venue Profit:      ${:.2}", results.venue_profit);
    println!("  Hold Percentage:   {:.2}%", results.hold_percentage * 100.0);
    println!();
    println!("  Avg Player P/L:    ${:.2}", results.avg_player_profit);
    println!("  Kalman Updates:    {}", results.num_kalman_updates);
}
