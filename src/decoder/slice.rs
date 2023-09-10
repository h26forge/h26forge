//! Slice header and data syntax element decoding.

use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbType;
use crate::common::data_structures::NALUheader;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::Slice;
use crate::common::data_structures::SliceData;
use crate::common::data_structures::SliceHeader;
use crate::common::data_structures::SubsetSPS;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::decoder_formatted_print;
use crate::common::helper::is_slice_type;
use crate::common::helper::ByteStream;
use crate::decoder::cabac::cabac_decode;
use crate::decoder::cabac::initialize_state;
use crate::decoder::cabac::CABACState;
use crate::decoder::expgolomb::exp_golomb_decode_one_wrapper;
use crate::decoder::macroblock::decode_macroblock_layer;
use log::debug;

/// Follows section 7.3.3.1
fn ref_pic_list_modification(sh: &mut SliceHeader, bs: &mut ByteStream) {
    if sh.slice_type % 5 != 2 && sh.slice_type % 5 != 4 {
        // ref_pic_list_modification_flag_l0
        let r = bs.read_bits(1);
        sh.ref_pic_list_modification_flag_l0 = 1 == r;
        decoder_formatted_print("SH: ref_pic_list_reordering_flag_l0", &r, 63);

        if sh.ref_pic_list_modification_flag_l0 {
            // do while trick for Rust: https://gist.github.com/huonw/8435502
            let mut i = 0;
            while {
                sh.modification_of_pic_nums_idc_l0
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

                decoder_formatted_print(
                    "SH: modification_of_pic_nums_idc_l0",
                    sh.modification_of_pic_nums_idc_l0[i],
                    63,
                );

                // abs_diff_pic_num_minus1
                if sh.modification_of_pic_nums_idc_l0[i] == 0
                    || sh.modification_of_pic_nums_idc_l0[i] == 1
                {
                    sh.abs_diff_pic_num_minus1_l0
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: abs_diff_pic_num_minus1_l0",
                        sh.abs_diff_pic_num_minus1_l0[i],
                        63,
                    );
                } else {
                    // push dummy values for later access
                    sh.abs_diff_pic_num_minus1_l0.push(0);
                }

                if sh.modification_of_pic_nums_idc_l0[i] == 2 {
                    sh.long_term_pic_num_l0
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: long_term_pic_num",
                        sh.long_term_pic_num_l0[i],
                        63,
                    );
                } else {
                    // push dummy values for later access
                    sh.long_term_pic_num_l0.push(0);
                }

                i += 1;

                // we do i-1 because we just added 1 above
                sh.modification_of_pic_nums_idc_l0[i - 1] != 3
            } {}
        }
    }

    if sh.slice_type % 5 == 1 {
        // ref_pic_list_modification_flag_l1
        let r = bs.read_bits(1);
        sh.ref_pic_list_modification_flag_l1 = r == 1;

        decoder_formatted_print("SH: ref_pic_list_reordering_flag_l1", &r, 63);

        if sh.ref_pic_list_modification_flag_l1 {
            let mut i = 0;
            while {
                sh.modification_of_pic_nums_idc_l1
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

                decoder_formatted_print(
                    "SH: modification_of_pic_nums_idc_l1",
                    sh.modification_of_pic_nums_idc_l1[i],
                    63,
                );

                // abs_diff_pic_num_minus1
                if sh.modification_of_pic_nums_idc_l1[i] == 0
                    || sh.modification_of_pic_nums_idc_l1[i] == 1
                {
                    sh.abs_diff_pic_num_minus1_l1
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: abs_diff_pic_num_minus1_l1",
                        sh.abs_diff_pic_num_minus1_l1[i],
                        63,
                    );
                } else {
                    // push dummy values
                    sh.abs_diff_pic_num_minus1_l1.push(0);
                }

                if sh.modification_of_pic_nums_idc_l1[i] == 2 {
                    sh.long_term_pic_num_l1
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: long_term_pic_num",
                        sh.long_term_pic_num_l1[i],
                        63,
                    );
                } else {
                    // push dummy values
                    sh.long_term_pic_num_l1.push(0);
                }

                i += 1;
                sh.modification_of_pic_nums_idc_l1[i - 1] != 3
            } {}
        }
    }
}

/// Follows section H.7.3.3.1.1
fn ref_pic_list_mvc_modification(sh: &mut SliceHeader, bs: &mut ByteStream) {
    if sh.slice_type % 5 != 2 && sh.slice_type % 5 != 4 {
        // ref_pic_list_modification_flag_l0
        let r = bs.read_bits(1);
        sh.ref_pic_list_modification_flag_l0 = 1 == r;
        decoder_formatted_print("SH: ref_pic_list_reordering_flag_l0", &r, 63);

        if sh.ref_pic_list_modification_flag_l0 {
            let mut i = 0;
            while {
                sh.modification_of_pic_nums_idc_l0
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

                decoder_formatted_print(
                    "SH: modification_of_pic_nums_idc_l0",
                    sh.modification_of_pic_nums_idc_l0[i],
                    63,
                );

                // abs_diff_pic_num_minus1
                if sh.modification_of_pic_nums_idc_l0[i] == 0
                    || sh.modification_of_pic_nums_idc_l0[i] == 1
                {
                    sh.abs_diff_pic_num_minus1_l0
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: abs_diff_pic_num_minus1_l0",
                        sh.abs_diff_pic_num_minus1_l0[i],
                        63,
                    );
                } else {
                    sh.abs_diff_pic_num_minus1_l0.push(0); //dummy
                }

                if sh.modification_of_pic_nums_idc_l0[i] == 2 {
                    sh.long_term_pic_num_l0
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: long_term_pic_num",
                        sh.long_term_pic_num_l0[i],
                        63,
                    );
                } else {
                    sh.long_term_pic_num_l0.push(0);
                }

                if sh.modification_of_pic_nums_idc_l0[i] == 4
                    || sh.modification_of_pic_nums_idc_l0[i] == 5
                {
                    sh.abs_diff_view_idx_minus1_l0
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: abs_diff_view_idx_minus1",
                        sh.abs_diff_view_idx_minus1_l0[i],
                        63,
                    );
                } else {
                    sh.abs_diff_view_idx_minus1_l0.push(0);
                }

                i += 1;

                // we do i-1 because we just added 1 above
                sh.modification_of_pic_nums_idc_l0[i - 1] != 3
            } {}
        }
    }

    if sh.slice_type % 5 == 1 {
        // ref_pic_list_modification_flag_l1
        let r = bs.read_bits(1);
        sh.ref_pic_list_modification_flag_l1 = r == 1;

        decoder_formatted_print("SH: ref_pic_list_reordering_flag_l1", &r, 63);

        if sh.ref_pic_list_modification_flag_l1 {
            let mut i = 0;
            while {
                sh.modification_of_pic_nums_idc_l1
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

                decoder_formatted_print(
                    "SH: modification_of_pic_nums_idc_l1",
                    sh.modification_of_pic_nums_idc_l1[i],
                    63,
                );

                // abs_diff_pic_num_minus1
                if sh.modification_of_pic_nums_idc_l1[i] == 0
                    || sh.modification_of_pic_nums_idc_l1[i] == 1
                {
                    sh.abs_diff_pic_num_minus1_l1
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: abs_diff_pic_num_minus1_l1",
                        sh.abs_diff_pic_num_minus1_l1[i],
                        63,
                    );
                } else {
                    sh.abs_diff_pic_num_minus1_l1.push(0);
                }

                if sh.modification_of_pic_nums_idc_l1[i] == 2 {
                    sh.long_term_pic_num_l1
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: long_term_pic_num",
                        sh.long_term_pic_num_l1[i],
                        63,
                    );
                } else {
                    sh.long_term_pic_num_l1.push(0);
                }

                if sh.modification_of_pic_nums_idc_l1[i] == 4
                    || sh.modification_of_pic_nums_idc_l1[i] == 5
                {
                    sh.abs_diff_view_idx_minus1_l1
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: abs_diff_view_idx_minus1",
                        sh.abs_diff_view_idx_minus1_l1[i],
                        63,
                    );
                } else {
                    sh.abs_diff_view_idx_minus1_l1.push(0);
                }

                i += 1;
                sh.modification_of_pic_nums_idc_l1[i - 1] != 3
            } {}
        }
    }
}

/// Follows section 7.3.3.2
fn pred_weight_table(sh: &mut SliceHeader, bs: &mut ByteStream, chroma_array_type: u8) {
    // luma_log2_weight_denom
    sh.luma_log2_weight_denom = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;

    decoder_formatted_print("SH: luma_log2_weight_denom", sh.luma_log2_weight_denom, 63);

    // chroma_log2_weight_denom
    if chroma_array_type != 0 {
        sh.chroma_log2_weight_denom = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SH: chroma_log2_weight_denom",
            sh.chroma_log2_weight_denom,
            63,
        );
    }

    // get weight tables
    for i in 0..(sh.num_ref_idx_l0_active_minus1 + 1) {
        let r = bs.read_bits(1);
        sh.luma_weight_l0_flag.push(r == 1);

        decoder_formatted_print("SH: luma_weight_flag_l0", &r, 63);

        if sh.luma_weight_l0_flag[i as usize] {
            let r = exp_golomb_decode_one_wrapper(bs, true, 0);
            sh.luma_weight_l0.push(r);

            decoder_formatted_print("SH: luma_weight_l0", r, 63);

            let r = exp_golomb_decode_one_wrapper(bs, true, 0);
            sh.luma_offset_l0.push(r);

            decoder_formatted_print("SH: luma_offset_l0", r, 63);
        } else {
            // this is to ensure our pushes are aligned with the index
            sh.luma_weight_l0.push(0);
            sh.luma_offset_l0.push(0);
        }

        if chroma_array_type != 0 {
            let r = bs.read_bits(1);
            sh.chroma_weight_l0_flag.push(r == 1);

            decoder_formatted_print("SH: chroma_weight_flag_l0", &r, 63);

            if sh.chroma_weight_l0_flag[i as usize] {
                let cwl0 = exp_golomb_decode_one_wrapper(bs, true, 0);
                decoder_formatted_print("SH: chroma_weight_l0", cwl0, 63);
                let col0 = exp_golomb_decode_one_wrapper(bs, true, 0);
                decoder_formatted_print("SH: chroma_offset_l0", col0, 63);
                let cwl1 = exp_golomb_decode_one_wrapper(bs, true, 0);
                decoder_formatted_print("SH: chroma_weight_l0", cwl1, 63);
                let col1 = exp_golomb_decode_one_wrapper(bs, true, 0);
                decoder_formatted_print("SH: chroma_offset_l0", col1, 63);

                sh.chroma_weight_l0.push(vec![cwl0, cwl1]);
                sh.chroma_offset_l0.push(vec![col0, col1]);
            } else {
                // to ensure the indices are aligned
                sh.chroma_weight_l0.push(vec![0, 0]);
                sh.chroma_offset_l0.push(vec![0, 0]);
            }
        }
    }
    // collect the l1 values
    if sh.slice_type % 5 == 1 {
        for i in 0..(sh.num_ref_idx_l1_active_minus1 + 1) {
            sh.luma_weight_l1_flag.push(bs.read_bits(1) == 1);

            decoder_formatted_print(
                "SH: luma_weight_l1_flag",
                sh.luma_weight_l1_flag[i as usize],
                63,
            );
            if sh.luma_weight_l1_flag[i as usize] {
                let r = exp_golomb_decode_one_wrapper(bs, true, 0);
                decoder_formatted_print("SH: luma_weight_l1", r, 63);
                sh.luma_weight_l1.push(r);

                let r = exp_golomb_decode_one_wrapper(bs, true, 0);
                sh.luma_offset_l1.push(r);

                decoder_formatted_print("SH: luma_offset_l1", r, 63);
            } else {
                // to ensure the indices are aligned
                sh.luma_weight_l1.push(0);
                sh.luma_offset_l1.push(0);
            }

            if chroma_array_type != 0 {
                sh.chroma_weight_l1_flag.push(bs.read_bits(1) == 1);

                decoder_formatted_print(
                    "SH: chroma_weight_l1_flag",
                    sh.chroma_weight_l1_flag[i as usize],
                    63,
                );
                if sh.chroma_weight_l1_flag[i as usize] {
                    let cwl0 = exp_golomb_decode_one_wrapper(bs, true, 0);
                    let col0 = exp_golomb_decode_one_wrapper(bs, true, 0);
                    let cwl1 = exp_golomb_decode_one_wrapper(bs, true, 0);
                    let col1 = exp_golomb_decode_one_wrapper(bs, true, 0);

                    decoder_formatted_print("SH: chroma_weight_l1", cwl0, 63);
                    decoder_formatted_print("SH: chroma_offset_l1", col0, 63);
                    decoder_formatted_print("SH: chroma_weight_l1", cwl1, 63);
                    decoder_formatted_print("SH: chroma_offset_l1", col1, 63);

                    sh.chroma_weight_l1.push(vec![cwl0, cwl1]);
                    sh.chroma_offset_l1.push(vec![col0, col1]);
                } else {
                    // to ensure the indices are aligned
                    sh.chroma_weight_l1.push(vec![0, 0]);
                    sh.chroma_offset_l1.push(vec![0, 0]);
                }
            }
        }
    }
}

/// Follows section 7.3.3.3
fn dec_ref_pic_marking(sh: &mut SliceHeader, bs: &mut ByteStream, idr_pic_flag: bool) {
    if idr_pic_flag {
        let r = bs.read_bits(1);
        sh.no_output_of_prior_pics_flag = r == 1;
        let s = bs.read_bits(1);
        sh.long_term_reference_flag = s == 1;

        decoder_formatted_print("SH: no_output_of_prior_pics_flag", &r, 63);
        decoder_formatted_print("SH: long_term_reference_flag", &s, 63);
    } else {
        let r = bs.read_bits(1);
        sh.adaptive_ref_pic_marking_mode_flag = r == 1;

        decoder_formatted_print("SH: adaptive_ref_pic_buffering_flag", &r, 63);

        if sh.adaptive_ref_pic_marking_mode_flag {
            let mut i = 0;

            while {
                sh.memory_management_control_operation
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

                decoder_formatted_print(
                    "SH: memory_management_control_operation",
                    sh.memory_management_control_operation[i],
                    63,
                );
                if sh.memory_management_control_operation[i] == 1
                    || sh.memory_management_control_operation[i] == 3
                {
                    sh.difference_of_pic_nums_minus1
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: difference_of_pic_nums_minus1",
                        sh.difference_of_pic_nums_minus1[i],
                        63,
                    );
                } else {
                    sh.difference_of_pic_nums_minus1.push(0);
                }

                if sh.memory_management_control_operation[i] == 2 {
                    sh.long_term_pic_num
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print("SH: long_term_pic_num", sh.long_term_pic_num[i], 63);
                } else {
                    sh.long_term_pic_num.push(0);
                }

                if sh.memory_management_control_operation[i] == 3
                    || sh.memory_management_control_operation[i] == 6
                {
                    sh.long_term_frame_idx
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: long_term_frame_idx",
                        sh.long_term_frame_idx[i],
                        63,
                    );
                } else {
                    sh.long_term_frame_idx.push(0);
                }
                if sh.memory_management_control_operation[i] == 4 {
                    sh.max_long_term_frame_idx_plus1
                        .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                    decoder_formatted_print(
                        "SH: max_long_term_frame_idx_plus1",
                        sh.max_long_term_frame_idx_plus1[i],
                        63,
                    );
                } else {
                    sh.max_long_term_frame_idx_plus1.push(0);
                }

                i += 1;
                // subtract 1 in the index
                sh.memory_management_control_operation[i - 1] != 0
            } {}
        }
    }
}

/// Follows section 7.3.3
fn decode_slice_header(
    bs: &mut ByteStream,
    nh: &NALUheader,
    spses: &Vec<SeqParameterSet>,
    ppses: &Vec<PicParameterSet>,
) -> (SliceHeader, usize, usize, VideoParameters) {
    let mut sh = SliceHeader::new();
    let mut pps_idx = 0; // search in reverse
    let mut sps_idx = 0; // search in reverse

    // first_mb_in_slice
    sh.first_mb_in_slice = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("SH: first_mb_in_slice", sh.first_mb_in_slice, 63);

    // slice_type
    sh.slice_type = exp_golomb_decode_one_wrapper(bs, false, 0) as u8;
    decoder_formatted_print("SH: slice_type", sh.slice_type, 63);

    // pic_parameter_set_id
    sh.pic_parameter_set_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("SH: pic_parameter_set_id", sh.pic_parameter_set_id, 63);
    let mut cur_pps_wrapper: Option<&PicParameterSet> = None;
    // retrieve the corresponding PPS
    for i in (0..ppses.len()).rev() {
        if ppses[i].pic_parameter_set_id == sh.pic_parameter_set_id {
            cur_pps_wrapper = Some(&ppses[i]);
            pps_idx = i;
            break;
        }
    }

    let p: &PicParameterSet;
    match cur_pps_wrapper {
        Some(x) => p = x,
        _ => panic!(
            "decode_slice_header - PPS with id {} not found",
            sh.pic_parameter_set_id
        ),
    }

    let mut cur_sps_wrapper: Option<&SeqParameterSet> = None;

    for i in (0..spses.len()).rev() {
        if spses[i].seq_parameter_set_id == p.seq_parameter_set_id {
            cur_sps_wrapper = Some(&spses[i]);
            sps_idx = i;
            break;
        }
    }

    let s: &SeqParameterSet;
    match cur_sps_wrapper {
        Some(x) => s = x,
        _ => panic!(
            "decode_slice_header - SPS with id {} not found",
            p.seq_parameter_set_id
        ),
    }

    let mut vp = VideoParameters::new(nh, p, s);

    // colour_plane_id
    if s.separate_colour_plane_flag {
        // consume two unsigned bits
        sh.colour_plane_id = bs.read_bits(2) as u8;
        decoder_formatted_print("SH: colour_plane_id", sh.colour_plane_id, 63);
    }

    // frame_num
    // - this is represented by log2_max_frame_num_minus4 + 4 bits in the bitstream
    sh.frame_num = bs.read_bits((s.log2_max_frame_num_minus4 + 4) as u8);
    decoder_formatted_print("SH: frame_num", sh.frame_num, 63);

    // field_pic_flag and bottom_field_flag
    if !s.frame_mbs_only_flag {
        sh.field_pic_flag = bs.read_bits(1) == 1;
        decoder_formatted_print("SH: field_pic_flag", sh.field_pic_flag, 63);

        // bottom_field_flag
        if sh.field_pic_flag {
            sh.bottom_field_flag = bs.read_bits(1) == 1;
            decoder_formatted_print("SH: bottom_field_flag", sh.bottom_field_flag, 63);
        }
    }

    //idr_pic_id
    // check the NAL Unit Type to see if it is an IDR picture
    if vp.idr_pic_flag {
        sh.idr_pic_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print("SH: idr_pic_id", sh.idr_pic_id, 63);
    }

    // pic_order_cnt_lsb and delta_pic_order_cnt_bottom
    if s.pic_order_cnt_type == 0 {
        // the length is log2_max_pic_order_cnt_lsb_minus4 + 4
        sh.pic_order_cnt_lsb = bs.read_bits(s.log2_max_pic_order_cnt_lsb_minus4 + 4);
        decoder_formatted_print("SH: pic_order_cnt_lsb", sh.pic_order_cnt_lsb, 63);

        // delta_pic_order_cnt_bottom
        if p.bottom_field_pic_order_in_frame_present_flag && !sh.field_pic_flag {
            sh.delta_pic_order_cnt_bottom = exp_golomb_decode_one_wrapper(bs, true, 0);
            decoder_formatted_print(
                "SH: delta_pic_order_cnt_bottom",
                sh.delta_pic_order_cnt_bottom,
                63,
            );
        }
    }

    // delta_pic_order_cnt[0] and [1]
    if s.pic_order_cnt_type == 1 && !s.delta_pic_order_always_zero_flag {
        sh.delta_pic_order_cnt
            .push(exp_golomb_decode_one_wrapper(bs, true, 0));
        decoder_formatted_print("SH: delta_pic_order_cnt[0]", sh.delta_pic_order_cnt[0], 63);

        // delta_pic_order_cnt[1]
        if p.bottom_field_pic_order_in_frame_present_flag && !sh.field_pic_flag {
            sh.delta_pic_order_cnt
                .push(exp_golomb_decode_one_wrapper(bs, true, 0));
            decoder_formatted_print("SH: delta_pic_order_cnt[1]", sh.delta_pic_order_cnt[1], 63);
        }
    }

    // redundant_pic_cnt
    if p.redundant_pic_cnt_present_flag {
        sh.redundant_pic_cnt = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print("SH: redundant_pic_cnt", sh.redundant_pic_cnt, 63);
    }

    // direct_spatial_mv_pred_flag
    if is_slice_type(sh.slice_type, "B") {
        let r = bs.read_bits(1);
        sh.direct_spatial_mv_pred_flag = r == 1;
        decoder_formatted_print("SH: direct_spatial_mv_pred_flag", &r, 63);
    }

    // num_ref_idx_active_override_flag and num_ref_idxl0_active_minus1
    //     and num_ref_idx_l1_active_minus1
    if is_slice_type(sh.slice_type, "P")
        || is_slice_type(sh.slice_type, "SP")
        || is_slice_type(sh.slice_type, "B")
    {
        let r = bs.read_bits(1);
        sh.num_ref_idx_active_override_flag = r == 1;
        decoder_formatted_print("SH: num_ref_idx_override_flag", &r, 63);

        // num_ref_idxl0_active_minus1
        if sh.num_ref_idx_active_override_flag {
            sh.num_ref_idx_l0_active_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
            decoder_formatted_print(
                "SH: num_ref_idx_l0_active_minus1",
                sh.num_ref_idx_l0_active_minus1,
                63,
            );
            // num_ref_idxl1_active_minus1
            if is_slice_type(sh.slice_type, "B") {
                sh.num_ref_idx_l1_active_minus1 =
                    exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
                decoder_formatted_print(
                    "SH: num_ref_idx_l1_active_minus1",
                    sh.num_ref_idx_l1_active_minus1,
                    63,
                );
            }
        } else {
            // if we're not overriding, then we set it to the default as dictated in the Spec (i.e. grab it from the PPS)
            sh.num_ref_idx_l0_active_minus1 = p.num_ref_idx_l0_default_active_minus1;
            sh.num_ref_idx_l1_active_minus1 = p.num_ref_idx_l1_default_active_minus1;
        }
    }

    if nh.nal_unit_type == 20 || nh.nal_unit_type == 21 {
        // ref_pic_list_mvc_modification (specified in Annex H: Multiview Video Coding)
        ref_pic_list_mvc_modification(&mut sh, bs);
    } else {
        ref_pic_list_modification(&mut sh, bs);
    }

    // pred_weight_table()
    if (p.weighted_pred_flag
        && (is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP")))
        || (p.weighted_bipred_idc == 1 && is_slice_type(sh.slice_type, "B"))
    {
        pred_weight_table(&mut sh, bs, vp.chroma_array_type);
    }

    // dec_ref_pic_marking()
    if nh.nal_ref_idc != 0 {
        dec_ref_pic_marking(&mut sh, bs, vp.idr_pic_flag);
    }

    // cabac_init_idc
    if p.entropy_coding_mode_flag
        && !is_slice_type(sh.slice_type, "I")
        && !is_slice_type(sh.slice_type, "SI")
    {
        sh.cabac_init_idc = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print("SH: cabac_init_idc", sh.cabac_init_idc, 63);
    }

    // slice_qp_delta
    sh.slice_qp_delta = exp_golomb_decode_one_wrapper(bs, true, 0);
    decoder_formatted_print("SH: slice_qp_delta", sh.slice_qp_delta, 63);

    // sp_for_switch_flag and slice_qs_delta
    if is_slice_type(sh.slice_type, "SP") || is_slice_type(sh.slice_type, "SI") {
        if is_slice_type(sh.slice_type, "SP") {
            sh.sp_for_switch_flag = bs.read_bits(1) == 1;
            decoder_formatted_print("SH: sp_for_switch_flag", sh.sp_for_switch_flag, 63);
        }
        sh.slice_qs_delta = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print("SH: slice_qs_delta", sh.slice_qs_delta, 63);
    }

    // disable_deblocking_filter_idc and slice_alpha_c0_offset_div2 and slice_beta_offset_div2
    if p.deblocking_filter_control_present_flag {
        sh.disable_deblocking_filter_idc = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SH: disable_deblocking_filter_idc",
            sh.disable_deblocking_filter_idc,
            63,
        );

        if sh.disable_deblocking_filter_idc != 1 {
            sh.slice_alpha_c0_offset_div2 = exp_golomb_decode_one_wrapper(bs, true, 0);
            sh.slice_beta_offset_div2 = exp_golomb_decode_one_wrapper(bs, true, 0);

            decoder_formatted_print(
                "SH: slice_alpha_c0_offset_div2",
                sh.slice_alpha_c0_offset_div2,
                63,
            );
            decoder_formatted_print("SH: slice_beta_offset_div2", sh.slice_beta_offset_div2, 63);
        }
    }

    // slice_group_change_cycle
    if p.num_slice_groups_minus1 > 0 && p.slice_group_map_type >= 3 && p.slice_group_map_type <= 5 {
        // size is dictated by equation 7-35:
        // Ceil(log2(PicSizeInMapUnits / SliceGroupChangeRate + 1))
        let bits_to_read = ((vp.pic_size_in_map_units / (p.slice_group_change_rate_minus1 + 1) + 1)
            as f64)
            .log2()
            .ceil() as u8;
        sh.slice_group_change_cycle = bs.read_bits(bits_to_read);
        decoder_formatted_print(
            "SH: slice_group_change_cycle",
            sh.slice_group_change_cycle,
            63,
        );
    }

    // Variables that are computed from the spec but not decoded from the stream
    // page 87
    if vp.idr_pic_flag {
        sh.prev_ref_frame_num = 0;
    } else {
        // see clause 8.2.5.2
        sh.prev_ref_frame_num = sh.frame_num;
    }
    // derivation from equation 7-25 in Spec
    sh.mbaff_frame_flag = s.mb_adaptive_frame_field_flag && !sh.field_pic_flag;
    // to be used in neighbor decoding
    vp.mbaff_frame_flag = sh.mbaff_frame_flag;

    // equation 7-26
    sh.pic_height_in_mbs = vp.frame_height_in_mbs
        / (1 + match sh.field_pic_flag {
            true => 1,
            false => 0,
        });

    // equation 7-27
    sh.pic_height_in_samples_luma = sh.pic_height_in_mbs * 16;

    // equation 7-28
    sh.pic_height_in_samples_chroma = sh.pic_height_in_mbs * (vp.mb_height_c as u32);

    // equation 7-29
    sh.pic_size_in_mbs = vp.pic_width_in_mbs * sh.pic_height_in_mbs;

    // bottom of section field_pic_flag
    // if !sh.field_pic_flag {
    //     sh.max_pic_num = vp.max_frame_num;
    //     sh.curr_pic_num = sh.frame_num;
    // } else {
    //     sh.max_pic_num = 2 * vp.max_frame_num;
    //     sh.curr_pic_num = 2 * sh.frame_num + 1;
    // }
    // equation 7-30
    sh.slice_qp_y = 26 + sh.slice_qp_delta + p.pic_init_qp_minus26;
    if sh.slice_qp_y < 0 || sh.slice_qp_y > 51 {
        println!(
            "[WARNING] slice_qp_y {} is outside of bounds [0, 51] - likely issues decoding",
            sh.slice_qp_y
        );
    }
    sh.qp_y_prev = sh.slice_qp_y;

    // equation 7-31
    sh.qs_y = (26 + sh.slice_qs_delta + p.pic_init_qs_minus26) as u8;

    // equation 7-32
    sh.filter_offset_a = sh.slice_alpha_c0_offset_div2 << 1;

    // equation 7-33
    sh.filter_offset_b = sh.slice_beta_offset_div2 << 1;

    (sh, pps_idx, sps_idx, vp)
}

/// Defined in Section 8.2.2. Handles Slice Groups
fn next_mb_addr(curr_mb_addr: usize, pic_size_in_mbs: usize, sgm: &Vec<u32>) -> usize {
    let mut next_addr = curr_mb_addr + 1;
    while next_addr < pic_size_in_mbs && sgm[next_addr] != sgm[curr_mb_addr] {
        next_addr += 1;
    }
    next_addr
}

/// Follows section 7.3.4
fn decode_slice_data(
    bs: &mut ByteStream,
    sh: &mut SliceHeader,
    s: &SeqParameterSet,
    p: &PicParameterSet,
    vp: &VideoParameters,
    decode_strict_fmo: bool,
) -> SliceData {
    // Picture Order Count, current_macroblock_number, current_slice_number, current_slice_type`
    let mut sd = SliceData::new();
    let mut cabac_state = CABACState::new();

    let mut num_macroblocks_to_encode: usize =
        ((s.pic_height_in_map_units_minus1 + 1) * (s.pic_width_in_mbs_minus1 + 1)) as usize;

    let sgm = if decode_strict_fmo {
        sh.generate_slice_group_map(s, p, vp)
    } else {
        vec![0; vp.pic_size_in_map_units as usize]
    };

    // multiply by 2 for frame slices in field-supporting videos
    if !s.frame_mbs_only_flag && !sh.field_pic_flag {
        num_macroblocks_to_encode *= 2;
    }

    if p.entropy_coding_mode_flag {
        // we perform byte alignment here
        if bs.byte_offset > 0 {
            bs.bytestream.pop_front();
            bs.byte_offset = 0;
        }
        cabac_state = initialize_state(bs);
    }
    // create the current Macroblock and set its address

    let mut curr_mb_addr = (sh.first_mb_in_slice
        * (1 + match sh.mbaff_frame_flag {
            true => 1,
            _ => 0,
        })) as usize;
    let mut curr_mb_idx = 0;
    let mut more_data_flag = true;
    let mut prev_mb_skipped = false;
    let mut prev_predicted_mb_field_decoding_flag = false; // necessary in MBAFF when the top MB is skipped, and the bottom one needs to decode mb_field_decoding_flag

    // do while trick
    while {
        if curr_mb_addr > num_macroblocks_to_encode - 1 {
            println!("[WARNING] Current MB address {} (MB index {}) is greater than expected number of macroblocks in frame/field {}", curr_mb_addr, curr_mb_idx, num_macroblocks_to_encode);
        }

        let mut curr_mb = MacroBlock::new();
        curr_mb.mb_idx = curr_mb_idx;
        curr_mb.mb_addr = curr_mb_addr;
        curr_mb.available = true;

        // These are 0 whenever chroma_format_idc is 0 or there is a separate_colour_plane_flag
        if vp.sub_height_c == 0 || vp.sub_width_c == 0 {
            curr_mb.num_c8x8 = 0;
        } else {
            curr_mb.num_c8x8 = (4 / (vp.sub_width_c * vp.sub_height_c)) as usize;
        }

        sd.macroblock_vec.push(curr_mb);
        debug!(target: "decode","");
        debug!(target: "decode","*********** POC: {} (I/P) MB: {} Slice: {} Type {} ***********",
            sh.pic_order_cnt_lsb,
            sd.macroblock_vec[curr_mb_idx].mb_addr,
            0,
            sh.slice_type % 5
        );
        if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") {
            if !p.entropy_coding_mode_flag {
                let mb_skip_run = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
                decoder_formatted_print("mb_skip_run", mb_skip_run, 63);

                //debug!(target: "decode","*********** Skipping over {} Macroblocks ***********", mb_skip_run);

                sd.mb_skip_run.push(mb_skip_run);

                prev_mb_skipped = mb_skip_run > 0;

                if prev_mb_skipped {
                    // set the current macroblock to skip type
                    if is_slice_type(sh.slice_type, "B") {
                        sd.macroblock_vec[curr_mb_idx].mb_type = MbType::BSkip;
                    } else {
                        sd.macroblock_vec[curr_mb_idx].mb_type = MbType::PSkip;
                    }
                    sd.mb_field_decoding_flag.push(false);
                }

                // this may be important for displaying info, but we can skip for now (hopefully)
                for i in 0..mb_skip_run {
                    curr_mb_addr = next_mb_addr(curr_mb_addr, sh.pic_size_in_mbs as usize, &sgm);
                    curr_mb_idx += 1;

                    curr_mb = MacroBlock::new();
                    curr_mb.mb_idx = curr_mb_idx;
                    curr_mb.mb_addr = curr_mb_addr;
                    curr_mb.available = true;
                    // avoid a div by zero -- occurs when separate colour plane flags or chroma format idc is 0 or 3
                    if vp.sub_width_c == 0 || vp.sub_height_c == 0 {
                        curr_mb.num_c8x8 = 0;
                    } else {
                        curr_mb.num_c8x8 = (4 / (vp.sub_width_c * vp.sub_height_c)) as usize;
                    }

                    if is_slice_type(sh.slice_type, "B") {
                        curr_mb.mb_type = MbType::BSkip;
                    } else {
                        curr_mb.mb_type = MbType::PSkip;
                    }

                    // to make sure the indices stay aligned, we push a collection of empty variables
                    sd.macroblock_vec.push(curr_mb);
                    sd.mb_skip_run.push(0);
                    // don't push for the last index because it gets set to the sh.field_pic_flag below
                    if i < mb_skip_run - 1 {
                        sd.mb_field_decoding_flag.push(false);
                    }
                }

                if mb_skip_run > 0 {
                    more_data_flag = bs.more_data();
                    if more_data_flag {
                        // Do an extra print
                        debug!(target: "decode","*********** POC: {} (I/P) MB: {} Slice: {} Type {} ***********",
                            sh.pic_order_cnt_lsb,
                            sd.macroblock_vec[curr_mb_idx].mb_addr,
                            0,
                            sh.slice_type % 5
                        );
                    }
                }
            } else {
                // do an inference of mb_field_decoding_flag based on neighbors
                if sh.mbaff_frame_flag {
                    // this inferring is necessary when the mb_field_decoding_flag
                    // is not present for both the top and bottom macroblock of a
                    // macroblock pair. This can happen when things are skipped

                    let mut recovered_from_above = false;

                    if sd.macroblock_vec[curr_mb_idx].mb_addr % 2 == 1 {
                        debug!(target: "decode","Inferring mb_field_decoding_flag: bottom pair, gonna try to copy from the top pair");
                        let mb_above: MacroBlock;
                        // for mb_b we know that we're the bottom pair, but first_mb_in_slice may mess us up so we just
                        // double check in the index to make sure there's a top macroblock
                        if curr_mb_idx < 1 {
                            mb_above = MacroBlock::new();
                        } else {
                            mb_above = sd.macroblock_vec[curr_mb_idx - 1].clone();
                        }

                        debug!(target: "decode","mb_above.mb_addr = {}", mb_above.mb_addr);

                        // if available and not skipped, copy it from above
                        if mb_above.available
                            && mb_above.mb_type != MbType::PSkip
                            && mb_above.mb_type != MbType::BSkip
                        {
                            debug!(target: "decode","Gonna copy from above - {}",sd.mb_field_decoding_flag[mb_above.mb_idx] );
                            sd.mb_field_decoding_flag
                                .push(sd.mb_field_decoding_flag[mb_above.mb_idx]);
                            recovered_from_above = true;
                        } else if mb_above.available
                            && (mb_above.mb_type == MbType::PSkip
                                || mb_above.mb_type == MbType::BSkip)
                        {
                            debug!(target: "decode","Above was skipped, so we'll use its predicted value - {}", prev_predicted_mb_field_decoding_flag );
                            sd.mb_field_decoding_flag
                                .push(prev_predicted_mb_field_decoding_flag);
                            recovered_from_above = true;
                        }
                    }

                    if !recovered_from_above {
                        // check neighbors for their flags
                        let mb_left: MacroBlock;
                        let mb_above: MacroBlock;

                        // if we're the top of the pair
                        if sd.macroblock_vec[curr_mb_idx].mb_addr % 2 == 0 {
                            debug!(target: "decode","Inferring mb_field_decoding_flag for top pair of an MBAFF frame");

                            // for the top pair, we first check if there's anything to the left
                            // if on the left most edge, then not ( modulo picture width *2 is equal to 0 or 1 )
                            // or if the curr_mb_idx is too small, then set to new
                            if (sd.macroblock_vec[curr_mb_idx].mb_addr as u32)
                                % (2 * vp.pic_width_in_mbs)
                                < 2
                                || curr_mb_idx < 2
                            {
                                debug!(target: "decode","sd.macroblock_vec[curr_mb_idx].mb_addr as u32 {}", sd.macroblock_vec[curr_mb_idx].mb_addr as u32);
                                debug!(target: "decode","2*vp.pic_width_in_mbs {}", 2*vp.pic_width_in_mbs);
                                debug!(target: "decode","curr_mb_idx {}", curr_mb_idx);
                                mb_left = MacroBlock::new();
                            } else {
                                mb_left = sd.macroblock_vec[curr_mb_idx - 2].clone();
                            }

                            // for mb_above, we need to check if we're at the top most part of the frame, or if not
                            // then get the MB right above us (which should have an odd MB value)
                            if (sd.macroblock_vec[curr_mb_idx].mb_addr as u32)
                                < (2 * vp.pic_width_in_mbs)
                                || (curr_mb_idx as u32) < (2 * vp.pic_width_in_mbs)
                            {
                                debug!(target: "decode","sd.macroblock_vec[curr_mb_idx].mb_addr as u32 {}", sd.macroblock_vec[curr_mb_idx].mb_addr as u32);
                                debug!(target: "decode","2*vp.pic_width_in_mbs {}", 2*vp.pic_width_in_mbs);
                                debug!(target: "decode","curr_mb_idx {}", curr_mb_idx);
                                mb_above = MacroBlock::new();
                            } else {
                                // we do +1 to get the odd MB
                                mb_above = sd.macroblock_vec
                                    [curr_mb_idx - (2 * vp.pic_width_in_mbs as usize) + 1]
                                    .clone();
                            }
                        } else {
                            // bottom of the pair
                            debug!(target: "decode","Inferring mb_field_decoding_flag for bottom pair of an MBAFF frame");

                            // first check if there's anything to the left
                            // if on the left most edge, then not ( modulo picture width *2 is equal to 0 or 1 )
                            // or if the curr_mb_idx is too small, then set to new
                            if (sd.macroblock_vec[curr_mb_idx].mb_addr as u32)
                                % (2 * vp.pic_width_in_mbs)
                                < 2
                                || (curr_mb_idx < 2)
                            {
                                debug!(target: "decode","sd.macroblock_vec[curr_mb_idx].mb_addr as u32 {}", sd.macroblock_vec[curr_mb_idx].mb_addr as u32);
                                debug!(target: "decode","2*vp.pic_width_in_mbs {}", 2*vp.pic_width_in_mbs);
                                debug!(target: "decode","curr_mb_idx {}", curr_mb_idx);
                                mb_left = MacroBlock::new();
                            } else {
                                mb_left = sd.macroblock_vec[curr_mb_idx - 2].clone();
                            }

                            // for mb_above we know that we're the bottom pair, but first_mb_in_slice may mess us up so we just
                            // double check in the index to make sure there's a top macroblock
                            if curr_mb_idx < 1 {
                                // if we didn't copy from above before, why would we do so now??
                                mb_above = MacroBlock::new();
                            } else {
                                mb_above = sd.macroblock_vec[curr_mb_idx - 1].clone();
                            }
                        }

                        debug!(target: "decode","mb_left.mb_addr {} and mb_left.mb_idx {}", mb_left.mb_addr, mb_left.mb_idx);
                        debug!(target: "decode","mb_above.mb_addr {} and mb_above.mb_idx {}", mb_above.mb_addr, mb_above.mb_idx);

                        if mb_left.available {
                            debug!(target: "decode","Copying from the left - {}", sd.mb_field_decoding_flag[mb_left.mb_idx]);
                            sd.mb_field_decoding_flag
                                .push(sd.mb_field_decoding_flag[mb_left.mb_idx]);
                        } else if mb_above.available {
                            debug!(target: "decode","Copying from above - {}", sd.mb_field_decoding_flag[mb_above.mb_idx]);
                            sd.mb_field_decoding_flag
                                .push(sd.mb_field_decoding_flag[mb_above.mb_idx]);
                        } else {
                            debug!(target: "decode","Just pushing false - should only happen on the left-most and top-most rows");
                            sd.mb_field_decoding_flag.push(false);
                        }
                    }
                    prev_predicted_mb_field_decoding_flag = sd.mb_field_decoding_flag[curr_mb_idx];
                }

                let r = cabac_decode(
                    "mb_skip_flag",
                    bs,
                    &mut cabac_state,
                    curr_mb_idx,
                    sh,
                    &mut sd,
                    vp,
                    0,
                    Vec::new(),
                );
                let res = match r {
                    1 => true,
                    _ => false,
                };

                sd.macroblock_vec[curr_mb_idx].mb_skip_flag = res;

                if sd.macroblock_vec[curr_mb_idx].mb_skip_flag {
                    if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
                        sd.macroblock_vec[curr_mb_idx].mb_type = MbType::PSkip;
                    } else if is_slice_type(sh.slice_type, "B") {
                        sd.macroblock_vec[curr_mb_idx].mb_type = MbType::BSkip;
                    }
                }
                decoder_formatted_print("mb_skip_flag", &r, 63);

                more_data_flag = !sd.macroblock_vec[curr_mb_idx].mb_skip_flag;
            }
        }
        if more_data_flag {
            // we can remove curr_mb.mb_addr %2 == 1 check, but leaving it in to follow the Spec
            if sh.mbaff_frame_flag
                && (sd.macroblock_vec[curr_mb_idx].mb_addr % 2 == 0
                    || (sd.macroblock_vec[curr_mb_idx].mb_addr % 2 == 1 && prev_mb_skipped))
            {
                let mb_field_decoding_flag: bool = if p.entropy_coding_mode_flag {
                    match cabac_decode(
                        "mb_field_decoding_flag",
                        bs,
                        &mut cabac_state,
                        curr_mb_idx,
                        sh,
                        &mut sd,
                        vp,
                        0,
                        Vec::new(),
                    ) {
                        1 => true,
                        _ => false,
                    }
                } else {
                    match bs.read_bits(1) {
                        1 => true,
                        _ => false,
                    }
                };

                // in case we added our inference for neighbor prediction above,
                // then rewrite that value
                if curr_mb_idx < sd.mb_field_decoding_flag.len() {
                    sd.mb_field_decoding_flag[curr_mb_idx] = mb_field_decoding_flag;
                } else {
                    sd.mb_field_decoding_flag.push(mb_field_decoding_flag);
                }

                // if we're in the bottom macroblock decoding this parameter, and the top macroblock was
                // skipped, then correct the mb_field_decoding_flag of that macroblock
                if sd.macroblock_vec[curr_mb_idx].mb_addr % 2 == 1 && prev_mb_skipped {
                    sd.mb_field_decoding_flag[curr_mb_idx - 1] = mb_field_decoding_flag;
                }

                decoder_formatted_print("mb_field_decoding_flag", mb_field_decoding_flag, 63);
            } else {
                // for balance
                // according to section 7.4.4, if mbaff is 0 then field decoding is equal to field pic flag
                if !sh.mbaff_frame_flag {
                    // don't have to worry about index check here because no MBAFF prediction
                    sd.mb_field_decoding_flag.push(sh.field_pic_flag);
                } else {
                    // mbaff is 1, and the field decoding flag is not present for both the top and bottom, then we take this route
                    //debug!(target: "decode"," howdy!! ");

                    // If we're here, then sh.mbaff_frame_flag is true, and the previous macroblock is not skipped
                    if sd.macroblock_vec[curr_mb_idx].mb_addr % 2 == 1 && curr_mb_idx > 0 {
                        // if the previous macroblock was a field, then this one should be too, if not, then we're not
                        if curr_mb_idx < sd.mb_field_decoding_flag.len() {
                            sd.mb_field_decoding_flag[curr_mb_idx] =
                                sd.mb_field_decoding_flag[curr_mb_idx - 1];
                        } else {
                            sd.mb_field_decoding_flag
                                .push(sd.mb_field_decoding_flag[curr_mb_idx - 1]);
                        }
                    } else {
                        if curr_mb_idx < sd.mb_field_decoding_flag.len() {
                            sd.mb_field_decoding_flag[curr_mb_idx] = false;
                        } else {
                            sd.mb_field_decoding_flag.push(false);
                        }
                    }

                    // From the spec:
                    // When MbaffFrameFlag is equal to 1 and mb_field_decoding_flag is not present for the top macroblock of a macroblock
                    // pair (because the top macroblock is skipped), a decoder must wait until mb_field_decoding_flag for the bottom macroblock is read
                    // (when the bottom macroblock is not skipped) or the value of mb_field_decoding_flag is inferred as specified above (when the bottom
                    // macroblock is also skipped) before it starts the decoding process for the top macroblock
                }
            }
            decode_macroblock_layer(curr_mb_idx, &mut cabac_state, bs, &mut sd, sh, vp, s, p);
        } else {
            // only push this in cases where we haven't added this list yet
            // only copy over values that are bottom macroblocks
            if curr_mb_idx > 0 {
                if curr_mb_idx >= sd.mb_field_decoding_flag.len() {
                    // if we skip, just copy over the previous value
                    sd.mb_field_decoding_flag
                        .push(sd.mb_field_decoding_flag[curr_mb_idx - 1]);
                }
            } else {
                // the case in which we skip the first macroblock in the slice
                // if we wrote something already, then put false
                if curr_mb_idx < sd.mb_field_decoding_flag.len() {
                    sd.mb_field_decoding_flag[curr_mb_idx] = false;
                } else {
                    // else push false
                    sd.mb_field_decoding_flag.push(false);
                }
            }
        }
        if !p.entropy_coding_mode_flag {
            more_data_flag = bs.more_data();
        } else {
            if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") {
                prev_mb_skipped = sd.macroblock_vec[curr_mb_idx].mb_skip_flag;
            }
            if sh.mbaff_frame_flag && sd.macroblock_vec[curr_mb_idx].mb_addr % 2 == 0 {
                more_data_flag = true;
                sd.end_of_slice_flag.push(false);
            } else {
                let r = cabac_decode(
                    "end_of_slice_flag",
                    bs,
                    &mut cabac_state,
                    curr_mb_idx,
                    sh,
                    &mut sd,
                    vp,
                    0,
                    Vec::new(),
                );
                sd.end_of_slice_flag.push(match r {
                    1 => true,
                    _ => false,
                });
                decoder_formatted_print("end_of_slice_flag", &r, 63);
                more_data_flag = !sd.end_of_slice_flag[sd.end_of_slice_flag.len() - 1];
            }
        }

        curr_mb_addr = next_mb_addr(curr_mb_addr, sh.pic_size_in_mbs as usize, &sgm);
        curr_mb_idx += 1;
        // variable we are checking
        more_data_flag
    } {}

    sd
}

/// Follows section 7.3.2.8
pub fn decode_slice_layer_without_partitioning_rbsp(
    nalu_data: &mut ByteStream,
    nh: &NALUheader,
    spses: &Vec<SeqParameterSet>,
    ppses: &Vec<PicParameterSet>,
    only_headers: bool,
    decode_strict_fmo: bool,
) -> Slice {
    let res = decode_slice_header(nalu_data, nh, spses, ppses);
    let mut sh = res.0;
    let p = &ppses[res.1];
    let s = &spses[res.2];
    let vp = res.3;
    // decode slice_data
    let sd: SliceData = if only_headers {
        SliceData::new()
    } else {
        decode_slice_data(nalu_data, &mut sh, s, p, &vp, decode_strict_fmo)
    };

    Slice { sh, sd }
}

/// Follows section 7.3.2.13
pub fn decode_slice_layer_extension_rbsp(
    nalu_data: &mut ByteStream,
    nh: &NALUheader,
    subset_spses: &Vec<SubsetSPS>,
    ppses: &Vec<PicParameterSet>,
    only_headers: bool,
    decode_strict_fmo: bool,
) -> Slice {
    if nh.svc_extension_flag {
        println!("[WARNING] NALU SVC Extension Flag enabled - Decoding may not be correct");
    } else if nh.avc_3d_extension_flag {
        println!("[WARNING] NALU AVC 3D Extension Flag enabled - Decoding may not be correct");
    }

    // TODO: Uncomment whenever implementation below is done

    //let sh : SliceHeader;
    //let sd : SliceData;
    //if nh.svc_extension_flag {
    //    let res = decode_slice_header_in_scalable_extension(); // specified in Annex G
    //    sh = res.0;
    //    let slice_skip_flag = res.1;
    //    if !slice_skip_flag {
    //        sd = decode_slice_data_in_scalable_extension(); // specified in Annex G
    //    } else {
    //        sd = SliceData::new();
    //    }
    //} else if nh.avc_3d_extension_flag {
    //    sh = decode_slice_header_in_3davc_extension();
    //    sd = decode_slice_data_in_3davc_extension();
    //} else {
    let mut spses = Vec::new();
    for s in subset_spses {
        spses.push(s.sps.clone());
    }
    decode_slice_layer_without_partitioning_rbsp(
        nalu_data,
        nh,
        &spses,
        ppses,
        only_headers,
        decode_strict_fmo,
    )
    //}
    //
    //return Slice{ sh : sh, sd : sd};
}

/// Follows section G.7.3.3.4
#[allow(dead_code)]
fn decode_slice_header_in_scalable_extension() -> (SliceHeader, bool) {
    let res = SliceHeader::new();
    let slice_skip_flag = true;
    // TODO: Annex G

    (res, slice_skip_flag)
}

/// Follows section G.7.3.4.1
#[allow(dead_code)]
fn decode_slice_data_in_scalable_extension() -> SliceData {
    // TODO: Annex G
    SliceData::new()
}

/// Follows section J.7.3.3.4
#[allow(dead_code)]
fn decode_slice_header_in_3davc_extension() -> SliceHeader {
    // TODO: Annex J
    SliceHeader::new()
}

/// Follows section J.7.3.4.1
#[allow(dead_code)]
fn decode_slice_data_in_3davc_extension() -> SliceData {
    // TODO: Annex J
    SliceData::new()
}
