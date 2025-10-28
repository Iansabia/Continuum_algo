// Quick test to verify P_max updates after adding shots
use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::models::hole::get_hole_by_id;

fn main() {
    println!("Testing P_max updates with MCMC...\n");

    let mut player = Player::new("test_player".to_string(), 15);
    let hole = get_hole_by_id(4).unwrap(); // 150 yard hole

    // Get initial P_max (based on handicap)
    let p_max_initial = player.calculate_p_max(hole);
    println!("Initial P_max (handicap 15): {:.2}x", p_max_initial);
    println!("Initial cached sigma: {:.2}ft\n", player.get_current_sigma(hole));

    // Add first shot
    println!("Adding shot 1: 25.0ft miss distance");
    player.add_shot_to_batch(hole, 25.0, 10.0);
    let p_max_after_1 = player.calculate_p_max(hole);
    println!("P_max after shot 1: {:.2}x", p_max_after_1);
    println!("Cached sigma after shot 1: {:.2}ft\n", player.get_current_sigma(hole));

    // Add second shot
    println!("Adding shot 2: 28.0ft miss distance");
    player.add_shot_to_batch(hole, 28.0, 10.0);
    let p_max_after_2 = player.calculate_p_max(hole);
    println!("P_max after shot 2: {:.2}x", p_max_after_2);
    println!("Cached sigma after shot 2: {:.2}ft\n", player.get_current_sigma(hole));

    // Add third shot
    println!("Adding shot 3: 22.0ft miss distance");
    player.add_shot_to_batch(hole, 22.0, 10.0);
    let p_max_after_3 = player.calculate_p_max(hole);
    println!("P_max after shot 3: {:.2}x", p_max_after_3);
    println!("Cached sigma after shot 3: {:.2}ft\n", player.get_current_sigma(hole));

    // Verify that P_max changed
    if (p_max_after_3 - p_max_initial).abs() > 0.1 {
        println!("✅ SUCCESS: P_max updated from {:.2}x to {:.2}x", p_max_initial, p_max_after_3);
        println!("   Change: {:.2}x ({:.1}%)", p_max_after_3 - p_max_initial,
                 ((p_max_after_3 - p_max_initial) / p_max_initial) * 100.0);
    } else {
        println!("❌ FAILURE: P_max did not update (stayed at {:.2}x)", p_max_initial);
    }
}
