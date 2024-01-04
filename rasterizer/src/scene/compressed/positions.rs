use math::Aabb;
use nalgebra_glm::Vec3;

use super::{IntegerTrait, Vec3Quantifier, Vec3QuantifierDesc};

/// The number of bits for the quantization
#[repr(usize)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub enum NumBits {
    Bit8 = 8,
    Bit16 = 16,
    Bit32 = 32,
}

/// The number of bits for the quantization
pub enum CompressedPositions {
    Bit8(CompressedPositionsRaw<u8>),
    Bit16(CompressedPositionsRaw<u16>),
    Bit32(CompressedPositionsRaw<u32>),
}

impl CompressedPositions {
    /// Creates a new compressed position object from the given positions in the specified
    /// precision.
    ///
    /// # Arguments
    /// * `positions` - The positions to compress.
    /// * `num_bits` - The number of bits used for quantifying the positions.
    pub fn new(positions: &[Vec3], num_bits: NumBits) -> Self {
        match num_bits {
            NumBits::Bit8 => {
                CompressedPositions::Bit8(CompressedPositionsRaw::<u8>::new(positions))
            }
            NumBits::Bit16 => {
                CompressedPositions::Bit16(CompressedPositionsRaw::<u16>::new(positions))
            }
            NumBits::Bit32 => {
                CompressedPositions::Bit32(CompressedPositionsRaw::<u32>::new(positions))
            }
        }
    }

    /// Returns the number of positions.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            CompressedPositions::Bit8(q) => q.len(),
            CompressedPositions::Bit16(q) => q.len(),
            CompressedPositions::Bit32(q) => q.len(),
        }
    }

    /// Returns true if the compressed positions are empty, i.e., have length 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the center of the page
    #[inline]
    pub fn get_de_quantify_descriptor(&self) -> &Vec3QuantifierDesc {
        match self {
            CompressedPositions::Bit8(q) => q.q.get_descriptor(),
            CompressedPositions::Bit16(q) => q.q.get_descriptor(),
            CompressedPositions::Bit32(q) => q.q.get_descriptor(),
        }
    }
}

/// Quantized positions using uniform mapping
pub struct CompressedPositionsRaw<Integer: IntegerTrait> {
    /// The operator for quantizing and de-quantizing position data
    q: Vec3Quantifier<Integer>,

    /// The quantized position data
    data: Vec<Integer>,
}

impl<Integer: IntegerTrait> CompressedPositionsRaw<Integer> {
    /// Creates a new compressed position object from the given positions in the specified
    /// precision.
    ///
    /// # Arguments
    /// * `positions` - The positions to compress.
    pub fn new(positions: &[Vec3]) -> Self {
        let mut data: Vec<Integer> = Vec::with_capacity(positions.len() * 3);

        let q = Self::quantize_positions(positions, &mut data);

        Self { q, data }
    }

    /// Creates and returns a new raw compressed positions object from the given elements.
    ///
    /// # Arguments
    /// * `data` - The quantized values of the vector.
    /// * `q` - the (de-)quantization operator.
    pub fn from_quantized(data: Vec<Integer>, q: Vec3Quantifier<Integer>) -> Self {
        Self { data, q }
    }

    /// Returns a reference onto the internal quantization operator.
    #[inline]
    pub fn get_quantization_operator(&self) -> &Vec3Quantifier<Integer> {
        &self.q
    }

    /// Returns the number of positions.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() / 3
    }

    /// Returns true if the compressed positions are empty, i.e., have length 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the i-th position in quantized form.
    ///
    /// # Arguments
    /// * `index` - The index of the position to de-quantify.
    #[inline]
    pub fn get_quantized_position(&self, index: usize) -> [Integer; 3] {
        let q = &self.data[index * 3..];
        [q[0], q[1], q[2]]
    }

    /// De-quantifies and returns the i-th position.
    ///
    /// # Arguments
    /// * `index` - The index of the position to de-quantify.
    #[inline]
    pub fn get_position(&self, index: usize) -> Vec3 {
        let q = &self.data[index * 3..];
        self.q.dequantize(q)
    }

    /// De-quantifies and returns the i-th position, but normalized, i.e.,
    /// within the range 0..1 in relation to the quantization volume.
    ///
    /// # Arguments
    /// * `index` - The index of the position to de-quantify.
    #[inline]
    pub fn get_position_normalized(&self, index: usize) -> Vec3 {
        let q = &self.data[index * 3..];
        self.q.dequantize_normalized(q)
    }

    /// De-quantifies and returns the i-th position, but normalized, i.e.,
    /// within the range 0..1 in relation to the quantization volume.
    ///
    /// # Arguments
    /// * `index` - The index of the position to de-quantify.
    /// * `num_bits` - The precision in number of bits.
    #[inline]
    pub fn get_position_normalized_n(&self, index: usize, num_bits: usize) -> Vec3 {
        let q = &self.data[index * 3..];
        self.q.dequantize_normalized_n(q, num_bits)
    }

    /// Quantizes the given positions and writes the quantized data into the provided data.
    /// Returns the quantization operator.
    ///
    /// # Arguments
    ///
    /// * `positions` - The positions to quantify.
    /// * `data` - The destination for writing the quantized positions
    fn quantize_positions(positions: &[Vec3], data: &mut Vec<Integer>) -> Vec3Quantifier<Integer> {
        let aabb = Self::compute_aabb(positions);

        let q = Vec3Quantifier::new(&aabb);

        let mut quantized_vec: [Integer; 3] = [Integer::zero(), Integer::zero(), Integer::zero()];
        for v in positions.iter() {
            q.quantize(&mut quantized_vec, v);
            data.extend(quantized_vec);
        }

        q
    }

    /// Computes an aabb bounding volume for the provided positions.
    ///
    /// # Argument
    /// * `positions` - The
    fn compute_aabb(positions: &[Vec3]) -> Aabb {
        let mut aabb = Aabb::new();

        positions.iter().for_each(|p| aabb.extend_pos(p));

        aabb
    }

    /// Returns the required number of quantization bits for the specified precision.
    ///
    /// # Arguments
    /// * `num_bits` - The minimal required bit precision.
    pub fn determine_num_bits(num_bits: usize) -> NumBits {
        let num_bits: NumBits = if num_bits <= 8 {
            NumBits::Bit8
        } else if num_bits <= 16 {
            NumBits::Bit16
        } else {
            NumBits::Bit32
        };

        num_bits
    }
}

impl<Integer: IntegerTrait> AsRef<[Integer]> for CompressedPositionsRaw<Integer> {
    #[inline]
    fn as_ref(&self) -> &[Integer] {
        self.data.as_ref()
    }
}

#[cfg(test)]
mod test {
    use cad_import::loader::{loader_off::LoaderOff, Loader, MemoryResource};
    use nalgebra_glm::Vec3;

    use crate::scene::compressed::{CompressedPositionsRaw, IntegerTrait};

    #[test]
    fn test_quantize_positions() {
        // load cylinder data
        let cylinder_data = include_bytes!("../../../../test_data/models/cylinder.off");
        let loader = LoaderOff::new();

        let memory_resource = MemoryResource::new(
            cylinder_data,
            loader.get_mime_types().first().unwrap().clone(),
        );

        let cylinder = loader.read(&memory_resource).unwrap();

        let mesh = cylinder
            .get_root_node()
            .get_shapes()
            .first()
            .unwrap()
            .get_parts()
            .first()
            .unwrap()
            .get_mesh()
            .clone();

        let positions: Vec<Vec3> = mesh
            .get_vertices()
            .get_positions()
            .iter()
            .map(|p| p.0)
            .collect();

        // compress cylinder positions
        test_quantized_positions::<u8>(&positions);
        test_quantized_positions::<u16>(&positions);
    }

    fn test_quantized_positions<Integer: IntegerTrait>(in_positions: &Vec<Vec3>) {
        let compressed_positions: CompressedPositionsRaw<Integer> =
            CompressedPositionsRaw::new(in_positions);

        let num_positions = compressed_positions.len();

        // decompress vertices
        let mut out_positions: Vec<Vec3> = Vec::new();
        out_positions.reserve_exact(num_positions);
        out_positions
            .extend((0..num_positions).map(|index| compressed_positions.get_position(index)));

        assert_eq!(in_positions.len(), out_positions.len());

        // compute maximal error along each of the axis
        let mut max_error = [0f32, 0f32, 0f32];
        for (p0, p1) in in_positions.iter().zip(out_positions.iter()) {
            let d = (*p0 - *p1).abs();

            max_error[0] = max_error[0].max(d[0]);
            max_error[1] = max_error[1].max(d[1]);
            max_error[2] = max_error[2].max(d[2]);
        }

        // check if all errors are within bounds
        let s = compressed_positions
            .get_quantization_operator()
            .get_descriptor()
            .get_extent()
            / (Integer::MAX.to_f64() as f32);
        assert!(max_error[0] <= s);
        assert!(max_error[1] <= s);
        assert!(max_error[2] <= s);
    }
}
