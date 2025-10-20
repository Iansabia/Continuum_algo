//! Venue Economics with Proper BVN Integration
//!
//! This demo properly integrates BVN mode by:
//! - Generating 2D (x,y) shot coordinates for BVN players
//! - Using calculate_p_max_bvn() with learned 4D state
//! - Storing 2D coordinates in shot outcomes
//! - Comparing actual 1D vs 2D simulation modes
//!
//! Run: cargo run --example demo_venue_bvn_proper

use continuum_golf_simulator::models::{
    hole::{Hole, HOLE_CONFIGURATIONS, ClubCategory},
    player::Player,
    shot::ShotOutcome,
};
use continuum_golf_simulator::math::distributions::{rayleigh_random, bvn_random, fat_tail_shot};
use rand::Rng;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Venue Economics: Proper 1D vs 2D BVN Comparison         ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let num_players = 10;
    let shots_per_player = 200;
    let wager_min = 5.0;
    let wager_max = 20.0;

    println!("Simulation Parameters:");
    println!("  Players: {}", num_players);
    println!("  Shots per player: {}", shots_per_player);
    println!("  Wager range: ${:.2} - ${:.2}\n", wager_min, wager_max);

    println!("═══════════════════════════════════════════════════════════\n");

    // Scenario 1: Pure 1D Rayleigh Mode
    println!("📊 SCENARIO 1: Pure 1D Rayleigh Mode\n");
    let results_1d = run_venue_1d_mode(num_players, shots_per_player, wager_min, wager_max);
    print_results("1D Rayleigh (Pure)", &results_1d);

    println!("\n═══════════════════════════════════════════════════════════\n");

    // Scenario 2: Pure 2D BVN Mode
    println!("📊 SCENARIO 2: Pure 2D BVN Mode (with player bias)\n");
    let results_bvn = run_venue_2d_mode(num_players, shots_per_player, wager_min, wager_max);
    print_results("2D BVN (Pure)", &results_bvn);

    println!("\n═══════════════════════════════════════════════════════════\n");

    // Comparison
    println!("📈 COMPARATIVE ANALYSIS\n");

    println!("Venue Profitability:");
    let profit_diff = results_bvn.venue_profit - results_1d.venue_profit;
    let profit_diff_pct = if results_1d.venue_profit.abs() > 0.0 {
        (profit_diff / results_1d.venue_profit.abs()) * 100.0
    } else {
        0.0
    };
    println!("  1D Rayleigh: ${:.2}", results_1d.venue_profit);
    println!("  2D BVN:      ${:.2}", results_bvn.venue_profit);
    println!("  Difference:  ${:+.2} ({:+.1}%)\n", profit_diff, profit_diff_pct);

    println!("Hold Percentage:");
    let hold_diff = results_bvn.hold_percentage - results_1d.hold_percentage;
    println!("  1D Rayleigh: {:.2}%", results_1d.hold_percentage * 100.0);
    println!("  2D BVN:      {:.2}%", results_bvn.hold_percentage * 100.0);
    println!("  Difference:  {:+.2}%", hold_diff * 100.0);
    println!("  Target:      ~15% (85% RTP)\n");

    println!("Player Outcomes:");
    let player_diff = results_bvn.avg_player_profit - results_1d.avg_player_profit;
    println!("  1D Rayleigh: ${:.2} avg per player", results_1d.avg_player_profit);
    println!("  2D BVN:      ${:.2} avg per player", results_bvn.avg_player_profit);
    println!("  Difference:  ${:+.2}\n", player_diff);

    println!("Kalman Learning:");
    println!("  1D Updates:  {}", results_1d.num_kalman_updates);
    println!("  4D Updates:  {} (learns bias + dispersion)", results_bvn.num_kalman_updates);

    println!("\n═══════════════════════════════════════════════════════════\n");

    println!("Key Insights:\n");
    println!("1. Both modes should converge to ~15% hold (85% RTP) over");
    println!("   sufficient sample size (thousands of shots).");
    println!();
    println!("2. BVN mode provides fairer P_max by detecting systematic");
    println!("   player bias that Rayleigh mode ignores.");
    println!();
    println!("3. Short-term variance is expected; long-term results");
    println!("   should be similar with proper calibration.");
    println!();
    println!("4. BVN advantage: Actionable coaching insights from");
    println!("   learned bias patterns [μ_x, μ_y, σ_x, σ_y].");

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

/// Run venue simulation in pure 1D Rayleigh mode
fn run_venue_1d_mode(
    num_players: usize,
    shots_per_player: usize,
    wager_min: f64,
    wager_max: f64,
) -> VenueResults {
    let mut total_wagered = 0.0;
    let mut total_payouts = 0.0;
    let mut total_shots = 0;
    let mut total_kalman_updates = 0;
    let mut player_profits = Vec::new();
    let mut rng = rand::thread_rng();

    for player_num in 0..num_players {
        let handicap = rng.gen_range(10..=20);
        let mut player = Player::new(format!("player_{}", player_num), handicap);

        let mut player_wagered = 0.0;
        let mut player_won = 0.0;

        for _shot_num in 0..shots_per_player {
            // Random hole selection
            let hole = &HOLE_CONFIGURATIONS[rng.gen_range(0..8)];
            let wager = rng.gen_range(wager_min..=wager_max);

            // Get current 1D skill
            let skill_profile = player.get_skill_for_hole(hole);
            let sigma = skill_profile.kalman_filter.estimate;

            // Calculate 1D P_max
            let p_max = player.calculate_p_max(hole);

            // Generate 1D shot (radial miss distance)
            let (miss_distance, is_fat_tail) = fat_tail_shot(sigma, 0.02, 3.0);

            // Calculate payout
            let multiplier = hole.calculate_payout(miss_distance, p_max);
            let payout = multiplier * wager;

            player_wagered += wager;
            player_won += payout;

            // Add to batch and update
            let batch_full = player.add_shot_to_batch(hole, miss_distance, wager);
            if batch_full {
                player.update_skill(hole, p_max);
                total_kalman_updates += 1;
            }
        }

        total_wagered += player_wagered;
        total_payouts += player_won;
        total_shots += shots_per_player;
        player_profits.push(player_won - player_wagered);
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

/// Run venue simulation in pure 2D BVN mode
fn run_venue_2d_mode(
    num_players: usize,
    shots_per_player: usize,
    wager_min: f64,
    wager_max: f64,
) -> VenueResults {
    let mut total_wagered = 0.0;
    let mut total_payouts = 0.0;
    let mut total_shots = 0;
    let mut total_kalman_updates = 0;
    let mut player_profits = Vec::new();
    let mut rng = rand::thread_rng();

    for player_num in 0..num_players {
        let handicap = rng.gen_range(10..=20);
        let mut player = Player::new(format!("player_{}", player_num), handicap);

        // Enable BVN mode with neutral initial guess
        player.enable_bvn_mode_all(30.0);

        // Give each player unique true bias characteristics
        // (These are hidden from the system - Kalman will learn them)
        let true_mu_x = rng.gen_range(-5.0..5.0);
        let true_mu_y = rng.gen_range(-5.0..5.0);
        let true_sigma_x = rng.gen_range(15.0..35.0);
        let true_sigma_y = rng.gen_range(15.0..35.0);

        let mut player_wagered = 0.0;
        let mut player_won = 0.0;

        for _shot_num in 0..shots_per_player {
            // Random hole selection
            let hole = &HOLE_CONFIGURATIONS[rng.gen_range(0..8)];
            let wager = rng.gen_range(wager_min..=wager_max);

            // Calculate 2D P_max using current learned BVN state
            let p_max = if let Some((mu_x, mu_y, sigma_x, sigma_y)) = player.get_bvn_state(hole.category) {
                player.calculate_p_max_bvn(hole, mu_x, mu_y, sigma_x, sigma_y)
            } else {
                // Fallback (shouldn't happen)
                player.calculate_p_max(hole)
            };

            // Generate 2D shot from TRUE player characteristics
            let (mut x_ft, mut y_ft) = bvn_random(true_mu_x, true_mu_y, true_sigma_x, true_sigma_y);

            // Apply fat-tail events (2% chance of 3× worse)
            let is_fat_tail = rng.gen::<f64>() < 0.02;
            if is_fat_tail {
                x_ft *= 3.0;
                y_ft *= 3.0;
            }

            let miss_distance = (x_ft * x_ft + y_ft * y_ft).sqrt();

            // Calculate payout
            let multiplier = hole.calculate_payout(miss_distance, p_max);
            let payout = multiplier * wager;

            player_wagered += wager;
            player_won += payout;

            // Add to batch and update (with 2D coordinates)
            let batch_full = player.add_shot_to_batch_2d(hole, x_ft, y_ft, wager);
            if batch_full {
                player.update_skill(hole, p_max);
                total_kalman_updates += 1;
            }
        }

        total_wagered += player_wagered;
        total_payouts += player_won;
        total_shots += shots_per_player;
        player_profits.push(player_won - player_wagered);
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
