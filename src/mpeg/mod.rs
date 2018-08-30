use std::fmt;
use std::io::Read;
use std::io::Take;

use byteorder::ByteOrder;
use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::u64;
use cast::u8;
use cast::usize;
use failure::Error;

pub mod iprp;
pub mod meta;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct FourCc(u32);

pub const FTYP: FourCc = FourCc(0x66747970); // ftyp
pub const HDLR: FourCc = FourCc(0x68646c72); // hdlr
pub const HEIC: FourCc = FourCc(0x68656963); // heic
pub const HVC1: FourCc = FourCc(0x68766331); // hvc1
pub const HVCC: FourCc = FourCc(0x68766343); // hvcC
pub const IINF: FourCc = FourCc(0x69696e66); // iinf
pub const ILOC: FourCc = FourCc(0x696c6f63); // iloc
pub const INFE: FourCc = FourCc(0x696e6665); // infe
pub const IPCO: FourCc = FourCc(0x6970636f); // ipco
pub const IPMA: FourCc = FourCc(0x69706d61); // ipma
pub const IPRP: FourCc = FourCc(0x69707270); // iprp
pub const IPSE: FourCc = FourCc(0x69707365); // ipse
pub const META: FourCc = FourCc(0x6d657461); // meta
pub const MDAT: FourCc = FourCc(0x6d646174); // mdat
pub const MOOV: FourCc = FourCc(0x6d6f6f76); // moov
pub const PITM: FourCc = FourCc(0x7069746d); // pitm

#[derive(Copy, Clone, Debug)]
pub struct BoxHeader {
    pub box_type: FourCc,
    pub size: u64,
    pub offset: u8,
}

pub struct ExtendedHeader {
    pub version: u8,
    pub flags: u32,
}

#[derive(Clone, Debug)]
pub struct FileType {
    pub major_brand: FourCc,
    pub minor_version: u32,
    pub brands: Vec<FourCc>,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,
    pub data_reference_index: u16,
    pub base_offset: u64,
    pub extents: Vec<Extent>,
}

#[derive(Copy, Clone, Debug)]
pub struct Extent {
    pub index: u64,
    pub offset: u64,
    pub length: u64,
}

#[derive(Clone, Debug)]
pub struct ItemInfo {
    id: u16,
    protection_index: u16,
    item_type: FourCc,
    item_name: String,
}

impl BoxHeader {
    pub fn data_size(&self) -> u64 {
        self.size - u64(self.offset)
    }
}

pub fn read_header<R: Read>(mut from: R) -> Result<BoxHeader, Error> {
    let size_low = from.read_u32::<BE>()?;
    let box_type = FourCc(from.read_u32::<BE>()?);

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

fn read_full_box_header<R: Read>(mut from: R) -> Result<ExtendedHeader, Error> {
    let data = from.read_u32::<BE>()?;
    Ok(ExtendedHeader {
        version: u8(data >> 24)?,
        flags: data & 0x00ff_ffff,
    })
}

pub fn parse_ftyp<R: Read>(from: &mut Take<R>) -> Result<FileType, Error> {
    let major_brand = FourCc(from.read_u32::<BE>()?);
    let minor_version = from.read_u32::<BE>()?;
    let remaining = from.limit();
    ensure!(0 == remaining % 4, "invalid brand list in 'ftyp'");
    let brand_names = usize(remaining / 4);
    let mut brands = Vec::with_capacity(brand_names);
    for _ in 0..brand_names {
        brands.push(FourCc(from.read_u32::<BE>()?));
    }

    assert_eq!(0, from.limit());

    Ok(FileType {
        major_brand,
        minor_version,
        brands,
    })
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

pub fn skip_box<R: Read>(mut from: R, header: &BoxHeader) -> Result<(), Error> {
    skip(&mut (&mut from).take(header.data_size()))
}

fn skip<R: Read>(child_data: &mut Take<R>) -> Result<(), Error> {
    let remaining = usize(child_data.limit());
    // TODO: don't have unbounded allocation here
    child_data.read_exact(&mut vec![0u8; remaining])?;
    Ok(())
}

impl fmt::Debug for FourCc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0u8; 4];
        BE::write_u32(&mut buf, self.0);
        write!(f, "{:?}", String::from_utf8_lossy(&buf))
    }
}

#[cfg(test)]
mod tests {
    fn pack_fourcc(str: &[u8]) -> u32 {
        use byteorder::ByteOrder;
        use byteorder::BE;
        BE::read_u32(&str)
    }

    #[test]
    fn packing_fourcc() {
        for key in &[
            "ftyp", "hdlr", "heic", "hvc1", "hvcC", "iinf", "iloc", "infe", "ipco", "ipma", "iprp",
            "ipse", "meta", "mdat", "moov", "pitm",
        ] {
            println!(
                "pub const {}: FourCc = FourCc(0x{:08x}); // {}",
                key.to_ascii_uppercase(),
                pack_fourcc(key.as_ref()),
                key
            );
        }
    }
}
