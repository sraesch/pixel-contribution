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
import { useEffect, useState } from "react";
import { Line } from 'react-chartjs-2';
import { AnglePixelContribInterpolator, LinearPixelContribInterpolator, PixelContribInterpolator, QuadraticPixelContribInterpolator } from "../interpolate";

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
 * A single data series for the graph.
 */
interface DataSeries {
    label: string;
    data: number[];
    borderColor: string;
    backgroundColor: string;
}

/**
 * A data series for the graph.
 */
interface DataSeriesList {
    labels: string[];
    datasets: DataSeries[];
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
 * Creates a new data series using the given interpolation operator.
 * 
 * @param color - The color of the data series.
 * @param i - The interpolation operator.
 * @param contrib_maps - The pixel contribution maps from which to interpolate.
 * @param pos - The position on the pixel contribution map to execute the interpolation on.
 *
 * @returns a new data series using the given interpolation operator.
 */
function createInterpolationDataSeries<I extends PixelContribInterpolator>(color: [number, number, number], i: I, contrib_maps: PixelContributionMap[], pos: [number, number]): DataSeries {
    let values: number[] = [];

    if (contrib_maps.length >= 3) {
        values = contrib_maps.map(contrib => {
            const angle = contrib.descriptor.camera_angle;
            return i.interpolate(angle, pos);
        });
    }

    return {
        label: `Pixel Contribution (${i.name} Interpolation)`,
        data: values,
        borderColor: `rgb(${color[0]}, ${color[1]}, ${color[2]})`,
        backgroundColor: `rgba(${color[0]}, ${color[1]}, ${color[2]}, 0.5)`,
    };
}

export function InterpolateAngleGraph(props: InterpolateAngleGraphProps): JSX.Element {
    const { contrib_maps, pos } = props;

    const [dataSeries, setDataSeries] = useState<DataSeriesList>(
        {
            labels: createLabels(contrib_maps),
            datasets: [],
        }
    );

    useEffect(() => {
        const labels = createLabels(contrib_maps);

        if (contrib_maps.length <= 2) {
            return;
        }

        const pixel_pos: [number, number] = pos === null ? [0, 0] : pos;
        const values: number[] = extractValues(contrib_maps, pixel_pos);
        const linear_interpolation = createInterpolationDataSeries([99, 255, 132], new LinearPixelContribInterpolator(contrib_maps), contrib_maps, pixel_pos);
        const angle_interpolation = createInterpolationDataSeries([99, 132, 255], new AnglePixelContribInterpolator(contrib_maps), contrib_maps, pixel_pos);
        const quadratic_interpolation = createInterpolationDataSeries([128, 128, 128], new QuadraticPixelContribInterpolator(contrib_maps), contrib_maps, pixel_pos);

        setDataSeries({
            labels,
            datasets: [
                {
                    label: 'Pixel Contribution',
                    data: values,
                    borderColor: 'rgb(255, 99, 132)',
                    backgroundColor: 'rgba(255, 99, 132, 0.5)',
                },
                linear_interpolation,
                angle_interpolation,
                quadratic_interpolation
            ],
        });

    }, [contrib_maps, pos]);

    return <Line options={options} data={dataSeries} />;
}