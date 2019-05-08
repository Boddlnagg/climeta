use num_traits::FromPrimitive;

pub(crate) mod db;
pub(crate) mod table;
pub(crate) mod pe;
pub(crate) mod columns;

pub(crate) trait BitView {
    fn get_bit(self, bit: usize) -> bool;
    fn get_enum<T: FromPrimitive>(self, mask: Self) -> T;
}

impl BitView for u16 {
    fn get_bit(self, bit: usize) -> bool {
        (1 << bit) & self != 0
    }
    fn get_enum<T: FromPrimitive>(self, mask: Self) -> T {
        T::from_u16(self & mask).unwrap()
    }
}

impl BitView for u32 {
    fn get_bit(self, bit: usize) -> bool {
        (1 << bit) & self != 0
    }
    fn get_enum<T: FromPrimitive>(self, mask: Self) -> T {
        T::from_u32(self & mask).unwrap()
    }
}

pub(crate) trait ByteView {
    unsafe fn view_as<T>(&self, offset: usize) -> &T;
    unsafe fn view_as_slice<T>(&self, offset: usize, count: usize) -> &[T];
    fn as_c_str(&self, offset: usize) -> &[u8];
    fn as_string(&self, offset: usize) -> Option<&[u8]>;
    fn sub(&self, start: usize, len: usize) -> &Self;
}

impl ByteView for [u8] {
    unsafe fn view_as<T>(&self, offset: usize) -> &T {
        &*(&self[offset] as *const u8 as *const T)
    }

    unsafe fn view_as_slice<T>(&self, offset: usize, count: usize) -> &[T] {
        std::slice::from_raw_parts(&self[offset] as *const u8 as *const T, count)
    }

    fn as_c_str(&self, offset: usize) -> &[u8] {
        match &self[offset..].iter().position(|b| *b == b'\0') {
            Some(idx) => &self[offset..(offset + idx)],
            None => &self[offset..]
        }
    }

    fn as_string(&self, offset: usize) -> Option<&[u8]> {
        let length = self[offset];
        match length {
            0 => Some(&[]), // empty string
            0xff => None, // null string
            _ => Some(self.sub(offset + 1, length as usize))
        }
    }

    fn sub(&self, start: usize, len: usize) -> &[u8] {
        &self[start..(start + len)]
    }
}

/// Returns the index of the first element in the range [start, end), whose mapping
/// (after being fed to `mapping`) is not less than (i.e. greater or equal to) `x`,
/// or `end` if no such element is found. 
pub(crate) fn lower_bound_with<T: Ord + Copy, F>(start: usize, end: usize, mapping: F, x: T) -> usize
    where F: Fn(usize) -> T
{
    debug_assert!(end >= start);
    let mut size = end - start;
    if size == 0 {
        return start;
    }
    let mut base = start;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        base = if mapping(mid) < x { mid } else { base };
        size -= half;
    }
    base + (mapping(base) < x) as usize
}

pub(crate) fn equal_range_with<T: Ord + Copy, F: Clone>(start: usize, end: usize, mapping: F, x: T) -> (usize, usize)
    where F: Fn(usize) -> T
{
    let lower = lower_bound_with(start, end, mapping.clone(), x);
    let mut upper = lower;
    while upper < end && mapping(upper) == x {
        upper += 1;
    }
    (lower, upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_bound_with() {
        let slice = &[0, 0, 1, 2, 4, 4, 4, 8, 16, 16];
        assert_eq!(lower_bound_with(0, 10, |idx| slice[idx], 0), 0);
        assert_eq!(lower_bound_with(0, 10, |idx| slice[idx], 1), 2);
        assert_eq!(lower_bound_with(1, 10, |idx| slice[idx], 1), 2);
        assert_eq!(lower_bound_with(3, 10, |idx| slice[idx], 1), 3);
        assert_eq!(lower_bound_with(0, 10, |idx| slice[idx], 16), 8);
        assert_eq!(lower_bound_with(0, 10, |idx| slice[idx], 17), 10);
        assert_eq!(lower_bound_with(7, 7, |idx| slice[idx], 0), 7); // empty input range
    }

    #[test]
    fn test_equal_range_with() {
        let slice = &[0, 0, 1, 2, 4, 4, 4, 8, 16, 16];
        assert_eq!(equal_range_with(0, 10, |idx| slice[idx], 0), (0, 2));
        assert_eq!(equal_range_with(0, 10, |idx| slice[idx], 1), (2, 3));
        assert_eq!(equal_range_with(3, 10, |idx| slice[idx], 0), (3, 3)); // empty output range
        assert_eq!(equal_range_with(0, 10, |idx| slice[idx], 3), (4, 4));
        assert_eq!(equal_range_with(0, 10, |idx| slice[idx], 4), (4, 7));
        assert_eq!(equal_range_with(0, 10, |idx| slice[idx], 16), (8, 10));
        assert_eq!(equal_range_with(0, 8, |idx| slice[idx], 16), (8, 8));
        assert_eq!(equal_range_with(7, 7, |idx| slice[idx], 0), (7, 7)); // empty input range
    }
}