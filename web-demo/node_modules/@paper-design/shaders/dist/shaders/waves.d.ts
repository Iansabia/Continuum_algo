import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * Waveform pattern
 *
 * Uniforms:
 * - u_colorBack, u_colorFront (RGBA)
 * - u_shape (float, fractional numbers allowed):
 *   ---- 0: zigzag
 *   ---- 1: sine wave
 *   ---- 2: irregular wave
 *   ---- 3: irregular wave
 * - u_amplitude, u_frequency, u_spacing: wave settings
 * - u_proportion: (0..1) blend point between 2 colors (0.5 = equal distribution)
 * - u_softness: color transition sharpness (0 = hard edge, 1 = smooth fade)
 *
 * */
export declare const wavesFragmentShader: string;
export interface WavesUniforms extends ShaderSizingUniforms {
    u_colorFront: [number, number, number, number];
    u_colorBack: [number, number, number, number];
    u_shape: number;
    u_frequency: number;
    u_amplitude: number;
    u_spacing: number;
    u_proportion: number;
    u_softness: number;
}
export interface WavesParams extends ShaderSizingParams {
    colorFront?: string;
    colorBack?: string;
    rotation?: number;
    shape?: number;
    frequency?: number;
    amplitude?: number;
    spacing?: number;
    proportion?: number;
    softness?: number;
}
