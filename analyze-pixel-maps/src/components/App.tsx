import { useEffect, useState } from "react";
import { PixelContribViews } from "./PixelContribViews";
import { InterpolateAngleGraph } from "./InterpolateAngleGraph";
import { InterpolationErrorViews } from "./InterpolationErrorViews";
import { AnglePixelContribInterpolator, LinearPixelContribInterpolator, PixelContribBarycentricInterpolator, PixelContribBarycentricInterpolatorFine, PixelContribValuePerAxisInterpolator, QuadraticPixelContribInterpolator } from "../interpolate";
import { PixelContributionMaps } from 'rs-analyze-pixel-maps';
import { SphereView } from "./SphereView";
import { EquatorGraph } from "./EquatorGraph";

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
      <h1>Pixel Contribution Analysis Page</h1>
      <div>
        <h2>Pixel Contribution Maps</h2>
        <PixelContribViews pixelContribMaps={pixelContrib} onSelectPixelContribSample={handleSelectPixelContribSample} />
      </div>
      <h2 style={{ marginTop: '2em' }}>Contribution values for fixed position</h2>
      <p style={{ color: 'gray' }}>
        Click on a pixel in the contribution maps above to see the contribution values for that position
        in the different contribution maps.
      </p>
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
      <div>
        <h2>Interpolated Angle Contribution Error</h2>
        <InterpolationErrorViews onSelectError={handleSelectError} contrib_maps={pixelContrib} interpolator={new LinearPixelContribInterpolator(pixelContrib)} />
        <InterpolationErrorViews onSelectError={handleSelectError} contrib_maps={pixelContrib} interpolator={new AnglePixelContribInterpolator(pixelContrib)} />
        <InterpolationErrorViews onSelectError={handleSelectError} contrib_maps={pixelContrib} interpolator={new QuadraticPixelContribInterpolator(pixelContrib)} />
      </div>

      <h2 style={{ marginTop: '2em' }}>Spherical view on the contribution</h2>
      <SphereView contrib_maps={pixelContrib} canvas_size={512} />
      <EquatorGraph contrib_maps={pixelContrib} />
      <div>
        <h2>Interpolated Sphere Contribution Error</h2>
        <InterpolationErrorViews onSelectError={handleSelectError} contrib_maps={pixelContrib} interpolator={new PixelContribValuePerAxisInterpolator(pixelContrib)} />
        <InterpolationErrorViews onSelectError={handleSelectError} contrib_maps={pixelContrib} interpolator={new PixelContribBarycentricInterpolator(pixelContrib)} />
        <InterpolationErrorViews onSelectError={handleSelectError} contrib_maps={pixelContrib} interpolator={new PixelContribBarycentricInterpolatorFine(pixelContrib)} />
      </div>
    </main>
  )
}

export default App
