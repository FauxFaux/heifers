use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Take;

use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::usize;
use failure::Error;

use mpeg;
use mpeg::iprp;
use mpeg::read_full_box_header;
use mpeg::read_header;
use mpeg::read_u4_pair;
use mpeg::read_value_of_size;
use mpeg::Extent;
use mpeg::FourCc;
use mpeg::Item;
use mpeg::ItemInfo;
use mpeg::skip;

pub fn parse<R: Read>(mut from: &mut Take<R>) -> Result<(), Error> {
    let _ = read_full_box_header(&mut from)?;

    while 0 != from.limit() {
        let child_header = read_header(&mut from)?;
        println!("| {}: {:?}", from.limit(), child_header);
        let mut child_data = (&mut from).take(child_header.data_size());
        match child_header.box_type {
            mpeg::HDLR => println!("| -> hdlr: {:?}", parse_hdlr(&mut child_data)?),
            mpeg::PITM => println!("| -> pitm: {:?}", parse_pitm(&mut child_data)?),
            mpeg::ILOC => println!("| -> iloc: {:?}", parse_iloc(&mut child_data)?),
            mpeg::IINF => println!("| -> iinf: {:?}", parse_iinf(&mut child_data)?),
            mpeg::IPRP => println!("| -> iprp: {:?}", iprp::parse_iprp(&mut child_data)?),
            _ => skip(&mut child_data)?,
        }

        ensure!(
            0 == child_data.limit(),
            "meta parser failed to parse a segment: {:?}",
            child_header
        );
    }

    Ok(())
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

pub fn parse_iloc<R: Read>(mut from: &mut Take<R>) -> Result<Vec<Item>, Error> {
    let extended = read_full_box_header(&mut from)?;
    ensure!(
        0 == extended.version,
        "unsupported iloc version: {}",
        extended.version
    );
    let (offset_size, length_size) = read_u4_pair(&mut from)?;
    let (base_offset_size, _reserved) = read_u4_pair(&mut from)?;
    let item_count = from.read_u16::<BE>()?;

    let mut items = Vec::with_capacity(usize(item_count));

    for _ in 0..item_count {
        let id = from.read_u16::<BE>()?;
        let data_reference_index = from.read_u16::<BE>()?;
        let base_offset = read_value_of_size(&mut from, base_offset_size)?;
        let extent_count = from.read_u16::<BE>()?;

        let mut extents = Vec::with_capacity(usize(extent_count));

        for _ in 0..extent_count {
            let offset = read_value_of_size(&mut from, offset_size)?;
            let length = read_value_of_size(&mut from, length_size)?;
            extents.push(Extent { offset, length })
        }

        items.push(Item {
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
