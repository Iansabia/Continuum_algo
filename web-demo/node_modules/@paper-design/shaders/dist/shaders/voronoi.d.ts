import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const voronoiMeta: {
    readonly maxColorCount: 5;
};
/**
 * Double-pass Voronoi pattern cell edges
 * Original algorithm: https://www.shadertoy.com/view/ldl3W8
 *
 * Uniforms:
 * - u_colorBack, u_colorGlow (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_stepsPerColor: discrete color steps between u_colors
 * - u_distortion (0..0.5): max distance the cell center moves away from regular grid
 * - u_gap: width of the stroke between the cells
 * - u_glow: radial glow around each cell center
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 *
 * Note: gaps can't be removed completely due to artifacts of Voronoi cells
 *
 */
export declare const voronoiFragmentShader: string;
export interface VoronoiUniforms extends ShaderSizingUniforms {
    u_colors: vec4[];
    u_colorsCount: number;
    u_stepsPerColor: number;
    u_colorGap: [number, number, number, number];
    u_colorGlow: [number, number, number, number];
    u_distortion: number;
    u_gap: number;
    u_glow: number;
    u_noiseTexture?: HTMLImageElement;
}
export interface VoronoiParams extends ShaderSizingParams, ShaderMotionParams {
    colors?: string[];
    stepsPerColor?: number;
    colorGap?: string;
    colorGlow?: string;
    distortion?: number;
    gap?: number;
    glow?: number;
}
