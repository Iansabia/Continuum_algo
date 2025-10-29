use continuum_golf_simulator::models::player::Player;
use continuum_golf_simulator::models::hole::get_hole_by_id;

fn main() {
    let player = Player::new("test".to_string(), 10);
    let hole = get_hole_by_id(4).unwrap(); // 150 yard hole
    
    println!("Handicap 10, Hole 4 (150yd):");
    println!("Initial P_max: {:.2}x", player.calculate_p_max(hole));
    println!("Initial sigma: {:.2}ft", player.get_current_sigma(hole));
}
