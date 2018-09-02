extern crate bitreader;
extern crate byteorder;
extern crate cast;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate more_asserts;

use std::io::Read;

use failure::Error;

mod file;
pub mod mpeg;

pub use file::Heif;
