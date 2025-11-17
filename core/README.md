# Continuum Golf Simulator - Core

Production-ready golf wagering simulator with skill-adaptive odds and anti-cheat detection.

## What Is This?

A Rust-based wagering engine designed to integrate with real golf simulators (TrackMan, GCQuad, GSPro, etc.). Players take shots, wager money, and receive payouts based on accuracy - with mathematically fair odds that adapt to skill level.

**Key Features:**
- **Adaptive Odds**: MCMC Bayesian inference adjusts to each player's skill in real-time
- **Provably Fair**: Equal expected value across all handicap levels
- **Anti-Cheat**: ML-based detection for suspicious betting patterns
- **High Performance**: Rust + WASM, optimized for real-time use

## Business Model

- **Target RTP**: 86-90% return to player (distance-dependent)
- **House Hold**: 10-14% consistent across all skill levels
- **Revenue**: Scales with shot volume, not player skill exploitation

## Quick Start

### Build Native Binary
```bash
cargo build --release
./target/release/continuum-golf-simulator --help
```

### Build WASM Module
```bash
wasm-pack build --target web --out-dir pkg
```

### Run Tests
```bash
cargo test --release
```

## Integration with Golf Simulators

### Current Status
✅ Core engine complete (odds, payouts, skill tracking)
✅ WASM bindings for web integration
❌ Hardware adapter layer (in development)

### Integration Strategy

**Phase 1: GSPro Connect API** (Universal - Supports 20+ Brands)
- TrackMan, GCQuad, Uneekor, SkyTrak, Mevo+, etc.
- WebSocket-based real-time shot data
- Fastest path to multi-brand support

**Phase 2: Direct Integrations** (Performance Optimization)
- Uneekor Open API
- TrackMan SDK (if available)
- Foresight API

### API Keys & Access

**GSPro Connect API:**
- Requires GSPro software license (~$250/year per simulator)
- API access included with license
- WebSocket endpoint: `ws://localhost:921` (local network)
- Documentation: https://gsprogolf.com/GSProConnect

**Shot Data Format:**
The core engine expects normalized shot data:
```rust
pub struct ShotOutcome {
    pub miss_distance_ft: f64,  // Radial distance from pin
    pub x_ft: Option<f64>,       // Lateral position (optional)
    pub y_ft: Option<f64>,       // Distance position (optional)
    pub multiplier: f64,         // Calculated payout multiplier
    pub payout: f64,             // Actual payout amount
    pub wager: f64,              // Amount wagered
    pub hole_id: u8,             // Target hole identifier
    pub is_fat_tail: bool,       // Extreme mishit flag
    pub p_max: f64,              // Win probability
}
```

**Adapter Layer** (Coming Soon):
```rust
// Convert GSPro data to Continuum format
impl From<GSProShotData> for ShotInput {
    fn from(data: GSProShotData) -> Self {
        ShotInput {
            carry_distance: data.carry_distance_yards,
            offline_distance: data.offline_yards,
            target_pin_x: data.pin_location.x,
            target_pin_y: data.pin_location.y,
        }
    }
}
```

### Venue Setup Requirements

**Hardware:**
- Golf simulator with shot tracking (any brand compatible with GSPro)
- Computer running GSPro software
- Network connection for API access

**Software:**
- GSPro license (required)
- Continuum integration service (provided)
- Optional: Venue management dashboard

**Per-Bay Configuration:**
```toml
[venue]
name = "TopGolf Stadium Drive"
num_bays = 12

[simulator]
brand = "TrackMan"
api_type = "GSPro Connect"
endpoint = "ws://192.168.1.100:921"

[wagering]
min_bet = 1.00
max_bet = 100.00
target_rtp = 0.88
```

## Implementation Roadmap

### Phase 1: GSPro Integration (1-2 weeks)
- [ ] WebSocket client for GSPro Connect API
- [ ] Shot data normalization adapter
- [ ] Real-time skill update on each shot
- [ ] Multi-bay venue support
- [ ] Connection health monitoring

### Phase 2: Venue Management (2-3 weeks)
- [ ] Bay configuration and assignment
- [ ] Player authentication/tracking
- [ ] Transaction processing
- [ ] Daily/weekly reporting
- [ ] Admin dashboard

### Phase 3: Advanced Features (4+ weeks)
- [ ] Tournament mode
- [ ] Multi-venue analytics
- [ ] Mobile kiosk interface
- [ ] Direct hardware integrations (Uneekor, TrackMan)

## Mathematical Framework

**Core Algorithm:**
1. **Shot Input**: Player hits shot, simulator reports (distance, lateral offset)
2. **Skill Update**: MCMC Bayesian inference adjusts player's dispersion estimate (σ)
3. **P_max Calculation**: Bivariate normal CDF determines win probability
4. **Payout**: If shot wins, player receives `wager × multiplier`

**Fairness Guarantee:**
All handicap levels have equal expected value (EV ≈ -12% house edge). A scratch golfer and a 30-handicap both lose the same percentage over time.

## Project Structure

```
core/
├── src/
│   ├── math/           # BVN distributions, MCMC, integration
│   ├── models/         # Player, Hole, Shot data structures
│   ├── simulators/     # Session, Venue, Tournament logic
│   ├── analytics/      # Metrics, exports, fairness validation
│   ├── anti_cheat.rs   # ML-based fraud detection
│   ├── lib.rs          # Public API exports
│   ├── main.rs         # CLI binary
│   └── wasm.rs         # Browser bindings
├── tests/              # Integration and validation tests
├── benches/            # Performance benchmarks
└── examples/           # Demo programs
```

## Performance

- **Shot Processing**: <1ms per shot
- **MCMC Update**: <5ms (skill estimation)
- **WASM Module**: ~800KB compressed
- **Memory**: <50MB per venue (100+ concurrent players)

## License

MIT

---

**For Demo/Marketing**: See `../web-demo/`
**For Business/Investment**: Contact info@continuum.golf
