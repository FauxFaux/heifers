extern crate bitreader;
extern crate byteorder;
extern crate cast;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate more_asserts;

use std::io::Read;

use cast::usize;
use failure::Error;
use std::io::Take;

pub mod mpeg;

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
        let header = mpeg::read_header(&mut from)?;
        // TODO: allow other headers before meta
        ensure!(
            mpeg::pack_box_type(*b"meta") == header.box_type,
            "file must follow with 'meta' marker: {:?}",
            header
        );

        let mut box_data = (&mut from).take(header.data_size());
        let _ = mpeg::read_full_box_header(&mut box_data)?;

        while 0 != box_data.limit() {
            let child_header = mpeg::read_header(&mut box_data)?;
            println!("| {}: {:?}", box_data.limit(), child_header);
            let mut child_data = (&mut box_data).take(child_header.data_size());
            if mpeg::pack_box_type(*b"hdlr") == child_header.box_type {
                println!("| -> hdlr: {:?}", mpeg::meta::parse_hdlr(&mut child_data)?);
            } else if mpeg::pack_box_type(*b"pitm") == child_header.box_type {
                println!("| -> pitm: {:?}", mpeg::meta::parse_pitm(&mut child_data)?);
            } else if mpeg::pack_box_type(*b"iloc") == child_header.box_type {
                println!("| -> iloc: {:?}", mpeg::meta::parse_iloc(&mut child_data)?);
            } else if mpeg::pack_box_type(*b"iinf") == child_header.box_type {
                println!("| -> iinf: {:?}", mpeg::meta::parse_iinf(&mut child_data)?);
            } else if mpeg::pack_box_type(*b"iprp") == child_header.box_type {
                println!("| -> iprp: {:?}", mpeg::iprp::parse_iprp(&mut child_data)?);
            } else {
                // skip unrecognised
                skip(&mut child_data)?;
            }

            ensure!(
                0 == child_data.limit(),
                "meta parser failed to parse a segment: {:?}",
                child_header
            );
        }
    }

    Ok(())
}

fn skip<R: Read>(child_data: &mut Take<R>) -> Result<(), Error> {
    let remaining = usize(child_data.limit());
    child_data.read_exact(&mut vec![0u8; remaining])?;
    Ok(())
}
