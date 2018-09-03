#[macro_use]
extern crate bitflags;
extern crate bitreader;
extern crate byteorder;
extern crate cast;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate more_asserts;
extern crate generic_array;

mod bit;
mod file;
pub mod hevc;
pub mod mpeg;

pub use file::Heif;
