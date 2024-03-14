import { PixelContribView } from "./PixelContribView";
import { PixelContributionMaps } from "rs-analyze-pixel-maps";


export interface PixelContribViewsProps {
    pixelContribMaps: PixelContributionMaps;
    onSelectPixelContribSample?: (pos_x: number, pos_y: number, angle: number) => void;
    scale?: number;
}

export function PixelContribViews(props: PixelContribViewsProps): JSX.Element {
    const { pixelContribMaps, onSelectPixelContribSample } = props;

    const scale = props.scale || 1.0;

    const handleSelectPixelContribSample = (pos_x: number, pos_y: number, angle: number) => {
        if (onSelectPixelContribSample) {
            onSelectPixelContribSample(pos_x, pos_y, angle);
        }
    };

    return (
        <div style={{
            display: "flex",
            flexDirection: "row",
            flexWrap: "wrap",
            justifyContent: "start",
            width: "100%",
            height: "100%",
        }}>
            {
                [...Array(pixelContribMaps.size()).keys()].map(i => {
                    const contrib = pixelContribMaps.get_map(i);
                    const descriptor = contrib.get_description();
                    // convert angle from radians to degrees and round it
                    const angle = Math.round(descriptor.camera_angle * 180 / Math.PI);

                    return (
                        <div key={i} style={{
                            margin: "1em",
                        }}>
                            <h3>Camera Angle: {angle}</h3>
                            <PixelContribView scale={scale} pixelContrib={contrib} onSelectPixelContribSample={handleSelectPixelContribSample} />
                        </div>
                    );
                })
            }
        </div >
    )
}