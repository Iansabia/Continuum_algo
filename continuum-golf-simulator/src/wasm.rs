// WASM Bridge for Continuum Golf Simulator
// Exports Rust simulator functions to JavaScript for browser-based demos

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::models::player::Player;
use crate::models::hole::{Hole, HOLE_CONFIGURATIONS, ClubCategory};
use crate::simulators::player_session::{SessionConfig, HoleSelection, run_session};
use crate::simulators::venue::{VenueConfig, PlayerArchetype, run_venue_simulation};
use crate::analytics::metrics::{calculate_expected_value, calculate_fairness_metric};
use crate::anti_cheat::{detect_ml_ensemble};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::HashMap;

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
    console_log!("Starting player session: handicap={}, shots={}", handicap, num_shots);

    // ---- Persistent Player Retrieval ----
    let mut player_guard = PERSISTENT_PLAYER.lock().unwrap();

    // Check if we need to create a new player or reset due to handicap change
    let needs_reset = player_guard.as_ref().map_or(false, |p| p.handicap != handicap);

    if needs_reset {
        console_log!("Handicap changed, resetting player profile");
        *player_guard = None;
    }

    let player = player_guard.get_or_insert_with(|| {
        console_log!("Creating new persistent player profile (handicap={})", handicap);
        Player::new(format!("wasm_player_{}", handicap), handicap)
    });

    // ---- Hole Selection ----
    let hole_selection = match hole_id {
        Some(id) => HoleSelection::Fixed(id),
        None => HoleSelection::Random,
    };

    // ---- Developer Mode ----
    let developer_mode = manual_miss_distance.map(|dist| {
        use crate::simulators::player_session::DeveloperMode;
        DeveloperMode {
            manual_miss_distance: Some(dist),
            disable_kalman: true, // Now MCMC-only
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
    };

    // ---- Run the session ----
    let result = run_session(player, config);

    // ---- Convert to WASM-friendly shot data ----
    let mut cumulative_net = 0.0;
    let wasm_shots: Vec<WasmShotOutcome> = result.shots.iter().enumerate().map(|(i, shot)| {
        cumulative_net += shot.payout - shot.wager;
        let hole = HOLE_CONFIGURATIONS.iter().find(|h| h.id == shot.hole_id).unwrap();

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
    }).collect();

    // ---- Extract MCMC skill states ----
    // Calculate current P_max for each club category using the most recent hole from that category
    let mut category_holes: HashMap<ClubCategory, &Hole> = HashMap::new();
    for shot in result.shots.iter().rev() {
        let hole = HOLE_CONFIGURATIONS.iter().find(|h| h.id == shot.hole_id).unwrap();
        category_holes.entry(hole.category).or_insert(hole);
    }

    // Pre-calculate P_max values to avoid borrow conflicts
    let mut category_p_max: HashMap<ClubCategory, f64> = HashMap::new();
    for (category, hole) in category_holes.iter() {
        let p_max = player.calculate_p_max(hole);
        category_p_max.insert(*category, p_max);
    }

    let wasm_skills: Vec<WasmSkillProfile> = player.skill_profiles.iter_mut().map(|(category, skill)| {
        let sigma_est = skill.mcmc_estimator.get_sigma_estimate();
        let conf = skill.mcmc_estimator.calculate_confidence();

        // Get pre-calculated P_max or use cached value
        let p_max = category_p_max.get(category).copied().unwrap_or_else(|| {
            skill.p_max_history.last().copied().unwrap_or(0.0)
        });

        WasmSkillProfile {
            category: format!("{:?}", category),
            sigma: sigma_est,
            confidence: conf,
            p_max_current: p_max,
        }
    }).collect();

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

        let ml_report = detect_ml_ensemble(
            &result.shots,
            historical_shots,
            Some(&confidence_history),
        );

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
    console_log!("Starting venue simulation: bays={}, hours={}", num_bays, hours);

    let config = VenueConfig {
        num_bays,
        hours,
        shots_per_hour,
        player_archetype: PlayerArchetype::BellCurve { mean: 15, std_dev: 7.0 },
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

#[wasm_bindgen]
pub fn validate_fairness(hole_id: u8) -> Result<JsValue, JsValue> {
    console_log!("Validating fairness for hole {}", hole_id);

    let hole = HOLE_CONFIGURATIONS.iter()
        .find(|h| h.id == hole_id)
        .ok_or_else(|| JsValue::from_str("Invalid hole ID"))?;

    // Call fairness metric with required arguments
    let handicaps_to_test = vec![0, 10, 20, 30];
    let trials_per_handicap = 1000;
    let fairness_report = calculate_fairness_metric(hole, handicaps_to_test.clone(), trials_per_handicap);

    let handicap_results: Vec<WasmHandicapEV> = handicaps_to_test.iter().map(|&hc| {
        let player = Player::new(format!("test_player_{}", hc), hc);
        let ev = calculate_expected_value(&player, hole, 10.0, 1000);
        let p_max = player.calculate_p_max(hole);

        WasmHandicapEV {
            handicap: hc,
            expected_value: ev,
            p_max,
        }
    }).collect();

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
    let hole = HOLE_CONFIGURATIONS.iter()
        .find(|h| h.id == hole_id)
        .ok_or_else(|| JsValue::from_str("Invalid hole ID"))?;

    serde_wasm_bindgen::to_value(&hole)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn get_all_holes() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&HOLE_CONFIGURATIONS)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
