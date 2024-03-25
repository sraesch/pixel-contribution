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
import { useEffect, useState } from "react";
import { Line } from 'react-chartjs-2';
import { PixelContributionMaps, PixelContributionMap, create_equator_series, create_equator_series_linear_interpolation } from "rs-analyze-pixel-maps";

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
    data: Float32Array;
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
function createLabels(angle_list: Float32Array): string[] {
    const labels: string[] = [];

    angle_list.forEach(angle => {
        labels.push(`Angle: ${(angle * 180 / Math.PI).toPrecision(3)}`);
    });

    return labels;
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
function createDataSeries(contrib_map: PixelContributionMap, angle_list: Float32Array): DataSeries {
    const values: Float32Array = create_equator_series(contrib_map, angle_list);

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
function createDataSeriesLinearInterpolation(contrib_map: PixelContributionMap, angle_list: Float32Array): DataSeries {
    const values: Float32Array = create_equator_series_linear_interpolation(contrib_map, angle_list);

    return {
        label: `Angle ${Math.round(contrib_map.get_description().camera_angle / Math.PI * 180)} (Linear Interpolation)`,
        data: values,
    };
}

const NUM_SAMPLES = 128;
const ANGLES: Float32Array = new Float32Array([...Array(NUM_SAMPLES).keys()].map(i => i * Math.PI / NUM_SAMPLES * 2.0));

export function EquatorGraph(props: EquatorGraphProps): JSX.Element {
    const { contrib_maps } = props;

    const [dataSeries, setDataSeries] = useState<DataSeriesList>(
        {
            labels: createLabels(ANGLES),
            datasets: [],
        }
    );

    const [currentIndex, setCurrentIndex] = useState(0);

    useEffect(() => {
        const labels = createLabels(ANGLES);

        let datasets: DataSeries[] = [];
        if (contrib_maps.size() >= 3) {
            // take first map, the middle map, and the last map
            datasets = [
                createDataSeries(contrib_maps.get_map(currentIndex), ANGLES),
                createDataSeriesLinearInterpolation(contrib_maps.get_map(currentIndex), ANGLES),
            ];
        } else {
            datasets = [...Array(contrib_maps.size()).keys()].map(i => createDataSeries(contrib_maps.get_map(i), ANGLES));
        }

        setDataSeries({
            labels,
            datasets,
        });

    }, [contrib_maps, currentIndex]);

    const handleChangeIndex = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setCurrentIndex(parseInt(event.target.value));
    }

    return (<div style={{ display: 'flex', flexDirection: 'column', marginTop: '32px', maxWidth: '1024px' }}>
        <div style={{ display: 'flex', flexDirection: 'row' }}>
            Camera Angle:
            <select style={{ maxWidth: '100px', marginLeft: '8px' }} value={currentIndex} onChange={handleChangeIndex}>
                {[...Array(contrib_maps.size()).keys()].map(i => {
                    const angle = contrib_maps.get_map(i).get_description().camera_angle;

                    return (<option key={i} value={i}>
                        {Math.round(angle * 180 / Math.PI)} degree
                    </option>);
                })}
            </select>
        </div>
        <Line options={options} data={dataSeries} />
    </div>
    );
}
