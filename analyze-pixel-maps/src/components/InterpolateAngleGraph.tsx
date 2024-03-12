import { PixelContributionMap } from "../pixel_contrib";

import {
    Chart as ChartJS,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend,
} from 'chart.js';
import { mat2, vec2 } from "gl-matrix";
import { useEffect, useState } from "react";
import { Line } from 'react-chartjs-2';

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend
);

const options = {
    responsive: true,
    plugins: {
        legend: {
            position: 'top' as const,
        },
        title: {
            display: true,
            text: 'Angle Interpolation Graph',
        },
    },
};

export interface InterpolateAngleGraphProps {
    contrib_maps: PixelContributionMap[];
    pos: [number, number] | null;
}

/**
 * A single data serie for the graph.
 */
interface DataSerie {
    label: string;
    data: number[];
    borderColor: string;
    backgroundColor: string;
}

/**
 * A data series for the graph.
 */
interface DataSeries {
    labels: string[];
    datasets: DataSerie[];
}

/**
 * @param contrib_maps - The pixel contribution maps from which to extract the angle.
 *
 * @returns the labels for the graph.
 */
function createLabels(contrib_maps: PixelContributionMap[]): string[] {
    return contrib_maps.map(contrib => {
        return `Angle: ${(contrib.descriptor.camera_angle * 180 / Math.PI).toPrecision(3)}`;
    });
}

/**
 * @param contrib_maps - The pixel contribution maps from which to extract the angle.
 * @param pos - The position on the pixel contribution map.
 *
 * @returns the extracted values from the pixel contribution maps at the given position.
 */
function extractValues(contrib_maps: PixelContributionMap[], pos: [number, number]): number[] {
    return contrib_maps.map(contrib => {
        return contrib.pixel_contrib[pos[1] * contrib.descriptor.map_size + pos[0]];
    });
}

/**
 * Takes the first and last contribution map and interpolates the angle between them
 * using linear interpolation.
 * 
 * @param contrib_maps - The pixel contribution maps from which to extract the angle. 
 * @param pos - The position on the pixel contribution map.
 *
 * @returns the interpolated values from the pixel contribution maps at the given position.
 */
function computeLinearInterpolation(contrib_maps: PixelContributionMap[], pos: [number, number]): number[] {
    if (contrib_maps.length === 0) {
        return [];
    }

    const first_map = contrib_maps[0];
    const last_map = contrib_maps[contrib_maps.length - 1];

    const first_angle = first_map.descriptor.camera_angle;
    const last_angle = last_map.descriptor.camera_angle;

    const index = pos[1] * first_map.descriptor.map_size + pos[0];

    const first_value = first_map.pixel_contrib[index];
    const last_value = last_map.pixel_contrib[index];

    return contrib_maps.map(contrib => {
        const angle = contrib.descriptor.camera_angle;
        const f = (angle - first_angle) / (last_angle - first_angle);
        return first_value * (1 - f) + last_value * f;
    });
}

/**
 * Takes the first and last contribution map and interpolates the angle between them
 * using angle interpolation.
 * 
 * @param contrib_maps - The pixel contribution maps from which to extract the angle. 
 * @param pos - The position on the pixel contribution map.
 *
 * @returns the interpolated values from the pixel contribution maps at the given position.
 */
function computeAngleInterpolation(contrib_maps: PixelContributionMap[], pos: [number, number]): number[] {
    if (contrib_maps.length === 0) {
        return [];
    }

    const first_map = contrib_maps[0];
    const last_map = contrib_maps[contrib_maps.length - 1];

    const first_angle = first_map.descriptor.camera_angle;
    const last_angle = last_map.descriptor.camera_angle;

    const index = pos[1] * first_map.descriptor.map_size + pos[0];

    const first_value = first_map.pixel_contrib[index];
    const last_value = last_map.pixel_contrib[index];

    const a_start = Math.tan(first_angle / 2.0);
    const a_last = Math.tan(last_angle / 2.0);

    return contrib_maps.map(contrib => {
        const angle = contrib.descriptor.camera_angle;
        const a = Math.tan(angle / 2.0);
        const t = (a - a_start) / (a_last - a_start);

        return first_value * (1 - t) + last_value * t;
    });
}

/**
 * Takes the first, middle and last contribution map and interpolates the angle between them
 * using a quadratic interpolation.
 * 
 * @param contrib_maps - The pixel contribution maps from which to extract the angle. 
 * @param pos - The position on the pixel contribution map.
 *
 * @returns the interpolated values from the pixel contribution maps at the given position.
 */
function computeQuadraticInterpolation(contrib_maps: PixelContributionMap[], pos: [number, number]): number[] {
    if (contrib_maps.length <= 2) {
        return [];
    }

    const first_map = contrib_maps[0];
    const last_map = contrib_maps[contrib_maps.length - 1];
    const middle_map = contrib_maps[Math.floor((contrib_maps.length) / 2)];

    const first_angle = first_map.descriptor.camera_angle;
    if (first_angle !== 0) {
        return [];
    }

    const last_angle = last_map.descriptor.camera_angle;
    const middle_angle = middle_map.descriptor.camera_angle;

    const index = pos[1] * first_map.descriptor.map_size + pos[0];

    const first_value = first_map.pixel_contrib[index];
    const last_value = last_map.pixel_contrib[index];
    const middle_value = middle_map.pixel_contrib[index];

    // determine the coefficients of the quadratic function
    const c = first_value;
    const A = mat2.invert(mat2.create(), mat2.fromValues(middle_angle * middle_angle, last_angle * last_angle, middle_angle, last_angle));
    const rhs = vec2.fromValues(middle_value - c, last_value - c);

    const coeffs = vec2.transformMat2(vec2.create(), rhs, A);
    const a = coeffs[0];
    const b = coeffs[1];

    return contrib_maps.map(contrib => {
        const angle = contrib.descriptor.camera_angle;

        return a * angle * angle + b * angle + c;
    });
}

export function InterpolateAngleGraph(props: InterpolateAngleGraphProps): JSX.Element {
    const { contrib_maps, pos } = props;

    const [dataSeries, setDataSeries] = useState<DataSeries>(
        {
            labels: createLabels(contrib_maps),
            datasets: [],
        }
    );

    useEffect(() => {
        const labels = createLabels(contrib_maps);

        const pixel_pos: [number, number] = pos === null ? [0, 0] : pos;
        const values: number[] = extractValues(contrib_maps, pixel_pos);
        const linear_interpolation: number[] = computeLinearInterpolation(contrib_maps, pixel_pos);
        const angle_interpolation: number[] = computeAngleInterpolation(contrib_maps, pixel_pos);
        const quadratic_interpolation: number[] = computeQuadraticInterpolation(contrib_maps, pixel_pos);

        setDataSeries({
            labels,
            datasets: [
                {
                    label: 'Pixel Contribution',
                    data: values,
                    borderColor: 'rgb(255, 99, 132)',
                    backgroundColor: 'rgba(255, 99, 132, 0.5)',
                },
                {
                    label: 'Pixel Contribution (Linear Interpolation)',
                    data: linear_interpolation,
                    borderColor: 'rgb(99, 255, 132)',
                    backgroundColor: 'rgba(99, 255, 132, 0.5)',
                },
                {
                    label: 'Pixel Contribution (Angle Interpolation)',
                    data: angle_interpolation,
                    borderColor: 'rgb(99, 132, 255)',
                    backgroundColor: 'rgba(99, 132, 255, 0.5)',
                },
                {
                    label: 'Pixel Contribution (Quadratic Interpolation)',
                    data: quadratic_interpolation,
                    borderColor: 'rgb(128, 128, 128)',
                    backgroundColor: 'rgba(128, 128, 128, 0.5)',
                }
            ],
        });

    }, [contrib_maps, pos]);

    return <Line options={options} data={dataSeries} />;
}