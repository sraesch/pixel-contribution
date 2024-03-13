
class Header {
    magic: string;
    version: number;

    constructor(magic: string, version: number) {
        this.magic = magic;
        this.version = version;
    }

    public static from_bytes(bytes: Uint8Array) {
        const magic = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
        const version = new DataView(bytes.buffer).getUint32(4, true);
        return new Header(magic, version);
    }

    /**
     * @returns {boolean} - True if the header is valid, false otherwise.
     */
    public is_valid(): boolean {
        return this.magic === 'PCMP' && this.version === 1;
    }
}

/** 
 * The descriptor for the pixel contribution map.
 */
export class PixelContribMapDescriptor {
    /// The size of the quadratic pixel contribution map.
    public map_size: number;

    /// The camera angle for the pixel contribution map. The angle is in radians.
    /// A value of 0 means that the camera is orthographic.
    public camera_angle: number;

    public constructor(map_size: number, camera_angle: number) {
        this.map_size = map_size;
        this.camera_angle = camera_angle;
    }
}

/**
 * The resulting pixel contribution for all possible views.
 */
export class PixelContributionMap {
    public descriptor: PixelContribMapDescriptor;

    /// The 2D map for the pixel contribution of each view. Each position on the map
    /// represents a view. The normalized position (u,v) is mapped to a normal using octahedral
    /// projection. The normal then defines the camera view direction onto the object.
    /// The pixel contribution values are in the range [0, 1].
    public pixel_contrib: Float32Array;

    public constructor(descriptor: PixelContribMapDescriptor, pixel_contrib: Float32Array) {
        this.descriptor = descriptor;
        this.pixel_contrib = pixel_contrib;
    }
}

/**
 * Load pixel contribution data from a URL.
 * 
 * @param url {string} - The URL to load the pixel contribution data from.
 */
export async function load_from_url(url: string): Promise<PixelContributionMap[]> {
    const response = await fetch(url);
    const data = new Uint8Array(await response.arrayBuffer());

    // read the header
    const header = Header.from_bytes(data.slice(0, 8));
    if (!header.is_valid()) {
        console.error('Invalid pixel contribution header');
        throw new Error('Invalid pixel contribution header');
    }

    // read the number of pixel contribution maps
    const num_maps = new DataView(data.buffer).getUint32(8, true);
    console.log(`Found ${num_maps} pixel contribution maps`);

    // read the pixel contribution maps
    let offset = 12;
    const maps: PixelContributionMap[] = [];
    for (let i = 0; i < num_maps; i++) {
        const map_size = new DataView(data.buffer).getUint32(offset, true);
        offset += 4;
        const camera_angle = new DataView(data.buffer).getFloat32(offset, true);
        offset += 4;
        const descriptor = new PixelContribMapDescriptor(map_size, camera_angle);

        const pixel_contrib = new Float32Array(data.buffer, offset, map_size * map_size);
        offset += map_size * map_size * 4;
        const map = new PixelContributionMap(descriptor, pixel_contrib);

        maps.push(map);
    }

    return maps;
}