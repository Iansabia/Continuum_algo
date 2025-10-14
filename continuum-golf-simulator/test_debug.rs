use continuum_golf_simulator::models::{player::Player, hole::get_hole_by_id};

fn main() {
    let player = Player::new("test".to_string(), 15);
    let hole = get_hole_by_id(1).unwrap();
    
    let skill = player.get_skill_for_hole(&hole);
    println!("Hole 1 (75 yds):");
    println!("  Sigma: {}", skill.kalman_filter.estimate);
    
    let p_max = player.calculate_p_max(&hole);
    println!("  P_max: {}", p_max);
    println!("  RTP target: {}", hole.rtp);
}
