import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const grainGradientMeta: {
    readonly maxColorCount: 7;
};
/**
 * Multi-color gradient with noise & grain over animated abstract shapes
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_softness (0..1): blur between color bands
 * - u_intensity (0..1): distortion between color bands
 * - u_noise (0..1): grainy noise independent of softness
 * - u_shape (float used as integer):
 * ---- 1: single sine wave
 * ---- 2: dots pattern
 * ---- 3: truchet pattern
 * ---- 4: corners (2 rounded rectangles)
 * ---- 5: ripple
 * ---- 6: blob (metaballs)
 * ---- 7: circle imitating 3d look
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 *
 * Note: grains are calculated using gl_FragCoord & u_resolution, meaning grains don't react to scaling and fit
 *
 */
export declare const grainGradientFragmentShader: string;
export interface GrainGradientUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_softness: number;
    u_intensity: number;
    u_noise: number;
    u_shape: (typeof GrainGradientShapes)[GrainGradientShape];
    u_noiseTexture?: HTMLImageElement;
}
export interface GrainGradientParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colors?: string[];
    softness?: number;
    intensity?: number;
    noise?: number;
    shape?: GrainGradientShape;
}
export declare const GrainGradientShapes: {
    wave: number;
    dots: number;
    truchet: number;
    corners: number;
    ripple: number;
    blob: number;
    sphere: number;
};
export type GrainGradientShape = keyof typeof GrainGradientShapes;
