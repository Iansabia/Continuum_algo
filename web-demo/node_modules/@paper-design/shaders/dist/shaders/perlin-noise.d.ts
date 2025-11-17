import type { ShaderMotionParams } from '../shader-mount.js';
import { type ShaderSizingParams, type ShaderSizingUniforms } from '../shader-sizing.js';
/**
 * 3d Perlin noise; original algorithm: https://www.shadertoy.com/view/NlSGDz
 *
 * Uniforms:
 * - u_colorBack, u_colorFront (RGBA)
 * - u_proportion: (0..1) blend point between 2 colors (0.5 = equal distribution)
 * - u_softness: color transition sharpness (0 = hard edge, 1 = smooth fade)
 * - u_octaveCount: more octaves => more detailed pattern
 * - u_persistence: roughness, falloff between octaves
 * - u_lacunarity: frequency step, typically around 2, defines how compressed is the pattern
 *
 */
export declare const perlinNoiseFragmentShader: string;
export interface PerlinNoiseUniforms extends ShaderSizingUniforms {
    u_colorFront: [number, number, number, number];
    u_colorBack: [number, number, number, number];
    u_proportion: number;
    u_softness: number;
    u_octaveCount: number;
    u_persistence: number;
    u_lacunarity: number;
}
export interface PerlinNoiseParams extends ShaderSizingParams, ShaderMotionParams {
    colorFront?: string;
    colorBack?: string;
    proportion?: number;
    softness?: number;
    octaveCount?: number;
    persistence?: number;
    lacunarity?: number;
}
