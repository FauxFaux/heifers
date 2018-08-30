use std::io::Read;
use std::io::Take;

use bitreader::BitReader;
use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::u16;
use cast::u32;
use cast::u64;
use cast::usize;
use failure::Error;

use mpeg::read_full_box_header;
use mpeg::read_header;
use skip;

#[derive(Clone, Debug)]
pub struct ItemPropertyAssociation {
    item_id: u32,
    associations: Vec<Association>,
}

#[derive(Copy, Clone, Debug)]
struct Association {
    essential: bool,
    property_index: u16,
}

#[derive(Clone, Debug)]
pub struct Hvcc {
    header: HvccHeader,
    nals: Vec<Nal>,
}

// what an absolute unit
#[derive(Copy, Clone, Debug)]
struct HvccHeader {
    configuration_version: u8,
    general_profile_space: u8,
    general_tier_flag: bool,
    general_profile_idc: u8,
    general_profile_compatibility_flags: u32,
    general_constraint_indicator_flags: u64,
    general_level_idc: u8,
    min_spatial_segmentation_idc: u16,
    parallelism_type: u8,
    chroma_format: u8,
    bit_depth_luma_minus8: u8,
    bit_depth_chroma_minus8: u8,
    avg_frame_rate: u16,
    constant_frame_rate: u8,
    num_temporal_layers: u8,
    temporal_id_nested: bool,
    length_size_minus_one: u8,
}

#[derive(Clone, Debug)]
struct Nal {
    completeness_and_nal_unit_type: u8,
    units: Vec<Vec<u8>>,
}

pub fn parse_iprp<R: Read>(mut from: &mut Take<R>) -> Result<(), Error> {
    while 0 != from.limit() {
        let child_header = read_header(&mut from)?;
        println!("| | {}: {:?}", from.limit(), child_header);
        let mut child_data = (&mut from).take(child_header.data_size());
        match child_header.box_type {
            super::IPMA => println!("| | -> ipma: {:?}", parse_ipma(&mut child_data)?),
            super::IPCO => println!("| | -> ipco: {:?}", parse_ipco(&mut child_data)?),
            _ => {
                println!("| | .. unsupported");
                skip(&mut child_data)?;
            }
        }

        ensure!(
            0 == child_data.limit(),
            "iprp parser failed to parse a segment: {:?}",
            child_header
        );
    }

    Ok(())
}

pub fn parse_ipco<R: Read>(mut from: &mut Take<R>) -> Result<(), Error> {
    while 0 != from.limit() {
        let child_header = read_header(&mut from)?;
        println!("| | | {}: {:?}", from.limit(), child_header);
        let mut child_data = (&mut from).take(child_header.data_size());
        match child_header.box_type {
            super::IPSE => println!("| | | -> ispe: {:?}", parse_ispe(&mut child_data)?),
            super::HVCC => println!("| | | -> hvcC: {:?}", parse_hvcc(&mut child_data)?),
            _ => {
                println!("| | | .. unsupported");
                skip(&mut child_data)?;
            }
        }

        ensure!(
            0 == child_data.limit(),
            "ipco parser failed to parse a segment: {:?}",
            child_header
        );
    }

    Ok(())
}

pub fn parse_ipma<R: Read>(mut from: &mut Take<R>) -> Result<Vec<ItemPropertyAssociation>, Error> {
    let extended = read_full_box_header(&mut from)?;
    let entry_count = from.read_u32::<BE>()?;

    let mut property_associations = Vec::with_capacity(usize(entry_count));

    for _ in 0..entry_count {
        let item_id = if extended.version < 1 {
            u32(from.read_u16::<BE>()?)
        } else {
            from.read_u32::<BE>()?
        };

        let association_count = from.read_u8()?;

        let mut associations = Vec::with_capacity(usize(association_count));

        for _ in 0..association_count {
            associations.push(if 0 != (extended.flags & 1) {
                let val = from.read_u16::<BE>()?;
                let mask = 0b1000_0000_0000_0000;
                Association {
                    essential: 0 != (val & mask),
                    property_index: val & (!mask),
                }
            } else {
                let val = from.read_u8()?;
                let mask = 0b1000_0000;
                Association {
                    essential: 0 != (val & mask),
                    property_index: u16(val & (!mask)),
                }
            });
        }

        property_associations.push(ItemPropertyAssociation {
            item_id,
            associations,
        })
    }

    Ok(property_associations)
}

pub fn parse_ispe<R: Read>(mut from: &mut Take<R>) -> Result<(u32, u32), Error> {
    let _ = read_full_box_header(&mut from)?;
    Ok((from.read_u32::<BE>()?, from.read_u32::<BE>()?))
}

pub fn parse_hvcc<R: Read>(from: &mut Take<R>) -> Result<Hvcc, Error> {
    let header = {
        let mut fixed = [0u8; 22];
        from.read_exact(&mut fixed)?;
        let mut bits = BitReader::new(&fixed);

        let configuration_version = bits.read_u8(8)?;
        let general_profile_space = bits.read_u8(2)?;
        let general_tier_flag = bits.read_bool()?;
        let general_profile_idc = bits.read_u8(5)?;
        let general_profile_compatibility_flags = bits.read_u32(32)?;
        let general_constraint_indicator_flags = bits.read_u64(48)?;
        let general_level_idc = bits.read_u8(8)?;
        skip_reserved(&mut bits, 4)?;
        let min_spatial_segmentation_idc = bits.read_u16(12)?;
        skip_reserved(&mut bits, 6)?;
        let parallelism_type = bits.read_u8(2)?;
        skip_reserved(&mut bits, 6)?;
        let chroma_format = bits.read_u8(2)?;
        skip_reserved(&mut bits, 5)?;
        let bit_depth_luma_minus8 = bits.read_u8(3)?;
        skip_reserved(&mut bits, 5)?;
        let bit_depth_chroma_minus8 = bits.read_u8(3)?;
        let avg_frame_rate = bits.read_u16(16)?;
        let constant_frame_rate = bits.read_u8(2)?;
        let num_temporal_layers = bits.read_u8(3)?;
        let temporal_id_nested = bits.read_bool()?;
        let length_size_minus_one = bits.read_u8(2)?;

        assert_eq!(
            bits.position(),
            8 * u64(fixed.len()),
            "bitreader should be empty as we're done"
        );

        HvccHeader {
            configuration_version,
            general_profile_space,
            general_tier_flag,
            general_profile_idc,
            general_profile_compatibility_flags,
            general_constraint_indicator_flags,
            general_level_idc,
            min_spatial_segmentation_idc,
            parallelism_type,
            chroma_format,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            avg_frame_rate,
            constant_frame_rate,
            num_temporal_layers,
            temporal_id_nested,
            length_size_minus_one,
        }
    };

    let num_of_arrays = from.read_u8()?;
    let mut nals = Vec::with_capacity(usize(num_of_arrays));

    for _ in 0..num_of_arrays {
        let completeness_and_nal_unit_type = from.read_u8()? & 0b1011_1111;

        let num_nal_units = from.read_u16::<BE>()?;
        let mut units = Vec::with_capacity(usize(num_nal_units));

        for _ in 0..num_nal_units {
            let nal_unit_length = from.read_u16::<BE>()?;
            let mut unit = vec![0u8; usize(nal_unit_length)];
            from.read_exact(&mut unit)?;
            units.push(unit);
        }

        nals.push(Nal {
            completeness_and_nal_unit_type,
            units,
        })
    }

    Ok(Hvcc { header, nals })
}

fn skip_reserved(bits: &mut BitReader, bit_count: u8) -> Result<(), Error> {
    let _ = bits.read_u8(bit_count)?;
    // TODO: validate this is ... unknown?
    Ok(())
}
