import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * 2-color dithering effect over animated abstract shapes
 *
 * Uniforms:
 * - u_colorBack, u_colorFront (RGBA)
 * - pxSize: px size relative to canvas resolution
 * - u_shape (float used as integer):
 * ---- 1: simplex noise pattern
 * ---- 2: warp noise pattern
 * ---- 3: columns of dots moving vertically
 * ---- 4: sine wave
 * ---- 5: ripple effect
 * ---- 6: swirl animation
 * ---- 7: rotating sphere
 *  - u_type (float used as integer)
 * ---- 1: random dithering
 * ---- 2: 2x2 Bayer matrix
 * ---- 3: 4x4 Bayer matrix
 * ---- 4: 8x8 Bayer matrix
 *
 * Note: pixelization is applied to the shapes BEFORE dithering, meaning pixels don't react to scaling and fit
 */
export declare const ditheringFragmentShader: string;
export interface DitheringUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colorFront: [number, number, number, number];
    u_shape: (typeof DitheringShapes)[DitheringShape];
    u_type: (typeof DitheringTypes)[DitheringType];
    u_pxSize: number;
}
export interface DitheringParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colorFront?: string;
    shape?: DitheringShape;
    type?: DitheringType;
    size?: number;
}
export declare const DitheringShapes: {
    readonly simplex: 1;
    readonly warp: 2;
    readonly dots: 3;
    readonly wave: 4;
    readonly ripple: 5;
    readonly swirl: 6;
    readonly sphere: 7;
};
export type DitheringShape = keyof typeof DitheringShapes;
export declare const DitheringTypes: {
    readonly random: 1;
    readonly '2x2': 2;
    readonly '4x4': 3;
    readonly '8x8': 4;
};
export type DitheringType = keyof typeof DitheringTypes;
