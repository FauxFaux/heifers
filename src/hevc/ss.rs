use bitreader::BitReader;
use failure::Error;

use hevc::pps;
use hevc::pps::PicParamSet;
use hevc::read_uvlc;
use hevc::sps;
use hevc::sps::SeqParamSet;

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
        let slice_type = read_uvlc(from)?;
        if pps.flags.contains(pps::Flags::OUTPUT_FLAG_PRESENT) {
            let pic_output_flag = from.read_bool()?;
        }
        if pps.flags.contains(pps::Flags::SEPARATE_COLOUR_PLANE) {
            let colour_plane_id = from.read_u8(2)?;
        }
        if nal_unit_type != super::NAL_IDR_W_RADL && nal_unit_type != super::NAL_IDR_N_LP {
            let slice_pic_order_cnt_lsb =
                from.read_u64(sps.log2_max_pic_order_cnt_lsb_minus4 + 4)?;
            let short_term_ref_pic_set_sps_flag = from.read_bool()?;
            if !short_term_ref_pic_set_sps_flag {
                bail!("short_term_ref_pic_set(num_short_term_ref_pic_sets)");
            } else if sps.num_short_term_ref_pic_sets > 1 {
                bail!("short_term_ref_pic_set_idx u(v)")
            }
            if sps.flags.contains(sps::Flags::LONG_TERM_REF_PICS_PRESENT) {
                if num_long_term_ref_pics_sps > 0 {
                    let num_long_term_sps = read_uvlc(from)?;
                }
                num_long_term_pics = read_uvlc(from)?;
                for i in 0..(num_long_term_sps + num_long_term_pics) {
                    if i < num_long_term_sps {
                        if num_long_term_ref_pics_sps > 1 {
                            bail!("lt_idx_sps[i] u(v)")
                        }
                    } else {
                        bail!("poc_lsb_lt[ i ] u(v)");
                        used_by_curr_pic_lt_flag[i] = from.read_bool()?;
                    }
                    delta_poc_msb_present_flag[i] = from.read_bool()?;
                    if delta_poc_msb_present_flag[i] {
                        delta_poc_msb_cycle_lt[i] = read_uvlc(from)?;
                    }
                }
            }
            if sps_temporal_mvp_enabled_flag {
                let slice_temporal_mvp_enabled_flag = from.read_bool()?;
            }
        }
        if sample_adaptive_offset_enabled_flag {
            let slice_sao_luma_flag = from.read_bool()?;
            let slice_sao_chroma_flag = from.read_bool()?;
        }
        if slice_type == P || slice_type == B {
            let num_ref_idx_active_override_flag = from.read_bool()?;
            if num_ref_idx_active_override_flag {
                let num_ref_idx_l0_active_minus1 = read_uvlc(from)?;
                if slice_type == B {
                    let num_ref_idx_l1_active_minus1 = read_uvlc(from)?;
                }
            }
            if lists_modification_present_flag && NumPocTotalCurr > 1 {
                ref_pic_lists_modification()
            }
            if slice_type == B {
                let mvd_l1_zero_flag = from.read_bool()?;
            }
            if cabac_init_present_flag {
                let cabac_init_flag = from.read_bool()?;
            }
            if slice_temporal_mvp_enabled_flag {
                if slice_type == B {
                    let collocated_from_l0_flag = from.read_bool()?;
                }
                if (collocated_from_l0_flag && num_ref_idx_l0_active_minus1 > 0)
                    || (!collocated_from_l0_flag && num_ref_idx_l1_active_minus1 > 0)
                {
                    let collocated_ref_idx = read_uvlc(from)?;
                }
            }
            if (weighted_pred_flag && slice_type == P) || (weighted_bipred_flag && slice_type == B)
            {
                pred_weight_table()
            }
            let five_minus_max_num_merge_cand = read_uvlc(from)?;
        }
        let slice_qp_delta = read_uvlc(from)?; // TODO: signed
        if pps_slice_chroma_qp_offsets_present_flag {
            let slice_cb_qp_offset = read_uvlc(from)?; // TODO: signed
            let slice_cr_qp_offset = read_uvlc(from)?; // TODO: signed
        }
        if deblocking_filter_override_enabled_flag {
            let deblocking_filter_override_flag = from.read_bool()?;
        }

        if deblocking_filter_override_flag {
            let slice_deblocking_filter_disabled_flag = from.read_bool()?;
            if !slice_deblocking_filter_disabled_flag {
                let slice_beta_offset_div2 = read_uvlc(from)?; // TODO: signed
                let slice_tc_offset_div2 = read_uvlc(from)?; // TODO: signed
            }
        }
        if pps_loop_filter_across_slices_enabled_flag
            && (slice_sao_luma_flag
                || slice_sao_chroma_flag
                || !slice_deblocking_filter_disabled_flag)
        {
            let slice_loop_filter_across_slices_enabled_flag = from.read_bool()?;
        }
    }

    if tiles_enabled_flag || entropy_coding_sync_enabled_flag {
        let num_entry_point_offsets = read_uvlc(from)?;
        if num_entry_point_offsets > 0 {
            let offset_len_minus1 = read_uvlc(from)?;
            for i in 0..num_entry_point_offsets {
                bail!("entry_point_offset_minus1[i] u(v)")
            }
        }
    }
    if slice_segment_header_extension_present_flag {
        let slice_segment_header_extension_length = read_uvlc(from)?;
        for i in 0..slice_segment_header_extension_length {
            bail!("slice_segment_header_extension_data_byte[i] u(8)")
        }
    }

    byte_alignment();

    Ok(())
}
