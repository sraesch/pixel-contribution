import { PixelContributionMap, PixelContributionMaps } from "rs-analyze-pixel-maps";
import { PixelContribInterpolator } from "./interpolate";

/**
 * Uses the interpolation operator to determine the interpolation error for each pixel contribution map.
 * The resulting
 *  
 * @param contrib_maps - The pixel contribution maps from which to define the interpolation.
 *
 * @returns 
 */
export function determine_interpolation_error<I extends PixelContribInterpolator>(interpolator: I, contrib_maps: PixelContributionMaps): PixelContributionMaps {
    const result = new PixelContributionMaps();

    const n = contrib_maps.size();
    for (let i = 0; i < n; i++) {
        const contrib = contrib_maps.get_map(i);
        const descriptor = contrib.get_description();

        const pixel_contrib = new Float32Array(descriptor.map_size * descriptor.map_size);
        for (let y = 0; y < descriptor.map_size; y++) {
            for (let x = 0; x < descriptor.map_size; x++) {
                const index = y * descriptor.map_size + x;
                pixel_contrib[index] = Math.abs(interpolator.interpolate(descriptor.camera_angle, [x, y]) - contrib.get_value_at_index(index));
            }
        }

        result.add_map(new PixelContributionMap(descriptor, pixel_contrib));
    }

    return result;
}