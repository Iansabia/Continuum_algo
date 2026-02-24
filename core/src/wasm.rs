// WASM Bridge for Continuum Golf Simulator
// Exports Rust simulator functions to JavaScript for browser-based demos

use crate::analytics::metrics::{calculate_expected_value, calculate_fairness_metric};
use crate::anti_cheat::detect_ml_ensemble;
use crate::models::hole::{ClubCategory, Hole, HOLE_CONFIGURATIONS};
use crate::models::player::{calculate_initial_dispersion, Player};
use crate::simulators::player_session::{run_session, HoleSelection, SessionConfig};
use crate::simulators::venue::{run_venue_simulation, PlayerArchetype, VenueConfig};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

static PERSISTENT_PLAYER: Lazy<Mutex<Option<Player>>> = Lazy::new(|| Mutex::new(None));

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// WASM-Friendly Data Structures
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct WasmShotOutcome {
    pub shot_number: usize,
    pub hole_id: u8,
    pub distance_yds: u16,
    pub wager: f64,
    pub miss_distance_ft: f64,
    pub multiplier: f64,
    pub payout: f64,
    pub cumulative_net: f64,
    pub is_fat_tail: bool,
    pub p_max: f64,
}

#[derive(Serialize, Deserialize)]
pub struct WasmSkillProfile {
    pub category: String,
    pub sigma: f64,
    pub confidence: f64,
    pub p_max_current: f64,
}

#[derive(Serialize, Deserialize)]
pub struct WasmAnomalyReport {
    pub is_suspicious: bool,
    pub confidence: f64,
    pub detected_patterns: Vec<String>,
    pub recommended_action: String,
}

#[derive(Serialize, Deserialize)]
pub struct WasmSessionResult {
    pub total_wagered: f64,
    pub total_won: f64,
    pub net_gain_loss: f64,
    pub session_house_edge: f64,
    pub shots: Vec<WasmShotOutcome>,
    pub final_skills: Vec<WasmSkillProfile>,
    pub anti_cheat_report: Option<WasmAnomalyReport>,
}

#[derive(Serialize, Deserialize)]
pub struct WasmVenueResult {
    pub total_wagered: f64,
    pub total_payouts: f64,
    pub net_profit: f64,
    pub hold_percentage: f64,
    pub profit_by_hour: Vec<(f64, f64)>,
    pub heatmap: WasmHeatmap,
}

#[derive(Serialize, Deserialize)]
pub struct WasmHeatmap {
    pub handicap_bins: Vec<String>,
    pub distance_bins: Vec<u16>,
    pub hold_percentages: Vec<Vec<f64>>,
}

#[derive(Serialize, Deserialize)]
pub struct WasmFairnessResult {
    pub hole_id: u8,
    pub handicap_results: Vec<WasmHandicapEV>,
    pub max_ev_difference: f64,
    pub is_fair: bool,
}

#[derive(Serialize, Deserialize)]
pub struct WasmHandicapEV {
    pub handicap: u8,
    pub expected_value: f64,
    pub p_max: f64,
}

#[derive(Serialize, Deserialize)]
pub struct WasmEnhancedPlayerResult {
    pub bay_id: usize,
    pub handicap: u8,
    pub pattern_type: String,
    pub sigma_x: f64,
    pub sigma_y: f64,
    pub rho: f64,
    pub boundary_points: Option<Vec<(f64, f64)>>, // Organic pattern boundary for visualization
    pub total_wagered: f64,
    pub total_won: f64,
    pub net: f64,
    pub rtp: f64,
    pub shots: Vec<WasmShotOutcome>,
}

#[derive(Serialize, Deserialize)]
pub struct WasmEnhancedVenueResult {
    pub total_wagered: f64,
    pub total_payouts: f64,
    pub net_profit: f64,
    pub hold_percentage: f64,
    pub total_shots: usize,
    pub num_bays: usize,
    pub avg_rtp: f64,
    pub players: Vec<WasmEnhancedPlayerResult>,
}

// ============================================================================
// WASM Exported Functions
// ============================================================================

#[wasm_bindgen]
pub fn reset_persistent_player() {
    console_log!("Resetting persistent player profile");
    let mut player_guard = PERSISTENT_PLAYER.lock().unwrap();
    *player_guard = None;
}

#[wasm_bindgen]
pub fn simulate_player_session(
    handicap: u8,
    num_shots: usize,
    wager_min: f64,
    wager_max: f64,
    hole_id: Option<u8>,
    manual_miss_distance: Option<f64>,
) -> Result<JsValue, JsValue> {
    console_log!(
        "Starting player session: handicap={}, shots={}",
        handicap,
        num_shots
    );

    // ---- Persistent Player Retrieval ----
    let mut player_guard = PERSISTENT_PLAYER.lock().unwrap();

    // Check if we need to create a new player or reset due to handicap change
    let needs_reset = player_guard
        .as_ref()
        .map_or(false, |p| p.handicap != handicap);

    if needs_reset {
        console_log!("Handicap changed, resetting player profile");
        *player_guard = None;
    }

    let player = player_guard.get_or_insert_with(|| {
        console_log!(
            "Creating new persistent player profile (handicap={})",
            handicap
        );
        Player::new(format!("wasm_player_{}", handicap), handicap)
    });

    // ---- Hole Selection ----
    let hole_selection = match hole_id {
        Some(id) => HoleSelection::Fixed(id),
        None => HoleSelection::Random,
    };

    // ---- Developer Mode ----
    // Developer shots are processed identically to real shots by MCMC
    // Only difference: miss distance comes from manual_miss_distance instead of simulation
    let developer_mode = manual_miss_distance.map(|dist| {
        use crate::simulators::player_session::DeveloperMode;
        DeveloperMode {
            manual_miss_distance: Some(dist),
        }
    });

    // ---- Session Configuration ----
    let config = SessionConfig {
        num_shots,
        wager_min,
        wager_max,
        hole_selection,
        developer_mode,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
        shot_generation_mode: None,
    };

    // ---- Run the session ----
    let result = run_session(player, config);

    // ---- Convert to WASM-friendly shot data ----
    let mut cumulative_net = 0.0;
    let wasm_shots: Vec<WasmShotOutcome> = result
        .shots
        .iter()
        .enumerate()
        .map(|(i, shot)| {
            cumulative_net += shot.payout - shot.wager;
            let hole = HOLE_CONFIGURATIONS
                .iter()
                .find(|h| h.id == shot.hole_id)
                .unwrap();

            WasmShotOutcome {
                shot_number: i + 1,
                hole_id: shot.hole_id,
                distance_yds: hole.distance_yds,
                wager: shot.wager,
                miss_distance_ft: shot.miss_distance_ft,
                multiplier: shot.multiplier,
                payout: shot.payout,
                cumulative_net,
                is_fat_tail: shot.is_fat_tail,
                p_max: shot.p_max,
            }
        })
        .collect();

    // ---- Extract MCMC skill states ----
    // Pre-calculate P_max for each category to avoid borrow conflicts
    // Use representative holes for each category
    let category_holes: HashMap<ClubCategory, &Hole> = [
        (ClubCategory::Wedge, &HOLE_CONFIGURATIONS[0]), // Hole 1: 75yd
        (ClubCategory::MidIron, &HOLE_CONFIGURATIONS[3]), // Hole 4: 150yd
        (ClubCategory::LongIron, &HOLE_CONFIGURATIONS[7]), // Hole 8: 250yd
    ]
    .iter()
    .copied()
    .collect();

    let mut category_p_max: HashMap<ClubCategory, f64> = HashMap::new();
    for (category, hole) in category_holes.iter() {
        let p_max = player.calculate_p_max(hole);
        category_p_max.insert(*category, p_max);
    }

    let wasm_skills: Vec<WasmSkillProfile> = player
        .skill_profiles
        .iter_mut()
        .map(|(category, skill)| {
            let sigma_est = skill.mcmc_estimator.get_sigma_estimate();
            let conf = skill.mcmc_estimator.calculate_confidence();

            // Get pre-calculated P_max
            let p_max = category_p_max.get(category).copied().unwrap_or(0.0);

            WasmSkillProfile {
                category: format!("{:?}", category),
                sigma: sigma_est,
                confidence: conf,
                p_max_current: p_max,
            }
        })
        .collect();

    // ---- Anti-Cheat Ensemble ----
    let anti_cheat_report = if result.shots.len() >= 10 {
        let confidence_history: Vec<(usize, f64)> = (1..=result.shots.len())
            .map(|i| (i, (i as f64 / 30.0 * 100.0).min(100.0)))
            .collect();

        let historical_shots = if result.shots.len() >= 30 {
            let split_point = (result.shots.len() as f64 * 0.7) as usize;
            Some(&result.shots[..split_point])
        } else {
            None
        };

        let ml_report =
            detect_ml_ensemble(&result.shots, historical_shots, Some(&confidence_history));

        Some(WasmAnomalyReport {
            is_suspicious: ml_report.is_suspicious,
            confidence: ml_report.ensemble_score,
            detected_patterns: ml_report.detected_patterns,
            recommended_action: ml_report.recommended_action,
        })
    } else {
        None
    };

    // ---- Package Final Result ----
    let wasm_result = WasmSessionResult {
        total_wagered: result.total_wagered,
        total_won: result.total_won,
        net_gain_loss: result.net_gain_loss,
        session_house_edge: result.session_house_edge,
        shots: wasm_shots,
        final_skills: wasm_skills,
        anti_cheat_report,
    };

    serde_wasm_bindgen::to_value(&wasm_result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn simulate_venue(
    num_bays: usize,
    hours: f64,
    shots_per_hour: usize,
    wager_min: f64,
    wager_max: f64,
) -> Result<JsValue, JsValue> {
    console_log!(
        "Starting venue simulation: bays={}, hours={}",
        num_bays,
        hours
    );

    let config = VenueConfig {
        num_bays,
        hours,
        shots_per_hour,
        player_archetype: PlayerArchetype::BellCurve {
            mean: 15,
            std_dev: 7.0,
        },
        wager_range: (wager_min, wager_max),
    };

    let result = run_venue_simulation(config);

    let wasm_heatmap = WasmHeatmap {
        handicap_bins: result.heatmap_data.handicap_bins.clone(),
        distance_bins: result.heatmap_data.distance_bins.clone(),
        hold_percentages: result.heatmap_data.hold_percentages.clone(),
    };

    let wasm_result = WasmVenueResult {
        total_wagered: result.total_wagered,
        total_payouts: result.total_payouts,
        net_profit: result.net_profit,
        hold_percentage: result.hold_percentage,
        profit_by_hour: result.profit_over_time.clone(),
        heatmap: wasm_heatmap,
    };

    serde_wasm_bindgen::to_value(&wasm_result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Enhanced venue simulation with detailed player tracking and random dispersion patterns
#[wasm_bindgen]
pub fn simulate_venue_enhanced(
    num_bays: usize,
    shots_per_hour: usize,
    hours_of_operation: usize,
    wager: f64,
) -> Result<JsValue, JsValue> {
    use crate::simulators::venue::generate_player_pool;

    let shots_per_bay = shots_per_hour * hours_of_operation;

    console_log!("Starting enhanced venue simulation: bays={}, shots_per_hour={}, hours={}, total_shots_per_bay={}",
        num_bays, shots_per_hour, hours_of_operation, shots_per_bay);

    // Generate diverse player pool with bell curve distribution
    let players = generate_player_pool(
        &PlayerArchetype::BellCurve {
            mean: 15,
            std_dev: 7.0,
        },
        num_bays,
    );

    let mut all_player_results = Vec::new();
    let mut rng = rand::thread_rng();

    // Simulate each bay with unique random dispersion pattern
    for (bay_idx, mut player) in players.into_iter().enumerate() {
        console_log!(
            "Simulating bay {} - Player handicap: {}",
            bay_idx + 1,
            player.handicap
        );

        // Calculate base sigma from player's handicap (using mid-iron distance as reference)
        let base_sigma = calculate_initial_dispersion(player.handicap, 150);

        // Generate random organic pattern for this player
        // Pattern size is based on handicap, but actual shape is randomized
        use crate::math::organic_patterns::OrganicPatternGenerator;
        use crate::simulators::player_session::ShotGenerationMode;

        let pattern_generator = OrganicPatternGenerator::create_random(&mut rng, base_sigma);
        let organic_pattern = pattern_generator.generate();
        let bvn_params = organic_pattern.to_bvn_parameters();

        console_log!(
            "Bay {} pattern: σ_x={:.1}, σ_y={:.1}, ρ={:.3}",
            bay_idx + 1,
            bvn_params.sigma_x,
            bvn_params.sigma_y,
            bvn_params.rho
        );

        // Store pattern metadata for display
        let pattern_name = "organic";
        let sigma_x = bvn_params.sigma_x;
        let sigma_y = bvn_params.sigma_y;
        let rho = bvn_params.rho;

        // Run session for this bay with organic pattern generation
        // Pattern size determines effective skill (not stated handicap)
        let session_config = SessionConfig {
            num_shots: shots_per_bay,
            wager_min: wager,
            wager_max: wager,
            hole_selection: HoleSelection::Random,
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
            shot_generation_mode: Some(ShotGenerationMode::OrganicPattern {
                pattern: organic_pattern.clone(),
                bvn_params,
            }),
        };

        let session_result = run_session(&mut player, session_config);

        // The sigma_x, sigma_y, rho values are stored for visualization purposes
        // The actual simulation uses the player's handicap-based sigma

        // Convert shots to serializable format
        let shots_data: Vec<WasmShotOutcome> = session_result
            .shots
            .iter()
            .enumerate()
            .map(|(i, shot)| {
                let hole = HOLE_CONFIGURATIONS
                    .iter()
                    .find(|h| h.id == shot.hole_id)
                    .unwrap();
                WasmShotOutcome {
                    shot_number: i + 1,
                    hole_id: shot.hole_id,
                    distance_yds: hole.distance_yds,
                    wager: shot.wager,
                    miss_distance_ft: shot.miss_distance_ft,
                    multiplier: shot.multiplier,
                    payout: shot.payout,
                    cumulative_net: 0.0, // Will be calculated on frontend
                    is_fat_tail: shot.is_fat_tail,
                    p_max: shot.p_max,
                }
            })
            .collect();

        let rtp = if session_result.total_wagered > 0.0 {
            (session_result.total_won / session_result.total_wagered) * 100.0
        } else {
            0.0
        };

        all_player_results.push(WasmEnhancedPlayerResult {
            bay_id: bay_idx + 1,
            handicap: player.handicap,
            pattern_type: pattern_name.to_string(),
            sigma_x,
            sigma_y,
            rho,
            boundary_points: Some(organic_pattern.boundary_points.clone()),
            total_wagered: session_result.total_wagered,
            total_won: session_result.total_won,
            net: session_result.net_gain_loss,
            rtp,
            shots: shots_data,
        });
    }

    // Aggregate statistics
    let total_wagered: f64 = all_player_results.iter().map(|p| p.total_wagered).sum();
    let total_won: f64 = all_player_results.iter().map(|p| p.total_won).sum();
    let net_profit = total_wagered - total_won;
    let hold_percentage = if total_wagered > 0.0 {
        (net_profit / total_wagered) * 100.0
    } else {
        0.0
    };
    let total_shots = num_bays * shots_per_bay;
    let avg_rtp = if total_wagered > 0.0 {
        (total_won / total_wagered) * 100.0
    } else {
        0.0
    };

    let result = WasmEnhancedVenueResult {
        total_wagered,
        total_payouts: total_won,
        net_profit,
        hold_percentage,
        total_shots,
        num_bays,
        avg_rtp,
        players: all_player_results,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Simulate a single bay for parallel processing via Web Workers
#[wasm_bindgen]
pub fn simulate_single_bay(
    bay_id: usize,
    handicap: u8,
    shots_per_bay: usize,
    wager: f64,
) -> Result<JsValue, JsValue> {
    use crate::math::organic_patterns::OrganicPatternGenerator;
    use crate::simulators::player_session::ShotGenerationMode;

    console_log!("Worker simulating bay {} - handicap: {}", bay_id, handicap);

    // Create player
    let mut player = Player::new(format!("bay_{}", bay_id), handicap);
    let mut rng = rand::thread_rng();

    // Calculate base sigma from player's handicap
    let base_sigma = calculate_initial_dispersion(player.handicap, 150);

    // Generate random organic pattern
    let pattern_generator = OrganicPatternGenerator::create_random(&mut rng, base_sigma);
    let organic_pattern = pattern_generator.generate();
    let bvn_params = organic_pattern.to_bvn_parameters();

    // Extract values before moving bvn_params
    let sigma_x = bvn_params.sigma_x;
    let sigma_y = bvn_params.sigma_y;
    let rho = bvn_params.rho;

    console_log!(
        "Bay {} pattern: σ_x={:.1}, σ_y={:.1}, ρ={:.3}",
        bay_id,
        sigma_x,
        sigma_y,
        rho
    );

    // Run session
    let session_config = SessionConfig {
        num_shots: shots_per_bay,
        wager_min: wager,
        wager_max: wager,
        hole_selection: HoleSelection::Random,
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
        shot_generation_mode: Some(ShotGenerationMode::OrganicPattern {
            pattern: organic_pattern.clone(),
            bvn_params,
        }),
    };

    let session_result = run_session(&mut player, session_config);

    // Convert shots to serializable format
    let shots_data: Vec<WasmShotOutcome> = session_result
        .shots
        .iter()
        .enumerate()
        .map(|(i, shot)| {
            let hole = HOLE_CONFIGURATIONS
                .iter()
                .find(|h| h.id == shot.hole_id)
                .unwrap();
            WasmShotOutcome {
                shot_number: i + 1,
                hole_id: shot.hole_id,
                distance_yds: hole.distance_yds,
                wager: shot.wager,
                miss_distance_ft: shot.miss_distance_ft,
                multiplier: shot.multiplier,
                payout: shot.payout,
                cumulative_net: 0.0,
                is_fat_tail: shot.is_fat_tail,
                p_max: shot.p_max,
            }
        })
        .collect();

    let rtp = if session_result.total_wagered > 0.0 {
        (session_result.total_won / session_result.total_wagered) * 100.0
    } else {
        0.0
    };

    let result = WasmEnhancedPlayerResult {
        bay_id,
        handicap: player.handicap,
        pattern_type: "organic".to_string(),
        sigma_x,
        sigma_y,
        rho,
        boundary_points: Some(organic_pattern.boundary_points.clone()),
        total_wagered: session_result.total_wagered,
        total_won: session_result.total_won,
        net: session_result.net_gain_loss,
        rtp,
        shots: shots_data,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn validate_fairness(hole_id: u8) -> Result<JsValue, JsValue> {
    console_log!("Validating fairness for hole {}", hole_id);

    let hole = HOLE_CONFIGURATIONS
        .iter()
        .find(|h| h.id == hole_id)
        .ok_or_else(|| JsValue::from_str("Invalid hole ID"))?;

    // Call fairness metric with required arguments
    let handicaps_to_test = vec![0, 10, 20, 30];
    let trials_per_handicap = 1000;
    let fairness_report =
        calculate_fairness_metric(hole, handicaps_to_test.clone(), trials_per_handicap);

    let handicap_results: Vec<WasmHandicapEV> = handicaps_to_test
        .iter()
        .map(|&hc| {
            let player = Player::new(format!("test_player_{}", hc), hc);
            let ev = calculate_expected_value(&player, hole, 10.0, 1000);
            let p_max = player.calculate_p_max(hole);

            WasmHandicapEV {
                handicap: hc,
                expected_value: ev,
                p_max,
            }
        })
        .collect();

    let wasm_result = WasmFairnessResult {
        hole_id,
        handicap_results,
        max_ev_difference: fairness_report.max_ev_difference,
        is_fair: fairness_report.is_fair,
    };

    serde_wasm_bindgen::to_value(&wasm_result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn get_hole_info(hole_id: u8) -> Result<JsValue, JsValue> {
    let hole = HOLE_CONFIGURATIONS
        .iter()
        .find(|h| h.id == hole_id)
        .ok_or_else(|| JsValue::from_str("Invalid hole ID"))?;

    serde_wasm_bindgen::to_value(&hole)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Run anti-cheat analysis on provided shot data without mutating player state
///
/// This function is stateless and only analyzes the provided shots.
/// Use this for real-time anti-cheat monitoring without corrupting MCMC.
#[wasm_bindgen]
pub fn analyze_anti_cheat(shots_json: JsValue) -> Result<JsValue, JsValue> {
    use crate::models::shot::ShotOutcome;

    // Deserialize shots from JSON
    let shots: Vec<ShotOutcome> = serde_wasm_bindgen::from_value(shots_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse shots: {}", e)))?;

    console_log!("Running anti-cheat analysis on {} shots", shots.len());

    // Build confidence history (placeholder based on shot count)
    let confidence_history: Vec<(usize, f64)> = (1..=shots.len())
        .map(|i| (i, (i as f64 / 30.0 * 100.0).min(100.0)))
        .collect();

    // Split for historical comparison if enough shots
    let historical_shots = if shots.len() >= 30 {
        let split_point = (shots.len() as f64 * 0.7) as usize;
        Some(&shots[..split_point])
    } else {
        None
    };

    // Run ML ensemble detection
    let ml_report = detect_ml_ensemble(&shots, historical_shots, Some(&confidence_history));

    let wasm_report = WasmAnomalyReport {
        is_suspicious: ml_report.is_suspicious,
        confidence: ml_report.ensemble_score,
        detected_patterns: ml_report.detected_patterns,
        recommended_action: ml_report.recommended_action,
    };

    serde_wasm_bindgen::to_value(&wasm_report)
        .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}

#[wasm_bindgen]
pub fn get_all_holes() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&HOLE_CONFIGURATIONS)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
