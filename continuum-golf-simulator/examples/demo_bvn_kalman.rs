/// Demonstration of BVN (Bivariate Normal) with 4D Kalman Filter
///
/// This demo shows the complete end-to-end workflow:
/// 1. Player starts with BVN mode enabled
/// 2. Player has systematic bias (tends to miss right and short)
/// 3. 4D Kalman filter learns [μ_x, μ_y, σ_x, σ_y] over shots
/// 4. P_max calculated using 2D BVN distribution
/// 5. Coaching insights from detected bias patterns
///
/// Run: cargo run --example demo_bvn_kalman

use continuum_golf_simulator::models::{
    hole::HOLE_CONFIGURATIONS,
    player::Player,
};
use continuum_golf_simulator::math::distributions::bvn_random;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  BVN + 4D Kalman Filter Demonstration                     ║");
    println!("║  Phase 9.1-9.4: Complete Integration                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Create a player with handicap 15
    let mut player = Player::new("demo_player".to_string(), 15);

    println!("Player: {} (Handicap: {})\n", player.id, player.handicap);

    // Use Hole 4 (150 yds, Mid-Iron)
    let hole = &HOLE_CONFIGURATIONS[3];
    println!("Hole: {} ({} yds, {:?})", hole.id, hole.distance_yds, hole.category);
    println!("Target: d_max={:.1} ft, k={:.1}, RTP={:.1}%\n",
        hole.d_max_ft, hole.k, hole.rtp * 100.0);

    // Enable BVN mode with initial guess (no bias, symmetric dispersion)
    let initial_sigma = 30.0; // About average for handicap 15 at 150yds
    player.enable_bvn_mode_all(initial_sigma);

    println!("✓ BVN mode enabled for all club categories");
    println!("  Initial state: μ_x=0, μ_y=0, σ_x={:.1}, σ_y={:.1}\n",
        initial_sigma, initial_sigma);

    // Simulate player with TRUE systematic bias
    // This player tends to:
    // - Miss 3 feet RIGHT on average (μ_x_true = 3.0)
    // - Miss 2 feet SHORT on average (μ_y_true = -2.0)
    // - Good lateral control (σ_x_true = 20.0)
    // - Poor distance control (σ_y_true = 35.0)
    let mu_x_true = 3.0;
    let mu_y_true = -2.0;
    let sigma_x_true = 20.0;
    let sigma_y_true = 35.0;

    println!("True player characteristics (unknown to system):");
    println!("  Lateral bias:  μ_x = {:.1} ft (tends right)", mu_x_true);
    println!("  Distance bias: μ_y = {:.1} ft (tends short)", mu_y_true);
    println!("  Lateral control:  σ_x = {:.1} ft (good)", sigma_x_true);
    println!("  Distance control: σ_y = {:.1} ft (poor)\n", sigma_y_true);

    println!("═══════════════════════════════════════════════════════════\n");
    println!("Simulating 25 shots and watching Kalman filter learn...\n");

    let wager = 10.0;

    for shot_num in 1..=25 {
        // Generate realistic (x,y) shot from true BVN distribution
        let (x_ft, y_ft) = bvn_random(mu_x_true, mu_y_true, sigma_x_true, sigma_y_true);

        // Add shot to batch (2D mode)
        let batch_full = player.add_shot_to_batch_2d(hole, x_ft, y_ft, wager);

        // Print shot details
        let miss_distance = (x_ft * x_ft + y_ft * y_ft).sqrt();
        print!("Shot {:2}: ({:6.2}, {:6.2}) → d={:6.2} ft", shot_num, x_ft, y_ft, miss_distance);

        // When batch is full (every 5 shots), update Kalman filter
        if batch_full {
            player.update_skill(hole, 0.0); // P_max unused in update

            // Get updated BVN state
            if let Some((mu_x, mu_y, sigma_x, sigma_y)) = player.get_bvn_state(hole.category) {
                println!(" → Kalman updated:");
                println!("         Learned: μ_x={:5.2}, μ_y={:5.2}, σ_x={:5.2}, σ_y={:5.2}",
                    mu_x, mu_y, sigma_x, sigma_y);

                // Calculate P_max using learned BVN distribution
                let p_max = player.calculate_p_max_bvn(hole, mu_x, mu_y, sigma_x, sigma_y, 0.0, None);
                println!("         P_max = {:.2}×", p_max);
            }
        } else {
            println!(" (batched)");
        }
    }

    println!("\n═══════════════════════════════════════════════════════════\n");
    println!("Final Kalman Filter State:\n");

    if let Some((mu_x, mu_y, sigma_x, sigma_y)) = player.get_bvn_state(hole.category) {
        println!("Learned Parameters:");
        println!("  μ_x = {:6.2} ft   (true: {:.1} ft)  → error: {:+.2} ft",
            mu_x, mu_x_true, mu_x - mu_x_true);
        println!("  μ_y = {:6.2} ft   (true: {:.1} ft)  → error: {:+.2} ft",
            mu_y, mu_y_true, mu_y - mu_y_true);
        println!("  σ_x = {:6.2} ft   (true: {:.1} ft)  → error: {:+.2} ft",
            sigma_x, sigma_x_true, sigma_x - sigma_x_true);
        println!("  σ_y = {:6.2} ft   (true: {:.1} ft)  → error: {:+.2} ft\n",
            sigma_y, sigma_y_true, sigma_y - sigma_y_true);

        // Calculate final P_max
        let p_max_learned = player.calculate_p_max_bvn(hole, mu_x, mu_y, sigma_x, sigma_y, 0.0, None);
        let p_max_true = player.calculate_p_max_bvn(hole, mu_x_true, mu_y_true, sigma_x_true, sigma_y_true, 0.0, None);

        println!("P_max Comparison:");
        println!("  Using learned params: {:.2}×", p_max_learned);
        println!("  Using true params:    {:.2}×", p_max_true);
        println!("  Difference:           {:+.2}×\n", p_max_learned - p_max_true);

        // Coaching insights
        println!("═══════════════════════════════════════════════════════════\n");
        println!("Coaching Insights (based on learned bias):\n");

        // Lateral bias analysis
        if mu_x.abs() > 2.0 {
            let direction = if mu_x > 0.0 { "RIGHT" } else { "LEFT" };
            println!("⚠ Systematic lateral bias detected!");
            println!("  → You consistently miss {:.1} ft {} of target", mu_x.abs(), direction);
            println!("  → Recommendation: Adjust aim {:.1} ft {} to compensate\n",
                mu_x.abs(), if mu_x > 0.0 { "left" } else { "right" });
        }

        // Distance bias analysis
        if mu_y.abs() > 2.0 {
            let tendency = if mu_y > 0.0 { "LONG" } else { "SHORT" };
            println!("⚠ Systematic distance bias detected!");
            println!("  → You consistently miss {:.1} ft {}", mu_y.abs(), tendency);
            println!("  → Recommendation: {} club selection\n",
                if mu_y > 0.0 { "Reduce" } else { "Increase" });
        }

        // Dispersion analysis
        if sigma_x > sigma_y * 1.3 {
            println!("📊 Dispersion pattern: Poor lateral control");
            println!("  → σ_x ({:.1} ft) >> σ_y ({:.1} ft)", sigma_x, sigma_y);
            println!("  → Focus on: Swing path consistency\n");
        } else if sigma_y > sigma_x * 1.3 {
            println!("📊 Dispersion pattern: Poor distance control");
            println!("  → σ_y ({:.1} ft) >> σ_x ({:.1} ft)", sigma_y, sigma_x);
            println!("  → Focus on: Club selection and tempo\n");
        } else {
            println!("✓ Balanced dispersion pattern (σ_x ≈ σ_y)\n");
        }

        // Overall assessment
        let total_bias = (mu_x * mu_x + mu_y * mu_y).sqrt();
        let avg_dispersion = (sigma_x + sigma_y) / 2.0;

        println!("Overall Performance:");
        println!("  Total bias magnitude: {:.1} ft", total_bias);
        println!("  Average dispersion:   {:.1} ft", avg_dispersion);

        if total_bias < 2.0 && avg_dispersion < 30.0 {
            println!("  Rating: ⭐⭐⭐ Excellent - minimal bias, tight dispersion");
        } else if total_bias < 4.0 && avg_dispersion < 40.0 {
            println!("  Rating: ⭐⭐ Good - manageable bias and dispersion");
        } else {
            println!("  Rating: ⭐ Needs improvement - significant bias or dispersion");
        }
    }

    println!("\n═══════════════════════════════════════════════════════════\n");
    println!("Key Takeaways:");
    println!();
    println!("1. 4D Kalman filter successfully learns player's true bias");
    println!("   and dispersion pattern from (x,y) shot coordinates");
    println!();
    println!("2. BVN distribution captures elliptical shot patterns that");
    println!("   radial Rayleigh model cannot detect (σ_x ≠ σ_y)");
    println!();
    println!("3. Systematic bias (μ_x, μ_y) provides actionable coaching");
    println!("   insights for aim adjustment and club selection");
    println!();
    println!("4. P_max calculation using learned BVN parameters ensures");
    println!("   fair payouts even with non-symmetric dispersion");
    println!();
    println!("5. Ready for camera integration (Phase 9.5) to use real");
    println!("   (x,y) coordinates instead of simulated data");
    println!("\n═══════════════════════════════════════════════════════════\n");
}
