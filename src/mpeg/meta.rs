use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Take;

use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::u32;
use cast::usize;
use failure::Error;

use mpeg;
use mpeg::iprp;
use mpeg::read_full_box_header;
use mpeg::read_header;
use mpeg::read_u4_pair;
use mpeg::read_value_of_size;
use mpeg::skip;
use mpeg::Extent;
use mpeg::FourCc;
use mpeg::ItemInfo;
use mpeg::ItemLoc;

// It's unclear that there should be at-least-, or precisely-, one of most of these.
// TODO: It's probably specified.
#[derive(Clone, Debug)]
pub struct RawMeta {
    pub handler: Vec<FourCc>,             // hdlr
    pub primary_item: Vec<u16>,           // pitm
    pub item_locators: Vec<Vec<ItemLoc>>, // iloc
    pub item_infos: Vec<Vec<ItemInfo>>,   // iinf
    pub item_props: Vec<iprp::RawProps>,  // iprp
}

pub fn parse<R: Read>(mut from: &mut Take<R>) -> Result<RawMeta, Error> {
    let _ = read_full_box_header(&mut from)?;

    let mut handler = Vec::with_capacity(1);
    let mut primary_item = Vec::with_capacity(1);
    let mut item_locators = Vec::with_capacity(1);
    let mut item_infos = Vec::with_capacity(1);
    let mut item_props = Vec::with_capacity(1);

    while 0 != from.limit() {
        let child_header = read_header(&mut from)?;
        let mut child_data = (&mut from).take(child_header.data_size());
        match child_header.box_type {
            mpeg::HDLR => handler.push(parse_hdlr(&mut child_data)?),
            mpeg::PITM => primary_item.push(parse_pitm(&mut child_data)?),
            mpeg::ILOC => item_locators.push(parse_iloc(&mut child_data)?),
            mpeg::IINF => item_infos.push(parse_iinf(&mut child_data)?),
            mpeg::IPRP => item_props.push(iprp::parse_iprp(&mut child_data)?),
            _ => skip(&mut child_data)?,
        }

        ensure!(
            0 == child_data.limit(),
            "meta parser failed to parse a segment: {:?}",
            child_header
        );
    }

    Ok(RawMeta {
        handler,
        primary_item,
        item_locators,
        item_infos,
        item_props,
    })
}

pub fn parse_hdlr<R: Read>(mut from: &mut Take<R>) -> Result<FourCc, Error> {
    ensure!(from.limit() >= 4 + 4 + 4 + 12, "hdlr box is too small");
    let extended = read_full_box_header(&mut from)?;
    ensure!(
        0 == extended.version,
        "unsupported hdlr version: {}",
        extended.version
    );

    from.read_exact(&mut [0u8; 4])?;
    let ret = FourCc(from.read_u32::<BE>()?);
    let remaining = usize(from.limit());
    from.read_exact(&mut vec![0u8; remaining])?;
    Ok(ret)
}

pub fn parse_pitm<R: Read>(mut from: &mut Take<R>) -> Result<u16, Error> {
    let extended = read_full_box_header(&mut from)?;
    ensure!(
        0 == extended.version,
        "unsupported pitm version: {}",
        extended.version
    );
    Ok(from.read_u16::<BE>()?)
}

pub fn parse_iloc<R: Read>(mut from: &mut Take<R>) -> Result<Vec<ItemLoc>, Error> {
    let extended = read_full_box_header(&mut from)?;
    ensure!(
        extended.version <= 2,
        "unsupported iloc version: {}",
        extended.version
    );
    let (offset_size, length_size) = read_u4_pair(&mut from)?;
    let (base_offset_size, mut index_size) = read_u4_pair(&mut from)?;

    if 0 == extended.version {
        index_size = 0;
    }

    let item_count = if extended.version < 2 {
        u32(from.read_u16::<BE>()?)
    } else {
        from.read_u32::<BE>()?
    };

    let mut items = Vec::with_capacity(usize(item_count));

    for _ in 0..item_count {
        let id = if extended.version < 2 {
            u32(from.read_u16::<BE>()?)
        } else {
            from.read_u32::<BE>()?
        };

        if extended.version > 0 {
            let _reserved = from.read_u8()?;
            let (_, construction_method) = read_u4_pair(&mut from)?;
        }

        let data_reference_index = from.read_u16::<BE>()?;
        let base_offset = read_value_of_size(&mut from, base_offset_size)?;
        let extent_count = from.read_u16::<BE>()?;

        let mut extents = Vec::with_capacity(usize(extent_count));

        for _ in 0..extent_count {
            let index = if index_size > 0 {
                read_value_of_size(&mut from, index_size)?
            } else {
                0
            };
            let offset = read_value_of_size(&mut from, offset_size)?;
            let length = read_value_of_size(&mut from, length_size)?;
            extents.push(Extent {
                index,
                offset,
                length,
            })
        }

        items.push(ItemLoc {
            id,
            data_reference_index,
            base_offset,
            extents,
        })
    }

    Ok(items)
}

pub fn parse_iinf<R: Read>(mut from: &mut Take<R>) -> Result<Vec<ItemInfo>, Error> {
    let extended = read_full_box_header(&mut from)?;
    ensure!(
        0 == extended.version,
        "unsupported iinf version: {}",
        extended.version
    );
    let entry_count = from.read_u16::<BE>()?;

    let mut entries = Vec::with_capacity(usize(entry_count));

    for _ in 0..entry_count {
        let header = read_header(&mut from)?;
        ensure!(
            super::INFE == header.box_type,
            "unexpected iinf child: {:?}",
            header
        );

        let mut infe = io::BufReader::new(from.take(header.data_size()));

        let extended = read_full_box_header(&mut infe)?;
        ensure!(
            2 == extended.version,
            "unsupported infe version: {}",
            extended.version
        );

        let id = infe.read_u16::<BE>()?;
        let protection_index = infe.read_u16::<BE>()?;
        let item_type = FourCc(infe.read_u32::<BE>()?);
        let mut item_name = Vec::new();
        infe.read_until(0, &mut item_name)?;

        // TODO: presumably this doesn't actually work, due to BufReader
        ensure!(
            0 == infe.get_ref().limit(),
            "failed to consume entire infe box: {}",
            infe.get_ref().limit()
        );

        entries.push(ItemInfo {
            id,
            protection_index,
            item_type,
            item_name: String::from_utf8_lossy(&item_name).to_string(),
        });
    }

    Ok(entries)
}
