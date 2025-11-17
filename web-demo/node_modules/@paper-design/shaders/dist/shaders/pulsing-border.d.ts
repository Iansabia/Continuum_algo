import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const pulsingBorderMeta: {
    readonly maxColorCount: 5;
    readonly maxSpots: 4;
};
/**
 * Color spots traveling around rectangular stroke (border)
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_roundness, u_thickness, u_softness: border parameters
 * - u_aspectRatio
 * - u_intensity: thickness of individual spots
 * - u_bloom: normal / additive color blending
 * - u_spotSize: angular size of spots
 * - u_spots (float used as int): number of spots rendered per color
 * - u_pulse: optional pulsing animation
 * - u_smoke, u_smokeSize: optional noisy shapes around the border
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 *
 */
export declare const pulsingBorderFragmentShader: string;
export interface PulsingBorderUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_roundness: number;
    u_thickness: number;
    u_marginLeft: number;
    u_marginRight: number;
    u_marginTop: number;
    u_marginBottom: number;
    u_aspectRatio: (typeof PulsingBorderAspectRatios)[PulsingBorderAspectRatio];
    u_softness: number;
    u_intensity: number;
    u_bloom: number;
    u_spots: number;
    u_spotSize: number;
    u_pulse: number;
    u_smoke: number;
    u_smokeSize: number;
    u_noiseTexture?: HTMLImageElement;
}
export interface PulsingBorderParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colors?: string[];
    roundness?: number;
    thickness?: number;
    margin?: number;
    marginLeft?: number;
    marginRight?: number;
    marginTop?: number;
    marginBottom?: number;
    aspectRatio?: PulsingBorderAspectRatio;
    softness?: number;
    intensity?: number;
    bloom?: number;
    spots?: number;
    spotSize?: number;
    pulse?: number;
    smoke?: number;
    smokeSize?: number;
}
export declare const PulsingBorderAspectRatios: {
    readonly auto: 0;
    readonly square: 1;
};
export type PulsingBorderAspectRatio = keyof typeof PulsingBorderAspectRatios;
