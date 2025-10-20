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
- [x] Implement `rayleigh_random(sigma: f64) -> f64` (Legacy - for backward compatibility)
  - Miss distance distribution: `d = σ * sqrt(-2 * ln(U))`
- [ ] **NEW: Implement `bvn_random(mu_x: f64, mu_y: f64, sigma_x: f64, sigma_y: f64) -> (f64, f64)`**
  - Bivariate Normal distribution for (x,y) coordinates
  - Box-Muller transform for 2D sampling
  - Enables directional bias and elliptical dispersion modeling
- [ ] **NEW: Implement `bvn_pdf(x: f64, y: f64, mu_x: f64, mu_y: f64, sigma_x: f64, sigma_y: f64) -> f64`**
  - 2D Gaussian probability density function
  - Required for P_max calculation with BVN
- [x] Implement `fat_tail_shot(sigma: f64, probability: f64, multiplier: f64) -> (f64, bool)`
  - 2% chance of 3× worse dispersion (configurable)
- [x] Add helper functions: `rayleigh_pdf`, `rayleigh_mean`, `rayleigh_variance`
- [ ] **NEW: Add BVN helper functions: `bvn_mean`, `bvn_covariance`**
- [x] Add unit tests for distribution properties (mean, variance)
  - **5 tests passing**: mean, variance, fat-tail frequency, PDF properties
- [ ] **NEW: Add unit tests for BVN (symmetry check, bias detection, covariance)**

#### `integration.rs` ✅
- [x] Implement `trapezoidal_rule(f: impl Fn(f64) -> f64, a: f64, b: f64, n: usize) -> f64`
  - Numerical integration for P_max calculation
- [x] Implement `adaptive_integration` for better accuracy
- [x] Implement `simpsons_rule` for higher-order accuracy
- [x] Implement `integrate_payout_function` for P_max calculation
- [x] Add unit tests and benchmarks
  - **6 tests passing**: trapezoidal, Simpson's, adaptive, payout integration

#### `kalman.rs` ✅
- [x] Define `KalmanState` struct (1D - Legacy):
  ```rust
  pub struct KalmanState {
      pub estimate: f64,           // Current skill estimate (σ)
      pub error_covariance: f64,   // Uncertainty P_k
      pub process_noise: f64,      // Q (skill drift)
      pub initial_estimate: f64,   // σ_0 for reset
  }
  ```
- [ ] **NEW: Define `KalmanState4D` struct:**
  ```rust
  pub struct KalmanState4D {
      pub estimate: [f64; 4],              // [μ_x, μ_y, σ_x, σ_y]
      pub error_covariance: [[f64; 4]; 4], // 4×4 covariance matrix
      pub process_noise: [[f64; 4]; 4],    // Q matrix
      pub initial_estimate: [f64; 4],      // Initial state
  }
  ```
- [x] Implement `KalmanState::new(initial_sigma: f64, process_noise: f64) -> Self`
- [ ] **NEW: Implement `KalmanState4D::new(mu_x, mu_y, sigma_x, sigma_y, process_noise) -> Self`**
  - Initialize 4D state vector and covariance matrices
- [x] Implement `predict(&mut self) -> (f64, f64)`
  - Returns (predicted_estimate, predicted_covariance)
- [ ] **NEW: Implement `predict_4d(&mut self)` for 4D state**
  - Matrix operations: x_prior = x_posterior, P_prior = P_posterior + Q
- [x] Implement `update(&mut self, measurement: f64, measurement_noise: f64)`
  - Kalman gain: `K = P_k / (P_k + R)`
  - Update estimate: `σ_k = σ_k-1 + K(z - σ_k-1)`
  - Update covariance: `P_k = (1 - K) * P_k-1`
- [ ] **NEW: Implement `update_4d(&mut self, x: f64, y: f64, measurement_noise: [[f64; 2]; 2])`**
  - Multivariate Kalman update with 2D measurements (x,y)
  - Variance updates from batch statistics
- [x] Implement `calculate_confidence(&self) -> f64`
  - Maps error_covariance (50-1000) to confidence (100%-0%)
  - Formula from JS: `100 * (1 - ln(P/50) / ln(1000/50))`
- [ ] **NEW: Implement `calculate_confidence_4d(&self) -> [f64; 4]`**
  - Confidence for each dimension: μ_x, μ_y, σ_x, σ_y
- [x] Implement helper functions: `debias_rayleigh_measurement`, `weighted_average_measurement`, `measurement_variance`
- [ ] **NEW: Add 4D matrix operations: matrix_mult, matrix_inv, matrix_transpose**
  - Can use `nalgebra` or implement manually for 4×4 matrices
- [x] Add tests validating convergence over multiple updates
  - **7 tests passing**: initialization, convergence, confidence, debiasing, weighted average, variance, reset
- [ ] **NEW: Add tests for 4D Kalman (bias convergence, elliptical dispersion)**

**Phase 1 Summary:**
- ✅ **18 unit tests passing** (5 distributions + 6 integration + 7 Kalman)
- ✅ **8 doc tests passing** (all example code verified)
- ✅ **Build successful** with all dependencies
- ✅ **CLI skeleton** created with clap (4 subcommands: player, venue, tournament, validate)

---

## Phase 2: Core Data Models (`src/models/`) ✅

### 2.1 Hole Configuration (`hole.rs`) ✅
- [x] Define `ClubCategory` enum:
  ```rust
  pub enum ClubCategory {
      Wedge,      // 75-125 yds
      MidIron,    // 150-175 yds
      LongIron,   // 200-250 yds
  }
  ```
- [x] Define `Hole` struct:
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
- [x] Implement `Hole::calculate_payout(miss_distance: f64, p_max: f64) -> f64`
  - Formula: `P(d) = P_max * (1 - d/d_max)^k` if d ≤ d_max, else 0
- [x] Implement `Hole::calculate_breakeven_radius(p_max: f64) -> f64`
  - Solve: `d_break = d_max * (1 - P_max^(-1/k))`
- [x] Create `HOLE_CONFIGURATIONS: [Hole; 8]` constant with data from business plan:
  ```
  ✅ CURRENT (15% uniform house edge):
  H1: 75yd,  d_max=17.95, RTP=0.85, k=5.0
  H2: 100yd, d_max=25.69, RTP=0.85, k=5.0
  H3: 125yd, d_max=36.71, RTP=0.85, k=5.5
  H4: 150yd, d_max=47.58, RTP=0.85, k=6.0
  H5: 175yd, d_max=59.09, RTP=0.85, k=6.0
  H6: 200yd, d_max=73.58, RTP=0.85, k=6.5
  H7: 225yd, d_max=84.84, RTP=0.85, k=6.5
  H8: 250yd, d_max=101.14, RTP=0.85, k=6.5

  (OLD variable edge: H1-H2=0.86, H3-H5=0.88, H6-H8=0.90)
  ```

### 2.2 Player Model (`player.rs`) ✅
- [x] Define `Player` struct (Legacy 1D):
  ```rust
  pub struct Player {
      pub id: String,
      pub handicap: u8,           // 0-30
      pub skill_profiles: HashMap<ClubCategory, SkillProfile>,
  }

  pub struct SkillProfile {
      pub kalman_filter: KalmanState,  // 1D (σ only)
      pub p_max_history: Vec<f64>,
      pub shot_batch: Vec<ShotRecord>,
  }

  pub struct ShotRecord {
      pub miss_distance: f64,  // Radial distance only
      pub wager: f64,
  }
  ```
- [ ] **NEW: Update to 4D model for BVN:**
  ```rust
  pub struct SkillProfile {
      pub kalman_filter: KalmanState4D,  // [μ_x, μ_y, σ_x, σ_y]
      pub p_max_history: Vec<f64>,
      pub shot_batch: Vec<ShotRecord>,
  }

  pub struct ShotRecord {
      pub x_ft: f64,              // NEW: Lateral coordinate
      pub y_ft: f64,              // NEW: Distance coordinate
      pub miss_distance_ft: f64,  // Derived: sqrt(x² + y²)
      pub wager: f64,
  }
  ```
- [x] Implement `Player::new(handicap: u8) -> Self`
  - Initialize 3 skill profiles (one per club category)
  - Calculate initial σ for each: `σ_0 = distance * 3 * (0.05 + (dist-75)/(250-75)*0.01) * (0.5 + handicap/30)`
  - Start with `error_covariance = 1000` (low confidence)
- [x] Implement `calculate_initial_dispersion(handicap: u8, distance_yds: u16) -> f64`
  - Matches JS formula exactly
- [x] Implement `get_skill_for_hole(&self, hole: &Hole) -> &SkillProfile`
- [x] Implement `calculate_p_max(&self, hole: &Hole) -> f64` (1D Rayleigh - Legacy)
  - Numerical integration: `∫[0, d_max] (1 - d/d_max)^k * PDF(d | σ) dd`
  - PDF is Rayleigh: `f(d) = (d/σ²) * exp(-d²/2σ²)`
  - Solve: `P_max = RTP / integral`
- [ ] **NEW: Implement `calculate_p_max_bvn(&self, hole: &Hole) -> f64`**
  - 2D numerical integration over (x,y) space
  - Cartesian grid: `∫∫ P(x,y) * BVN_PDF(x,y | μ_x, μ_y, σ_x, σ_y) dx dy`
  - Adaptive bounds: ±4σ_x, ±4σ_y (covers 99.99% probability)
  - Payout still based on radial distance: `P(x,y) = k * (d_max / sqrt(x² + y²))^k`
- [x] Implement `update_skill(&mut self, hole: &Hole, batch: Vec<ShotRecord>, p_max: f64)` (1D - Legacy)
  - Calculate wager-weighted average miss: `z = Σ(miss_i * wager_i) / Σ(wager_i)`
  - Unbias for Rayleigh: `z_unbiased = z / sqrt(π/2)`
  - Calculate batch variance for dynamic measurement noise
  - Update Kalman filter
  - Clear shot batch, append p_max to history
- [ ] **NEW: Implement `update_skill_bvn(&mut self, hole: &Hole, batch: Vec<ShotRecord>)`**
  - Calculate wager-weighted means: `μ̂_x = Σ(x_i * w_i) / Σw_i`, `μ̂_y = Σ(y_i * w_i) / Σw_i`
  - Calculate sample variances: `σ̂_x² = Σ(x_i - μ̂_x)² / (n-1)`, `σ̂_y² = Σ(y_i - μ̂_y)² / (n-1)`
  - Update 4D Kalman filter with (x,y) measurements
  - No debiasing needed (BVN is unbiased)

### 2.3 Shot Outcome (`shot.rs`) ✅
- [x] Define `ShotOutcome` struct (Legacy - radial only):
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
- [ ] **NEW: Update `ShotOutcome` for (x,y) coordinates:**
  ```rust
  pub struct ShotOutcome {
      pub x_ft: f64,              // NEW: Lateral coordinate
      pub y_ft: f64,              // NEW: Distance coordinate
      pub miss_distance_ft: f64,  // Derived: sqrt(x² + y²)
      pub multiplier: f64,
      pub payout: f64,
      pub wager: f64,
      pub hole_id: u8,
      pub is_fat_tail: bool,
  }
  ```
- [x] Implement `simulate_shot(sigma: f64, fat_tail_prob: f64, fat_tail_mult: f64) -> (f64, bool)` (1D - Legacy)
  - 2% chance: sample from σ * 3.0
  - 98% chance: sample from σ
  - Return (miss_distance, is_fat_tail)
- [ ] **NEW: Implement `simulate_shot_bvn(mu_x, mu_y, sigma_x, sigma_y, fat_tail_prob, fat_tail_mult) -> ((f64, f64), bool)`**
  - Sample (x,y) from BVN(μ_x, μ_y, σ_x, σ_y)
  - 2% chance: multiply σ_x and σ_y by 3.0
  - Return ((x, y), is_fat_tail)

**Phase 2 Summary:**
- ✅ **All 3 core data models implemented** (hole, player, shot)
- ✅ **52 unit tests passing** (14 hole + 14 player + 11 shot + 13 from Phase 1)
- ✅ **Comprehensive P_max calculation** with numerical integration
- ✅ **Kalman filter integration** for adaptive skill tracking
- ✅ **Shot batching and high-stakes detection** implemented
- ✅ **Full serialization support** with serde for all models

---

## Phase 3: Simulation Engines (`src/simulators/`) ✅

### 3.1 Player Session Simulator (`player_session.rs`) ✅
- [x] Define `SessionConfig` struct:
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
- [x] Define `SessionResult` struct:
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
- [x] Implement `run_session(player: &mut Player, config: SessionConfig) -> SessionResult`
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

### 3.2 Venue Economics Simulator (`venue.rs`) ✅
- [x] Define `VenueConfig` struct:
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
- [x] Define `VenueResult` struct:
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
- [x] Implement `generate_player_pool(archetype: PlayerArchetype, size: usize) -> Vec<Player>`
  - Sample handicaps based on archetype distribution
- [x] Implement `run_venue_simulation(config: VenueConfig) -> VenueResult`
  - Create virtual player pool (one per bay)
  - Calculate total_shots = bays × hours × shots_per_hour
  - Track profit at intervals for time series
  - Aggregate heatmap data (6 handicap bins × 8 holes)
  - Use parallel processing with `rayon` for speed ✅
- [x] Added comprehensive unit tests for venue simulation

### 3.3 Tournament Simulator (`tournament.rs`) ✅
- [x] Define `TournamentConfig` struct:
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
- [x] Define `TournamentResult` struct:
  ```rust
  pub struct TournamentResult {
      pub leaderboard: Vec<(String, f64)>,    // (player_id, score)
      pub total_pool: f64,
      pub house_rake: f64,
      pub prize_pool: f64,
      pub payouts: Vec<(String, f64)>,
  }
  ```
- [x] Implement `run_tournament(config: TournamentConfig) -> TournamentResult`
  - Generate players, simulate attempts (configurable per player)
  - Track best score per player (max for longest drive, min for CTP)
  - Sort leaderboard and distribute prizes correctly

**Phase 3 Summary:**
- ✅ **All 3 simulation engines implemented** (player_session, venue, tournament)
- ✅ **78 unit tests passing** (26 new tests in Phase 3 + 52 from previous phases)
- ✅ **Parallel processing** with Rayon for venue simulations
- ✅ **Comprehensive configurations** with flexible options
- ✅ **Developer mode** for manual testing and debugging
- ✅ **Heatmap data generation** for visualization
- ✅ **Multiple game modes** (Closest to Pin, Longest Drive)
- ✅ **Flexible payout structures** (Winner Takes All, Top 2, Top 3)

---

## Phase 4: Analytics & Validation (`src/analytics/`) ✅

### 4.1 Metrics (`metrics.rs`) ✅
- [x] Implement `calculate_expected_value(player: &Player, hole: &Hole, wager: f64) -> f64`
  - Monte Carlo: run 10,000 trials, average net gain/loss
  - Validates house edge calculation
- [x] Implement `validate_rtp_across_skills(hole: &Hole, handicap_range: Vec<u8>) -> Vec<RtpValidationResult>`
  - For each handicap, simulate shots
  - Calculate actual RTP: `total_won / total_wagered`
  - Verify fairness across skill levels
- [x] Implement `calculate_fairness_metric(hole: &Hole) -> FairnessReport`
  - Compare EV across all handicaps
  - Report multiplier ratio and EV difference
  - Validate fairness principle
- [x] Implement `analyze_kalman_convergence(session: &SessionResult) -> ConvergenceReport`
  - Track error_covariance over time
  - Calculate skill confidence trajectory
  - Flag convergence status

### 4.2 Data Export (`export.rs`) ✅
- [x] Implement `export_session_csv(result: &SessionResult, path: &str) -> Result<()>`
  - Columns: shot_num, hole, wager, miss_distance, multiplier, payout, cumulative_net, is_fat_tail
- [x] Implement `export_venue_json(result: &VenueResult, path: &str) -> Result<()>`
  - Full nested structure for visualization tools
- [x] Implement `export_heatmap_csv(heatmap: &HeatmapData, path: &str) -> Result<()>`
  - Matrix format: rows=distances, cols=handicaps, values=hold%
- [x] Implement `export_pmax_history(player: &Player, path: &str) -> Result<()>`
  - Time series of P_max values for each club category
- [x] Implement `export_convergence_csv` for Kalman filter analysis

**Phase 4 Summary:**
- ✅ **All analytics modules implemented** (metrics and export)
- ✅ **88 unit tests passing** (10 new tests in Phase 4 + 78 from previous phases)
- ✅ **Expected value calculations** with Monte Carlo simulation
- ✅ **RTP validation** across skill levels
- ✅ **Fairness metrics** verifying EV equality
- ✅ **Kalman convergence analysis** for skill tracking
- ✅ **Comprehensive data export** (CSV, JSON formats)
- ✅ **Demo example** showcasing all Phase 4 functionality

---

## Phase 5: CLI Interface (`src/main.rs`) ✅

### 5.1 Command Structure ✅
```bash
continuum-golf-simulator [COMMAND] [OPTIONS]

Commands:
  player       Run player session simulation
  venue        Run venue economics simulation
  tournament   Run tournament simulation
  validate     Run validation tests
```

### 5.2 Implement Commands ✅
- [x] **Player Command**
  ```bash
  --handicap <0-30>           Starting handicap
  --shots <N>                 Number of shots to simulate
  --wager-min <$>            Minimum wager
  --wager-max <$>            Maximum wager
  --hole <id>                Fixed hole (or random)
  --developer-mode           Enable manual miss input
  --export <path.csv>        Export results
  ```
  - ✅ Interactive mode: prompt for each shot's manual miss if enabled
  - ✅ Print real-time stats after each batch update

- [x] **Venue Command**
  ```bash
  --bays <N>                 Number of hitting bays
  --hours <H>                Operating hours
  --shots-per-hour <N>       Average shots per bay per hour
  --archetype <uniform|bell|beginners|experts>
  --wager-min <$>            Minimum wager
  --wager-max <$>            Maximum wager
  --export-json <path.json>
  --export-heatmap <path.csv>
  --progress                 Show progress bar
  ```
  - ✅ Use `rayon` for parallel bay simulation
  - ✅ Print summary: profit, hold%, ARPU

- [x] **Tournament Command**
  ```bash
  --mode <longest|ctp>
  --hole <id>                For CTP mode
  --players <N>
  --entry-fee <$>
  --rake <percent>
  --payout <winner|top2|top3>
  --attempts <N>             Attempts per player
  ```
  - ✅ Print top 10 leaderboard
  - ✅ Show financial breakdown

- [x] **Validate Command**
  ```bash
  --test <all|rtp|fairness|convergence>
  --verbose                  Show detailed output
  ```
  - ✅ Run test suites, report pass/fail
  - ✅ Generate validation report

### 5.3 Output Formatting ✅
- [x] Pretty-print tables with alignment (using prettytable-rs)
- [x] Color-code gains (green) vs losses (red) (using colored crate)
- [x] Show progress spinners for long operations (using indicatif)
- [x] ASCII art logo on startup (custom CONTINUUM banner)

**Phase 5 Summary:**
- ✅ **All 4 CLI commands implemented** (player, venue, tournament, validate)
- ✅ **Professional output formatting** with colored tables and progress bars
- ✅ **ASCII art branding** on startup
- ✅ **Comprehensive help system** with clap
- ✅ **All features tested** and working correctly
- ✅ **Export functionality** for CSV and JSON formats

---

## Phase 6: Testing & Benchmarking ✅

### 6.1 Unit Tests (`tests/`) ✅
- [x] Test all mathematical functions (distributions, integration, Kalman)
- [x] Test hole payout calculations against known values
- [x] Test player initialization and skill updates
- [x] Test edge cases (zero wager, d > d_max, etc.)
- **88 unit tests passing** in src/ modules

### 6.2 Integration Tests ✅
- [x] Run 10,000-shot session, validate RTP within ±1%
- [x] Verify Kalman convergence (validate updates occur)
- [x] Confirm fairness: handicap 5 vs 25 have equal EV
- [x] Test venue simulation with different archetypes
- [x] Validate tournament payout distribution sums correctly
- [x] High-stakes update logic verification
- [x] Breakeven radius validation
- [x] Fat-tail impact testing
- **8 comprehensive integration tests** in tests/integration_tests.rs

### 6.3 Validation Tests (`tests/validation_tests.rs`) ✅
Replicate business plan claims:
- [x] **RTP by Distance**: ✅ **UPDATED: All holes 85% (uniform 15% house edge)**
- [x] **House Edge**: ✅ **UPDATED: All holes 15% (uniform)**
  - OLD: Short=14%, Mid=12%, Long=10% (variable)
  - NEW: All holes=15% (uniform)
- [x] **Fairness**: All handicaps have same EV at same hole
- [x] **Breakeven Radius**: Matches formula `d_max * (1 - P_max^(-1/k))`
- [x] **Fat-Tail Impact**: 2% of shots increase risk by 3×
- [x] **High-Stakes Logic**: Wager ≥10× average triggers immediate update
- [x] **Hole Configurations**: Verify all 8 holes match business plan specs
- [x] **Kalman Convergence**: Verify filter convergence properties
- [x] **Rayleigh Distribution**: Validate statistical properties (Legacy)
- [ ] **NEW: BVN Distribution**: Validate symmetry, bias detection, covariance
- [ ] **NEW: 4D Kalman Convergence**: Verify [μ_x, μ_y, σ_x, σ_y] convergence
- [ ] **NEW: P_max Consistency**: Verify 2D integration matches 1D when symmetric
- [x] **System-Wide RTP**: Comprehensive multi-hole/multi-handicap validation
- **10 validation tests passing** (Legacy) + **3 new BVN tests planned** = **13 total**

### 6.4 Benchmarks (`benches/`) ✅
- [x] Benchmark single shot simulation (target: <1μs)
- [x] Benchmark P_max calculation (target: <100μs)
- [x] Benchmark 10,000-shot session (target: <1s)
- [x] Benchmark venue simulations (small/medium/large)
- [x] Benchmark tournament simulations
- [x] Benchmark mathematical distributions
- [x] Benchmark Kalman filter operations
- [x] Benchmark integration methods
- [x] Benchmark shot batch operations
- [x] Benchmark complete shot workflow
- [x] Benchmark player generation
- [x] Benchmark heatmap data generation
- **13 comprehensive benchmark groups** in benches/simulation_bench.rs

**Phase 6 Summary:**
- ✅ **88 unit tests passing** (all mathematical functions, models, simulators, analytics)
- ✅ **8 integration tests** (RTP, fairness, convergence, high-stakes, breakeven, fat-tail, venue, tournament)
- ✅ **10 validation tests** (all business plan claims verified)
- ✅ **13 benchmark groups** (performance profiling across all components)
- ✅ **Build successful** with comprehensive test coverage
- ✅ **All critical validations passing** (RTP, fairness, convergence)
- ✅ **Ready for production** - all business logic validated

---

## Phase 7: Advanced Features (Post-MVP)

### 7.1 Data Export & Reports
- [ ] Generate CSV/JSON exports with comprehensive data
- [ ] Create summary reports with statistics
- [ ] Export heatmaps and time-series data

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

## Phase 9: Camera Integration & BVN Migration 📷

**Goal:** Transition from 1D Rayleigh (radial distance) to 2D Bivariate Normal (x,y coordinates) to enable bias detection and elliptical dispersion modeling using camera-captured ball positions.

**See:** `CAMERA_INTEGRATION.md` and `BVN_MIGRATION.md` for detailed specifications.

### 9.1 BVN Mathematical Foundations
- [ ] Implement `bvn_random()` in `src/math/distributions.rs`
- [ ] Implement `bvn_pdf()` for 2D probability density
- [ ] Add BVN unit tests (symmetry, bias, covariance)
- [ ] Benchmark BVN sampling (<1 μs per sample)

### 9.2 4D Kalman Filter
- [ ] Implement `KalmanState4D` struct in `src/math/kalman.rs`
- [ ] Add 4×4 matrix operations (mult, inv, transpose)
  - Option A: Use `nalgebra` crate
  - Option B: Manual implementation for 4×4 matrices
- [ ] Implement `predict_4d()` for state prediction
- [ ] Implement `update_4d(x, y)` for 2D measurements
- [ ] Add convergence tests for 4D state

### 9.3 P_max Calculation Update
- [ ] Implement `calculate_p_max_bvn()` with 2D integration
- [ ] Use Cartesian grid (±4σ_x, ±4σ_y adaptive bounds)
- [ ] Validate against 1D P_max (should match when μ_x=μ_y=0, σ_x=σ_y=σ)
- [ ] Benchmark performance (<10 ms per calculation)
- [ ] Add caching/memoization if needed

### 9.4 Data Model Migration
- [ ] Update `ShotRecord` with (x,y) fields
- [ ] Update `ShotOutcome` with (x,y) fields
- [ ] Modify `Player::update_skill()` to accept coordinates
- [ ] Add backward compatibility layer (radial-only → derive x,y)
- [ ] Update database schema for (x,y) storage
- [ ] Create migration script for historical data

### 9.5 Camera System Integration
- [ ] Set up camera hardware and mounting
- [ ] Implement ball detection (OpenCV contours)
- [ ] Implement ball ID reading (QR/OCR/Barcode)
- [ ] Implement homography transformation (pixel → real-world)
- [ ] Calibration process (4 corner markers)
- [ ] Accuracy validation (<2 inch RMSE target)

### 9.6 API Integration
- [ ] Create `POST /api/shots` endpoint
- [ ] Accept (x,y,ball_id) in request
- [ ] Integrate with `run_session()` simulator
- [ ] Return updated skill parameters in response
- [ ] Add error handling for camera failures

### 9.7 UI/Analytics Updates
- [ ] Add bias visualization (systematic miss direction)
- [ ] Add elliptical confidence regions (σ_x vs σ_y)
- [ ] Add player tendency heatmaps
- [ ] Add bias-adjusted coaching tips
- [ ] Create operator analytics dashboard

### 9.8 Testing & Validation
- [ ] **Test 1: Symmetry Check** - BVN reduces to Rayleigh when μ_x=μ_y=0, σ_x=σ_y
- [ ] **Test 2: Bias Detection** - Kalman correctly estimates systematic bias
- [ ] **Test 3: P_max Consistency** - 2D integration matches 1D for symmetric case
- [ ] **Test 4: RTP Preservation** - 85% RTP maintained with BVN
- [ ] **Test 5: Performance** - P_max calculation <10 ms
- [ ] **Test 6: Camera Accuracy** - Position measurement <2 inch RMSE

**Phase 9 Summary:**
- ⏳ **Mathematical foundations** (BVN distribution, 4D Kalman)
- ⏳ **P_max migration** (1D → 2D integration)
- ⏳ **Data model updates** (radial → Cartesian coordinates)
- ⏳ **Camera integration** (hardware, CV pipeline, API)
- ⏳ **UI enhancements** (bias visualization, coaching)
- ⏳ **Comprehensive testing** (6 validation tests)

**Timeline:** 4-7 weeks
**Dependencies:** Phase 1-6 complete, camera hardware acquired
**Business Impact:** Enables bias detection, improves fairness perception, unlocks coaching features

---

## Phase 8: Web Interface for Investor Showcase 🌐

**Goal:** Create an interactive web demo deployable to Vercel/GitHub Pages

### 8.1 Architecture & Setup ✅
- [x] **WebAssembly Compilation**
  - [x] Add `wasm-bindgen` and `wasm-pack` to dependencies
  - [x] Create `src/wasm.rs` module for WASM exports
  - [ ] Compile Rust simulator to WASM (runs in browser, no server needed!) - READY
  - [ ] Optimize WASM binary size (<500KB) - PENDING

- [x] **Frontend Stack Selection**
  - [x] Choose framework: React + TypeScript (recommended for investor appeal)
  - [x] Set up Vite for fast development
  - [x] Configure Tailwind CSS for professional UI

### 8.2 Core Simulator Interface (`web/`)

#### Landing Page ✅
- [x] **Hero Section**
  - [x] Tagline: "Fair, Dynamic, Profitable - Golf Wagering Reimagined"
  - [x] Quick stats: RTP ranges, fairness guarantee, Kalman adaptation
  - [ ] Animated golf ball trajectory (canvas/Three.js) - OPTIONAL
  - [ ] CTA: "Try Live Demo" button - OPTIONAL

#### Interactive Simulator Dashboard ✅
- [x] **Player Session Simulator**
  ```
  Controls:
  ✅ Handicap slider (0-30)
  ✅ Number of shots (10-1000)
  ✅ Wager range ($1-$100)
  [ ] Hole selection dropdown (H1-H8) - READY (needs WASM)

  Live Visualization:
  ✅ Running P/L chart (updating line graph)
  ✅ Skill confidence meter (0-100%)
  ✅ Summary statistics dashboard
  [ ] Real-time shot-by-shot animation - OPTIONAL
  [ ] Miss distance scatter plot - OPTIONAL
  ```

- [x] **Venue Economics Dashboard**
  ```
  Controls:
  ✅ Number of bays (10-100)
  ✅ Operating hours (1-24)
  ✅ Shots per hour (50-150)
  ✅ Wager range

  Visualizations:
  ✅ Summary statistics (handle, payouts, profit, hold%)
  ✅ Hourly revenue bar chart
  [ ] Player archetype selection - PENDING
  [ ] Hold percentage gauge - OPTIONAL
  [ ] Player distribution pie chart - OPTIONAL
  ```

- [x] **Fairness Validator**
  ```
  Interactive Proof:
  ✅ Select any hole (H1-H8)
  ✅ Run validation for handicaps 0, 10, 20, 30
  ✅ Show EV is identical (±0.5%)
  ✅ Fairness status indicator
  ✅ Comparison table with P_max and EV
  ✅ Educational explanation
  ```

### 8.3 Advanced Visualizations ✅

#### Real-Time Charts (using Recharts) ✅
- [x] **Cumulative P/L Chart**
  - [x] Line graph showing profit/loss over time
  - [x] Responsive design with CartesianGrid
  - [x] Custom tooltip and legend

- [x] **Hourly Profit Bar Chart**
  - [x] Bar chart for venue economics
  - [x] Hour-by-hour breakdown
  - [x] Professional styling

- [ ] **Shot Trajectory Viewer** - OPTIONAL
  - [ ] 2D scatter plot: distance vs. angle
  - [ ] Color-coded by payout multiplier
  - [ ] Animated shot landing with payout popup
  - [ ] Breakeven radius circle overlay

- [ ] **Kalman Filter Evolution** - PENDING (future enhancement)
  - [ ] Time-series: skill estimate (σ) over shots
  - [ ] Confidence band (P_k → shaded area)

- [ ] **Profitability Heatmap** - PENDING (future enhancement)

- [ ] **Revenue Projection Calculator** - PENDING (future enhancement)

### 8.4 Educational Content

- [ ] **"How It Works" Explainer**
  - [ ] Step-by-step animated breakdown:
    1. Player takes shot → Rayleigh distribution
    2. Miss distance measured → Kalman update
    3. Dynamic P_max calculated → Fairness maintained
    4. Payout awarded → RTP guaranteed
  - [ ] Toggle "Technical Details" for formulas

- [ ] **Math Behind the Magic**
  - [ ] Interactive Rayleigh distribution visualizer
  - [ ] Kalman filter prediction/update animation
  - [ ] P_max calculation explainer with sliders
  - [ ] "Try Your Own Parameters" sandbox

- [ ] **Competitive Analysis**
  - [ ] Side-by-side comparison: Continuum vs. Traditional TopGolf
  - [ ] Engagement metrics, revenue per bay, player retention
  - [ ] Animated infographic

### 8.5 Investor-Focused Features

- [ ] **Business Metrics Dashboard**
  - [ ] Key metrics cards:
    - Revenue per bay per hour
    - Average session length
    - Player lifetime value (estimated)
    - Profit margin breakdown
  - [ ] Growth projections chart
  - [ ] Unit economics calculator

- [ ] **Scenario Builder**
  - [ ] "What if we opened in [city]?"
  - [ ] Select: venue size, pricing, target demographic
  - [ ] Output: 5-year financial model
  - [ ] Downloadable PDF report

- [ ] **Risk Analysis**
  - [ ] Monte Carlo simulation: 1000 venue scenarios
  - [ ] Distribution of outcomes (best/worst/median)
  - [ ] Sensitivity analysis: which variables matter most?
  - [ ] "Stress Test" mode: recession, competition, etc.

### 8.6 Technical Architecture

```
┌─────────────────────────────────────────────┐
│         Frontend (React + TypeScript)       │
│  ┌──────────────────────────────────────┐  │
│  │  Components:                          │  │
│  │  - SimulatorControls                  │  │
│  │  - LiveCharts (Chart.js)              │  │
│  │  - AnimatedGolfBall (Three.js)        │  │
│  │  - DataTable (TanStack Table)         │  │
│  └──────────────────────────────────────┘  │
│                    ↓                         │
│  ┌──────────────────────────────────────┐  │
│  │     WASM Bridge (wasm-bindgen)        │  │
│  │  - run_player_session()               │  │
│  │  - run_venue_simulation()             │  │
│  │  - validate_fairness()                │  │
│  └──────────────────────────────────────┘  │
│                    ↓                         │
│  ┌──────────────────────────────────────┐  │
│  │   Rust Simulator (compiled to WASM)  │  │
│  │  - All Phase 1-6 code runs in browser│  │
│  │  - No server needed!                  │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 8.7 Implementation Steps ✅

- [x] **Step 1: WASM Bridge**
  ```rust
  // src/wasm.rs
  use wasm_bindgen::prelude::*;

  #[wasm_bindgen]
  pub fn simulate_player_session(
      handicap: u8,
      shots: usize,
      wager_min: f64,
      wager_max: f64
  ) -> JsValue {
      // Run simulation, return JSON
  }
  ```
  ✅ COMPLETED: Full WASM bridge with all functions implemented

- [ ] **Step 2: React Hooks** - READY (needs WASM compilation)
  ```typescript
  // hooks/useSimulator.ts
  import init, { simulate_player_session } from '../wasm/continuum_simulator';

  export function useSimulator() {
      const [results, setResults] = useState(null);
      const runSimulation = async (params) => {
          await init(); // Load WASM
          const data = simulate_player_session(...params);
          setResults(JSON.parse(data));
      };
      return { results, runSimulation };
  }
  ```

- [x] **Step 3: Deploy Pipeline**
  - [x] Created GitHub Actions workflow for automated deployment
  - [ ] Build WASM: `wasm-pack build --target web` - READY
  - [ ] Build frontend: `npm run build` - READY
  - [ ] Deploy to GitHub Pages - AUTOMATED via workflow
  - [ ] Custom domain: `continuum-demo.com` - OPTIONAL

### 8.8 UI/UX Design Requirements

- [ ] **Brand Identity**
  - [ ] Color scheme: Golf green (#2D5016), Gold (#D4AF37), Dark navy (#1A1D29)
  - [ ] Typography: Montserrat (headings), Inter (body)
  - [ ] Logo: Minimalist golf ball with data waves

- [ ] **Responsive Design**
  - [ ] Desktop (1920×1080): Full dashboard layout
  - [ ] Tablet (768×1024): Stacked components
  - [ ] Mobile (375×667): Simplified controls, scrollable charts

- [ ] **Accessibility**
  - [ ] WCAG 2.1 AA compliance
  - [ ] Keyboard navigation
  - [ ] Screen reader support
  - [ ] High contrast mode

### 8.9 Performance Targets

- [ ] **Load Time**
  - [ ] Initial page load: <2 seconds
  - [ ] WASM initialization: <500ms
  - [ ] Time to interactive: <3 seconds

- [ ] **Simulation Speed**
  - [ ] 100 shots: <100ms
  - [ ] 1,000 shots: <500ms
  - [ ] 10,000 shots: <3 seconds
  - [ ] Real-time updates: 60 FPS animations

- [ ] **Bundle Size**
  - [ ] WASM binary: <500 KB (gzipped)
  - [ ] JavaScript: <200 KB
  - [ ] Total: <1 MB initial load

### 8.10 Deployment & Sharing ✅

- [x] **GitHub Pages Setup**
  - [x] Configured GitHub Actions workflow
  - [x] Auto-deploy on push to main
  - [ ] gh-pages branch will be created automatically
  - [ ] URL: `https://iansabia.github.io/Continuum_algo` - PENDING (first deploy)

- [ ] **Vercel Alternative** - OPTIONAL
  - [ ] Connect GitHub repo
  - [ ] Auto-deploy on push
  - [ ] Custom domain support
  - [ ] URL: `https://continuum-golf.vercel.app`

- [ ] **Shareable Features** - PENDING (future enhancement)
  - [ ] Export simulation results to PDF
  - [ ] Share link with pre-configured scenario
  - [ ] Embed widget for pitch decks
  - [ ] QR code generator for tablet demos

### 8.11 Investor Pitch Integration

- [ ] **Pitch Deck Companion**
  - [ ] "Live Demo" slide with QR code
  - [ ] Embedded calculator in deck
  - [ ] Video recording of key features

- [ ] **Meeting Tools**
  - [ ] "Presenter Mode" (simplified controls)
  - [ ] Pre-loaded impressive scenarios
  - [ ] One-click reset to demo state
  - [ ] Export button for meeting notes

---

## Phase 8 Success Metrics

- [ ] **Functionality**: All core simulations work in browser via WASM
- [ ] **Performance**: <3s load time, smooth 60 FPS animations
- [ ] **Usability**: Non-technical investors can run scenarios independently
- [ ] **Visual Impact**: Professional, polished UI that "wows" on first impression
- [ ] **Shareability**: Easy to send link, works on mobile/tablet
- [ ] **Credibility**: Technical depth visible but not overwhelming

---

**Phase 8 Priority:** HIGH (for investor showcase)
**Estimated Timeline:** 2-3 weeks after Phase 6 completion
**Technologies:** Rust + WASM + React + TypeScript + Chart.js + Tailwind
**Deployment:** Vercel (recommended) or GitHub Pages

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