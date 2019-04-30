#[macro_use]
extern crate num_derive;
extern crate num_traits;

use num_traits::FromPrimitive;

mod pe;
mod table;
pub mod schema;
pub mod database;

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

type Result<T> = ::std::result::Result<T, Box<std::error::Error>>; // TODO: better error type
