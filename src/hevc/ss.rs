use bitreader::BitReader;
use cast::u16;
use cast::u64;
use cast::u8;
use cast::usize;
use failure::Error;

use hevc::pps;
use hevc::pps::PicParamSet;
use hevc::read_uvlc;
use hevc::sps;
use hevc::sps::SeqParamSet;

const SLICE_TYPE_B: u8 = 0;
const SLICE_TYPE_P: u8 = 1;
const SLICE_TYPE_I: u8 = 2;

bitflags! {
    #[derive(Default)]
    pub struct Flags: u32 {
        const FIRST_SLICE_SEGMENT_IN_PIC              = 1 <<  0;
        const NO_OUTPUT_OF_PRIOR_PICS                 = 1 <<  1;
        const DEPENDENT_SLICE_SEGMENT                 = 1 <<  2;
        const PIC_OUTPUT                              = 1 <<  3;
        const SHORT_TERM_REF_PIC_SET_SPS              = 1 <<  4;
        const USED_BY_CURR_PIC_LT                     = 1 <<  5;
        const DELTA_POC_MSB_PRESENT                   = 1 <<  6;
        const SLICE_TEMPORAL_MVP_ENABLED              = 1 <<  7;
        const SLICE_SAO_LUMA                          = 1 <<  8;
        const SLICE_SAO_CHROMA                        = 1 <<  9;
        const NUM_REF_IDX_ACTIVE_OVERRIDE             = 1 << 10;
        const MVD_L1_ZERO                             = 1 << 11;
        const CABAC_INIT                              = 1 << 12;
        const COLLOCATED_FROM_L0                      = 1 << 13;
        const DEBLOCKING_FILTER_OVERRIDE              = 1 << 14;
        const SLICE_DEBLOCKING_FILTER_DISABLED        = 1 << 15;
        const SLICE_LOOP_FILTER_ACROSS_SLICES_ENABLED = 1 << 16;
    }
}

pub struct SliceSegmentHeader {
    pub flags: Flags,
}

pub fn slice_segment_header(
    nal_unit_type: u8,
    from: &mut BitReader,
    pps: &PicParamSet,
    sps: &SeqParamSet,
) -> Result<SliceSegmentHeader, Error> {
    let mut flags = Flags::default();

    flags |= read_flag(from, Flags::FIRST_SLICE_SEGMENT_IN_PIC)?;

    if nal_unit_type >= super::NAL_BLA_W_LP && nal_unit_type <= super::NAL_RSV_IRAP_VCL23 {
        flags |= read_flag(from, Flags::NO_OUTPUT_OF_PRIOR_PICS)?;
    }

    let slice_pic_parameter_set_id = read_uvlc(from)?;
    if !flags.contains(Flags::FIRST_SLICE_SEGMENT_IN_PIC) {
        if pps
            .flags
            .contains(pps::Flags::DEPENDENT_SLICE_SEGMENTS_ENABLED)
        {
            flags |= read_flag(from, Flags::DEPENDENT_SLICE_SEGMENT)?;
        }
        let slice_segment_address = read_uvlc(from)?;
    }

    if !flags.contains(Flags::DEPENDENT_SLICE_SEGMENT) {
        let _slice_reserved_flag = from.read_u64(pps.num_extra_slice_header_bits)?;
        let slice_type = {
            let val = read_uvlc(from)?;
            ensure!(val < 3, "invalid slice type: {}", val);
            u8(val).unwrap()
        };
        if pps.flags.contains(pps::Flags::OUTPUT_FLAG_PRESENT) {
            flags |= read_flag(from, Flags::PIC_OUTPUT)?;
        }
        if sps.flags.contains(sps::Flags::SEPARATE_COLOUR_PLANE) {
            let colour_plane_id = from.read_u8(2)?;
        }
        if nal_unit_type != super::NAL_IDR_W_RADL && nal_unit_type != super::NAL_IDR_N_LP {
            let slice_pic_order_cnt_lsb =
                from.read_u64(sps.log2_max_pic_order_cnt_lsb_minus4 + 4)?;
            flags |= read_flag(from, Flags::SHORT_TERM_REF_PIC_SET_SPS)?;
            if !flags.contains(Flags::SHORT_TERM_REF_PIC_SET_SPS) {
                bail!("short_term_ref_pic_set(num_short_term_ref_pic_sets)");
            } else if sps.num_short_term_ref_pic_sets > 1 {
                bail!("short_term_ref_pic_set_idx u(v)")
            }
            let mut num_long_term_sps = 0u8;
            if sps.flags.contains(sps::Flags::LONG_TERM_REF_PICS_PRESENT) {
                if sps.num_long_term_ref_pics_sps > 0 {
                    num_long_term_sps = {
                        let val = read_uvlc(from)?;
                        ensure!(
                            val <= u64(sps.num_long_term_ref_pics_sps),
                            "num_long_term_sps out of range: {}",
                            val
                        );
                        u8(val).expect("sps.num.. is u8")
                    };
                }
                let num_long_term_pics = {
                    let val = read_uvlc(from)?;
                    u8(val).map_err(|_| {
                        format_err!(
                            "implementation limitation: num_long_term_pics must be <=255, not {}",
                            val
                        )
                    })?
                };
                let some_num_records = (u16(num_long_term_sps) + u16(num_long_term_pics));
                let mut used_by_curr_pic_lt_flag = vec![false; usize(some_num_records)];
                let mut delta_poc_msb_present_flag = vec![false; usize(some_num_records)];
                let mut delta_poc_msb_cycle_lt = vec![0u64; usize(some_num_records)];
                for i in 0..some_num_records {
                    if i < u16(num_long_term_sps) {
                        if sps.num_long_term_ref_pics_sps > 1 {
                            bail!("lt_idx_sps[i] u(v)")
                        }
                    } else {
                        bail!("poc_lsb_lt[ i ] u(v)");
                        used_by_curr_pic_lt_flag[usize(i)] = from.read_bool()?;
                    }
                    delta_poc_msb_present_flag[usize(i)] = from.read_bool()?;
                    if delta_poc_msb_present_flag[usize(i)] {
                        delta_poc_msb_cycle_lt[usize(i)] = read_uvlc(from)?;
                    }
                }
            }
            if sps.flags.contains(sps::Flags::SPS_TEMPORAL_MVP_ENABLED) {
                flags |= read_flag(from, Flags::SLICE_TEMPORAL_MVP_ENABLED)?;
            }
        }
        if sps
            .flags
            .contains(sps::Flags::SAMPLE_ADAPTIVE_OFFSET_ENABLED)
        {
            flags |= read_flag(from, Flags::SLICE_SAO_LUMA)?;
            flags |= read_flag(from, Flags::SLICE_SAO_CHROMA)?;
        }

        if slice_type == SLICE_TYPE_P || slice_type == SLICE_TYPE_B {
            let mut num_ref_idx_l0_active_minus1 = unimplemented!("default value");
            let mut num_ref_idx_l1_active_minus1 = unimplemented!("default value");
            flags |= read_flag(from, Flags::NUM_REF_IDX_ACTIVE_OVERRIDE)?;
            if flags.contains(Flags::NUM_REF_IDX_ACTIVE_OVERRIDE) {
                num_ref_idx_l0_active_minus1 = read_uvlc(from)?;
                if slice_type == SLICE_TYPE_B {
                    num_ref_idx_l1_active_minus1 = read_uvlc(from)?;
                }
            }
            if pps.flags.contains(pps::Flags::LISTS_MODIFICATION_PRESENT)
                && bail!("NumPocTotalCurr > 1")
            {
                bail!("ref_pic_lists_modification()")
            }
            if slice_type == SLICE_TYPE_B {
                flags |= read_flag(from, Flags::MVD_L1_ZERO)?;
            }
            if pps.flags.contains(pps::Flags::CABAC_INIT_PRESENT) {
                flags |= read_flag(from, Flags::CABAC_INIT)?;
            }

            let mut collocated_from_l0_flag = false;
            if flags.contains(Flags::SLICE_TEMPORAL_MVP_ENABLED) {
                if slice_type == SLICE_TYPE_B {
                    collocated_from_l0_flag = from.read_bool()?;
                }
                if (collocated_from_l0_flag && num_ref_idx_l0_active_minus1 > 0)
                    || (!collocated_from_l0_flag && num_ref_idx_l1_active_minus1 > 0)
                {
                    let collocated_ref_idx = read_uvlc(from)?;
                }
            }
            if (pps.flags.contains(pps::Flags::WEIGHTED_PRED) && slice_type == SLICE_TYPE_P)
                || (pps.flags.contains(pps::Flags::WEIGHTED_BIPRED) && slice_type == SLICE_TYPE_B)
            {
                bail!("pred_weight_table()")
            }
            let five_minus_max_num_merge_cand = read_uvlc(from)?;
        }
        let slice_qp_delta = read_uvlc(from)?; // TODO: signed
        if pps
            .flags
            .contains(pps::Flags::PPS_SLICE_CHROMA_QP_OFFSETS_PRESENT)
        {
            let slice_cb_qp_offset = read_uvlc(from)?; // TODO: signed
            let slice_cr_qp_offset = read_uvlc(from)?; // TODO: signed
        }

        if pps
            .flags
            .contains(pps::Flags::DEBLOCKING_FILTER_OVERRIDE_ENABLED)
        {
            flags |= read_flag(from, Flags::DEBLOCKING_FILTER_OVERRIDE)?;
        }

        if flags.contains(Flags::DEBLOCKING_FILTER_OVERRIDE) {
            flags |= read_flag(from, Flags::SLICE_DEBLOCKING_FILTER_DISABLED)?;
            if !flags.contains(Flags::SLICE_DEBLOCKING_FILTER_DISABLED) {
                let slice_beta_offset_div2 = read_uvlc(from)?; // TODO: signed
                let slice_tc_offset_div2 = read_uvlc(from)?; // TODO: signed
            }
        }
        if pps
            .flags
            .contains(pps::Flags::PPS_LOOP_FILTER_ACROSS_SLICES_ENABLED)
            && (flags.contains(Flags::SLICE_SAO_LUMA)
                || flags.contains(Flags::SLICE_SAO_CHROMA)
                || !flags.contains(Flags::SLICE_DEBLOCKING_FILTER_DISABLED))
        {
            flags |= read_flag(from, Flags::SLICE_LOOP_FILTER_ACROSS_SLICES_ENABLED)?;
        }
    }

    if pps.flags.contains(pps::Flags::TILES_ENABLED)
        || pps.flags.contains(pps::Flags::ENTROPY_CODING_SYNC_ENABLED)
    {
        let num_entry_point_offsets = read_uvlc(from)?;
        if num_entry_point_offsets > 0 {
            let offset_len = read_uvlc(from)? + 1;

            ensure!(offset_len <= 32, "offset_len too long: {}", offset_len);

            for i in 0..num_entry_point_offsets {
                let entry_point_offset_minus1 = from.read_u32(u8(offset_len).unwrap())?;
            }
        }
    }
    if pps
        .flags
        .contains(pps::Flags::SLICE_SEGMENT_HEADER_EXTENSION_PRESENT)
    {
        let slice_segment_header_extension_length = read_uvlc(from)?;
        for i in 0..slice_segment_header_extension_length {
            bail!("slice_segment_header_extension_data_byte[i] u(8)")
        }
    }

    byte_alignment(from)?;

    Ok(SliceSegmentHeader { flags })
}

fn byte_alignment(from: &mut BitReader) -> Result<(), Error> {
    ensure!(from.read_bool()?, "byte_alignment requires high bit");
    while !from.is_aligned(1) {
        ensure!(!from.read_bool()?, "byte_alignment requires low bit");
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
