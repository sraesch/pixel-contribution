import { vec2, vec3 } from "gl-matrix";

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

    /// Returns the index for the given camera direction vector.
    ///
    /// # Arguments
    /// * `dir` - The camera direction vector to the object.
    public index_from_camera_dir(dir: vec3): number {
        const uv: vec2 = encode_octahedron_normal(dir);
        vec2.scale(uv, uv, this.map_size);
        vec2.sub(uv, uv, vec2.fromValues(0.5, 0.5));

        const u = clamp(Math.round(uv[0]), 0, this.map_size - 1);
        const v = clamp(Math.round(uv[1]), 0, this.map_size - 1);

        return v * this.map_size + u;
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
 * 
 * @returns {Promise<PixelContributionMap[]>} - The pixel contribution data.
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

/**
 * Consumes a normal and returns the encoded octahedron normal as a 2D vector in the range [0, 1].
 * 
 * @param in_normal - The normal to encode
 * 
 * @returns {vec2} - The encoded normal as a 2D vector in the range [0, 1].
 */
export function encode_octahedron_normal(in_normal: vec3): vec2 {
    const normal = vec3.normalize(vec3.create(), in_normal);
    const abs_sum = Math.abs(normal[0]) + Math.abs(normal[1]) + Math.abs(normal[2]);

    normal[0] /= abs_sum;
    normal[1] /= abs_sum;

    if (normal[2] < 0.0) {
        const tmp = normal[0];
        normal[0] = wrap_octahedron_normal_value(normal[0], normal[1]);
        normal[1] = wrap_octahedron_normal_value(normal[1], tmp);
    }

    return vec2.fromValues(normal[0] * 0.5 + 0.5, normal[1] * 0.5 + 0.5);
}

/**
 * Consumes a normal encoded as octahedron in the range [0,1] and returns the decoded normal.
 * 
 * @param octahedron - The normal encoded as octahedron.
 * 
 * @returns {vec3} - The decoded normal.
 */
export function decode_octahedron_normal(in_octahedron: vec2): vec3 {
    const octahedron = vec2.scale(vec2.create(), in_octahedron, 2.0);
    vec2.sub(octahedron, octahedron, vec2.fromValues(1.0, 1.0));

    const z = 1.0 - Math.abs(octahedron[0]) - Math.abs(octahedron[1]);

    const x = z >= 0.0 ? octahedron[0] : wrap_octahedron_normal_value(octahedron[0], octahedron[1]);
    const y = z >= 0.0 ? octahedron[1] : wrap_octahedron_normal_value(octahedron[1], octahedron[0]);

    return vec3.normalize(vec3.create(), vec3.fromValues(x, y, z));
}

/**
 * Wraps the octahedron normal value.
 * 
 * @param x - The x value to wrap.
 * @param y - The y value to wrap.
 * 
 * @returns {number} - The wrapped value.
 */
function wrap_octahedron_normal_value(v1: number, v2: number): number {
    return (1.0 - Math.abs(v2)) * ((v1 >= 0.0) ? 1.0 : -1.0);
}

/**
 * Constraint a value to lie between two further values.
 * 
 * @param x - The value to constraint.
 * @param min_value - The lower bound for the value constraint.
 * @param max_value - The upper bound for the value constraint.
 * 
 * @returns {number} - The constrained value.
 */
export function clamp(x: number, min_value: number, max_value: number): number {
    if (x < min_value) {
        return min_value;
    } else if (x > max_value) {
        return max_value;
    } else {
        return x;
    }
}
