#[macro_use]
extern crate failure;
extern crate heifers;

use std::env;
use std::fs;
use std::io;

use failure::Error;

fn main() -> Result<(), Error> {
    println!(
        "{:?}",
        heifers::Heif::new(io::BufReader::new(fs::File::open(
            env::args_os()
                .nth(1)
                .ok_or_else(|| format_err!("usage: filename"))?
        )?))?
    );
    Ok(())
}
