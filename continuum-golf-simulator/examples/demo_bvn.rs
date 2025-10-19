/// Demonstration of BVN (Bivariate Normal) distribution
///
/// This example shows how BVN models 2D shot dispersion with bias,
/// compared to the legacy 1D Rayleigh model.

use continuum_golf_simulator::math::distributions::{
    bvn_random, fat_tail_shot_bvn, rayleigh_random,
};

fn main() {
    println!("=== BVN Distribution Demo ===\n");

    // Example 1: Player with rightward bias and better distance control
    println!("Example 1: Player with systematic bias");
    println!("  - Bias: 5 ft right, 2 ft long (μ_x=5, μ_y=2)");
    println!("  - Lateral dispersion: σ_x=15 ft");
    println!("  - Distance dispersion: σ_y=8 ft (better distance control)");
    println!("  - Hole: 150 yards (pin at origin)\n");

    let mu_x = 5.0;  // Tends to miss 5 ft right
    let mu_y = 2.0;  // Tends to miss 2 ft long
    let sigma_x = 15.0;  // Lateral precision
    let sigma_y = 8.0;   // Distance precision (better!)

    println!("10 sample shots:");
    for i in 1..=10 {
        let (x, y) = bvn_random(mu_x, mu_y, sigma_x, sigma_y);
        let distance = (x * x + y * y).sqrt();

        let lateral = if x > 0.0 { "right" } else { "left" };
        let depth = if y > 0.0 { "long" } else { "short" };

        println!(
            "  Shot {}: {:.1} ft {} of pin, {:.1} ft {} → {:.1} ft from hole",
            i, x.abs(), lateral, y.abs(), depth, distance
        );
    }

    // Example 2: Symmetric player (like Rayleigh)
    println!("\n\nExample 2: No bias (symmetric, like Rayleigh)");
    println!("  - Bias: None (μ_x=0, μ_y=0)");
    println!("  - Equal dispersion: σ_x=σ_y=20 ft\n");

    let mu_x_sym = 0.0;
    let mu_y_sym = 0.0;
    let sigma_sym = 20.0;

    println!("10 sample shots (BVN):");
    for i in 1..=10 {
        let (x, y) = bvn_random(mu_x_sym, mu_y_sym, sigma_sym, sigma_sym);
        let distance = (x * x + y * y).sqrt();
        println!("  Shot {}: ({:.1}, {:.1}) → {:.1} ft", i, x, y, distance);
    }

    println!("\n10 sample shots (Legacy Rayleigh):");
    for i in 1..=10 {
        let distance = rayleigh_random(sigma_sym);
        println!("  Shot {}: {:.1} ft (no directional info)", i, distance);
    }

    // Example 3: Fat-tail events
    println!("\n\nExample 3: Fat-tail events (2% chance of 3× worse dispersion)");
    println!("  - Normal: σ_x=12, σ_y=12");
    println!("  - Fat-tail: σ_x=36, σ_y=36\n");

    let mut normal_count = 0;
    let mut fat_tail_count = 0;

    for i in 1..=50 {
        let ((x, y), is_fat_tail) = fat_tail_shot_bvn(0.0, 0.0, 12.0, 12.0, 0.02, 3.0);
        let distance = (x * x + y * y).sqrt();

        if is_fat_tail {
            fat_tail_count += 1;
            println!(
                "  Shot {}: ({:.1}, {:.1}) → {:.1} ft ⚠️ FAT-TAIL EVENT",
                i, x, y, distance
            );
        } else {
            normal_count += 1;
        }
    }

    println!("\nResults over 50 shots:");
    println!("  - Normal shots: {}", normal_count);
    println!("  - Fat-tail shots: {} ({:.1}%)", fat_tail_count, (fat_tail_count as f64 / 50.0) * 100.0);
    println!("  - Expected: ~1 fat-tail (2%)");

    // Example 4: Coaching insights
    println!("\n\n=== Coaching Insights (enabled by BVN) ===\n");

    println!("Player A (symmetric):");
    println!("  - Bias: (0, 0) → Aim is perfect!");
    println!("  - Dispersion: σ_x=20, σ_y=20 → Equal precision");
    println!("  - Advice: Keep doing what you're doing\n");

    println!("Player B (rightward bias):");
    println!("  - Bias: (8, -2) → You miss 8 ft right, 2 ft short on average");
    println!("  - Dispersion: σ_x=18, σ_y=10 → Better distance control than lateral");
    println!("  - Advice: Aim 8 ft left of pin, work on lateral consistency\n");

    println!("Player C (precision mismatch):");
    println!("  - Bias: (1, 0) → Nearly perfect aim");
    println!("  - Dispersion: σ_x=25, σ_y=10 → Distance is 2.5× more precise than lateral");
    println!("  - Advice: Focus on club face alignment, your distance is excellent\n");

    println!("=== End of Demo ===");
    println!("\nNote: This demo uses the new BVN math functions from Phase 9.1.");
    println!("Full integration requires:");
    println!("  - Phase 9.2: 4D Kalman filter (learn μ_x, μ_y, σ_x, σ_y from shots)");
    println!("  - Phase 9.3: 2D P_max calculation");
    println!("  - Phase 9.4: (x,y) data storage");
    println!("  - Phase 9.5: Camera integration for actual (x,y) capture");
}
