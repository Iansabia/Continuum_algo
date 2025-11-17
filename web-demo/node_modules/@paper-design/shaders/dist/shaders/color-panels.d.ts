import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
export declare const colorPanelsMeta: {
    readonly maxColorCount: 7;
};
/**
 * Pseudo-3D panels rotating around a central axis
 *
 * Uniforms include:
 * - u_colorBack (RGBA)
 * - u_colors (vec4[]), u_colorsCount (float used as integer)
 * - u_density: angle between every 2 panels
 * - u_angle1, u_angle2: skew angle applied to all panes
 * - u_length: panel length (relative to total height)
 * - u_edges: faking edges effect
 * - u_blur: side blur (0 for sharp edges)
 * - u_fadeIn: transparency near central axis
 * - u_fadeOut: transparency near viewer
 * - u_gradient: color mixing within panes (0 = single color, 1 = two colors)
 *
 */
export declare const colorPanelsFragmentShader: string;
export interface ColorPanelsUniforms extends ShaderSizingUniforms {
    u_colors: vec4[];
    u_colorsCount: number;
    u_colorBack: [number, number, number, number];
    u_angle1: number;
    u_angle2: number;
    u_length: number;
    u_edges: boolean;
    u_blur: number;
    u_fadeIn: number;
    u_fadeOut: number;
    u_density: number;
    u_gradient: number;
}
export interface ColorPanelsParams extends ShaderSizingParams, ShaderMotionParams {
    colors?: string[];
    colorBack?: string;
    angle1?: number;
    angle2?: number;
    length?: number;
    edges?: boolean;
    blur?: number;
    fadeIn?: number;
    fadeOut?: number;
    density?: number;
    gradient?: number;
}
