use math::Aabb;
use nalgebra_glm::{Mat4, U16Vec3, Vec3, Vec4};
use std::{marker::PhantomData, mem::size_of};

use super::IntegerTrait;

/// Callback function for rounding a f64 value to an integer
pub type RoundFunction = fn(x: f64) -> f64;

/// Returns the rounded value of the given float.
fn round_function(x: f64) -> f64 {
    x.round()
}

/// The given number floating point number x is being mapped to a fixed point number for the given
/// range.
///
/// # Arguments
///
/// * `lower_bound` -  The lower bound of the range
/// * `upper_bound` -  The upper bound of the range
/// * `x` -  The value to quantify. Must in between the given bounds
/// * `round_function` - The round function for rounding the float value.
pub fn quantize<Integer: IntegerTrait>(
    lower_bound: f64,
    upper_bound: f64,
    x: f64,
    round_function: RoundFunction,
) -> Integer {
    debug_assert!(
        lower_bound <= x && x <= upper_bound,
        "Value is not inside given range!!!"
    );

    let int_max = Integer::MAX;

    // determine the maximal ranges
    let float_max: f64 = int_max.to_f64();

    // map the input value to the range [0, 1] using the lower and upper bound
    let range: f64 = upper_bound - lower_bound;
    let lambda: f64 = if range > 0.0 {
        (x - lower_bound) / range
    } else {
        0.0
    };

    // using the rounding function, map to the range [0, float_max]
    let float_q: f64 = round_function(float_max * lambda);

    // finally, cast the quantized value to int
    if float_q < 0.0 {
        Integer::MIN
    } else if float_q >= float_max {
        int_max
    } else {
        Integer::from_f64(float_q)
    }
}

/// Returns a dequantified floating point value relative to the given number of bits and range.
///
/// * `lower_bound` -  The lower bound of the range
/// * `upper_bound` -  The upper bound of the range
/// * `q` - The quantized value
/// * `num_bits` - The number of bits used for dequantization
pub fn dequantize<Integer: IntegerTrait>(
    lower_bound: f64,
    upper_bound: f64,
    q: Integer,
    num_bits: usize,
) -> f64 {
    debug_assert!(lower_bound <= upper_bound, "Invalid range");

    // determine the maximal ranges
    let int_max = if num_bits == size_of::<Integer>() * 8 {
        Integer::MAX
    } else {
        Integer::from_u32((1u32 << num_bits) - 1u32)
    };

    // determine the maximal ranges
    let float_max: f64 = int_max.to_f64();

    // map the input value to the range [0, 1]
    let lambda: f64 = q.to_f64() / float_max;

    // map to the range [0, float_max]
    let range: f64 = upper_bound - lower_bound;

    upper_bound.min(lower_bound + lambda * range)
}

/// Returns a dequantified floating point value by assuming that all bits of the
/// integer are being used.
///
/// * `lower_bound` -  The lower bound of the range
/// * `upper_bound` -  The upper bound of the range
/// * `q` - The quantized value
pub fn dequantize_full<Integer: IntegerTrait>(
    lower_bound: f64,
    upper_bound: f64,
    q: Integer,
) -> f64 {
    debug_assert!(lower_bound <= upper_bound, "Invalid range");
    let float_max: f64 = Integer::MAX.to_f64();

    // map the input value to the range [0, 1]
    let lambda: f64 = q.to_f64() / float_max;

    // map to the range [0, float_max]
    let range: f64 = upper_bound - lower_bound;

    upper_bound.min(lower_bound + lambda * range)
}

/// Returns the quantified vector.
///
/// # Arguments
///
/// * `x` - The vector to quantify.
/// * `aabb` - The bounding volume used for quantifying the vector.
pub fn quantize_vec<Integer: IntegerTrait>(x: &Vec3, aabb: &Aabb) -> [Integer; 3] {
    let min = aabb.get_min();
    let max = aabb.get_max();

    [
        quantize(min[0] as f64, max[0] as f64, x[0] as f64, round_function),
        quantize(min[1] as f64, max[1] as f64, x[1] as f64, round_function),
        quantize(min[2] as f64, max[2] as f64, x[2] as f64, round_function),
    ]
}

/// Returns the dequantified vector.
///
/// # Arguments
///
/// * `q` - The quantified vector.
/// * `aabb` - The bounding volume used for quantifying the vector.
/// * `num_bits` - The number of bits used for dequantization.
#[inline]
pub fn dequantize_vec<Integer: IntegerTrait>(q: &[Integer], aabb: &Aabb, num_bits: usize) -> Vec3 {
    debug_assert!(q.len() >= 3);

    let min = aabb.get_min();
    let max = aabb.get_max();

    Vec3::new(
        dequantize(min[0] as f64, max[0] as f64, q[0], num_bits) as f32,
        dequantize(min[1] as f64, max[1] as f64, q[1], num_bits) as f32,
        dequantize(min[2] as f64, max[2] as f64, q[2], num_bits) as f32,
    )
}

/**
 * Reduces the accuracy of the given vector3.
 *
 * # Arguments
 * * `x` - The vector to reduce.
 * * `num_bits` - The number of bits to remove/ to reduce.
 */
#[inline]
pub fn reduce_accuracy_uvec16(x: &mut U16Vec3, num_bits: u16) {
    x[0] = reduce_accuracy_u16(x[0], num_bits);
    x[1] = reduce_accuracy_u16(x[1], num_bits);
    x[2] = reduce_accuracy_u16(x[2], num_bits);
}

/// Returns a reduced integer by removing the given number of bits.
///
/// # Arguments
/// * `x` - The value to be reduced.
/// * `num_bits` - The number of bits to remove.
#[inline]
pub fn reduce_accuracy_u16(x: u16, num_bits: u16) -> u16 {
    x >> num_bits
}

/// Returns the given integer and removes the lower n bits.
///
/// # Arguments
/// * `x` - The integer whose lower bits will be reduced
/// * `n` - The number of bits to remove.
#[inline]
pub fn reduce_lower_bits<Integer: IntegerTrait>(x: Integer, n: usize) -> Integer {
    (x >> n) << n
}

/// The descriptor for the de-quantifier operator
#[derive(Clone, Copy)]
pub struct Vec3QuantifierDesc {
    /// The lower bound for the vectors
    pub lower_bound: Vec3,

    /// The extent of the quantization volume
    pub extent: f32,
}

impl Vec3QuantifierDesc {
    /// Returns the center of the quantization volume.
    #[inline]
    pub fn get_center(&self) -> Vec3 {
        let r = self.extent / 2f32;
        let center: Vec3 = self.lower_bound + Vec3::new(r, r, r);

        center
    }

    /// Returns the extent of the quantization volume
    #[inline]
    pub fn get_extent(&self) -> f32 {
        self.extent
    }

    /// Returns a matrix that contains the operations for de-quantizing a normalized vector.
    pub fn get_dequantization_matrix(&self) -> Mat4 {
        let mut result = Mat4::new_translation(&self.lower_bound);
        result.set_diagonal(&Vec4::new(self.extent, self.extent, self.extent, 1f32));

        result
    }
}

/// The operator for quantizing and de-quantizing position data.
#[derive(Clone, Copy)]
pub struct Vec3Quantifier<Integer: IntegerTrait> {
    desc: Vec3QuantifierDesc,
    phantom: PhantomData<Integer>,
}

impl<Integer: IntegerTrait> Vec3Quantifier<Integer> {
    /// Creates a new quantifier operator for the given bounding volume.
    ///
    /// # Arguments
    /// * `volume` - The bounding volume used for quantizing and de-quantizing.
    pub fn new(volume: &Aabb) -> Self {
        debug_assert!(!volume.is_empty());

        let lower_bound = *volume.get_min();
        let extent0 = volume.get_size().amax();
        let extent1 = if extent0 > 0f32 { extent0 } else { 1f32 };

        let desc = Vec3QuantifierDesc {
            lower_bound,
            extent: extent1,
        };

        Self {
            desc,
            phantom: PhantomData {},
        }
    }

    /// Returns the internal descriptor, that contains the quantization parameter.
    #[inline]
    pub fn get_descriptor(&self) -> &Vec3QuantifierDesc {
        &self.desc
    }

    /// Returns a fully de-quantized vector
    #[inline]
    pub fn dequantize(&self, q: &[Integer]) -> Vec3 {
        let lambda = Vec3::new(q[0].to_f32(), q[1].to_f32(), q[2].to_f32()) / Integer::F32_MAX;

        self.desc.lower_bound + lambda * self.desc.get_extent()
    }

    /// Quantizes the given vector
    ///
    /// # Arguments
    /// * `q` - Mutable reference for returning the quantized values.
    /// * `x` - The vector to quantize.
    pub fn quantize(&self, q: &mut [Integer], x: &Vec3) {
        // map the input value to the range [0, 1] using the lower and upper bound
        let lambda = (x - self.desc.lower_bound) / self.desc.extent;
        let lambda = nalgebra_glm::clamp(&lambda, 0f32, 1f32);

        // using the rounding function, map to the range [0, float_max]
        let float_q = lambda * Integer::F32_MAX;
        let float_q = nalgebra_glm::round(&float_q);

        // finally, cast the quantized value to int
        q[0] = Integer::from_f32(float_q[0]);
        q[1] = Integer::from_f32(float_q[1]);
        q[2] = Integer::from_f32(float_q[2]);
    }

    /// Returns a fully de-quantized normalized vector, i.e., each value is still normalized between 0 and 1.
    #[inline]
    pub fn dequantize_normalized(&self, q: &[Integer]) -> Vec3 {
        Vec3::new(q[0].to_f32(), q[1].to_f32(), q[2].to_f32()) / Integer::F32_MAX
    }

    /// Returns a fully de-quantized normalized vector with the specified precision in bits.
    /// That is, each value is still normalized between 0 and 1.
    ///
    /// # Arguments
    /// * `q` - The quantized integer to process
    /// * `n` - The precision to dequantize in bits.
    #[inline]
    pub fn dequantize_normalized_n(&self, q: &[Integer], n: usize) -> Vec3 {
        let bits_to_remove = Integer::NUM_BITS - n;
        let max_float = Integer::F32_MAX - ((1 << bits_to_remove) - 1) as f32;

        Vec3::new(
            reduce_lower_bits(q[0], bits_to_remove).to_f32(),
            reduce_lower_bits(q[1], bits_to_remove).to_f32(),
            reduce_lower_bits(q[2], bits_to_remove).to_f32(),
        ) / max_float
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    use super::*;

    fn get_test_values() -> Vec<f64> {
        vec![-4.0, -12.0, 3.0, 7.0, 13.5, 24.0]
    }

    fn get_test_vecs() -> Vec<Vec3> {
        vec![
            Vec3::new(0.000000, 0.923880, 0.382683),
            Vec3::new(0.000000, 0.831470, 0.555570),
            Vec3::new(0.162212, 0.815493, 0.555570),
            Vec3::new(0.180240, 0.906127, 0.382683),
            Vec3::new(0.000000, 0.382683, -0.923880),
            Vec3::new(0.000000, 0.555570, -0.831470),
            Vec3::new(0.108386, 0.544895, -0.831470),
            Vec3::new(0.074658, 0.375330, -0.923880),
            Vec3::new(0.000000, 0.980785, 0.195090),
            Vec3::new(0.000000, 0.923880, 0.382683),
            Vec3::new(0.180240, 0.906127, 0.382683),
            Vec3::new(0.191342, 0.961940, 0.195090),
        ]
    }

    #[test]
    fn test_quantization_simple_u8() {
        test_quantization_simple::<u8>();
    }

    #[test]
    fn test_quantization_simple_u16() {
        test_quantization_simple::<u16>();
    }

    #[test]
    fn test_quantization_vec_u8() {
        test_quantization_vec::<u8>();
    }

    #[test]
    fn test_quantization_vec_u16() {
        test_quantization_vec::<u16>();
    }

    fn test_quantization_simple<Integer: IntegerTrait>() {
        let values = get_test_values();
        let mut max_range = f64::MIN;
        let mut min_range = f64::MAX;

        for x in values.iter() {
            min_range = min_range.min(*x);
            max_range = max_range.max(*x);
        }

        let range = [min_range, max_range];

        let num_bits = size_of::<Integer>() * 8;
        for x in values {
            let v = quantize::<Integer>(range[0], range[1], x, |x: f64| x.round());
            let v2 = dequantize(range[0], range[1], v, num_bits);
            assert!(
                (v2 - x).abs() <= (range[1] - range[0]) / Integer::MAX.to_f64(),
                "Error is too high"
            );
        }
    }

    fn test_quantization_vec<Integer: IntegerTrait>() {
        let vecs = get_test_vecs();

        let mut aabb = Aabb::new();
        vecs.iter().for_each(|v| aabb.extend_pos(v));

        let s = aabb.get_size();

        let num_bits = size_of::<Integer>() * 8;
        for v in vecs.iter() {
            let q = quantize_vec::<Integer>(v, &aabb);
            let v2 = dequantize_vec(&q, &aabb, num_bits);
            assert!(
                (v2[0] - v[0]).abs() as f64 <= (s[0] as f64) / Integer::MAX.to_f64(),
                "Error is too high"
            );
        }
    }

    fn max_bit<Integer: IntegerTrait>(x: Integer) -> usize {
        let mut result = 0;
        let mut x = x;

        while x > Integer::zero() {
            x = x >> 1usize;
            result += 1;
        }

        result
    }

    #[test]
    fn test_boundaries() {
        let float_min = -10.0;
        let float_max = 10.0;

        assert_eq!(
            max_bit(quantize::<u8>(
                float_min,
                float_max,
                float_min,
                round_function
            )),
            0
        );

        assert_eq!(
            max_bit(quantize::<u16>(
                float_min,
                float_max,
                float_min,
                round_function
            )),
            0
        );

        assert_eq!(
            max_bit(quantize::<u32>(
                float_min,
                float_max,
                float_min,
                round_function
            )),
            0
        );

        assert_eq!(
            max_bit(quantize::<u8>(
                float_min,
                float_max,
                float_max,
                round_function
            )),
            8
        );

        assert_eq!(
            max_bit(quantize::<u16>(
                float_min,
                float_max,
                float_max,
                round_function
            )),
            16
        );

        assert_eq!(
            max_bit(quantize::<u32>(
                float_min,
                float_max,
                float_max,
                round_function
            )),
            32
        );
    }

    #[test]
    fn test_boundaries2() {
        let float_min = -10.0;
        let float_max = 10.0;

        assert_eq!(
            dequantize(
                float_min,
                float_max,
                quantize::<u8>(float_min, float_max, float_min, round_function),
                8
            ),
            float_min
        );

        assert_eq!(
            dequantize(
                float_min,
                float_max,
                quantize::<u16>(float_min, float_max, float_min, round_function),
                16
            ),
            float_min
        );

        assert_eq!(
            dequantize(
                float_min,
                float_max,
                quantize::<u32>(float_min, float_max, float_min, round_function),
                32
            ),
            float_min
        );

        assert_eq!(
            dequantize(
                float_min,
                float_max,
                quantize::<u8>(float_min, float_max, float_max, round_function),
                8
            ),
            float_max
        );

        assert_eq!(
            dequantize(
                float_min,
                float_max,
                quantize::<u16>(float_min, float_max, float_max, round_function),
                16
            ),
            float_max
        );

        assert_eq!(
            dequantize(
                float_min,
                float_max,
                quantize::<u32>(float_min, float_max, float_max, round_function),
                32
            ),
            float_max
        );
    }

    #[test]
    fn test_quantization_precision() {
        let mut r = ChaCha8Rng::seed_from_u64(2);

        let float_min = -10.0;
        let float_max = 10.0;

        for num_bits in 1usize..17usize {
            for _ in 0..1000 {
                let x0 = r.gen_range(float_min..float_max);
                let qv = quantize::<u16>(float_min, float_max, x0, round_function);
                let qv = reduce_accuracy_u16(qv, 16 - num_bits as u16);

                assert!(max_bit(qv) <= num_bits);

                let x1 = dequantize::<u16>(float_min, float_max, qv, num_bits);

                assert!((x0 - x1).abs() <= (float_max - float_min) / ((1 << num_bits) as f64));
            }
        }
    }
}
