/// Phase 4 Demo: Analytics & Validation

use continuum_golf_simulator::models::{player::Player, hole::get_hole_by_id};
use continuum_golf_simulator::simulators::player_session::{SessionConfig, run_session, HoleSelection};
use continuum_golf_simulator::simulators::venue::{VenueConfig, run_venue_simulation, PlayerArchetype};
use continuum_golf_simulator::analytics::{
    calculate_expected_value,
    validate_rtp_across_skills,
    calculate_fairness_metric,
    analyze_kalman_convergence,
    export_session_csv,
    export_venue_json,
    export_heatmap_csv,
    export_pmax_history,
};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║      CONTINUUM GOLF SIMULATOR - PHASE 4 DEMONSTRATION         ║");
    println!("║           Analytics & Validation Capabilities                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // SECTION 1: Expected Value
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 SECTION 1: Expected Value Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let hole4 = get_hole_by_id(4).unwrap();
    let player_hcp15 = Player::new("player_15".to_string(), 15);
    
    let ev = calculate_expected_value(&player_hcp15, &hole4, 10.0, 10000);
    let theoretical_ev = 10.0 * (hole4.rtp - 1.0);
    
    println!("Monte Carlo EV (10,000 trials): ${:.2}", ev);
    println!("Theoretical EV (RTP formula):   ${:.2}", theoretical_ev);
    println!("House Edge:                     {:.2}%\n", (1.0 - hole4.rtp) * 100.0);

    // SECTION 2: RTP Validation
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 SECTION 2: RTP Validation Across Handicaps");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let hole1 = get_hole_by_id(1).unwrap();
    let rtp_results = validate_rtp_across_skills(&hole1, vec![0, 10, 20, 30], 5000);
    
    println!("┌──────────┬─────────────┬─────────────┬───────────────┐");
    println!("│ Handicap │ Actual RTP  │ Target RTP  │ Deviation     │");
    println!("├──────────┼─────────────┼─────────────┼───────────────┤");
    for result in &rtp_results {
        println!(
            "│ {:^8} │ {:^11.4} │ {:^11.4} │ {:>11.2}% │",
            result.handicap, result.actual_rtp, result.target_rtp, result.deviation_percent
        );
    }
    println!("└──────────┴─────────────┴─────────────┴───────────────┘\n");

    // SECTION 3: Fairness
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚖️  SECTION 3: Fairness Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let hole6 = get_hole_by_id(6).unwrap();
    let fairness_report = calculate_fairness_metric(&hole6, vec![0, 10, 20, 30], 5000);
    
    println!("┌──────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Handicap │     EV      │   P_max     │   Sigma     │");
    println!("├──────────┼─────────────┼─────────────┼─────────────┤");
    for comp in &fairness_report.comparisons {
        println!(
            "│ {:^8} │ ${:>9.2} │ {:>9.2}× │ {:>9.2} ft │",
            comp.handicap, comp.expected_value, comp.p_max, comp.skill_sigma
        );
    }
    println!("└──────────┴─────────────┴─────────────┴─────────────┘");
    println!("Max EV Difference: ${:.2}", fairness_report.max_ev_difference);
    println!("Is Fair:           {}\n", if fairness_report.is_fair { "✓ YES" } else { "✗ NO" });

    // SECTION 4: Convergence
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔬 SECTION 4: Kalman Filter Convergence");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut player = Player::new("test_player".to_string(), 15);
    let config = SessionConfig {
        num_shots: 100,
        wager_min: 5.0,
        wager_max: 15.0,
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };
    
    let session_result = run_session(&mut player, config);
    let convergence_reports = analyze_kalman_convergence(&session_result);
    
    for (_category, report) in &convergence_reports {
        println!("Category: {}", report.club_category);
        println!("  Final Confidence: {:.2}%", report.final_confidence);
        println!("  Converged:        {}\n", if report.converged { "✓ YES" } else { "✗ NO" });
    }

    // SECTION 5: Export
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💾 SECTION 5: Data Export");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    export_session_csv(&session_result, "demo_session_phase4.csv").expect("CSV export failed");
    println!("  ✓ Session CSV exported");

    export_pmax_history(&player, "demo_pmax_history_phase4.csv").expect("P_max export failed");
    println!("  ✓ P_max history exported");

    let venue_config = VenueConfig {
        num_bays: 10,
        hours: 2.0,
        shots_per_hour: 100,
        player_archetype: PlayerArchetype::BellCurve { mean: 15, std_dev: 5.0 },
        wager_range: (5.0, 15.0),
    };
    let venue_result = run_venue_simulation(venue_config);
    
    export_venue_json(&venue_result, "demo_venue_phase4.json").expect("Venue JSON export failed");
    println!("  ✓ Venue JSON exported");

    export_heatmap_csv(&venue_result.heatmap_data, "demo_heatmap_phase4.csv").expect("Heatmap export failed");
    println!("  ✓ Heatmap CSV exported\n");
    
    println!("Venue: Total Wagered ${:.2}, Net Profit ${:.2}, Hold {:.2}%\n", 
        venue_result.total_wagered, venue_result.net_profit, venue_result.hold_percentage);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ PHASE 4 DEMONSTRATION COMPLETE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
