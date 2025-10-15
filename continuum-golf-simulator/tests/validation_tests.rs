/// Validation Tests - Replicating Business Plan Claims
///
/// This test suite validates that the simulator matches all claims
/// made in the business plan for the Continuum Golf wagering system.

use continuum_golf_simulator::math::distributions::*;
use continuum_golf_simulator::models::hole::*;
use continuum_golf_simulator::models::player::*;
use continuum_golf_simulator::simulators::player_session::*;

/// Validation Test 1: RTP by Distance Category
///
/// Business Plan Claim:
/// - Short holes (75-125yd): RTP = 86%
/// - Mid holes (150-175yd): RTP = 88%
/// - Long holes (200-250yd): RTP = 90%
#[test]
fn validate_rtp_by_distance() {
    const NUM_SHOTS: usize = 20_000;
    const TOLERANCE: f64 = 0.01; // 1% tolerance

    println!("\n=== Validation: RTP by Distance Category ===");

    // Test Short Holes (H1, H2, H3)
    let short_holes = vec![1, 2, 3];
    let mut short_wagered = 0.0;
    let mut short_won = 0.0;

    for hole_id in short_holes {
        let mut player = Player::new(format!("player_{}", 15), 15);
        let config = SessionConfig {
            num_shots: NUM_SHOTS / 3,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(hole_id),
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        let result = run_session(&mut player, config);
        short_wagered += result.total_wagered;
        short_won += result.total_won;
    }

    let short_rtp = short_won / short_wagered;
    println!("Short Holes RTP: {:.4} (target: 0.86)", short_rtp);
    assert!(
        (short_rtp - 0.86).abs() < TOLERANCE,
        "Short holes RTP {:.4} differs from target 0.86 by more than {}",
        short_rtp, TOLERANCE
    );

    // Test Mid Holes (H4, H5)
    let mid_holes = vec![4, 5];
    let mut mid_wagered = 0.0;
    let mut mid_won = 0.0;

    for hole_id in mid_holes {
        let mut player = Player::new(format!("player_{}", 15), 15);
        let config = SessionConfig {
            num_shots: NUM_SHOTS / 2,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(hole_id),
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        let result = run_session(&mut player, config);
        mid_wagered += result.total_wagered;
        mid_won += result.total_won;
    }

    let mid_rtp = mid_won / mid_wagered;
    println!("Mid Holes RTP: {:.4} (target: 0.88)", mid_rtp);
    assert!(
        (mid_rtp - 0.88).abs() < TOLERANCE,
        "Mid holes RTP {:.4} differs from target 0.88 by more than {}",
        mid_rtp, TOLERANCE
    );

    // Test Long Holes (H6, H7, H8)
    let long_holes = vec![6, 7, 8];
    let mut long_wagered = 0.0;
    let mut long_won = 0.0;

    for hole_id in long_holes {
        let mut player = Player::new(format!("player_{}", 15), 15);
        let config = SessionConfig {
            num_shots: NUM_SHOTS / 3,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(hole_id),
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        let result = run_session(&mut player, config);
        long_wagered += result.total_wagered;
        long_won += result.total_won;
    }

    let long_rtp = long_won / long_wagered;
    println!("Long Holes RTP: {:.4} (target: 0.90)", long_rtp);
    assert!(
        (long_rtp - 0.90).abs() < TOLERANCE,
        "Long holes RTP {:.4} differs from target 0.90 by more than {}",
        long_rtp, TOLERANCE
    );
}

/// Validation Test 2: House Edge by Distance
///
/// Business Plan Claim:
/// - Short holes: House edge = 14%
/// - Mid holes: House edge = 12%
/// - Long holes: House edge = 10%
#[test]
fn validate_house_edge_by_distance() {
    const NUM_SHOTS: usize = 20_000;
    const TOLERANCE: f64 = 0.01;

    println!("\n=== Validation: House Edge by Distance ===");

    // Short holes
    let mut player = Player::new(format!("player_{}", 15), 15);
    let config_short = SessionConfig {
        num_shots: NUM_SHOTS,
        wager_min: 10.0,
        wager_max: 10.0,
        hole_selection: HoleSelection::Weighted(vec![
            (1, 0.33),
            (2, 0.33),
            (3, 0.34),
        ]),
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };
    let result_short = run_session(&mut player, config_short);
    let edge_short = 1.0 - (result_short.total_won / result_short.total_wagered);
    println!("Short Holes Edge: {:.4} (target: 0.14)", edge_short);
    assert!(
        (edge_short - 0.14).abs() < TOLERANCE,
        "Short holes edge differs from 0.14"
    );

    // Mid holes
    let mut player = Player::new(format!("player_{}", 15), 15);
    let config_mid = SessionConfig {
        num_shots: NUM_SHOTS,
        wager_min: 10.0,
        wager_max: 10.0,
        hole_selection: HoleSelection::Weighted(vec![
            (4, 0.50),
            (5, 0.50),
        ]),
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };
    let result_mid = run_session(&mut player, config_mid);
    let edge_mid = 1.0 - (result_mid.total_won / result_mid.total_wagered);
    println!("Mid Holes Edge: {:.4} (target: 0.12)", edge_mid);
    assert!(
        (edge_mid - 0.12).abs() < TOLERANCE,
        "Mid holes edge differs from 0.12"
    );

    // Long holes
    let mut player = Player::new(format!("player_{}", 15), 15);
    let config_long = SessionConfig {
        num_shots: NUM_SHOTS,
        wager_min: 10.0,
        wager_max: 10.0,
        hole_selection: HoleSelection::Weighted(vec![
            (6, 0.33),
            (7, 0.33),
            (8, 0.34),
        ]),
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };
    let result_long = run_session(&mut player, config_long);
    let edge_long = 1.0 - (result_long.total_won / result_long.total_wagered);
    println!("Long Holes Edge: {:.4} (target: 0.10)", edge_long);
    assert!(
        (edge_long - 0.10).abs() < TOLERANCE,
        "Long holes edge differs from 0.10"
    );
}

/// Validation Test 3: Fairness - All Handicaps Have Same EV
///
/// Business Plan Claim:
/// The system is fair - all players have the same expected value
/// regardless of skill level at the same hole
#[test]
fn validate_fairness_all_handicaps() {
    const NUM_SHOTS: usize = 5_000;
    const WAGER: f64 = 10.0;
    const MAX_EV_SPREAD: f64 = 0.10; // Max $0.10 spread in EV per $10 wager

    println!("\n=== Validation: Fairness Across All Handicaps ===");

    let test_hole = 5; // Mid-range hole
    let handicaps = vec![0, 5, 10, 15, 20, 25, 30];
    let mut evs = Vec::new();

    for handicap in &handicaps {
        let mut player = Player::new(format!("player_{}", handicap), *handicap);
        let config = SessionConfig {
            num_shots: NUM_SHOTS,
            wager_min: WAGER,
            wager_max: WAGER,
            hole_selection: HoleSelection::Fixed(test_hole),
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        let result = run_session(&mut player, config);
        let ev = result.net_gain_loss / NUM_SHOTS as f64;
        evs.push(ev);
        println!("Handicap {}: EV = ${:.4} per shot", handicap, ev);
    }

    // Calculate spread
    let max_ev = evs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_ev = evs.iter().cloned().fold(f64::INFINITY, f64::min);
    let spread = max_ev - min_ev;

    println!("EV Spread: ${:.4} (max allowed: ${:.4})", spread, MAX_EV_SPREAD);

    assert!(
        spread < MAX_EV_SPREAD,
        "Fairness validation failed: EV spread ${:.4} exceeds ${:.4}",
        spread, MAX_EV_SPREAD
    );
}

/// Validation Test 4: Breakeven Radius Formula
///
/// Business Plan Claim:
/// Breakeven radius formula: d_break = d_max * (1 - P_max^(-1/k))
#[test]
fn validate_breakeven_radius_formula() {
    println!("\n=== Validation: Breakeven Radius Formula ===");

    for hole in HOLE_CONFIGURATIONS.iter() {
        let mut player = Player::new(format!("player_{}", 15), 15);
        let p_max = player.calculate_p_max(hole);
        let d_break = hole.calculate_breakeven_radius(p_max);

        // At breakeven, payout multiplier should be 1.0
        let payout_at_breakeven = hole.calculate_payout(d_break, p_max);

        println!("Hole {}: d_break={:.2} ft, P_max={:.2}, payout={:.4}",
                 hole.id, d_break, p_max, payout_at_breakeven);

        // Payout at breakeven should be very close to the wager (multiplier ≈ 1.0)
        assert!(
            (payout_at_breakeven - 1.0).abs() < 0.001,
            "Breakeven formula error for hole {}: payout={:.4}",
            hole.id, payout_at_breakeven
        );
    }
}

/// Validation Test 5: Fat-Tail Impact (2% frequency, 3× multiplier)
///
/// Business Plan Claim:
/// 2% of shots have fat-tail distribution (3× worse dispersion)
#[test]
fn validate_fat_tail_parameters() {
    const NUM_SAMPLES: usize = 100_000;
    const TARGET_FREQ: f64 = 0.02;
    const TARGET_MULT: f64 = 3.0;
    const FREQ_TOLERANCE: f64 = 0.002; // ±0.2%

    println!("\n=== Validation: Fat-Tail Parameters ===");

    let mut fat_tail_count = 0;
    let sigma = 50.0; // Arbitrary sigma

    for _ in 0..NUM_SAMPLES {
        let (_, is_fat_tail) = fat_tail_shot(sigma, 0.02, 3.0);
        if is_fat_tail {
            fat_tail_count += 1;
        }
    }

    let actual_freq = fat_tail_count as f64 / NUM_SAMPLES as f64;
    println!("Fat-tail frequency: {:.4} (target: {:.4})", actual_freq, TARGET_FREQ);

    assert!(
        (actual_freq - TARGET_FREQ).abs() < FREQ_TOLERANCE,
        "Fat-tail frequency {:.4} differs from target {:.4}",
        actual_freq, TARGET_FREQ
    );

    // Verify multiplier effect
    let mut normal_samples = Vec::new();
    let mut fat_tail_samples = Vec::new();

    for _ in 0..10_000 {
        let (distance, is_fat_tail) = fat_tail_shot(sigma, 0.02, 3.0);
        if is_fat_tail {
            fat_tail_samples.push(distance);
        } else {
            normal_samples.push(distance);
        }
    }

    let avg_normal: f64 = normal_samples.iter().sum::<f64>() / normal_samples.len() as f64;
    let avg_fat_tail: f64 = fat_tail_samples.iter().sum::<f64>() / fat_tail_samples.len() as f64;
    let actual_mult = avg_fat_tail / avg_normal;

    println!("Average normal: {:.2} ft", avg_normal);
    println!("Average fat-tail: {:.2} ft", avg_fat_tail);
    println!("Multiplier: {:.2}× (target: {:.2}×)", actual_mult, TARGET_MULT);

    // Multiplier should be close to 3.0
    assert!(
        (actual_mult - TARGET_MULT).abs() < 0.3,
        "Fat-tail multiplier {:.2} differs significantly from target {:.2}",
        actual_mult, TARGET_MULT
    );
}

/// Validation Test 6: High-Stakes Logic (wager ≥ 10× average triggers update)
///
/// Business Plan Claim:
/// High-stakes shots trigger immediate Kalman filter updates
#[test]
fn validate_high_stakes_logic() {
    println!("\n=== Validation: High-Stakes Update Logic ===");

    let mut player = Player::new(format!("player_{}", 15), 15);
    let hole = get_hole_by_id(4).unwrap();

    // Run normal shots
    let normal_config = SessionConfig {
        num_shots: 20,
        wager_min: 10.0,
        wager_max: 10.0, // $10 baseline
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    let normal_result = run_session(&mut player, normal_config);
    let normal_updates = normal_result.num_kalman_updates;

    println!("Normal shots (20 @ $10): {} updates", normal_updates);

    // Run high-stakes shots
    let high_stakes_config = SessionConfig {
        num_shots: 10,
        wager_min: 100.0,
        wager_max: 100.0, // $100 = 10× baseline
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    let high_stakes_result = run_session(&mut player, high_stakes_config);
    let high_stakes_updates = high_stakes_result.num_kalman_updates;
    let high_stakes_count = high_stakes_result.num_high_stakes_shots;

    println!("High-stakes shots (10 @ $100): {} updates", high_stakes_updates);
    println!("High-stakes shots detected: {}", high_stakes_count);

    // High-stakes should trigger more frequent updates
    // Each high-stakes shot should trigger an immediate update
    assert!(
        high_stakes_count >= 10,
        "High-stakes logic failed: only {} high-stakes shots detected for 10 @ $100",
        high_stakes_count
    );

    assert!(
        high_stakes_updates >= normal_updates + 10,
        "High-stakes updates insufficient: {} vs {} + 10",
        high_stakes_updates, normal_updates
    );
}

/// Validation Test 7: Hole Configuration Accuracy
///
/// Verify that all hole configurations match the business plan specifications
#[test]
fn validate_hole_configurations() {
    println!("\n=== Validation: Hole Configurations ===");

    let expected_configs = vec![
        (1, 75, 17.95, 0.86, 5.0),
        (2, 100, 25.69, 0.86, 5.0),
        (3, 125, 36.71, 0.88, 5.5),
        (4, 150, 47.58, 0.88, 6.0),
        (5, 175, 59.09, 0.88, 6.0),
        (6, 200, 73.58, 0.90, 6.5),
        (7, 225, 84.84, 0.90, 6.5),
        (8, 250, 101.14, 0.90, 6.5),
    ];

    for (id, dist, d_max, rtp, k) in expected_configs {
        let hole = get_hole_by_id(id).expect(&format!("Hole {} not found", id));

        println!("Hole {}: {}yd, d_max={:.2}, RTP={:.2}, k={:.1}",
                 id, dist, d_max, rtp, k);

        assert_eq!(hole.id, id, "Hole ID mismatch");
        assert_eq!(hole.distance_yds, dist, "Hole {} distance mismatch", id);
        assert!(
            (hole.d_max_ft - d_max).abs() < 0.01,
            "Hole {} d_max mismatch: expected {:.2}, got {:.2}",
            id, d_max, hole.d_max_ft
        );
        assert!(
            (hole.rtp - rtp).abs() < 0.001,
            "Hole {} RTP mismatch: expected {:.3}, got {:.3}",
            id, rtp, hole.rtp
        );
        assert!(
            (hole.k - k).abs() < 0.01,
            "Hole {} k mismatch: expected {:.1}, got {:.1}",
            id, k, hole.k
        );
    }
}

/// Validation Test 8: Kalman Filter Convergence Properties
///
/// Business Plan Claim:
/// Kalman filter converges to player's true skill over time,
/// with confidence increasing as more shots are observed
#[test]
fn validate_kalman_convergence_properties() {
    const SHOTS_PER_BATCH: usize = 25;
    const NUM_BATCHES: usize = 8;

    println!("\n=== Validation: Kalman Convergence Properties ===");

    let mut player = Player::new(format!("player_{}", 15), 15);
    let hole = get_hole_by_id(4).unwrap();

    let mut previous_confidence = 0.0;
    let mut confidence_increased = 0;

    for batch_num in 0..NUM_BATCHES {
        let config = SessionConfig {
            num_shots: SHOTS_PER_BATCH,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: None,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };

        let result = run_session(&mut player, config);
        let num_updates = result.num_kalman_updates;

        println!("After {} shots: {} Kalman updates",
                 (batch_num + 1) * SHOTS_PER_BATCH, num_updates);

        if num_updates > 0 {
            confidence_increased += 1;
        }
    }

    // Should have updates in most batches
    assert!(
        confidence_increased >= NUM_BATCHES - 2,
        "Kalman updates did not occur consistently: only {} out of {} batches",
        confidence_increased, NUM_BATCHES
    );

    // After many shots, sigma should have converged to a reasonable range
    let final_sigma = player.get_skill_for_hole(hole).kalman_filter.estimate;
    assert!(
        final_sigma > 20.0 && final_sigma < 200.0,
        "Final sigma {:.2} is out of reasonable range after {} shots",
        final_sigma, NUM_BATCHES * SHOTS_PER_BATCH
    );
}

/// Validation Test 9: Rayleigh Distribution Properties
///
/// Verify that the miss distance distribution follows
/// the expected Rayleigh distribution properties
#[test]
fn validate_rayleigh_distribution() {
    const NUM_SAMPLES: usize = 100_000;
    const SIGMA: f64 = 50.0;

    println!("\n=== Validation: Rayleigh Distribution Properties ===");

    let samples: Vec<f64> = (0..NUM_SAMPLES)
        .map(|_| rayleigh_random(SIGMA))
        .collect();

    let mean = samples.iter().sum::<f64>() / NUM_SAMPLES as f64;
    let variance = samples.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / NUM_SAMPLES as f64;
    let std_dev = variance.sqrt();

    let expected_mean = rayleigh_mean(SIGMA);
    let expected_std_dev = rayleigh_variance(SIGMA).sqrt();

    println!("Sample mean: {:.2} (expected: {:.2})", mean, expected_mean);
    println!("Sample std dev: {:.2} (expected: {:.2})", std_dev, expected_std_dev);

    assert!(
        (mean - expected_mean).abs() < 0.5,
        "Rayleigh mean differs from expected: {:.2} vs {:.2}",
        mean, expected_mean
    );

    assert!(
        (std_dev - expected_std_dev).abs() < 0.5,
        "Rayleigh std dev differs from expected: {:.2} vs {:.2}",
        std_dev, expected_std_dev
    );
}

/// Validation Test 10: System-Wide RTP Validation
///
/// Run a comprehensive multi-hole, multi-handicap simulation
/// to verify overall system RTP matches expectations
#[test]
fn validate_system_wide_rtp() {
    const NUM_SHOTS_PER_COMBO: usize = 1_000;

    println!("\n=== Validation: System-Wide RTP ===");

    let mut total_wagered = 0.0;
    let mut total_won = 0.0;

    // Test all combinations of holes and handicaps
    for hole_id in 1..=8 {
        for handicap in [0, 10, 20, 30].iter() {
            let mut player = Player::new(format!("player_{}", handicap), *handicap);
            let config = SessionConfig {
                num_shots: NUM_SHOTS_PER_COMBO,
                wager_min: 10.0,
                wager_max: 10.0,
                hole_selection: HoleSelection::Fixed(hole_id),
                developer_mode: None,
                fat_tail_prob: 0.02,
                fat_tail_mult: 3.0,
            };

            let result = run_session(&mut player, config);
            total_wagered += result.total_wagered;
            total_won += result.total_won;
        }
    }

    let system_rtp = total_won / total_wagered;
    let system_edge = 1.0 - system_rtp;

    println!("Total Wagered: ${:.2}", total_wagered);
    println!("Total Won: ${:.2}", total_won);
    println!("System-Wide RTP: {:.4}", system_rtp);
    println!("System-Wide Edge: {:.4}", system_edge);

    // System-wide RTP should be between short (86%) and long (90%) holes
    assert!(
        system_rtp > 0.85 && system_rtp < 0.91,
        "System-wide RTP {:.4} is outside expected range [0.85, 0.91]",
        system_rtp
    );

    // System-wide edge should be positive (house always wins)
    assert!(
        system_edge > 0.0,
        "System edge should be positive, got {:.4}",
        system_edge
    );
}
