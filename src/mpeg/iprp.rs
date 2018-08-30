use std::io::Read;
use std::io::Take;

use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::u16;
use cast::u32;
use cast::usize;
use failure::Error;

use mpeg::pack_box_type;
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

pub fn parse_iprp<R: Read>(mut from: &mut Take<R>) -> Result<(), Error> {
    while 0 != from.limit() {
        let child_header = read_header(&mut from)?;
        println!("| | {}: {:?}", from.limit(), child_header);
        let mut child_data = (&mut from).take(child_header.data_size());
        if pack_box_type(*b"ipma") == child_header.box_type {
            println!("| | -> ipma: {:?}", parse_ipma(&mut child_data)?);
        } else if pack_box_type(*b"ipco") == child_header.box_type {
            println!("| | -> ipco: {:?}", parse_ipco(&mut child_data)?);
        } else {
            println!("| | .. unsupported");
            skip(&mut child_data)?;
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
        if pack_box_type(*b"ispe") == child_header.box_type {
            println!("| | | -> ispe: {:?}", parse_ispe(&mut child_data)?);
        } else {
            println!("| | | .. unsupported");
            skip(&mut child_data)?;
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
