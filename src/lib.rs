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

macro_rules! fourcc {
    ($str:tt) => {{
        use cast::u32;
        let bytes: &[u8] = $str.as_ref();
        assert_eq!(4, bytes.len());
        ::mpeg::FourCc(u32(bytes[0]) << 24 | u32(bytes[1]) << 16 | u32(bytes[2]) << 8 | u32(bytes[3]))
    }};
}

pub mod mpeg;

pub fn meta<R: Read>(mut from: R) -> Result<(), Error> {
    {
        let header = mpeg::read_header(&mut from)?;

        // TODO: allow other headers before ftyp
        ensure!(
            fourcc!("ftyp") == header.box_type,
            "file must start with 'ftyp' marker: {:?}",
            header
        );

        let heic_brand = fourcc!("heic");
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
            fourcc!("meta") == header.box_type,
            "file must follow with 'meta' marker: {:?}",
            header
        );

        let mut box_data = (&mut from).take(header.data_size());
        let _ = mpeg::read_full_box_header(&mut box_data)?;

        while 0 != box_data.limit() {
            let child_header = mpeg::read_header(&mut box_data)?;
            println!("| {}: {:?}", box_data.limit(), child_header);
            let mut child_data = (&mut box_data).take(child_header.data_size());
            if fourcc!("hdlr") == child_header.box_type {
                println!("| -> hdlr: {:?}", mpeg::meta::parse_hdlr(&mut child_data)?);
            } else if fourcc!("pitm") == child_header.box_type {
                println!("| -> pitm: {:?}", mpeg::meta::parse_pitm(&mut child_data)?);
            } else if fourcc!("iloc") == child_header.box_type {
                println!("| -> iloc: {:?}", mpeg::meta::parse_iloc(&mut child_data)?);
            } else if fourcc!("iinf") == child_header.box_type {
                println!("| -> iinf: {:?}", mpeg::meta::parse_iinf(&mut child_data)?);
            } else if fourcc!("iprp") == child_header.box_type {
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


#[cfg(test)]
mod tests {
    use mpeg::FourCc;
    #[test]
    fn packing_fourcc() {
        assert_eq!(FourCc(0x666F7572), fourcc!("four"));
        assert_eq!("\"four\"", format!("{:?}", fourcc!("four")));
    }
}
