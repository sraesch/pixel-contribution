import { useEffect, useRef } from "react";
import { PixelContributionMap } from "../pixel_contrib";
import createColormap from "colormap";

export interface PixelContribViewProps {
    pixelContrib: PixelContributionMap;
}

export function PixelContribView(props: PixelContribViewProps): JSX.Element {
    const { pixelContrib } = props;

    const canvasRef = useRef<null | HTMLCanvasElement>(null);

    useEffect(() => {
        const map_size = pixelContrib.descriptor.map_size;

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

        const colorMap = createColormap({
            colormap: "jet",
            nshades: 256,
            format: "rgba",
            alpha: 1,
        });

        const image_data = ctx.createImageData(map_size, map_size);

        for (let y = 0; y < map_size; y++) {
            for (let x = 0; x < map_size; x++) {
                const value = pixelContrib.pixel_contrib[y * map_size + x];
                const color = colorMap[Math.floor(value * 255)];

                const i = (y * map_size + x) * 4;
                image_data.data[i + 0] = color[0];
                image_data.data[i + 1] = color[1];
                image_data.data[i + 2] = color[2];
                image_data.data[i + 3] = 255;
            }
        }

        ctx.putImageData(image_data, 0, 0);

    }, [pixelContrib]);

    return (
        <canvas ref={canvasRef} width={pixelContrib.descriptor.map_size} height={pixelContrib.descriptor.map_size} />
    );
}