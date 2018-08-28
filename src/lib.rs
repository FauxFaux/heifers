extern crate byteorder;
extern crate cast;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate more_asserts;

use std::io::Read;

use cast::u64;
use cast::usize;
use failure::Error;

pub mod mpeg;

#[derive(Debug, Fail)]
enum HeifError {
    #[fail(display = "other invalid file: {}", msg)]
    InvalidFile { msg: String },
}

pub fn meta<R: Read>(mut from: R) -> Result<(), Error> {
    {
        let header = mpeg::read_header(&mut from)?;

        // TODO: allow other headers before ftyp
        ensure!(
            mpeg::pack_box_type(*b"ftyp") == header.box_type,
            "file must start with 'ftyp' marker: {:?}",
            header
        );

        let heic_brand = mpeg::pack_box_type(*b"heic");
        let file_type = mpeg::parse_ftyp((&mut from).take(header.data_size()))?;

        ensure!(
            file_type.major_brand == heic_brand || file_type.brands.contains(&heic_brand),
            "file is not an heic file: {:?}",
            file_type
        );
    }

    {
        let mut header = mpeg::read_header(&mut from)?;
        // TODO: allow other headers before meta
        ensure!(
            mpeg::pack_box_type(*b"meta") == header.box_type,
            "file must follow with 'meta' marker: {:?}",
            header
        );

        {
            let _ = mpeg::read_full_box_header(&mut from)?;
            header.offset += 4;
        }

        let mut box_data = (&mut from).take(header.data_size());
        while 0 != box_data.limit() {
            let child_header = mpeg::read_header(&mut box_data)?;
            println!("{}: {:?}", box_data.limit(), child_header);
            dirty_skip(&mut box_data, &child_header)?;
        }
    }

    Ok(())
}

fn dirty_skip<R: Read>(mut from: R, header: &mpeg::BoxHeader) -> Result<(), Error> {
    from.read_exact(&mut vec![0u8; usize(header.data_size())])?;
    Ok(())
}
