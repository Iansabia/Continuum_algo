import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 *
 * Fractal-like structure made of several layers of sine arches
 * Original algorithm: https://x.com/zozuar/status/1625182758745128981/
 *
 * Uniforms:
 * - u_colorBack, u_colorMid, u_colorFront (RGBA)
 * - u_brightness, u_contrast
 *
 */
export declare const neuroNoiseFragmentShader: string;
export interface NeuroNoiseUniforms extends ShaderSizingUniforms {
    u_colorFront: [number, number, number, number];
    u_colorMid: [number, number, number, number];
    u_colorBack: [number, number, number, number];
    u_brightness: number;
    u_contrast: number;
}
export interface NeuroNoiseParams extends ShaderSizingParams, ShaderMotionParams {
    colorFront?: string;
    colorMid?: string;
    colorBack?: string;
    brightness?: number;
    contrast?: number;
}
