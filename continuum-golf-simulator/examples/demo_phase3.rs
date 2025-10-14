//! Phase 3 Demo: Simulation Engines
//!
//! This example demonstrates the three main simulation engines:
//! 1. Player Session Simulator - Individual player gaming sessions
//! 2. Venue Economics Simulator - Full venue operations with parallel processing
//! 3. Tournament Simulator - Competitive tournaments with multiple game modes

use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::simulators::player_session::{
    run_session, HoleSelection, SessionConfig,
};
use continuum_golf_simulator::simulators::venue::{
    run_venue_simulation, PlayerArchetype, VenueConfig,
};
use continuum_golf_simulator::simulators::tournament::{
    run_tournament, GameMode, PayoutStructure, TournamentConfig,
};

fn main() {
    println!("{}", "=".repeat(80));
    println!("CONTINUUM GOLF SIMULATOR - PHASE 3 DEMONSTRATION");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Player Session Simulator
    demo_player_session();
    println!();

    // Demo 2: Venue Economics Simulator
    demo_venue_simulation();
    println!();

    // Demo 3: Tournament Simulator
    demo_tournament();
    println!();

    println!("{}", "=".repeat(80));
    println!("PHASE 3 COMPLETE - ALL SIMULATION ENGINES OPERATIONAL");
    println!("{}", "=".repeat(80));
}

fn demo_player_session() {
    println!("📊 DEMO 1: Player Session Simulator");
    println!("{}", "-".repeat(80));

    // Create a player with handicap 15
    let mut player = Player::new("John Doe".to_string(), 15);

    // Configure a 50-shot session
    let config = SessionConfig {
        num_shots: 50,
        wager_min: 5.0,
        wager_max: 15.0,
        hole_selection: HoleSelection::Random,
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    println!("Player: {} (Handicap: {})", player.id, player.handicap);
    println!("Session: {} shots, ${}-${} wager range", config.num_shots, config.wager_min, config.wager_max);
    println!();

    // Run the simulation
    let result = run_session(&mut player, config);

    // Display results
    println!("Results:");
    println!("  Total Wagered:     ${:.2}", result.total_wagered);
    println!("  Total Won:         ${:.2}", result.total_won);
    println!("  Net Gain/Loss:     ${:.2}", result.net_gain_loss);
    println!("  House Edge:        {:.2}%", result.house_edge_percent());
    println!("  Win Rate:          {:.1}%", result.win_rate());
    println!("  Kalman Updates:    {}", result.num_kalman_updates);
    println!("  High-Stakes Shots: {}", result.num_high_stakes_shots);
    println!();

    println!("Final Skill Profiles:");
    for (category, sigma) in &result.final_skill_profiles {
        println!("  {}: σ = {:.2} ft", category, sigma);
    }
}

fn demo_venue_simulation() {
    println!("🏢 DEMO 2: Venue Economics Simulator");
    println!("{}", "-".repeat(80));

    // Configure venue simulation
    let config = VenueConfig {
        num_bays: 10,
        hours: 4.0,
        shots_per_hour: 50,
        player_archetype: PlayerArchetype::BellCurve {
            mean: 15,
            std_dev: 5.0,
        },
        wager_range: (5.0, 20.0),
    };

    println!("Venue: {} bays, {:.1} hours operation", config.num_bays, config.hours);
    println!("Players: Bell curve distribution (mean={}, σ=5.0)", 15);
    println!("Expected shots: {}", config.num_bays * config.shots_per_hour * config.hours as usize);
    println!();

    // Run parallel simulation
    println!("Running parallel simulation...");
    let result = run_venue_simulation(config);

    // Display results
    println!();
    println!("Venue Results:");
    println!("  Total Shots:       {}", result.total_shots);
    println!("  Total Wagered:     ${:.2}", result.total_wagered);
    println!("  Total Payouts:     ${:.2}", result.total_payouts);
    println!("  Net Profit:        ${:.2}", result.net_profit);
    println!("  Hold Percentage:   {:.2}%", result.hold_percentage * 100.0);
    println!();

    println!("Payout Distribution:");
    for (i, count) in result.payout_distribution.iter().enumerate() {
        if *count > 0 {
            let label = if i == 10 { "10x+".to_string() } else { format!("{}x", i) };
            println!("  {}: {} shots ({:.1}%)",
                label, count, (*count as f64 / result.total_shots as f64) * 100.0);
        }
    }
    println!();

    println!("Heatmap Structure:");
    println!("  Handicap Bins: {}", result.heatmap_data.handicap_bins.len());
    println!("  Distance Bins: {}", result.heatmap_data.distance_bins.len());
    println!("  Matrix Size: {}x{}",
        result.heatmap_data.hold_percentages.len(),
        result.heatmap_data.hold_percentages[0].len()
    );
}

fn demo_tournament() {
    println!("🏆 DEMO 3: Tournament Simulator");
    println!("{}", "-".repeat(80));

    // Configure tournament
    let config = TournamentConfig {
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
    };

    println!("Tournament: Closest to Pin (Hole 4 - 150 yds)");
    println!("Players: {}", config.num_players);
    println!("Entry Fee: ${:.2}", config.entry_fee);
    println!("House Rake: {:.0}%", config.house_rake_percent * 100.0);
    println!();

    // Run tournament
    println!("Running tournament...");
    let result = run_tournament(config);

    // Display results
    println!();
    println!("Tournament Results:");
    println!("  Prize Pool:        ${:.2} (after ${:.2} rake)",
        result.prize_pool, result.house_rake);
    println!();

    println!("Top 10 Leaderboard:");
    for (i, (player_id, score)) in result.leaderboard.iter().take(10).enumerate() {
        let rank = i + 1;
        let prize = result.payouts.iter()
            .find(|(id, _)| id == player_id)
            .map(|(_, amt)| format!(" - ${:.2}", amt))
            .unwrap_or_default();
        println!("  {:2}. {} - {:.2} ft{}", rank, player_id, score, prize);
    }
    println!();

    println!("Prize Distribution:");
    let total_paid: f64 = result.payouts.iter().map(|(_, amt)| amt).sum();
    for (player_id, amount) in &result.payouts {
        println!("  {}: ${:.2}", player_id, amount);
    }
    println!("  Total Paid: ${:.2}", total_paid);
}
