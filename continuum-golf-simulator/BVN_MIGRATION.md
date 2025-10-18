# Bivariate Normal Distribution Migration Plan

## Date: October 18, 2025

## Executive Summary

This document outlines the migration from **Rayleigh distribution** (1D radial distance model) to **Bivariate Normal distribution** (2D coordinate-based model with systematic bias detection). This upgrade enables the simulator to model directional shot tendencies and elliptical dispersion patterns using (x,y) coordinate data from the camera system.

---

## Why Migrate to BVN?

### Current Limitations (Rayleigh Distribution)

**What It Models:**
- Radial miss distance only: `d = sqrt(x² + y²)`
- Assumes **circular symmetry** (no directional bias)
- Single parameter: `σ` (radial standard deviation)

**What It Cannot Model:**
1. **Systematic Bias:** Player consistently missing left/right or long/short
2. **Directional Variance:** Different accuracy in lateral vs. distance control
3. **Elliptical Dispersion:** Shot patterns that are wider than they are long (or vice versa)

**Example Limitation:**
```
Player A: Misses 10 ft right, 5 ft short (consistently)
Player B: Misses randomly in all directions (average 11.2 ft)

Rayleigh sees both as: σ ≈ 11-12 ft (same skill)
Reality: Player A has bias, Player B doesn't
```

### New Capabilities (Bivariate Normal Distribution)

**What It Models:**
- Full (x,y) coordinate distribution
- 4 parameters: `[μ_x, μ_y, σ_x, σ_y]`
  - `μ_x`: Lateral bias (average miss left/right)
  - `μ_y`: Distance bias (average miss long/short)
  - `σ_x`: Lateral precision (standard deviation left/right)
  - `σ_y`: Distance precision (standard deviation long/short)

**New Features Enabled:**
1. **Bias Detection:** "You miss 3 ft right and 2 ft long on average"
2. **Directional Coaching:** "Work on lateral control (σ_x = 15 ft, σ_y = 8 ft)"
3. **Adaptive Payouts:** Better shots in weak direction get higher rewards
4. **Fraud Detection:** Unnatural bias patterns indicate cheating

**Example Improvement:**
```
Player A: [μ_x = +10, μ_y = -5, σ_x = 8, σ_y = 6]
  → Systematic 10 ft right, 5 ft short bias
  → Moderate precision in both axes

Player B: [μ_x = 0, μ_y = 0, σ_x = 12, σ_y = 12]
  → No systematic bias
  → Lower precision (same as Player A's radial σ ≈ 12)

BVN correctly distinguishes these patterns.
Rayleigh treats them identically.
```

---

## Mathematical Model Transition

### Rayleigh Distribution (Current)

**Probability Density Function:**
```
f(d | σ) = (d / σ²) × exp(-d² / (2σ²))

Where:
  d = sqrt(x² + y²)  (radial distance)
  σ = radial standard deviation
```

**Random Sampling:**
```rust
pub fn rayleigh_random(sigma: f64) -> f64 {
    let u: f64 = rng.gen();
    sigma * (-2.0 * u.ln()).sqrt()
}
```

**Parameter Estimation:**
- Observed mean: `E[d] = σ × sqrt(π/2) ≈ 1.253σ`
- Debiased estimate: `σ = d_mean / sqrt(π/2)`

### Bivariate Normal Distribution (Proposed)

**Probability Density Function:**
```
f(x, y | μ_x, μ_y, σ_x, σ_y) =
    (1 / (2π × σ_x × σ_y)) ×
    exp(-0.5 × [(x - μ_x)²/σ_x² + (y - μ_y)²/σ_y²])

Where:
  x, y = Cartesian coordinates (ft from pin)
  μ_x, μ_y = Mean bias in each direction
  σ_x, σ_y = Standard deviation in each direction
```

**Random Sampling (Box-Muller Transform):**
```rust
pub fn bvn_random(mu_x: f64, mu_y: f64, sigma_x: f64, sigma_y: f64) -> (f64, f64) {
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();

    // Box-Muller transform
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
    let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).sin();

    // Scale and shift
    let x = mu_x + sigma_x * z0;
    let y = mu_y + sigma_y * z1;

    (x, y)
}
```

**Parameter Estimation:**
- Sample mean: `μ̂_x = Σx_i / n`, `μ̂_y = Σy_i / n`
- Sample variance: `σ̂_x² = Σ(x_i - μ̂_x)² / (n-1)`, `σ̂_y² = Σ(y_i - μ̂_y)² / (n-1)`

### Comparison Table

| Aspect | Rayleigh | Bivariate Normal |
|--------|----------|------------------|
| **Dimensions** | 1D (radial) | 2D (Cartesian) |
| **Parameters** | 1 (`σ`) | 4 (`μ_x, μ_y, σ_x, σ_y`) |
| **Symmetry** | Circular | Elliptical |
| **Bias Detection** | ❌ No | ✅ Yes |
| **Data Required** | Distance only | (x,y) coordinates |
| **Complexity** | Low | Medium |
| **Realism** | Moderate | High |

---

## Kalman Filter Migration

### Current Implementation (1D Scalar)

**State Vector:**
```rust
pub struct KalmanState {
    pub estimate: f64,           // σ (single value)
    pub error_covariance: f64,   // P_k (scalar uncertainty)
}
```

**Update Equation:**
```rust
// Predict
estimate_prior = estimate_posterior
P_prior = P_posterior + process_noise

// Update
K = P_prior / (P_prior + measurement_noise)
estimate_posterior = estimate_prior + K × (measurement - estimate_prior)
P_posterior = (1 - K) × P_prior
```

### Proposed Implementation (4D Vector)

**State Vector:**
```rust
pub struct KalmanState {
    pub estimate: [f64; 4],              // [μ_x, μ_y, σ_x, σ_y]
    pub error_covariance: [[f64; 4]; 4], // 4×4 covariance matrix
}
```

**Update Equation (Multivariate):**
```rust
// Predict
x_prior = x_posterior
P_prior = P_posterior + Q  // Q is 4×4 process noise

// Update
innovation = measurement - H × x_prior  // H is measurement matrix
S = H × P_prior × H^T + R  // Innovation covariance
K = P_prior × H^T × S^(-1)  // Kalman gain (4×4 matrix)

x_posterior = x_prior + K × innovation
P_posterior = (I - K × H) × P_prior
```

**Measurement Matrix (H):**
```rust
// We observe (x, y) directly, but estimate [μ_x, μ_y, σ_x, σ_y]
// Single measurement affects means, batch affects variances

// For single shot (x_i, y_i):
H = [[1, 0, 0, 0],   // x measurement observes μ_x
     [0, 1, 0, 0]]   // y measurement observes μ_y

// For batch variance update:
// Compute sample variances separately and update σ_x, σ_y
```

### Migration Complexity

**What Changes:**
- Scalar → Vector/Matrix operations
- 1 estimate → 4 estimates
- Simple algebra → Linear algebra (matrix inversion)
- 5-10 lines of code → 30-50 lines

**Libraries Needed:**
- `nalgebra` for matrix operations
- Or implement simple 4×4 matrix inversion manually

---

## P_max Calculation Migration

### Current Approach (1D Integration)

**Formula:**
```
P_max = ∫[0 to ∞] P(d) × f(d | σ) dd

Where:
  P(d) = k × (d_max / d)^k  (payout curve)
  f(d | σ) = Rayleigh PDF
```

**Implementation:**
```rust
pub fn calculate_p_max_fresh(&self, hole: &Hole) -> f64 {
    let sigma = skill.kalman_filter.estimate;
    let k = hole.k;
    let d_max = hole.d_max_ft;

    // Numerical integration over radial distance
    let mut sum = 0.0;
    for i in 0..1000 {
        let d = i as f64 * 0.1;  // 0 to 100 ft in 0.1 ft steps
        let payout = k * (d_max / d.max(1.0)).powf(k);
        let prob = rayleigh_pdf(d, sigma);
        sum += payout * prob * 0.1;  // Riemann sum
    }
    sum
}
```

### Proposed Approach (2D Integration)

**Formula (Cartesian Coordinates):**
```
P_max = ∫∫ P(x, y) × f(x, y | μ_x, μ_y, σ_x, σ_y) dx dy

Where:
  P(x, y) = k × (d_max / sqrt(x² + y²))^k  (payout still based on distance)
  f(x, y) = Bivariate Normal PDF
  Integration over: x ∈ [-∞, +∞], y ∈ [-∞, +∞]
```

**Implementation (Cartesian Grid):**
```rust
pub fn calculate_p_max_bvn(&self, hole: &Hole) -> f64 {
    let mu_x = state[0];
    let mu_y = state[1];
    let sigma_x = state[2];
    let sigma_y = state[3];
    let k = hole.k;
    let d_max = hole.d_max_ft;

    // Practical bounds: ±4σ covers 99.99% of probability
    let x_min = mu_x - 4.0 * sigma_x;
    let x_max = mu_x + 4.0 * sigma_x;
    let y_min = mu_y - 4.0 * sigma_y;
    let y_max = mu_y + 4.0 * sigma_y;

    let step = 0.5;  // 0.5 ft grid resolution
    let mut sum = 0.0;

    let mut x = x_min;
    while x <= x_max {
        let mut y = y_min;
        while y <= y_max {
            let d = (x * x + y * y).sqrt().max(1.0);
            let payout = k * (d_max / d).powf(k);
            let prob = bvn_pdf(x, y, mu_x, mu_y, sigma_x, sigma_y);
            sum += payout * prob * step * step;  // Area element
            y += step;
        }
        x += step;
    }
    sum
}
```

**Alternative: Polar Coordinates**
```rust
// May be more efficient for circular integration bounds
// Convert (r, θ) → (x, y) = (r cos θ, r sin θ)
// Integrate over r ∈ [0, ∞], θ ∈ [0, 2π]

pub fn calculate_p_max_bvn_polar(&self, hole: &Hole) -> f64 {
    // Integration in polar coordinates with Jacobian r
    let mut sum = 0.0;
    for r in (0..200).map(|i| i as f64 * 0.5) {  // 0 to 100 ft
        for theta in (0..360).map(|i| i as f64 * PI / 180.0) {
            let x = r * theta.cos();
            let y = r * theta.sin();
            let payout = k * (d_max / r.max(1.0)).powf(k);
            let prob = bvn_pdf(x, y, mu_x, mu_y, sigma_x, sigma_y);
            sum += payout * prob * r * (PI / 180.0) * 0.5;  // r dθ dr
        }
    }
    sum
}
```

### Computational Complexity

| Method | Grid Size | Evaluations | Time Estimate |
|--------|-----------|-------------|---------------|
| 1D Rayleigh | 1000 points | 1,000 | <1 ms |
| 2D Cartesian | 200×200 | 40,000 | ~5 ms |
| 2D Polar | 200×360 | 72,000 | ~10 ms |
| Adaptive Quadrature | Variable | 5,000-20,000 | ~3 ms |

**Recommendation:** Use 2D Cartesian grid with adaptive bounds (±4σ) for balance of accuracy and speed.

---

## Data Model Changes

### Shot Record Structure

**Current:**
```rust
pub struct ShotRecord {
    pub miss_distance_ft: f64,  // Only radial distance
    pub wager: f64,
}
```

**Proposed:**
```rust
pub struct ShotRecord {
    pub x_ft: f64,              // NEW: Lateral coordinate
    pub y_ft: f64,              // NEW: Distance coordinate
    pub miss_distance_ft: f64,  // Derived: sqrt(x² + y²)
    pub wager: f64,
}
```

### Shot Outcome Structure

**Current:**
```rust
pub struct ShotOutcome {
    pub miss_distance_ft: f64,
    pub multiplier: f64,
    pub payout: f64,
    pub wager: f64,
    pub hole_id: u8,
    pub is_fat_tail: bool,
}
```

**Proposed:**
```rust
pub struct ShotOutcome {
    pub x_ft: f64,              // NEW
    pub y_ft: f64,              // NEW
    pub miss_distance_ft: f64,  // Kept for backward compatibility
    pub multiplier: f64,
    pub payout: f64,
    pub wager: f64,
    pub hole_id: u8,
    pub is_fat_tail: bool,
}
```

### Skill Profile Structure

**Current:**
```rust
pub struct SkillProfile {
    pub club_category: ClubCategory,
    pub kalman_filter: KalmanState,
    pub p_max_history: Vec<f64>,
}

pub struct KalmanState {
    pub estimate: f64,           // σ
    pub error_covariance: f64,
}
```

**Proposed:**
```rust
pub struct SkillProfile {
    pub club_category: ClubCategory,
    pub kalman_filter: KalmanState4D,  // NEW
    pub p_max_history: Vec<f64>,
}

pub struct KalmanState4D {
    pub estimate: [f64; 4],              // [μ_x, μ_y, σ_x, σ_y]
    pub error_covariance: [[f64; 4]; 4], // 4×4 matrix
}
```

---

## Implementation Phases

### Phase 1: Mathematical Foundations (Week 1)

**Tasks:**
- ✅ Document BVN migration plan (this file)
- ⏳ Implement BVN distribution functions in `src/math/distributions.rs`
  - `bvn_random(mu_x, mu_y, sigma_x, sigma_y) -> (f64, f64)`
  - `bvn_pdf(x, y, mu_x, mu_y, sigma_x, sigma_y) -> f64`
- ⏳ Add unit tests for BVN (compare to known values)
- ⏳ Benchmark performance (target: <1 μs per sample)

**Deliverables:**
- `src/math/distributions.rs` with BVN functions
- Tests passing for standard cases (μ=0, σ=1)

### Phase 2: Kalman Filter Upgrade (Weeks 2-3)

**Tasks:**
- ⏳ Implement 4D Kalman filter in `src/math/kalman.rs`
  - Add matrix operations (or import `nalgebra`)
  - Implement predict() for 4D state
  - Implement update() with 4×4 covariance
- ⏳ Add initialization logic (convert handicap → [μ_x, μ_y, σ_x, σ_y])
- ⏳ Test convergence with simulated (x,y) data
- ⏳ Validate against 1D Kalman (when μ_x = μ_y = 0, σ_x = σ_y = σ)

**Deliverables:**
- `KalmanState4D` struct with full functionality
- Tests showing convergence to true parameters

### Phase 3: P_max Calculation Update (Week 4)

**Tasks:**
- ⏳ Implement 2D numerical integration in `src/models/player.rs`
  - Cartesian grid method with adaptive bounds
  - Polar coordinate alternative (optional)
- ⏳ Add caching/memoization for performance
- ⏳ Validate against 1D P_max (should match when symmetric)
- ⏳ Benchmark computation time (target: <10 ms)

**Deliverables:**
- `calculate_p_max_bvn()` function
- Performance tests showing acceptable latency

### Phase 4: Data Model Migration (Week 5)

**Tasks:**
- ⏳ Update `ShotRecord` and `ShotOutcome` with (x,y) fields
- ⏳ Modify `Player::update_skill()` to accept (x,y) coordinates
- ⏳ Add backward compatibility (derive x,y from radial if needed)
- ⏳ Update database schema to store coordinates
- ⏳ Migrate existing data (set x=0, y=d for old radial-only shots)

**Deliverables:**
- Updated data structures
- Migration script for historical data
- Backward compatibility layer

### Phase 5: Camera Integration (Weeks 6-7)

**Tasks:**
- ⏳ Implement camera capture and homography (see CAMERA_INTEGRATION.md)
- ⏳ Create API endpoint for (x,y) shot submission
- ⏳ Integrate with player_session simulator
- ⏳ Test end-to-end flow: camera → (x,y) → Kalman → payout
- ⏳ Validate accuracy with 100+ real shots

**Deliverables:**
- Working camera system at pilot venue
- API endpoint accepting (x,y) coordinates
- Validation report showing <2 inch accuracy

### Phase 6: UI and Analytics (Week 8+)

**Tasks:**
- ⏳ Add bias visualization to web interface
- ⏳ Display elliptical confidence regions (σ_x vs σ_y)
- ⏳ Show player tendency heatmaps
- ⏳ Implement bias-adjusted coaching tips
- ⏳ Create analytics dashboard for operators

**Deliverables:**
- Updated web interface with BVN visualizations
- Player-facing bias reports
- Operator analytics dashboard

---

## Validation & Testing

### Test 1: Symmetry Check

**Goal:** Verify BVN reduces to Rayleigh when bias is zero and variances are equal.

**Test Case:**
```rust
let mu_x = 0.0;
let mu_y = 0.0;
let sigma_x = 50.0;
let sigma_y = 50.0;

// Generate 10,000 samples from BVN
let samples: Vec<(f64, f64)> = (0..10000)
    .map(|_| bvn_random(mu_x, mu_y, sigma_x, sigma_y))
    .collect();

// Convert to radial distances
let radial: Vec<f64> = samples.iter()
    .map(|(x, y)| (x*x + y*y).sqrt())
    .collect();

// Compare to Rayleigh(σ=50)
let rayleigh_samples: Vec<f64> = (0..10000)
    .map(|_| rayleigh_random(50.0))
    .collect();

// Statistical test: Kolmogorov-Smirnov
assert!(ks_test(&radial, &rayleigh_samples) > 0.05);  // p-value > 0.05
```

**Pass Criteria:** K-S test p-value > 0.05 (distributions are statistically identical)

### Test 2: Bias Detection

**Goal:** Verify Kalman filter correctly estimates systematic bias.

**Test Case:**
```rust
// True parameters
let true_mu_x = 10.0;   // 10 ft right bias
let true_mu_y = -5.0;   // 5 ft short bias
let true_sigma_x = 8.0;
let true_sigma_y = 6.0;

// Initialize Kalman with no prior knowledge
let mut kalman = KalmanState4D::new_unbiased();

// Feed 100 shots with true bias
for _ in 0..100 {
    let (x, y) = bvn_random(true_mu_x, true_mu_y, true_sigma_x, true_sigma_y);
    kalman.update(x, y);
}

// Check convergence
assert!((kalman.estimate[0] - true_mu_x).abs() < 1.0);  // μ_x within 1 ft
assert!((kalman.estimate[1] - true_mu_y).abs() < 1.0);  // μ_y within 1 ft
assert!((kalman.estimate[2] - true_sigma_x).abs() < 2.0);  // σ_x within 2 ft
assert!((kalman.estimate[3] - true_sigma_y).abs() < 2.0);  // σ_y within 2 ft
```

**Pass Criteria:** All parameter estimates within tolerance after 100 shots

### Test 3: P_max Consistency

**Goal:** Verify 2D integration matches 1D when appropriate.

**Test Case:**
```rust
// Symmetric case (should match Rayleigh)
let mu_x = 0.0;
let mu_y = 0.0;
let sigma_x = 50.0;
let sigma_y = 50.0;

let hole = HOLE_CONFIGURATIONS[3];  // 150-yard hole

// Calculate P_max both ways
let p_max_1d = calculate_p_max_rayleigh(&hole, 50.0);
let p_max_2d = calculate_p_max_bvn(&hole, mu_x, mu_y, sigma_x, sigma_y);

// Should be within 1% due to numerical integration tolerance
assert!((p_max_1d - p_max_2d).abs() / p_max_1d < 0.01);
```

**Pass Criteria:** <1% difference between 1D and 2D methods for symmetric case

### Test 4: RTP Preservation

**Goal:** Verify target RTP (85%) is maintained with BVN.

**Test Case:**
```rust
// Simulate 1000 shots with BVN
let player = Player::new_with_handicap(15);
let hole = HOLE_CONFIGURATIONS[3];
let wager = 10.0;

let mut total_wagered = 0.0;
let mut total_won = 0.0;

for _ in 0..1000 {
    let (x, y) = /* simulate shot with player skill */;
    let outcome = hole.calculate_payout(x, y, wager, &player);
    total_wagered += wager;
    total_won += outcome.payout;
}

let rtp = total_won / total_wagered;
assert!((rtp - 0.85).abs() < 0.02);  // Within 2% of target
```

**Pass Criteria:** RTP within 83-87% range (target 85% ± tolerance)

### Test 5: Performance Benchmark

**Goal:** Ensure P_max calculation doesn't slow down gameplay.

**Test Case:**
```rust
use std::time::Instant;

let start = Instant::now();
for _ in 0..1000 {
    let p_max = calculate_p_max_bvn(&hole, mu_x, mu_y, sigma_x, sigma_y);
}
let duration = start.elapsed();

let avg_time_ms = duration.as_millis() / 1000;
assert!(avg_time_ms < 10);  // <10ms per calculation
```

**Pass Criteria:** Average calculation time <10 ms

---

## Backward Compatibility

### Migration Strategy

**Option A: Hard Cutover**
- Deploy BVN at specific date
- All new shots use (x,y) coordinates
- Historical shots remain radial-only
- Simple but loses bias detection for existing players

**Option B: Gradual Migration**
- Support both Rayleigh and BVN simultaneously
- Existing players stay on Rayleigh until 20+ (x,y) shots collected
- Auto-switch to BVN when sufficient data
- Complex but preserves existing skill estimates

**Recommendation:** Option B (gradual migration)

### Data Model Compatibility

**Database Schema:**
```sql
-- Add new columns (nullable for backward compatibility)
ALTER TABLE shots ADD COLUMN x_ft REAL;
ALTER TABLE shots ADD COLUMN y_ft REAL;

-- For old data, x and y are NULL
-- For new data, both are populated
-- miss_distance_ft is always populated (derived if needed)
```

**Code Logic:**
```rust
impl ShotRecord {
    pub fn new_radial(distance: f64, wager: f64) -> Self {
        // Old API: radial distance only
        ShotRecord {
            x_ft: 0.0,  // Placeholder
            y_ft: distance,  // Store in y for now
            miss_distance_ft: distance,
            wager,
        }
    }

    pub fn new_cartesian(x: f64, y: f64, wager: f64) -> Self {
        // New API: full (x,y) coordinates
        ShotRecord {
            x_ft: x,
            y_ft: y,
            miss_distance_ft: (x*x + y*y).sqrt(),
            wager,
        }
    }
}
```

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| 2D integration too slow | Medium | High | Optimize grid, use caching, benchmark early |
| Kalman doesn't converge | Low | High | Test with simulated data, tune process noise |
| Camera accuracy insufficient | Medium | Critical | Validate with test shots, require <2 inch RMSE |
| Matrix library dependency | Low | Medium | Use lightweight `nalgebra` or custom 4×4 impl |

### Business Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Players dislike coordinate system | Low | Medium | Explain benefits (bias detection, fairness) |
| Migration breaks existing gameplay | Medium | High | Extensive testing, gradual rollout |
| Increased complexity → bugs | Medium | Medium | Comprehensive test suite, code reviews |
| Operator training burden | Medium | Low | Document well, provide training materials |

---

## Success Metrics

### Technical KPIs
- ✅ BVN RTP within ±2% of target (85%)
- ✅ P_max calculation <10 ms per call
- ✅ Kalman convergence within 50 shots
- ✅ Camera accuracy <2 inch RMSE

### Business KPIs
- ✅ Zero regression in existing gameplay (for radial-only players)
- ✅ >90% camera uptime at pilot venue
- ✅ Player satisfaction ≥4.5/5 (post-migration survey)
- ✅ Bias detection catches at least 1 fraud case in first 6 months

---

## Timeline

### Development Phase (Weeks 1-8)
- Week 1: Mathematical foundations (BVN, testing)
- Weeks 2-3: Kalman filter upgrade
- Week 4: P_max calculation
- Week 5: Data model migration
- Weeks 6-7: Camera integration
- Week 8+: UI/Analytics

### Testing Phase (Weeks 9-10)
- Validation tests (symmetry, bias detection, RTP, performance)
- Integration testing with camera system
- User acceptance testing at pilot venue

### Deployment Phase (Weeks 11-12)
- Deploy to pilot venue
- Monitor for 2 weeks
- Collect feedback
- Fix any issues

### Rollout (Months 4-6)
- Deploy to additional venues
- Train operators
- Monitor long-term stability
- Optimize based on production data

**Total Estimated Time:** 4-6 months from start to full rollout

---

## Related Documentation

- See **CAMERA_INTEGRATION.md** for hardware and computer vision details
- See **continuum_checklist.md** for integration into overall project plan
- See **WEB_INTERFACE_PLAN.md** for UI/UX changes needed

---

## Appendix: BVN vs. Rician Comparison

### Why Not Rician?

**Rician Distribution:**
- Models radial distance with systematic bias: `R ~ Rice(ν, σ)`
- Parameters: `ν` (bias magnitude), `σ` (spread)
- **Limitation:** Captures bias magnitude but not direction

**Example:**
```
Player misses 10 ft right on average
vs.
Player misses 10 ft long on average

Rician sees both as: ν ≈ 10 ft (same bias magnitude)
BVN distinguishes: [μ_x=10, μ_y=0] vs. [μ_x=0, μ_y=10]
```

**When Rician Makes Sense:**
- You only have radial distance (no camera)
- You suspect bias exists but can't measure direction
- Intermediate upgrade path from Rayleigh

**Why BVN is Better:**
- You already have (x,y) data from camera
- Rician throws away directional information
- BVN enables coaching and advanced analytics

**Recommendation:** Skip Rician, go straight to BVN with camera data.

---

**Prepared By:** Engineering Team
**Classification:** Internal - Technical Specification
**Status:** 🟡 PLANNING PHASE
**Target Completion:** Q2 2026
