/* tslint:disable */
/* eslint-disable */
export function init_panic_hook(): void;
export function reset_persistent_player(): void;
export function simulate_player_session(handicap: number, num_shots: number, wager_min: number, wager_max: number, hole_id?: number | null, manual_miss_distance?: number | null): any;
export function simulate_venue(num_bays: number, hours: number, shots_per_hour: number, wager_min: number, wager_max: number): any;
/**
 * Enhanced venue simulation with detailed player tracking and random dispersion patterns
 */
export function simulate_venue_enhanced(num_bays: number, shots_per_hour: number, hours_of_operation: number, wager: number): any;
/**
 * Simulate a single bay for parallel processing via Web Workers
 */
export function simulate_single_bay(bay_id: number, handicap: number, shots_per_bay: number, wager: number): any;
export function validate_fairness(hole_id: number): any;
export function get_hole_info(hole_id: number): any;
/**
 * Run anti-cheat analysis on provided shot data without mutating player state
 *
 * This function is stateless and only analyzes the provided shots.
 * Use this for real-time anti-cheat monitoring without corrupting MCMC.
 */
export function analyze_anti_cheat(shots_json: any): any;
export function get_all_holes(): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly reset_persistent_player: () => void;
  readonly simulate_player_session: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
  readonly simulate_venue: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
  readonly simulate_venue_enhanced: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly simulate_single_bay: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly validate_fairness: (a: number) => [number, number, number];
  readonly get_hole_info: (a: number) => [number, number, number];
  readonly analyze_anti_cheat: (a: any) => [number, number, number];
  readonly get_all_holes: () => [number, number, number];
  readonly init_panic_hook: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
