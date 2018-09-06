use bitreader::BitReader;
use failure::Error;

use hevc::rbsp_trailing_bits;
use hevc::read_uvlc;

bitflags! {
    #[derive(Default)]
    struct Flags: u32 {
        const DEPENDENT_SLICE_SEGMENTS_ENABLED       = 1 <<  0;
        const OUTPUT_FLAG_PRESENT                    = 1 <<  1;
        const SIGN_DATA_HIDING_ENABLED               = 1 <<  2;
        const CABAC_INIT_PRESENT                     = 1 <<  3;
        const CONSTRAINED_INTRA_PRED                 = 1 <<  4;
        const TRANSFORM_SKIP_ENABLED                 = 1 <<  5;
        const CU_QP_DELTA_ENABLED                    = 1 <<  6;
        const PPS_SLICE_CHROMA_QP_OFFSETS_PRESENT    = 1 <<  7;
        const WEIGHTED_PRED                          = 1 <<  8;
        const WEIGHTED_BIPRED                        = 1 <<  9;
        const TRANSQUANT_BYPASS_ENABLED              = 1 << 10;
        const TILES_ENABLED                          = 1 << 11;
        const ENTROPY_CODING_SYNC_ENABLED            = 1 << 12;
        const PPS_LOOP_FILTER_ACROSS_SLICES_ENABLED  = 1 << 13;
        const DEBLOCKING_FILTER_CONTROL_PRESENT      = 1 << 14;
        const UNIFORM_SPACING                        = 1 << 15;
        const LOOP_FILTER_ACROSS_TILES_ENABLED       = 1 << 16;
        const DEBLOCKING_FILTER_OVERRIDE_ENABLED     = 1 << 17;
        const PPS_DEBLOCKING_FILTER_DISABLED         = 1 << 18;
        const PPS_SCALING_LIST_DATA_PRESENT          = 1 << 19;
        const LISTS_MODIFICATION_PRESENT             = 1 << 20;
        const SLICE_SEGMENT_HEADER_EXTENSION_PRESENT = 1 << 21;
        const PPS_EXTENSION                          = 1 << 22;
    }
}

pub fn picture_parameter_set(from: &mut BitReader) -> Result<(), Error> {
    let mut flags = Flags::default();

    let pps_pic_parameter_set_id = read_uvlc(from)?;
    let pps_seq_parameter_set_id = read_uvlc(from)?;
    flags |= read_flag(from, Flags::DEPENDENT_SLICE_SEGMENTS_ENABLED)?;
    flags |= read_flag(from, Flags::OUTPUT_FLAG_PRESENT)?;
    let num_extra_slice_header_bits = from.read_u8(3)?;
    flags |= read_flag(from, Flags::SIGN_DATA_HIDING_ENABLED)?;
    flags |= read_flag(from, Flags::CABAC_INIT_PRESENT)?;
    let num_ref_idx_l0_default_active_minus1 = read_uvlc(from)?;
    let num_ref_idx_l1_default_active_minus1 = read_uvlc(from)?;
    let init_qp_minus26 = read_uvlc(from)?; // TODO: signed
    flags |= read_flag(from, Flags::CONSTRAINED_INTRA_PRED)?;
    flags |= read_flag(from, Flags::TRANSFORM_SKIP_ENABLED)?;
    let diff_cu_qp_delta_depth = if from.read_bool()? {
        flags |= Flags::CU_QP_DELTA_ENABLED;
        read_uvlc(from)?
    } else {
        0
    };
    let pps_cb_qp_offset = read_uvlc(from)?; // TODO: signed
    let pps_cr_qp_offset = read_uvlc(from)?; // TODO: signed
    flags |= read_flag(from, Flags::PPS_SLICE_CHROMA_QP_OFFSETS_PRESENT)?;
    flags |= read_flag(from, Flags::WEIGHTED_PRED)?;
    flags |= read_flag(from, Flags::WEIGHTED_BIPRED)?;
    flags |= read_flag(from, Flags::TRANSQUANT_BYPASS_ENABLED)?;
    flags |= read_flag(from, Flags::TILES_ENABLED)?;
    flags |= read_flag(from, Flags::ENTROPY_CODING_SYNC_ENABLED)?;
    if flags.contains(Flags::TILES_ENABLED) {
        let num_tile_columns_minus1 = read_uvlc(from)?;
        let num_tile_rows_minus1 = read_uvlc(from)?;
        flags |= read_flag(from, Flags::UNIFORM_SPACING)?;

        if !flags.contains(Flags::UNIFORM_SPACING) {
            for _ in 0..num_tile_columns_minus1 {
                read_uvlc(from)?;
            }

            for _ in 0..num_tile_rows_minus1 {
                read_uvlc(from)?;
            }
        }

        flags |= read_flag(from, Flags::LOOP_FILTER_ACROSS_TILES_ENABLED)?;
    }
    flags |= read_flag(from, Flags::PPS_LOOP_FILTER_ACROSS_SLICES_ENABLED)?;
    flags |= read_flag(from, Flags::DEBLOCKING_FILTER_CONTROL_PRESENT)?;
    if flags.contains(Flags::DEBLOCKING_FILTER_CONTROL_PRESENT) {
        flags |= read_flag(from, Flags::DEBLOCKING_FILTER_OVERRIDE_ENABLED)?;
        flags |= read_flag(from, Flags::PPS_DEBLOCKING_FILTER_DISABLED)?;
        if !flags.contains(Flags::PPS_DEBLOCKING_FILTER_DISABLED) {
            let pps_beta_offset_div2 = read_uvlc(from)?; // TODO: signed
            let pps_tc_offset_div2 = read_uvlc(from)?; // TODO: signed
        }
    }
    flags |= read_flag(from, Flags::PPS_SCALING_LIST_DATA_PRESENT)?;
    ensure!(
        !flags.contains(Flags::PPS_SCALING_LIST_DATA_PRESENT),
        "can't handle scaling lists"
    );
    flags |= read_flag(from, Flags::LISTS_MODIFICATION_PRESENT)?;
    let log2_parallel_merge_level_minus2 = read_uvlc(from)?;
    flags |= read_flag(from, Flags::SLICE_SEGMENT_HEADER_EXTENSION_PRESENT)?;
    flags |= read_flag(from, Flags::PPS_EXTENSION)?;
    ensure!(
        !flags.contains(Flags::PPS_EXTENSION),
        "can't handle pps extensions"
    );

    rbsp_trailing_bits(from)?;

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
    use cast::u64;

    #[test]
    fn pps() {
        let bytes = [193, 114, 176, 98, 64];

        let mut reader = BitReader::new(&bytes);

        super::picture_parameter_set(&mut reader).unwrap();
        assert_eq!(u64(bytes.len()) * 8, reader.position());
    }
}
