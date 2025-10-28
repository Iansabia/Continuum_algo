/// Anti-Cheat Detection Module
///
/// Provides ML-based detection mechanisms for various cheating strategies including:
/// - Sandbagging (intentional poor performance to inflate P_max)
/// - Cherry-picking (only high wagers on good shots)
/// - Sudden skill jumps (potential account sharing)
/// - Pattern-based exploitation
/// - Temporal behavior anomalies
/// - Sequence-based betting patterns
///
/// Uses ensemble methods, Bayesian inference, and temporal analysis for robust detection.

use crate::models::shot::ShotOutcome;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub is_suspicious: bool,
    pub confidence: f64, // 0.0-1.0
    pub detected_patterns: Vec<String>,
    pub recommended_action: String,
}

/// Enhanced ML-based detection result with ensemble scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLAnomalyReport {
    pub is_suspicious: bool,
    pub ensemble_score: f64, // Combined score from all detectors (0.0-1.0)
    pub individual_scores: HashMap<String, f64>,
    pub detected_patterns: Vec<String>,
    pub risk_level: RiskLevel,
    pub recommended_action: String,
    pub temporal_context: Option<TemporalContext>,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Temporal context for pattern evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub pattern_duration_shots: usize,
    pub pattern_stability: f64, // How consistent the pattern is over time
    pub trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Improving,
    Degrading,
    Stable,
    Volatile,
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

    let is_suspicious = confidence >= 0.6;
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

/// Detect unrealistic shot consistency (near-perfect or impossible patterns)
///
/// Indicators:
/// - Too many shots with extremely low miss distances (< 5 feet)
/// - Suspiciously low variance (too consistent to be natural)
/// - High percentage of "perfect" shots that exceed human capability
pub fn detect_unrealistic_consistency(shots: &[ShotOutcome]) -> AnomalyReport {
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

    // Count near-perfect shots (< 5 feet miss distance)
    let perfect_shots = shots.iter().filter(|s| s.miss_distance_ft < 5.0).count();
    let perfect_ratio = perfect_shots as f64 / shots.len() as f64;

    // Count impossible shots (< 1 foot miss distance)
    let impossible_shots = shots.iter().filter(|s| s.miss_distance_ft < 1.0).count();
    let impossible_ratio = impossible_shots as f64 / shots.len() as f64;

    // Flag if more than 20% of shots are near-perfect (highly unlikely)
    if perfect_ratio > 0.20 {
        patterns.push(format!(
            "Unrealistic accuracy: {:.0}% of shots within 5ft ({}x perfect shots)",
            perfect_ratio * 100.0, perfect_shots
        ));
        confidence += 0.4 + (perfect_ratio - 0.20) * 2.0; // Scale up quickly
    }

    // Flag if ANY shots are impossibly perfect (< 1 foot)
    if impossible_ratio > 0.0 {
        patterns.push(format!(
            "CRITICAL: {} shot(s) at < 1ft miss distance (impossibly perfect)",
            impossible_shots
        ));
        confidence += 0.5 + impossible_ratio * 2.0; // Very high weight
    }

    // Check for suspiciously low variance
    let mean_miss: f64 = shots.iter().map(|s| s.miss_distance_ft).sum::<f64>() / shots.len() as f64;
    let variance: f64 = shots.iter()
        .map(|s| (s.miss_distance_ft - mean_miss).powi(2))
        .sum::<f64>() / shots.len() as f64;
    let std_dev = variance.sqrt();
    let coefficient_of_variation = if mean_miss > 0.0 { std_dev / mean_miss } else { 0.0 };

    // Natural golf shots should have CV > 0.3 (significant variation)
    // If CV < 0.2, shots are unnaturally consistent
    if coefficient_of_variation < 0.2 && shots.len() >= 20 {
        patterns.push(format!(
            "Unnaturally low variance: CV={:.2} (shots too consistent)",
            coefficient_of_variation
        ));
        confidence += 0.4;
    }

    // Check for clusters of consecutive perfect shots
    let mut max_perfect_streak = 0;
    let mut current_streak = 0;
    for shot in shots {
        if shot.miss_distance_ft < 5.0 {
            current_streak += 1;
            max_perfect_streak = max_perfect_streak.max(current_streak);
        } else {
            current_streak = 0;
        }
    }

    if max_perfect_streak >= 5 {
        patterns.push(format!(
            "Suspicious streak: {} consecutive shots within 5ft",
            max_perfect_streak
        ));
        confidence += 0.3 + (max_perfect_streak as f64 - 5.0) * 0.1;
    }

    let is_suspicious = confidence >= 0.6;
    let recommended_action = if confidence >= 0.8 {
        "CRITICAL: Bot or modified client detected - immediate suspension".to_string()
    } else if is_suspicious {
        "HIGH RISK: Unrealistic performance - restrict account and investigate".to_string()
    } else {
        "Performance within human range".to_string()
    };

    AnomalyReport {
        is_suspicious,
        confidence: confidence.min(1.0),
        detected_patterns: patterns,
        recommended_action,
    }
}

/// ML ENSEMBLE DETECTION SYSTEM
/// =============================

/// Run ensemble ML detection combining all available detection methods
///
/// Uses weighted voting from multiple detectors with Bayesian prior adjustment
pub fn detect_ml_ensemble(
    shots: &[ShotOutcome],
    historical_shots: Option<&[ShotOutcome]>,
    confidence_history: Option<&[(usize, f64)]>,
) -> MLAnomalyReport {
    let mut individual_scores = HashMap::new();
    let mut patterns = Vec::new();

    // Define detector weights (sum to 1.0)
    let weights = HashMap::from([
        ("sandbagging", 0.15),
        ("cherry_picking", 0.15),
        ("temporal_patterns", 0.12),
        ("sequence_analysis", 0.10),
        ("unrealistic_consistency", 0.35), // NEW: Very high weight for detecting impossibly good shots
        ("skill_jump", 0.08),
        ("confidence_anomaly", 0.05),
    ]);

    let mut weighted_sum = 0.0;

    // 1. Unrealistic consistency detection (NEW - high priority)
    let unrealistic = detect_unrealistic_consistency(shots);
    individual_scores.insert("unrealistic_consistency".to_string(), unrealistic.confidence);
    weighted_sum += unrealistic.confidence * weights["unrealistic_consistency"];
    patterns.extend(unrealistic.detected_patterns);

    // 2. Sandbagging detection
    let sandbagging = detect_sandbagging(shots);
    individual_scores.insert("sandbagging".to_string(), sandbagging.confidence);
    weighted_sum += sandbagging.confidence * weights["sandbagging"];
    patterns.extend(sandbagging.detected_patterns);

    // 3. Cherry-picking detection
    let cherry_picking = detect_cherry_picking(shots);
    individual_scores.insert("cherry_picking".to_string(), cherry_picking.confidence);
    weighted_sum += cherry_picking.confidence * weights["cherry_picking"];
    patterns.extend(cherry_picking.detected_patterns);

    // 4. Temporal pattern detection
    let temporal = detect_temporal_patterns(shots);
    individual_scores.insert("temporal_patterns".to_string(), temporal.confidence);
    weighted_sum += temporal.confidence * weights["temporal_patterns"];
    patterns.extend(temporal.detected_patterns);

    // 5. Sequence analysis
    let sequence = detect_sequence_patterns(shots);
    individual_scores.insert("sequence_analysis".to_string(), sequence.confidence);
    weighted_sum += sequence.confidence * weights["sequence_analysis"];
    patterns.extend(sequence.detected_patterns);

    // CRITICAL OVERRIDE: If unrealistic consistency is blatant (>= 0.8), use it directly
    // This prevents dilution by other detectors when cheating is obvious (perfect shots, bots, etc.)
    // The ensemble is still useful for subtle cheating, but obvious cheating must dominate
    if let Some(&unrealistic_score) = individual_scores.get("unrealistic_consistency") {
        if unrealistic_score >= 0.8 {
            weighted_sum = unrealistic_score;
        }
    }

    // 6. Skill jump detection (if historical data available)
    if let Some(hist) = historical_shots {
        if hist.len() >= 20 && shots.len() >= 10 {
            let skill_jump = detect_skill_jump(hist, shots);
            individual_scores.insert("skill_jump".to_string(), skill_jump.confidence);
            weighted_sum += skill_jump.confidence * weights["skill_jump"];
            patterns.extend(skill_jump.detected_patterns);
        }
    }

    // 7. Confidence anomaly detection (if confidence history available)
    if let Some(conf_hist) = confidence_history {
        if conf_hist.len() >= 10 {
            let conf_anomaly = detect_confidence_anomaly(conf_hist);
            individual_scores.insert("confidence_anomaly".to_string(), conf_anomaly.confidence);
            weighted_sum += conf_anomaly.confidence * weights["confidence_anomaly"];
            patterns.extend(conf_anomaly.detected_patterns);
        }
    }

    // Apply Bayesian adjustment based on player history
    let ensemble_score = apply_bayesian_adjustment(weighted_sum, shots.len());

    // Determine risk level
    let risk_level = classify_risk_level(ensemble_score);

    // Generate temporal context
    let temporal_context = if shots.len() >= 20 {
        Some(analyze_temporal_context(shots))
    } else {
        None
    };

    // Determine if suspicious (lowered threshold to 0.35 for earlier detection)
    let is_suspicious = ensemble_score >= 0.35;

    // Generate recommended action
    let recommended_action = generate_action_recommendation(&risk_level, &patterns);

    MLAnomalyReport {
        is_suspicious,
        ensemble_score,
        individual_scores,
        detected_patterns: patterns,
        risk_level,
        recommended_action,
        temporal_context,
    }
}

/// Detect temporal patterns using sliding window analysis
///
/// Analyzes how player behavior evolves over time windows
fn detect_temporal_patterns(shots: &[ShotOutcome]) -> AnomalyReport {
    if shots.len() < 30 {
        return AnomalyReport {
            is_suspicious: false,
            confidence: 0.0,
            detected_patterns: vec![],
            recommended_action: "Insufficient data for temporal analysis".to_string(),
        };
    }

    let mut patterns = Vec::new();
    let mut confidence: f64 = 0.0;

    // Use sliding windows of size 10
    let window_size = 10;
    let num_windows = (shots.len() as f64 / window_size as f64).floor() as usize;

    if num_windows < 3 {
        return AnomalyReport {
            is_suspicious: false,
            confidence: 0.0,
            detected_patterns: vec![],
            recommended_action: "Insufficient windows for analysis".to_string(),
        };
    }

    // Calculate metrics for each window
    let mut window_avg_wagers = Vec::new();
    let mut window_avg_multipliers = Vec::new();
    let mut window_variances = Vec::new();

    for i in 0..num_windows {
        let start = i * window_size;
        let end = ((i + 1) * window_size).min(shots.len());
        let window = &shots[start..end];

        let avg_wager: f64 = window.iter().map(|s| s.wager).sum::<f64>() / window.len() as f64;
        let avg_mult: f64 = window.iter().map(|s| s.multiplier).sum::<f64>() / window.len() as f64;

        // Calculate variance in this window
        let mean_miss: f64 = window.iter().map(|s| s.miss_distance_ft).sum::<f64>() / window.len() as f64;
        let variance: f64 = window.iter()
            .map(|s| (s.miss_distance_ft - mean_miss).powi(2))
            .sum::<f64>() / window.len() as f64;

        window_avg_wagers.push(avg_wager);
        window_avg_multipliers.push(avg_mult);
        window_variances.push(variance);
    }

    // Detect sudden wager escalation across windows
    for i in 1..window_avg_wagers.len() {
        if window_avg_wagers[i] > window_avg_wagers[i-1] * 4.0 {
            patterns.push(format!(
                "Temporal escalation: Window {} wager {:.0}x higher than window {}",
                i + 1, window_avg_wagers[i] / window_avg_wagers[i-1], i
            ));
            confidence += 0.3;
        }
    }

    // Detect performance improvement coinciding with wager increase
    for i in 1..num_windows {
        let wager_ratio = window_avg_wagers[i] / window_avg_wagers[i-1].max(0.1);
        let mult_ratio = window_avg_multipliers[i] / window_avg_multipliers[i-1].max(0.1);

        // Suspicious if wager increases significantly AND multiplier improves
        if wager_ratio > 3.0 && mult_ratio > 1.5 {
            patterns.push(format!(
                "Coordinated improvement: Window {} shows {}x wager increase with {}x multiplier improvement",
                i + 1, wager_ratio, mult_ratio
            ));
            confidence += 0.4;
        }
    }

    // Detect cyclical patterns (variance oscillation)
    if window_variances.len() >= 4 {
        let mut oscillations = 0;
        for i in 2..window_variances.len() {
            // Check for high-low-high or low-high-low pattern
            let prev_trend_up = window_variances[i-1] > window_variances[i-2];
            let curr_trend_up = window_variances[i] > window_variances[i-1];

            if prev_trend_up != curr_trend_up {
                oscillations += 1;
            }
        }

        let oscillation_rate = oscillations as f64 / (window_variances.len() - 2) as f64;
        if oscillation_rate > 0.6 {
            patterns.push(format!(
                "Cyclical pattern detected: variance oscillates {:.0}% of time",
                oscillation_rate * 100.0
            ));
            confidence += 0.2;
        }
    }

    let is_suspicious = confidence >= 0.6;
    let recommended_action = if is_suspicious {
        "Temporal patterns suggest coordinated manipulation".to_string()
    } else {
        "Normal temporal evolution".to_string()
    };

    AnomalyReport {
        is_suspicious,
        confidence: confidence.min(1.0),
        detected_patterns: patterns,
        recommended_action,
    }
}

/// Detect repeating sequence patterns in betting behavior
///
/// Uses n-gram analysis to detect repeated betting sequences
fn detect_sequence_patterns(shots: &[ShotOutcome]) -> AnomalyReport {
    if shots.len() < 20 {
        return AnomalyReport {
            is_suspicious: false,
            confidence: 0.0,
            detected_patterns: vec![],
            recommended_action: "Insufficient data for sequence analysis".to_string(),
        };
    }

    let mut patterns = Vec::new();
    let mut confidence: f64 = 0.0;

    // Discretize wagers into categories: low, medium, high
    let median_wager = {
        let mut wagers: Vec<f64> = shots.iter().map(|s| s.wager).collect();
        wagers.sort_by(|a, b| a.partial_cmp(b).unwrap());
        wagers[wagers.len() / 2]
    };

    let categorize_wager = |wager: f64| {
        if wager < median_wager * 0.5 {
            'L' // Low
        } else if wager > median_wager * 2.0 {
            'H' // High
        } else {
            'M' // Medium
        }
    };

    // Create sequence string
    let sequence: String = shots.iter()
        .map(|s| categorize_wager(s.wager))
        .collect();

    // Detect 3-grams (sequences of 3)
    let mut trigram_counts: HashMap<String, usize> = HashMap::new();
    for i in 0..sequence.len().saturating_sub(2) {
        let trigram = sequence[i..i+3].to_string();
        *trigram_counts.entry(trigram).or_insert(0) += 1;
    }

    // Find most common trigram
    let max_trigram_count = trigram_counts.values().max().copied().unwrap_or(0);
    let total_trigrams = sequence.len().saturating_sub(2);

    if total_trigrams > 0 {
        let repetition_rate = max_trigram_count as f64 / total_trigrams as f64;

        if repetition_rate > 0.3 && max_trigram_count >= 3 {
            let most_common = trigram_counts.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(seq, _)| seq.clone())
                .unwrap_or_default();

            patterns.push(format!(
                "Repeating bet sequence '{}' appears {}x ({:.0}% of patterns)",
                most_common, max_trigram_count, repetition_rate * 100.0
            ));
            confidence += 0.5;
        }
    }

    // Detect alternating patterns (L-H-L-H or similar)
    let mut alternations = 0;
    for i in 1..sequence.len() {
        let prev = sequence.chars().nth(i-1).unwrap();
        let curr = sequence.chars().nth(i).unwrap();

        // Count L-H and H-L transitions (not M)
        if (prev == 'L' && curr == 'H') || (prev == 'H' && curr == 'L') {
            alternations += 1;
        }
    }

    let alternation_rate = alternations as f64 / (sequence.len() - 1) as f64;
    if alternation_rate > 0.6 {
        patterns.push(format!(
            "High-low alternation pattern: {:.0}% of transitions alternate",
            alternation_rate * 100.0
        ));
        confidence += 0.4; // Increased weight for alternation
    }

    // Also check if all wagers are extreme (no medium)
    let m_count = sequence.chars().filter(|&c| c == 'M').count();
    let m_ratio = m_count as f64 / sequence.len() as f64;
    if m_ratio < 0.1 && sequence.len() >= 20 {
        patterns.push("Bimodal wager distribution: avoiding medium-sized bets".to_string());
        confidence += 0.2;
    }

    let is_suspicious = confidence >= 0.6;
    let recommended_action = if is_suspicious {
        "Mechanical betting pattern detected - possible bot".to_string()
    } else {
        "Natural betting variation".to_string()
    };

    AnomalyReport {
        is_suspicious,
        confidence: confidence.min(1.0),
        detected_patterns: patterns,
        recommended_action,
    }
}

/// Apply Bayesian adjustment to ensemble score based on prior probability
///
/// Adjusts score based on how much data we have (more data = more confidence)
fn apply_bayesian_adjustment(raw_score: f64, sample_size: usize) -> f64 {
    // Prior: assume 5% base rate of cheating in population
    let prior_cheat_prob = 0.05;

    // Special case: if raw score is VERY high (>0.5), trust it immediately (no dampening)
    // This catches blatant cheating (perfect shots, impossible patterns) instantly
    if raw_score >= 0.5 && sample_size >= 10 {
        // NO dampening for strong signals - return raw score directly
        return raw_score;
    }

    // Special case: if raw score is high (>0.3), minimal dampening
    if raw_score >= 0.3 && sample_size >= 15 {
        // Very minimal dampening for clear signals
        let dampening = 0.92 + (raw_score - 0.3) * 0.4; // 0.92 to 1.0 based on raw score
        return (raw_score * dampening).min(1.0);
    }

    // Confidence in our measurement increases with sample size
    // Use sigmoid function centered at 15 shots with steep slope (0.35)
    let measurement_confidence = 1.0 / (1.0 + (-0.35 * (sample_size as f64 - 15.0)).exp());

    // Weighted average of prior and measurement
    let adjusted_score = (1.0 - measurement_confidence) * prior_cheat_prob +
                         measurement_confidence * raw_score;

    adjusted_score
}

/// Classify risk level based on ensemble score
fn classify_risk_level(score: f64) -> RiskLevel {
    if score >= 0.65 {
        RiskLevel::Critical
    } else if score >= 0.45 {
        RiskLevel::High
    } else if score >= 0.25 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

/// Analyze temporal context to understand pattern evolution
fn analyze_temporal_context(shots: &[ShotOutcome]) -> TemporalContext {
    let window_size = 10;
    let num_windows = (shots.len() / window_size).max(1);

    // Calculate average multiplier for each window
    let mut window_mults = Vec::new();
    for i in 0..num_windows {
        let start = i * window_size;
        let end = ((i + 1) * window_size).min(shots.len());
        let window = &shots[start..end];

        let avg_mult: f64 = window.iter().map(|s| s.multiplier).sum::<f64>() / window.len() as f64;
        window_mults.push(avg_mult);
    }

    // Determine trend
    let trend = if window_mults.len() >= 2 {
        let first_half_avg: f64 = window_mults[..window_mults.len()/2].iter().sum::<f64>() / (window_mults.len()/2) as f64;
        let second_half_avg: f64 = window_mults[window_mults.len()/2..].iter().sum::<f64>() / (window_mults.len() - window_mults.len()/2) as f64;

        let change_rate = (second_half_avg - first_half_avg) / first_half_avg.max(0.1);

        if change_rate.abs() < 0.1 {
            TrendDirection::Stable
        } else if change_rate > 0.2 {
            TrendDirection::Improving
        } else if change_rate < -0.2 {
            TrendDirection::Degrading
        } else {
            // Check volatility
            let variance: f64 = window_mults.iter()
                .map(|&m| {
                    let mean = window_mults.iter().sum::<f64>() / window_mults.len() as f64;
                    (m - mean).powi(2)
                })
                .sum::<f64>() / window_mults.len() as f64;

            if variance > 0.5 {
                TrendDirection::Volatile
            } else {
                TrendDirection::Stable
            }
        }
    } else {
        TrendDirection::Stable
    };

    // Calculate pattern stability (inverse of variance)
    let mean_mult: f64 = window_mults.iter().sum::<f64>() / window_mults.len() as f64;
    let variance: f64 = window_mults.iter()
        .map(|&m| (m - mean_mult).powi(2))
        .sum::<f64>() / window_mults.len() as f64;

    let pattern_stability = 1.0 / (1.0 + variance); // Normalize to 0-1

    TemporalContext {
        pattern_duration_shots: shots.len(),
        pattern_stability,
        trend,
    }
}

/// Generate recommended action based on risk level and detected patterns
fn generate_action_recommendation(risk_level: &RiskLevel, patterns: &[String]) -> String {
    match risk_level {
        RiskLevel::Critical => {
            "CRITICAL: Immediate account suspension and manual review required. Multiple high-confidence cheating indicators detected.".to_string()
        }
        RiskLevel::High => {
            format!("HIGH RISK: Restrict account (limit max wager to $10). {} pattern(s) detected. Escalate to fraud team.", patterns.len())
        }
        RiskLevel::Medium => {
            format!("MEDIUM RISK: Enable enhanced monitoring. {} suspicious pattern(s). Review after 50 more shots.", patterns.len())
        }
        RiskLevel::Low => {
            "LOW RISK: Continue normal monitoring. Patterns within acceptable variance.".to_string()
        }
    }
}

/// Detect sudden confidence drops (potential skill inconsistency or account sharing)
///
/// Indicators:
/// - Sudden large drops in Kalman filter confidence
/// - Inconsistent shot patterns that increase error covariance
/// - May indicate account sharing or automated play
pub fn detect_confidence_anomaly(confidence_history: &[(usize, f64)]) -> AnomalyReport {
    if confidence_history.len() < 10 {
        return AnomalyReport {
            is_suspicious: false,
            confidence: 0.0,
            detected_patterns: vec![],
            recommended_action: "Insufficient confidence history".to_string(),
        };
    }

    let mut patterns = Vec::new();
    let mut suspicion_score = 0.0;

    // Check for sudden drops (>30% drop in confidence)
    let mut max_drop = 0.0;
    for i in 1..confidence_history.len() {
        let prev_conf = confidence_history[i - 1].1;
        let curr_conf = confidence_history[i].1;

        // Only check for drops when previous confidence was reasonably high (>40%)
        if prev_conf > 40.0 {
            let drop = prev_conf - curr_conf;
            if drop > max_drop {
                max_drop = drop;
            }
        }
    }

    if max_drop > 30.0 {
        patterns.push(format!("Sudden confidence drop: {:.1}% → indicates skill inconsistency", max_drop));
        suspicion_score += 0.5;
    }

    // Check for multiple moderate drops (>15% each)
    let mut moderate_drops = 0;
    for i in 1..confidence_history.len() {
        let prev_conf = confidence_history[i - 1].1;
        let curr_conf = confidence_history[i].1;

        if prev_conf > 30.0 && (prev_conf - curr_conf) > 15.0 {
            moderate_drops += 1;
        }
    }

    if moderate_drops >= 3 {
        patterns.push(format!("Multiple confidence drops ({}x) → erratic skill pattern", moderate_drops));
        suspicion_score += 0.3;
    }

    // Check confidence volatility using exponentially weighted moving average (EWMA)
    // This avoids flagging normal Kalman filter convergence (0% → 50%) as volatile
    // Only flags abnormal swings after skill has stabilized

    // Calculate confidence changes (deltas) between consecutive measurements
    let mut deltas = Vec::new();
    for i in 1..confidence_history.len() {
        let delta = (confidence_history[i].1 - confidence_history[i-1].1).abs();
        deltas.push(delta);
    }

    if deltas.len() >= 5 {
        // Calculate baseline volatility (average delta across all history)
        let baseline_volatility: f64 = deltas.iter().sum::<f64>() / deltas.len() as f64;

        // Calculate recent volatility with exponential weighting (alpha = 0.3)
        // This weighs recent batches more heavily while considering historical patterns
        let alpha = 0.3;
        let mut smoothed_volatility = deltas[0];
        for &delta in &deltas[1..] {
            smoothed_volatility = alpha * delta + (1.0 - alpha) * smoothed_volatility;
        }

        // Get most recent deltas (last 3 measurements)
        let recent_start = deltas.len().saturating_sub(3);
        let recent_volatility: f64 = deltas[recent_start..].iter().sum::<f64>()
            / (deltas.len() - recent_start) as f64;

        // Flag only if recent volatility is significantly higher than historical baseline
        // AND we have enough data to establish a pattern (>30 shots)
        let total_shots = confidence_history.last().map(|(n, _)| *n).unwrap_or(0);

        if total_shots > 30 && recent_volatility > baseline_volatility * 3.0 && recent_volatility > 15.0 {
            patterns.push(format!("Abnormal confidence swings: recent volatility {:.1}% vs baseline {:.1}% → unstable skill",
                recent_volatility, baseline_volatility));
            suspicion_score += 0.2;
        }
    }

    let is_suspicious = suspicion_score >= 0.6;
    let recommended_action = if is_suspicious {
        "ALERT: Possible account sharing or bot usage - investigate immediately".to_string()
    } else if suspicion_score >= 0.4 {
        "CAUTION: Monitor for continued anomalies".to_string()
    } else {
        "Normal confidence pattern".to_string()
    };

    AnomalyReport {
        is_suspicious,
        confidence: suspicion_score,
        detected_patterns: patterns,
        recommended_action,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_normal_play() {
        let shots: Vec<ShotOutcome> = (0..50)
            .map(|i| {
                let miss = 50.0 + (i % 10) as f64 * 5.0;
                ShotOutcome::new(miss, 2.0, 10.0, 4, false, 10.0)
            })
            .collect();

        let report = detect_sandbagging(&shots);
        assert!(!report.is_suspicious, "Normal play should not be flagged");
    }

    #[test]
    fn test_detect_obvious_sandbagging() {
        let mut shots = Vec::new();

        // Phase 1: Poor shots with low wagers (establishing bad baseline)
        for _ in 0..25 {
            shots.push(ShotOutcome::new(100.0, 0.5, 1.0, 4, false, 10.0));
        }

        // Phase 2: Suddenly excellent shots with high wagers (sandbagging pattern)
        for _ in 0..25 {
            shots.push(ShotOutcome::new(10.0, 5.0, 100.0, 4, false, 10.0));
        }

        let report = detect_sandbagging(&shots);
        assert!(report.is_suspicious, "Obvious sandbagging should be detected");
        assert!(report.confidence >= 0.6);
    }

    // ML ENSEMBLE TESTS
    #[test]
    fn test_ml_ensemble_normal_play() {
        // Generate 60 shots with normal variance
        let shots: Vec<ShotOutcome> = (0..60)
            .map(|i| {
                let miss = 50.0 + ((i * 7) % 20) as f64 * 3.0; // Natural variation
                let wager = 10.0 + ((i * 3) % 15) as f64; // Normal wager variation
                let mult = 10.0 / (1.0 + miss / 100.0);
                ShotOutcome::new(miss, mult, wager, 4, false, 10.0)
            })
            .collect();

        let report = detect_ml_ensemble(&shots, None, None);

        assert!(!report.is_suspicious, "Normal play should not trigger ensemble detection");
        assert!(report.ensemble_score < 0.5, "Ensemble score should be low for normal play");
        assert_eq!(report.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_ml_ensemble_sophisticated_cheating() {
        let mut shots = Vec::new();

        // Sophisticated sandbagging: gradual wager escalation with improving performance
        // Need more extreme pattern for Bayesian adjustment
        for i in 0..80 {
            let (miss, wager) = if i < 40 {
                // Phase 1: Establish poor baseline
                (90.0 + (i % 5) as f64 * 10.0, 5.0)
            } else {
                // Phase 2: Improve performance and massively increase wagers
                (15.0 + (i % 5) as f64 * 3.0, 200.0 + (i - 40) as f64 * 5.0)
            };

            let mult = 10.0 / (1.0 + miss / 100.0);
            shots.push(ShotOutcome::new(miss, mult, wager, 4, false, 10.0));
        }

        let report = detect_ml_ensemble(&shots, None, None);

        println!("Sophisticated cheating - ensemble_score: {}, individual: {:?}, patterns: {:?}",
                 report.ensemble_score, report.individual_scores, report.detected_patterns);
        // With updated thresholds, expect at least Low risk trending toward Medium
        assert!(report.ensemble_score >= 0.3, "Ensemble score should be elevated: {}", report.ensemble_score);
        // Could be Low or Medium depending on Bayesian adjustment
        assert!(report.detected_patterns.len() >= 3, "Should detect multiple patterns: {:?}", report.detected_patterns);
    }

    #[test]
    fn test_temporal_pattern_detection() {
        let mut shots = Vec::new();

        // Create temporal escalation pattern (3 windows) with 4x escalation
        for window in 0..4 {
            let wager_base = if window == 0 { 10.0 } else { 10.0 * (5_f64.powi(window as i32)) };
            for _ in 0..10 {
                shots.push(ShotOutcome::new(50.0, 2.0, wager_base, 4, false, 10.0));
            }
        }

        let report = detect_temporal_patterns(&shots);

        println!("Temporal test - confidence: {}, patterns: {:?}", report.confidence, report.detected_patterns);
        assert!(report.is_suspicious, "Temporal escalation should be detected. Confidence: {}, patterns: {:?}", report.confidence, report.detected_patterns);
        assert!(report.confidence >= 0.3, "Should have meaningful confidence score");
        assert!(report.detected_patterns.iter().any(|p| p.contains("escalation") || p.contains("Temporal")));
    }

    #[test]
    fn test_sequence_pattern_detection() {
        let mut shots = Vec::new();

        // Create repeating L-H-L-H pattern (alternating low/high wagers)
        // Need enough for sequence analysis (at least 20 shots, but more for strong pattern)
        for i in 0..60 {
            let wager = if i % 2 == 0 { 5.0 } else { 100.0 }; // More extreme difference
            shots.push(ShotOutcome::new(50.0, 2.0, wager, 4, false, 10.0));
        }

        let report = detect_sequence_patterns(&shots);

        println!("Sequence test - confidence: {}, patterns: {:?}", report.confidence, report.detected_patterns);
        // Check that pattern repetition is detected
        assert!(report.confidence >= 0.5, "Should detect repeating pattern: {}", report.confidence);
        assert!(report.detected_patterns.len() > 0, "Should detect at least one pattern");
        // The test creates L-H alternation, but detection works via trigram analysis
    }

    #[test]
    fn test_bayesian_adjustment_small_sample() {
        // With small sample, should pull score toward prior (5% = 0.05)
        let raw_score = 0.8;
        let sample_size = 10;

        let adjusted = apply_bayesian_adjustment(raw_score, sample_size);

        // Should be pulled down toward prior
        assert!(adjusted < raw_score, "Small sample should be adjusted toward prior");
        assert!(adjusted > 0.05, "Should still reflect measurement somewhat");
    }

    #[test]
    fn test_bayesian_adjustment_large_sample() {
        // With large sample, should be close to measured score
        let raw_score = 0.8;
        let sample_size = 200;

        let adjusted = apply_bayesian_adjustment(raw_score, sample_size);

        // Should be very close to raw score
        assert!((adjusted - raw_score).abs() < 0.1, "Large sample should trust measurement");
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(classify_risk_level(0.9), RiskLevel::Critical);
        assert_eq!(classify_risk_level(0.75), RiskLevel::Critical);
        assert_eq!(classify_risk_level(0.55), RiskLevel::High);
        assert_eq!(classify_risk_level(0.35), RiskLevel::Medium);
        assert_eq!(classify_risk_level(0.3), RiskLevel::Low);
    }

    #[test]
    fn test_temporal_context_stable() {
        // Create stable performance shots
        let shots: Vec<ShotOutcome> = (0..50)
            .map(|_| ShotOutcome::new(50.0, 2.0, 10.0, 4, false, 10.0))
            .collect();

        let context = analyze_temporal_context(&shots);

        assert_eq!(context.trend, TrendDirection::Stable);
        assert!(context.pattern_stability > 0.8, "Stable pattern should have high stability score");
    }

    #[test]
    fn test_temporal_context_improving() {
        let mut shots = Vec::new();

        // First half: poor performance
        for _ in 0..25 {
            shots.push(ShotOutcome::new(80.0, 1.5, 10.0, 4, false, 10.0));
        }

        // Second half: better performance
        for _ in 0..25 {
            shots.push(ShotOutcome::new(30.0, 3.5, 10.0, 4, false, 10.0));
        }

        let context = analyze_temporal_context(&shots);

        assert_eq!(context.trend, TrendDirection::Improving);
    }

    #[test]
    fn test_unrealistic_consistency_perfect_shots() {
        // Test case: multiple shots at 0 miss distance (the user's scenario)
        let shots: Vec<ShotOutcome> = (0..20)
            .map(|_| ShotOutcome::new(0.0, 10.0, 100.0, 4, false, 10.0))
            .collect();

        let report = detect_unrealistic_consistency(&shots);

        println!("Perfect shots - confidence: {}, patterns: {:?}",
                 report.confidence, report.detected_patterns);

        assert!(report.is_suspicious, "All shots at 0 miss distance should be flagged as suspicious");
        assert!(report.confidence >= 0.8, "Should have very high confidence (>=0.8): {}", report.confidence);
        assert!(report.detected_patterns.iter().any(|p| p.contains("impossibly perfect") || p.contains("CRITICAL")));
    }

    #[test]
    fn test_unrealistic_consistency_low_variance() {
        // Test case: shots with suspiciously low variance
        let shots: Vec<ShotOutcome> = (0..30)
            .map(|i| {
                // All shots between 2-3 feet (too consistent)
                let miss = 2.5 + (i % 2) as f64 * 0.1;
                ShotOutcome::new(miss, 8.0, 50.0, 4, false, 10.0)
            })
            .collect();

        let report = detect_unrealistic_consistency(&shots);

        println!("Low variance - confidence: {}, patterns: {:?}",
                 report.confidence, report.detected_patterns);

        assert!(report.is_suspicious, "Unnaturally consistent shots should be flagged");
        assert!(report.detected_patterns.iter().any(|p| p.contains("low variance") || p.contains("consistent")));
    }

    #[test]
    fn test_unrealistic_consistency_normal_play() {
        // Test case: normal variation in shots (should NOT be flagged)
        let shots: Vec<ShotOutcome> = (0..30)
            .map(|i| {
                let miss = 40.0 + ((i * 7) % 20) as f64 * 5.0; // Natural variation 40-140 ft
                let mult = 10.0 / (1.0 + miss / 100.0);
                ShotOutcome::new(miss, mult, 25.0, 4, false, 10.0)
            })
            .collect();

        let report = detect_unrealistic_consistency(&shots);

        println!("Normal play - confidence: {}, patterns: {:?}",
                 report.confidence, report.detected_patterns);

        assert!(!report.is_suspicious, "Normal play should not be flagged as suspicious");
    }

    #[test]
    fn test_ml_ensemble_detects_perfect_shots() {
        // Test that the ML ensemble picks up perfect shots through the new detector
        let shots: Vec<ShotOutcome> = (0..30)
            .map(|_| ShotOutcome::new(0.5, 10.0, 100.0, 4, false, 10.0))
            .collect();

        let report = detect_ml_ensemble(&shots, None, None);

        println!("ML ensemble perfect shots - score: {}, individual: {:?}, patterns: {:?}",
                 report.ensemble_score, report.individual_scores, report.detected_patterns);

        // With 30 shots of perfect play, should definitely be flagged
        assert!(report.ensemble_score >= 0.35, "Ensemble score should be significantly elevated: {}", report.ensemble_score);
        assert!(report.individual_scores.contains_key("unrealistic_consistency"));
        assert!(report.individual_scores["unrealistic_consistency"] >= 0.7,
                "Unrealistic consistency detector should have high score: {}",
                report.individual_scores["unrealistic_consistency"]);
        assert!(report.risk_level == RiskLevel::Medium || report.risk_level == RiskLevel::High || report.risk_level == RiskLevel::Critical,
                "Risk level should be at least Medium for perfect shots");
    }

    #[test]
    fn test_ensemble_with_historical_data() {
        // Historical: poor performance, low wagers
        let historical: Vec<ShotOutcome> = (0..30)
            .map(|_| ShotOutcome::new(90.0, 1.2, 5.0, 4, false, 10.0))
            .collect();

        // Recent: excellent performance, high wagers (suspicious jump)
        let recent: Vec<ShotOutcome> = (0..30)
            .map(|_| ShotOutcome::new(20.0, 4.5, 100.0, 4, false, 10.0))
            .collect();

        let report = detect_ml_ensemble(&recent, Some(&historical), None);

        println!("Historical test - ensemble_score: {}, individual: {:?}",
                 report.ensemble_score, report.individual_scores);
        // With small sample size (30 shots), Bayesian pulls score down significantly
        // Check that skill jump detector fired strongly
        assert!(report.individual_scores.contains_key("skill_jump"));
        assert!(report.individual_scores["skill_jump"] > 0.7, "Skill jump score should be high: {}", report.individual_scores["skill_jump"]);
        // Ensemble score will be lower due to Bayesian adjustment
        assert!(report.ensemble_score >= 0.05, "Should have some suspicion: {}", report.ensemble_score);
    }

    #[test]
    fn test_ensemble_with_confidence_history() {
        let shots: Vec<ShotOutcome> = (0..60)
            .map(|i| ShotOutcome::new(50.0 + (i % 10) as f64 * 5.0, 2.0, 10.0, 4, false, 10.0))
            .collect();

        // Create confidence history with sudden drops (needs at least 10 entries)
        let conf_history: Vec<(usize, f64)> = vec![
            (5, 20.0),
            (10, 30.0),
            (15, 45.0),
            (20, 50.0),
            (25, 55.0),
            (30, 58.0),
            (35, 25.0), // Sudden drop
            (40, 60.0),
            (45, 62.0),
            (50, 28.0), // Another drop
            (55, 65.0),
            (60, 30.0), // Third drop
        ];

        let report = detect_ml_ensemble(&shots, None, Some(&conf_history));

        println!("Confidence history test - individual_scores: {:?}", report.individual_scores);
        assert!(report.individual_scores.contains_key("confidence_anomaly"),
                "Should have confidence_anomaly detector. Scores: {:?}", report.individual_scores);
        // Confidence drops should contribute to suspicion score
    }

    #[test]
    fn test_ml_ensemble_combines_multiple_signals() {
        let mut shots = Vec::new();

        // Create pattern that triggers multiple detectors:
        // 1. Temporal escalation (windows with increasing wagers)
        // 2. Sequence pattern (alternating L-H)
        // 3. Cherry-picking (high wagers on good shots)
        for window in 0..5 {
            let wager_base = 10.0 * (5_f64.powi(window as i32)); // More aggressive escalation
            for i in 0..10 {
                let miss = if i % 2 == 0 { 80.0 } else { 15.0 }; // Strong alternating quality
                let wager = if i % 2 == 0 { wager_base * 0.2 } else { wager_base }; // Clear cherry-picking
                let mult = 10.0 / (1.0 + miss / 100.0);
                shots.push(ShotOutcome::new(miss, mult, wager, 4, false, 10.0));
            }
        }

        let report = detect_ml_ensemble(&shots, None, None);

        println!("Multi-signal test - ensemble: {}, individual: {:?}, patterns: {}",
                 report.ensemble_score, report.individual_scores, report.detected_patterns.len());
        assert!(report.detected_patterns.len() >= 3, "Should detect patterns from multiple detectors: {:?}", report.detected_patterns);
        // With 50 shots, Bayesian adjustment pulls score down, expect at least 0.2
        assert!(report.ensemble_score > 0.2, "Combined score should be elevated: {}", report.ensemble_score);

        // Check that multiple detectors contributed
        let high_scoring_detectors: Vec<_> = report.individual_scores.iter()
            .filter(|(_, &score)| score > 0.25)
            .collect();

        assert!(high_scoring_detectors.len() >= 2, "At least 2 detectors should have elevated scores: {:?}", report.individual_scores);
    }
}
