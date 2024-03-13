import { describe, it, expect } from 'vitest';
import { clamp, decode_octahedron_normal, encode_octahedron_normal } from './pixel_contrib';
import { vec3 } from 'gl-matrix';


describe('test clamp function', () => {
    it('clamp(3, 0, 10)', () => {
        expect(clamp(3, 0, 10)).toBe(3);
    });

    it('clamp(-2, 0, 10)', () => {
        expect(clamp(-2, 0, 10)).toBe(0);
    });

    it('clamp(11, 0, 10)', () => {
        expect(clamp(11, 0, 10)).toBe(10);
    });
});

describe('test octahedron normal encoding', () => {
    const num = 20;
    const pi = Math.PI;
    const pi_2 = pi / 2;

    for (let i = 0; i <= num; ++i) {
        // beta is angle between -PI/2 and +PI/2
        const beta = i / num * pi - pi_2;

        // determine the radius on the 2D XY-plane
        const r2 = Math.cos(beta);

        // determine value for Z
        const z = Math.sin(beta);

        for (let j = 0; j < num; ++j) {
            // alpha is angle between 0 and 2 * PI
            const alpha = j / num * 2.0 * pi;

            // determine value for X and Y
            const x = Math.cos(alpha) * r2;
            const y = Math.sin(alpha) * r2;

            const nrm = vec3.fromValues(x, y, z);

            // octahedron encoding
            const octahedron = encode_octahedron_normal(nrm);
            it('octahedron within range', () => {
                expect(0.0 <= octahedron[0] && octahedron[0] <= 1.0).toBe(true);
                expect(0.0 <= octahedron[1] && octahedron[1] <= 1.0).toBe(true);
            });

            // octahedron decoding
            const nrm2 = decode_octahedron_normal(octahedron);

            // compute error
            const angle_error = Math.abs(1.0 - vec3.dot(nrm, nrm2));

            it('check octahedron encoding accuracy', () => {
                expect(angle_error <= 1e-6).toBe(true);
            });
        }
    }
});