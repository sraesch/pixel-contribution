import { useEffect, useRef } from "react";
import { ColorMapType, PixelContributionMap } from "rs-analyze-pixel-maps";

/**
 * Callback for when a pixel is selected in the PixelContribView.
 */
export type OnSelectPixelContribSample = (pos_x: number, pos_y: number, angle: number) => void;

export interface PixelContribViewProps {
    pixelContrib: PixelContributionMap;
    onSelectPixelContribSample?: OnSelectPixelContribSample;
    scale?: number;
    colormap: ColorMapType;
}

export function PixelContribView(props: PixelContribViewProps): JSX.Element {
    const { pixelContrib, onSelectPixelContribSample, colormap } = props;

    const scale = props.scale || 1.0;

    const handleCanvasClick = (event: React.MouseEvent<HTMLCanvasElement>, angle: number) => {
        if (!onSelectPixelContribSample) {
            return;
        }

        const canvas = event.currentTarget;
        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;

        const map_size = pixelContrib.get_description().map_size;
        const pos_x = Math.floor((x / rect.width) * map_size);
        const pos_y = Math.floor((y / rect.height) * map_size);

        onSelectPixelContribSample(pos_x, pos_y, angle);
    }

    const canvasRef = useRef<null | HTMLCanvasElement>(null);

    useEffect(() => {
        const map_size = pixelContrib.get_description().map_size;

        const canvas = canvasRef.current;
        if (!canvas) {
            return;
        }

        canvas.width = map_size;
        canvas.height = map_size;

        const ctx = canvas.getContext("2d");
        if (!ctx) {
            console.error("Could not get 2D context");
            return;
        }

        const image_data = pixelContrib.draw_image(scale, colormap);

        ctx.putImageData(image_data, 0, 0);

    }, [pixelContrib, scale, colormap]);

    return (
        <canvas ref={canvasRef}
            width={pixelContrib.get_description().map_size}
            height={pixelContrib.get_description().map_size}
            onClick={event => handleCanvasClick(event, pixelContrib.get_description().camera_angle)} />
    );
}