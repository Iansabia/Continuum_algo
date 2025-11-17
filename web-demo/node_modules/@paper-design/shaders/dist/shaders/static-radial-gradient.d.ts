import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const staticRadialGradientMeta: {
    readonly maxColorCount: 10;
};
/**
 * N-colors radial gradient
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_radius - circle radius
 * - u_focalDistance, u_focalAngle (float) - gradient center offset to the circle center
 * - u_falloff (-1 .. 1, float) - color points distribution (0 for linear gradient)
 * - u_mixing (0 .. 1, float) - 0 for stepped gradient, 0.5 for smooth transitions, 1 for pronounced color points
 * - u_distortion, u_distortionShift, u_distortionFreq - radial distortion (effective with u_distortion > 0)
 * - u_grainMixer - shape distortion
 * - u_grainOverlay - post-processing blending
 */
export declare const staticRadialGradientFragmentShader: string;
export interface StaticRadialGradientUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_radius: number;
    u_focalDistance: number;
    u_focalAngle: number;
    u_falloff: number;
    u_mixing: number;
    u_distortion: number;
    u_distortionShift: number;
    u_distortionFreq: number;
    u_grainMixer: number;
    u_grainOverlay: number;
}
export interface StaticRadialGradientParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colors?: string[];
    radius?: number;
    focalDistance?: number;
    focalAngle?: number;
    falloff?: number;
    mixing?: number;
    distortion?: number;
    distortionShift?: number;
    distortionFreq?: number;
    grainMixer?: number;
    grainOverlay?: number;
}
