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

pub fn slice_segment_header(
    nal_unit_type: u8,
    from: &mut BitReader,
    pps: &PicParamSet,
    sps: &SeqParamSet,
) -> Result<(), Error> {
    let first_slice_segment_in_pic_flag = from.read_bool()?;

    if nal_unit_type >= super::NAL_BLA_W_LP && nal_unit_type <= super::NAL_RSV_IRAP_VCL23 {
        let no_output_of_prior_pics_flag = from.read_bool()?;
    }

    let mut dependent_slice_segment_flag = false;
    let slice_pic_parameter_set_id = read_uvlc(from)?;
    if !first_slice_segment_in_pic_flag {
        if pps
            .flags
            .contains(pps::Flags::DEPENDENT_SLICE_SEGMENTS_ENABLED)
        {
            dependent_slice_segment_flag = from.read_bool()?;
        }
        let slice_segment_address = read_uvlc(from)?;
    }

    if !dependent_slice_segment_flag {
        let _slice_reserved_flag = from.read_u64(pps.num_extra_slice_header_bits)?;
        let slice_type = {
            let val = read_uvlc(from)?;
            ensure!(val < 3, "invalid slice type: {}", val);
            u8(val).unwrap()
        };
        if pps.flags.contains(pps::Flags::OUTPUT_FLAG_PRESENT) {
            let pic_output_flag = from.read_bool()?;
        }
        if sps.flags.contains(sps::Flags::SEPARATE_COLOUR_PLANE) {
            let colour_plane_id = from.read_u8(2)?;
        }
        let mut slice_temporal_mvp_enabled_flag = false;
        if nal_unit_type != super::NAL_IDR_W_RADL && nal_unit_type != super::NAL_IDR_N_LP {
            let slice_pic_order_cnt_lsb =
                from.read_u64(sps.log2_max_pic_order_cnt_lsb_minus4 + 4)?;
            let short_term_ref_pic_set_sps_flag = from.read_bool()?;
            if !short_term_ref_pic_set_sps_flag {
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
                slice_temporal_mvp_enabled_flag = from.read_bool()?;
            }
        }
        let mut slice_sao_luma_flag = false;
        let mut slice_sao_chroma_flag = false;
        if sps
            .flags
            .contains(sps::Flags::SAMPLE_ADAPTIVE_OFFSET_ENABLED)
        {
            slice_sao_luma_flag = from.read_bool()?;
            slice_sao_chroma_flag = from.read_bool()?;
        }

        if slice_type == SLICE_TYPE_P || slice_type == SLICE_TYPE_B {
            let mut num_ref_idx_l0_active_minus1 = unimplemented!("default value");
            let mut num_ref_idx_l1_active_minus1 = unimplemented!("default value");
            let num_ref_idx_active_override_flag = from.read_bool()?;
            if num_ref_idx_active_override_flag {
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
                let mvd_l1_zero_flag = from.read_bool()?;
            }
            if pps.flags.contains(pps::Flags::CABAC_INIT_PRESENT) {
                let cabac_init_flag = from.read_bool()?;
            }

            let mut collocated_from_l0_flag = false;
            if slice_temporal_mvp_enabled_flag {
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

        let mut deblocking_filter_override_flag = false;
        if pps
            .flags
            .contains(pps::Flags::DEBLOCKING_FILTER_OVERRIDE_ENABLED)
        {
            deblocking_filter_override_flag = from.read_bool()?;
        }

        let mut slice_deblocking_filter_disabled_flag = false;
        if deblocking_filter_override_flag {
            slice_deblocking_filter_disabled_flag = from.read_bool()?;
            if !slice_deblocking_filter_disabled_flag {
                let slice_beta_offset_div2 = read_uvlc(from)?; // TODO: signed
                let slice_tc_offset_div2 = read_uvlc(from)?; // TODO: signed
            }
        }
        if pps
            .flags
            .contains(pps::Flags::PPS_LOOP_FILTER_ACROSS_SLICES_ENABLED)
            && (slice_sao_luma_flag
                || slice_sao_chroma_flag
                || !slice_deblocking_filter_disabled_flag)
        {
            let slice_loop_filter_across_slices_enabled_flag = from.read_bool()?;
        }
    }

    if pps.flags.contains(pps::Flags::TILES_ENABLED)
        || pps.flags.contains(pps::Flags::ENTROPY_CODING_SYNC_ENABLED)
    {
        let num_entry_point_offsets = read_uvlc(from)?;
        if num_entry_point_offsets > 0 {
            let offset_len_minus1 = read_uvlc(from)?;
            for i in 0..num_entry_point_offsets {
                bail!("entry_point_offset_minus1[i] u(v)")
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

    bail!("byte_alignment()");

    Ok(())
}
