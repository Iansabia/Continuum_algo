# Continuum Golf - Web Demo

Interactive web demonstration of the Continuum Golf wagering simulator, built with React + TypeScript + WebAssembly.

## Features

- **Player Session Simulator**: Run virtual player sessions with configurable handicaps and wagers
- **Venue Economics Dashboard**: Simulate venue-wide operations with multiple bays
- **Fairness Validator**: Interactive proof showing equal expected value across all skill levels
- **Real-time Charts**: Visualize cumulative P/L, hourly profits, and skill evolution
- **Powered by Rust**: All simulations run in WASM for blazing-fast performance

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

2. Build the WASM module (from parent directory):
```bash
cd ..
wasm-pack build --target web --out-dir web/src/wasm
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

### Vercel

1. Connect your GitHub repository to Vercel
2. Set build command: `cd continuum-golf-simulator/web && npm run build`
3. Set output directory: `continuum-golf-simulator/web/dist`
4. Deploy automatically on push

## Technology Stack

- **Frontend**: React 18 + TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS
- **Charts**: Recharts
- **Backend**: Rust compiled to WebAssembly
- **Performance**: 100% client-side, no server needed

## License

MIT
