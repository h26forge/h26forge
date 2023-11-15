//! Parameter Set (SPS, PPS, VUI, extensions) syntax element decoding.

use crate::common::data_structures::AVC3DSPSExtension;
use crate::common::data_structures::HRDParameters;
use crate::common::data_structures::MVCDSPSExtension;
use crate::common::data_structures::MVCDVUIParameters;
use crate::common::data_structures::MVCSPSExtension;
use crate::common::data_structures::MVCVUIParameters;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::SPSExtension;
use crate::common::data_structures::SVCSPSExtension;
use crate::common::data_structures::SVCVUIParameters;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::SubsetSPS;
use crate::common::data_structures::VUIParameters;
use crate::common::helper::decoder_formatted_print;
use crate::common::helper::ByteStream;
use crate::decoder::expgolomb::exp_golomb_decode_one_wrapper;
use log::debug;

/// Described in 7.3.2.3 -- Picture Parameter Set
pub fn decode_pic_parameter_set(
    bs: &mut ByteStream,
    spses: &Vec<SeqParameterSet>,
    subset_spses: &Vec<SubsetSPS>,
) -> PicParameterSet {
    let mut res = PicParameterSet::new();
    res.available = true;

    res.pic_parameter_set_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("PPS: pic_parameter_set_id", &res.pic_parameter_set_id, 63);

    res.seq_parameter_set_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("PPS: seq_parameter_set_id", &res.seq_parameter_set_id, 63);

    let mut cur_sps_wrapper: Option<&SeqParameterSet> = None;

    for i in (0..spses.len()).rev() {
        if spses[i].seq_parameter_set_id == res.seq_parameter_set_id {
            cur_sps_wrapper = Some(&spses[i]);
            break;
        }
    }

    let s: &SeqParameterSet;
    match cur_sps_wrapper {
        Some(x) => s = x,
        _ => {
            // try subset sps
            for i in (0..subset_spses.len()).rev() {
                if subset_spses[i].sps.seq_parameter_set_id == res.seq_parameter_set_id {
                    cur_sps_wrapper = Some(&subset_spses[i].sps);
                    break;
                }
            }

            match cur_sps_wrapper {
                Some(x) => {
                    s = x;
                    res.is_subset_pps = true;
                }
                _ => panic!(
                    "decode_pic_parameter_set - SPS or SubsetSPS with id {} not found",
                    res.seq_parameter_set_id
                ),
            }
        }
    }

    res.entropy_coding_mode_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "PPS: entropy_coding_mode_flag",
        &res.entropy_coding_mode_flag,
        63,
    );
    println!(
        "\t PPS: entropy_coding_mode_flag {}",
        res.entropy_coding_mode_flag
    );

    res.bottom_field_pic_order_in_frame_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "PPS: bottom_field_pic_order_in_frame_present_flag",
        &res.bottom_field_pic_order_in_frame_present_flag,
        63,
    );

    res.num_slice_groups_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "PPS: num_slice_groups_minus1",
        &res.num_slice_groups_minus1,
        63,
    );

    if res.num_slice_groups_minus1 > 0 {
        println!("[WARNING] Correct FMO decoding/encoding not yet completely available - issues may arise");
        res.slice_group_map_type = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print("PPS: slice_group_map_type", &res.slice_group_map_type, 63);

        if res.slice_group_map_type == 0 {
            for i in 0..=res.num_slice_groups_minus1 {
                res.run_length_minus1
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                decoder_formatted_print(
                    "PPS: run_length_minus1",
                    &res.run_length_minus1[i as usize],
                    63,
                );
            }
        } else if res.slice_group_map_type == 2 {
            for i in 0..res.num_slice_groups_minus1 {
                res.top_left
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                decoder_formatted_print("PPS: top_left", &res.top_left[i as usize], 63);

                res.bottom_right
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                decoder_formatted_print("PPS: bottom_right", &res.bottom_right[i as usize], 63);
            }
        } else if res.slice_group_map_type == 3
            || res.slice_group_map_type == 4
            || res.slice_group_map_type == 5
        {
            res.slice_group_change_direction_flag = 1 == bs.read_bits(1);
            decoder_formatted_print(
                "PPS: slice_group_change_direction_flag",
                &res.slice_group_change_direction_flag,
                63,
            );

            res.slice_group_change_rate_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
            decoder_formatted_print(
                "PPS: slice_group_change_rate_minus1",
                &res.slice_group_change_rate_minus1,
                63,
            );
        } else if res.slice_group_map_type == 6 {
            res.pic_size_in_map_units_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
            decoder_formatted_print(
                "PPS: pic_size_in_map_units_minus1",
                &res.pic_size_in_map_units_minus1,
                63,
            );

            let bits_to_read = ((res.num_slice_groups_minus1 + 1) as f64).log2().ceil() as u8;
            for _ in 0..=res.pic_size_in_map_units_minus1 {
                res.slice_group_id.push(bs.read_bits(bits_to_read));
            }
            decoder_formatted_print("PPS: slice_group_id", &res.slice_group_id, 63);
        }
    }

    res.num_ref_idx_l0_default_active_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "PPS: num_ref_idx_l0_default_active_minus1",
        &res.num_ref_idx_l0_default_active_minus1,
        63,
    );

    res.num_ref_idx_l1_default_active_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "PPS: num_ref_idx_l1_default_active_minus1",
        &res.num_ref_idx_l1_default_active_minus1,
        63,
    );

    res.weighted_pred_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("PPS: weighted_pred_flag", &res.weighted_pred_flag, 63);

    res.weighted_bipred_idc = bs.read_bits(2) as u8;
    decoder_formatted_print("PPS: weighted_bipred_idc", &res.weighted_bipred_idc, 63);

    res.pic_init_qp_minus26 = exp_golomb_decode_one_wrapper(bs, true, 0);
    decoder_formatted_print("PPS: pic_init_qp_minus26", &res.pic_init_qp_minus26, 63);

    res.pic_init_qs_minus26 = exp_golomb_decode_one_wrapper(bs, true, 0);
    decoder_formatted_print("PPS: pic_init_qs_minus26", &res.pic_init_qs_minus26, 63);

    res.chroma_qp_index_offset = exp_golomb_decode_one_wrapper(bs, true, 0);
    decoder_formatted_print(
        "PPS: chroma_qp_index_offset",
        &res.chroma_qp_index_offset,
        63,
    );

    res.deblocking_filter_control_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "PPS: deblocking_filter_control_present_flag",
        &res.deblocking_filter_control_present_flag,
        63,
    );

    res.constrained_intra_pred_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "PPS: constrained_intra_pred_flag",
        &res.constrained_intra_pred_flag,
        63,
    );

    res.redundant_pic_cnt_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "PPS: redundant_pic_cnt_present_flag",
        &res.redundant_pic_cnt_present_flag,
        63,
    );

    // rbsp_more_data()
    if bs.more_data() {
        res.more_data_flag = true;
        res.transform_8x8_mode_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "PPS: transform_8x8_mode_flag",
            &res.transform_8x8_mode_flag,
            63,
        );

        res.pic_scaling_matrix_present_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "PPS: pic_scaling_matrix_present_flag",
            &res.pic_scaling_matrix_present_flag,
            63,
        );

        if res.pic_scaling_matrix_present_flag {
            let max_val = 6 + match s.chroma_format_idc != 3 {
                true => 2,
                false => 6,
            } * match res.transform_8x8_mode_flag {
                true => 1,
                false => 0,
            };
            for i in 0..max_val {
                res.pic_scaling_list_present_flag.push(1 == bs.read_bits(1));
                decoder_formatted_print(
                    "PPS: pic_scaling_list_present_flag",
                    &res.pic_scaling_list_present_flag[i],
                    63,
                );

                // ensure that each i has a value
                res.delta_scale_4x4.push(Vec::new());
                res.scaling_list_4x4.push(Vec::new());
                res.use_default_scaling_matrix_4x4.push(false);

                res.delta_scale_8x8.push(Vec::new());
                res.scaling_list_8x8.push(Vec::new());
                res.use_default_scaling_matrix_8x8.push(false);

                if res.pic_scaling_list_present_flag[i] {
                    if i < 6 {
                        decode_scaling_list(
                            bs,
                            &mut res.delta_scale_4x4[i],
                            &mut res.scaling_list_4x4[i],
                            16,
                            &mut res.use_default_scaling_matrix_4x4[i],
                        );
                    } else {
                        decode_scaling_list(
                            bs,
                            &mut res.delta_scale_8x8[i],
                            &mut res.scaling_list_8x8[i],
                            64,
                            &mut res.use_default_scaling_matrix_8x8[i],
                        );
                    }
                }
            }
        }

        res.second_chroma_qp_index_offset = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print(
            "PPS: second_chroma_qp_index_offset",
            &res.second_chroma_qp_index_offset,
            63,
        );
    }

    res
}

/// Described in 7.3.2.1.1 -- Sequence Parameter Set
pub fn decode_seq_parameter_set(bs: &mut ByteStream) -> SeqParameterSet {
    let mut res: SeqParameterSet = SeqParameterSet::new();

    res.available = true;

    res.profile_idc = bs.read_bits(8) as u8;
    decoder_formatted_print("SPS: profile_idc", &res.profile_idc, 63);
    println!("\t SPS: profile_idc {}", res.profile_idc);

    res.constraint_set0_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: constraint_set0_flag", &res.constraint_set0_flag, 63);

    res.constraint_set1_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: constraint_set1_flag", &res.constraint_set1_flag, 63);

    res.constraint_set2_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: constraint_set2_flag", &res.constraint_set2_flag, 63);

    res.constraint_set3_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: constraint_set3_flag", &res.constraint_set3_flag, 63);

    res.constraint_set4_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: constraint_set4_flag", &res.constraint_set4_flag, 63);

    res.constraint_set5_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: constraint_set5_flag", &res.constraint_set5_flag, 63);

    res.reserved_zero_2bits = bs.read_bits(2) as u8;
    decoder_formatted_print("SPS: reserved_zero_2bits", &res.reserved_zero_2bits, 63);

    // 7.4.2.1.1
    //assert_eq!(res.reserved_zero_2bits, 0);

    res.level_idc = bs.read_bits(8) as u8;
    decoder_formatted_print("SPS: level_idc", &res.level_idc, 63);

    res.seq_parameter_set_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    //assert!(res.seq_parameter_set_id < 32);
    decoder_formatted_print("SPS: seq_parameter_set_id", &res.seq_parameter_set_id, 63);

    if res.profile_idc == 100
        || res.profile_idc == 110
        || res.profile_idc == 122
        || res.profile_idc == 244
        || res.profile_idc == 44
        || res.profile_idc == 83
        || res.profile_idc == 86
        || res.profile_idc == 118
        || res.profile_idc == 128
        || res.profile_idc == 138
        || res.profile_idc == 139
        || res.profile_idc == 134
        || res.profile_idc == 135
    {
        res.chroma_format_idc = exp_golomb_decode_one_wrapper(bs, false, 0) as u8;

        // Not-spec compliant
        if res.chroma_format_idc >= 4 {
            debug!(target: "decode","[WARNING] res.chroma_format_idc >= 4");
        }

        decoder_formatted_print("SPS: chroma_format_idc", &res.chroma_format_idc, 63);
        println!("\t SPS: chroma_format_idc {}", res.chroma_format_idc);

        if res.chroma_format_idc == 3 {
            res.separate_colour_plane_flag = 1 == bs.read_bits(1);
            decoder_formatted_print(
                "SPS: separate_colour_plane_flag",
                &res.separate_colour_plane_flag,
                63,
            );
        }

        res.bit_depth_luma_minus8 = exp_golomb_decode_one_wrapper(bs, false, 0) as u8;

        // Not-spec compliant
        if res.bit_depth_luma_minus8 >= 7 {
            debug!(target: "decode","[WARNING] res.bit_depth_luma_minus8 >= 7");
        }

        decoder_formatted_print("SPS: bit_depth_luma_minus8", &res.bit_depth_luma_minus8, 63);
        println!(
            "\t SPS: bit_depth_luma_minus8 {}",
            res.bit_depth_luma_minus8
        );

        res.bit_depth_chroma_minus8 = exp_golomb_decode_one_wrapper(bs, false, 0) as u8;
        // Not-spec compliant
        if res.bit_depth_chroma_minus8 >= 7 {
            debug!(target: "decode","[WARNING] res.bit_depth_chroma_minus8 >= 7");
        }

        decoder_formatted_print(
            "SPS: bit_depth_chroma_minus8",
            &res.bit_depth_chroma_minus8,
            63,
        );
        println!(
            "\t SPS: bit_depth_chroma_minus8 {}",
            res.bit_depth_chroma_minus8
        );

        res.qpprime_y_zero_transform_bypass_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SPS: qpprime_y_zero_transform_bypass_flag",
            &res.qpprime_y_zero_transform_bypass_flag,
            63,
        );

        res.seq_scaling_matrix_present_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SPS: seq_scaling_matrix_present_flag",
            &res.seq_scaling_matrix_present_flag,
            63,
        );

        if res.seq_scaling_matrix_present_flag {
            let cur_max = match res.chroma_format_idc != 3 {
                true => 8,
                _ => 12,
            };

            for i in 0..cur_max {
                res.seq_scaling_list_present_flag.push(1 == bs.read_bits(1));
                decoder_formatted_print(
                    "SPS: seq_scaling_list_present_flag",
                    &res.seq_scaling_list_present_flag[i],
                    63,
                );

                // ensure that each i has a value
                res.delta_scale_4x4.push(Vec::new());
                res.scaling_list_4x4.push(Vec::new());
                res.use_default_scaling_matrix_4x4.push(false);

                res.delta_scale_8x8.push(Vec::new());
                res.scaling_list_8x8.push(Vec::new());
                res.use_default_scaling_matrix_8x8.push(false);

                if res.seq_scaling_list_present_flag[i] {
                    if i < 6 {
                        decode_scaling_list(
                            bs,
                            &mut res.delta_scale_4x4[i],
                            &mut res.scaling_list_4x4[i],
                            16,
                            &mut res.use_default_scaling_matrix_4x4[i],
                        );
                    } else {
                        decode_scaling_list(
                            bs,
                            &mut res.delta_scale_8x8[i],
                            &mut res.scaling_list_8x8[i],
                            64,
                            &mut res.use_default_scaling_matrix_8x8[i],
                        );
                    }
                }
            }
        }
    }

    res.log2_max_frame_num_minus4 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "SPS: log2_max_frame_num_minus4",
        &res.log2_max_frame_num_minus4,
        63,
    );

    res.pic_order_cnt_type = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("SPS: pic_order_cnt_type", &res.pic_order_cnt_type, 63);

    if res.pic_order_cnt_type == 0 {
        res.log2_max_pic_order_cnt_lsb_minus4 = exp_golomb_decode_one_wrapper(bs, false, 0) as u8;
        decoder_formatted_print(
            "SPS: log2_max_pic_order_cnt_lsb_minus4",
            &res.log2_max_pic_order_cnt_lsb_minus4,
            63,
        );
    } else if res.pic_order_cnt_type == 1 {
        res.delta_pic_order_always_zero_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SPS: delta_pic_order_always_zero_flag",
            &res.delta_pic_order_always_zero_flag,
            63,
        );

        res.offset_for_non_ref_pic = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print(
            "SPS: offset_for_non_ref_pic",
            &res.offset_for_non_ref_pic,
            63,
        );

        res.offset_for_top_to_bottom_field = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print(
            "SPS: offset_for_top_to_bottom_field",
            &res.offset_for_top_to_bottom_field,
            63,
        );

        res.num_ref_frames_in_pic_order_cnt_cycle =
            exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SPS: num_ref_frames_in_pic_order_cnt_cycle",
            &res.num_ref_frames_in_pic_order_cnt_cycle,
            63,
        );

        for i in 0..res.num_ref_frames_in_pic_order_cnt_cycle {
            res.offset_for_ref_frame
                .push(exp_golomb_decode_one_wrapper(bs, true, 0));
            decoder_formatted_print(
                "SPS: offset_for_ref_frame",
                &res.offset_for_ref_frame[i as usize],
                63,
            );
        }
    }

    res.max_num_ref_frames = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("SPS: max_num_ref_frames", &res.max_num_ref_frames, 63);

    res.gaps_in_frame_num_value_allowed_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SPS: gaps_in_frame_num_value_allowed_flag",
        &res.gaps_in_frame_num_value_allowed_flag,
        63,
    );

    res.pic_width_in_mbs_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "SPS: pic_width_in_mbs_minus1",
        &res.pic_width_in_mbs_minus1,
        63,
    );

    res.pic_height_in_map_units_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "SPS: pic_height_in_map_units_minus1",
        &res.pic_height_in_map_units_minus1,
        63,
    );

    res.frame_mbs_only_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: frame_mbs_only_flag", &res.frame_mbs_only_flag, 63);

    if !res.frame_mbs_only_flag {
        res.mb_adaptive_frame_field_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SPS: mb_adaptive_frame_field_flag",
            &res.mb_adaptive_frame_field_flag,
            63,
        );
    }

    res.direct_8x8_inference_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SPS: direct_8x8_inference_flag",
        &res.direct_8x8_inference_flag,
        63,
    );

    res.frame_cropping_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("SPS: frame_cropping_flag", &res.frame_cropping_flag, 63);

    if res.frame_cropping_flag {
        res.frame_crop_left_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SPS: frame_crop_left_offset",
            &res.frame_crop_left_offset,
            63,
        );

        res.frame_crop_right_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SPS: frame_crop_right_offset",
            &res.frame_crop_right_offset,
            63,
        );

        res.frame_crop_top_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print("SPS: frame_crop_top_offset", &res.frame_crop_top_offset, 63);

        res.frame_crop_bottom_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SPS: frame_crop_bottom_offset",
            &res.frame_crop_bottom_offset,
            63,
        );
    }

    res.vui_parameters_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SPS: vui_parameters_present_flag",
        &res.vui_parameters_present_flag,
        63,
    );

    if res.vui_parameters_present_flag {
        res.vui_parameters = decode_vui_parameters(bs);
    }

    res
}

/// Described in 7.3.2.1.1.1 -- Scaling list
fn decode_scaling_list(
    bs: &mut ByteStream,
    delta_scaling_list: &mut Vec<i32>,
    scaling_list: &mut Vec<i32>,
    size_of_scaling_list: usize,
    use_default_scaling_matrix_flag: &mut bool,
) {
    let mut last_scale = 8;
    let mut next_scale = 8;

    let mut udsm_flag: bool = false;

    for j in 0..size_of_scaling_list {
        if next_scale != 0 {
            let delta_scale = exp_golomb_decode_one_wrapper(bs, true, 0);
            delta_scaling_list.push(delta_scale);
            decoder_formatted_print("   : delta_sl", &delta_scale, 63);

            next_scale = (last_scale + delta_scale + 256) % 256;
            udsm_flag = j == 0 && next_scale == 0;
        } else {
            delta_scaling_list.push(0); // to make sure indices line up
        }
        scaling_list.push(match next_scale == 0 {
            true => last_scale,
            false => next_scale,
        });
        last_scale = scaling_list[j];
    }

    *use_default_scaling_matrix_flag = udsm_flag;
}

/// Described in G.7.3.2.1.4 -- Sequence Parameter Set SVC extension
fn decode_sps_svc_extension(chroma_array_type: u8, bs: &mut ByteStream) -> SVCSPSExtension {
    let mut res = SVCSPSExtension::new();

    res.inter_layer_deblocking_filter_control_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SVC SPS: num_views_minus1",
        &res.inter_layer_deblocking_filter_control_present_flag,
        63,
    );
    res.extended_spatial_scalability_idc = bs.read_bits(2) as u8;
    decoder_formatted_print(
        "SVC SPS: extended_spatial_scalability_idc",
        &res.extended_spatial_scalability_idc,
        63,
    );

    if chroma_array_type == 1 || chroma_array_type == 2 {
        res.chroma_phase_x_plus1_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SVC SPS: chroma_phase_x_plus1_flag",
            &res.chroma_phase_x_plus1_flag,
            63,
        );
    }
    if chroma_array_type == 1 {
        res.chroma_phase_y_plus1 = bs.read_bits(2) as u8;
        decoder_formatted_print(
            "SVC SPS: chroma_phase_y_plus1",
            &res.chroma_phase_y_plus1,
            63,
        );
    }

    if res.extended_spatial_scalability_idc == 1 {
        if chroma_array_type > 0 {
            res.seq_ref_layer_chroma_phase_x_plus1_flag = 1 == bs.read_bits(1);
            decoder_formatted_print(
                "SVC SPS: seq_ref_layer_chroma_phase_x_plus1_flag",
                &res.seq_ref_layer_chroma_phase_x_plus1_flag,
                63,
            );

            res.seq_ref_layer_chroma_phase_y_plus1 = bs.read_bits(2) as u8;
            decoder_formatted_print(
                "SVC SPS: seq_ref_layer_chroma_phase_y_plus1",
                &res.seq_ref_layer_chroma_phase_y_plus1,
                63,
            );
        }
        res.seq_scaled_ref_layer_left_offset = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_left_offset",
            &res.seq_scaled_ref_layer_left_offset,
            63,
        );

        res.seq_scaled_ref_layer_top_offset = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_top_offset",
            &res.seq_scaled_ref_layer_top_offset,
            63,
        );

        res.seq_scaled_ref_layer_right_offset = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_right_offset",
            &res.seq_scaled_ref_layer_right_offset,
            63,
        );

        res.seq_scaled_ref_layer_bottom_offset = exp_golomb_decode_one_wrapper(bs, true, 0);
        decoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_bottom_offset",
            &res.seq_scaled_ref_layer_bottom_offset,
            63,
        );
    }
    res.seq_tcoeff_level_prediction_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SVC SPS: seq_tcoeff_level_prediction_flag",
        &res.seq_tcoeff_level_prediction_flag,
        63,
    );

    if res.seq_tcoeff_level_prediction_flag {
        res.adaptive_tcoeff_level_prediction_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SVC SPS: adaptive_tcoeff_level_prediction_flag",
            &res.adaptive_tcoeff_level_prediction_flag,
            63,
        );
    }
    res.slice_header_restriction_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SVC SPS: slice_header_restriction_flag",
        &res.slice_header_restriction_flag,
        63,
    );

    res
}

/// Described in H.7.3.2.1.4 -- Sequence Parameter Set MVC extension
fn decode_sps_mvc_extension(
    profile_idc: u8,
    frame_mbs_only_flag: bool,
    bs: &mut ByteStream,
) -> MVCSPSExtension {
    let mut res = MVCSPSExtension::new();

    res.num_views_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as usize;
    decoder_formatted_print("MVC SPS: num_views_minus1", &res.num_views_minus1, 63);
    for _ in 0..=res.num_views_minus1 {
        res.view_id
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
    }
    decoder_formatted_print("MVC SPS: res.view_id[]", &res.view_id, 63);

    // 0th index is skipped
    res.num_anchor_refs_l0.push(0);
    res.anchor_refs_l0.push(Vec::new());
    res.num_anchor_refs_l1.push(0);
    res.anchor_refs_l1.push(Vec::new());

    for i in 1..=res.num_views_minus1 {
        res.num_anchor_refs_l0
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

        res.anchor_refs_l0.push(Vec::new());
        for _ in 0..res.num_anchor_refs_l0[i] {
            res.anchor_refs_l0[i].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        }
        res.num_anchor_refs_l1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        res.anchor_refs_l1.push(Vec::new());
        for _ in 0..res.num_anchor_refs_l1[i] {
            res.anchor_refs_l1[i].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        }
    }
    decoder_formatted_print("MVC SPS: num_anchor_refs_l0", &res.num_anchor_refs_l0, 63);
    decoder_formatted_print("MVC SPS: num_anchor_refs_l1", &res.num_anchor_refs_l1, 63);

    // push this into the 0th position
    res.num_non_anchor_refs_l0.push(0);
    res.non_anchor_refs_l0.push(Vec::new());
    res.num_non_anchor_refs_l1.push(0);
    res.non_anchor_refs_l1.push(Vec::new());

    for i in 1..=res.num_views_minus1 {
        res.num_non_anchor_refs_l0
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        res.non_anchor_refs_l0.push(Vec::new());
        for _ in 0..res.num_non_anchor_refs_l0[i] {
            res.non_anchor_refs_l0[i].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        }
        res.num_non_anchor_refs_l1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        res.non_anchor_refs_l1.push(Vec::new());
        for _ in 0..res.num_non_anchor_refs_l1[i] {
            res.non_anchor_refs_l1[i].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        }
    }
    decoder_formatted_print(
        "MVC SPS: num_non_anchor_refs_l0",
        &res.num_non_anchor_refs_l0,
        63,
    );
    decoder_formatted_print(
        "MVC SPS: num_non_anchor_refs_l1",
        &res.num_non_anchor_refs_l1,
        63,
    );

    res.num_level_values_signalled_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as usize;
    decoder_formatted_print(
        "MVC SPS: num_level_values_signalled_minus1",
        &res.num_level_values_signalled_minus1,
        63,
    );
    for i in 0..=res.num_level_values_signalled_minus1 {
        res.level_idc.push(bs.read_bits(8) as u8);
        res.num_applicable_ops_minus1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as usize);

        // insert new
        res.applicable_op_temporal_id.push(Vec::new());
        res.applicable_op_num_target_views_minus1.push(Vec::new());
        res.applicable_op_target_view_id.push(Vec::new());
        res.applicable_op_num_views_minus1.push(Vec::new());
        for j in 0..=res.num_applicable_ops_minus1[i] {
            res.applicable_op_temporal_id[i].push(bs.read_bits(3) as u8);
            res.applicable_op_num_target_views_minus1[i]
                .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

            //insert new
            res.applicable_op_target_view_id[i].push(Vec::new());
            for _ in 0..=res.applicable_op_num_target_views_minus1[i][j] {
                res.applicable_op_target_view_id[i][j]
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            }
            res.applicable_op_num_views_minus1[i]
                .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        }
    }
    decoder_formatted_print("MVC SPS: level_idc[]", &res.level_idc, 63);
    decoder_formatted_print(
        "MVC SPS: num_applicable_ops_minus1[]",
        &res.num_applicable_ops_minus1,
        63,
    );
    decoder_formatted_print(
        "MVC SPS: applicable_op_temporal_id[][]",
        &res.applicable_op_temporal_id,
        63,
    );
    decoder_formatted_print(
        "MVC SPS: applicable_op_num_target_views_minus1[][]",
        &res.applicable_op_num_target_views_minus1,
        63,
    );
    decoder_formatted_print(
        "MVC SPS: applicable_op_target_view_id[][][]",
        &res.applicable_op_target_view_id,
        63,
    );
    decoder_formatted_print(
        "MVC SPS: applicable_op_num_views_minus1[][]",
        &res.applicable_op_num_views_minus1,
        63,
    );

    if profile_idc == 134 {
        res.mfc_format_idc = bs.read_bits(6) as u8;
        decoder_formatted_print("MVC SPS: mfc_format_idc", &res.mfc_format_idc, 63);
        if res.mfc_format_idc == 0 || res.mfc_format_idc == 1 {
            res.default_grid_position_flag = 1 == bs.read_bits(1);
            decoder_formatted_print(
                "MVC SPS: default_grid_position_flag",
                &res.default_grid_position_flag,
                63,
            );
            if !res.default_grid_position_flag {
                res.view0_grid_position_x = bs.read_bits(4) as u8;
                decoder_formatted_print(
                    "MVC SPS: view0_grid_position_x",
                    &res.view0_grid_position_x,
                    63,
                );
                res.view0_grid_position_y = bs.read_bits(4) as u8;
                decoder_formatted_print(
                    "MVC SPS: view0_grid_position_y",
                    &res.view0_grid_position_y,
                    63,
                );
                res.view1_grid_position_x = bs.read_bits(4) as u8;
                decoder_formatted_print(
                    "MVC SPS: view1_grid_position_x",
                    &res.view1_grid_position_x,
                    63,
                );
                res.view1_grid_position_y = bs.read_bits(4) as u8;
                decoder_formatted_print(
                    "MVC SPS: view1_grid_position_y",
                    &res.view1_grid_position_y,
                    63,
                );
            }
        }
        res.rpu_filter_enabled_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "MVC SPS: rpu_filter_enabled_flag",
            &res.rpu_filter_enabled_flag,
            63,
        );
        if !frame_mbs_only_flag {
            res.rpu_field_processing_flag = 1 == bs.read_bits(1);
            decoder_formatted_print(
                "MVC SPS: rpu_field_processing_flag",
                &res.rpu_field_processing_flag,
                63,
            );
        }
    }

    res
}

/// Described in I.7.3.2.1.5 -- Sequence Parameter Set MVCD extension
fn decode_sps_mvcd_extension(bs: &mut ByteStream) -> MVCDSPSExtension {
    let mut res = MVCDSPSExtension::new();
    res.num_views_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;

    for i in 0..=res.num_views_minus1 {
        res.view_id.push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        res.depth_view_present_flag.push(1 == bs.read_bits(1));
        res.depth_view_id[res.num_depth_views as usize] = res.view_id[i as usize];
        res.num_depth_views += match res.depth_view_present_flag[i as usize] {true => 1, false => 0};
        res.texture_view_present_flag.push(1 == bs.read_bits(1));
    }
    for i in 1..=res.num_views_minus1 {
        res.num_anchor_refs_l0.push(0);
        res.num_anchor_refs_l1.push(0);
        res.anchor_ref_l0.push(Vec::new());
        res.anchor_ref_l1.push(Vec::new());
        if res.depth_view_present_flag[i as usize] {
            res.num_anchor_refs_l0[i as usize] = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
            for _ in 0..res.num_anchor_refs_l0[i as usize] {
                res.anchor_ref_l0[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            }
            res.num_anchor_refs_l1[i as usize] = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
            for _ in 0..res.num_anchor_refs_l1[i as usize] {
                res.anchor_ref_l1[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            }
        }
    }

    for i in 1..=res.num_views_minus1 {
        res.num_non_anchor_refs_l0.push(0);
        res.num_non_anchor_refs_l1.push(0);
        res.non_anchor_ref_l0.push(Vec::new());
        res.non_anchor_ref_l1.push(Vec::new());
        if res.depth_view_present_flag[i as usize] {
            res.num_non_anchor_refs_l0[i as usize] = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
            for _ in 0..res.num_non_anchor_refs_l0[i as usize] {
                res.non_anchor_ref_l0[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            }
            res.num_non_anchor_refs_l1[i as usize] = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
            for _ in 0..res.num_non_anchor_refs_l1[i as usize] {
                res.non_anchor_ref_l1[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            }
        }
    }
    res.num_level_values_signalled_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    for i in 0..=res.num_level_values_signalled_minus1 {
        res.level_idc.push(bs.read_bits(8) as u8);
        res.num_applicable_ops_minus1.push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        res.applicable_op_temporal_id.push(Vec::new());
        res.applicable_op_num_target_views_minus1.push(Vec::new());
        res.applicable_op_target_view_id.push(Vec::new());
        res.applicable_op_depth_flag.push(Vec::new());
        res.applicable_op_texture_flag.push(Vec::new());
        res.applicable_op_num_texture_views_minus1.push(Vec::new());
        res.applicable_op_num_depth_views.push(Vec::new());
        for j in 0..=res.num_applicable_ops_minus1[i as usize] {
            res.applicable_op_temporal_id[i as usize].push(bs.read_bits(3) as u8);
            res.applicable_op_num_target_views_minus1[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            res.applicable_op_target_view_id[i as usize].push(Vec::new());
            res.applicable_op_depth_flag[i as usize].push(Vec::new());
            res.applicable_op_texture_flag[i as usize].push(Vec::new());
            for _ in 0..=res.applicable_op_num_target_views_minus1[i as usize][j as usize]{
                res.applicable_op_target_view_id[i as usize][j as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                res.applicable_op_depth_flag[i as usize][j as usize].push(1 == bs.read_bits(1));
                res.applicable_op_texture_flag[i as usize][j as usize].push(1 == bs.read_bits(1));
            }
            res.applicable_op_num_texture_views_minus1[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            res.applicable_op_num_depth_views[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        }
    }
    res.mvcd_vui_parameters_present_flag = 1 == bs.read_bits(1);
    if res.mvcd_vui_parameters_present_flag {
        res.mvcd_vui_parameters = decode_vui_mvcd_parameters(bs);
    }
    res.texture_vui_parameters_present_flag = 1 == bs.read_bits(1);
    if res.texture_vui_parameters_present_flag {
        res.mvc_vui_parameters_extension = decode_vui_mvc_parameters(bs);
    }

    res
}

/// Described in J.7.3.2.1.5 -- Sequence parameter Set 3D-AVC extension
fn decode_sps_3davc_extension(_bs: &mut ByteStream) -> AVC3DSPSExtension {
    let res = AVC3DSPSExtension::new();
    // TODO: 3D AVC SPS Decoding
    println!("decode_sps_3davc_extension - not yet supported");
    res
}

/// Described in 7.3.2.1.2 -- Sequence Parameter Set Extension
pub fn decode_sps_extension(bs: &mut ByteStream) -> SPSExtension {
    let mut res = SPSExtension::new();
    res.seq_parameter_set_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "SPS Extension: seq_parameter_set_id",
        &res.seq_parameter_set_id,
        63,
    );
    res.aux_format_idc = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("SPS Extension: aux_format_idc", &res.aux_format_idc, 63);

    if res.aux_format_idc != 0 {
        res.bit_depth_aux_minus8 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SPS Extension: bit_depth_aux_minus8",
            &res.bit_depth_aux_minus8,
            63,
        );
        res.alpha_incr_flag = 1 == bs.read_bits(1);
        decoder_formatted_print("SPS Extension: alpha_incr_flag", &res.alpha_incr_flag, 63);

        let bits_to_read = (res.bit_depth_aux_minus8 + 9) as u8;
        res.alpha_opaque_value = bs.read_bits(bits_to_read) as u32;
        decoder_formatted_print(
            "SPS Extension: alpha_opaque_value",
            &res.alpha_opaque_value,
            63,
        );

        let bits_to_read = (res.bit_depth_aux_minus8 + 9) as u8;
        res.alpha_transparent_value = bs.read_bits(bits_to_read) as u32;
        decoder_formatted_print(
            "SPS Extension: alpha_transparent_value",
            &res.alpha_transparent_value,
            63,
        );
    }
    res.additional_extension_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SPS Extension: additional_extension_flag",
        &res.additional_extension_flag,
        63,
    );

    // rbsp_trailing_bits()

    res
}

/// Described in 7.3.2.1.3 -- Subset Sequence Parameter Set
pub fn decode_subset_sps(bs: &mut ByteStream) -> SubsetSPS {
    let mut res = SubsetSPS::new();
    res.sps = decode_seq_parameter_set(bs);

    if res.sps.profile_idc == 83 || res.sps.profile_idc == 86 {
        let chroma_array_type = match res.sps.separate_colour_plane_flag {
            true => res.sps.chroma_format_idc,
            false => 0,
        };
        res.sps_svc = decode_sps_svc_extension(chroma_array_type, bs); // specified in Annex G
        res.svc_vui_parameters_present_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "Subset SPS: svc_vui_parameters_present_flag",
            &res.svc_vui_parameters_present_flag,
            63,
        );
        if res.svc_vui_parameters_present_flag {
            res.svc_vui = decode_vui_svc_parameters(bs); // specified in Annex G
        }
    } else if res.sps.profile_idc == 118 || res.sps.profile_idc == 128 || res.sps.profile_idc == 134
    {
        res.bit_equal_to_one = bs.read_bits(1) as u8;
        res.sps_mvc =
            decode_sps_mvc_extension(res.sps.profile_idc, res.sps.frame_mbs_only_flag, bs); // specified in Annex H
        res.mvc_vui_parameters_present_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "Subset SPS: mvc_vui_parameters_present_flag",
            &res.mvc_vui_parameters_present_flag,
            63,
        );
        if res.mvc_vui_parameters_present_flag {
            res.mvc_vui = decode_vui_mvc_parameters(bs); // specified in Annex H
        }
    } else if res.sps.profile_idc == 138 || res.sps.profile_idc == 135 {
        res.bit_equal_to_one = bs.read_bits(1) as u8;
        res.sps_mvcd = decode_sps_mvcd_extension(bs); // specified in Annex I
    } else if res.sps.profile_idc == 139 {
        res.bit_equal_to_one = bs.read_bits(1) as u8;
        res.sps_mvcd = decode_sps_mvcd_extension(bs); // specified in Annex I
        res.sps_3davc = decode_sps_3davc_extension(bs); // specified in Annex J
    }

    res.additional_extension2_flag.push(1 == bs.read_bits(1));
    decoder_formatted_print(
        "Subset SPS: additional_extension2_flag",
        &res.additional_extension2_flag[0],
        63,
    );
    if res.additional_extension2_flag[0] {
        while bs.more_data() {
            res.additional_extension2_flag.push(1 == bs.read_bits(1));
        }
    }

    res
}

/// Described in E.1.1 -- VUI parameters
fn decode_vui_parameters(bs: &mut ByteStream) -> VUIParameters {
    let mut res = VUIParameters::new();

    res.aspect_ratio_info_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: aspect_ratio_info_present_flag",
        &res.aspect_ratio_info_present_flag,
        63,
    );

    if res.aspect_ratio_info_present_flag {
        res.aspect_ratio_idc = bs.read_bits(8) as u8;
        decoder_formatted_print("VUI: aspect_ratio_idc", &res.aspect_ratio_idc, 63);

        // see table E-1 for parsing aspect_ratio_idc
        if res.aspect_ratio_idc == 255 {
            // Extended_SAR
            res.sar_width = bs.read_bits(16) as u16;
            decoder_formatted_print("VUI: sar_width", &res.sar_width, 63);

            res.sar_height = bs.read_bits(16) as u16;
            decoder_formatted_print("VUI: sar_height", &res.sar_height, 63);
        }
    }

    res.overscan_info_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: overscan_info_present_flag",
        &res.overscan_info_present_flag,
        63,
    );

    if res.overscan_info_present_flag {
        res.overscan_appropriate_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "VUI: overscan_appropriate_flag",
            &res.overscan_appropriate_flag,
            63,
        );
    }

    res.video_signal_type_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: video_signal_type_present_flag",
        &res.video_signal_type_present_flag,
        63,
    );

    if res.video_signal_type_present_flag {
        res.video_format = bs.read_bits(3) as u8;
        decoder_formatted_print("VUI: video_format", &res.video_format, 63);

        res.video_full_range_flag = 1 == bs.read_bits(1);
        decoder_formatted_print("VUI: video_full_range_flag", &res.video_full_range_flag, 63);

        res.colour_description_present_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "VUI: colour_description_present_flag",
            &res.colour_description_present_flag,
            63,
        );

        if res.colour_description_present_flag {
            res.colour_primaries = bs.read_bits(8) as u8;
            decoder_formatted_print("VUI: colour_primaries", &res.colour_primaries, 63);

            res.transfer_characteristics = bs.read_bits(8) as u8;
            decoder_formatted_print(
                "VUI: transfer_characteristics",
                &res.transfer_characteristics,
                63,
            );

            res.matrix_coefficients = bs.read_bits(8) as u8;
            decoder_formatted_print("VUI: matrix_coefficients", &res.matrix_coefficients, 63);
        }
    }

    res.chroma_loc_info_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: chroma_loc_info_present_flag",
        &res.chroma_loc_info_present_flag,
        63,
    );

    if res.chroma_loc_info_present_flag {
        res.chroma_sample_loc_type_top_field = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "VUI: chroma_sample_loc_type_top_field",
            &res.chroma_sample_loc_type_top_field,
            63,
        );

        res.chroma_sample_loc_type_bottom_field =
            exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "VUI: chroma_sample_loc_type_bottom_field",
            &res.chroma_sample_loc_type_bottom_field,
            63,
        );
    }

    res.timing_info_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: timing_info_present_flag",
        &res.timing_info_present_flag,
        63,
    );

    if res.timing_info_present_flag {
        res.num_units_in_tick = bs.read_bits(32);
        decoder_formatted_print("VUI: num_units_in_tick", &res.num_units_in_tick, 63);

        res.time_scale = bs.read_bits(32);
        decoder_formatted_print("VUI: time_scale", &res.time_scale, 63);

        res.fixed_frame_rate_flag = 1 == bs.read_bits(1);
        decoder_formatted_print("VUI: fixed_frame_rate_flag", &res.fixed_frame_rate_flag, 63);
    }

    res.nal_hrd_parameters_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: nal_hrd_parameters_present_flag",
        &res.nal_hrd_parameters_present_flag,
        63,
    );

    if res.nal_hrd_parameters_present_flag {
        res.nal_hrd_parameters = decode_hrd_parameters(bs);
    }

    res.vcl_hrd_parameters_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: vcl_hrd_parameters_present_flag",
        &res.vcl_hrd_parameters_present_flag,
        63,
    );

    if res.vcl_hrd_parameters_present_flag {
        res.vcl_hrd_parameters = decode_hrd_parameters(bs);
    }

    if res.nal_hrd_parameters_present_flag || res.vcl_hrd_parameters_present_flag {
        res.low_delay_hrd_flag = 1 == bs.read_bits(1);
        decoder_formatted_print("VUI: low_delay_hrd_flag", &res.low_delay_hrd_flag, 63);
    }

    res.pic_struct_present_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: pic_struct_present_flag",
        &res.pic_struct_present_flag,
        63,
    );

    res.bitstream_restriction_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "VUI: bitstream_restriction_flag",
        &res.bitstream_restriction_flag,
        63,
    );

    if res.bitstream_restriction_flag {
        res.motion_vectors_over_pic_boundaries_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "VUI: motion_vectors_over_pic_boundaries_flag",
            &res.motion_vectors_over_pic_boundaries_flag,
            63,
        );

        res.max_bytes_per_pic_denom = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "VUI: max_bytes_per_pic_denom",
            &res.max_bytes_per_pic_denom,
            63,
        );

        res.max_bits_per_mb_denom = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print("VUI: max_bits_per_mb_denom", &res.max_bits_per_mb_denom, 63);

        res.log2_max_mv_length_horizontal = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "VUI: log2_max_mv_length_horizontal",
            &res.log2_max_mv_length_horizontal,
            63,
        );

        res.log2_max_mv_length_vertical = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "VUI: log2_max_mv_length_vertical",
            &res.log2_max_mv_length_vertical,
            63,
        );

        res.max_num_reorder_frames = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "VUI: max_num_reorder_frames",
            &res.max_num_reorder_frames,
            63,
        );

        res.max_dec_frame_buffering = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "VUI: max_dec_frame_buffering",
            &res.max_dec_frame_buffering,
            63,
        );
    }

    res
}

/// Described in E.1.2 -- HRD parameters
fn decode_hrd_parameters(bs: &mut ByteStream) -> HRDParameters {
    let mut res = HRDParameters::new();

    res.cpb_cnt_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print("HRD: cpb_cnt_minus1", &res.cpb_cnt_minus1, 63);
    res.bit_rate_scale = bs.read_bits(4) as u8;
    decoder_formatted_print("HRD: bit_rate_scale", &res.bit_rate_scale, 63);
    res.cpb_size_scale = bs.read_bits(4) as u8;
    decoder_formatted_print("HRD: cpb_size_scale", &res.cpb_size_scale, 63);

    // iterator is sched_sel_idx
    for sched_sel_idx in 0..=res.cpb_cnt_minus1 {
        res.bit_rate_value_minus1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        decoder_formatted_print(
            "HRD: bit_rate_value_minus1[]",
            &res.bit_rate_value_minus1[sched_sel_idx as usize],
            63,
        );
        res.cpb_size_values_minus1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        decoder_formatted_print(
            "HRD: cpb_size_values_minus1[]",
            &res.cpb_size_values_minus1[sched_sel_idx as usize],
            63,
        );
        res.cbr_flag.push(1 == bs.read_bits(1));
        decoder_formatted_print("HRD: cbr_flag[]", &res.cbr_flag[sched_sel_idx as usize], 63);
    }

    res.initial_cpb_removal_delay_length_minus1 = bs.read_bits(5) as u8;
    decoder_formatted_print(
        "HRD: initial_cpb_removal_delay_length_minus1",
        &res.initial_cpb_removal_delay_length_minus1,
        63,
    );
    res.cpb_removal_delay_length_minus1 = bs.read_bits(5) as u8;
    decoder_formatted_print(
        "HRD: cpb_removal_delay_length_minus1",
        &res.cpb_removal_delay_length_minus1,
        63,
    );
    res.dpb_output_delay_length_minus1 = bs.read_bits(5) as u8;
    decoder_formatted_print(
        "HRD: dpb_output_delay_length_minus1",
        &res.dpb_output_delay_length_minus1,
        63,
    );
    res.time_offset_length = bs.read_bits(5) as u8;
    decoder_formatted_print("HRD: time_offset_length", &res.time_offset_length, 63);
    res
}

/// Described in G.14.1 -- SVC VUI parameters extension
fn decode_vui_svc_parameters(mut bs: &mut ByteStream) -> SVCVUIParameters {
    let mut res = SVCVUIParameters::new();

    res.vui_ext_num_entries_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "SVC VUI: vui_ext_num_entries_minus1",
        &res.vui_ext_num_entries_minus1,
        63,
    );

    for i in 0..=(res.vui_ext_num_entries_minus1 as usize) {
        res.vui_ext_dependency_id.push(bs.read_bits(3) as u8);
        decoder_formatted_print(
            "SVC VUI: vui_ext_dependency_id[]",
            &res.vui_ext_dependency_id[i],
            63,
        );

        res.vui_ext_quality_id.push(bs.read_bits(4) as u8);
        decoder_formatted_print(
            "SVC VUI: vui_ext_quality_id[]",
            &res.vui_ext_quality_id[i],
            63,
        );

        res.vui_ext_temporal_id.push(bs.read_bits(3) as u8);
        decoder_formatted_print(
            "SVC VUI: vui_ext_temporal_id[]",
            &res.vui_ext_temporal_id[i],
            63,
        );

        res.vui_ext_timing_info_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "SVC VUI: vui_ext_timing_info_present_flag[]",
            &res.vui_ext_timing_info_present_flag[i],
            63,
        );

        if res.vui_ext_timing_info_present_flag[i] {
            res.vui_ext_num_units_in_tick.push(bs.read_bits(32));
            decoder_formatted_print(
                "SVC VUI: vui_ext_num_units_in_tick[]",
                &res.vui_ext_num_units_in_tick[i],
                63,
            );
            res.vui_ext_time_scale.push(bs.read_bits(32));
            decoder_formatted_print(
                "SVC VUI: vui_ext_time_scale[]",
                &res.vui_ext_time_scale[i],
                63,
            );
            res.vui_ext_fixed_frame_rate_flag.push(1 == bs.read_bits(1));
            decoder_formatted_print(
                "SVC VUI: vui_ext_fixed_frame_rate_flag[]",
                &res.vui_ext_fixed_frame_rate_flag[i],
                63,
            );
        } else {
            res.vui_ext_num_units_in_tick.push(0);
            res.vui_ext_time_scale.push(0);
            res.vui_ext_fixed_frame_rate_flag.push(false);
        }

        res.vui_ext_nal_hrd_parameters_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "SVC VUI: vui_ext_nal_hrd_parameters_present_flag[]",
            &res.vui_ext_nal_hrd_parameters_present_flag[i],
            63,
        );
        if res.vui_ext_nal_hrd_parameters_present_flag[i] {
            res.vui_ext_nal_hrd_parameters
                .push(decode_hrd_parameters(&mut bs));
        }

        res.vui_ext_vcl_hrd_parameters_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "SVC VUI: vui_ext_vcl_hrd_parameters_present_flag[]",
            &res.vui_ext_vcl_hrd_parameters_present_flag[i],
            63,
        );
        if res.vui_ext_vcl_hrd_parameters_present_flag[i] {
            res.vui_ext_vcl_hrd_parameters
                .push(decode_hrd_parameters(&mut bs));
        }

        if res.vui_ext_nal_hrd_parameters_present_flag[i]
            || res.vui_ext_vcl_hrd_parameters_present_flag[i]
        {
            res.vui_ext_low_delay_hrd_flag.push(1 == bs.read_bits(1));
            decoder_formatted_print(
                "SVC VUI: vui_ext_low_delay_hrd_flag[]",
                &res.vui_ext_low_delay_hrd_flag[i],
                63,
            );
        }
        res.vui_ext_pic_struct_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "SVC VUI: vui_ext_pic_struct_present_flag[]",
            &res.vui_ext_pic_struct_present_flag[i],
            63,
        );
    }

    res
}

/// Described in H.14.1 -- MVC VUI parameters extension syntax
fn decode_vui_mvc_parameters(mut bs: &mut ByteStream) -> MVCVUIParameters {
    let mut res = MVCVUIParameters::new();

    res.vui_mvc_num_ops_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "MVC VUI: vui_mvc_num_ops_minus1",
        &res.vui_mvc_num_ops_minus1,
        63,
    );

    for i in 0..=(res.vui_mvc_num_ops_minus1 as usize) {
        res.vui_mvc_temporal_id.push(bs.read_bits(3) as u8);
        decoder_formatted_print(
            "MVC VUI: vui_mvc_temporal_id[]",
            &res.vui_mvc_temporal_id[i],
            63,
        );

        res.vui_mvc_num_target_output_views_minus1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        decoder_formatted_print(
            "MVC VUI: vui_mvc_num_target_output_views_minus1[]",
            &res.vui_mvc_num_target_output_views_minus1[i],
            63,
        );

        res.vui_mvc_view_id.push(Vec::new());
        for _ in 0..=(res.vui_mvc_num_target_output_views_minus1[i] as usize) {
            res.vui_mvc_view_id[i].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        }
        decoder_formatted_print("MVC VUI: vui_mvc_view_id[][]", &res.vui_mvc_view_id[i], 63);

        res.vui_mvc_timing_info_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "MVC VUI: vui_mvc_timing_info_present_flag[]",
            &res.vui_mvc_timing_info_present_flag[i],
            63,
        );

        if res.vui_mvc_timing_info_present_flag[i] {
            res.vui_mvc_num_units_in_tick.push(bs.read_bits(32));
            decoder_formatted_print(
                "MVC VUI: vui_mvc_num_units_in_tick[]",
                &res.vui_mvc_num_units_in_tick[i],
                63,
            );
            res.vui_mvc_time_scale.push(bs.read_bits(32));
            decoder_formatted_print(
                "MVC VUI: vui_mvc_time_scale[]",
                &res.vui_mvc_time_scale[i],
                63,
            );
            res.vui_mvc_fixed_frame_rate_flag.push(1 == bs.read_bits(1));
            decoder_formatted_print(
                "MVC VUI: vui_mvc_fixed_frame_rate_flag[]",
                &res.vui_mvc_fixed_frame_rate_flag[i],
                63,
            );
        } else {
            res.vui_mvc_num_units_in_tick.push(0);
            res.vui_mvc_time_scale.push(0);
            res.vui_mvc_fixed_frame_rate_flag.push(false);
        }

        res.vui_mvc_nal_hrd_parameters_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "MVC VUI: vui_mvc_nal_hrd_parameters_present_flag[]",
            &res.vui_mvc_nal_hrd_parameters_present_flag[i],
            63,
        );
        if res.vui_mvc_nal_hrd_parameters_present_flag[i] {
            res.vui_mvc_nal_hrd_parameters
                .push(decode_hrd_parameters(&mut bs));
        }

        res.vui_mvc_vcl_hrd_parameters_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "MVC VUI: vui_mvc_vcl_hrd_parameters_present_flag[]",
            &res.vui_mvc_vcl_hrd_parameters_present_flag[i],
            63,
        );
        if res.vui_mvc_vcl_hrd_parameters_present_flag[i] {
            res.vui_mvc_vcl_hrd_parameters
                .push(decode_hrd_parameters(&mut bs));
        }

        if res.vui_mvc_nal_hrd_parameters_present_flag[i]
            || res.vui_mvc_vcl_hrd_parameters_present_flag[i]
        {
            res.vui_mvc_low_delay_hrd_flag.push(1 == bs.read_bits(1));
            decoder_formatted_print(
                "MVC VUI: vui_ext_low_delay_hrd_flag[]",
                &res.vui_mvc_low_delay_hrd_flag[i],
                63,
            );
        }
        res.vui_mvc_pic_struct_present_flag
            .push(1 == bs.read_bits(1));
        decoder_formatted_print(
            "MVC VUI: vui_mvc_pic_struct_present_flag[]",
            &res.vui_mvc_pic_struct_present_flag[i],
            63,
        );
    }

    res
}

/// Described in I.14.1 -- MVCD VUI parameters extension syntax
fn decode_vui_mvcd_parameters(bs: &mut ByteStream) -> MVCDVUIParameters {
    let mut res = MVCDVUIParameters::new();

    res.vui_mvcd_num_ops_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;

    for i in 0..=res.vui_mvcd_num_ops_minus1 {
        res.vui_mvcd_temporal_id.push(bs.read_bits(3) as u8);
        res.vui_mvcd_num_target_output_views_minus1.push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);

        res.vui_mvcd_view_id.push(Vec::new());
        res.vui_mvcd_depth_flag.push(Vec::new());
        res.vui_mvcd_texture_flag.push(Vec::new());
        for _ in 0..=res.vui_mvcd_num_target_output_views_minus1[i as usize] {
            res.vui_mvcd_view_id[i as usize].push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            res.vui_mvcd_depth_flag[i as usize].push(1 == bs.read_bits(1));
            res.vui_mvcd_texture_flag[i as usize].push(1 == bs.read_bits(1));
        }
        res.vui_mvcd_timing_info_present_flag.push(1 == bs.read_bits(1));
        if res.vui_mvcd_timing_info_present_flag[i as usize]{
            res.vui_mvcd_num_units_in_tick.push(bs.read_bits(32) as u32);
            res.vui_mvcd_time_scale.push(bs.read_bits(32) as u32);
            res.vui_mvcd_fixed_frame_rate_flag.push(1 == bs.read_bits(1));
        }
        res.vui_mvcd_nal_hrd_parameters_present_flag.push(1 == bs.read_bits(1));
        if res.vui_mvcd_nal_hrd_parameters_present_flag[i as usize] {
            res.vui_mvcd_nal_hrd_parameters.push(decode_hrd_parameters(bs));
        }
        res.vui_mvcd_vcl_hrd_parameters_present_flag.push(1 == bs.read_bits(1));
        if res.vui_mvcd_vcl_hrd_parameters_present_flag[i as usize] {
            res.vui_mvcd_vcl_hrd_parameters.push(decode_hrd_parameters(bs));
        }
        if res.vui_mvcd_nal_hrd_parameters_present_flag[i as usize] || res.vui_mvcd_vcl_hrd_parameters_present_flag[i as usize] {
            res.vui_mvcd_low_delay_hrd_flag.push(1 == bs.read_bits(1))
        }
        res.vui_mvcd_pic_struct_present_flag.push(1 == bs.read_bits(1));
    }

    res
}