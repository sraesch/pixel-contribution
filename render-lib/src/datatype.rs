use gl::types::*;

/// The data type in which vertex and index data is encoded
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum DataType {
    Byte = gl::BYTE,                    // GL_BYTE
    UnsignedByte = gl::UNSIGNED_BYTE,   // GL_UNSIGNED_BYTE
    Short = gl::SHORT,                  // GL_SHORT
    UnsignedShort = gl::UNSIGNED_SHORT, // GL_UNSIGNED_SHORT
    Int = gl::INT,                      // GL_INT
    UnsignedInt = gl::UNSIGNED_INT,     // GL_UNSIGNED_INT
    Float = gl::FLOAT,                  // GL_FLOAT
}

pub trait DataTypeTrait {
    fn to_type() -> DataType;
}

impl DataType {
    #[inline]
    /// Returns the size in bytes of the data type
    pub fn size(&self) -> usize {
        match self {
            DataType::Byte => 1usize,
            DataType::UnsignedByte => 1usize,
            DataType::Short => 2usize,
            DataType::UnsignedShort => 2usize,
            DataType::Int => 4usize,
            DataType::UnsignedInt => 4usize,
            DataType::Float => 4usize,
        }
    }

    pub fn from_type<Type: DataTypeTrait>() -> Self {
        Type::to_type()
    }

    /// Returns true if the datatype is an integer and false otherwise.
    #[inline]
    pub fn is_integer(self) -> bool {
        self != DataType::Float
    }

    /// Returns the corresponding OpenGL type
    #[inline]
    pub fn to_gl_type(self) -> GLenum {
        self as GLenum
    }
}

impl DataTypeTrait for i8 {
    fn to_type() -> DataType {
        DataType::Byte
    }
}

impl DataTypeTrait for u8 {
    fn to_type() -> DataType {
        DataType::UnsignedByte
    }
}

impl DataTypeTrait for i16 {
    fn to_type() -> DataType {
        DataType::Short
    }
}

impl DataTypeTrait for u16 {
    fn to_type() -> DataType {
        DataType::UnsignedShort
    }
}

impl DataTypeTrait for i32 {
    fn to_type() -> DataType {
        DataType::Int
    }
}

impl DataTypeTrait for u32 {
    fn to_type() -> DataType {
        DataType::UnsignedInt
    }
}

impl DataTypeTrait for f32 {
    fn to_type() -> DataType {
        DataType::Float
    }
}

#[cfg(test)]
mod tests {
    use super::DataType;

    #[test]
    fn test_data_type_size() {
        assert_eq!(DataType::Byte.size(), 1);
        assert_eq!(DataType::UnsignedByte.size(), 1);

        assert_eq!(DataType::Short.size(), 2);
        assert_eq!(DataType::UnsignedShort.size(), 2);

        assert_eq!(DataType::Int.size(), 4);
        assert_eq!(DataType::UnsignedInt.size(), 4);

        assert_eq!(DataType::Float.size(), 4);
    }

    #[test]
    fn test_from_type() {
        assert_eq!(DataType::from_type::<i8>(), DataType::Byte);
        assert_eq!(DataType::from_type::<u8>(), DataType::UnsignedByte);

        assert_eq!(DataType::from_type::<i16>(), DataType::Short);
        assert_eq!(DataType::from_type::<u16>(), DataType::UnsignedShort);

        assert_eq!(DataType::from_type::<i32>(), DataType::Int);
        assert_eq!(DataType::from_type::<u32>(), DataType::UnsignedInt);

        assert_eq!(DataType::from_type::<f32>(), DataType::Float);
    }
}
