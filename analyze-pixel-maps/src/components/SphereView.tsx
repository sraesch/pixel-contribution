import { useEffect, useRef, useState } from "react";
import { vec3 } from "gl-matrix";
import createColormap from "colormap";
import { PixelContributionMaps } from 'rs-analyze-pixel-maps';

export interface SphereViewProps {
    contrib_maps: PixelContributionMaps;
}

const colorMap = createColormap({
    colormap: "jet",
    nshades: 256,
    format: "rgba",
    alpha: 1,
});

export function SphereView(props: SphereViewProps): JSX.Element {
    const [contribMapIndex, setContribMapIndex] = useState<number>(0);
    const canvasRef = useRef<null | HTMLCanvasElement>(null);

    const { contrib_maps } = props;

    const handleChangeIndex = (event: React.ChangeEvent<HTMLSelectElement>) => {
        const index = event.target.selectedIndex;
        setContribMapIndex(index);
    }

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) {
            return;
        }

        const contrib_map = contrib_maps.get_map(contribMapIndex);
        const map_size = contrib_map.get_description().map_size;

        canvas.width = map_size;
        canvas.height = map_size;

        const ctx = canvas.getContext("2d");
        if (!ctx) {
            console.error("Could not get 2D context");
            return;
        }

        const image_data = ctx.createImageData(map_size, map_size);

        for (let y = 0; y < map_size; y++) {
            for (let x = 0; x < map_size; x++) {
                const dir = vec3.fromValues(x, y, 0);
                vec3.sub(dir, dir, vec3.fromValues(map_size * 0.5, map_size * 0.5, 0));
                vec3.scale(dir, dir, 2.0 / map_size);

                let color = [0, 0, 0];
                if (vec3.len(dir) <= 1.0) {
                    dir[2] = Math.sqrt(1.0 - vec3.dot(dir, dir));
                    vec3.normalize(dir, dir);

                    const index = contrib_map.get_description().index_from_camera_dir(dir[0], dir[1], dir[2]);
                    const value = contrib_map.get_value_at_index(index);

                    color = colorMap[Math.floor(value * 255)];
                }

                const i = (y * map_size + x) * 4;
                image_data.data[i + 0] = color[0];
                image_data.data[i + 1] = color[1];
                image_data.data[i + 2] = color[2];
                image_data.data[i + 3] = 255;
            }
        }

        ctx.putImageData(image_data, 0, 0);

    }, [contrib_maps, contribMapIndex]);

    if (contrib_maps.size() === 0) {
        return <div></div>;
    }

    return (<div style={{
        display: "flex",
        flexDirection: "column",
    }}>
        <select onChange={handleChangeIndex} style={{ maxWidth: '256px' }}>
            {[...Array(contrib_maps.size()).keys()].map(i => {
                const d = contrib_maps.get_map_descriptor(i);

                return (
                    <option key={i}>
                        Angle={Math.round(d.camera_angle * 180 / Math.PI)}
                    </option>);
            })}
        </select>
        <canvas ref={canvasRef} style={{
            width: contrib_maps.get_map_descriptor(0).map_size,
            height: contrib_maps.get_map_descriptor(0).map_size,
        }}></canvas>
    </div>);
}