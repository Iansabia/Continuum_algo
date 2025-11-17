# WASM vs CLI Performance Analysis

## Problem Summary

**CLI Simulation**: Utilizes all CPU cores, runs very fast
**WASM Simulation**: Runs single-threaded, very slow, doesn't use CPU power

## Root Cause Analysis

### CLI Implementation (Fast - Multi-threaded)

Location: `src/simulators/venue.rs:152`

```rust
// Run sessions in parallel for each bay
let bay_results: Vec<_> = players
    .into_par_iter()  // ⚡ PARALLEL ITERATION using Rayon
    .map(|mut player| {
        // ... simulation logic for each bay
    })
    .collect();
```

**Key Points:**
- Uses `rayon` crate's `into_par_iter()` for parallel processing
- Each bay simulation runs on a separate CPU thread
- Fully utilizes multi-core CPUs (M-series chips have 8-16 cores)
- For 20 bays, can run ~8-16 bays simultaneously on modern hardware

### WASM Implementation (Slow - Single-threaded)

Location: `src/wasm.rs:366`

```rust
// Simulate each bay with unique random dispersion pattern
for (bay_idx, mut player) in players.into_iter().enumerate() {
    // ⚠️ SEQUENTIAL ITERATION - processes one bay at a time
    // ... simulation logic for each bay
}
```

**Key Points:**
- Uses regular `for` loop with sequential iteration
- Processes bays one at a time
- Only uses a single JavaScript execution thread
- No parallelization whatsoever

## Why WASM Can't Use Rayon Directly

### WebAssembly Threading Limitations

1. **No Native Threads**: WebAssembly doesn't have native thread support like native code
2. **SharedArrayBuffer Required**: Multi-threading in WASM requires SharedArrayBuffer
3. **Browser Restrictions**: Many browsers have disabled SharedArrayBuffer due to Spectre/Meltdown vulnerabilities
4. **Rayon Incompatibility**: Rayon relies on OS-level threads which don't exist in WASM environment

### Current WASM Thread Support Status

```
✅ Chrome/Edge: Supports with special headers
⚠️ Firefox: Supports but disabled by default
❌ Safari: Limited/experimental support
```

## Performance Impact

### Scenario: 20 bays × 100 shots = 2,000 total shots

**CLI (Native - Parallel)**
- Uses 8-10 CPU cores simultaneously
- Processes ~8-10 bays at once
- **Estimated time**: ~2-3 seconds

**WASM (Browser - Sequential)**
- Uses 1 JavaScript thread
- Processes 1 bay at a time
- **Estimated time**: ~20-30 seconds (10x slower)

## Solutions

### Option 1: Web Workers (Recommended)

**Approach**: Manually implement parallelization using Web Workers

**Pros:**
✅ Works in all modern browsers
✅ No special headers required
✅ Can parallelize bay simulations
✅ Good browser compatibility

**Cons:**
❌ More complex implementation
❌ Requires message passing between workers
❌ Each worker needs its own WASM instance

**Implementation Strategy:**
1. Split bay simulations into chunks
2. Create Web Worker pool (4-8 workers)
3. Each worker gets its own WASM module instance
4. Distribute bays across workers
5. Aggregate results when all workers complete

### Option 2: Chunked Processing with Progress Updates

**Approach**: Process bays in small chunks with UI updates

**Pros:**
✅ Simple to implement
✅ Provides progress feedback
✅ Prevents UI freezing
✅ Works everywhere

**Cons:**
❌ Still sequential (not truly parallel)
❌ Slightly slower than full parallelization
❌ More overhead from yielding to browser

**Implementation Strategy:**
```rust
#[wasm_bindgen]
pub async fn simulate_venue_chunked(
    num_bays: usize,
    chunk_size: usize,
    progress_callback: js_sys::Function
) -> Result<JsValue, JsValue> {
    for chunk in bays.chunks(chunk_size) {
        // Process chunk
        // Report progress
        yield_to_browser().await;
    }
}
```

### Option 3: WASM Threads (Experimental)

**Approach**: Use wasm-bindgen-rayon for native-like threading

**Pros:**
✅ True parallelization
✅ Can reuse existing Rayon code
✅ Best performance when available

**Cons:**
❌ Requires SharedArrayBuffer
❌ Needs special HTTP headers
❌ Limited browser support
❌ Complex deployment

**Required Headers:**
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

## Recommended Implementation

### Short-term: Web Workers (Option 1)

1. Create a Web Worker pool manager
2. Instantiate WASM in each worker
3. Distribute bay simulations across workers
4. Use comlink or postMessage for communication

### Code Structure:
```
web/
  src/
    workers/
      venue-simulation-worker.ts  # Individual worker
      worker-pool.ts              # Pool manager
    components/
      VenueSimulator.tsx          # Updated to use workers
```

### Long-term: Hybrid Approach

1. Detect browser capabilities
2. Use WASM threads if available
3. Fall back to Web Workers
4. Show performance recommendations to users

## Estimated Performance Gains

| Approach | Speed Improvement | Browser Support | Complexity |
|----------|------------------|-----------------|------------|
| Current (Sequential) | 1x (baseline) | 100% | Low |
| Web Workers (4-8) | 4-8x | 99%+ | Medium |
| Chunked Processing | 1.2-1.5x | 100% | Low |
| WASM Threads | 8-12x | 30-40% | High |

## Conclusion

**The CLI is fast because it uses Rayon's parallel iterators (`into_par_iter()`) to run multiple bay simulations simultaneously across CPU cores.**

**The WASM version is slow because it processes bays sequentially in a single JavaScript thread.**

**Best solution: Implement Web Workers to achieve 4-8x speedup with excellent browser compatibility.**
