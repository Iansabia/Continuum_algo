import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * Static grid pattern
 *
 * Uniforms:
 * - u_colorBack, u_colorFill, u_colorStroke (vec4 RGBA)
 * - u_dotSize (px): base shape size
 * - u_sizeRange (0..1): randomizes the size of shape between 0 and u_dotSize
 * - u_strokeWidth (px): the stroke (to be added to u_dotSize)
 * - u_gapX, u_gapY (px): pattern spacing
 * - u_opacityRange (0..1): variety of shape opacity
 * - u_shape (float used as integer):
 *   ---- 1: circle
 *   ---- 2: square
 *   ---- 3: triangle
 *   ---- 4: diamond
 *   ---- 5: cross
 *
 */
export declare const dotGridFragmentShader: string;
export interface DotGridUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colorFill: [number, number, number, number];
    u_colorStroke: [number, number, number, number];
    u_dotSize: number;
    u_gapX: number;
    u_gapY: number;
    u_strokeWidth: number;
    u_sizeRange: number;
    u_opacityRange: number;
    u_shape: (typeof DotGridShapes)[DotGridShape];
}
export interface DotGridParams extends ShaderSizingParams {
    colorBack?: string;
    colorFill?: string;
    colorStroke?: string;
    size?: number;
    gapX?: number;
    gapY?: number;
    strokeWidth?: number;
    sizeRange?: number;
    opacityRange?: number;
    shape?: DotGridShape;
}
export declare const DotGridShapes: {
    readonly circle: 0;
    readonly diamond: 1;
    readonly square: 2;
    readonly triangle: 3;
};
export type DotGridShape = keyof typeof DotGridShapes;
