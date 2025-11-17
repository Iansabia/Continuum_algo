import type { vec4 } from '../types.js';
import type { ShaderMotionParams } from '../shader-mount.js';
import type { ShaderSizingParams, ShaderSizingUniforms } from '../shader-sizing.js';
export declare const heatmapMeta: {
    readonly maxColorCount: 10;
};
export declare const heatmapFragmentShader: string;
export declare function toProcessedHeatmap(file: File | string): Promise<{
    blob: Blob;
}>;
export interface HeatmapUniforms extends ShaderSizingUniforms {
    u_image: HTMLImageElement | string;
    u_contour: number;
    u_angle: number;
    u_noise: number;
    u_innerGlow: number;
    u_outerGlow: number;
    u_colorBack: [number, number, number, number];
    u_colors: vec4[];
    u_colorsCount: number;
}
export interface HeatmapParams extends ShaderSizingParams, ShaderMotionParams {
    image: HTMLImageElement | string;
    contour?: number;
    angle?: number;
    noise?: number;
    innerGlow?: number;
    outerGlow?: number;
    colorBack?: string;
    colors?: string[];
}
