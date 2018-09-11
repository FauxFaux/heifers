use std::io::Read;
use std::io::Take;

use bitreader::BitReader;
use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::u16;
use cast::u32;
use cast::usize;
use failure::Error;

use bit::typenum;
use bit::Bits;
use mpeg::read_full_box_header;
use mpeg::read_header;
use mpeg::skip;
use mpeg::FourCc;

#[derive(Clone, Debug)]
pub enum Property {
    HvcCodecSettings(Hvcc),
    Size((u32, u32)),
    Unknown(FourCc),
}

#[derive(Clone, Debug)]
pub struct RawProps {
    pub containers: Vec<Vec<Property>>,
    pub associations: Vec<Vec<ItemPropertyAssociation>>,
}

#[derive(Clone, Debug)]
pub struct ItemPropertyAssociation {
    pub item_id: u32,
    pub associations: Vec<Association>,
}

#[derive(Copy, Clone, Debug)]
pub struct Association {
    pub essential: bool,
    pub property_index: u16,
}

#[derive(Clone, Debug)]
pub struct Hvcc {
    header: HvccHeader,
    pub nals: Vec<Nal>,
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
pub struct Nal {
    pub completeness_and_nal_unit_type: u8,
    pub units: Vec<Vec<u8>>,
}

pub fn parse_iprp<R: Read>(mut from: &mut Take<R>) -> Result<RawProps, Error> {
    let mut containers = Vec::with_capacity(1);
    let mut associations = Vec::with_capacity(1);

    while 0 != from.limit() {
        let child_header = read_header(&mut from)?;
        let mut child_data = (&mut from).take(child_header.data_size());
        match child_header.box_type {
            super::IPCO => containers.push(parse_ipco(&mut child_data)?),
            super::IPMA => associations.push(parse_ipma(&mut child_data)?),
            _ => skip(&mut child_data)?,
        }

        ensure!(
            0 == child_data.limit(),
            "iprp parser failed to parse a segment: {:?}",
            child_header
        );
    }

    Ok(RawProps {
        containers,
        associations,
    })
}

pub fn parse_ipco<R: Read>(mut from: &mut Take<R>) -> Result<Vec<Property>, Error> {
    let mut properties = Vec::with_capacity(2);

    while 0 != from.limit() {
        let child_header = read_header(&mut from)?;
        let mut child_data = (&mut from).take(child_header.data_size());
        match child_header.box_type {
            super::ISPE => properties.push(Property::Size(parse_ispe(&mut child_data)?)),
            super::HVCC => {
                properties.push(Property::HvcCodecSettings(parse_hvcc(&mut child_data)?))
            }
            other => {
                properties.push(Property::Unknown(other));
                skip(&mut child_data)?
            }
        }

        ensure!(
            0 == child_data.limit(),
            "ipco parser failed to parse a segment: {:?}",
            child_header
        );
    }

    Ok(properties)
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

pub fn parse_hvcc<R: Read>(mut from: &mut Take<R>) -> Result<Hvcc, Error> {
    let header = {
        let mut bits = Bits::<typenum::U22>::read_exact(&mut from)?;

        let header = HvccHeader {
            configuration_version: bits.read_u8(8),
            general_profile_space: bits.read_u8(2),
            general_tier_flag: bits.read_bool(),
            general_profile_idc: bits.read_u8(5),
            general_profile_compatibility_flags: bits.read_u32(32),
            general_constraint_indicator_flags: bits.read_u64(48),
            general_level_idc: bits.read_u8(8),
            min_spatial_segmentation_idc: bits.skip(4).read_u16(12),
            parallelism_type: bits.skip(6).read_u8(2),
            chroma_format: bits.skip(6).read_u8(2),
            bit_depth_luma_minus8: bits.skip(5).read_u8(3),
            bit_depth_chroma_minus8: bits.skip(5).read_u8(3),
            avg_frame_rate: bits.read_u16(16),
            constant_frame_rate: bits.read_u8(2),
            num_temporal_layers: bits.read_u8(3),
            temporal_id_nested: bits.read_bool(),
            length_size_minus_one: bits.read_u8(2),
        };

        assert!(bits.done());

        header
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
