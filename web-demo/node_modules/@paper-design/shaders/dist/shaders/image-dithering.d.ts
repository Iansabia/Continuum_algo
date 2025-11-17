import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * Dithering effect over user texture using 3-color palette
 * or colors sampled from original image
 *
 * Uniforms:
 * - u_colorBack, u_colorFront, u_colorHighlight (RGBA)
 *   (u_colorHighlight to be the lightest parts of u_colorFront pixels)
 * - pxSize: px size set relative to canvas resolution
 * - u_type (float used as integer)
 * ---- 1: random dithering
 * ---- 2: 2x2 Bayer matrix
 * ---- 3: 4x4 Bayer matrix
 * ---- 4: 8x8 Bayer matrix
 * - u_originalColors - switching between 3 colors palette and original image colors
 * - u_colorSteps - number of colors to use (applies to both color modes)
 *
 * Note: pixelization is applied to the shapes BEFORE dithering,
 *       meaning pixels don't react to scaling and fit
 */
export declare const imageDitheringFragmentShader: string;
export interface ImageDitheringUniforms extends ShaderSizingUniforms {
    u_image: HTMLImageElement | string;
    u_colorFront: [number, number, number, number];
    u_colorBack: [number, number, number, number];
    u_colorHighlight: [number, number, number, number];
    u_type: (typeof DitheringTypes)[DitheringType];
    u_pxSize: number;
    u_colorSteps: number;
    u_originalColors: boolean;
}
export interface ImageDitheringParams extends ShaderSizingParams, ShaderMotionParams {
    image: HTMLImageElement | string;
    colorFront?: string;
    colorBack?: string;
    colorHighlight?: string;
    type?: DitheringType;
    size?: number;
    colorSteps?: number;
    originalColors?: boolean;
}
export declare const DitheringTypes: {
    readonly random: 1;
    readonly '2x2': 2;
    readonly '4x4': 3;
    readonly '8x8': 4;
};
export type DitheringType = keyof typeof DitheringTypes;
