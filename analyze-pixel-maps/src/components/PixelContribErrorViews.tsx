import { useEffect, useState } from "react";
import { PixelContribInterpolator } from "../interpolate";
import { PixelContributionMap } from "../pixel_contrib";
import { determine_interpolation_error } from "../interpolation_error";
import { PixelContribViews } from "./PixelContribViews";

export interface PixelContribErrorViewsProps {
    contrib_maps: PixelContributionMap[];
    interpolator: PixelContribInterpolator;
    scale?: number;
    onSelectError?: (error: number) => void;
}

export function PixelContribErrorViews(props: PixelContribErrorViewsProps): JSX.Element {
    const { contrib_maps, interpolator } = props;

    const scale = props.scale || 1.0;

    const [errorMaps, setErrorMaps] = useState<PixelContributionMap[]>([]);

    useEffect(() => {
        setErrorMaps(determine_interpolation_error(interpolator, contrib_maps));

    }, [contrib_maps, interpolator]);

    const handleSelectError = (pos_x: number, pos_y: number, angle: number) => {
        if (props.onSelectError) {
            const error_map = errorMaps.find(contrib => contrib.descriptor.camera_angle === angle);
            if (!error_map) {
                return;
            }

            const index = pos_y * error_map.descriptor.map_size + pos_x;
            const error = error_map.pixel_contrib[index];

            props.onSelectError(error);
        }
    };

    return (
        <div>
            <h1>Pixel Contribution Error Views {interpolator.name}</h1>
            <PixelContribViews onSelectPixelContribSample={handleSelectError} scale={scale} pixelContribMaps={errorMaps} />
        </div>
    );
}