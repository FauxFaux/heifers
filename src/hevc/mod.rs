use std::io::Read;

use bitreader::BitReader;
use byteorder::ReadBytesExt;
use byteorder::BE;
use failure::Error;

mod pps;
mod vps;

use bit::typenum;
use bit::Bits;

#[derive(Copy, Clone, Debug)]
struct NalUnitHeader {
    unit_type: u8,
    nuh_layer_id: u8,
    nuh_temporal_id_plus_1: u8,
}

const NAL_SLICE_SEGMENT_LAYER: u8 = 19;

pub fn dump<R: Read>(from: R) -> Result<(), Error> {
    let nal_unit_header = nal_unit_header(from)?;

    ensure!(
        NAL_SLICE_SEGMENT_LAYER == nal_unit_header.unit_type,
        "only supports segment layers"
    );

    println!("{:?}", nal_unit_header);
    Ok(())
}

fn nal_unit_header<R: Read>(mut from: R) -> Result<NalUnitHeader, Error> {
    let len = from.read_u32::<BE>()?;

    let mut bits = Bits::<typenum::U2>::read_exact(from)?;

    ensure!(!bits.read_bool(), "invalid bit prefix for nal unit header");
    let unit_type = bits.read_u8(6);
    let nuh_layer_id = bits.read_u8(6);
    let nuh_temporal_id_plus_1 = bits.read_u8(3);
    assert!(bits.done());

    Ok(NalUnitHeader {
        unit_type,
        nuh_layer_id,
        nuh_temporal_id_plus_1,
    })
}

fn read_uvlc(from: &mut BitReader) -> Result<u64, Error> {
    let mut leading_zeros = 0;
    while !from.read_bool()? {
        leading_zeros += 1;

        ensure!(leading_zeros <= 63, "too many leading zeros in uvlc");
    }

    Ok(from.read_u64(leading_zeros)? + (1 << leading_zeros) - 1)
}

fn rbsp_trailing_bits(from: &mut BitReader) -> Result<(), Error> {
    ensure!(from.read_bool()?, "rbsp_trailing_bits must start with one");
    while !from.is_aligned(1) {
        ensure!(!from.read_bool()?, "rbsp_trailing_bits must be zeros");
    }
    Ok(())
}
