//! Slice header and data syntax element encoding.

use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbType;
use crate::common::data_structures::NALUHeader;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::Slice;
use crate::common::data_structures::SliceHeader;
use crate::common::data_structures::SubsetSPS;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::bitstream_to_bytestream;
use crate::common::helper::encoder_formatted_print;
use crate::common::helper::is_slice_type;
use crate::encoder::cabac;
use crate::encoder::expgolomb::exp_golomb_encode_one;
use crate::encoder::macroblock::encode_macroblock;
use log::debug;

use super::binarization_functions::generate_unsigned_binary;

/// Reference pic list modifications moves around reference pictures inside
/// list 0 and list 1
fn encode_slice_header_ref_pic_list_modification(sh: &SliceHeader) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    // for B and P/SP slices
    if sh.slice_type % 5 != 2 && sh.slice_type % 5 != 4 {
        bitstream_array.push(match sh.ref_pic_list_modification_flag_l0 {
            true => 1u8,
            _ => 0u8,
        });
        encoder_formatted_print(
            "SH: ref_pic_list_modification_flag_l0",
            sh.ref_pic_list_modification_flag_l0,
            63,
        );

        if sh.ref_pic_list_modification_flag_l0 {
            for i in 0..sh.modification_of_pic_nums_idc_l0.len() {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.modification_of_pic_nums_idc_l0[i] as i32,
                    false,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("SH: modification_of_pic_nums_idc_l0[{}]", i).as_str(),
                    sh.modification_of_pic_nums_idc_l0[i],
                    63,
                );

                if sh.modification_of_pic_nums_idc_l0[i] == 0
                    || sh.modification_of_pic_nums_idc_l0[i] == 1
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.abs_diff_pic_num_minus1_l0[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: abs_diff_pic_num_minus1_l0[{}]", i).as_str(),
                        sh.abs_diff_pic_num_minus1_l0[i],
                        63,
                    );
                } else if sh.modification_of_pic_nums_idc_l0[i] == 2 {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.long_term_pic_num_l0[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: long_term_pic_num_l0[{}]", i).as_str(),
                        sh.long_term_pic_num_l0[i],
                        63,
                    );
                }
            }
        }
    }

    // for B slices
    if sh.slice_type % 5 == 1 {
        bitstream_array.push(match sh.ref_pic_list_modification_flag_l1 {
            true => 1u8,
            _ => 0u8,
        });
        encoder_formatted_print(
            "SH: ref_pic_list_modification_flag_l1",
            sh.ref_pic_list_modification_flag_l1,
            63,
        );

        if sh.ref_pic_list_modification_flag_l1 {
            for i in 0..sh.modification_of_pic_nums_idc_l1.len() {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.modification_of_pic_nums_idc_l1[i] as i32,
                    false,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("SH: modification_of_pic_nums_idc_l1[{}]", i).as_str(),
                    sh.modification_of_pic_nums_idc_l1[i],
                    63,
                );

                if sh.modification_of_pic_nums_idc_l1[i] == 0
                    || sh.modification_of_pic_nums_idc_l1[i] == 1
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.abs_diff_pic_num_minus1_l1[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: abs_diff_pic_num_minus1_l1[{}]", i).as_str(),
                        sh.abs_diff_pic_num_minus1_l1[i],
                        63,
                    );
                } else if sh.modification_of_pic_nums_idc_l1[i] == 2 {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.long_term_pic_num_l1[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: long_term_pic_num_l1[{}]", i).as_str(),
                        sh.long_term_pic_num_l1[i],
                        63,
                    );
                }
            }
        }
    }

    bitstream_array
}

fn encode_slice_header_ref_pic_list_mvc_modification(sh: &SliceHeader) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    // for B and P/SP slices
    if sh.slice_type % 5 != 2 && sh.slice_type % 5 != 4 {
        bitstream_array.push(match sh.ref_pic_list_modification_flag_l0 {
            true => 1u8,
            _ => 0u8,
        });
        encoder_formatted_print(
            "SH: ref_pic_list_modification_flag_l0",
            sh.ref_pic_list_modification_flag_l0,
            63,
        );

        if sh.ref_pic_list_modification_flag_l0 {
            for i in 0..sh.modification_of_pic_nums_idc_l0.len() {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.modification_of_pic_nums_idc_l0[i] as i32,
                    false,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("SH: modification_of_pic_nums_idc_l0[{}]", i).as_str(),
                    sh.modification_of_pic_nums_idc_l0[i],
                    63,
                );

                if sh.modification_of_pic_nums_idc_l0[i] == 0
                    || sh.modification_of_pic_nums_idc_l0[i] == 1
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.abs_diff_pic_num_minus1_l0[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: abs_diff_pic_num_minus1_l0[{}]", i).as_str(),
                        sh.abs_diff_pic_num_minus1_l0[i],
                        63,
                    );
                } else if sh.modification_of_pic_nums_idc_l0[i] == 2 {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.long_term_pic_num_l0[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: long_term_pic_num_l0[{}]", i).as_str(),
                        sh.long_term_pic_num_l0[i],
                        63,
                    );
                } else if sh.modification_of_pic_nums_idc_l0[i] == 4
                    || sh.modification_of_pic_nums_idc_l0[i] == 5
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.abs_diff_view_idx_minus1_l0[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: abs_diff_view_idx_minus1_l0[{}]", i).as_str(),
                        sh.abs_diff_view_idx_minus1_l0[i],
                        63,
                    );
                }
            }
        }
    }

    // for B slices
    if sh.slice_type % 5 == 1 {
        bitstream_array.push(match sh.ref_pic_list_modification_flag_l1 {
            true => 1u8,
            _ => 0u8,
        });
        encoder_formatted_print(
            "SH: ref_pic_list_modification_flag_l1",
            sh.ref_pic_list_modification_flag_l1,
            63,
        );

        if sh.ref_pic_list_modification_flag_l1 {
            for i in 0..sh.modification_of_pic_nums_idc_l1.len() {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.modification_of_pic_nums_idc_l1[i] as i32,
                    false,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("SH: modification_of_pic_nums_idc_l1[{}]", i).as_str(),
                    sh.modification_of_pic_nums_idc_l1[i],
                    63,
                );

                if sh.modification_of_pic_nums_idc_l1[i] == 0
                    || sh.modification_of_pic_nums_idc_l1[i] == 1
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.abs_diff_pic_num_minus1_l1[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: abs_diff_pic_num_minus1_l1[{}]", i).as_str(),
                        sh.abs_diff_pic_num_minus1_l1[i],
                        63,
                    );
                } else if sh.modification_of_pic_nums_idc_l1[i] == 2 {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.long_term_pic_num_l1[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: long_term_pic_num_l1[{}]", i).as_str(),
                        sh.long_term_pic_num_l1[i],
                        63,
                    );
                } else if sh.modification_of_pic_nums_idc_l1[i] == 4
                    || sh.modification_of_pic_nums_idc_l1[i] == 5
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.abs_diff_view_idx_minus1_l1[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: abs_diff_view_idx_minus1_l1[{}]", i).as_str(),
                        sh.abs_diff_view_idx_minus1_l1[i],
                        63,
                    );
                }
            }
        }
    }

    bitstream_array
}

fn encode_slice_header_pred_weight_table(sh: &SliceHeader, vp: &VideoParameters) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    // luma_log2_weight_denom ue(v)
    bitstream_array.append(&mut exp_golomb_encode_one(
        sh.luma_log2_weight_denom as i32,
        false,
        0,
        false,
    ));
    encoder_formatted_print("SH: luma_log2_weight_denom", sh.luma_log2_weight_denom, 63);

    if vp.chroma_array_type != 0 {
        // chroma_log2_weight_denom ue(v)
        bitstream_array.append(&mut exp_golomb_encode_one(
            sh.chroma_log2_weight_denom as i32,
            false,
            0,
            false,
        ));
        encoder_formatted_print(
            "SH: chroma_log2_weight_denom",
            sh.chroma_log2_weight_denom,
            63,
        );
    }

    // for P, SP, and B slices
    for i in 0..sh.num_ref_idx_l0_active_minus1 + 1 {
        let ii = i as usize;
        // luma_weight_l0_flag u(1)
        bitstream_array.push(match sh.luma_weight_l0_flag[ii] {
            true => 1u8,
            false => 0u8,
        });
        encoder_formatted_print(
            format!("SH: luma_weight_l0_flag[{}]", ii).as_str(),
            sh.luma_weight_l0_flag[ii],
            63,
        );

        if sh.luma_weight_l0_flag[ii] {
            // luma_weight_l0[i] se(v)
            bitstream_array.append(&mut exp_golomb_encode_one(
                sh.luma_weight_l0[ii],
                true,
                0,
                false,
            ));

            encoder_formatted_print(
                format!("SH: luma_weight_l0[{}]", ii).as_str(),
                sh.luma_weight_l0[ii],
                63,
            );

            // luma_offset_l0[i] se(v)
            bitstream_array.append(&mut exp_golomb_encode_one(
                sh.luma_offset_l0[ii],
                true,
                0,
                false,
            ));

            encoder_formatted_print(
                format!("SH: luma_offset_l0[{}]", ii).as_str(),
                sh.luma_offset_l0[ii],
                63,
            );
        }

        if vp.chroma_array_type != 0 {
            // chroma_weight_l0_flag[ii]
            bitstream_array.push(match sh.chroma_weight_l0_flag[ii] {
                true => 1u8,
                false => 0u8,
            });
            encoder_formatted_print(
                format!("SH: chroma_weight_l0_flag[{}]", ii).as_str(),
                sh.chroma_weight_l0_flag[ii],
                63,
            );

            if sh.chroma_weight_l0_flag[ii] {
                for j in 0..2 {
                    // chroma_weight_l0[i][j] se(v)
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.chroma_weight_l0[ii][j],
                        true,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: chroma_weight_l0[{}][{}]", ii, j).as_str(),
                        sh.chroma_weight_l0[ii][j],
                        63,
                    );

                    // chroma_offset_l0[i][j] se(v)
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.chroma_offset_l0[ii][j],
                        true,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: chroma_offset_l0[{}][{}]", ii, j).as_str(),
                        sh.chroma_offset_l0[ii][j],
                        63,
                    );
                }
            }
        }
    }

    // B slices
    if sh.slice_type % 5 == 1 {
        for i in 0..sh.num_ref_idx_l1_active_minus1 + 1 {
            let ii = i as usize;
            // luma_weight_l1_flag u(1)
            bitstream_array.push(match sh.luma_weight_l1_flag[ii] {
                true => 1u8,
                false => 0u8,
            });
            encoder_formatted_print(
                format!("SH: luma_weight_l1_flag[{}]", ii).as_str(),
                sh.luma_weight_l1_flag[ii],
                63,
            );

            if sh.luma_weight_l1_flag[ii] {
                // luma_weight_l1[i] se(v)
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.luma_weight_l1[ii],
                    true,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("SH: luma_weight_l1[{}]", ii).as_str(),
                    sh.luma_weight_l1[ii],
                    63,
                );

                // luma_offset_l1[i] se(v)
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.luma_offset_l1[ii],
                    true,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("SH: luma_offset_l1[{}]", ii).as_str(),
                    sh.luma_offset_l1[ii],
                    63,
                );
            }

            if vp.chroma_array_type != 0 {
                // chroma_weight_l1_flag[ii]
                bitstream_array.push(match sh.chroma_weight_l1_flag[ii] {
                    true => 1u8,
                    false => 0u8,
                });
                encoder_formatted_print(
                    format!("SH: chroma_weight_l1_flag[{}]", ii).as_str(),
                    sh.chroma_weight_l1_flag[ii],
                    63,
                );

                if sh.chroma_weight_l1_flag[ii] {
                    for j in 0..2 {
                        // chroma_weight_l1[i][j] se(v)
                        bitstream_array.append(&mut exp_golomb_encode_one(
                            sh.chroma_weight_l1[ii][j],
                            true,
                            0,
                            false,
                        ));
                        encoder_formatted_print(
                            format!("SH: chroma_weight_l1[{}][{}]", ii, j).as_str(),
                            sh.chroma_weight_l1[ii][j],
                            63,
                        );

                        // chroma_offset_l1[i][j] se(v)
                        bitstream_array.append(&mut exp_golomb_encode_one(
                            sh.chroma_offset_l1[ii][j],
                            true,
                            0,
                            false,
                        ));
                        encoder_formatted_print(
                            format!("SH: chroma_offset_l1[{}][{}]", ii, j).as_str(),
                            sh.chroma_offset_l1[ii][j],
                            63,
                        );
                    }
                }
            }
        }
    }

    bitstream_array
}

fn encode_slice_header_dec_ref_pic_marking(sh: &SliceHeader, vp: &VideoParameters) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    if vp.idr_pic_flag {
        // no_output_of_prior_pics_flag u(1)
        bitstream_array.append(&mut vec![match sh.no_output_of_prior_pics_flag {
            true => 1u8,
            _ => 0u8,
        }]);
        encoder_formatted_print(
            "SH: no_output_of_prior_pics_flag",
            sh.no_output_of_prior_pics_flag,
            63,
        );

        // long_term_reference_flag u(1)
        bitstream_array.append(&mut vec![match sh.long_term_reference_flag {
            true => 1u8,
            _ => 0u8,
        }]);
        encoder_formatted_print(
            "SH: long_term_reference_flag",
            sh.long_term_reference_flag,
            63,
        );
    } else {
        bitstream_array.push(match sh.adaptive_ref_pic_marking_mode_flag {
            true => 1u8,
            false => 0u8,
        });
        encoder_formatted_print(
            "SH: adaptive_ref_pic_marking_mode_flag",
            sh.adaptive_ref_pic_marking_mode_flag,
            63,
        );

        if sh.adaptive_ref_pic_marking_mode_flag {
            for i in 0..sh.memory_management_control_operation.len() {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.memory_management_control_operation[i] as i32,
                    false,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("SH: memory_management_control_operation[{}]", i).as_str(),
                    sh.memory_management_control_operation[i],
                    63,
                );

                if sh.memory_management_control_operation[i] == 1
                    || sh.memory_management_control_operation[i] == 3
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.difference_of_pic_nums_minus1[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: difference_of_pic_nums_minus1[{}]", i).as_str(),
                        sh.difference_of_pic_nums_minus1[i],
                        63,
                    );
                }

                if sh.memory_management_control_operation[i] == 2 {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.long_term_pic_num[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: long_term_pic_num[{}]", i).as_str(),
                        sh.long_term_pic_num[i],
                        63,
                    );
                }

                if sh.memory_management_control_operation[i] == 3
                    || sh.memory_management_control_operation[i] == 6
                {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.long_term_frame_idx[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: long_term_frame_idx[{}]", i).as_str(),
                        sh.long_term_frame_idx[i],
                        63,
                    );
                }

                if sh.memory_management_control_operation[i] == 4 {
                    bitstream_array.append(&mut exp_golomb_encode_one(
                        sh.max_long_term_frame_idx_plus1[i] as i32,
                        false,
                        0,
                        false,
                    ));
                    encoder_formatted_print(
                        format!("SH: max_long_term_frame_idx_plus1[{}]", i).as_str(),
                        sh.max_long_term_frame_idx_plus1[i],
                        63,
                    );
                }
            }
        }
    }

    bitstream_array
}

/// Encodes Slice Header elements and returns a ByteStream of
/// the elements
fn encode_slice_header(
    bitstream_array: &mut Vec<u8>,
    nh: &NALUHeader,
    sh: &SliceHeader,
    s: &SeqParameterSet,
    p: &PicParameterSet,
    vp: &VideoParameters,
) {
    // Follows section 7.3.3 to determine encoding
    let mut res = exp_golomb_encode_one(sh.first_mb_in_slice as i32, false, 0, false);
    bitstream_array.append(&mut res);
    encoder_formatted_print("SH: first_mb_in_slice", sh.first_mb_in_slice, 63);

    let mut res = exp_golomb_encode_one(sh.slice_type as i32, false, 0, false);
    bitstream_array.append(&mut res);
    encoder_formatted_print("SH: slice_type", sh.slice_type, 63);

    let mut res = exp_golomb_encode_one(sh.pic_parameter_set_id as i32, false, 0, false);
    bitstream_array.append(&mut res);
    encoder_formatted_print("SH: pic_parameter_set_id", sh.pic_parameter_set_id, 63);

    if s.separate_colour_plane_flag {
        let mut res = vec![(sh.colour_plane_id & 2) >> 1, sh.colour_plane_id & 1]; // 2-bit unsigned values
        bitstream_array.append(&mut res);
        encoder_formatted_print("SH: colour_plane_id", sh.colour_plane_id, 63);
    }

    // frame_num u(v)
    // length is defined by SPS log2_max_frame_num_minus4 + 4 numbers of bits

    bitstream_array.append(&mut generate_unsigned_binary(
        sh.frame_num,
        (s.log2_max_frame_num_minus4 + 4) as usize,
    ));
    encoder_formatted_print("SH: frame_num", sh.frame_num, 63);

    // field_pic_flag
    if !s.frame_mbs_only_flag {
        bitstream_array.push(match sh.field_pic_flag {
            true => 1u8,
            _ => 0u8,
        });
        encoder_formatted_print("SH: field_pic_flag", sh.field_pic_flag, 63);
        if sh.field_pic_flag {
            bitstream_array.push(match sh.bottom_field_flag {
                true => 1u8,
                _ => 0u8,
            });
            encoder_formatted_print("SH: bottom_field_flag", sh.bottom_field_flag, 63);
        }
    }

    // idr_pic_id ue(v)
    if vp.idr_pic_flag {
        bitstream_array.append(&mut exp_golomb_encode_one(
            sh.idr_pic_id as i32,
            false,
            0,
            false,
        ));
        encoder_formatted_print("SH: idr_pic_id", sh.idr_pic_id, 63);
    }

    if s.pic_order_cnt_type == 0 {
        // pic_order_cnt_lsb u(v)
        // length is defined by SPS log2_max_pic_order_cnt_lsb_minus4 + 4 number of bits
        let mut pic_order_cnt_lsb_vec: Vec<u8> = generate_unsigned_binary(
            sh.pic_order_cnt_lsb,
            s.log2_max_pic_order_cnt_lsb_minus4 as usize + 4,
        );

        bitstream_array.append(&mut pic_order_cnt_lsb_vec);
        encoder_formatted_print("SH: pic_order_cnt_lsb", sh.pic_order_cnt_lsb, 63);

        // delta_pic_order_cnt_bottom se(v)
        if p.bottom_field_pic_order_in_frame_present_flag && !sh.field_pic_flag {
            bitstream_array.append(&mut exp_golomb_encode_one(
                sh.delta_pic_order_cnt_bottom,
                true,
                0,
                false,
            ));
            encoder_formatted_print(
                "SH: delta_pic_order_cnt_bottom",
                sh.delta_pic_order_cnt_bottom,
                63,
            );
        }
    }

    if s.pic_order_cnt_type == 1 && !s.delta_pic_order_always_zero_flag {
        bitstream_array.append(&mut exp_golomb_encode_one(
            sh.delta_pic_order_cnt[0],
            true,
            0,
            false,
        ));
        encoder_formatted_print("SH: delta_pic_order_cnt[0]", sh.delta_pic_order_cnt[0], 63);
        if p.bottom_field_pic_order_in_frame_present_flag && !sh.field_pic_flag {
            bitstream_array.append(&mut exp_golomb_encode_one(
                sh.delta_pic_order_cnt[1],
                true,
                0,
                false,
            ));
            encoder_formatted_print("SH: delta_pic_order_cnt[1]", sh.delta_pic_order_cnt[1], 63);
        }
    }

    // redundant_pic_cnt ue(v)
    if p.redundant_pic_cnt_present_flag {
        bitstream_array.append(&mut exp_golomb_encode_one(
            sh.redundant_pic_cnt as i32,
            false,
            0,
            false,
        ));
        encoder_formatted_print("SH: redundant_pic_cnt", sh.redundant_pic_cnt, 63);
    }

    if is_slice_type(sh.slice_type, "B") {
        bitstream_array.push(match sh.direct_spatial_mv_pred_flag {
            true => 1u8,
            false => 0u8,
        });
        encoder_formatted_print(
            "SH: direct_spatial_mv_pred_flag",
            sh.direct_spatial_mv_pred_flag,
            63,
        );
    }

    if is_slice_type(sh.slice_type, "P")
        || is_slice_type(sh.slice_type, "SP")
        || is_slice_type(sh.slice_type, "B")
    {
        bitstream_array.push(match sh.num_ref_idx_active_override_flag {
            true => 1u8,
            false => 0u8,
        });
        encoder_formatted_print(
            "SH: num_ref_idx_active_override_flag",
            sh.num_ref_idx_active_override_flag,
            63,
        );

        if sh.num_ref_idx_active_override_flag {
            bitstream_array.append(&mut exp_golomb_encode_one(
                sh.num_ref_idx_l0_active_minus1 as i32,
                false,
                0,
                false,
            ));
            encoder_formatted_print(
                "SH: num_ref_idx_l0_active_minus1",
                sh.num_ref_idx_l0_active_minus1,
                63,
            );
            if is_slice_type(sh.slice_type, "B") {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    sh.num_ref_idx_l1_active_minus1 as i32,
                    false,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    "SH: num_ref_idx_l1_active_minus1",
                    sh.num_ref_idx_l1_active_minus1,
                    63,
                );
            }
        }
    }

    if nh.nal_unit_type == 20 || nh.nal_unit_type == 21 {
        // Annex H
        bitstream_array.append(&mut encode_slice_header_ref_pic_list_mvc_modification(sh));
    } else {
        // ref_pic_list_modification()
        bitstream_array.append(&mut encode_slice_header_ref_pic_list_modification(sh));
    }

    if (p.weighted_pred_flag
        && (is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP")))
        || (p.weighted_bipred_idc == 1 && is_slice_type(sh.slice_type, "B"))
    {
        // pred_weight_table()
        bitstream_array.append(&mut encode_slice_header_pred_weight_table(sh, vp));
    }

    if nh.nal_ref_idc != 0 {
        // dec_ref_pic_marking()
        bitstream_array.append(&mut encode_slice_header_dec_ref_pic_marking(sh, vp));
    }

    // cabac_init_idc
    if p.entropy_coding_mode_flag
        && !is_slice_type(sh.slice_type, "I")
        && !is_slice_type(sh.slice_type, "SI")
    {
        bitstream_array.append(&mut exp_golomb_encode_one(
            sh.cabac_init_idc as i32,
            false,
            0,
            false,
        ));
        encoder_formatted_print("SH: cabac_init_idc", sh.cabac_init_idc, 63);
    }

    // slice_qp_delta se(v)
    bitstream_array.append(&mut exp_golomb_encode_one(
        sh.slice_qp_delta,
        true,
        0,
        false,
    ));
    encoder_formatted_print("SH: slice_qp_delta", sh.slice_qp_delta, 63);

    if is_slice_type(sh.slice_type, "SP") || is_slice_type(sh.slice_type, "SI") {
        if is_slice_type(sh.slice_type, "SP") {
            bitstream_array.push(match sh.sp_for_switch_flag {
                true => 1u8,
                false => 0u8,
            });
            encoder_formatted_print("SH: sp_for_switch_flag", sh.sp_for_switch_flag, 63);
        }
        // slice_qs_delta se(v)
        bitstream_array.append(&mut exp_golomb_encode_one(
            sh.slice_qs_delta,
            true,
            0,
            false,
        ));
        encoder_formatted_print("SH: slice_qs_delta", sh.slice_qs_delta, 63);
    }

    if p.deblocking_filter_control_present_flag {
        // disable_deblocking_filter_idc ue(v)
        bitstream_array.append(&mut exp_golomb_encode_one(
            sh.disable_deblocking_filter_idc as i32,
            false,
            0,
            false,
        ));
        encoder_formatted_print(
            "SH: disable_deblocking_filter_idc",
            sh.disable_deblocking_filter_idc,
            63,
        );

        if sh.disable_deblocking_filter_idc != 1 {
            // slice_alpha_c0_offset_div2 se(v)
            bitstream_array.append(&mut exp_golomb_encode_one(
                sh.slice_alpha_c0_offset_div2,
                true,
                0,
                false,
            ));
            encoder_formatted_print(
                "SH: slice_alpha_c0_offset_div2",
                sh.slice_alpha_c0_offset_div2,
                63,
            );
            // slice_beta_offset_div2 se(v)
            bitstream_array.append(&mut exp_golomb_encode_one(
                sh.slice_beta_offset_div2,
                true,
                0,
                false,
            ));
            encoder_formatted_print("SH: slice_beta_offset_div2", sh.slice_beta_offset_div2, 63);
        }
    }

    if p.num_slice_groups_minus1 > 0 && p.slice_group_map_type >= 3 && p.slice_group_map_type <= 5 {
        // number of bits defined by equation 7-35
        let bits_to_write = (((p.pic_size_in_map_units_minus1 + 1)
            / (p.slice_group_change_rate_minus1 + 1)
            + 1) as f64)
            .log2()
            .ceil() as usize;

        let mut slice_group_change_cycle: Vec<u8> =
            generate_unsigned_binary(sh.slice_group_change_cycle, bits_to_write);

        bitstream_array.append(&mut slice_group_change_cycle);
        encoder_formatted_print(
            "SH: slice_group_change_cycle",
            sh.slice_group_change_cycle,
            63,
        );
    }

    // padding bit is 1 due to cabac_alignment_one_bit
    if p.entropy_coding_mode_flag {
        while bitstream_array.len() % 8 != 0 {
            bitstream_array.push(1);
        }
    }
}

/// Encodes the Slice Data, consisting of MacroBlocks and various flags
fn encode_slice_data(
    bitstream_array: &mut Vec<u8>,
    slice: &Slice,
    s: &SeqParameterSet,
    p: &PicParameterSet,
    vp: &VideoParameters,
    silent_mode: bool,
) {
    // Set the bin_counts_in_nal_units to 0 for now
    let mut cs = cabac::initialize_state(0);

    // VP takes into consideration SPS frame_mbs_only into height
    let mut num_macroblocks_to_encode: usize =
        (vp.frame_height_in_mbs * vp.pic_width_in_mbs) as usize;

    // Fields have half the number of macroblocks
    if !s.frame_mbs_only_flag && slice.sh.field_pic_flag {
        num_macroblocks_to_encode /= 2;
    }

    if !silent_mode {
        if num_macroblocks_to_encode > slice.sd.macroblock_vec.len() {
            println!(
                "[WARNING] More macroblocks requested {} than available {}. Slice will end early",
                num_macroblocks_to_encode,
                slice.sd.macroblock_vec.len()
            );
        } else if num_macroblocks_to_encode < slice.sd.macroblock_vec.len() {
            println!("[WARNING] More macroblocks available {} in slice than requested {}. End of Slice Flag encoding may fail", slice.sd.macroblock_vec.len(), num_macroblocks_to_encode);
        }

        if (slice.sh.first_mb_in_slice as usize) >= num_macroblocks_to_encode {
            println!("[WARNING] first_mb_in_slice {} is larger than number of macroblocks to encode {}. Macroblock data may not be read", slice.sh.first_mb_in_slice, num_macroblocks_to_encode);
        } else if (slice.sh.first_mb_in_slice as usize) > 0 {
            println!("[WARNING] non-zero first_mb_in_slice - {}. Not all macroblock data may be read when decoding", slice.sh.first_mb_in_slice);
        }
    }

    // if equal then it's chill
    let mut prev_mb_skipped = false;
    let mut prev_predicted_value = false; // necessary in MBAFF when the top MB is skipped, and the bottom one needs to decode mb_field_decoding_flag
    let mut i = 0;
    while i < num_macroblocks_to_encode {
        if i >= slice.sd.macroblock_vec.len() {
            if !silent_mode {
                println!("[WARNING] exiting early");
            }
            break;
        }
        let mut mb = &slice.sd.macroblock_vec[i];
        debug!(target: "encode","");
        debug!(target: "encode","*********** POC: {} (I/P) MB: {} Slice: 0 Type {} **********", slice.sh.pic_order_cnt_lsb, mb.mb_addr, slice.sh.slice_type % 5);
        debug!(target: "encode","");

        let mut more_data: bool = true;
        if !is_slice_type(slice.sh.slice_type, "I") && !is_slice_type(slice.sh.slice_type, "SI") {
            if p.entropy_coding_mode_flag {
                // For mbaff, we have to get the neighbor based off the predicted mb_field_decoding_flag rather than the actual recovered value.
                // We first calculate the predicted value, then determine whether to flip it or not.
                // We take this approach because randomly generated videos won't know whether to flip or not correctly.

                let mut flip_curr_mb_frame_flag = false;
                if slice.sh.mbaff_frame_flag {
                    // calculate
                    let mut calculated_mb_field_decoding_flag = slice.sd.mb_field_decoding_flag[i];

                    let mut recovered_from_above = false;

                    if slice.sd.macroblock_vec[i].mb_addr % 2 == 1 {
                        debug!(target: "encode","Inferring mb_field_decoding_flag: bottom pair, gonna try to copy from the top pair");
                        let mb_above: MacroBlock;
                        // for mb_b we know that we're the bottom pair, but first_mb_in_slice may mess us up so we just
                        // double check in the index to make sure there's a top macroblock
                        if i < 1 {
                            mb_above = MacroBlock::new();
                        } else {
                            mb_above = slice.sd.macroblock_vec[i - 1].clone();
                        }

                        debug!(target: "encode","mb_above.mb_addr = {}", mb_above.mb_addr);

                        // if available and not skipped, copy it from above
                        if mb_above.available
                            && mb_above.mb_type != MbType::PSkip
                            && mb_above.mb_type != MbType::BSkip
                        {
                            debug!(target: "encode","Gonna copy from above - {}", slice.sd.mb_field_decoding_flag[mb_above.mb_idx] );
                            calculated_mb_field_decoding_flag =
                                slice.sd.mb_field_decoding_flag[mb_above.mb_idx];
                            recovered_from_above = true;
                        } else if mb_above.available
                            && (mb_above.mb_type == MbType::PSkip
                                || mb_above.mb_type == MbType::BSkip)
                        {
                            debug!(target: "encode","Above was skipped, so we'll use its predicted value - {}", prev_predicted_value );
                            calculated_mb_field_decoding_flag = prev_predicted_value;
                            recovered_from_above = true;
                        }
                    }

                    if !recovered_from_above {
                        // check neighbors for their flags
                        let mb_left: MacroBlock;
                        let mb_above: MacroBlock;

                        // if we're the top of the pair
                        if slice.sd.macroblock_vec[i].mb_addr % 2 == 0 {
                            debug!(target: "encode","Inferring mb_field_decoding_flag for top pair of an MBAFF frame");

                            // for the top pair, we first check if there's anything to the left
                            // if on the left most edge, then not ( modulo picture width *2 is equal to 0 or 1 )
                            // or if i is too small, then set to new
                            if (slice.sd.macroblock_vec[i].mb_addr as u32)
                                % (2 * vp.pic_width_in_mbs)
                                < 2
                                || i < 2
                            {
                                debug!(target: "encode","sd.macroblock_vec[i].mb_addr as u32 {}", slice.sd.macroblock_vec[i].mb_addr as u32);
                                debug!(target: "encode","2*vp.pic_width_in_mbs {}", 2*vp.pic_width_in_mbs);
                                debug!(target: "encode","i {}", i);
                                mb_left = MacroBlock::new();
                            } else {
                                mb_left = slice.sd.macroblock_vec[i - 2].clone();
                            }

                            // for mb_above, we need to check if we're at the top most part of the frame, or if not
                            // then get the MB right above us (which should have an odd MB value)
                            if (slice.sd.macroblock_vec[i].mb_addr as u32)
                                < (2 * vp.pic_width_in_mbs)
                                || (i as u32) < (2 * vp.pic_width_in_mbs)
                            {
                                debug!(target: "encode","sd.macroblock_vec[i].mb_addr as u32 {}", slice.sd.macroblock_vec[i].mb_addr as u32);
                                debug!(target: "encode","2*vp.pic_width_in_mbs {}", 2*vp.pic_width_in_mbs);
                                debug!(target: "encode","i {}", i);
                                mb_above = MacroBlock::new();
                            } else {
                                // we do +1 to get the odd MB
                                mb_above = slice.sd.macroblock_vec
                                    [i - (2 * vp.pic_width_in_mbs as usize) + 1]
                                    .clone();
                            }
                        } else {
                            // bottom of the pair
                            debug!(target: "encode","Inferring mb_field_decoding_flag for bottom pair of an MBAFF frame");

                            // first check if there's anything to the left
                            // if on the left most edge, then not ( modulo picture width *2 is equal to 0 or 1 )
                            // or if the i is too small, then set to new
                            if (slice.sd.macroblock_vec[i].mb_addr as u32)
                                % (2 * vp.pic_width_in_mbs)
                                < 2
                                || (i < 2)
                            {
                                debug!(target: "encode","sd.macroblock_vec[i].mb_addr as u32 {}", slice.sd.macroblock_vec[i].mb_addr as u32);
                                debug!(target: "encode","2*vp.pic_width_in_mbs {}", 2*vp.pic_width_in_mbs);
                                debug!(target: "encode","i {}", i);
                                mb_left = MacroBlock::new();
                            } else {
                                mb_left = slice.sd.macroblock_vec[i - 2].clone();
                            }

                            // for mb_above we know that we're the bottom pair, but first_mb_in_slice may mess us up so we just
                            // double check in the index to make sure there's a top macroblock
                            if i < 1 {
                                // if we didn't copy from above before, why would we do so now??
                                mb_above = MacroBlock::new();
                            } else {
                                mb_above = slice.sd.macroblock_vec[i - 1].clone();
                            }
                        }

                        debug!(target: "encode","mb_left.mb_addr {} and mb_left.mb_idx {}", mb_left.mb_addr, mb_left.mb_idx);
                        debug!(target: "encode","mb_above.mb_addr {} and mb_above.mb_idx {}", mb_above.mb_addr, mb_above.mb_idx);

                        if mb_left.available {
                            debug!(target: "encode","Copying from the left - {}", slice.sd.mb_field_decoding_flag[mb_left.mb_idx]);
                            calculated_mb_field_decoding_flag =
                                slice.sd.mb_field_decoding_flag[mb_left.mb_idx];
                        } else if mb_above.available {
                            debug!(target: "encode","Copying from above - {}", slice.sd.mb_field_decoding_flag[mb_above.mb_idx]);
                            calculated_mb_field_decoding_flag =
                                slice.sd.mb_field_decoding_flag[mb_above.mb_idx];
                        } else {
                            debug!(target: "encode","Just pushing false - should only happen on the left-most and top-most rows");
                            calculated_mb_field_decoding_flag = false;
                        }
                    }

                    if calculated_mb_field_decoding_flag != slice.sd.mb_field_decoding_flag[i] {
                        flip_curr_mb_frame_flag = true;
                    }

                    prev_predicted_value = calculated_mb_field_decoding_flag; // update for next run
                }
                cabac::cabac_encode_mb_skip_flag(
                    mb.mb_skip_flag,
                    &slice.sh,
                    bitstream_array,
                    &mut cs,
                    slice
                        .sd
                        .get_neighbor(mb.mb_idx, flip_curr_mb_frame_flag, vp),
                );
                more_data = !mb.mb_skip_flag;
            } else {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    slice.sd.mb_skip_run[i] as i32,
                    false,
                    0,
                    false,
                ));
                encoder_formatted_print(
                    format!("mb_skip_run[{}]", i).as_str(),
                    slice.sd.mb_skip_run[i],
                    63,
                );

                prev_mb_skipped = slice.sd.mb_skip_run[i] > 0; // make sure to set it to False if not skipped

                // when decoding with mb_skip_run we'll insert empty Macroblocks in the vector for simplicity
                // we can just return if that's the case
                if slice.sd.mb_skip_run[i] > 0 {
                    //debug!(target: "encode","*********** Skipping over {} Macroblocks ***********", slice.sd.mb_skip_run[i]);
                    i += slice.sd.mb_skip_run[i] as usize;
                    // for the case when mb_skip_run is at the end
                    if i < num_macroblocks_to_encode && i < slice.sd.macroblock_vec.len() {
                        debug!(target: "encode","*********** POC: {} (I/P) MB: {} Slice: {} Type {} ***********",
                            slice.sh.pic_order_cnt_lsb,
                            slice.sd.macroblock_vec[i].mb_addr,
                            0,
                            slice.sh.slice_type % 5
                        );
                        mb = &slice.sd.macroblock_vec[i];
                    } else {
                        more_data = false;
                    }
                }
            }
        }

        if more_data {
            if slice.sh.mbaff_frame_flag
                && (mb.mb_addr % 2 == 0 || (mb.mb_addr % 2 == 1 && prev_mb_skipped))
            {
                if p.entropy_coding_mode_flag {
                    cabac::cabac_encode_mb_field_decoding_flag(
                        slice.sd.mb_field_decoding_flag[i],
                        &slice.sh,
                        &slice.sd,
                        &vp,
                        i,
                        bitstream_array,
                        &mut cs,
                    );
                } else {
                    bitstream_array.push(match slice.sd.mb_field_decoding_flag[i] {
                        true => 1,
                        false => 0,
                    });
                }
                encoder_formatted_print(
                    "mb_field_decoding_flag",
                    slice.sd.mb_field_decoding_flag[i],
                    63,
                );
            }
            // encode each macroblock
            encode_macroblock(bitstream_array, mb, slice, s, p, vp, &mut cs);
        }

        if p.entropy_coding_mode_flag {
            if !is_slice_type(slice.sh.slice_type, "I") && !is_slice_type(slice.sh.slice_type, "SI")
            {
                prev_mb_skipped = mb.mb_skip_flag;
            }

            // Even macroblocks in mbaff don't have an end_of_slice_flag
            if !slice.sh.mbaff_frame_flag || !(mb.mb_addr % 2 == 0) {
                // flip the expression because we don't care about more_data flag after here
                if i == num_macroblocks_to_encode - 1 {
                    cabac::cabac_encode_end_of_slice_flag(true, bitstream_array, &mut cs);
                } else {
                    cabac::cabac_encode_end_of_slice_flag(
                        slice.sd.end_of_slice_flag[i],
                        bitstream_array,
                        &mut cs,
                    );
                }
            }
        }

        i += 1;
    }
}

/// Follows section 7.3.2.8
pub fn encode_slice(
    nh: &NALUHeader,
    slice: &Slice,
    s: &SeqParameterSet,
    p: &PicParameterSet,
    vp: &VideoParameters,
    silent_mode: bool,
) -> Vec<u8> {
    // this is a bitstream
    let mut bitstream_array: Vec<u8> = Vec::new();

    // encode the header
    encode_slice_header(&mut bitstream_array, nh, &slice.sh, s, p, vp);
    // encode the macroblocks
    encode_slice_data(&mut bitstream_array, slice, s, p, vp, silent_mode);

    //let l = res.len();
    //res[l - 1] = res[l - 1] | 1; // set the last bit to 1

    // push rbsp 1 bit
    // if last bit is 1 then no need to set rbsp bit?
    bitstream_array.push(1);

    bitstream_to_bytestream(bitstream_array, 0)
}

/// Follows section 7.3.2.13
pub fn encode_slice_layer_extension_rbsp(
    nh: &NALUHeader,
    slice: &Slice,
    s: &SubsetSPS,
    p: &PicParameterSet,
    vp: &VideoParameters,
    silent_mode: bool,
) -> Vec<u8> {
    //let mut bitstream_array = Vec::new();

    // TODO: Uncomment whenever implementation below is done

    //if nh.svc_extension_flag {
    //    bitstream_array.append(&mut encode_slice_header_in_scalable_extension(nh, &slice.sh, s, p, vp)); // Annex G
    //    let slice_skip_flag = true;
    //    if !slice_skip_flag {
    //        bitstream_array.append(&mut encode_slice_data_in_scalable_extension()); // Annex G
    //    }
    //} else if nh.avc_3d_extension_flag {
    //    bitstream_array.append(&mut encode_slice_header_in_3davc_extension(nh, &slice.sh, s, p, vp)); // Annex J
    //    bitstream_array.append(&mut encode_slice_data_in_3davc_extension()); // Annex J
    //} else {
    encode_slice(nh, slice, &s.sps, p, vp, silent_mode)
    //}

    // rbsp trailing bit
    //bitstream_array.push(1);
    //
    //return bitstream_to_bytestream(bitstream_array, 0);
}

#[allow(dead_code)]
fn encode_slice_header_in_scalable_extension(
    _nh: &NALUHeader,
    _sh: &SliceHeader,
    _s: &SubsetSPS,
    _p: &PicParameterSet,
    _vp: &VideoParameters,
) -> Vec<u8> {
    let bitstream_array = Vec::new();
    // TODO: Annex G
    println!("encode_slice_header_in_scalable_extension - not yet supported");
    bitstream_array
}

#[allow(dead_code)]
fn encode_slice_data_in_scalable_extension() -> Vec<u8> {
    let bitstream_array = Vec::new();
    // TODO: Annex G
    println!("encode_slice_data_in_scalable_extension - not yet supported");
    bitstream_array
}

#[allow(dead_code)]
fn encode_slice_header_in_3davc_extension(
    _nh: &NALUHeader,
    _sh: &SliceHeader,
    _s: &SubsetSPS,
    _p: &PicParameterSet,
    _vp: &VideoParameters,
) -> Vec<u8> {
    let bitstream_array = Vec::new();
    // TODO: Annex J
    println!("encode_slice_header_in_3davc_extension - not yet supported");
    bitstream_array
}

#[allow(dead_code)]
fn encode_slice_data_in_3davc_extension() -> Vec<u8> {
    let bitstream_array = Vec::new();
    // TODO: Annex J
    println!("encode_slice_data_in_3davc_extension - not yet supported");
    bitstream_array
}
