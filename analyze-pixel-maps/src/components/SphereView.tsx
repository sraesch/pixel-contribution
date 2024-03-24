import { useEffect, useRef, useState } from "react";
import { clamp } from "../utils";
import { quat, vec3 } from "gl-matrix";
import createColormap from "colormap";
import { PixelContributionMaps } from 'rs-analyze-pixel-maps';

export interface SphereViewProps {
    contrib_maps: PixelContributionMaps;
    canvas_size: number;
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
    const [angleX, setAngleX] = useState<number>(0);
    const [angleY, setAngleY] = useState<number>(0);
    const [scale, setScale] = useState<number>(1.0);

    const { contrib_maps, canvas_size } = props;

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

        const angle_x_radians = angleX * Math.PI / 180;
        const angle_y_radians = angleY * Math.PI / 180;
        const qx = quat.setAxisAngle(quat.create(), vec3.fromValues(1, 0, 0), angle_x_radians);
        const qy = quat.setAxisAngle(quat.create(), vec3.fromValues(0, 1, 0), angle_y_radians);

        for (let y = 0; y < canvas_size; y++) {
            for (let x = 0; x < canvas_size; x++) {
                const dir = vec3.fromValues(x, y, 0);
                vec3.sub(dir, dir, vec3.fromValues(canvas_size * 0.5, canvas_size * 0.5, 0));
                vec3.scale(dir, dir, 2.0 / canvas_size);

                let color = [0, 0, 0];
                if (vec3.len(dir) <= 1.0) {
                    dir[2] = Math.sqrt(1.0 - vec3.dot(dir, dir));
                    vec3.normalize(dir, dir);

                    // rotate the direction vector
                    vec3.transformQuat(dir, dir, qy);
                    vec3.transformQuat(dir, dir, qx);

                    const index = contrib_map.get_description().index_from_camera_dir(dir[0], dir[1], dir[2]);
                    const value = contrib_map.get_value_at_index(index);

                    color = colorMap[clamp(Math.floor(value * scale * 255), 0, 255)];
                }

                // determine the index into the image data array
                const px = clamp(Math.round(x / canvas_size * map_size), 0, map_size - 1);
                const py = clamp(Math.round(y / canvas_size * map_size), 0, map_size - 1);

                const i = (py * map_size + px) * 4;
                image_data.data[i + 0] = color[0];
                image_data.data[i + 1] = color[1];
                image_data.data[i + 2] = color[2];
                image_data.data[i + 3] = 255;
            }
        }

        ctx.putImageData(image_data, 0, 0);

    }, [contrib_maps, canvas_size, contribMapIndex, angleX, angleY, scale]);

    if (contrib_maps.size() === 0) {
        return <div></div>;
    }

    const handleOnChangeAngleY = (event: React.ChangeEvent<HTMLInputElement>) => {
        const value = event.target.value;
        setAngleY(parseInt(value));
    }

    const handleOnChangeAngleX = (event: React.ChangeEvent<HTMLInputElement>) => {
        const value = event.target.value;
        setAngleX(parseInt(value));
    }

    const handleScale = (event: React.ChangeEvent<HTMLInputElement>) => {
        const value = event.target.value;
        setScale(parseFloat(value));
    }

    return (<div style={{
        display: "flex",
        flexDirection: "column",
    }}>
        <span style={{ marginTop: '8px' }}>
            Contrib Map(Angle):
            <select onChange={handleChangeIndex} style={{ maxWidth: '90px', marginLeft: '8px' }}>
                {[...Array(contrib_maps.size()).keys()].map(i => {
                    const d = contrib_maps.get_map_descriptor(i);

                    return (
                        <option key={i}>
                            Angle={Math.round(d.camera_angle * 180 / Math.PI)}
                        </option>);
                })}
            </select>
        </span>
        <span style={{ marginTop: '8px' }}>
            Angle (Degree) X:
            <input type="number" value={angleX} id="angleX" name="angleX" min="0" max="360" style={{ maxWidth: '48px', marginLeft: '1rem' }} onChange={handleOnChangeAngleX} />
        </span>
        <span style={{ marginTop: '8px' }}>
            Angle (Degree) Y:
            <input type="number" value={angleY} id="angleY" name="angleY" min="0" max="360" style={{ maxWidth: '48px', marginLeft: '1rem' }} onChange={handleOnChangeAngleY} />
        </span>
        <span style={{ marginTop: '8px' }}>
            Scale Colormap:
            <input type="number" value={scale} id="scale" name="scale" min="0.1" max="4" step={0.1} style={{ maxWidth: '48px', marginLeft: '1rem' }} onChange={handleScale} />
        </span>
        <canvas ref={canvasRef} style={{
            width: canvas_size,
            height: canvas_size,
        }} />
    </div >);
}