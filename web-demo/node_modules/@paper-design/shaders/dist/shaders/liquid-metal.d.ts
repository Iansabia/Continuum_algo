import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 *
 * Fluid motion imitation applied over user image
 * (animated stripe pattern getting distorted with shape edges)
 *
 * Uniforms:
 * - u_colorBack, u_colorTint (RGBA)
 * - u_repetition: density of pattern stripes
 * - u_softness: blur between stripes
 * - u_shiftRed & u_shiftBlue: color dispersion between the stripes
 * - u_distortion: pattern distortion on the whole canvas
 * - u_contour: distortion power over the shape edges
 * - u_shape (float used as integer):
 * ---- 0: canvas-screen rectangle, needs u_worldWidth = u_worldHeight = 0 to be responsive (see vertex shader)
 * ---- 1: static circle
 * ---- 2: animated flower-like polar shape
 * ---- 3: animated metaballs
 *
 */
export declare const liquidMetalFragmentShader: string;
export declare const POISSON_CONFIG_OPTIMIZED: {
    measurePerformance: boolean;
    workingSize: number;
    iterations: number;
};
export declare function toProcessedLiquidMetal(file: File | string): Promise<{
    imageData: ImageData;
    pngBlob: Blob;
}>;
export interface LiquidMetalUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colorTint: [number, number, number, number];
    u_image: HTMLImageElement | string | undefined;
    u_repetition: number;
    u_shiftRed: number;
    u_shiftBlue: number;
    u_contour: number;
    u_softness: number;
    u_distortion: number;
    u_angle: number;
    u_shape: (typeof LiquidMetalShapes)[LiquidMetalShape];
    u_isImage: boolean;
}
export interface LiquidMetalParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colorTint?: string;
    image?: HTMLImageElement | string | undefined;
    repetition?: number;
    shiftRed?: number;
    shiftBlue?: number;
    contour?: number;
    softness?: number;
    distortion?: number;
    angle?: number;
    shape?: LiquidMetalShape;
}
export declare const LiquidMetalShapes: {
    readonly none: 0;
    readonly circle: 1;
    readonly daisy: 2;
    readonly diamond: 3;
    readonly metaballs: 4;
};
export type LiquidMetalShape = keyof typeof LiquidMetalShapes;
