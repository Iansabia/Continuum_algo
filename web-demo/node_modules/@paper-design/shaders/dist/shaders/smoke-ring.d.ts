import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const smokeRingMeta: {
    readonly maxColorCount: 10;
    readonly maxNoiseIterations: 8;
};
/**
 * Radial gradient with layered FBM displacement, masked with ring shape
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_thickness, u_radius, u_innerShape: ring mask settings
 * - u_noiseIterations, u_noiseScale: how detailed is the noise (number of fbm layers & noise frequency)
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 */
export declare const smokeRingFragmentShader: string;
export interface SmokeRingUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_noiseScale: number;
    u_thickness: number;
    u_radius: number;
    u_innerShape: number;
    u_noiseIterations: number;
    u_noiseTexture?: HTMLImageElement;
}
export interface SmokeRingParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colors?: string[];
    noiseScale?: number;
    thickness?: number;
    radius?: number;
    innerShape?: number;
    noiseIterations?: number;
}
