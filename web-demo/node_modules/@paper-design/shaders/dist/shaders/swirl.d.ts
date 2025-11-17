import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const swirlMeta: {
    readonly maxColorCount: 10;
};
/**
 * Twisting radial bands
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_bandCount (float, used as int): number of sectors
 * - u_twist: sectors twist intensity (0 = linear)
 * - u_softness: color transition sharpness (0 = hard edge, 1 = smooth fade)
 * - u_noise, u_noiseFrequency: simplex noise distortion over the shape
 *
 */
export declare const swirlFragmentShader: string;
export interface SwirlUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_bandCount: number;
    u_twist: number;
    u_center: number;
    u_proportion: number;
    u_softness: number;
    u_noiseFrequency: number;
    u_noise: number;
}
export interface SwirlParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colors?: string[];
    bandCount?: number;
    twist?: number;
    center?: number;
    proportion?: number;
    softness?: number;
    noiseFrequency?: number;
    noise?: number;
}
