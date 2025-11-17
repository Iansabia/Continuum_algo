# Continuum Golf - Web Demo

**Marketing website and interactive demo interface**

This is the web-based demo and marketing site for Continuum Golf. It showcases the simulator capabilities through an interactive UI but is **NOT** intended for production venue integration.

## What This Is

A React/TypeScript web application featuring:

- 🎯 Interactive landing page with 3D visualizations
- 📊 Live simulator demo with real-time analytics
- 📈 Venue economics dashboard
- 🎨 Marketing materials and product showcase

## What This Is NOT

❌ **Not for production venue integration** - Use `../core/` for real simulator deployments
❌ **Not optimized for scale** - This is a demo/marketing tool
❌ **Mock UI only** - Real integrations should use the core Rust library

## Development

### Prerequisites

- Node.js 18+ and npm
- Rust toolchain with wasm-pack
- wasm-bindgen-cli

### Setup

1. Install dependencies:
```bash
npm install
```

2. Build the WASM module:
```bash
cd ../core
wasm-pack build --target web --out-dir ../web-demo/src/wasm
```

Or use the npm script (runs automatically before build):
```bash
npm run prebuild
```

3. Start development server:
```bash
npm run dev
```

4. Build for production:
```bash
npm run build
```

## Deployment

### GitHub Pages

1. Build the project:
```bash
npm run build
```

2. Deploy to GitHub Pages:
```bash
# From repository root
git subtree push --prefix continuum-golf-simulator/web/dist origin gh-pages
```

### Vercel (Recommended)

The project is configured for Vercel deployment. The root `vercel.json` automatically:
- Builds the WASM module from `../core/`
- Installs npm dependencies
- Builds the React app
- Deploys to production

Simply connect your GitHub repo to Vercel and it will deploy automatically on push.

## Technology Stack

- **Frontend**: React 18 + TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS
- **Charts**: Recharts
- **Backend**: Rust compiled to WebAssembly
- **Performance**: 100% client-side, no server needed

## Core Simulator Integration

This web demo integrates with the core simulator via WASM bindings. The simulator code lives in `../core/` and is compiled to WebAssembly for browser use.

**For production venue integrations**, use the core Rust library directly, not this web interface.

## License

MIT

---

**Note:** This is the demo website. For production simulator integration, see `../core/`
