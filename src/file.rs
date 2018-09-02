use std::io::Read;

use failure::Error;

use mpeg;

// TODO: not Debug
#[derive(Debug)]
pub struct Heif {
    raw: mpeg::meta::RawMeta,
}

impl Heif {
    pub fn new<R: Read>(from: R) -> Result<Heif, Error> {
        let raw = mpeg::load_meta(from)?;

        Ok(Heif { raw })
    }
}
