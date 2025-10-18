// Test to verify RTP calculation
use continuum_golf_simulator::models::hole::HOLE_CONFIGURATIONS;
use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::math::integration::trapezoidal_rule;

fn main() {
    // Test all 8 holes with a handicap 10 player
    let handicap = 10;
    println!("Testing with Handicap {} player", handicap);
    println!();

    for hole in HOLE_CONFIGURATIONS.iter() {
        println!("Hole {}: {} yards, d_max={:.2} ft, RTP={:.3}, k={:.1}",
                 hole.id, hole.distance_yds, hole.d_max_ft, hole.rtp, hole.k);

        // Create a player
        let player = Player::new("test".to_string(), handicap);

        // Get the skill for this hole (uses the default initialized sigma)
        let skill = player.get_skill_for_hole(hole);
        let sigma = skill.kalman_filter.estimate;
        println!("  Default sigma = {:.2} ft", sigma);

        // Calculate P_max
        let p_max = player.calculate_p_max(hole);
        println!("  P_max = {:.4}", p_max);

        // Now compute the actual expected return by integrating
        let integrand = |d: f64| -> f64 {
            let rayleigh_pdf = (d / (sigma * sigma)) * (-d * d / (2.0 * sigma * sigma)).exp();

            if d > hole.d_max_ft {
                // Miss - no payout
                0.0
            } else {
                // Score - get payout
                let payout_multiplier = hole.calculate_payout(d, p_max);
                payout_multiplier * rayleigh_pdf
            }
        };

        let upper_bound = (hole.d_max_ft * 1.5).max(sigma * 5.0);
        let expected_return = trapezoidal_rule(integrand, 0.0, upper_bound, 5000);

        println!("  Expected return = {:.4}", expected_return);
        println!("  Actual hold % = {:.2}%", (1.0 - expected_return) * 100.0);
        println!("  Target hold % = {:.2}%", (1.0 - hole.rtp) * 100.0);

        // Also compute probability of scoring
        let prob_score_integrand = |d: f64| -> f64 {
            let rayleigh_pdf = (d / (sigma * sigma)) * (-d * d / (2.0 * sigma * sigma)).exp();
            if d <= hole.d_max_ft {
                rayleigh_pdf
            } else {
                0.0
            }
        };
        let prob_score = trapezoidal_rule(prob_score_integrand, 0.0, upper_bound, 5000);
        println!("  P(score) = {:.4}", prob_score);
        println!();
    }
}
