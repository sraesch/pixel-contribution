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