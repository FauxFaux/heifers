#[macro_use]
extern crate failure;
extern crate heifers;

use std::env;
use std::fs;
use std::io;

use failure::Error;

fn main() -> Result<(), Error> {
    let mut src = io::BufReader::new(fs::File::open(
        env::args_os()
            .nth(1)
            .ok_or_else(|| format_err!("usage: filename"))?,
    )?);
    let file = heifers::Heif::new(&mut src)?;

    file.bit_stream(1, &mut src, io::stdout())?;

    Ok(())
}
