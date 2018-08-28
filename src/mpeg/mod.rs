use std::io::Read;
use std::io::Take;

use byteorder::ByteOrder;
use byteorder::ReadBytesExt;
use byteorder::BE;
use cast::u64;
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

#[derive(Clone, Debug)]
pub struct FileType {
    pub major_brand: BoxType,
    pub minor_version: u32,
    pub brands: Vec<BoxType>,
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

pub fn read_full_box_header<R: Read>(mut from: R) -> Result<(), Error> {
    from.read_u32::<BE>()?;
    Ok(())
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
