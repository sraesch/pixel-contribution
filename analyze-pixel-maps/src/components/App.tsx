import { useEffect, useState } from "react";
import { PixelContributionMap, load_from_url } from "../pixel_contrib";
import { PixelContribView } from "./PixelContribView";

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
  const [pixelContrib, setPixelContrib] = useState<PixelContributionMap[]>([]);

  // load pixel contribution data from a URL
  useEffect(() => {
    const url = getPixelContribURL();
    if (url) {
      console.log(`Loading pixel contributions from ${url}`);
      load_from_url(url).then(pixel_contrib => {
        console.log(`Loaded pixel contributions for ${pixel_contrib.length} maps`);
        setPixelContrib(pixel_contrib);
      });
    }
  }, []);

  return (
    <main>
      {pixelContrib.map((contrib, i) => {
        // convert angle from radians to degrees and round it
        const angle = Math.round(contrib.descriptor.camera_angle * 180 / Math.PI);

        return (
          <div key={i} style={{
            margin: "1em",
          }}>
            <h2>Camera Angle: {angle}</h2>
            <PixelContribView pixelContrib={contrib} />
          </div>
        );
      })}
    </main>
  )
}

export default App
