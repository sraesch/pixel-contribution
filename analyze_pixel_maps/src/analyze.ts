import { PixelContributionMap } from "./pixel_contrib";
import * as d3 from "d3";

export function analyze(pixel_contrib: PixelContributionMap[]) {
    const container = document.querySelector("#container") as HTMLElement;

    pixel_contrib.forEach((contrib, i) => {
        const map_size = contrib.descriptor.map_size;

        const canvas = document.createElement("canvas");
        canvas.id = `contrib_${i}`;

        canvas.width = map_size;
        canvas.height = map_size;


        const ctx = canvas.getContext("2d");
        if (!ctx) {
            throw new Error("Could not get 2D context");
        }

        const image_data = ctx.createImageData(map_size, map_size);

        for (let y = 0; y < map_size; y++) {
            for (let x = 0; x < map_size; x++) {
                const value = contrib.pixel_contrib[y * map_size + x];

                const i = (y * map_size + x) * 4;
                image_data.data[i + 0] = value * 255;
                image_data.data[i + 1] = 0;
                image_data.data[i + 2] = 0;
                image_data.data[i + 3] = 255;
            }
        }

        ctx.putImageData(image_data, 0, 0);

        container.append(canvas);
    });
}