import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const warpMeta: {
    readonly maxColorCount: 10;
};
/**
 * Iterative layered + swirl-based distortion applied over different layouts (shapes)
 *
 * Uniforms:
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_proportion: (0..1) blend point between 2 colors (0.5 = equal distribution)
 * - u_softness: color transition sharpness (0 = hard edge, 1 = smooth fade)
 * - u_shape (float used as integer):
 * ---- 0: checks
 * ---- 1: stripes
 * ---- 2: 2x halves of canvas (mapping the canvas height regardless of resolution)
 * - u_shapeScale: the scale of layouts (underlying shapes)
 * - u_distortion: value noise distortion over the UV coordinate
 * - u_swirl, u_swirlIterations: swirly distortion (layering curves effect)
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 *
 */
export declare const warpFragmentShader: string;
export interface WarpUniforms extends ShaderSizingUniforms {
    u_colors: vec4[];
    u_colorsCount: number;
    u_proportion: number;
    u_softness: number;
    u_shape: (typeof WarpPatterns)[WarpPattern];
    u_shapeScale: number;
    u_distortion: number;
    u_swirl: number;
    u_swirlIterations: number;
    u_noiseTexture?: HTMLImageElement;
}
export interface WarpParams extends ShaderSizingParams, ShaderMotionParams {
    colors?: string[];
    rotation?: number;
    proportion?: number;
    softness?: number;
    shape?: WarpPattern;
    shapeScale?: number;
    distortion?: number;
    swirl?: number;
    swirlIterations?: number;
}
export declare const WarpPatterns: {
    readonly checks: 0;
    readonly stripes: 1;
    readonly edge: 2;
};
export type WarpPattern = keyof typeof WarpPatterns;
