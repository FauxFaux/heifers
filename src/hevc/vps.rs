use bitreader::BitReader;
use failure::Error;

use hevc::rbsp_trailing_bits;
use hevc::read_uvlc;

bitflags! {
    #[derive(Default)]
    struct Flags: u32 {
        const VPS_TEMPORAL_ID_NESTING       = 1 <<  0;

    }
}

pub fn video_parameter_set(from: &mut BitReader) -> Result<(), Error> {
    let mut flags = Flags::default();

    Ok(())
}

#[inline]
fn read_flag(from: &mut BitReader, flag: Flags) -> Result<Flags, Error> {
    Ok(if from.read_bool()? {
        flag
    } else {
        Flags::default()
    })
}

#[cfg(test)]
mod tests {
    extern crate hexdump;

    use bitreader::BitReader;

    #[test]
    fn vps() {
        let bytes = [12, 1, 255, 255, 4, 8, 0, 0, 3, 0, 159, 168, 0, 0, 3, 0, 0, 60, 186, 2, 64];

        let mut reader = BitReader::new(&bytes);

        super::video_parameter_set(&mut reader).unwrap();
    }
}
