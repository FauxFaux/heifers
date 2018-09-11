use bitreader::BitReader;
use failure::Error;

use hevc::pps;
use hevc::pps::PicParamSet;
use hevc::read_uvlc;

pub fn slice_segment_header(
    unit_type: u8,
    from: &mut BitReader,
    pps: &PicParamSet,
) -> Result<(), Error> {
    let first_slice_segment_in_pic_flag = from.read_bool()?;

    if unit_type >= super::NAL_BLA_W_LP && unit_type <= super::NAL_RSV_IRAP_VCL23 {
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

    ensure!(dependent_slice_segment_flag, "unsupported dependent slice");
    Ok(())
}
