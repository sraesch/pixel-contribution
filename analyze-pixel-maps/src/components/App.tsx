import { useEffect, useState } from "react";
import { PixelContribViews } from "./PixelContribViews";
import { InterpolateAngleGraph } from "./InterpolateAngleGraph";
import { PixelContribErrorViews } from "./PixelContribErrorViews";
import { AnglePixelContribInterpolator, LinearPixelContribInterpolator, QuadraticPixelContribInterpolator } from "../interpolate";
import { PixelContributionMaps } from 'rs-analyze-pixel-maps';

/**
 * @returns {string | null} - The pixel contribution URL from the query string, or null if it is
 *                            not present.
 */
function getPixelContribURL(): string | null {
  const queryString = window.location.search;
  const urlParams = new URLSearchParams(queryString);
  return urlParams.get('pixel_contrib');
}

function App(): JSX.Element {
  const [pixelContrib, setPixelContrib] = useState<PixelContributionMaps | null>(null);
  const [contribPos, setContribPos] = useState<[number, number]>([0, 0]);

  // load pixel contribution data from a URL
  useEffect(() => {
    const url = getPixelContribURL();
    if (url) {
      console.log(`Loading pixel contributions from ${url}`);

      PixelContributionMaps.from_reader(url).then((pixel_contrib: PixelContributionMaps): void => {
        console.log(`Loaded pixel contributions for ${pixel_contrib.size()} maps`);
        setPixelContrib(pixel_contrib);
      });
    }
  }, []);

  const handleSelectPixelContribSample = (pos_x: number, pos_y: number, angle: number) => {
    console.log(`Selected pixel sample at (${pos_x}, ${pos_y}) with angle ${angle}`);
    setContribPos([pos_x, pos_y]);
  };

  if (pixelContrib === null || pixelContrib.size() <= 2) {
    return <div></div>;
  }

  const handleSelectError = (error: number) => {
    alert(`Selected error: ${error}`);
  };

  return (
    <main>
      <PixelContribViews pixelContribMaps={pixelContrib} onSelectPixelContribSample={handleSelectPixelContribSample} />
      <div style={{
        width: "90%",
        height: "90%",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        margin: "4em",
      }}>
        <InterpolateAngleGraph contrib_maps={pixelContrib} pos={contribPos} />
      </div>
      <PixelContribErrorViews onSelectError={handleSelectError} scale={10.0} contrib_maps={pixelContrib} interpolator={new LinearPixelContribInterpolator(pixelContrib)} />
      <PixelContribErrorViews onSelectError={handleSelectError} scale={10.0} contrib_maps={pixelContrib} interpolator={new AnglePixelContribInterpolator(pixelContrib)} />
      <PixelContribErrorViews onSelectError={handleSelectError} scale={10.0} contrib_maps={pixelContrib} interpolator={new QuadraticPixelContribInterpolator(pixelContrib)} />
    </main>
  )
}

export default App
