import { useEffect, useState } from "react";
import { PixelContribInterpolator } from "../interpolate";
import { determine_interpolation_error } from "../interpolation_error";
import { PixelContribViews } from "./PixelContribViews";
import { PixelContributionMaps } from "rs-analyze-pixel-maps";

export interface PixelContribErrorViewsProps {
    contrib_maps: PixelContributionMaps;
    interpolator: PixelContribInterpolator;
    scale?: number;
    onSelectError?: (error: number) => void;
}

export function PixelContribErrorViews(props: PixelContribErrorViewsProps): JSX.Element {
    const { contrib_maps, interpolator } = props;

    const scale = props.scale || 1.0;

    const [errorMaps, setErrorMaps] = useState<PixelContributionMaps>(new PixelContributionMaps());

    useEffect(() => {
        setErrorMaps(determine_interpolation_error(interpolator, contrib_maps));

    }, [contrib_maps, interpolator]);

    const handleSelectError = (pos_x: number, pos_y: number, angle: number) => {
        if (props.onSelectError) {
            let error_map_index = -1;
            for (let i = 0; i < errorMaps.size(); i++) {
                if (errorMaps.get_map_descriptor(i).camera_angle === angle) {
                    error_map_index = i;
                    break;
                }
            }

            if (error_map_index < 0) {
                return;
            }

            const error = errorMaps.get_value(error_map_index, pos_x, pos_y);

            props.onSelectError(error);
        }
    };

    return (
        <div>
            <h3>Pixel Contribution Error Views {interpolator.name}</h3>
            <PixelContribViews onSelectPixelContribSample={handleSelectError} scale={scale} pixelContribMaps={errorMaps} />
        </div>
    );
}