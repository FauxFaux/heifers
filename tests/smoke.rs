extern crate cast;
extern crate failure;
extern crate heifers;

use std::io::Cursor;

use failure::Error;

#[test]
fn road() -> Result<(), Error> {
    println!(
        "{:?}",
        heifers::Heif::new(Cursor::new(&include_bytes!("data/road.heic")[..]))?
    );
    Ok(())
}
