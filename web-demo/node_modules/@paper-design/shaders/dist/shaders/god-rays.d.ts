import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const godRaysMeta: {
    readonly maxColorCount: 5;
};
/**
 * Radial rays animated from center
 *
 * Uniforms:
 * - u_colorBack, u_colorBloom (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_density: frequency of sector shapes
 * - u_intensity: rays visibility within sectors
 * - u_spotty: density of spots on the ray (higher = more spots)
 * - u_midSize, u_midIntensity: central shape over the rays
 * - u_bloom (0..1): normal to additive blending mix
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 */
export declare const godRaysFragmentShader: string;
export interface GodRaysUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colorBloom: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_spotty: number;
    u_midSize: number;
    u_midIntensity: number;
    u_density: number;
    u_intensity: number;
    u_bloom: number;
    u_noiseTexture?: HTMLImageElement;
}
export interface GodRaysParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colorBloom?: string;
    colors?: string[];
    spotty?: number;
    midSize?: number;
    midIntensity?: number;
    density?: number;
    intensity?: number;
    bloom?: number;
}
