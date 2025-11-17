# Continuum Golf

**Fair, skill-adaptive golf wagering simulator for commercial golf simulators**

This repository contains two distinct projects:

## 📁 Project Structure

```
Continuum_algo/
├── core/           # Production simulator (for venue integration)
└── web-demo/       # Marketing website and demo UI
```

### 🎯 [core/](./core/) - Production Simulator

**For real-world golf simulator integration**

The core Continuum algorithm - a high-performance Rust-based golf wagering simulator designed for integration with physical golf simulators (TrackMan, GCQuad, etc.).

**Key Features:**
- Proprietary odds engine with dynamic P_max calculations
- MCMC Bayesian inference for adaptive skill modeling
- ML-based anti-cheat detection
- Venue economics and tournament simulations
- Target RTP: 86% (short), 88% (mid), 90% (long)

**Use this for:**
- Production venue deployments
- Integration with golf simulator hardware
- Building real wagering systems

[**→ See core/README.md for details**](./core/README.md)

---

### 🌐 [web-demo/](./web-demo/) - Marketing Website

**For demonstrations and marketing**

React/TypeScript web application showcasing the simulator through an interactive UI. Includes landing page, 3D visualizations, and live demo dashboard.

**Use this for:**
- Marketing and product demonstrations
- Understanding simulator capabilities
- Interactive visualizations

**NOT for:**
- Production venue integration
- Real wagering systems
- Scale deployments

[**→ See web-demo/README.md for details**](./web-demo/README.md)

---

## 🚀 Quick Start

### Core Simulator
```bash
cd core
cargo build --release
./target/release/continuum-golf-simulator --help
```

### Web Demo
```bash
cd web-demo
npm install
npm run dev
```

## 📊 Architecture

The core simulator is written in pure Rust and compiled to:
1. **Native binary** - CLI for testing and simulations
2. **WASM module** - Browser integration for web-demo and venue kiosks

The web-demo uses the WASM module to provide real-time, client-side simulations.

## 🔧 For Golf Simulator Operators

If you're looking to integrate Continuum Golf wagering with your physical golf simulator:

1. **Start with [core/](./core/)** - This contains the production simulator
2. Review [core/SIMULATOR_INTEGRATION_ROADMAP.md](./core/SIMULATOR_INTEGRATION_ROADMAP.md)
3. Review [core/MATH_OVERVIEW.md](./core/MATH_OVERVIEW.md) for the mathematical framework

The `web-demo/` is for marketing purposes only and should not be used for production deployments.

## 📄 License

MIT License - See LICENSE file for details

## 👨‍💻 Author

Ian Sabia

---

**Need help?** Check the README files in each subdirectory for detailed documentation.
