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
            mpeg::FTYP == header.box_type,
            "file must start with 'ftyp' marker: {:?}",
            header
        );

        let file_type = mpeg::parse_ftyp((&mut from).take(header.data_size()))?;

        ensure!(
            file_type.major_brand == mpeg::HEIC || file_type.brands.contains(&mpeg::HEIC),
            "file is not an heic file: {:?}",
            file_type
        );
    }

    {
        let header = mpeg::read_header(&mut from)?;
        // TODO: allow other headers before meta
        ensure!(
            mpeg::META == header.box_type,
            "file must follow with 'meta' marker: {:?}",
            header
        );

        let mut box_data = (&mut from).take(header.data_size());
        let _ = mpeg::read_full_box_header(&mut box_data)?;

        while 0 != box_data.limit() {
            let child_header = mpeg::read_header(&mut box_data)?;
            println!("| {}: {:?}", box_data.limit(), child_header);
            let mut child_data = (&mut box_data).take(child_header.data_size());
            match child_header.box_type {
                mpeg::HDLR => println!("| -> hdlr: {:?}", mpeg::meta::parse_hdlr(&mut child_data)?),
                mpeg::PITM => println!("| -> pitm: {:?}", mpeg::meta::parse_pitm(&mut child_data)?),
                mpeg::ILOC => println!("| -> iloc: {:?}", mpeg::meta::parse_iloc(&mut child_data)?),
                mpeg::IINF => println!("| -> iinf: {:?}", mpeg::meta::parse_iinf(&mut child_data)?),
                mpeg::IPRP => println!("| -> iprp: {:?}", mpeg::iprp::parse_iprp(&mut child_data)?),
                _ => skip(&mut child_data)?,
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
