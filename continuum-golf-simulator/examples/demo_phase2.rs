// Demo of Phase 2 - Core Data Models
//
// Run with: cargo run --example demo_phase2

use continuum_golf_simulator::models::hole::get_hole_by_id;
use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::models::shot::simulate_shot;

fn main() {
    println!("🏌️  Continuum Golf Simulator - Phase 2 Demo\n");
    println!("{}", "=".repeat(60));

    // Create a player
    let mut player = Player::new("Demo Player".to_string(), 15);
    println!("\n✅ Created player: {} (Handicap: {})", player.id, player.handicap);

    // Get a hole
    let hole = get_hole_by_id(4).unwrap(); // 150 yard hole
    println!("\n✅ Selected Hole #{}: {} yards", hole.id, hole.distance_yds);
    println!("   - Max scoring radius: {:.2} ft", hole.d_max_ft);
    println!("   - RTP: {:.0}%", hole.rtp * 100.0);
    println!("   - Steepness (k): {}", hole.k);

    // Calculate P_max for this player/hole combination
    let p_max = player.calculate_p_max(hole);
    println!("\n✅ Calculated P_max: {:.2}×", p_max);
    println!("   (Maximum payout multiplier for perfect shot)");

    // Get player's current skill estimate
    let sigma = player.get_current_sigma(hole);
    let confidence = player.get_skill_confidence(hole);
    println!("\n✅ Player's skill estimate (σ): {:.2} ft", sigma);
    println!("   Confidence: {:.1}%", confidence);

    // Simulate some shots
    println!("\n🎯 Simulating 10 shots:");
    println!("{}", "-".repeat(60));

    let mut total_wagered = 0.0;
    let mut total_won = 0.0;

    for i in 1..=10 {
        let wager = 10.0; // $10 per shot

        // Simulate shot with 2% fat-tail probability
        let (miss_distance, is_fat_tail) = simulate_shot(sigma, 0.02, 3.0);

        // Calculate payout
        let multiplier = hole.calculate_payout(miss_distance, p_max);
        let payout = multiplier * wager;
        let net = payout - wager;

        total_wagered += wager;
        total_won += payout;

        // Add to player's batch
        let batch_full = player.add_shot_to_batch(hole, miss_distance, wager);

        let fat_tail_indicator = if is_fat_tail { " ⚠️ FAT TAIL" } else { "" };

        println!(
            "Shot {:2}: Miss={:5.1}ft | Payout={:5.2}× (${:6.2}) | Net={:+7.2}{}",
            i, miss_distance, multiplier, payout, net, fat_tail_indicator
        );

        // Update skill when batch is full
        if batch_full {
            player.update_skill(hole, p_max);
            let new_sigma = player.get_current_sigma(hole);
            let new_confidence = player.get_skill_confidence(hole);
            println!("   📊 Skill updated! σ={:.2}ft, Confidence={:.1}%",
                     new_sigma, new_confidence);
        }
    }

    // Process any remaining shots in batch
    if player.get_batch_size(hole) > 0 {
        player.update_skill(hole, p_max);
    }

    println!("{}", "-".repeat(60));
    println!("\n📊 Session Summary:");
    println!("   Total Wagered: ${:.2}", total_wagered);
    println!("   Total Won:     ${:.2}", total_won);
    println!("   Net Result:    ${:+.2}", total_won - total_wagered);
    println!("   House Edge:    {:.1}%", (1.0 - total_won/total_wagered) * 100.0);

    // Final skill stats
    let final_sigma = player.get_current_sigma(hole);
    let final_confidence = player.get_skill_confidence(hole);
    println!("\n🎯 Final Skill Estimate:");
    println!("   σ = {:.2} ft (Confidence: {:.1}%)", final_sigma, final_confidence);

    // Calculate breakeven radius
    let breakeven = hole.calculate_breakeven_radius(p_max);
    println!("\n💰 Breakeven Analysis:");
    println!("   Breakeven radius: {:.2} ft", breakeven);
    println!("   (Need to land within this distance to break even)");

    println!("\n{}", "=".repeat(60));
    println!("✨ Demo complete! Phase 2 models are working perfectly.");
    println!("🚀 Next: Phase 3 will add full simulation engines!");
}
