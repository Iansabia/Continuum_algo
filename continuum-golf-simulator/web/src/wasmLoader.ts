import init, { simulate_player_session } from "./wasm/continuum_golf_simulator.js";

let wasmReady = false;

export async function initWasm() {
  if (!wasmReady) {
    await init();
    console.log("✅ WASM module initialized");
    wasmReady = true;
  }
  return { simulate_player_session };
}
