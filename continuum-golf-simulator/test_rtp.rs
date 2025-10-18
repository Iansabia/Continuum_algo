// Test to verify RTP calculation
use continuum_golf_simulator::models::hole::{Hole, HOLE_CONFIGURATIONS};
use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::math::integration::trapezoidal_rule;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn main() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Test with Hole 1 and different skill levels
    let hole = &HOLE_CONFIGURATIONS[0]; // Hole 1: 75 yards, d_max=17.95, RTP=0.85, k=5.0

    println!("Testing Hole {}: {} yards, d_max={:.2} ft, RTP={:.3}, k={:.1}",
             hole.id, hole.distance_yds, hole.d_max_ft, hole.rtp, hole.k);
    println!();

    // Test different skill levels (sigma values)
    for sigma in [5.0, 7.0, 10.0, 15.0, 20.0] {
        println!("Testing with sigma = {:.1} ft", sigma);

        // Create a player with this exact sigma
        let mut player = Player::new(10, &mut rng);

        // Manually set the sigma value for this hole category
        let skill = player.get_skill_for_hole_mut(hole);
        skill.kalman_filter.estimate = sigma;

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
        println!("  Target RTP = {:.3}", hole.rtp);
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
