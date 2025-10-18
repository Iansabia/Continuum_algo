// Test to verify if Kalman updates affect hold percentage
use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::simulators::player_session::{SessionConfig, HoleSelection, DeveloperMode, run_session};

fn main() {
    println!("Testing hold percentage WITH and WITHOUT Kalman updates");
    println!("{}", "=".repeat(70));
    println!();

    for handicap in [5, 10, 20] {
        println!("Handicap {}", handicap);
        println!("{}", "-".repeat(70));

        // Test WITH Kalman updates (normal mode)
        let mut player_with_kalman = Player::new(format!("player_hc{}", handicap), handicap);
        let config_with_kalman = SessionConfig {
            num_shots: 10000,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Random,
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: None,
                disable_kalman: false, // Normal mode
            }),
            ..Default::default()
        };

        let result_with = run_session(&mut player_with_kalman, config_with_kalman);
        let hold_with = (1.0 - result_with.total_won / result_with.total_wagered) * 100.0;

        // Test WITHOUT Kalman updates
        let mut player_without_kalman = Player::new(format!("player_hc{}_no_kalman", handicap), handicap);
        let config_without_kalman = SessionConfig {
            num_shots: 10000,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Random,
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: None,
                disable_kalman: true, // No updates
            }),
            ..Default::default()
        };

        let result_without = run_session(&mut player_without_kalman, config_without_kalman);
        let hold_without = (1.0 - result_without.total_won / result_without.total_wagered) * 100.0;

        println!("  With Kalman updates:");
        println!("    Total wagered:  ${:.2}", result_with.total_wagered);
        println!("    Total won:      ${:.2}", result_with.total_won);
        println!("    Hold %:         {:.2}%", hold_with);
        println!("    Kalman updates: {}", result_with.num_kalman_updates);
        println!();

        println!("  Without Kalman updates:");
        println!("    Total wagered:  ${:.2}", result_without.total_wagered);
        println!("    Total won:      ${:.2}", result_without.total_won);
        println!("    Hold %:         {:.2}%", hold_without);
        println!("    Kalman updates: {}", result_without.num_kalman_updates);
        println!();

        println!("  Difference:       {:.2}%", hold_with - hold_without);
        println!();
    }
}
