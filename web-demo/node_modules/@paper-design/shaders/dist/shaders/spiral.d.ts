import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * 2-color spiral shape
 *
 * Uniforms:
 * - u_colorBack, u_colorFront (RGBA)
 * - u_density: spacing falloff to simulate radial perspective (0 = no perspective)
 * - u_strokeWidth: thickness of stroke
 * - u_strokeTaper: stroke loosing width further from center (0 for full visibility)
 * - u_distortion: per-arch shift
 * - u_strokeCap: extra width at the center (no effect on u_strokeWidth = 0.5)
 * - u_noiseFrequency, u_noise: simplex noise distortion over the shape
 * - u_softness: color transition sharpness (0 = hard edge, 1 = smooth fade)
 *
 */
export declare const spiralFragmentShader: string;
export interface SpiralUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colorFront: [number, number, number, number];
    u_density: number;
    u_distortion: number;
    u_strokeWidth: number;
    u_strokeTaper: number;
    u_strokeCap: number;
    u_noise: number;
    u_noiseFrequency: number;
    u_softness: number;
}
export interface SpiralParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colorFront?: string;
    density?: number;
    distortion?: number;
    strokeWidth?: number;
    strokeTaper?: number;
    strokeCap?: number;
    noise?: number;
    noiseFrequency?: number;
    softness?: number;
}
