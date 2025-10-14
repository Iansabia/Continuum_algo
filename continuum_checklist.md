# Continuum Golf Simulator - Rust Rewrite Plan

## Project Overview
Rebuild the Continuum Golf wagering simulator in Rust for superior performance, type safety, and modularity. The simulator models the proprietary odds engine, player skill adaptation (Kalman filter), and venue economics.

---

## 🚀 Infrastructure Setup (Completed Before Phase 1)

- [x] **Git Repository Initialized** - Created local git repository
- [x] **Initial Commit** - Added continuum_checklist.md to version control
- [x] **MCP Servers Configured** - Set up 5 MCP servers for enhanced workflow:
  - [x] GitHub MCP - Issue tracking, PR management
  - [x] Filesystem MCP - Enhanced file operations
  - [x] Memory MCP - Persistent context storage
  - [x] Sequential-Thinking MCP - Complex mathematical reasoning
  - [x] Playwright MCP - Browser automation for testing
- [x] **SQLite Database File Created** - continuum_sim.db initialized
- [x] **Documentation Created** - MCP_SETUP.md added with full MCP usage guide

---

## Phase 1: Project Setup & Core Math ✅

### 1.1 Initialize Rust Project ✅
- [x] Create new Rust project with `cargo new continuum-golf-simulator --lib`
- [x] Set up project structure (see directory tree below)
- [x] Configure `Cargo.toml` with dependencies:
  - [x] `rand = "0.8"` - Random number generation
  - [x] `rand_distr = "0.4"` - Statistical distributions
  - [x] `serde = { version = "1.0", features = ["derive"] }` - Serialization
  - [x] `serde_json = "1.0"` - JSON export
  - [x] `csv = "1.3"` - CSV export
  - [x] `clap = { version = "4.5", features = ["derive"] }` - CLI interface
  - [x] `statrs = "0.17"` - Statistical functions
  - [x] `nalgebra = "0.33"` - Linear algebra (for Kalman)
  - [x] `rayon = "1.10"` - Parallel processing
  - [x] `criterion = "0.5"` (dev-dep) - Benchmarking
  - [x] `approx = "0.5"` (dev-dep) - Float comparisons

### 1.2 Core Mathematical Functions (`src/math/`) ✅

#### `distributions.rs` ✅
- [x] Implement `normal_random(mean: f64, std_dev: f64) -> f64`
  - Box-Muller transform for normal distribution
- [x] Implement `rayleigh_random(sigma: f64) -> f64`
  - Miss distance distribution: `d = σ * sqrt(-2 * ln(U))`
- [x] Implement `fat_tail_shot(sigma: f64, probability: f64, multiplier: f64) -> (f64, bool)`
  - 2% chance of 3× worse dispersion (configurable)
- [x] Add helper functions: `rayleigh_pdf`, `rayleigh_mean`, `rayleigh_variance`
- [x] Add unit tests for distribution properties (mean, variance)
  - **5 tests passing**: mean, variance, fat-tail frequency, PDF properties

#### `integration.rs` ✅
- [x] Implement `trapezoidal_rule(f: impl Fn(f64) -> f64, a: f64, b: f64, n: usize) -> f64`
  - Numerical integration for P_max calculation
- [x] Implement `adaptive_integration` for better accuracy
- [x] Implement `simpsons_rule` for higher-order accuracy
- [x] Implement `integrate_payout_function` for P_max calculation
- [x] Add unit tests and benchmarks
  - **6 tests passing**: trapezoidal, Simpson's, adaptive, payout integration

#### `kalman.rs` ✅
- [x] Define `KalmanState` struct:
  ```rust
  pub struct KalmanState {
      pub estimate: f64,           // Current skill estimate (σ)
      pub error_covariance: f64,   // Uncertainty P_k
      pub process_noise: f64,      // Q (skill drift)
      pub initial_estimate: f64,   // σ_0 for reset
  }
  ```
- [x] Implement `KalmanState::new(initial_sigma: f64, process_noise: f64) -> Self`
- [x] Implement `predict(&mut self) -> (f64, f64)`
  - Returns (predicted_estimate, predicted_covariance)
- [x] Implement `update(&mut self, measurement: f64, measurement_noise: f64)`
  - Kalman gain: `K = P_k / (P_k + R)`
  - Update estimate: `σ_k = σ_k-1 + K(z - σ_k-1)`
  - Update covariance: `P_k = (1 - K) * P_k-1`
- [x] Implement `calculate_confidence(&self) -> f64`
  - Maps error_covariance (50-1000) to confidence (100%-0%)
  - Formula from JS: `100 * (1 - ln(P/50) / ln(1000/50))`
- [x] Implement helper functions: `debias_rayleigh_measurement`, `weighted_average_measurement`, `measurement_variance`
- [x] Add tests validating convergence over multiple updates
  - **7 tests passing**: initialization, convergence, confidence, debiasing, weighted average, variance, reset

**Phase 1 Summary:**
- ✅ **18 unit tests passing** (5 distributions + 6 integration + 7 Kalman)
- ✅ **8 doc tests passing** (all example code verified)
- ✅ **Build successful** with all dependencies
- ✅ **CLI skeleton** created with clap (4 subcommands: player, venue, tournament, validate)

---

## Phase 2: Core Data Models (`src/models/`)

### 2.1 Hole Configuration (`hole.rs`)
- [ ] Define `ClubCategory` enum:
  ```rust
  pub enum ClubCategory {
      Wedge,      // 75-125 yds
      MidIron,    // 150-175 yds
      LongIron,   // 200-250 yds
  }
  ```
- [ ] Define `Hole` struct:
  ```rust
  pub struct Hole {
      pub id: u8,
      pub distance_yds: u16,
      pub d_max_ft: f64,        // Scoring radius
      pub rtp: f64,             // Return to player (0.86-0.90)
      pub k: f64,               // Steepness (5.0-6.5)
      pub category: ClubCategory,
  }
  ```
- [ ] Implement `Hole::calculate_payout(miss_distance: f64, p_max: f64) -> f64`
  - Formula: `P(d) = P_max * (1 - d/d_max)^k` if d ≤ d_max, else 0
- [ ] Implement `Hole::calculate_breakeven_radius(p_max: f64) -> f64`
  - Solve: `d_break = d_max * (1 - P_max^(-1/k))`
- [ ] Create `HOLE_CONFIGURATIONS: [Hole; 8]` constant with data from business plan:
  ```
  H1: 75yd,  d_max=17.95, RTP=0.86, k=5.0
  H2: 100yd, d_max=25.69, RTP=0.86, k=5.0
  H3: 125yd, d_max=36.71, RTP=0.88, k=5.5
  H4: 150yd, d_max=47.58, RTP=0.88, k=6.0
  H5: 175yd, d_max=59.09, RTP=0.88, k=6.0
  H6: 200yd, d_max=73.58, RTP=0.90, k=6.5
  H7: 225yd, d_max=84.84, RTP=0.90, k=6.5
  H8: 250yd, d_max=101.14, RTP=0.90, k=6.5
  ```

### 2.2 Player Model (`player.rs`)
- [ ] Define `Player` struct:
  ```rust
  pub struct Player {
      pub id: String,
      pub handicap: u8,           // 0-30
      pub skill_profiles: HashMap<ClubCategory, SkillProfile>,
  }
  
  pub struct SkillProfile {
      pub kalman_filter: KalmanState,
      pub p_max_history: Vec<f64>,
      pub shot_batch: Vec<ShotRecord>,
  }
  
  pub struct ShotRecord {
      pub miss_distance: f64,
      pub wager: f64,
  }
  ```
- [ ] Implement `Player::new(handicap: u8) -> Self`
  - Initialize 3 skill profiles (one per club category)
  - Calculate initial σ for each: `σ_0 = distance * 3 * (0.05 + (dist-75)/(250-75)*0.01) * (0.5 + handicap/30)`
  - Start with `error_covariance = 1000` (low confidence)
- [ ] Implement `calculate_initial_dispersion(handicap: u8, distance_yds: u16) -> f64`
  - Matches JS formula exactly
- [ ] Implement `get_skill_for_hole(&self, hole: &Hole) -> &SkillProfile`
- [ ] Implement `calculate_p_max(&self, hole: &Hole) -> f64`
  - Numerical integration: `∫[0, d_max] (1 - d/d_max)^k * PDF(d | σ) dd`
  - PDF is Rayleigh: `f(d) = (d/σ²) * exp(-d²/2σ²)`
  - Solve: `P_max = RTP / integral`
- [ ] Implement `update_skill(&mut self, hole: &Hole, batch: Vec<ShotRecord>, p_max: f64)`
  - Calculate wager-weighted average miss: `z = Σ(miss_i * wager_i) / Σ(wager_i)`
  - Unbias for Rayleigh: `z_unbiased = z / sqrt(π/2)`
  - Calculate batch variance for dynamic measurement noise
  - Update Kalman filter
  - Clear shot batch, append p_max to history

### 2.3 Shot Outcome (`shot.rs`)
- [ ] Define `ShotOutcome` struct:
  ```rust
  pub struct ShotOutcome {
      pub miss_distance_ft: f64,
      pub multiplier: f64,
      pub payout: f64,
      pub wager: f64,
      pub hole_id: u8,
      pub is_fat_tail: bool,      // Flagged extreme mishit
  }
  ```
- [ ] Implement `simulate_shot(sigma: f64, fat_tail_prob: f64, fat_tail_mult: f64) -> (f64, bool)`
  - 2% chance: sample from σ * 3.0
  - 98% chance: sample from σ
  - Return (miss_distance, is_fat_tail)

---

## Phase 3: Simulation Engines (`src/simulators/`)

### 3.1 Player Session Simulator (`player_session.rs`)
- [ ] Define `SessionConfig` struct:
  ```rust
  pub struct SessionConfig {
      pub num_shots: usize,
      pub wager_range: (f64, f64),     // Min/max per shot
      pub hole_selection: HoleSelection,
      pub developer_mode: Option<DeveloperMode>,
  }
  
  pub enum HoleSelection {
      Random,
      Weighted(Vec<(u8, f64)>),        // (hole_id, probability)
      Fixed(u8),
  }
  
  pub struct DeveloperMode {
      pub manual_miss_distance: Option<f64>,
      pub disable_kalman: bool,
  }
  ```
- [ ] Define `SessionResult` struct:
  ```rust
  pub struct SessionResult {
      pub total_wagered: f64,
      pub total_won: f64,
      pub net_gain_loss: f64,
      pub shots: Vec<ShotOutcome>,
      pub final_skill_profiles: HashMap<ClubCategory, SkillProfile>,
      pub session_house_edge: f64,
  }
  ```
- [ ] Implement `run_session(player: &mut Player, config: SessionConfig) -> SessionResult`
  - Loop for num_shots:
    1. Select hole (random or weighted)
    2. Get player's skill profile for hole category
    3. Calculate P_max
    4. Simulate shot
    5. Calculate payout
    6. Add to shot batch
    7. Check if batch is full (5 shots default) OR high-stakes shot (10× avg wager)
    8. If batch complete, update Kalman filter
  - Return aggregated metrics

### 3.2 Venue Economics Simulator (`venue.rs`)
- [ ] Define `VenueConfig` struct:
  ```rust
  pub struct VenueConfig {
      pub num_bays: usize,
      pub hours: f64,
      pub shots_per_hour: usize,
      pub player_archetype: PlayerArchetype,
  }
  
  pub enum PlayerArchetype {
      Uniform,              // Random 0-30 handicap
      BellCurve { mean: u8, std_dev: f64 },
      SkewedHigh,          // Mostly beginners
      SkewedLow,           // Mostly experts
  }
  ```
- [ ] Define `VenueResult` struct:
  ```rust
  pub struct VenueResult {
      pub total_wagered: f64,
      pub total_payouts: f64,
      pub net_profit: f64,
      pub hold_percentage: f64,
      pub profit_over_time: Vec<(f64, f64)>,  // (hour, cumulative_profit)
      pub heatmap_data: HeatmapData,
      pub payout_distribution: [usize; 11],   // Bins: 0x, 1x, ..., 10x+
  }
  
  pub struct HeatmapData {
      pub handicap_bins: Vec<String>,         // "0-4", "5-9", etc.
      pub distance_bins: Vec<u16>,            // Hole distances
      pub hold_percentages: Vec<Vec<f64>>,    // [handicap][distance] -> hold%
  }
  ```
- [ ] Implement `generate_player_pool(archetype: PlayerArchetype, size: usize) -> Vec<Player>`
  - Sample handicaps based on archetype distribution
- [ ] Implement `run_venue_simulation(config: VenueConfig) -> VenueResult`
  - Create virtual player pool (one per bay)
  - Calculate total_shots = bays × hours × shots_per_hour
  - Track profit at intervals for time series
  - Aggregate heatmap data (7 handicap bins × 8 holes)
  - Use parallel processing with `rayon` for speed
- [ ] Add progress callback for long simulations

### 3.3 Tournament Simulator (`tournament.rs`)
- [ ] Define `TournamentConfig` struct:
  ```rust
  pub struct TournamentConfig {
      pub game_mode: GameMode,
      pub num_players: usize,
      pub entry_fee: f64,
      pub house_rake_percent: f64,
      pub payout_structure: PayoutStructure,
  }
  
  pub enum GameMode {
      LongestDrive,
      ClosestToPin { hole_id: u8 },
  }
  
  pub enum PayoutStructure {
      WinnerTakesAll,
      Top3 { first: f64, second: f64, third: f64 },
      Top2 { first: f64, second: f64 },
  }
  ```
- [ ] Define `TournamentResult` struct:
  ```rust
  pub struct TournamentResult {
      pub leaderboard: Vec<(String, f64)>,    // (player_id, score)
      pub total_pool: f64,
      pub house_rake: f64,
      pub prize_pool: f64,
      pub payouts: Vec<(String, f64)>,
  }
  ```
- [ ] Implement `run_tournament(config: TournamentConfig) -> TournamentResult`
  - Generate players, simulate attempts (5 shots each)
  - Track best score per player (max for longest, min for CTP)
  - Sort leaderboard and distribute prizes

---

## Phase 4: Analytics & Validation (`src/analytics/`)

### 4.1 Metrics (`metrics.rs`)
- [ ] Implement `calculate_expected_value(player: &Player, hole: &Hole, wager: f64) -> f64`
  - Monte Carlo: run 10,000 trials, average net gain/loss
  - Should equal `wager * (RTP - 1)` within tolerance
- [ ] Implement `validate_rtp_across_skills(hole: &Hole, handicap_range: Vec<u8>) -> Vec<(u8, f64)>`
  - For each handicap, simulate 10,000 shots
  - Calculate actual RTP: `total_won / total_wagered`
  - Assert all RTPs within ±2% of posted RTP
- [ ] Implement `calculate_fairness_metric(hole: &Hole) -> FairnessReport`
  - Compare EV for handicap 0 vs 30
  - Report multiplier ratio and EV difference
  - Should be < 1% difference
- [ ] Implement `analyze_kalman_convergence(session: &SessionResult) -> ConvergenceReport`
  - Track error_covariance over time
  - Calculate skill confidence trajectory
  - Flag if confidence plateaus before 80%

### 4.2 Data Export (`export.rs`)
- [ ] Implement `export_session_csv(result: &SessionResult, path: &str) -> Result<()>`
  - Columns: shot_num, hole, wager, miss_distance, multiplier, payout, cumulative_net
- [ ] Implement `export_venue_json(result: &VenueResult, path: &str) -> Result<()>`
  - Full nested structure for visualization tools
- [ ] Implement `export_heatmap_csv(heatmap: &HeatmapData, path: &str) -> Result<()>`
  - Matrix format: rows=distances, cols=handicaps, values=hold%
- [ ] Implement `export_pmax_history(player: &Player, path: &str) -> Result<()>`
  - Time series of P_max values for each club category

---

## Phase 5: CLI Interface (`src/main.rs`)

### 5.1 Command Structure
```bash
continuum-golf-simulator [COMMAND] [OPTIONS]

Commands:
  player       Run player session simulation
  venue        Run venue economics simulation
  tournament   Run tournament simulation
  validate     Run validation tests
```

### 5.2 Implement Commands
- [ ] **Player Command**
  ```bash
  --handicap <0-30>           Starting handicap
  --shots <N>                 Number of shots to simulate
  --wager-min <$>            Minimum wager
  --wager-max <$>            Maximum wager
  --hole <id>                Fixed hole (or random)
  --developer-mode           Enable manual miss input
  --export <path.csv>        Export results
  ```
  - Interactive mode: prompt for each shot's manual miss if enabled
  - Print real-time stats after each batch update
  
- [ ] **Venue Command**
  ```bash
  --bays <N>                 Number of hitting bays
  --hours <H>                Operating hours
  --shots-per-hour <N>       Average shots per bay per hour
  --archetype <uniform|bell|beginners|experts>
  --export-json <path.json>
  --export-heatmap <path.csv>
  --progress                 Show progress bar
  ```
  - Use `rayon` for parallel bay simulation
  - Print summary: profit, hold%, ARPU
  
- [ ] **Tournament Command**
  ```bash
  --mode <longest|ctp>
  --hole <id>                For CTP mode
  --players <N>
  --entry-fee <$>
  --rake <percent>
  --payout <winner|top2|top3>
  --randomize                Randomize all parameters
  ```
  - Print top 10 leaderboard
  - Show financial breakdown
  
- [ ] **Validate Command**
  ```bash
  --test <all|rtp|fairness|convergence>
  --verbose                  Show detailed output
  ```
  - Run test suites, report pass/fail
  - Generate validation report

### 5.3 Output Formatting
- [ ] Pretty-print tables with alignment
- [ ] Color-code gains (green) vs losses (red)
- [ ] Show progress spinners for long operations
- [ ] ASCII art logo on startup

---

## Phase 6: Testing & Benchmarking

### 6.1 Unit Tests (`tests/`)
- [ ] Test all mathematical functions (distributions, integration, Kalman)
- [ ] Test hole payout calculations against known values
- [ ] Test player initialization and skill updates
- [ ] Test edge cases (zero wager, d > d_max, etc.)

### 6.2 Integration Tests
- [ ] Run 10,000-shot session, validate RTP within ±1%
- [ ] Verify Kalman convergence (confidence > 80% after 50 shots)
- [ ] Confirm fairness: handicap 5 vs 25 have equal EV
- [ ] Test venue simulation with different archetypes
- [ ] Validate tournament payout distribution sums correctly

### 6.3 Validation Tests (`tests/validation_tests.rs`)
Replicate business plan claims:
- [ ] **RTP by Distance**: Short=86%, Mid=88%, Long=90%
- [ ] **House Edge**: Short=14%, Mid=12%, Long=10%
- [ ] **Fairness**: All handicaps have same EV at same hole
- [ ] **Breakeven Radius**: Matches formula `d_max * (1 - P_max^(-1/k))`
- [ ] **Fat-Tail Impact**: 2% of shots increase risk by 3×
- [ ] **High-Stakes Logic**: Wager ≥10× average triggers immediate update

### 6.4 Benchmarks (`benches/`)
- [ ] Benchmark single shot simulation (target: <1μs)
- [ ] Benchmark P_max calculation (target: <100μs)
- [ ] Benchmark 10,000-shot session (target: <1s)
- [ ] Benchmark 200k-visitor venue sim (target: <10s)

---

## Phase 7: Advanced Features (Post-MVP)

### 7.1 Web Visualization
- [ ] Generate HTML reports with Plotters
- [ ] Create interactive charts (via JSON export + JavaScript)
- [ ] Real-time simulation dashboard

### 7.2 Parameter Optimization
- [ ] Grid search for optimal k and d_max values
- [ ] Genetic algorithm for maximum engagement + target RTP
- [ ] A/B testing framework

### 7.3 Machine Learning Enhancements
- [ ] Predict player churn based on loss rate
- [ ] Optimal wager recommendation engine
- [ ] Anomaly detection for cheating (sudden skill jumps)

### 7.4 Multi-Venue Modeling
- [ ] Simulate franchise network
- [ ] Regional player archetype differences
- [ ] Cross-venue player tracking

---

## Directory Structure
```
continuum-golf-simulator/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Public API
│   ├── math/
│   │   ├── mod.rs
│   │   ├── distributions.rs
│   │   ├── integration.rs
│   │   └── kalman.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── player.rs
│   │   ├── hole.rs
│   │   └── shot.rs
│   ├── simulators/
│   │   ├── mod.rs
│   │   ├── player_session.rs
│   │   ├── venue.rs
│   │   └── tournament.rs
│   ├── analytics/
│   │   ├── mod.rs
│   │   ├── metrics.rs
│   │   └── export.rs
│   └── config/
│       ├── mod.rs
│       └── constants.rs
├── tests/
│   ├── integration_tests.rs
│   └── validation_tests.rs
├── benches/
│   └── simulation_bench.rs
└── examples/
    ├── basic_session.rs
    ├── venue_simulation.rs
    └── fairness_validation.rs
```

---

## Key Improvements Over JavaScript Version

### 1. **Performance**
- **JS**: Single-threaded, slow numerical integration
- **Rust**: Parallel venue simulations, optimized math libraries
- **Expected Speedup**: 10-100× for large simulations

### 2. **Type Safety**
- **JS**: Runtime errors possible (e.g., accessing undefined fields)
- **Rust**: Compile-time guarantees, no null pointer exceptions
- **Benefit**: Catch bugs before production

### 3. **Better Kalman Filter Implementation**
- **JS**: Manual Kalman math, potential for errors
- **Rust**: Use `nalgebra` for matrix operations, more robust
- **New Features**:
  - Wager-weighted updates are properly formalized
  - Dynamic batching with explicit rules
  - Confidence score is mathematically derived, not heuristic

### 4. **Advanced Analytics**
- **JS**: Basic charts only
- **Rust**: Export data for any visualization tool
- **New Metrics**:
  - True expected value calculations (Monte Carlo)
  - Statistical tests for RTP validation
  - Convergence analysis for Kalman filter
  - Heatmaps of profitability by skill/distance

### 5. **Reproducibility**
- **JS**: Random seed not configurable
- **Rust**: Set RNG seed for deterministic testing
- **Benefit**: Debug edge cases, compare scenarios

### 6. **Modularity**
- **JS**: Monolithic HTML file
- **Rust**: Clean separation of concerns
- **Benefit**: Easy to extend, test individual components

### 7. **Validation Suite**
- **JS**: No automated tests
- **Rust**: Comprehensive test coverage
- **New Tests**:
  - RTP validation across all holes/handicaps
  - Fairness proofs (EV equality)
  - Kalman convergence tests
  - Edge case handling (zero wager, extreme mishits)

---

## Critical Mathematical Validations

### Test 1: RTP Accuracy
```rust
// For each hole, simulate 100,000 shots across handicaps 0-30
// Aggregate: total_wagered, total_won
// Assert: (total_won / total_wagered) == hole.rtp ± 0.01
```

### Test 2: Fairness (EV Equality)
```rust
// For hole H4 (150yds):
//   Player A: handicap 5  → P_max ≈ 10.2×
//   Player B: handicap 25 → P_max ≈ 7.8×
// Run 10,000 trials each, calculate average net gain
// Assert: |EV_A - EV_B| < $0.01 per $1 wagered
```

### Test 3: Kalman Convergence
```rust
// Start player at handicap 15
// Simulate 100 shots at H4
// Track error_covariance over time
// Assert: final confidence > 80%
// Assert: final σ within 10% of true σ (measured from actual shots)
```

### Test 4: Breakeven Radius
```rust
// For hole H6 (200yds, RTP=0.90, k=6.5):
//   Calculate P_max for average player
//   Calculate d_break = d_max * (1 - P_max^(-1/k))
// Simulate 10,000 shots at exactly d_break
// Assert: average multiplier ≈ 1.0 (breakeven)
```

### Test 5: High-Stakes Update Logic
```rust
// Player has shot batch [10, 12, 11] (misses in ft) with wagers [$5, $5, $5]
// Next shot: miss=8ft, wager=$100 (20× average)
// Assert: 
//   1. Batch [10,12,11] triggers update immediately
//   2. Shot [8] triggers separate immediate update
//   3. P_max history has 2 new entries
```

---

## Example CLI Usage

### Scenario 1: Beginner Testing Skill
```bash
# Simulate 50 shots as a 25-handicap beginner
continuum-golf-simulator player \
  --handicap 25 \
  --shots 50 \
  --wager-min 5 \
  --wager-max 10 \
  --export beginner_session.csv

# Output:
# Session Complete!
# ─────────────────────────────────
# Total Wagered:    $375.00
# Total Won:        $312.48
# Net Gain/Loss:    -$62.52
# Session Edge:     16.67%
# ─────────────────────────────────
# Final Skill Confidence:
#   Wedge:     78% (σ = 42.3 ft)
#   Mid-Iron:  65% (σ = 58.1 ft)
#   Long-Iron: 51% (σ = 81.7 ft)
# ─────────────────────────────────
# Results exported to: beginner_session.csv
```

### Scenario 2: Venue Economics
```bash
# Simulate a Friday night at 50 bays
continuum-golf-simulator venue \
  --bays 50 \
  --hours 8 \
  --shots-per-hour 100 \
  --archetype bell \
  --export-json venue_friday.json \
  --export-heatmap heatmap.csv \
  --progress

# Output:
# Simulating 40,000 total shots...
# ████████████████████████████████ 100%
# 
# Venue Simulation Results
# ═══════════════════════════════════════
# Total Handle:       $1,200,543.00
# Total Payouts:      $1,056,071.00
# Net Profit:         $144,472.00
# Hold Percentage:    12.03%
# ───────────────────────────────────────
# Peak Hour Profit:   Hour 6 ($28,901)
# ARPU:               $30.01
# ═══════════════════════════════════════
# Exports:
#   - venue_friday.json
#   - heatmap.csv
```

### Scenario 3: Validation
```bash
# Run all validation tests
continuum-golf-simulator validate --test all --verbose

# Output:
# Running Validation Suite...
# 
# ✓ RTP Test (Short Holes):  86.1% (target: 86.0%) ✓
# ✓ RTP Test (Mid Holes):    87.9% (target: 88.0%) ✓
# ✓ RTP Test (Long Holes):   90.2% (target: 90.0%) ✓
# ✓ Fairness Test (H4):      ΔEV = 0.003% ✓
# ✓ Kalman Convergence:      Final confidence = 84% ✓
# ✓ Breakeven Radius (H6):   1.01× (target: 1.00×) ✓
# ✓ Fat-Tail Frequency:      2.03% (target: 2.00%) ✓
# 
# All tests passed! System validated.
```

---

## Next Steps for Claude Code

1. **Start with Phase 1**: Set up project, implement core math functions
2. **Test incrementally**: After each module, write unit tests
3. **Validate against JS**: For same inputs, outputs should match (use RNG seed)
4. **Optimize after correctness**: Profile hot paths, parallelize where beneficial
5. **Document thoroughly**: Explain Kalman updates, RTP calculations, fairness proofs

**Priority Order:**
1. Math foundations (distributions, integration, Kalman)
2. Core models (Player, Hole, Shot)
3. Simple player session simulator
4. CLI for player simulation
5. Validation tests (compare to business plan claims)
6. Venue simulator
7. Export functionality
8. Tournament mode
9. Advanced analytics

---

## Success Metrics

- [ ] **Performance**: 10,000-shot simulation in <1 second
- [ ] **Accuracy**: RTP within ±1% of target for all holes
- [ ] **Fairness**: EV difference across handicaps <0.5%
- [ ] **Reliability**: 100% test coverage for math functions
- [ ] **Usability**: CLI commands are intuitive, outputs are clear
- [ ] **Validation**: All business plan claims are reproducible

---

This checklist should guide you through a complete rewrite that is faster, more robust, and better suited for rigorous economic modeling. The modular structure allows for easy extension (e.g., adding new game modes, optimizing parameters, or building a web dashboard).