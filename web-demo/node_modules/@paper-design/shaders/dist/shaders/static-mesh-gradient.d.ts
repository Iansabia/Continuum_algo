import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const staticMeshGradientMeta: {
    readonly maxColorCount: 10;
};
/**
 * A composition of N color spots (one per color)
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_waveX, u_waveY - power of sine wave distortion along X and Y axes
 * - u_waveXShift, u_waveYShift - each wave phase offset
 * - u_mixing (0 .. 1, float) - 0 for stepped gradient, 0.5 for smooth transitions, 1 for pronounced color points
 * - u_grainMixer - shape distortion
 * - u_grainOverlay - post-processing blending
 */
export declare const staticMeshGradientFragmentShader: string;
export interface StaticMeshGradientUniforms extends ShaderSizingUniforms {
    u_colors: vec4[];
    u_colorsCount: number;
    u_positions: number;
    u_waveX: number;
    u_waveXShift: number;
    u_waveY: number;
    u_waveYShift: number;
    u_mixing: number;
    u_grainMixer: number;
    u_grainOverlay: number;
}
export interface StaticMeshGradientParams extends ShaderSizingParams, ShaderMotionParams {
    colors?: string[];
    positions?: number;
    waveX?: number;
    waveXShift?: number;
    waveY?: number;
    waveYShift?: number;
    mixing?: number;
    grainMixer?: number;
    grainOverlay?: number;
}
