extern crate bitreader;
extern crate byteorder;
extern crate cast;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate more_asserts;

use std::io::Read;

use failure::Error;

pub mod mpeg;

pub fn open<R: Read>(mut from: R) -> Result<(), Error> {
    let file_type = loop {
        let header = mpeg::read_header(&mut from)?;

        match header.box_type {
            mpeg::FTYP => break mpeg::parse_ftyp(&mut (&mut from).take(header.data_size()))?,
            mpeg::META | mpeg::MOOV => bail!("invalid header before 'ftyp': {:?}", header),
            _ => mpeg::skip_box(&mut from, &header)?,
        }
    };

    ensure!(
        file_type.major_brand == mpeg::HEIC || file_type.brands.contains(&mpeg::HEIC),
        "file is not an heic file: {:?}",
        file_type
    );

    Ok(loop {
        let header = mpeg::read_header(&mut from)?;

        match header.box_type {
            mpeg::META => break mpeg::meta::parse(&mut (&mut from).take(header.data_size()))?,
            mpeg::FTYP | mpeg::MOOV | mpeg::MDAT => {
                bail!("invalid header before 'meta': {:?}", header)
            }
            _ => mpeg::skip_box(&mut from, &header)?,
        }
    })
}
