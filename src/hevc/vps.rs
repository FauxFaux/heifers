use bitreader::BitReader;
use cast::usize;
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

    let vps_id = from.read_u8(4)?;
    let _reserved = from.read_u8(2)?;
    let max_layers_minus_1 = from.read_u8(6)?;
    flags |= read_flag(from, Flags::VPS_TEMPORAL_ID_NESTING)?;
    let _reserved = from.read_u16(16)?;
    profile_tier_level(from, max_layers_minus_1 + 1)?;
    Ok(())
}

pub fn profile_tier_level(from: &mut BitReader, max_sub_layers: u8) -> Result<(), Error> {
    let general_profile_space = from.read_u8(2)?;
    let general_tier_flag = from.read_bool()?;
    let general_profile_idc = from.read_u8(5)?;
    let general_profile_compatibility_flags = from.read_u32(32)?;
    let general_progressive_source_flag = from.read_bool()?;
    let general_interlaced_source_flag = from.read_bool()?;
    let general_non_packed_constraint_flag = from.read_bool()?;
    let general_frame_only_constraint_flag = from.read_bool()?;
    let _reserved = from.read_u64(44)?;
    let general_level_idc = from.read_u8(8)?;
    assert_eq!(60, general_level_idc);
    let mut sub_layer_profile_present_flag = vec![false; usize(max_sub_layers)];
    let mut sub_layer_level_present_flag = vec![false; usize(max_sub_layers)];
    for i in 0..max_sub_layers {
        sub_layer_profile_present_flag.push(from.read_bool()?);
        sub_layer_level_present_flag.push(from.read_bool()?);
    }
    if max_sub_layers > 0 {
        let _reserved = from.read_u16(16)?;
    }
    for i in 0..max_sub_layers {
        if sub_layer_profile_present_flag[usize(i)] {
            let sub_layer_profile_space = from.read_u8(2)?;
            let sub_layer_tier_flag = from.read_bool()?;
            let sub_layer_profile_idc = from.read_u8(5)?;
            let sub_layer_profile_compatibility_flags = from.read_u32(32)?;
            let sub_layer_progressive_source_flag = from.read_bool()?;
            let sub_layer_interlaced_source_flag = from.read_bool()?;
            let sub_layer_non_packed_constraint_flag = from.read_bool()?;
            let sub_layer_frame_only_constraint_flag = from.read_bool()?;
            let sub_layer_reserved_zero_44bits = from.read_u64(44)?;
        }
        if sub_layer_level_present_flag[usize(i)] {
            let sub_layer_level_idc = from.read_u8(8)?;
        }
    }

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
    use bitreader::BitReader;

    #[test]
    fn vps() {
        let bytes = [
            12, 1, 255, 255, 4, 8, 0, 0, 3, 0, 159, 168, 0, 0, 3, 0, 0, 60, 186, 2, 64,
        ];

        let mut reader = BitReader::new(&bytes);

        super::video_parameter_set(&mut reader).unwrap();
    }
}
