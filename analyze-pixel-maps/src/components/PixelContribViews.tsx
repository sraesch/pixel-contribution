import { PixelContributionMap } from "../pixel_contrib";
import { PixelContribView } from "./PixelContribView";

export interface PixelContribViewsProps {
    pixelContribMaps: PixelContributionMap[];
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
                pixelContribMaps.map((contrib, i) => {
                    // convert angle from radians to degrees and round it
                    const angle = Math.round(contrib.descriptor.camera_angle * 180 / Math.PI);

                    return (
                        <div key={i} style={{
                            margin: "1em",
                        }}>
                            <h2>Camera Angle: {angle}</h2>
                            <PixelContribView scale={scale} pixelContrib={contrib} onSelectPixelContribSample={handleSelectPixelContribSample} />
                        </div>
                    );
                })
            }
        </div >
    )
}