//! Player Session Simulator
//!
//! Simulates individual player gaming sessions with:
//! - Configurable shot counts and wager ranges
//! - Flexible hole selection strategies
//! - Real-time Kalman filter updates for skill tracking
//! - Batch processing and high-stakes shot detection
//! - Developer mode for manual testing

use crate::models::{
    hole::{get_hole_by_id, Hole, HOLE_CONFIGURATIONS},
    player::Player,
    shot::{simulate_shot, ShotOutcome},
};
use crate::anti_cheat::{detect_cherry_picking, detect_sandbagging, AnomalyReport};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Errors that can occur during session configuration validation
#[derive(Debug, Clone, PartialEq)]
pub enum SessionConfigError {
    /// Developer mode attempted in production environment
    DeveloperModeInProduction,
    /// Invalid wager range
    InvalidWagerRange(String),
    /// Invalid configuration
    InvalidConfig(String),
}

/// Configuration for a player gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Number of shots to simulate in the session
    pub num_shots: usize,
    /// Minimum wager per shot
    pub wager_min: f64,
    /// Maximum wager per shot
    pub wager_max: f64,
    /// Strategy for selecting which hole to play
    pub hole_selection: HoleSelection,
    /// Optional developer mode settings for testing
    pub developer_mode: Option<DeveloperMode>,
    /// Fat-tail probability (default: 0.02 = 2%)
    pub fat_tail_prob: f64,
    /// Fat-tail multiplier (default: 3.0)
    pub fat_tail_mult: f64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            num_shots: 100,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Random,
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        }
    }
}

impl SessionConfig {
    /// Check if we're running in production mode
    ///
    /// Returns true if CONTINUUM_ENV=production or CONTINUUM_PRODUCTION=true
    pub fn is_production() -> bool {
        env::var("CONTINUUM_ENV")
            .map(|v| v.to_lowercase() == "production")
            .unwrap_or(false)
        || env::var("CONTINUUM_PRODUCTION")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
    }

    /// Validate configuration for production deployment
    ///
    /// # Errors
    ///
    /// Returns `SessionConfigError` if:
    /// - Developer mode is enabled in production environment
    /// - Wager range is invalid (min > max or negative values)
    /// - Other configuration issues detected
    ///
    /// # Production Safety
    ///
    /// This function prevents the critical security vulnerability where developer
    /// mode with manual miss distances allows trivial exploitation. Production
    /// environments MUST reject any configuration with developer_mode set.
    pub fn validate(&self) -> Result<(), SessionConfigError> {
        // CRITICAL: Block developer mode in production
        if Self::is_production() && self.developer_mode.is_some() {
            return Err(SessionConfigError::DeveloperModeInProduction);
        }

        // Validate wager range
        if self.wager_min < 0.0 || self.wager_max < 0.0 {
            return Err(SessionConfigError::InvalidWagerRange(
                "Wagers cannot be negative".to_string()
            ));
        }

        if self.wager_min > self.wager_max {
            return Err(SessionConfigError::InvalidWagerRange(
                format!("Min wager (${:.2}) exceeds max wager (${:.2})",
                        self.wager_min, self.wager_max)
            ));
        }

        // Validate shot count
        if self.num_shots == 0 {
            return Err(SessionConfigError::InvalidConfig(
                "Number of shots must be greater than 0".to_string()
            ));
        }

        // Validate fat-tail parameters
        if self.fat_tail_prob < 0.0 || self.fat_tail_prob > 1.0 {
            return Err(SessionConfigError::InvalidConfig(
                format!("Fat-tail probability must be between 0.0 and 1.0, got {}",
                        self.fat_tail_prob)
            ));
        }

        if self.fat_tail_mult <= 0.0 {
            return Err(SessionConfigError::InvalidConfig(
                format!("Fat-tail multiplier must be positive, got {}",
                        self.fat_tail_mult)
            ));
        }

        Ok(())
    }

    /// Validate and log any developer mode usage attempt
    ///
    /// This method both validates the configuration AND logs suspicious activity.
    /// In production, it logs any attempt to use developer mode for security monitoring.
    pub fn validate_with_logging(&self, player_id: &str) -> Result<(), SessionConfigError> {
        // Check if developer mode is attempted
        if self.developer_mode.is_some() {
            let is_prod = Self::is_production();

            // Log the attempt (in production, this should trigger an alert)
            eprintln!(
                "⚠️  SECURITY ALERT: Developer mode attempted by player '{}' (production={})",
                player_id, is_prod
            );

            if is_prod {
                eprintln!("❌ BLOCKED: Developer mode is disabled in production");
            }
        }

        // Run standard validation
        self.validate()
    }
}

/// Strategy for selecting which hole to play
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HoleSelection {
    /// Random selection from all 8 holes
    Random,
    /// Weighted probabilities for each hole
    /// Vec of (hole_id, probability) pairs that must sum to 1.0
    Weighted(Vec<(u8, f64)>),
    /// Always play the same hole
    Fixed(u8),
}

/// Developer mode settings for manual testing
///
/// ⚠️ SECURITY WARNING: Developer mode should NEVER be accessible to real players.
/// Manual miss distances allow complete control over shot outcomes, enabling
/// trivial exploitation of the payout system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperMode {
    /// If set, use this miss distance instead of simulating
    /// ⚠️ CRITICAL: This must never be available in production
    pub manual_miss_distance: Option<f64>,
    /// If true, disable Kalman filter updates (skill stays constant)
    pub disable_kalman: bool,
}

/// Results from a completed player session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResult {
    /// Total amount wagered across all shots
    pub total_wagered: f64,
    /// Total amount won (payouts) across all shots
    pub total_won: f64,
    /// Net gain or loss (total_won - total_wagered)
    pub net_gain_loss: f64,
    /// All shot outcomes in chronological order
    pub shots: Vec<ShotOutcome>,
    /// Final skill profiles after all Kalman updates
    pub final_skill_profiles: HashMap<String, f64>, // ClubCategory -> sigma
    /// Actual house edge for this session
    pub session_house_edge: f64,
    /// Number of Kalman updates performed
    pub num_kalman_updates: usize,
    /// Number of high-stakes shots (triggered immediate updates)
    pub num_high_stakes_shots: usize,
    /// Anti-cheat detection report for cherry-picking
    pub cherry_picking_report: Option<AnomalyReport>,
    /// Anti-cheat detection report for sandbagging
    pub sandbagging_report: Option<AnomalyReport>,
}

impl SessionResult {
    /// Calculate session house edge as percentage
    pub fn house_edge_percent(&self) -> f64 {
        if self.total_wagered > 0.0 {
            (1.0 - self.total_won / self.total_wagered) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate average wager per shot
    pub fn avg_wager(&self) -> f64 {
        if !self.shots.is_empty() {
            self.total_wagered / self.shots.len() as f64
        } else {
            0.0
        }
    }

    /// Calculate win rate (percentage of shots with payout > 0)
    pub fn win_rate(&self) -> f64 {
        if self.shots.is_empty() {
            return 0.0;
        }
        let wins = self.shots.iter().filter(|s| s.payout > 0.0).count();
        (wins as f64 / self.shots.len() as f64) * 100.0
    }

    /// Determine if account should be flagged based on anti-cheat reports
    ///
    /// Returns true if any anti-cheat report has confidence >= threshold (default 0.7)
    pub fn should_flag_account(&self, confidence_threshold: f64) -> bool {
        if let Some(ref report) = self.cherry_picking_report {
            if report.confidence >= confidence_threshold {
                return true;
            }
        }

        if let Some(ref report) = self.sandbagging_report {
            if report.confidence >= confidence_threshold {
                return true;
            }
        }

        false
    }

    /// Get recommended action based on anti-cheat analysis
    ///
    /// # Returns
    ///
    /// - `None` if no suspicious activity detected
    /// - `Some(action)` with recommended action string if suspicious
    ///
    /// # Threshold Levels
    ///
    /// - 0.7-0.8: "Monitor closely for continued pattern"
    /// - 0.8-0.9: "Flag for manual review"
    /// - 0.9+: "Immediate suspension pending investigation"
    pub fn get_recommended_action(&self) -> Option<String> {
        let mut max_confidence = 0.0;
        let mut max_action = String::new();

        if let Some(ref report) = self.cherry_picking_report {
            if report.confidence > max_confidence {
                max_confidence = report.confidence;
                max_action = report.recommended_action.clone();
            }
        }

        if let Some(ref report) = self.sandbagging_report {
            if report.confidence > max_confidence {
                max_confidence = report.confidence;
                max_action = report.recommended_action.clone();
            }
        }

        if max_confidence >= 0.7 {
            Some(if max_confidence >= 0.9 {
                "URGENT: Immediate suspension pending investigation".to_string()
            } else if max_confidence >= 0.8 {
                format!("Flag for manual review - {}", max_action)
            } else {
                format!("Monitor closely - {}", max_action)
            })
        } else {
            None
        }
    }

    /// Generate a security alert for logging/monitoring systems
    ///
    /// Returns a formatted alert message if account should be flagged
    pub fn generate_security_alert(&self, player_id: &str) -> Option<String> {
        if !self.should_flag_account(0.7) {
            return None;
        }

        let action = self.get_recommended_action().unwrap_or_default();
        let mut alerts = Vec::new();

        if let Some(ref report) = self.cherry_picking_report {
            if report.is_suspicious {
                alerts.push(format!(
                    "Cherry-picking (confidence: {:.1}%): {}",
                    report.confidence * 100.0,
                    report.detected_patterns.join(", ")
                ));
            }
        }

        if let Some(ref report) = self.sandbagging_report {
            if report.is_suspicious {
                alerts.push(format!(
                    "Sandbagging (confidence: {:.1}%): {}",
                    report.confidence * 100.0,
                    report.detected_patterns.join(", ")
                ));
            }
        }

        Some(format!(
            "🚨 SECURITY ALERT - Player: {}\nAction: {}\nDetected: {}\nSession RTP: {:.1}% (target: 85%)",
            player_id,
            action,
            alerts.join("; "),
            (self.total_won / self.total_wagered) * 100.0
        ))
    }
}

/// Run a player gaming session simulation
///
/// # Arguments
/// * `player` - Mutable reference to player (skill will be updated)
/// * `config` - Session configuration parameters
///
/// # Returns
/// SessionResult with all shot outcomes and final statistics
pub fn run_session(player: &mut Player, config: SessionConfig) -> SessionResult {
    let mut rng = rand::thread_rng();
    let mut shots = Vec::with_capacity(config.num_shots);
    let mut total_wagered = 0.0;
    let mut total_won = 0.0;
    let mut num_kalman_updates = 0;
    let mut num_high_stakes_shots = 0;

    for shot_num in 0..config.num_shots {
        // Select hole based on strategy
        let hole = select_hole(&config.hole_selection, &mut rng);

        // Determine wager for this shot
        let wager = rng.gen_range(config.wager_min..=config.wager_max);

        // Get player's current skill for this hole's category
        let skill_profile = player.get_skill_for_hole(hole);
        let current_sigma = skill_profile.kalman_filter.estimate;

        // Calculate P_max for current skill level
        let p_max = player.calculate_p_max(hole);

        // Simulate or use manual miss distance
        let (miss_distance, is_fat_tail) = if let Some(ref dev_mode) = config.developer_mode {
            if let Some(manual_dist) = dev_mode.manual_miss_distance {
                (manual_dist, false)
            } else {
                simulate_shot(current_sigma, config.fat_tail_prob, config.fat_tail_mult)
            }
        } else {
            simulate_shot(current_sigma, config.fat_tail_prob, config.fat_tail_mult)
        };

        // Calculate payout
        let payout_multiplier = hole.calculate_payout(miss_distance, p_max);
        let payout_amount = payout_multiplier * wager;

        // Create shot outcome
        let outcome = ShotOutcome::new(
            miss_distance,
            payout_multiplier,
            wager,
            hole.id,
            is_fat_tail,
        );

        total_wagered += wager;
        total_won += payout_amount;
        shots.push(outcome);

        // SECURITY FIX: Track wager for lifetime average (cross-session detection)
        player.track_wager(wager);

        // Add shot to batch (unless Kalman is disabled)
        if config.developer_mode.as_ref().map_or(true, |dm| !dm.disable_kalman) {
            // SECURITY FIX: Use lifetime average wager if available, otherwise use session average
            let lifetime_avg = player.get_lifetime_avg_wager();
            let session_avg_wager = if shot_num > 0 {
                total_wagered / (shot_num + 1) as f64
            } else {
                wager
            };

            // Use the more conservative of lifetime or session average
            let reference_avg = if lifetime_avg > 0.0 {
                lifetime_avg.max(session_avg_wager)
            } else {
                session_avg_wager
            };

            // SECURITY FIX: More aggressive high-stakes detection (2x reference average instead of 10x batch average)
            let is_high_stakes = wager >= 2.0 * reference_avg;

            if is_high_stakes {
                num_high_stakes_shots += 1;
                // Process existing batch first if it has shots
                let skill = player.get_skill_for_hole(hole);
                if !skill.shot_batch.is_empty() {
                    player.update_skill(hole, p_max);
                    num_kalman_updates += 1;
                }
            }

            // Add shot to batch
            let batch_full = player.add_shot_to_batch(hole, miss_distance, wager);

            // Update if batch is full or this is a high-stakes shot
            if batch_full || is_high_stakes {
                player.update_skill(hole, p_max);
                num_kalman_updates += 1;
            }
        }
    }

    // Process any remaining shots in batches at end of session
    if config.developer_mode.as_ref().map_or(true, |dm| !dm.disable_kalman) {
        for hole in HOLE_CONFIGURATIONS.iter() {
            let skill = player.get_skill_for_hole(hole);
            if !skill.shot_batch.is_empty() {
                let p_max = player.calculate_p_max(hole);
                player.update_skill(hole, p_max);
                num_kalman_updates += 1;
            }
        }
    }

    // Collect final skill profiles
    let final_skill_profiles = player
        .skill_profiles
        .iter()
        .map(|(cat, profile)| {
            (format!("{:?}", cat), profile.kalman_filter.estimate)
        })
        .collect();

    let net_gain_loss = total_won - total_wagered;
    let session_house_edge = if total_wagered > 0.0 {
        1.0 - (total_won / total_wagered)
    } else {
        0.0
    };

    // SECURITY FIX: Run anti-cheat detection on session results
    let cherry_picking_report = if shots.len() >= 10 {
        Some(detect_cherry_picking(&shots))
    } else {
        None
    };

    let sandbagging_report = if shots.len() >= 20 {
        Some(detect_sandbagging(&shots))
    } else {
        None
    };

    SessionResult {
        total_wagered,
        total_won,
        net_gain_loss,
        shots,
        final_skill_profiles,
        session_house_edge,
        num_kalman_updates,
        num_high_stakes_shots,
        cherry_picking_report,
        sandbagging_report,
    }
}

/// Select a hole based on the configured strategy
fn select_hole<'a>(selection: &HoleSelection, rng: &mut impl Rng) -> &'a Hole {
    match selection {
        HoleSelection::Random => {
            let idx = rng.gen_range(0..HOLE_CONFIGURATIONS.len());
            &HOLE_CONFIGURATIONS[idx]
        }
        HoleSelection::Weighted(weights) => {
            let roll: f64 = rng.gen();
            let mut cumulative = 0.0;
            for (hole_id, prob) in weights {
                cumulative += prob;
                if roll < cumulative {
                    return get_hole_by_id(*hole_id).expect("Invalid hole_id in weights");
                }
            }
            // Fallback to last hole if rounding errors occur
            let last_id = weights.last().map(|(id, _)| *id).unwrap_or(1);
            get_hole_by_id(last_id).expect("Invalid hole_id in weights")
        }
        HoleSelection::Fixed(hole_id) => {
            get_hole_by_id(*hole_id).expect("Invalid hole_id in Fixed selection")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.num_shots, 100);
        assert_eq!(config.wager_min, 5.0);
        assert_eq!(config.wager_max, 10.0);
        assert_eq!(config.fat_tail_prob, 0.02);
        assert_eq!(config.fat_tail_mult, 3.0);
    }

    #[test]
    fn test_hole_selection_fixed() {
        let selection = HoleSelection::Fixed(3);
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let hole = select_hole(&selection, &mut rng);
            assert_eq!(hole.id, 3);
        }
    }

    #[test]
    fn test_hole_selection_random() {
        let selection = HoleSelection::Random;
        let mut rng = rand::thread_rng();
        let mut seen_holes = std::collections::HashSet::new();

        // Should see multiple different holes over 100 selections
        for _ in 0..100 {
            let hole = select_hole(&selection, &mut rng);
            seen_holes.insert(hole.id);
        }

        assert!(seen_holes.len() > 1, "Random selection should pick different holes");
    }

    #[test]
    fn test_hole_selection_weighted() {
        // 100% weight on hole 5
        let selection = HoleSelection::Weighted(vec![(5, 1.0)]);
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let hole = select_hole(&selection, &mut rng);
            assert_eq!(hole.id, 5);
        }
    }

    #[test]
    fn test_run_session_basic() {
        let mut player = Player::new("test_player".to_string(), 15);
        let config = SessionConfig {
            num_shots: 10,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: None,
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        assert_eq!(result.shots.len(), 10);
        assert!(result.total_wagered >= 50.0 && result.total_wagered <= 100.0);
        assert_eq!(result.net_gain_loss, result.total_won - result.total_wagered);
        // House edge can be negative in individual sessions (player wins)
        // Typically should be between -5.0 and 1.0 for small sample sizes
        assert!(result.session_house_edge >= -5.0 && result.session_house_edge <= 1.0);
    }

    #[test]
    fn test_run_session_developer_mode_manual_miss() {
        let mut player = Player::new("test_player".to_string(), 15);
        let config = SessionConfig {
            num_shots: 5,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: Some(5.0), // Always miss by 5ft
                disable_kalman: false,
            }),
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        // All shots should have exactly 5ft miss distance
        for shot in &result.shots {
            assert_eq!(shot.miss_distance_ft, 5.0);
        }
    }

    #[test]
    fn test_run_session_developer_mode_disable_kalman() {
        let mut player = Player::new("test_player".to_string(), 15);
        let hole = get_hole_by_id(4).unwrap();
        let initial_sigma = player.get_skill_for_hole(hole).kalman_filter.estimate;

        let config = SessionConfig {
            num_shots: 20,
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: None,
                disable_kalman: true, // No updates
            }),
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        assert_eq!(result.num_kalman_updates, 0);

        // Skill should not have changed
        let final_sigma = player.get_skill_for_hole(hole).kalman_filter.estimate;
        assert_eq!(initial_sigma, final_sigma);
    }

    #[test]
    fn test_session_result_calculations() {
        let result = SessionResult {
            total_wagered: 100.0,
            total_won: 88.0,
            net_gain_loss: -12.0,
            shots: vec![
                ShotOutcome::new(10.0, 2.0, 10.0, 1, false),
                ShotOutcome::new(30.0, 0.0, 10.0, 1, false),
                ShotOutcome::new(15.0, 1.5, 10.0, 1, false),
                ShotOutcome::new(8.0, 2.3, 10.0, 1, false),
                ShotOutcome::new(25.0, 0.0, 10.0, 1, false),
                ShotOutcome::new(12.0, 1.8, 10.0, 1, false),
                ShotOutcome::new(20.0, 0.0, 10.0, 1, false),
                ShotOutcome::new(9.0, 2.1, 10.0, 1, false),
                ShotOutcome::new(30.0, 0.0, 10.0, 1, false),
                ShotOutcome::new(11.0, 1.9, 10.0, 1, false),
            ],
            final_skill_profiles: HashMap::new(),
            session_house_edge: 0.12,
            num_kalman_updates: 1,
            num_high_stakes_shots: 0,
            cherry_picking_report: None,
            sandbagging_report: None,
        };

        assert_eq!(result.house_edge_percent(), 12.0);
        assert_eq!(result.avg_wager(), 10.0);
        // 6 out of 10 shots have payout > 0 (shots 1, 3, 4, 6, 8, 10)
        assert_eq!(result.win_rate(), 60.0);
    }

    #[test]
    fn test_session_kalman_updates_occur() {
        let mut player = Player::new("test_player".to_string(), 20);
        let config = SessionConfig {
            num_shots: 25, // Should trigger multiple batch updates
            wager_min: 5.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(3),
            developer_mode: None,
            ..Default::default()
        };

        let result = run_session(&mut player, config);

        // Should have at least some Kalman updates
        assert!(result.num_kalman_updates > 0,
            "Expected Kalman updates, got {}", result.num_kalman_updates);
    }

    #[test]
    fn test_config_validation_normal() {
        let config = SessionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_wager_range() {
        let config = SessionConfig {
            wager_min: 100.0,
            wager_max: 10.0, // min > max
            ..Default::default()
        };

        match config.validate() {
            Err(SessionConfigError::InvalidWagerRange(_)) => {},
            _ => panic!("Expected InvalidWagerRange error"),
        }
    }

    #[test]
    fn test_config_validation_negative_wager() {
        let config = SessionConfig {
            wager_min: -5.0,
            wager_max: 10.0,
            ..Default::default()
        };

        match config.validate() {
            Err(SessionConfigError::InvalidWagerRange(_)) => {},
            _ => panic!("Expected InvalidWagerRange error"),
        }
    }

    #[test]
    fn test_config_validation_zero_shots() {
        let config = SessionConfig {
            num_shots: 0,
            ..Default::default()
        };

        match config.validate() {
            Err(SessionConfigError::InvalidConfig(_)) => {},
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_config_validation_invalid_fat_tail_prob() {
        let config = SessionConfig {
            fat_tail_prob: 1.5, // > 1.0
            ..Default::default()
        };

        match config.validate() {
            Err(SessionConfigError::InvalidConfig(_)) => {},
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_config_validation_dev_mode_without_production() {
        // Without production env var, developer mode should be allowed
        std::env::remove_var("CONTINUUM_ENV");
        std::env::remove_var("CONTINUUM_PRODUCTION");

        let config = SessionConfig {
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: Some(20.0),
                disable_kalman: false,
            }),
            ..Default::default()
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_dev_mode_in_production() {
        // Set production environment
        std::env::set_var("CONTINUUM_ENV", "production");

        let config = SessionConfig {
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: Some(20.0),
                disable_kalman: false,
            }),
            ..Default::default()
        };

        match config.validate() {
            Err(SessionConfigError::DeveloperModeInProduction) => {},
            _ => panic!("Expected DeveloperModeInProduction error"),
        }

        // Clean up
        std::env::remove_var("CONTINUUM_ENV");
    }

    #[test]
    fn test_is_production_detection() {
        // Test CONTINUUM_ENV
        std::env::set_var("CONTINUUM_ENV", "production");
        assert!(SessionConfig::is_production());
        std::env::set_var("CONTINUUM_ENV", "PRODUCTION");
        assert!(SessionConfig::is_production());
        std::env::set_var("CONTINUUM_ENV", "development");
        assert!(!SessionConfig::is_production());
        std::env::remove_var("CONTINUUM_ENV");

        // Test CONTINUUM_PRODUCTION
        std::env::set_var("CONTINUUM_PRODUCTION", "true");
        assert!(SessionConfig::is_production());
        std::env::set_var("CONTINUUM_PRODUCTION", "TRUE");
        assert!(SessionConfig::is_production());
        std::env::set_var("CONTINUUM_PRODUCTION", "1");
        assert!(SessionConfig::is_production());
        std::env::set_var("CONTINUUM_PRODUCTION", "false");
        assert!(!SessionConfig::is_production());
        std::env::remove_var("CONTINUUM_PRODUCTION");

        // Test both unset
        assert!(!SessionConfig::is_production());
    }

    #[test]
    fn test_account_flagging_no_suspicious_activity() {
        use crate::anti_cheat::AnomalyReport;

        let result = SessionResult {
            total_wagered: 100.0,
            total_won: 85.0,
            net_gain_loss: -15.0,
            shots: vec![],
            final_skill_profiles: HashMap::new(),
            session_house_edge: 0.15,
            num_kalman_updates: 5,
            num_high_stakes_shots: 0,
            cherry_picking_report: Some(AnomalyReport {
                is_suspicious: false,
                confidence: 0.2,
                detected_patterns: vec![],
                recommended_action: "Continue monitoring".to_string(),
            }),
            sandbagging_report: None,
        };

        assert!(!result.should_flag_account(0.7));
        assert!(result.get_recommended_action().is_none());
        assert!(result.generate_security_alert("test_player").is_none());
    }

    #[test]
    fn test_account_flagging_cherry_picking_detected() {
        use crate::anti_cheat::AnomalyReport;

        let result = SessionResult {
            total_wagered: 100.0,
            total_won: 109.0,
            net_gain_loss: 9.0,
            shots: vec![],
            final_skill_profiles: HashMap::new(),
            session_house_edge: -0.09,
            num_kalman_updates: 3,
            num_high_stakes_shots: 5,
            cherry_picking_report: Some(AnomalyReport {
                is_suspicious: true,
                confidence: 0.85,
                detected_patterns: vec![
                    "Strong positive correlation: high wagers on good shots".to_string()
                ],
                recommended_action: "Limit max wager variance per session".to_string(),
            }),
            sandbagging_report: None,
        };

        assert!(result.should_flag_account(0.7));
        assert!(result.get_recommended_action().is_some());

        let action = result.get_recommended_action().unwrap();
        assert!(action.contains("Flag for manual review"));

        let alert = result.generate_security_alert("test_player").unwrap();
        assert!(alert.contains("SECURITY ALERT"));
        assert!(alert.contains("test_player"));
        assert!(alert.contains("Cherry-picking"));
    }

    #[test]
    fn test_account_flagging_high_confidence_threshold() {
        use crate::anti_cheat::AnomalyReport;

        let result = SessionResult {
            total_wagered: 100.0,
            total_won: 150.0,
            net_gain_loss: 50.0,
            shots: vec![],
            final_skill_profiles: HashMap::new(),
            session_house_edge: -0.50,
            num_kalman_updates: 2,
            num_high_stakes_shots: 10,
            cherry_picking_report: Some(AnomalyReport {
                is_suspicious: true,
                confidence: 0.95,
                detected_patterns: vec![
                    "Extreme exploitation detected".to_string()
                ],
                recommended_action: "Immediate suspension".to_string(),
            }),
            sandbagging_report: None,
        };

        assert!(result.should_flag_account(0.7));

        let action = result.get_recommended_action().unwrap();
        assert!(action.contains("URGENT"));
        assert!(action.contains("Immediate suspension"));
    }

    #[test]
    fn test_account_flagging_custom_threshold() {
        use crate::anti_cheat::AnomalyReport;

        let result = SessionResult {
            total_wagered: 100.0,
            total_won: 92.0,
            net_gain_loss: -8.0,
            shots: vec![],
            final_skill_profiles: HashMap::new(),
            session_house_edge: 0.08,
            num_kalman_updates: 4,
            num_high_stakes_shots: 1,
            sandbagging_report: Some(AnomalyReport {
                is_suspicious: true,
                confidence: 0.65, // Below default 0.7 threshold
                detected_patterns: vec![
                    "Moderate variance pattern".to_string()
                ],
                recommended_action: "Monitor".to_string(),
            }),
            cherry_picking_report: None,
        };

        // Should not flag with default threshold
        assert!(!result.should_flag_account(0.7));

        // Should flag with lower threshold
        assert!(result.should_flag_account(0.6));
    }
}