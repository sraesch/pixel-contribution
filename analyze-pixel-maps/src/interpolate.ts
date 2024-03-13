import { mat2, vec2 } from "gl-matrix";
import { PixelContributionMap } from "./pixel_contrib";

export interface PixelContribInterpolator {
    readonly name: string;

    /**
     * @param angle - The angle to interpolate.
     * @param pos - The position on the pixel contribution map.
     * 
     * @returns the interpolated value from the pixel contribution maps at the given position.
     */
    interpolate(angle: number, pos: [number, number]): number;
}

/**
 * A simple linear interpolator for pixel contributions that interpolates between the first and
 * last pixel contribution map using the angle.
 */
export class LinearPixelContribInterpolator implements PixelContribInterpolator {
    private first_map: PixelContributionMap;
    private last_map: PixelContributionMap;

    public readonly name = "Linear";


    /**
     * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
     */
    constructor(contrib_maps: PixelContributionMap[]) {
        this.first_map = contrib_maps[0];
        this.last_map = contrib_maps[contrib_maps.length - 1];
    }

    public interpolate(angle: number, pos: [number, number]): number {
        const first_map = this.first_map;
        const last_map = this.last_map;

        const first_angle = first_map.descriptor.camera_angle;
        const last_angle = last_map.descriptor.camera_angle;

        const index = pos[1] * first_map.descriptor.map_size + pos[0];

        const first_value = first_map.pixel_contrib[index];
        const last_value = last_map.pixel_contrib[index];

        const f = (angle - first_angle) / (last_angle - first_angle);
        return first_value * (1 - f) + last_value * f;
    }
}

/**
 * A angle interpolator for pixel contributions that interpolates between the first and
 * last pixel contribution map using the angle.
 * In contrast to the linear interpolator, this interpolator uses the tangens of the angle
 * to interpolate the pixel contributions.
 */
export class AnglePixelContribInterpolator implements PixelContribInterpolator {
    private first_map: PixelContributionMap;
    private last_map: PixelContributionMap;

    public readonly name = "Angle";


    /**
     * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
     */
    constructor(contrib_maps: PixelContributionMap[]) {
        this.first_map = contrib_maps[0];
        this.last_map = contrib_maps[contrib_maps.length - 1];
    }

    public interpolate(angle: number, pos: [number, number]): number {
        const first_map = this.first_map;
        const last_map = this.last_map;

        const first_angle = first_map.descriptor.camera_angle;
        const last_angle = last_map.descriptor.camera_angle;

        const index = pos[1] * first_map.descriptor.map_size + pos[0];

        const first_value = first_map.pixel_contrib[index];
        const last_value = last_map.pixel_contrib[index];

        const a_start = Math.tan(first_angle / 2.0);
        const a_last = Math.tan(last_angle / 2.0);

        const a = Math.tan(angle / 2.0);
        const t = (a - a_start) / (a_last - a_start);

        return first_value * (1 - t) + last_value * t;
    }
}

/**
 * A quadratic interpolator for pixel contributions that interpolates using a quadratic polynomial
 * using the first, middle and last pixel contribution map based on the angle as input.
 */
export class QuadraticPixelContribInterpolator implements PixelContribInterpolator {
    private first_map: PixelContributionMap;
    private middle_map: PixelContributionMap;
    private last_map: PixelContributionMap;

    /**
     * The matrix A is used for determining the coefficients of the quadratic polynomial.
     */
    private A: mat2;

    public readonly name = "Quadratic";

    public constructor(contrib_maps: PixelContributionMap[]) {
        if (contrib_maps.length <= 3) {
            throw new Error("Not enough contribution maps given");
        }

        this.first_map = contrib_maps[0];
        this.last_map = contrib_maps[contrib_maps.length - 1];
        this.middle_map = contrib_maps[Math.floor(contrib_maps.length / 2)];

        const x0 = this.first_map.descriptor.camera_angle;
        const x1 = this.middle_map.descriptor.camera_angle;
        const x2 = this.last_map.descriptor.camera_angle;

        console.assert(x0 === 0, "The first angle must be 0");
        console.assert(x0 < x1 && x1 < x2, "The angles are not in ascending order");

        this.A = mat2.invert(mat2.create(), mat2.fromValues(x1 * x1, x2 * x2, x1, x2));
    }

    public interpolate(angle: number, pos: [number, number]): number {
        const first_map = this.first_map;
        const middle_map = this.middle_map;
        const last_map = this.last_map;

        const index = pos[1] * first_map.descriptor.map_size + pos[0];

        const y0 = first_map.pixel_contrib[index];
        const y1 = middle_map.pixel_contrib[index];
        const y2 = last_map.pixel_contrib[index];

        // determine the polynomial coefficients a,b,c
        const c = y0; // as x0 = 0
        const rhs = vec2.fromValues(y1 - c, y2 - c);
        const coeffs = vec2.transformMat2(vec2.create(), rhs, this.A);
        const a = coeffs[0];
        const b = coeffs[1];

        // evaluate the quadratic polynomial
        return a * angle * angle + b * angle + c;
    }
}