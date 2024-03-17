import { clamp } from "../pixel_contrib";

import {
    Chart as ChartJS,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend,
    Colors,
} from 'chart.js';
import { vec3 } from "gl-matrix";
import { useEffect, useState } from "react";
import { Line } from 'react-chartjs-2';
import { PixelContributionMaps, PixelContributionMap } from "rs-analyze-pixel-maps";

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend,
    Colors
);

const options = {
    responsive: true,
    plugins: {
        legend: {
            position: 'top' as const,
        },
        title: {
            display: true,
            text: 'Equator Graph',
        },
    },
};

export interface EquatorGraphProps {
    contrib_maps: PixelContributionMaps;
}

/**
 * A single data series for the graph.
 */
interface DataSeries {
    label: string;
    data: number[];
}

/**
 * A data series for the graph.
 */
interface DataSeriesList {
    labels: string[];
    datasets: DataSeries[];
}

/**
 * @param angle_list - The list of angles to create labels for.
 *
 * @returns the labels for the graph.
 */
function createLabels(angle_list: number[]): string[] {
    return angle_list.map(angle => {
        return `Angle: ${(angle * 180 / Math.PI).toPrecision(3)}`;
    });
}

/**
 * Creates a new data series for the given pixel contribution map by sampling the values at the
 * equator.
 * 
 * @param contrib_map - The pixel contribution maps from which to interpolate.
 * @param angle_list - The list of angles to sample at.
 *
 * @returns a new data series using the given interpolation operator.
 */
function createDataSeries(contrib_map: PixelContributionMap, angle_list: number[]): DataSeries {
    const values = angle_list.map(angle => {
        const x = Math.cos(angle);
        const y = Math.sin(angle);

        vec3.fromValues(x, y, 0);

        const index = contrib_map.get_description().index_from_camera_dir(x, y, 0);

        return contrib_map.get_value_at_index(index);
    });

    return {
        label: `Angle ${Math.round(contrib_map.get_description().camera_angle / Math.PI * 180)}`,
        data: values,
    };
}

/**
 * Creates a new data series for the given pixel contribution map by sampling the values at the
 * equator using linear interpolation.
 * 
 * @param contrib_map - The pixel contribution maps from which to interpolate.
 * @param angle_list - The list of angles to sample at.
 *
 * @returns a new data series using the given interpolation operator.
 */
function createDataSeriesLinearInterpolation(contrib_map: PixelContributionMap, angle_list: number[]): DataSeries {
    const PI_FRAC_2: number = Math.PI / 2;

    const values = angle_list.map(angle => {
        // determine previous angle
        const prev_angle = Math.floor(angle / PI_FRAC_2) * PI_FRAC_2;
        const next_angle = clamp(prev_angle + PI_FRAC_2, 0, Math.PI * 2);

        // evaluate the contribution at the previous and next angle
        const prev_index = contrib_map.get_description().index_from_camera_dir(Math.cos(prev_angle), Math.sin(prev_angle), 0);
        const next_index = contrib_map.get_description().index_from_camera_dir(Math.cos(next_angle), Math.sin(next_angle), 0);
        const prev_value = contrib_map.get_value_at_index(prev_index);
        const next_value = contrib_map.get_value_at_index(next_index);

        // determine the current value using linear interpolation
        const t = (angle - prev_angle) / (next_angle - prev_angle);
        const value = prev_value * (1 - t) + next_value * t;

        return value;
    });

    return {
        label: `Angle ${Math.round(contrib_map.get_description().camera_angle / Math.PI * 180)} (Linear Interpolation)`,
        data: values,
    };
}

const NUM_SAMPLES = 128;
const ANGLES = [...Array(NUM_SAMPLES).keys()].map(i => i * Math.PI / NUM_SAMPLES * 2.0);

export function EquatorGraph(props: EquatorGraphProps): JSX.Element {
    const { contrib_maps } = props;

    const [dataSeries, setDataSeries] = useState<DataSeriesList>(
        {
            labels: createLabels(ANGLES),
            datasets: [],
        }
    );

    useEffect(() => {
        const labels = createLabels(ANGLES);

        let datasets: DataSeries[] = [];
        if (contrib_maps.size() >= 3) {
            // take first map, the middle map, and the last map
            datasets = [
                createDataSeries(contrib_maps.get_map(0), ANGLES),
                createDataSeries(contrib_maps.get_map(Math.floor(contrib_maps.size() / 2)), ANGLES),
                createDataSeries(contrib_maps.get_map(contrib_maps.size() - 1), ANGLES),
                createDataSeriesLinearInterpolation(contrib_maps.get_map(0), ANGLES),
                createDataSeriesLinearInterpolation(contrib_maps.get_map(Math.floor(contrib_maps.size() / 2)), ANGLES),
                createDataSeriesLinearInterpolation(contrib_maps.get_map(contrib_maps.size() - 1), ANGLES),
            ];
        } else {
            datasets = [...Array(contrib_maps.size()).keys()].map(i => createDataSeries(contrib_maps.get_map(i), ANGLES));
        }

        setDataSeries({
            labels,
            datasets,
        });

    }, [contrib_maps]);

    return <Line options={options} data={dataSeries} />;
}