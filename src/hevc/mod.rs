use std::io::Read;

use byteorder::ReadBytesExt;
use byteorder::BE;
use failure::Error;

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
