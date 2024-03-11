import { analyze } from "./analyze";
import { load_from_url } from "./pixel_contrib";

/**
 * @returns {string | null} - The pixel contribution URL from the query string, or null if it is
 *                            not present.
 */
function get_pixel_contributions_url(): string | null {
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);
    return urlParams.get('pixel_contrib');
}

async function init() {
    const url = get_pixel_contributions_url();
    if (url) {
        console.log(`Loading pixel contributions from ${url}`);
        const pixel_contrib = await load_from_url(url);
        console.log(`Loaded pixel contributions for ${pixel_contrib.length} maps`);

        analyze(pixel_contrib);
    } else {
        console.error('No pixel contribution URL found in query string');
    }
}

init();