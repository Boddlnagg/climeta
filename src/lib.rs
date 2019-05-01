#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod core;

pub mod schema;
pub mod database;

type Result<T> = ::std::result::Result<T, Box<std::error::Error>>; // TODO: better error type
