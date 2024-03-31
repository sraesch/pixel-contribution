import { useState } from "react";
import { PixelContribView } from "./PixelContribView";
import { ColorMapType, PixelContributionMaps } from "rs-analyze-pixel-maps";

export interface PixelContribViewsProps {
    pixelContribMaps: PixelContributionMaps;
    onSelectPixelContribSample?: (pos_x: number, pos_y: number, angle: number) => void;
}

export function PixelContribViews(props: PixelContribViewsProps): JSX.Element {
    const { pixelContribMaps, onSelectPixelContribSample } = props;
    const [scale, setScale] = useState<number>(1.0);
    const [colormap, setColormap] = useState<ColorMapType>(ColorMapType.Turbo);

    const handleSelectPixelContribSample = (pos_x: number, pos_y: number, angle: number) => {
        if (onSelectPixelContribSample) {
            onSelectPixelContribSample(pos_x, pos_y, angle);
        }
    };

    const handleScale = (event: React.ChangeEvent<HTMLInputElement>) => {
        const value = parseFloat(event.target.value);
        setScale(value);
    };

    const handleChangeIndex = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setColormap(parseInt(event.target.value));
    }

    return (
        <div>
            <span style={{ marginTop: '8px' }}>
                Scale Colormap:
                <input type="number" value={scale} id="scale" name="scale" min="0.1" max="20" step={0.1} style={{ maxWidth: '48px', marginLeft: '1rem' }} onChange={handleScale} />
            </span>
            <span style={{ marginTop: '8px' }}>
                Colormap:
                <select style={{ maxWidth: '100px', marginLeft: '8px' }} value={colormap} onChange={handleChangeIndex}>
                    <option value={ColorMapType.Turbo}>Turbo</option>
                    <option value={ColorMapType.Gray}>Gray</option>
                </select>
            </span>
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
                                <PixelContribView scale={scale} pixelContrib={contrib} colormap={colormap} onSelectPixelContribSample={handleSelectPixelContribSample} />
                            </div>
                        );
                    })
                }
            </div >
        </div>
    )
}