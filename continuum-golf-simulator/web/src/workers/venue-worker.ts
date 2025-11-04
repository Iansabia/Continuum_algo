// Web Worker for parallel venue simulation
import init, { simulate_single_bay } from '../wasm/continuum_golf_simulator';
import * as Comlink from 'comlink';

// Initialize WASM module once per worker
let wasmInitialized = false;

async function ensureWasmInitialized() {
  if (!wasmInitialized) {
    await init();
    wasmInitialized = true;
    console.log('✅ WASM initialized in worker');
  }
}

export interface BaySimulationRequest {
  bayId: number;
  handicap: number;
  shotsPerBay: number;
  wager: number;
}

export interface BaySimulationResult {
  bay_id: number;
  handicap: number;
  pattern_type: string;
  sigma_x: number;
  sigma_y: number;
  rho: number;
  boundary_points?: Array<[number, number]>;
  total_wagered: number;
  total_won: number;
  net: number;
  rtp: number;
  shots: any[];
}

const workerApi = {
  async simulateBay(request: BaySimulationRequest): Promise<BaySimulationResult> {
    await ensureWasmInitialized();

    console.log(`Worker simulating bay ${request.bayId} (handicap: ${request.handicap})`);

    // Call WASM function to simulate a single bay
    const result = simulate_single_bay(
      request.bayId,
      request.handicap,
      request.shotsPerBay,
      request.wager
    );

    return result as BaySimulationResult;
  },
};

Comlink.expose(workerApi);
