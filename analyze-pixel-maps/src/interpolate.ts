import {
    BarycentricInterpolator,
    LinearAngle, PixelContributionMaps, QuadraticAngle, TangentAngle,
    ValuePerAxisInterpolator
} from "rs-analyze-pixel-maps";

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

/**
 * A very simple interpolator, where only one value per axis is being stored.
 */
export class PixelContribValuePerAxisInterpolator implements PixelContribInterpolator {
    /**
     * An interpolator for each pixel contribution map, where the key is the angle.
     */
    private interpolator: Map<number, ValuePerAxisInterpolator>;

    public readonly name = "ValuePerAxis";

    /**
     * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
     */
    constructor(contrib_maps: PixelContributionMaps) {
        this.interpolator = new Map();

        [...Array(contrib_maps.size()).keys()].map(i => {
            const map = contrib_maps.get_map(i);
            const angle = map.get_description().camera_angle;

            this.interpolator.set(angle, new ValuePerAxisInterpolator(map));
        });
    }

    public interpolate(angle: number, pos: [number, number]): number {
        // try to find the interpolator for the given angle
        const op = this.interpolator.get(angle);
        if (!op) {
            return 0;
        }

        const desc = op.get_descriptor();
        const index = desc.get_index(pos[0], pos[1]);
        const dir = desc.camera_dir_from_index(index);

        return op.interpolate(dir[0], dir[1], dir[2]);
    }
}

/**
 * A very simple interpolator, where only one value per axis is being stored.
 */
export class PixelContribBarycentricInterpolator implements PixelContribInterpolator {
    /**
     * An interpolator for each pixel contribution map, where the key is the angle.
     */
    private interpolator: Map<number, BarycentricInterpolator>;

    public readonly name = "Barycentric";

    /**
     * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
     */
    constructor(contrib_maps: PixelContributionMaps) {
        this.interpolator = new Map();

        [...Array(contrib_maps.size()).keys()].map(i => {
            const map = contrib_maps.get_map(i);
            const angle = map.get_description().camera_angle;

            this.interpolator.set(angle, new BarycentricInterpolator(map));
        });
    }

    public interpolate(angle: number, pos: [number, number]): number {
        // try to find the interpolator for the given angle
        const op = this.interpolator.get(angle);
        if (!op) {
            return 0;
        }

        const desc = op.get_descriptor();
        const index = desc.get_index(pos[0], pos[1]);
        const dir = desc.camera_dir_from_index(index);

        return op.interpolate(dir[0], dir[1], dir[2]);
    }
}