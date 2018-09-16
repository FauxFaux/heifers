use bitreader::BitReader;
use cast::u8;
use failure::Error;

use hevc::rbsp_trailing_bits;
use hevc::read_uvlc;
use hevc::vps::profile_tier_level;

bitflags! {
    #[derive(Default)]
    pub struct Flags: u16 {
        const SPS_TEMPORAL_ID_NESTING             = 1 <<  0;
        const SEPARATE_COLOUR_PLANE               = 1 <<  1;
        const CONFORMANCE_WINDOW                  = 1 <<  2;
        const SPS_SUB_LAYER_ORDERING_INFO_PRESENT = 1 <<  3;
        const SCALING_LIST_ENABLED                = 1 <<  4;
        const SPS_SCALING_LIST_DATA_PRESENT       = 1 <<  5;
        const AMP_ENABLED                         = 1 <<  6;
        const SAMPLE_ADAPTIVE_OFFSET_ENABLED      = 1 <<  7;
        const PCM_ENABLED                         = 1 <<  8;
        const PCM_LOOP_FILTER_DISABLED            = 1 <<  9;
        const LONG_TERM_REF_PICS_PRESENT          = 1 << 10;
        const USED_BY_CURR_PIC_LT_SPS             = 1 << 11;
        const SPS_TEMPORAL_MVP_ENABLED            = 1 << 12;
        const STRONG_INTRA_SMOOTHING_ENABLED      = 1 << 13;
        const VUI_PARAMETERS_PRESENT              = 1 << 14;
        const SPS_EXTENSION                       = 1 << 15;
    }
}

pub struct SeqParamSet {
    pub log2_max_pic_order_cnt_lsb_minus4: u8,
    pub num_short_term_ref_pic_sets: u8,
    pub num_long_term_ref_pics_sps: u8,
    pub flags: Flags,
}

pub fn seq_parameter_set(from: &mut BitReader) -> Result<SeqParamSet, Error> {
    let mut flags = Flags::default();

    let sps_video_parameter_set_id = from.read_u8(4)?;
    let sps_max_sub_layers_minus1 = from.read_u8(3)?;
    flags |= read_flag(from, Flags::SPS_TEMPORAL_ID_NESTING)?;
    // minus1 here is complicated, it appears to actually want the minus1
    profile_tier_level(from, sps_max_sub_layers_minus1)?;
    let sps_seq_parameter_set_id = read_uvlc(from)?;
    let chroma_format_idc = read_uvlc(from)?;
    if 3 == chroma_format_idc {
        flags |= read_flag(from, Flags::SEPARATE_COLOUR_PLANE)?;
    }
    let pic_width_in_luma_samples = read_uvlc(from)?;
    let pic_height_in_luma_samples = read_uvlc(from)?;
    flags |= read_flag(from, Flags::CONFORMANCE_WINDOW)?;
    if flags.contains(Flags::CONFORMANCE_WINDOW) {
        let conf_win_left_offset = read_uvlc(from)?;
        let conf_win_right_offset = read_uvlc(from)?;
        let conf_win_top_offset = read_uvlc(from)?;
        let conf_win_bottom_offset = read_uvlc(from)?;
    }
    let bit_depth_luma_minus8 = read_uvlc(from)?;
    let bit_depth_chroma_minus8 = read_uvlc(from)?;
    let log2_max_pic_order_cnt_lsb_minus4 = {
        let val = read_uvlc(from)?;
        ensure!(
            val <= 12,
            "log2_max_pic_order_cnt_lsb_minus4 out of range: {}",
            val
        );
        u8(val).unwrap()
    };
    flags |= read_flag(from, Flags::SPS_SUB_LAYER_ORDERING_INFO_PRESENT)?;
    if flags.contains(Flags::SPS_SUB_LAYER_ORDERING_INFO_PRESENT) {
        for i in 0..(sps_max_sub_layers_minus1 + 1) {
            let sps_max_dec_pic_buffering_minus1 = read_uvlc(from)?;
            let sps_max_num_reorder_pics = read_uvlc(from)?;
            let sps_max_latency_increase_plus1 = read_uvlc(from)?;
        }
    }

    let log2_min_luma_coding_block_size_minus3 = read_uvlc(from)?;
    let log2_diff_max_min_luma_coding_block_size = read_uvlc(from)?;
    let log2_min_transform_block_size_minus2 = read_uvlc(from)?;
    let log2_diff_max_min_transform_block_size = read_uvlc(from)?;
    let max_transform_hierarchy_depth_inter = read_uvlc(from)?;
    let max_transform_hierarchy_depth_intra = read_uvlc(from)?;
    flags |= read_flag(from, Flags::SCALING_LIST_ENABLED)?;
    if flags.contains(Flags::SCALING_LIST_ENABLED) {
        flags |= read_flag(from, Flags::SPS_SCALING_LIST_DATA_PRESENT)?;
        if flags.contains(Flags::SPS_SCALING_LIST_DATA_PRESENT) {
            bail!("scaling_list_data()");
        }
    }

    flags |= read_flag(from, Flags::AMP_ENABLED)?;
    flags |= read_flag(from, Flags::SAMPLE_ADAPTIVE_OFFSET_ENABLED)?;
    flags |= read_flag(from, Flags::PCM_ENABLED)?;

    if flags.contains(Flags::PCM_ENABLED) {
        let pcm_sample_bit_depth_luma_minus1 = from.read_u8(4)?;
        let pcm_sample_bit_depth_chroma_minus1 = from.read_u8(4)?;
        let log2_min_pcm_luma_coding_block_size_minus3 = read_uvlc(from)?;
        let log2_diff_max_min_pcm_luma_coding_block_size = read_uvlc(from)?;
        flags |= read_flag(from, Flags::PCM_LOOP_FILTER_DISABLED)?;
    }

    let num_short_term_ref_pic_sets = {
        let val = read_uvlc(from)?;
        ensure!(
            val <= 64,
            "num_short_term_ref_pic_sets out of range: {}",
            val
        );
        u8(val).unwrap()
    };
    for i in 0..num_short_term_ref_pic_sets {
        short_term_ref_pic_set(from, num_short_term_ref_pic_sets, i)?;
    }
    flags |= read_flag(from, Flags::LONG_TERM_REF_PICS_PRESENT)?;
    let mut num_long_term_ref_pics_sps = 0u8;
    if flags.contains(Flags::LONG_TERM_REF_PICS_PRESENT) {
        num_long_term_ref_pics_sps = {
            let val = read_uvlc(from)?;
            ensure!(
                val <= 32,
                "num_long_term_ref_pics_sps out of range: {}",
                val
            );
            u8(val).unwrap()
        };

        for i in 0..num_long_term_ref_pics_sps {
            let lt_ref_pic_poc_lsb_sps = read_uvlc(from)?;
            flags |= read_flag(from, Flags::USED_BY_CURR_PIC_LT_SPS)?;
        }
    }
    flags |= read_flag(from, Flags::SPS_TEMPORAL_MVP_ENABLED)?;
    flags |= read_flag(from, Flags::STRONG_INTRA_SMOOTHING_ENABLED)?;
    flags |= read_flag(from, Flags::VUI_PARAMETERS_PRESENT)?;
    if flags.contains(Flags::VUI_PARAMETERS_PRESENT) {
        vui_parameters(from)?;
    }
    flags |= read_flag(from, Flags::SPS_EXTENSION)?;
    ensure!(
        !flags.contains(Flags::SPS_EXTENSION),
        "unsupported sps extension"
    );
    rbsp_trailing_bits(from)?;
    Ok(SeqParamSet {
        log2_max_pic_order_cnt_lsb_minus4,
        num_short_term_ref_pic_sets,
        num_long_term_ref_pics_sps,
        flags,
    })
}

fn vui_parameters(from: &mut BitReader) -> Result<(), Error> {
    const EXTENDED_SAR: u8 = 255;
    let aspect_ratio_info_present_flag = from.read_bool()?;
    if aspect_ratio_info_present_flag {
        let aspect_ratio_idc = from.read_u8(8)?;
        if aspect_ratio_idc == EXTENDED_SAR {
            let sar_width = from.read_u16(16)?;
            let sar_height = from.read_u16(16)?;
        }
    }
    let overscan_info_present_flag = from.read_bool()?;
    if overscan_info_present_flag {
        let overscan_appropriate_flag = from.read_bool()?;
    }
    let video_signal_type_present_flag = from.read_bool()?;
    if video_signal_type_present_flag {
        let video_format = from.read_u8(3)?;
        let video_full_range_flag = from.read_bool()?;
        let colour_description_present_flag = from.read_bool()?;
        if colour_description_present_flag {
            let colour_primaries = from.read_u8(8)?;
            let transfer_characteristics = from.read_u8(8)?;
            let matrix_coeffs = from.read_u8(8)?;
        }
    }
    let chroma_loc_info_present_flag = from.read_bool()?;
    if chroma_loc_info_present_flag {
        let chroma_sample_loc_type_top_field = read_uvlc(from)?;
        let chroma_sample_loc_type_bottom_field = read_uvlc(from)?;
    }
    let neutral_chroma_indication_flag = from.read_bool()?;
    let field_seq_flag = from.read_bool()?;
    let frame_field_info_present_flag = from.read_bool()?;
    let default_display_window_flag = from.read_bool()?;
    if default_display_window_flag {
        let def_disp_win_left_offset = read_uvlc(from)?;
        let def_disp_win_right_offset = read_uvlc(from)?;
        let def_disp_win_top_offset = read_uvlc(from)?;
        let def_disp_win_bottom_offset = read_uvlc(from)?;
    }
    let vui_timing_info_present_flag = from.read_bool()?;
    if vui_timing_info_present_flag {
        let vui_num_units_in_tick = from.read_u32(32)?;
        let vui_time_scale = from.read_u32(32)?;
        let vui_poc_proportional_to_timing_flag = from.read_bool()?;
        if vui_poc_proportional_to_timing_flag {
            let vui_num_ticks_poc_diff_one_minus1 = read_uvlc(from)?;
        }
        let vui_hrd_parameters_present_flag = from.read_bool()?;
        if vui_hrd_parameters_present_flag {
            bail!("hrd_parameters(1, sps_max_sub_layers_minus1)");
        }
    }
    let bitstream_restriction_flag = from.read_bool()?;
    if bitstream_restriction_flag {
        let tiles_fixed_structure_flag = from.read_bool()?;
        let motion_vectors_over_pic_boundaries_flag = from.read_bool()?;
        let restricted_ref_pic_lists_flag = from.read_bool()?;
        let min_spatial_segmentation_idc = read_uvlc(from)?;
        let max_bytes_per_pic_denom = read_uvlc(from)?;
        let max_bits_per_min_cu_denom = read_uvlc(from)?;
        let log2_max_mv_length_horizontal = read_uvlc(from)?;
        let log2_max_mv_length_vertical = read_uvlc(from)?;
    }

    Ok(())
}

fn short_term_ref_pic_set(
    from: &mut BitReader,
    num_short_term_ref_pic_sets: u8,
    st_rps_idx: u8,
) -> Result<(), Error> {
    let inter_ref_pic_set_prediction_flag = if st_rps_idx != 0 {
        from.read_bool()?
    } else {
        false
    };

    if inter_ref_pic_set_prediction_flag {
        bail!("unimplemented inter-ref");
        if st_rps_idx == num_short_term_ref_pic_sets {
            let delta_idx_minus1 = read_uvlc(from)?;
        }
        let delta_rps_sign = from.read_bool()?;
        let abs_delta_rps_minus1 = read_uvlc(from)?;
        for j in 0..unimplemented!("NumDeltaPocs[RefRpsIdx]") {
            let used_by_curr_pic_flag = from.read_bool()?;
            if !used_by_curr_pic_flag {
                let use_delta_flag = from.read_bool()?;
            }
        }
    } else {
        let num_negative_pics = read_uvlc(from)?;
        let num_positive_pics = read_uvlc(from)?;
        for i in 0..num_negative_pics {
            let delta_poc_s0_minus1 = read_uvlc(from)?;
            let used_by_curr_pic_s0_flag = from.read_bool()?;
        }
        for i in 0..num_positive_pics {
            let delta_poc_s1_minus1 = read_uvlc(from)?;
            let used_by_curr_pic_s1_flag = from.read_bool()?;
        }
    }
    Ok(())
}

fn un_nal(bytes: &[u8]) -> Vec<u8> {
    let mut ret = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if i < bytes.len() - 2 && 0x00 == bytes[i] && 0x00 == bytes[i + 1] && 0x03 == bytes[i + 2] {
            ret.push(0);
            ret.push(0);
            i += 3;
        } else {
            ret.push(bytes[i]);
            i += 1;
        }
    }

    ret
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
    fn sps() {
        let bytes = [
            1, 4, 8, 0, 0, 3, 0, 159, 168, 0, 0, 3, 0, 0, 60, 160, 11, 72, 12, 31, 89, 110, 164,
            146, 138, 224, 16, 0, 0, 3, 0, 16, 0, 0, 3, 0, 16, 128,
        ];
        let un_nalled = super::un_nal(&bytes);
        println!("{:?}", un_nalled);
        let mut reader = BitReader::new(&un_nalled);

        super::seq_parameter_set(&mut reader).unwrap();
    }
}
