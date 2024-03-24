import { describe, it, expect } from 'vitest';
import { clamp } from './utils';


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
