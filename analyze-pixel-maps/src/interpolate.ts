import { LinearAngle, PixelContributionMaps, QuadraticAngle, TangentAngle } from "rs-analyze-pixel-maps";

/**
 * An interpolator for pixel contributions. For a given angle and position on the pixel contribution
 * map, the interpolator returns the interpolated value from the pixel contribution maps.
 */
export interface PixelContribInterpolator {
    /**
     * The name of the interpolator.
     */
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
    private interpolator: LinearAngle;

    public readonly name = "Linear";

    /**
     * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
     */
    constructor(contrib_maps: PixelContributionMaps) {
        this.interpolator = new LinearAngle(contrib_maps);
    }

    public interpolate(angle: number, pos: [number, number]): number {
        return this.interpolator.interpolate(angle, pos[0], pos[1]);
    }
}

/**
 * A angle interpolator for pixel contributions that interpolates between the first and
 * last pixel contribution map using the angle.
 * In contrast to the linear interpolator, this interpolator uses the tangent of the angle
 * to interpolate the pixel contributions.
 */
export class AnglePixelContribInterpolator implements PixelContribInterpolator {
    private interpolator: TangentAngle;

    public readonly name = "Angle";

    /**
     * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
     */
    constructor(contrib_maps: PixelContributionMaps) {
        this.interpolator = new TangentAngle(contrib_maps);
    }

    public interpolate(angle: number, pos: [number, number]): number {
        return this.interpolator.interpolate(angle, pos[0], pos[1]);
    }
}

/**
 * A quadratic interpolator for pixel contributions that interpolates using a quadratic polynomial
 * using the first, middle and last pixel contribution map based on the angle as input.
 */
export class QuadraticPixelContribInterpolator implements PixelContribInterpolator {
    private interpolator: QuadraticAngle;

    public readonly name = "Quadratic";

    public constructor(contrib_maps: PixelContributionMaps) {
        const n = contrib_maps.size();

        if (n <= 2) {
            throw new Error("Not enough contribution maps given");
        }

        this.interpolator = new QuadraticAngle(contrib_maps);
    }

    public interpolate(angle: number, pos: [number, number]): number {
        return this.interpolator.interpolate(angle, pos[0], pos[1]);
    }
}