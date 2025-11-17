# Continuum Golf - Web Demo

Interactive marketing website and simulator demonstration.

## Purpose

This is a **demo-only** web application built to showcase the Continuum Golf wagering simulator. It is **NOT** intended for production use at golf venues.

**Use this for:**
- Product demonstrations
- Investor presentations
- Understanding simulator capabilities
- Marketing materials

**Do NOT use this for:**
- Production venue deployments
- Real money wagering
- Multi-user systems

## What's Inside

- Landing page with 3D visualizations
- Interactive simulator demo
- Venue economics dashboard
- Live skill tracking charts

## Development

### Prerequisites
- Node.js 18+
- Rust + wasm-pack (for WASM compilation)

### Setup

```bash
# Install dependencies
npm install

# Build WASM module from core
cd ../core
wasm-pack build --target web --out-dir ../web-demo/src/wasm

# Start dev server
cd ../web-demo
npm run dev
```

### Build for Production

```bash
npm run build
```

The build script automatically:
1. Compiles the core Rust simulator to WASM
2. Bundles the React application
3. Outputs to `dist/`

## Deployment

Configured for Vercel with automatic deployments on push to `main`.

**Vercel Settings:**
- Root Directory: `web-demo`
- Build Command: `bash build.sh`
- Output Directory: `dist`

## Tech Stack

- React 18 + TypeScript
- Vite (build tool)
- Tailwind CSS
- Three.js / React Three Fiber
- Recharts (data visualization)
- WASM (Rust simulator)

## License

MIT
