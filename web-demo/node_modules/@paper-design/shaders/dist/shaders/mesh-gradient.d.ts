import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const meshGradientMeta: {
    readonly maxColorCount: 10;
};
/**
 * A composition of N color spots (one per color) with 2 types of
 * distortions applied to the coordinate space
 *
 * Uniforms:
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_distortion: warp distortion
 * - u_swirl: vortex distortion
 * - u_grainMixer: shape distortion
 * - u_grainOverlay: post-processing blending
 */
export declare const meshGradientFragmentShader: string;
export interface MeshGradientUniforms extends ShaderSizingUniforms {
    u_colors: vec4[];
    u_colorsCount: number;
    u_distortion: number;
    u_swirl: number;
    u_grainMixer: number;
    u_grainOverlay: number;
}
export interface MeshGradientParams extends ShaderSizingParams, ShaderMotionParams {
    colors?: string[];
    distortion?: number;
    swirl?: number;
    grainMixer?: number;
    grainOverlay?: number;
}
