import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * Mimicking water surface distortion with a combination of noises;
 * Can be applied over the texture or just be used as an animated pattern
 *
 * Uniforms:
 * - u_colorBack, u_colorHighlight (RGBA)
 * - u_size: pattern scale relative to the image
 * - u_caustic: power of caustic distortion
 * - u_layering: the power of 2nd layer of caustic distortion
 * - u_edges: caustic distortion power on the image edges
 * - u_waves: additional distortion based in Simplex noise, independent from caustic
 * - u_highlights: a coloring added over the image/background, following the caustic shape
 *
 */
export declare const waterFragmentShader: string;
export interface WaterUniforms extends ShaderSizingUniforms {
    u_image: HTMLImageElement | string | undefined;
    u_colorBack: [number, number, number, number];
    u_colorHighlight: [number, number, number, number];
    u_highlights: number;
    u_layering: number;
    u_edges: number;
    u_caustic: number;
    u_waves: number;
    u_size: number;
}
export interface WaterParams extends ShaderSizingParams, ShaderMotionParams {
    image?: HTMLImageElement | string;
    colorBack?: string;
    colorHighlight?: string;
    highlights?: number;
    layering?: number;
    edges?: number;
    caustic?: number;
    waves?: number;
    size?: number;
}
