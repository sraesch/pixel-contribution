use byte_slice_cast::FromByteSlice;

use core::ops::Shl;

use std::{mem::size_of, ops::Shr};

/// Extended trait for integers
pub trait IntegerTrait:
    num::Integer + Copy + Shl<usize, Output = Self> + Shr<usize, Output = Self> + FromByteSlice
{
    const MAX: Self;
    const MIN: Self;

    const F32_MAX: f32;

    const NUM_BITS: usize = size_of::<Self>() * 8;

    fn from_f64(x: f64) -> Self;
    fn from_f32(x: f32) -> Self;
    fn from_u32(x: u32) -> Self;

    fn to_f64(&self) -> f64;
    fn to_f32(&self) -> f32;
    fn to_u32(&self) -> u32;
    fn to_usize(&self) -> usize;
}

impl IntegerTrait for u8 {
    const MAX: Self = u8::MAX;
    const MIN: Self = u8::MIN;

    const F32_MAX: f32 = Self::MAX as f32;

    #[inline]
    fn from_f64(x: f64) -> Self {
        x as Self
    }

    #[inline]
    fn from_f32(x: f32) -> Self {
        x as Self
    }

    #[inline]
    fn from_u32(x: u32) -> Self {
        x as Self
    }

    #[inline]
    fn to_f64(&self) -> f64 {
        *self as f64
    }

    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }

    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl IntegerTrait for u16 {
    const MAX: Self = u16::MAX;
    const MIN: Self = u16::MIN;

    const F32_MAX: f32 = Self::MAX as f32;

    #[inline]
    fn from_f64(x: f64) -> Self {
        x as Self
    }

    #[inline]
    fn from_f32(x: f32) -> Self {
        x as Self
    }

    #[inline]
    fn from_u32(x: u32) -> Self {
        x as Self
    }

    #[inline]
    fn to_f64(&self) -> f64 {
        *self as f64
    }

    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }

    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl IntegerTrait for u32 {
    const MAX: Self = u32::MAX;
    const MIN: Self = u32::MIN;

    const F32_MAX: f32 = Self::MAX as f32;

    #[inline]
    fn from_f64(x: f64) -> Self {
        x as Self
    }

    #[inline]
    fn from_f32(x: f32) -> Self {
        x as Self
    }

    #[inline]
    fn from_u32(x: u32) -> Self {
        x as Self
    }

    #[inline]
    fn to_f64(&self) -> f64 {
        *self as f64
    }

    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }

    #[inline]
    fn to_u32(&self) -> u32 {
        *self
    }

    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl IntegerTrait for usize {
    const MAX: Self = usize::MAX;
    const MIN: Self = usize::MIN;

    const F32_MAX: f32 = Self::MAX as f32;

    #[inline]
    fn from_f64(x: f64) -> Self {
        x as Self
    }

    #[inline]
    fn from_f32(x: f32) -> Self {
        x as Self
    }

    #[inline]
    fn from_u32(x: u32) -> Self {
        x as Self
    }

    #[inline]
    fn to_f64(&self) -> f64 {
        *self as f64
    }

    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }

    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    #[inline]
    fn to_usize(&self) -> usize {
        *self
    }
}
