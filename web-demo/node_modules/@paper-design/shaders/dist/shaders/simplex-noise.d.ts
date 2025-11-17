import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const simplexNoiseMeta: {
    readonly maxColorCount: 10;
};
/**
 * Color Gradient mapped over a combination of 2 Simplex noises
 *
 * Uniforms:
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_stepsPerColor (float, used as int): discrete color steps between u_colors
 * - u_softness: color transition sharpness (0 = hard edge, 1 = smooth fade)
 *
 * */
export declare const simplexNoiseFragmentShader: string;
export interface SimplexNoiseUniforms extends ShaderSizingUniforms {
    u_colors: vec4[];
    u_colorsCount: number;
    u_stepsPerColor: number;
    u_softness: number;
}
export interface SimplexNoiseParams extends ShaderSizingParams, ShaderMotionParams {
    colors?: string[];
    stepsPerColor?: number;
    softness?: number;
}
