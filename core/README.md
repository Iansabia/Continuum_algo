# Continuum Golf Simulator - Core

**Production-ready golf wagering simulator for real-world golf simulator integration**

This is the core Continuum algorithm - the actual simulator that will be integrated with physical golf simulators at venues. This codebase contains no UI dependencies and is optimized for production use.

## What This Is

A high-performance Rust-based golf wagering simulator featuring:

- **Proprietary Odds Engine**: Dynamic P_max calculations ensuring fairness across all skill levels
- **Adaptive Skill Modeling**: MCMC Bayesian inference for real-time player skill adaptation
- **Anti-Cheat Detection**: ML-based anomaly detection for identifying suspicious patterns
- **Venue Economics**: Tournament and multi-bay simulations with detailed analytics
- **Target RTP**: 86% (short), 88% (mid), 90% (long) distance shots

## Key Features

✅ **Zero UI Dependencies** - Pure Rust, CLI, and WASM bindings only
✅ **Production Optimized** - Release builds with LTO and aggressive optimization
✅ **Well Tested** - Comprehensive test suite with benchmarks
✅ **Documented** - Mathematical models and integration guides included

## Building

### CLI Binary
```bash
cargo build --release
./target/release/continuum-golf-simulator --help
```

### WASM Library (for web integration)
```bash
wasm-pack build --target web --out-dir pkg
```

### Run Tests
```bash
cargo test --release
```

### Run Benchmarks
```bash
cargo bench
```

## Integration

This simulator is designed to integrate with real golf simulator hardware (TrackMan, GCQuad, etc.). The WASM bindings allow browser-based integration for in-venue kiosks and mobile apps.

For integration documentation, see `SIMULATOR_INTEGRATION_ROADMAP.md`.

## Mathematical Models

For detailed mathematical documentation, see:
- `MATH_OVERVIEW.md` - Complete mathematical framework
- `WASM_PERFORMANCE_ANALYSIS.md` - Performance characteristics

## License

MIT License - See LICENSE file for details

---

**Note:** This is the production simulator. For the marketing website and demo UI, see `../web-demo/`
