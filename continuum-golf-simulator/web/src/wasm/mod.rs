use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};
use web_sys::console;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import the `console.log` function from the `console` module of web_sys
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make console.log easier to use
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Shot {
    pub x: f64,
    pub y: f64,
    pub distance_from_target: f64,
    pub score: f64,
    pub timestamp: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerSession {
    pub shots: Vec<Shot>,
    pub total_score: f64,
    pub wager: f64,
    pub payout: f64,
    pub net_profit: f64,
    pub skill_estimate: f64,
    pub confidence: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VenueState {
    pub total_revenue: f64,
    pub total_payouts: f64,
    pub net_profit: f64,
    pub active_players: u32,
    pub sessions_completed: u32,
    pub average_session_length: f64,
}

#[wasm_bindgen]
pub struct GolfSimulator {
    rng_seed: u64,
}

#[wasm_bindgen]
impl GolfSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GolfSimulator {
        console_log!("✅ WASM Golf Simulator initialized");
        GolfSimulator { rng_seed: 12345 }
    }

    #[wasm_bindgen]
    pub fn simulate_shot(&mut self, handicap: f64, target_x: f64, target_y: f64) -> JsValue {
        // Simple shot simulation using handicap as skill factor
        let skill_factor = 1.0 / (1.0 + handicap / 10.0);
        
        // Generate shot with some randomness
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand1 = (self.rng_seed as f64) / (u64::MAX as f64) - 0.5;
        
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand2 = (self.rng_seed as f64) / (u64::MAX as f64) - 0.5;
        
        let accuracy = skill_factor * 50.0; // Better players have higher accuracy
        let shot_x = target_x + rand1 * accuracy;
        let shot_y = target_y + rand2 * accuracy;
        
        let distance_from_target = ((shot_x - target_x).powi(2) + (shot_y - target_y).powi(2)).sqrt();
        
        // Score based on distance (closer = better score)
        let max_distance = 100.0;
        let score = ((max_distance - distance_from_target.min(max_distance)) / max_distance * 100.0).max(0.0);
        
        let shot = Shot {
            x: shot_x,
            y: shot_y,
            distance_from_target,
            score,
            timestamp: js_sys::Date::now(),
        };
        
        serde_wasm_bindgen::to_value(&shot).unwrap()
    }

    #[wasm_bindgen]
    pub fn simulate_player_session(&mut self, handicap: f64, num_shots: u32, wager: f64) -> JsValue {
        let mut shots = Vec::new();
        let mut total_score = 0.0;
        
        for _ in 0..num_shots {
            let shot_result = self.simulate_shot(handicap, 0.0, 0.0);
            let shot: Shot = serde_wasm_bindgen::from_value(shot_result).unwrap();
            total_score += shot.score;
            shots.push(shot);
        }
        
        let average_score = total_score / num_shots as f64;
        
        // Calculate payout based on performance (85% RTP)
        let base_payout = wager * 0.85;
        let performance_multiplier = (average_score / 50.0).min(2.0); // Cap at 2x
        let payout = base_payout * performance_multiplier;
        
        let net_profit = payout - wager;
        
        // Estimate skill (inverse of handicap, normalized)
        let skill_estimate = 1.0 / (1.0 + handicap / 20.0);
        let confidence = (num_shots as f64 / 100.0).min(1.0); // More shots = higher confidence
        
        let session = PlayerSession {
            shots,
            total_score,
            wager,
            payout,
            net_profit,
            skill_estimate,
            confidence,
        };
        
        console_log!("Session completed: {} shots, score: {:.2}, payout: ${:.2}", 
                    num_shots, average_score, payout);
        
        serde_wasm_bindgen::to_value(&session).unwrap()
    }

    #[wasm_bindgen]
    pub fn validate_fairness(&self, handicap_range: &[f64], num_simulations: u32) -> JsValue {
        let mut results = HashMap::new();
        
        for &handicap in handicap_range {
            let mut total_payout = 0.0;
            let mut total_wager = 0.0;
            
            for _ in 0..num_simulations {
                let wager = 10.0; // Standard wager for fairness testing
                let mut sim = GolfSimulator::new();
                let session_result = sim.simulate_player_session(handicap, 20, wager);
                let session: PlayerSession = serde_wasm_bindgen::from_value(session_result).unwrap();
                
                total_payout += session.payout;
                total_wager += session.wager;
            }
            
            let rtp = total_payout / total_wager;
            results.insert(handicap.to_string(), rtp);
        }
        
        console_log!("Fairness validation completed for {} handicap levels", handicap_range.len());
        
        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    #[wasm_bindgen]
    pub fn simulate_venue(&mut self, num_bays: u32, hours: f64) -> JsValue {
        let mut total_revenue = 0.0;
        let mut total_payouts = 0.0;
        let mut sessions_completed = 0;
        let mut total_session_time = 0.0;
        
        // Simulate players arriving at random intervals
        let sessions_per_hour = 2.0; // Average sessions per bay per hour
        let total_sessions = (num_bays as f64 * hours * sessions_per_hour) as u32;
        
        for _ in 0..total_sessions {
            // Random handicap between 0-30
            self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let handicap = (self.rng_seed as f64) / (u64::MAX as f64) * 30.0;
            
            // Random wager between $5-50
            self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let wager = 5.0 + (self.rng_seed as f64) / (u64::MAX as f64) * 45.0;
            
            // Random session length (10-50 shots)
            self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let num_shots = 10 + ((self.rng_seed as f64) / (u64::MAX as f64) * 40.0) as u32;
            
            let session_result = self.simulate_player_session(handicap, num_shots, wager);
            let session: PlayerSession = serde_wasm_bindgen::from_value(session_result).unwrap();
            
            total_revenue += session.wager;
            total_payouts += session.payout;
            sessions_completed += 1;
            total_session_time += num_shots as f64 * 0.5; // Assume 30 seconds per shot
        }
        
        let venue_state = VenueState {
            total_revenue,
            total_payouts,
            net_profit: total_revenue - total_payouts,
            active_players: num_bays, // Simplified: assume all bays active
            sessions_completed,
            average_session_length: total_session_time / sessions_completed as f64,
        };
        
        console_log!("Venue simulation: {} sessions, revenue: ${:.2}, profit: ${:.2}", 
                    sessions_completed, total_revenue, venue_state.net_profit);
        
        serde_wasm_bindgen::to_value(&venue_state).unwrap()
    }
}

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("🚀 Continuum Golf WASM module loaded successfully");
}