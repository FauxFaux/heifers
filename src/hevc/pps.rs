use bitreader::BitReader;
use failure::Error;

use hevc::read_uvlc;

bitflags! {
    #[derive(Default)]
    struct Flags: usize {
        const DEPENDENT_SLICE_SEGMENTS_ENABLED      = 1 <<  0;
        const OUTPUT_FLAG_PRESENT                   = 1 <<  1;
        const SIGN_DATA_HIDING_ENABLED              = 1 <<  2;
        const CABAC_INIT_PRESENT                    = 1 <<  3;
        const CONSTRAINED_INTRA_PRED                = 1 <<  4;
        const TRANSFORM_SKIP_ENABLED                = 1 <<  5;
        const CU_QP_DELTA_ENABLED                   = 1 <<  6;
        const PPS_SLICE_CHROMA_QP_OFFSETS_PRESENT   = 1 <<  7;
        const WEIGHTED_PRED                         = 1 <<  8;
        const WEIGHTED_BIPRED                       = 1 <<  9;
        const TRANSQUANT_BYPASS_ENABLED             = 1 << 10;
        const TILES_ENABLED                         = 1 << 11;
        const ENTROPY_CODING_SYNC_ENABLED           = 1 << 12;
        const PPS_LOOP_FILTER_ACROSS_SLICES_ENABLED = 1 << 13;
        const DEBLOCKING_FILTER_CONTROL_PRESENT     = 1 << 14;
    }
}

pub fn picture_parameter_set(from: &mut BitReader) -> Result<(), Error> {
    let mut flags = Flags::default();
    let id = read_uvlc(from)?;
    let id = read_uvlc(from)?;
    flags |= read_flag(from, Flags::DEPENDENT_SLICE_SEGMENTS_ENABLED)?;
    flags |= read_flag(from, Flags::OUTPUT_FLAG_PRESENT)?;
    let num_extra_slice_header_bits = from.read_u8(3)?;
    flags |= read_flag(from, Flags::SIGN_DATA_HIDING_ENABLED)?;
    flags |= read_flag(from, Flags::CABAC_INIT_PRESENT)?;
    let num_ref_idx_l0_default_active_minus1 = read_uvlc(from)?;
    let num_ref_idx_l1_default_active_minus1 = read_uvlc(from)?;

    // TODO: signed
    let init_qp_minus26 = read_uvlc(from)?;

    flags |= read_flag(from, Flags::CONSTRAINED_INTRA_PRED)?;
    flags |= read_flag(from, Flags::TRANSFORM_SKIP_ENABLED)?;

    let diff_cu_qp_delta_depth = if from.read_bool()? {
        flags |= Flags::CU_QP_DELTA_ENABLED;
        read_uvlc(from)?
    } else {
        0
    };

    let pps_cb_qp_offset = read_uvlc(from)?;
    let pps_cr_qp_offset = read_uvlc(from)?;

    flags |= read_flag(from, Flags::WEIGHTED_PRED)?;
    flags |= read_flag(from, Flags::WEIGHTED_BIPRED)?;
    flags |= read_flag(from, Flags::TRANSQUANT_BYPASS_ENABLED)?;
    flags |= read_flag(from, Flags::TILES_ENABLED)?;
    flags |= read_flag(from, Flags::ENTROPY_CODING_SYNC_ENABLED)?;

    ensure!(
        !flags.contains(Flags::TILES_ENABLED),
        "can't parse a 'tiles' configuration data right now"
    );

    flags |= read_flag(from, Flags::PPS_LOOP_FILTER_ACROSS_SLICES_ENABLED)?;
    flags |= read_flag(from, Flags::DEBLOCKING_FILTER_CONTROL_PRESENT)?;

    ensure!(
        !flags.contains(Flags::DEBLOCKING_FILTER_CONTROL_PRESENT),
        "can't parse a 'deblocking' configuration data right now"
    );

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

#[test]
fn pps() {
    picture_parameter_set(&mut BitReader::new(&[68, 1, 193, 114, 176, 98, 64])).unwrap();
}
