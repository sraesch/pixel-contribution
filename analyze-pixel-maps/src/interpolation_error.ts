import { PixelContribInterpolator } from "./interpolate";
import { PixelContribMapDescriptor, PixelContributionMap } from "./pixel_contrib";

/**
 * Uses the interpolation operator to determine the interpolation error for each pixel contribution map.
 * The resulting
 *  
 * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
 *
 * @returns 
 */
export function determine_interpolation_error<I extends PixelContribInterpolator>(i: I, contrib_maps: PixelContributionMap[]): PixelContributionMap[] {
    return contrib_maps.map(contrib => {
        const descriptor: PixelContribMapDescriptor = {
            map_size: contrib.descriptor.map_size,
            camera_angle: contrib.descriptor.camera_angle
        };

        const pixel_contrib = new Float32Array(contrib.descriptor.map_size * contrib.descriptor.map_size);
        for (let y = 0; y < contrib.descriptor.map_size; y++) {
            for (let x = 0; x < contrib.descriptor.map_size; x++) {
                const index = y * contrib.descriptor.map_size + x;
                pixel_contrib[index] = Math.abs(i.interpolate(contrib.descriptor.camera_angle, [x, y]) - contrib.pixel_contrib[index]);
            }
        }

        return new PixelContributionMap(descriptor, pixel_contrib);
    });
}