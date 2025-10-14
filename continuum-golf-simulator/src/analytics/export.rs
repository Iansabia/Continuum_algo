/// Data export module
///
/// Provides functions for exporting simulation results to various formats:
/// - CSV for spreadsheet analysis
/// - JSON for web visualization tools
/// - Specialized formats for heatmaps and time-series data

use crate::models::player::Player;
use crate::simulators::player_session::SessionResult;
use crate::simulators::venue::VenueResult;
use crate::simulators::venue::HeatmapData;
use csv::Writer;
use std::error::Error;
use std::fs::File;
use std::io::Write;

/// Export session results to CSV format
///
/// Creates a CSV file with detailed shot-by-shot data including:
/// - Shot number, hole, wager, miss distance, multiplier, payout, cumulative net
///
/// # Arguments
/// * `result` - The session result to export
/// * `path` - Output file path (e.g., "session_results.csv")
///
/// # Returns
/// Result indicating success or error
///
/// # Example
/// ```no_run
/// use continuum_golf_simulator::models::player::Player;
/// use continuum_golf_simulator::simulators::player_session::{SessionConfig, run_session, HoleSelection};
/// use continuum_golf_simulator::analytics::export::export_session_csv;
///
/// let mut player = Player::new(15);
/// let config = SessionConfig {
///     num_shots: 100,
///     wager_range: (5.0, 10.0),
///     hole_selection: HoleSelection::Random,
///     developer_mode: None,
/// };
/// let result = run_session(&mut player, config);
/// export_session_csv(&result, "my_session.csv").unwrap();
/// ```
pub fn export_session_csv(result: &SessionResult, path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    
    // Write header
    wtr.write_record(&[
        "shot_num",
        "hole_id",
        "hole_distance_yds",
        "wager",
        "miss_distance_ft",
        "multiplier",
        "payout",
        "net_gain_loss",
        "cumulative_net",
        "is_fat_tail",
    ])?;
    
    let mut cumulative_net = 0.0;
    
    for (i, shot) in result.shots.iter().enumerate() {
        let net = shot.payout - shot.wager;
        cumulative_net += net;
        
        let hole = crate::models::hole::get_hole_by_id(shot.hole_id).unwrap();
        
        wtr.write_record(&[
            (i + 1).to_string(),
            shot.hole_id.to_string(),
            hole.distance_yds.to_string(),
            format!("{:.2}", shot.wager),
            format!("{:.2}", shot.miss_distance_ft),
            format!("{:.2}", shot.multiplier),
            format!("{:.2}", shot.payout),
            format!("{:.2}", net),
            format!("{:.2}", cumulative_net),
            shot.is_fat_tail.to_string(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Export venue results to JSON format
///
/// Creates a comprehensive JSON file with all venue simulation data including:
/// - Financial metrics (wagered, payouts, profit, hold%)
/// - Time-series profit data
/// - Heatmap data
/// - Payout distribution
///
/// # Arguments
/// * `result` - The venue result to export
/// * `path` - Output file path (e.g., "venue_results.json")
///
/// # Returns
/// Result indicating success or error
///
/// # Example
/// ```no_run
/// use continuum_golf_simulator::simulators::venue::{VenueConfig, run_venue_simulation, PlayerArchetype};
/// use continuum_golf_simulator::analytics::export::export_venue_json;
///
/// let config = VenueConfig {
///     num_bays: 50,
///     hours: 8.0,
///     shots_per_hour: 100,
///     player_archetype: PlayerArchetype::Uniform,
/// };
/// let result = run_venue_simulation(config);
/// export_venue_json(&result, "venue_results.json").unwrap();
/// ```
pub fn export_venue_json(result: &VenueResult, path: &str) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(result)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export heatmap data to CSV format
///
/// Creates a CSV matrix with:
/// - Rows: Distance bins (hole distances)
/// - Columns: Handicap bins
/// - Values: Hold percentages
///
/// # Arguments
/// * `heatmap` - The heatmap data to export
/// * `path` - Output file path (e.g., "heatmap.csv")
///
/// # Returns
/// Result indicating success or error
///
/// # Example
/// ```no_run
/// use continuum_golf_simulator::simulators::venue::{VenueConfig, run_venue_simulation, PlayerArchetype};
/// use continuum_golf_simulator::analytics::export::export_heatmap_csv;
///
/// let config = VenueConfig {
///     num_bays: 50,
///     hours: 8.0,
///     shots_per_hour: 100,
///     player_archetype: PlayerArchetype::Uniform,
/// };
/// let result = run_venue_simulation(config);
/// export_heatmap_csv(&result.heatmap_data, "heatmap.csv").unwrap();
/// ```
pub fn export_heatmap_csv(heatmap: &HeatmapData, path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    
    // Build header: ["Distance", "Handicap 0-4", "Handicap 5-9", ...]
    let mut header = vec!["Distance (yds)".to_string()];
    header.extend(heatmap.handicap_bins.clone());
    wtr.write_record(&header)?;
    
    // Write data rows
    for (i, distance) in heatmap.distance_bins.iter().enumerate() {
        let mut row = vec![distance.to_string()];

        for j in 0..heatmap.handicap_bins.len() {
            // Bounds check to prevent panics
            if i < heatmap.hold_percentages.len() && j < heatmap.hold_percentages[i].len() {
                let hold_pct = heatmap.hold_percentages[i][j];
                row.push(format!("{:.2}", hold_pct));
            } else {
                row.push("0.00".to_string());
            }
        }

        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Export P_max history to CSV format
///
/// Creates a time-series CSV showing how P_max values evolved for each club category
/// as the player's skill was updated via Kalman filtering.
///
/// # Arguments
/// * `player` - The player whose P_max history to export
/// * `path` - Output file path (e.g., "pmax_history.csv")
///
/// # Returns
/// Result indicating success or error
///
/// # Example
/// ```no_run
/// use continuum_golf_simulator::models::player::Player;
/// use continuum_golf_simulator::simulators::player_session::{SessionConfig, run_session, HoleSelection};
/// use continuum_golf_simulator::analytics::export::export_pmax_history;
///
/// let mut player = Player::new(15);
/// let config = SessionConfig {
///     num_shots: 100,
///     wager_range: (5.0, 10.0),
///     hole_selection: HoleSelection::Random,
///     developer_mode: None,
/// };
/// let result = run_session(&mut player, config);
/// export_pmax_history(&player, "pmax_history.csv").unwrap();
/// ```
pub fn export_pmax_history(player: &Player, path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    
    // Write header
    wtr.write_record(&["update_num", "club_category", "p_max"])?;
    
    for (category, profile) in &player.skill_profiles {
        let category_name = match category {
            crate::models::hole::ClubCategory::Wedge => "Wedge",
            crate::models::hole::ClubCategory::MidIron => "MidIron",
            crate::models::hole::ClubCategory::LongIron => "LongIron",
        };
        
        for (i, p_max) in profile.p_max_history.iter().enumerate() {
            wtr.write_record(&[
                (i + 1).to_string(),
                category_name.to_string(),
                format!("{:.4}", p_max),
            ])?;
        }
    }
    
    wtr.flush()?;
    Ok(())
}

/// Export convergence data to CSV format
///
/// Creates a CSV showing Kalman filter convergence metrics over time.
///
/// # Arguments
/// * `convergence_data` - Vector of (shot_number, confidence, sigma) tuples
/// * `path` - Output file path (e.g., "convergence.csv")
///
/// # Returns
/// Result indicating success or error
pub fn export_convergence_csv(
    convergence_data: Vec<(usize, f64, f64)>,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    
    wtr.write_record(&["shot_num", "confidence_pct", "skill_sigma"])?;
    
    for (shot_num, confidence, sigma) in convergence_data {
        wtr.write_record(&[
            shot_num.to_string(),
            format!("{:.2}", confidence),
            format!("{:.2}", sigma),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::player::Player;
    use crate::simulators::player_session::{SessionConfig, run_session, HoleSelection};
    use crate::simulators::venue::{VenueConfig, run_venue_simulation, PlayerArchetype};
    use std::fs;

    #[test]
    fn test_export_session_csv() {
        let mut player = Player::new("test_player".to_string(), 15);
        let config = SessionConfig {
            num_shots: 20,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        let result = run_session(&mut player, config);
        
        let path = "test_session.csv";
        export_session_csv(&result, path).unwrap();
        
        // Verify file exists and has content
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("shot_num"));
        assert!(contents.contains("hole_id"));
        assert!(contents.contains("cumulative_net"));
        
        // Cleanup
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_export_venue_json() {
        let config = VenueConfig {
            num_bays: 5,
            hours: 1.0,
            shots_per_hour: 50,
            player_archetype: PlayerArchetype::Uniform,
            wager_range: (5.0, 10.0),
        };
        let result = run_venue_simulation(config);

        let path = "test_venue.json";
        export_venue_json(&result, path).unwrap();
        
        // Verify file exists and is valid JSON
        let contents = fs::read_to_string(path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert!(parsed["total_wagered"].is_number());
        assert!(parsed["net_profit"].is_number());
        
        // Cleanup
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_export_heatmap_csv() {
        let config = VenueConfig {
            num_bays: 5,
            hours: 1.0,
            shots_per_hour: 50,
            player_archetype: PlayerArchetype::Uniform,
            wager_range: (5.0, 10.0),
        };
        let result = run_venue_simulation(config);

        let path = "test_heatmap.csv";
        export_heatmap_csv(&result.heatmap_data, path).unwrap();
        
        // Verify file exists and has proper structure
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("Distance"));
        // Heatmap has bins like "0-4", "5-9", etc
        assert!(contents.len() > 0);
        
        // Cleanup
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_export_pmax_history() {
        let mut player = Player::new("test_player".to_string(), 15);
        let config = SessionConfig {
            num_shots: 30,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Random,
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        let _result = run_session(&mut player, config);
        
        let path = "test_pmax_history.csv";
        export_pmax_history(&player, path).unwrap();
        
        // Verify file exists
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("update_num"));
        assert!(contents.contains("club_category"));
        assert!(contents.contains("p_max"));
        
        // Cleanup
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_export_convergence_csv() {
        let test_data = vec![
            (1, 20.0, 45.5),
            (10, 50.0, 42.3),
            (20, 70.0, 41.1),
            (30, 85.0, 40.8),
        ];
        
        let path = "test_convergence.csv";
        export_convergence_csv(test_data, path).unwrap();
        
        // Verify file exists
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("shot_num"));
        assert!(contents.contains("confidence_pct"));
        assert!(contents.contains("skill_sigma"));
        
        // Cleanup
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_session_csv_row_count() {
        let mut player = Player::new("test_player".to_string(), 10);
        let config = SessionConfig {
            num_shots: 15,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(1),
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        let result = run_session(&mut player, config);
        
        let path = "test_row_count.csv";
        export_session_csv(&result, path).unwrap();
        
        let contents = fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        
        // Should have header + 15 data rows = 16 total
        assert_eq!(lines.len(), 16);
        
        // Cleanup
        fs::remove_file(path).ok();
    }
}
