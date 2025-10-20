//! Debug P_max calculations for BVN mode
//!
//! This script helps identify why BVN P_max might be too low

use continuum_golf_simulator::models::{
    hole::HOLE_CONFIGURATIONS,
    player::Player,
};

fn main() {
    println!("═══ Debugging BVN P_max Calculations ═══\n");

    let hole = &HOLE_CONFIGURATIONS[3]; // 150 yds Mid-Iron
    println!("Hole: {} ({} yds)", hole.id, hole.distance_yds);
    println!("  d_max = {:.1} ft", hole.d_max_ft);
    println!("  k = {:.1}", hole.k);
    println!("  RTP = {:.1}%\n", hole.rtp * 100.0);

    let mut player = Player::new("test_player".to_string(), 15);

    // Test 1: Compare 1D vs 2D with same σ (symmetric, no bias)
    println!("TEST 1: Symmetric distribution, no bias");
    println!("  Parameters: μ_x=0, μ_y=0, σ_x=25, σ_y=25\n");

    let sigma_1d = 25.0;
    let p_max_1d = player.calculate_p_max(hole);
    println!("  1D P_max (σ=25): {:.2}×", p_max_1d);

    player.enable_bvn_mode_all(25.0);
    let p_max_2d = if let Some((mu_x, mu_y, sigma_x, sigma_y)) = player.get_bvn_state(hole.category) {
        println!("  2D state: μ_x={:.1}, μ_y={:.1}, σ_x={:.1}, σ_y={:.1}", mu_x, mu_y, sigma_x, sigma_y);
        player.calculate_p_max_bvn(hole, mu_x, mu_y, sigma_x, sigma_y)
    } else {
        0.0
    };
    println!("  2D P_max (BVN): {:.2}×", p_max_2d);
    println!("  Ratio (2D/1D): {:.2}\n", p_max_2d / p_max_1d);

    // Test 2: Very tight dispersion (should have high P_max)
    println!("TEST 2: Tight dispersion");
    println!("  Parameters: μ_x=0, μ_y=0, σ_x=5, σ_y=5\n");

    let p_max_bvn_tight = player.calculate_p_max_bvn(hole, 0.0, 0.0, 5.0, 5.0);
    println!("  2D P_max: {:.2}×", p_max_bvn_tight);
    println!("  Expected: Very high (player very accurate)\n");

    // Test 3: Wide dispersion (should have low P_max)
    println!("TEST 3: Wide dispersion");
    println!("  Parameters: μ_x=0, μ_y=0, σ_x=50, σ_y=50\n");

    let p_max_bvn_wide = player.calculate_p_max_bvn(hole, 0.0, 0.0, 50.0, 50.0);
    println!("  2D P_max: {:.2}×", p_max_bvn_wide);
    println!("  Expected: Low (player inaccurate)\n");

    // Test 4: With bias (shifted mean)
    println!("TEST 4: With bias");
    println!("  Parameters: μ_x=10, μ_y=-5, σ_x=20, σ_y=20\n");

    let p_max_bvn_bias = player.calculate_p_max_bvn(hole, 10.0, -5.0, 20.0, 20.0);
    println!("  2D P_max: {:.2}×", p_max_bvn_bias);
    println!("  Expected: Lower than centered (bias reduces hit probability)\n");

    // Test 5: Elliptical (different σ_x and σ_y)
    println!("TEST 5: Elliptical dispersion");
    println!("  Parameters: μ_x=0, μ_y=0, σ_x=15, σ_y=35\n");

    let p_max_bvn_ellipse = player.calculate_p_max_bvn(hole, 0.0, 0.0, 15.0, 35.0);
    println!("  2D P_max: {:.2}×", p_max_bvn_ellipse);
    println!("  Expected: Between tight and wide\n");

    // Test 6: VERY tight (σ=1) - almost perfect player
    println!("TEST 6: VERY tight dispersion (σ=1)");
    println!("  Parameters: μ_x=0, μ_y=0, σ_x=1, σ_y=1\n");

    let p_max_bvn_perfect = player.calculate_p_max_bvn(hole, 0.0, 0.0, 1.0, 1.0);
    println!("  2D P_max: {:.2}×", p_max_bvn_perfect);
    println!("  Expected: EXTREMELY high (near-perfect player)\n");

    println!("═══════════════════════════════════════\n");
    println!("Summary:");
    println!("  If P_max values are unexpectedly low (< 1.0), there's a bug.");
    println!("  If 1D and 2D differ significantly for symmetric case, there's inconsistency.");
}
