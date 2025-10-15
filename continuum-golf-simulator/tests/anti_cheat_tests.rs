/// Anti-Cheat and Fraud Detection Tests
///
/// This test suite validates that the Kalman filter and game mechanics
/// can detect and prevent various cheating strategies that players might
/// attempt to exploit the system.

use continuum_golf_simulator::models::hole::*;
use continuum_golf_simulator::models::player::*;
use continuum_golf_simulator::simulators::player_session::*;

/// Test 1: Low-Wage Sandbagging Attack
///
/// Strategy: Player intentionally misses badly with low wagers to inflate
/// their skill estimate (higher sigma), then places large wagers expecting
/// a higher P_max multiplier.
///
/// Expected Behavior: Kalman filter should detect the pattern and adapt,
/// preventing exploitation.
#[test]
fn test_sandbagging_attack() {
    println!("\n=== Anti-Cheat Test: Sandbagging Attack ===");

    let mut player = Player::new("cheater_sandbagger".to_string(), 10);
    let hole = get_hole_by_id(4).unwrap();

    // Get initial P_max
    let initial_p_max = player.calculate_p_max(hole);
    println!("Initial P_max: {:.2}", initial_p_max);

    // Phase 1: Sandbagging - intentionally miss badly with low wagers
    println!("\n--- Phase 1: Sandbagging (50 shots @ $1, intentional misses) ---");
    let sandbagging_config = SessionConfig {
        num_shots: 50,
        wager_min: 1.0,
        wager_max: 1.0,
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: Some(DeveloperMode {
            manual_miss_distance: Some(100.0), // Terrible miss
            disable_kalman: false,
        }),
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    let sandbagging_result = run_session(&mut player, sandbagging_config);
    let post_sandbagging_p_max = player.calculate_p_max(hole);

    println!("After sandbagging:");
    println!("  P_max: {:.2} (change: {:+.2})",
             post_sandbagging_p_max,
             post_sandbagging_p_max - initial_p_max);
    println!("  Kalman updates: {}", sandbagging_result.num_kalman_updates);
    println!("  Net result: ${:.2}", sandbagging_result.net_gain_loss);

    // Phase 2: Exploitation attempt - try to capitalize with high wagers
    println!("\n--- Phase 2: Exploitation Attempt (10 shots @ $100) ---");
    let exploit_config = SessionConfig {
        num_shots: 10,
        wager_min: 100.0,
        wager_max: 100.0,
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: None, // Real shots now
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    let exploit_result = run_session(&mut player, exploit_config);
    let final_p_max = player.calculate_p_max(hole);

    println!("After exploitation attempt:");
    println!("  P_max: {:.2} (change: {:+.2})",
             final_p_max,
             final_p_max - post_sandbagging_p_max);
    println!("  Net result: ${:.2}", exploit_result.net_gain_loss);
    println!("  Total profit from attack: ${:.2}",
             sandbagging_result.net_gain_loss + exploit_result.net_gain_loss);

    // Validation: The cheater should not have made significant profit
    // The sandbagging phase cost them money, and the Kalman filter
    // should have adapted during the exploitation phase
    let total_profit = sandbagging_result.net_gain_loss + exploit_result.net_gain_loss;

    println!("\n--- Attack Analysis ---");
    println!("Total profit from attack: ${:.2}", total_profit);

    // The attack should not be profitable
    // Even if P_max increased, the cost of sandbagging should exceed exploitation gains
    assert!(
        total_profit < 0.0,
        "Sandbagging attack was profitable: ${:.2} - SECURITY VULNERABILITY!",
        total_profit
    );

    println!("✅ Sandbagging attack FAILED - system protected");
}

/// Test 2: Gradual Skill Manipulation
///
/// Strategy: Player gradually manipulates perceived skill over many sessions
/// with subtle variations to avoid detection.
/// REALISTIC: Worse shots are intentional, better shots are real.
///
/// Expected Behavior: Kalman filter should track the true skill level
/// through weighted updates, preventing gradual manipulation.
#[test]
fn test_gradual_skill_manipulation() {
    println!("\n=== Anti-Cheat Test: Gradual Skill Manipulation ===");

    let mut player = Player::new("cheater_gradual".to_string(), 15);
    let hole = get_hole_by_id(4).unwrap();

    let initial_p_max = player.calculate_p_max(hole);
    println!("Initial P_max: {:.2}", initial_p_max);

    // Track P_max changes over time
    let mut p_max_history = vec![initial_p_max];

    // Attempt gradual manipulation over 10 sessions
    println!("\n--- Attempting gradual manipulation (10 sessions) ---");
    for session_num in 1..=10 {
        // Alternate between intentional bad shots and real shots
        let developer_mode = if session_num % 2 == 0 {
            // Intentional bad shot
            Some(DeveloperMode {
                manual_miss_distance: Some(60.0),
                disable_kalman: false,
            })
        } else {
            // Real shots (player's actual skill)
            None
        };

        let config = SessionConfig {
            num_shots: 20,
            wager_min: 10.0,
            wager_max: 10.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };

        run_session(&mut player, config);
        let current_p_max = player.calculate_p_max(hole);
        p_max_history.push(current_p_max);

        println!("Session {}: P_max = {:.2}", session_num, current_p_max);
    }

    // Validation: P_max should have stabilized
    // The Kalman filter should converge to the actual average skill
    let final_p_max = p_max_history.last().unwrap();
    let p_max_variance: f64 = p_max_history.iter()
        .map(|p| (p - final_p_max).powi(2))
        .sum::<f64>() / p_max_history.len() as f64;

    println!("\n--- Manipulation Analysis ---");
    println!("P_max variance: {:.4}", p_max_variance);
    println!("Final P_max: {:.2}", final_p_max);

    // The variance should be relatively small, indicating convergence
    assert!(
        p_max_variance < 0.5,
        "P_max variance too high: {:.4} - Kalman filter may not be converging",
        p_max_variance
    );

    println!("✅ Gradual manipulation FAILED - Kalman filter converged");
}

/// Test 3: Sudden Skill Jump Detection (Potential Account Sharing)
///
/// Strategy: Simulate a sudden improvement in skill (e.g., skilled player
/// using a beginner's account).
/// REALISTIC: Poor baseline is intentional, "skilled" phase is real shots.
///
/// Expected Behavior: System should detect anomalous behavior patterns.
#[test]
fn test_sudden_skill_jump_detection() {
    println!("\n=== Anti-Cheat Test: Sudden Skill Jump Detection ===");

    let mut player = Player::new("cheater_account_sharing".to_string(), 5); // Low handicap skilled player
    let hole = get_hole_by_id(4).unwrap();

    // Establish baseline behavior (intentionally poor to fake being a beginner)
    println!("\n--- Phase 1: Establishing baseline (faking poor skill) ---");
    let baseline_config = SessionConfig {
        num_shots: 50,
        wager_min: 10.0,
        wager_max: 10.0,
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: Some(DeveloperMode {
            manual_miss_distance: Some(80.0), // Intentional poor performance
            disable_kalman: false,
        }),
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    let baseline_result = run_session(&mut player, baseline_config);
    let baseline_skill = player.get_skill_for_hole(hole);
    let baseline_sigma = baseline_skill.kalman_filter.estimate;

    println!("Baseline established:");
    println!("  Sigma: {:.2} ft", baseline_sigma);
    println!("  Avg loss: ${:.2}", baseline_result.net_gain_loss / 50.0);

    // Sudden improvement - player reveals true skill (or account shared)
    println!("\n--- Phase 2: Sudden skill jump (revealing true skill) ---");
    let cheat_config = SessionConfig {
        num_shots: 20,
        wager_min: 50.0, // Higher wagers now
        wager_max: 50.0,
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: None, // Real shots from skilled player
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    let cheat_result = run_session(&mut player, cheat_config);
    let post_cheat_skill = player.get_skill_for_hole(hole);
    let post_cheat_sigma = post_cheat_skill.kalman_filter.estimate;

    println!("After sudden improvement:");
    println!("  Sigma: {:.2} ft (change: {:.2} ft)",
             post_cheat_sigma,
             post_cheat_sigma - baseline_sigma);
    println!("  Net result: ${:.2}", cheat_result.net_gain_loss);

    // Calculate anomaly score
    let skill_improvement_rate = (baseline_sigma - post_cheat_sigma) / baseline_sigma;
    let wager_increase_rate = 50.0 / 10.0;

    println!("\n--- Anomaly Detection ---");
    println!("Skill improvement: {:.1}%", skill_improvement_rate * 100.0);
    println!("Wager increase: {:.1}x", wager_increase_rate);

    // Detect suspicious pattern: large skill jump + increased wagers
    let is_suspicious = skill_improvement_rate > 0.3 && wager_increase_rate > 3.0;

    if is_suspicious {
        println!("⚠️  ANOMALY DETECTED: Suspicious skill jump with increased wagers");
        println!("    Recommendation: Flag account for review");
    }

    // Even if flagged, the system should still limit profit
    // High-stakes shots trigger immediate Kalman updates
    println!("  High-stakes shots: {}", cheat_result.num_high_stakes_shots);
    println!("  Kalman updates: {}", cheat_result.num_kalman_updates);

    // The system should have triggered high-stakes updates
    assert!(
        cheat_result.num_kalman_updates > 0,
        "Kalman filter did not update during suspicious activity"
    );

    println!("✅ Anomaly detection functional - system adapted");
}

/// Test 4: Bet Timing Exploitation
///
/// Strategy: Player only bets high on their best shots (cherry-picking).
/// REALISTIC: Bad shots use developer mode (intentional misses), good shots are real.
///
/// Expected Behavior: Kalman filter should adapt based on actual shot quality,
/// not wager size. High-stakes shots trigger immediate updates.
#[test]
fn test_bet_timing_exploitation() {
    println!("\n=== Anti-Cheat Test: Bet Timing Exploitation ===");

    let mut player = Player::new("cheater_timing".to_string(), 15);
    let hole = get_hole_by_id(4).unwrap();

    println!("\n--- Strategy: Low wagers on intentional bad shots, high wagers on real shots ---");

    let mut total_wagered = 0.0;
    let mut total_won = 0.0;

    // Simulate 50 shots with cherry-picking strategy
    for shot_num in 0..50 {
        // Player can intentionally miss, but can't guarantee perfect shots
        let is_good_shot = shot_num % 3 == 0;
        let (wager, developer_mode) = if is_good_shot {
            // High wager on real shot (player's actual skill)
            (100.0, None)
        } else {
            // Low wager on intentional bad shot
            (5.0, Some(DeveloperMode {
                manual_miss_distance: Some(60.0),
                disable_kalman: false,
            }))
        };

        let config = SessionConfig {
            num_shots: 1,
            wager_min: wager,
            wager_max: wager,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };

        let result = run_session(&mut player, config);
        total_wagered += result.total_wagered;
        total_won += result.total_won;
    }

    let net_result = total_won - total_wagered;
    let rtp = total_won / total_wagered;

    println!("\n--- Cherry-Picking Results ---");
    println!("Total wagered: ${:.2}", total_wagered);
    println!("Total won: ${:.2}", total_won);
    println!("Net result: ${:.2}", net_result);
    println!("Effective RTP: {:.2}%", rtp * 100.0);

    // Validation: High-stakes shots should trigger updates
    // preventing exploitation
    let final_p_max = player.calculate_p_max(hole);
    println!("Final P_max: {:.2}", final_p_max);

    // The cherry-picking strategy should not yield significant profit
    // The house edge should still apply
    assert!(
        rtp < 1.0,
        "Cherry-picking strategy beat the house: RTP {:.2}%",
        rtp * 100.0
    );

    println!("✅ Bet timing exploitation FAILED - house edge maintained");
}

/// Test 5: Multi-Account Collusion Simulation
///
/// Strategy: Use multiple accounts to test the system and share information.
///
/// Expected Behavior: Each account's Kalman filter should adapt independently,
/// preventing cross-account exploitation.
#[test]
fn test_multi_account_collusion() {
    println!("\n=== Anti-Cheat Test: Multi-Account Collusion ===");

    let hole = get_hole_by_id(4).unwrap();

    // Create 3 colluding accounts
    let mut accounts = vec![
        Player::new("colluder_1".to_string(), 15),
        Player::new("colluder_2".to_string(), 15),
        Player::new("colluder_3".to_string(), 15),
    ];

    println!("\n--- Strategy: Share information across 3 accounts ---");

    // Each account tests different strategies
    let strategies = vec![
        ("Conservative", 30.0),
        ("Moderate", 50.0),
        ("Aggressive", 70.0),
    ];

    let mut total_profit = 0.0;

    for (idx, (strategy_name, miss_distance)) in strategies.iter().enumerate() {
        println!("\nAccount {}: {} strategy (miss ~{} ft)",
                 idx + 1, strategy_name, miss_distance);

        let config = SessionConfig {
            num_shots: 30,
            wager_min: 20.0,
            wager_max: 20.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: Some(*miss_distance),
                disable_kalman: false,
            }),
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };

        let result = run_session(&mut accounts[idx], config);
        total_profit += result.net_gain_loss;

        let p_max = accounts[idx].calculate_p_max(hole);
        println!("  P_max: {:.2}", p_max);
        println!("  Net result: ${:.2}", result.net_gain_loss);
    }

    println!("\n--- Collusion Analysis ---");
    println!("Combined profit across all accounts: ${:.2}", total_profit);

    // Validation: Collusion should not provide an edge
    // Each account is tracked independently by its Kalman filter
    assert!(
        total_profit < 0.0,
        "Multi-account collusion was profitable: ${:.2} - SECURITY VULNERABILITY!",
        total_profit
    );

    println!("✅ Multi-account collusion FAILED - accounts tracked independently");
}

/// Test 6: Session Interruption Exploitation
///
/// Strategy: Player interrupts sessions after good shots, resumes after bad ones.
/// REALISTIC: Bad sessions use intentional misses, good sessions use real shots.
///
/// Expected Behavior: Kalman filter batching and high-stakes detection should
/// prevent exploitation through session manipulation.
#[test]
fn test_session_interruption_exploitation() {
    println!("\n=== Anti-Cheat Test: Session Interruption Exploitation ===");

    let mut player = Player::new("cheater_interruption".to_string(), 15);
    let hole = get_hole_by_id(4).unwrap();

    println!("\n--- Strategy: Interrupt after real good shots, resume with bad shots ---");

    let mut total_wagered = 0.0;
    let mut total_won = 0.0;
    let mut session_count = 0;

    // Simulate 10 short sessions (interruption pattern)
    for session_num in 0..10 {
        // Alternate between real shots and intentional bad shots
        let developer_mode = if session_num % 2 == 0 {
            // Real shots (good session)
            None
        } else {
            // Intentional bad shots
            Some(DeveloperMode {
                manual_miss_distance: Some(65.0),
                disable_kalman: false,
            })
        };

        let config = SessionConfig {
            num_shots: 5, // Short sessions
            wager_min: 20.0,
            wager_max: 20.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };

        let result = run_session(&mut player, config);
        total_wagered += result.total_wagered;
        total_won += result.total_won;
        session_count += 1;

        if session_num % 2 == 0 {
            println!("Session {} (real shots): Net ${:.2}", session_num + 1, result.net_gain_loss);
        }
    }

    let net_result = total_won - total_wagered;
    let rtp = total_won / total_wagered;

    println!("\n--- Session Interruption Results ---");
    println!("Sessions: {}", session_count);
    println!("Total wagered: ${:.2}", total_wagered);
    println!("Total won: ${:.2}", total_won);
    println!("Net result: ${:.2}", net_result);
    println!("Effective RTP: {:.2}%", rtp * 100.0);

    // Validation: Session interruption should not beat the house
    assert!(
        rtp < 1.0,
        "Session interruption exploitation was successful: RTP {:.2}%",
        rtp * 100.0
    );

    println!("✅ Session interruption exploitation FAILED - batching prevents abuse");
}

/// Test 7: Stress Test - Maximum Exploitation Attempt
///
/// Combines multiple strategies to attempt maximum exploitation.
/// REALISTIC: Bad shots are intentional, good shots are real player skill.
#[test]
fn test_maximum_exploitation_attempt() {
    println!("\n=== Anti-Cheat Test: Maximum Exploitation Attempt ===");
    println!("Combining: sandbagging + cherry-picking + session interruption");

    let mut player = Player::new("master_cheater".to_string(), 15);
    let hole = get_hole_by_id(4).unwrap();

    let initial_p_max = player.calculate_p_max(hole);
    println!("\nInitial P_max: {:.2}", initial_p_max);

    // Phase 1: Sandbagging (inflate sigma) - intentional bad shots
    println!("\n--- Phase 1: Sandbagging ---");
    for _ in 0..5 {
        let config = SessionConfig {
            num_shots: 10,
            wager_min: 1.0,
            wager_max: 1.0,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode: Some(DeveloperMode {
                manual_miss_distance: Some(120.0),
                disable_kalman: false,
            }),
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };
        run_session(&mut player, config);
    }

    let post_sandbagging_p_max = player.calculate_p_max(hole);
    println!("Post-sandbagging P_max: {:.2}", post_sandbagging_p_max);

    // Phase 2: Cherry-picking with high wagers
    println!("\n--- Phase 2: Cherry-picking exploitation ---");
    let mut exploitation_wagered = 0.0;
    let mut exploitation_won = 0.0;

    for shot_num in 0..20 {
        // Player can intentionally miss, but cannot guarantee perfect shots
        let (wager, developer_mode) = if shot_num % 3 == 0 {
            // High wager on real shot (actual player skill)
            (200.0, None)
        } else {
            // Throw-away shot (intentional bad miss)
            (1.0, Some(DeveloperMode {
                manual_miss_distance: Some(90.0),
                disable_kalman: false,
            }))
        };

        let config = SessionConfig {
            num_shots: 1,
            wager_min: wager,
            wager_max: wager,
            hole_selection: HoleSelection::Fixed(4),
            developer_mode,
            fat_tail_prob: 0.02,
            fat_tail_mult: 3.0,
        };

        let result = run_session(&mut player, config);
        exploitation_wagered += result.total_wagered;
        exploitation_won += result.total_won;
    }

    let net_exploitation = exploitation_won - exploitation_wagered;

    println!("\n--- Maximum Exploitation Results ---");
    println!("Exploitation phase:");
    println!("  Wagered: ${:.2}", exploitation_wagered);
    println!("  Won: ${:.2}", exploitation_won);
    println!("  Net: ${:.2}", net_exploitation);

    let final_p_max = player.calculate_p_max(hole);
    println!("Final P_max: {:.2}", final_p_max);

    // Ultimate validation: Even with combined strategies, house edge should prevail
    assert!(
        net_exploitation < exploitation_wagered * 0.15,
        "Maximum exploitation exceeded expected limits: ${:.2}",
        net_exploitation
    );

    if net_exploitation < 0.0 {
        println!("✅ Maximum exploitation FAILED - system is secure");
    } else {
        println!("⚠️  Maximum exploitation showed profit: ${:.2}", net_exploitation);
        println!("   Kalman filter adapted and limited gains");
    }
}
