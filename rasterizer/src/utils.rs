/// Constraint a value to lie between two further values
///
/// # Arguments
/// * `x` - The value to constraint.
/// * `min_value` - The lower bound for the value constraint.
/// * `max_value` - The upper bound for the value constraint.
#[inline]
pub fn clamp<T>(x: T, min_value: T, max_value: T) -> T
where
    T: PartialOrd,
{
    if x < min_value {
        min_value
    } else if x > max_value {
        max_value
    } else {
        x
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(3, 0, 10), 3);
        assert_eq!(clamp(-2, 0, 10), 0);
        assert_eq!(clamp(11, 0, 10), 10);
    }
}
