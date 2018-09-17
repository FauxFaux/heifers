use std::io::Read;

use bitreader::BitReader;
use byteorder::ReadBytesExt;
use byteorder::BE;
use failure::Error;

pub mod nal;
pub mod pps;
pub mod sps;
mod ss;
mod vps;

use bit::typenum;
use bit::Bits;
use hevc::pps::PicParamSet;
use hevc::sps::SeqParamSet;

#[derive(Copy, Clone, Debug)]
struct NalUnitHeader {
    unit_type: u8,
    nuh_layer_id: u8,
    nuh_temporal_id_plus_1: u8,
}

const NAL_BLA_W_LP: u8 = 16;
const NAL_BLA_W_RADL: u8 = 17;
const NAL_BLA_N_LP: u8 = 18;
const NAL_IDR_W_RADL: u8 = 19;
const NAL_IDR_N_LP: u8 = 20;
const NAL_CRA_NUT: u8 = 21;
const NAL_RSV_IRAP_VCL22: u8 = 22;
const NAL_RSV_IRAP_VCL23: u8 = 23;

pub const NAL_SPS_NUT: u8 = 33;
pub const NAL_PPS_NUT: u8 = 34;

pub fn dump<R: Read>(mut from: R, pps: &PicParamSet, sps: &SeqParamSet) -> Result<(), Error> {
    let nal_unit_header = nal_unit_header(&mut from)?;

    ensure!(
        nal_unit_header.unit_type >= NAL_BLA_W_LP && nal_unit_header.unit_type <= NAL_CRA_NUT,
        "only supports segment layers, not {}",
        nal_unit_header.unit_type
    );

    let mut v = Vec::new();
    from.read_to_end(&mut v)?;

    ss::slice_segment_header(nal_unit_header.unit_type, &mut BitReader::new(&v), pps, sps)?;

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
