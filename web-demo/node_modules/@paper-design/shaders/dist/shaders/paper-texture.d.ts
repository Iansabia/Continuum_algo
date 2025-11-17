import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * Mimicking paper texture with a combination of noises
 *
 * Uniforms:
 * - u_colorFront, u_colorBack (RGBA)
 * - u_contrast - mixing front and back colors
 * - u_roughness - pixel noise, related to canvas => not scalable
 * - u_fiber, u_fiberSize - curly shaped noise
 * - u_crumples, u_crumpleSize - cell-based pattern
 * - u_folds, u_foldCount - lines pattern, 15 max
 * - u_drops - metaballs-like pattern
 * - u_seed - applied to folds, crumples and dots
 * - u_fade - big-scale noise mask applied to everything but roughness
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 *
 */
export declare const paperTextureFragmentShader: string;
export interface PaperTextureUniforms extends ShaderSizingUniforms {
    u_image: HTMLImageElement | string | undefined;
    u_noiseTexture?: HTMLImageElement;
    u_colorFront: [number, number, number, number];
    u_colorBack: [number, number, number, number];
    u_contrast: number;
    u_roughness: number;
    u_fiber: number;
    u_fiberSize: number;
    u_crumples: number;
    u_foldCount: number;
    u_folds: number;
    u_fade: number;
    u_crumpleSize: number;
    u_drops: number;
    u_seed: number;
}
export interface PaperTextureParams extends ShaderSizingParams, ShaderMotionParams {
    image?: HTMLImageElement | string;
    colorFront?: string;
    colorBack?: string;
    contrast?: number;
    roughness?: number;
    fiber?: number;
    fiberSize?: number;
    crumples?: number;
    foldCount?: number;
    folds?: number;
    fade?: number;
    crumpleSize?: number;
    drops?: number;
    seed?: number;
}
