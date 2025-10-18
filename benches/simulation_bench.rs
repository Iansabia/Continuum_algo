// Benchmark suite for Continuum Golf Simulator
//
// To run: cargo bench
// To run specific bench: cargo bench --bench simulation_bench <pattern>

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use continuum_golf_simulator::math::distributions::*;
use continuum_golf_simulator::math::integration::*;
use continuum_golf_simulator::math::kalman::*;
use continuum_golf_simulator::models::hole::*;
use continuum_golf_simulator::models::player::*;
use continuum_golf_simulator::models::shot::*;
use continuum_golf_simulator::simulators::player_session::*;
use continuum_golf_simulator::simulators::venue::*;
use continuum_golf_simulator::simulators::tournament::*;

/// Benchmark: Single shot simulation
///
/// Target: <1μs per shot
fn benchmark_single_shot(c: &mut Criterion) {
    let sigma = 50.0;

    c.bench_function("single_shot_standard", |b| {
        b.iter(|| {
            let (distance, _) = black_box(simulate_shot(sigma, 0.02, 3.0));
            distance
        });
    });

    c.bench_function("single_shot_rayleigh", |b| {
        b.iter(|| {
            black_box(rayleigh_random(sigma))
        });
    });

    c.bench_function("single_shot_fat_tail", |b| {
        b.iter(|| {
            let (distance, _) = black_box(fat_tail_shot(sigma, 0.02, 3.0));
            distance
        });
    });
}

/// Benchmark: P_max calculation (numerical integration)
///
/// Target: <100μs per calculation
fn benchmark_p_max_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("p_max_calculation");

    // Test across different holes (different distances and parameters)
    for hole_id in [1, 4, 8].iter() {
        let hole = get_hole_by_id(*hole_id).unwrap();
        let mut player = Player::new(15);

        group.bench_with_input(
            BenchmarkId::new("hole", hole_id),
            hole_id,
            |b, _| {
                b.iter(|| {
                    black_box(player.calculate_p_max(hole))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Payout calculation
///
/// Target: <100ns per payout
fn benchmark_payout_calculation(c: &mut Criterion) {
    let hole = get_hole_by_id(4).unwrap();
    let p_max = 8.5;

    c.bench_function("payout_calculation", |b| {
        b.iter(|| {
            let miss_distance = 25.0;
            black_box(hole.calculate_payout(miss_distance, p_max))
        });
    });
}

/// Benchmark: Kalman filter operations
///
/// Target: <1μs per update
fn benchmark_kalman_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("kalman_operations");

    group.bench_function("kalman_predict", |b| {
        let mut kalman = KalmanState::new(50.0, 1.0);
        b.iter(|| {
            black_box(kalman.predict())
        });
    });

    group.bench_function("kalman_update", |b| {
        let mut kalman = KalmanState::new(50.0, 1.0);
        b.iter(|| {
            kalman.update(black_box(52.0), black_box(10.0));
        });
    });

    group.bench_function("kalman_confidence", |b| {
        let kalman = KalmanState::new(50.0, 1.0);
        b.iter(|| {
            black_box(kalman.calculate_confidence())
        });
    });

    group.finish();
}

/// Benchmark: Numerical integration methods
///
/// Target: <10μs per integration
fn benchmark_integration_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("integration_methods");

    let f = |x: f64| x * x;

    group.bench_function("trapezoidal_rule", |b| {
        b.iter(|| {
            black_box(trapezoidal_rule(f, 0.0, 10.0, 100))
        });
    });

    group.bench_function("simpsons_rule", |b| {
        b.iter(|| {
            black_box(simpsons_rule(f, 0.0, 10.0, 100))
        });
    });

    group.bench_function("adaptive_integration", |b| {
        b.iter(|| {
            black_box(adaptive_integration(f, 0.0, 10.0, 1e-6, 1000))
        });
    });

    group.finish();
}

/// Benchmark: Player session simulations at various scales
///
/// Target: 10,000 shots in <1s
fn benchmark_player_sessions(c: &mut Criterion) {
    let mut group = c.benchmark_group("player_sessions");
    group.sample_size(20); // Reduce sample size for longer benchmarks

    for num_shots in [10, 100, 1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("shots", num_shots),
            num_shots,
            |b, &shots| {
                b.iter(|| {
                    let mut player = Player::new(15);
                    let config = SessionConfig {
                        num_shots: shots,
                        wager_range: (10.0, 10.0),
                        hole_selection: HoleSelection::Random,
                        developer_mode: None,
                    };
                    black_box(run_session(&mut player, config))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Venue simulations at various scales
///
/// Target: 200k-visitor venue sim in <10s
fn benchmark_venue_simulations(c: &mut Criterion) {
    let mut group = c.benchmark_group("venue_simulations");
    group.sample_size(10); // Very few samples for large benchmarks

    // Small venue (1 bay, 1 hour, 100 shots/hr = 100 shots)
    group.bench_function("small_venue", |b| {
        b.iter(|| {
            let config = VenueConfig {
                num_bays: 1,
                hours: 1.0,
                shots_per_hour: 100,
                player_archetype: PlayerArchetype::Uniform,
                wager_range: (5.0, 15.0),
            };
            black_box(run_venue_simulation(config))
        });
    });

    // Medium venue (10 bays, 4 hours, 100 shots/hr = 4,000 shots)
    group.bench_function("medium_venue", |b| {
        b.iter(|| {
            let config = VenueConfig {
                num_bays: 10,
                hours: 4.0,
                shots_per_hour: 100,
                player_archetype: PlayerArchetype::BellCurve { mean: 15, std_dev: 5.0 },
                wager_range: (5.0, 15.0),
            };
            black_box(run_venue_simulation(config))
        });
    });

    // Large venue (50 bays, 8 hours, 100 shots/hr = 40,000 shots)
    group.bench_function("large_venue", |b| {
        b.iter(|| {
            let config = VenueConfig {
                num_bays: 50,
                hours: 8.0,
                shots_per_hour: 100,
                player_archetype: PlayerArchetype::BellCurve { mean: 15, std_dev: 5.0 },
                wager_range: (5.0, 15.0),
            };
            black_box(run_venue_simulation(config))
        });
    });

    group.finish();
}

/// Benchmark: Tournament simulations
///
/// Target: <100ms for 100-player tournament
fn benchmark_tournaments(c: &mut Criterion) {
    let mut group = c.benchmark_group("tournaments");

    for num_players in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("players", num_players),
            num_players,
            |b, &players| {
                b.iter(|| {
                    let config = TournamentConfig {
                        game_mode: GameMode::ClosestToPin { hole_id: 4 },
                        num_players: players,
                        entry_fee: 20.0,
                        house_rake_percent: 10.0,
                        payout_structure: PayoutStructure::Top3 {
                            first: 0.50,
                            second: 0.30,
                            third: 0.20,
                        },
                        attempts_per_player: 3,
                    };
                    black_box(run_tournament(config))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Mathematical distributions
///
/// Target: <100ns per sample
fn benchmark_distributions(c: &mut Criterion) {
    let mut group = c.benchmark_group("distributions");

    group.bench_function("normal_random", |b| {
        b.iter(|| {
            black_box(normal_random(0.0, 1.0))
        });
    });

    group.bench_function("rayleigh_random", |b| {
        b.iter(|| {
            black_box(rayleigh_random(50.0))
        });
    });

    group.bench_function("rayleigh_pdf", |b| {
        b.iter(|| {
            black_box(rayleigh_pdf(25.0, 50.0))
        });
    });

    group.bench_function("fat_tail_shot", |b| {
        b.iter(|| {
            black_box(fat_tail_shot(50.0, 0.02, 3.0))
        });
    });

    group.finish();
}

/// Benchmark: Shot batch operations
///
/// Target: <1μs per operation
fn benchmark_shot_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("shot_batch_operations");

    group.bench_function("add_shot_to_batch", |b| {
        let mut batch = ShotBatch::new();
        b.iter(|| {
            batch.add_shot(black_box(25.0), black_box(10.0));
        });
    });

    group.bench_function("is_batch_full", |b| {
        let mut batch = ShotBatch::new();
        for _ in 0..3 {
            batch.add_shot(25.0, 10.0);
        }
        b.iter(|| {
            black_box(batch.is_full())
        });
    });

    group.bench_function("is_high_stakes", |b| {
        let mut batch = ShotBatch::new();
        for _ in 0..3 {
            batch.add_shot(25.0, 10.0);
        }
        b.iter(|| {
            black_box(batch.is_high_stakes(black_box(100.0)))
        });
    });

    group.finish();
}

/// Benchmark: Complete player workflow (shot + payout + Kalman update)
///
/// Target: <100μs per complete cycle
fn benchmark_complete_workflow(c: &mut Criterion) {
    c.bench_function("complete_shot_workflow", |b| {
        let mut player = Player::new(15);
        let hole = get_hole_by_id(4).unwrap();

        b.iter(|| {
            // 1. Calculate P_max
            let p_max = player.calculate_p_max(hole);

            // 2. Simulate shot
            let skill = player.get_skill_for_hole(hole);
            let sigma = skill.kalman_filter.estimate;
            let (miss_distance, is_fat_tail) = simulate_shot(sigma, 0.02, 3.0);

            // 3. Calculate payout
            let wager = 10.0;
            let payout = hole.calculate_payout(miss_distance, p_max) * wager;

            // 4. Create shot outcome
            let outcome = ShotOutcome {
                miss_distance_ft: miss_distance,
                multiplier: payout / wager,
                payout,
                wager,
                hole_id: hole.id,
                is_fat_tail,
            };

            black_box(outcome)
        });
    });
}

/// Benchmark: Player archetype generation
///
/// Target: <10ms for 1000 players
fn benchmark_player_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("player_generation");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("players", size),
            size,
            |b, &num| {
                b.iter(|| {
                    black_box(generate_player_pool(
                        PlayerArchetype::BellCurve { mean: 15, std_dev: 5.0 },
                        num
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Heatmap data generation
///
/// Target: <1ms for full heatmap
fn benchmark_heatmap_generation(c: &mut Criterion) {
    c.bench_function("build_heatmap_data", |b| {
        let config = VenueConfig {
            num_bays: 10,
            hours: 2.0,
            shots_per_hour: 100,
            player_archetype: PlayerArchetype::Uniform,
            wager_range: (5.0, 15.0),
        };

        b.iter(|| {
            let result = run_venue_simulation(config.clone());
            black_box(result.heatmap_data)
        });
    });
}

criterion_group!(
    benches,
    benchmark_single_shot,
    benchmark_p_max_calculation,
    benchmark_payout_calculation,
    benchmark_kalman_operations,
    benchmark_integration_methods,
    benchmark_player_sessions,
    benchmark_venue_simulations,
    benchmark_tournaments,
    benchmark_distributions,
    benchmark_shot_batch_operations,
    benchmark_complete_workflow,
    benchmark_player_generation,
    benchmark_heatmap_generation,
);

criterion_main!(benches);
