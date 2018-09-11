extern crate cast;
extern crate failure;
extern crate heifers;

use std::io::Cursor;
use std::io::Read;

use failure::Error;

#[test]
fn road() -> Result<(), Error> {
    let bytes = &include_bytes!("data/road.heic")[..];
    let mut file = Cursor::new(bytes);
    let heif = heifers::Heif::new(&mut file)?;
    println!("{:?}", heif);
    let item = heif.primary_item_id();

    let mut data = Vec::new();
    heif.open_item_data(file, item)?.read_to_end(&mut data)?;

    assert_eq!(&bytes[333..], data.as_slice());

    let pps = heif.find_pps(item)?;

    println!("{:?}", heifers::hevc::dump(Cursor::new(data), &pps)?);
    Ok(())
}
