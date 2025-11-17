import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const dotOrbitMeta: {
    readonly maxColorCount: 10;
};
/**
 * Animated dot pattern with dots orbiting around their grid positions
 *
 * Uniforms:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_stepsPerColor: discrete color steps between u_colors
 * - u_size (0..1): dot radius (relative to cell size)
 * - u_sizeRange (0..1): randomizes dot radius between 0 and u_size
 * - u_spreading (0..1): max orbit distance of each dot around the cell center
 *
 * - u_noiseTexture (sampler2D): pre-computed randomizer source
 *
 */
export declare const dotOrbitFragmentShader: string;
export interface DotOrbitUniforms extends ShaderSizingUniforms {
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
    u_size: number;
    u_sizeRange: number;
    u_spreading: number;
    u_stepsPerColor: number;
    u_noiseTexture?: HTMLImageElement;
}
export interface DotOrbitParams extends ShaderSizingParams, ShaderMotionParams {
    colorBack?: string;
    colors?: string[];
    size?: number;
    sizeRange?: number;
    spreading?: number;
    stepsPerColor?: number;
}
