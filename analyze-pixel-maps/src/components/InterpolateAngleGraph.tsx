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
import { AnglePixelContribInterpolator, LinearPixelContribInterpolator, QuadraticPixelContribInterpolator } from "../interpolate";

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

    const i = new LinearPixelContribInterpolator(contrib_maps);

    return contrib_maps.map(contrib => {
        const angle = contrib.descriptor.camera_angle;
        return i.interpolate(angle, pos);
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

    const i = new AnglePixelContribInterpolator(contrib_maps);

    return contrib_maps.map(contrib => {
        const angle = contrib.descriptor.camera_angle;
        return i.interpolate(angle, pos);
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

    const i = new QuadraticPixelContribInterpolator(contrib_maps);

    return contrib_maps.map(contrib => {
        const angle = contrib.descriptor.camera_angle;
        return i.interpolate(angle, pos);
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