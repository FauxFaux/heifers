use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Take;

use byteorder::ByteOrder;
use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::u32;
use cast::u64;
use cast::u8;
use cast::usize;
use failure::Error;

use std::fmt;
use HeifError;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct BoxType(u32);

#[derive(Copy, Clone, Debug)]
pub struct BoxHeader {
    pub box_type: BoxType,
    pub size: u64,
    pub offset: u8,
}

pub struct ExtendedHeader {
    pub version: u8,
    pub flags: u32,
}

#[derive(Clone, Debug)]
pub struct FileType {
    pub major_brand: BoxType,
    pub minor_version: u32,
    pub brands: Vec<BoxType>,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub data_reference_index: u16,
    pub base_offset: u64,
    pub extents: Vec<Extent>,
}

#[derive(Copy, Clone, Debug)]
pub struct Extent {
    pub offset: u64,
    pub length: u64,
}

#[derive(Clone, Debug)]
pub struct ItemInfo {
    id: u16,
    protection_index: u16,
    item_type: BoxType,
    item_name: String,
}

impl BoxHeader {
    pub fn data_size(&self) -> u64 {
        self.size - u64(self.offset)
    }
}

pub fn read_header<R: Read>(mut from: R) -> Result<BoxHeader, Error> {
    let size_low = from.read_u32::<BE>()?;
    let box_type = BoxType(from.read_u32::<BE>()?);

    let (offset, size) = match size_low {
        1 => {
            let size_high = from.read_u64::<BE>()?;
            ensure!(
                size_high >= 16,
                "second-order size: {} must be >= 16",
                size_high
            );
            (16, size_high)
        }
        0 | 2...7 => bail!("unsupported box length: {}", size_low),
        other => (8, u64(other)),
    };

    Ok(BoxHeader {
        box_type,
        size,
        offset,
    })
}

pub fn read_full_box_header<R: Read>(mut from: R) -> Result<ExtendedHeader, Error> {
    let data = from.read_u32::<BE>()?;
    Ok(ExtendedHeader {
        version: u8(data >> 24)?,
        flags: data & 0x00ff_ffff,
    })
}

pub fn parse_ftyp<R: Read>(mut from: Take<R>) -> Result<FileType, Error> {
    let major_brand = BoxType(from.read_u32::<BE>()?);
    let minor_version = from.read_u32::<BE>()?;
    let remaining = from.limit();
    ensure!(0 == remaining % 4, "invalid brand list in 'ftyp'");
    let brand_names = usize(remaining / 4);
    let mut brands = Vec::with_capacity(brand_names);
    for _ in 0..brand_names {
        brands.push(BoxType(from.read_u32::<BE>()?));
    }

    assert_eq!(0, from.limit());

    Ok(FileType {
        major_brand,
        minor_version,
        brands,
    })
}

pub fn parse_hdlr<R: Read>(mut from: &mut Take<R>) -> Result<BoxType, Error> {
    ensure!(from.limit() >= 4 + 4 + 4 + 12, "hdlr box is too small");
    let extended = read_full_box_header(&mut from)?;
    ensure!(
        0 == extended.version,
        "unsupported hdlr version: {}",
        extended.version
    );

    from.read_exact(&mut [0u8; 4])?;
    let ret = BoxType(from.read_u32::<BE>()?);
    let remaining = usize(from.limit());
    println!("{}", remaining);
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
            pack_box_type(*b"infe") == header.box_type,
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
        let item_type = BoxType(infe.read_u32::<BE>()?);
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

fn read_u4_pair<R: Read>(mut from: R) -> Result<(u8, u8), Error> {
    let byte = from.read_u8()?;
    Ok(((byte >> 4) & 0xf, byte & 0xf))
}

fn read_value_of_size<R: Read>(mut from: R, bytes: u8) -> Result<u64, Error> {
    Ok(match bytes {
        4 => u64(from.read_u32::<BE>()?),
        8 => from.read_u64::<BE>()?,
        other => bail!("unsupported size: {}", other),
    })
}

impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0u8; 4];
        BE::write_u32(&mut buf, self.0);
        write!(f, "{:?}", String::from_utf8_lossy(&buf))
    }
}

#[inline]
pub fn pack_box_type(str: [u8; 4]) -> BoxType {
    BoxType(BE::read_u32(&str))
}
