import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const metaballsMeta: {
    readonly maxColorCount: 8;
    readonly maxBallsCount: 20;
};
/**
 * N circular shapes moving around the center, blending and merging based on distance
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_size: ball base size
 * - u_sizeRange (0..1): randomizes the size of balls between 0 and u_size
 * - u_count: number of balls on the canvas
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 *
 */
export declare const metaballsFragmentShader: string;
export interface MetaballsUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_count: number;
    u_size: number;
    u_noiseTexture?: HTMLImageElement;
}
export interface MetaballsParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colors?: string[];
    count?: number;
    size?: number;
}
