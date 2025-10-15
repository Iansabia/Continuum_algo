/// Anti-Cheat Detection Module
///
/// Provides detection mechanisms for various cheating strategies including:
/// - Sandbagging (intentional poor performance to inflate P_max)
/// - Cherry-picking (only high wagers on good shots)
/// - Sudden skill jumps (potential account sharing)
/// - Pattern-based exploitation

use crate::models::shot::ShotOutcome;
use serde::{Deserialize, Serialize};

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub is_suspicious: bool,
    pub confidence: f64, // 0.0-1.0
    pub detected_patterns: Vec<String>,
    pub recommended_action: String,
}

/// Detect sandbagging pattern
///
/// Indicators:
/// - High variance in miss distances
/// - Low wagers on bad shots
/// - Sudden high wagers after establishing poor baseline
pub fn detect_sandbagging(shots: &[ShotOutcome]) -> AnomalyReport {
    if shots.len() < 20 {
        return AnomalyReport {
            is_suspicious: false,
            confidence: 0.0,
            detected_patterns: vec![],
            recommended_action: "Insufficient data".to_string(),
        };
    }

    let mut patterns = Vec::new();
    let mut confidence = 0.0;

    // Check variance in miss distances
    let mean_miss: f64 = shots.iter().map(|s| s.miss_distance_ft).sum::<f64>() / shots.len() as f64;
    let variance: f64 = shots.iter()
        .map(|s| (s.miss_distance_ft - mean_miss).powi(2))
        .sum::<f64>() / shots.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev > mean_miss * 0.8 {
        patterns.push(format!("High variance in shot quality (σ={:.1})", std_dev));
        confidence += 0.3;
    }

    // Check correlation between wager size and shot quality
    let correlation = calculate_wager_quality_correlation(shots);
    if correlation < -0.5 {
        patterns.push(format!("Negative correlation: high wagers on bad shots ({:.2})", correlation));
        confidence += 0.4;
    }

    // Check for wager pattern changes
    if shots.len() >= 50 {
        let first_half_avg_wager: f64 = shots[..25].iter().map(|s| s.wager).sum::<f64>() / 25.0;
        let second_half_avg_wager: f64 = shots[25..].iter().map(|s| s.wager).sum::<f64>() / 25.0;

        if second_half_avg_wager > first_half_avg_wager * 5.0 {
            patterns.push("Sudden wager increase after baseline period".to_string());
            confidence += 0.3;
        }
    }

    let is_suspicious = confidence > 0.6;
    let recommended_action = if is_suspicious {
        "Flag for manual review - potential sandbagging".to_string()
    } else {
        "Continue monitoring".to_string()
    };

    AnomalyReport {
        is_suspicious,
        confidence,
        detected_patterns: patterns,
        recommended_action,
    }
}

/// Detect cherry-picking (bet timing exploitation)
///
/// Indicators:
/// - Low wagers correlated with poor shots
/// - High wagers correlated with good shots
pub fn detect_cherry_picking(shots: &[ShotOutcome]) -> AnomalyReport {
    if shots.len() < 10 {
        return AnomalyReport {
            is_suspicious: false,
            confidence: 0.0,
            detected_patterns: vec![],
            recommended_action: "Insufficient data".to_string(),
        };
    }

    let mut patterns = Vec::new();
    let mut confidence = 0.0;

    // Calculate correlation between wager and payout multiplier
    let correlation = calculate_wager_quality_correlation(shots);

    if correlation > 0.5 {
        patterns.push(format!("Strong positive correlation: high wagers on good shots ({:.2})", correlation));
        confidence += 0.5;
    }

    // Check for bimodal wager distribution
    let wagers: Vec<f64> = shots.iter().map(|s| s.wager).collect();
    let (low_wagers, high_wagers) = partition_wagers(&wagers);

    if !low_wagers.is_empty() && !high_wagers.is_empty() {
        let low_avg_mult: f64 = shots.iter()
            .filter(|s| s.wager < wagers.iter().sum::<f64>() / wagers.len() as f64)
            .map(|s| s.multiplier)
            .sum::<f64>() / low_wagers.len() as f64;

        let high_avg_mult: f64 = shots.iter()
            .filter(|s| s.wager >= wagers.iter().sum::<f64>() / wagers.len() as f64)
            .map(|s| s.multiplier)
            .sum::<f64>() / high_wagers.len() as f64;

        if high_avg_mult > low_avg_mult * 1.5 {
            patterns.push("Bimodal betting: significantly better multipliers on high wagers".to_string());
            confidence += 0.4;
        }
    }

    let is_suspicious = confidence > 0.6;
    let recommended_action = if is_suspicious {
        "Limit max wager variance per session".to_string()
    } else {
        "Normal betting pattern".to_string()
    };

    AnomalyReport {
        is_suspicious,
        confidence,
        detected_patterns: patterns,
        recommended_action,
    }
}

/// Detect sudden skill jumps (potential account sharing)
///
/// Requires historical shots from previous sessions for comparison
pub fn detect_skill_jump(
    historical_shots: &[ShotOutcome],
    recent_shots: &[ShotOutcome],
) -> AnomalyReport {
    if historical_shots.len() < 20 || recent_shots.len() < 10 {
        return AnomalyReport {
            is_suspicious: false,
            confidence: 0.0,
            detected_patterns: vec![],
            recommended_action: "Insufficient data for comparison".to_string(),
        };
    }

    let mut patterns = Vec::new();
    let mut confidence = 0.0;

    // Compare average performance
    let historical_avg_miss: f64 = historical_shots.iter()
        .map(|s| s.miss_distance_ft)
        .sum::<f64>() / historical_shots.len() as f64;

    let recent_avg_miss: f64 = recent_shots.iter()
        .map(|s| s.miss_distance_ft)
        .sum::<f64>() / recent_shots.len() as f64;

    let improvement_rate = (historical_avg_miss - recent_avg_miss) / historical_avg_miss;

    if improvement_rate > 0.4 {
        patterns.push(format!("Sudden skill improvement: {:.1}% better", improvement_rate * 100.0));
        confidence += 0.5;
    }

    // Check wager increase coinciding with skill jump
    let historical_avg_wager: f64 = historical_shots.iter().map(|s| s.wager).sum::<f64>() / historical_shots.len() as f64;
    let recent_avg_wager: f64 = recent_shots.iter().map(|s| s.wager).sum::<f64>() / recent_shots.len() as f64;

    if recent_avg_wager > historical_avg_wager * 3.0 && improvement_rate > 0.3 {
        patterns.push("Skill jump coincides with increased wagers".to_string());
        confidence += 0.4;
    }

    let is_suspicious = confidence > 0.7;
    let recommended_action = if is_suspicious {
        "URGENT: Flag for immediate review - possible account sharing".to_string()
    } else if confidence > 0.5 {
        "Monitor closely for continued pattern".to_string()
    } else {
        "Normal skill progression".to_string()
    };

    AnomalyReport {
        is_suspicious,
        confidence,
        detected_patterns: patterns,
        recommended_action,
    }
}

/// Calculate correlation between wager size and shot quality (inverse of miss distance)
fn calculate_wager_quality_correlation(shots: &[ShotOutcome]) -> f64 {
    if shots.len() < 2 {
        return 0.0;
    }

    let n = shots.len() as f64;
    let mean_wager: f64 = shots.iter().map(|s| s.wager).sum::<f64>() / n;
    let mean_quality: f64 = shots.iter().map(|s| s.multiplier).sum::<f64>() / n;

    let numerator: f64 = shots.iter()
        .map(|s| (s.wager - mean_wager) * (s.multiplier - mean_quality))
        .sum();

    let wager_variance: f64 = shots.iter()
        .map(|s| (s.wager - mean_wager).powi(2))
        .sum();

    let quality_variance: f64 = shots.iter()
        .map(|s| (s.multiplier - mean_quality).powi(2))
        .sum();

    if wager_variance == 0.0 || quality_variance == 0.0 {
        return 0.0;
    }

    numerator / (wager_variance.sqrt() * quality_variance.sqrt())
}

/// Partition wagers into low and high groups
fn partition_wagers(wagers: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let median = {
        let mut sorted = wagers.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[sorted.len() / 2]
    };

    let low: Vec<f64> = wagers.iter().filter(|&&w| w < median).copied().collect();
    let high: Vec<f64> = wagers.iter().filter(|&&w| w >= median).copied().collect();

    (low, high)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_normal_play() {
        let shots: Vec<ShotOutcome> = (0..50)
            .map(|i| ShotOutcome {
                miss_distance_ft: 50.0 + (i % 10) as f64 * 5.0,
                multiplier: 2.0,
                payout: 20.0,
                wager: 10.0,
                hole_id: 4,
                is_fat_tail: false,
            })
            .collect();

        let report = detect_sandbagging(&shots);
        assert!(!report.is_suspicious, "Normal play should not be flagged");
    }

    #[test]
    fn test_detect_obvious_sandbagging() {
        let mut shots = Vec::new();

        // Phase 1: Poor shots with low wagers
        for _ in 0..25 {
            shots.push(ShotOutcome {
                miss_distance_ft: 100.0,
                multiplier: 0.5,
                payout: 0.5,
                wager: 1.0,
                hole_id: 4,
                is_fat_tail: false,
            });
        }

        // Phase 2: Sudden high wagers
        for _ in 0..25 {
            shots.push(ShotOutcome {
                miss_distance_ft: 90.0,
                multiplier: 1.0,
                payout: 100.0,
                wager: 100.0,
                hole_id: 4,
                is_fat_tail: false,
            });
        }

        let report = detect_sandbagging(&shots);
        assert!(report.is_suspicious, "Obvious sandbagging should be detected");
        assert!(report.confidence > 0.6);
    }
}
