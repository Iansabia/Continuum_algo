# WASM Integration Guide

This guide explains how to compile the Rust simulator to WebAssembly and integrate it with the React frontend.

## Prerequisites

- Rust toolchain installed (`rustup`)
- Node.js 18+ and npm
- wasm-pack (will be installed below)

## Step 1: Install wasm-pack

```bash
cargo install wasm-pack
```

Or use the one-liner:
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

## Step 2: Build WASM Module

From the `continuum-golf-simulator` directory:

```bash
wasm-pack build --target web --out-dir web/src/wasm
```

This will:
- Compile Rust code to WebAssembly
- Generate JavaScript bindings
- Create TypeScript definitions
- Output to `web/src/wasm/` directory

Expected output files:
```
web/src/wasm/
├── continuum_golf_simulator.d.ts    # TypeScript definitions
├── continuum_golf_simulator.js      # JavaScript bindings
├── continuum_golf_simulator_bg.wasm # WebAssembly binary
└── package.json                     # NPM package metadata
```

## Step 3: Verify WASM Build

Check that the WASM module exports the expected functions:

```bash
cat web/src/wasm/continuum_golf_simulator.d.ts
```

You should see exports for:
- `simulate_player_session()`
- `simulate_venue()`
- `validate_fairness()`
- `get_hole_info()`
- `get_all_holes()`

## Step 4: Install Web Dependencies

```bash
cd web
npm install
```

## Step 5: Update useSimulator Hook

Replace the placeholder simulation logic in `src/hooks/useSimulator.ts` with WASM calls:

```typescript
import init, { simulate_player_session } from '../wasm/continuum_golf_simulator';

// Initialize WASM on first use
const [wasmReady, setWasmReady] = useState(false);

useEffect(() => {
  init().then(() => {
    setWasmReady(true);
    console.log('WASM module initialized');
  });
}, []);

// In shootOnce function, replace placeholder with:
if (!wasmReady) {
  console.warn('WASM not ready yet');
  return;
}

const result = simulate_player_session(
  handicap,
  1, // single shot
  wager,
  wager,
  null // hole_id (optional)
);
```

## Step 6: Test Locally

Start the development server:

```bash
npm run dev
```

Open browser to `http://localhost:5173` and:
1. Open browser console (F12)
2. Check for "WASM module initialized" message
3. Click "Shoot" button
4. Verify shot appears on target visualizer
5. Check that Kalman filter updates correctly

## Step 7: Build for Production

```bash
npm run build
```

This creates optimized build in `web/dist/` directory.

## Troubleshooting

### WASM module not found
- Ensure `wasm-pack build` completed successfully
- Check that `web/src/wasm/` directory exists
- Verify `vite-plugin-wasm` is in `package.json`

### TypeScript errors
- Run `npm run dev` to trigger Vite's type checking
- Check that `.d.ts` file exists in `web/src/wasm/`

### Runtime errors
- Check browser console for WASM initialization errors
- Ensure `init()` is called before using WASM functions
- Verify WASM binary is being served correctly (Network tab)

### Performance issues
- WASM binary should be < 500KB (gzipped)
- First load may take 1-2 seconds
- Subsequent calls should be < 10ms

## Integration Checklist

- [ ] Install wasm-pack
- [ ] Build WASM module
- [ ] Verify exported functions
- [ ] Install web dependencies
- [ ] Update useSimulator.ts with WASM calls
- [ ] Test locally
- [ ] Build for production
- [ ] Deploy to GitHub Pages

## Advanced: Optimizing WASM Binary

To reduce WASM binary size:

```bash
# Build in release mode with optimizations
wasm-pack build --target web --out-dir web/src/wasm --release

# Use wasm-opt (from binaryen toolkit)
wasm-opt web/src/wasm/continuum_golf_simulator_bg.wasm -O3 -o optimized.wasm
```

Expected size reduction: ~30-40%

## Next Steps

Once WASM is integrated:
1. Remove placeholder simulation logic from `useSimulator.ts`
2. Test with various handicaps and wager amounts
3. Verify Kalman filter convergence
4. Benchmark performance (shots/second)
5. Deploy to production

## Reference

- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [Vite WASM plugin](https://github.com/Menci/vite-plugin-wasm)
- [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/)
