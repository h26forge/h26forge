//! Parameter Set (SPS, PPS, VUI, extensions) syntax element encoding.

use crate::common::data_structures::AVC3DSPSExtension;
use crate::common::data_structures::HRDParameters;
use crate::common::data_structures::MVCDSPSExtension;
use crate::common::data_structures::MVCSPSExtension;
use crate::common::data_structures::MVCVUIParameters;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::SPSExtension;
use crate::common::data_structures::SVCSPSExtension;
use crate::common::data_structures::SVCVUIParameters;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::SubsetSPS;
use crate::common::data_structures::VUIParameters;
use crate::common::helper::bitstream_to_bytestream;
use crate::encoder::binarization_functions::generate_unsigned_binary;
use crate::encoder::expgolomb::exp_golomb_encode_one;

/// Described in 7.3.2.1.1 -- Sequence Parameter Set
///
/// - `s`: SeqParameterSet object to encode
/// - 'return_bitstream`: if True, returns the bit sequence rather than an encoded byte array
pub fn encode_sps(s: &SeqParameterSet, return_bitstream: bool) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    bitstream_array.append(&mut generate_unsigned_binary(s.profile_idc as u32, 8));

    bitstream_array.push(match s.constraint_set0_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match s.constraint_set1_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match s.constraint_set2_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match s.constraint_set3_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match s.constraint_set4_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match s.constraint_set5_flag {
        true => 1u8,
        false => 0u8,
    });

    bitstream_array.push((s.reserved_zero_2bits & 2) >> 1);
    bitstream_array.push(s.reserved_zero_2bits & 1);

    bitstream_array.append(&mut generate_unsigned_binary(s.level_idc as u32, 8));

    bitstream_array.append(&mut exp_golomb_encode_one(
        s.seq_parameter_set_id as i32,
        false,
        0,
        false,
    ));

    if s.profile_idc == 100
        || s.profile_idc == 110
        || s.profile_idc == 122
        || s.profile_idc == 244
        || s.profile_idc == 44
        || s.profile_idc == 83
        || s.profile_idc == 86
        || s.profile_idc == 118
        || s.profile_idc == 128
        || s.profile_idc == 138
        || s.profile_idc == 139
        || s.profile_idc == 134
        || s.profile_idc == 135
    {
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.chroma_format_idc as i32,
            false,
            0,
            false,
        ));

        if s.chroma_format_idc == 3 {
            bitstream_array.push(match s.separate_colour_plane_flag {
                true => 1u8,
                false => 0u8,
            });
        }

        bitstream_array.append(&mut exp_golomb_encode_one(
            s.bit_depth_luma_minus8 as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.bit_depth_chroma_minus8 as i32,
            false,
            0,
            false,
        ));

        bitstream_array.push(match s.qpprime_y_zero_transform_bypass_flag {
            true => 1u8,
            false => 0u8,
        });
        bitstream_array.push(match s.seq_scaling_matrix_present_flag {
            true => 1u8,
            false => 0u8,
        });

        if s.seq_scaling_matrix_present_flag {
            let cur_max = match s.chroma_format_idc != 3 {
                true => 8,
                false => 12,
            };

            for i in 0..cur_max {
                bitstream_array.push(match s.seq_scaling_list_present_flag[i] {
                    true => 1u8,
                    false => 0u8,
                });
                if s.seq_scaling_list_present_flag[i] {
                    if i < 6 {
                        bitstream_array
                            .append(&mut encode_scaling_list(s.delta_scale_4x4[i].clone(), 16));
                    } else {
                        bitstream_array
                            .append(&mut encode_scaling_list(s.delta_scale_8x8[i].clone(), 64));
                    }
                }
            }
        }
    }

    bitstream_array.append(&mut exp_golomb_encode_one(
        s.log2_max_frame_num_minus4 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.append(&mut exp_golomb_encode_one(
        s.pic_order_cnt_type as i32,
        false,
        0,
        false,
    ));

    if s.pic_order_cnt_type == 0 {
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.log2_max_pic_order_cnt_lsb_minus4 as i32,
            false,
            0,
            false,
        ));
    } else if s.pic_order_cnt_type == 1 {
        bitstream_array.push(match s.delta_pic_order_always_zero_flag {
            true => 1u8,
            false => 0u8,
        });

        bitstream_array.append(&mut exp_golomb_encode_one(
            s.offset_for_non_ref_pic,
            true,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.offset_for_top_to_bottom_field,
            true,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.num_ref_frames_in_pic_order_cnt_cycle as i32,
            false,
            0,
            false,
        ));

        for i in 0..s.num_ref_frames_in_pic_order_cnt_cycle {
            bitstream_array.append(&mut exp_golomb_encode_one(
                s.offset_for_ref_frame[i as usize],
                true,
                0,
                false,
            ));
        }
    }

    bitstream_array.append(&mut exp_golomb_encode_one(
        s.max_num_ref_frames as i32,
        false,
        0,
        false,
    ));
    bitstream_array.push(match s.gaps_in_frame_num_value_allowed_flag {
        true => 1u8,
        false => 0u8,
    });

    bitstream_array.append(&mut exp_golomb_encode_one(
        s.pic_width_in_mbs_minus1 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.append(&mut exp_golomb_encode_one(
        s.pic_height_in_map_units_minus1 as i32,
        false,
        0,
        false,
    ));

    bitstream_array.push(match s.frame_mbs_only_flag {
        true => 1u8,
        false => 0u8,
    });
    if !s.frame_mbs_only_flag {
        bitstream_array.push(match s.mb_adaptive_frame_field_flag {
            true => 1u8,
            false => 0u8,
        });
    }

    bitstream_array.push(match s.direct_8x8_inference_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match s.frame_cropping_flag {
        true => 1u8,
        false => 0u8,
    });

    if s.frame_cropping_flag {
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.frame_crop_left_offset as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.frame_crop_right_offset as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.frame_crop_top_offset as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            s.frame_crop_bottom_offset as i32,
            false,
            0,
            false,
        ));
    }
    bitstream_array.push(match s.vui_parameters_present_flag {
        true => 1u8,
        false => 0u8,
    });
    if s.vui_parameters_present_flag {
        bitstream_array.append(&mut encode_vui_parameters(&s.vui_parameters));
    }

    if return_bitstream {
        return bitstream_array;
    }

    // insert rbsp_stop_one_bit
    bitstream_array.push(1);

    s.encoder_pretty_print();

    bitstream_to_bytestream(bitstream_array, 0)
}

/// Follows 7.3.2.1.1.1 -- Scaling list
fn encode_scaling_list(delta_scaling_list: Vec<i32>, size_of_scaling_list: usize) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    let mut last_scale = 8;
    let mut next_scale = 8;

    for j in 0..size_of_scaling_list {
        if next_scale != 0 {
            let delta_scale = delta_scaling_list[j];

            bitstream_array.append(&mut exp_golomb_encode_one(delta_scale, true, 0, false));

            next_scale = (last_scale + delta_scale + 256) % 256;
        }
        if next_scale != 0 {
            last_scale = next_scale;
        }
        //last_scale = scaling_list[j];
    }

    bitstream_array
}

/// Takes in VUIParameters and encodes the syntax elements into a bitstream
fn encode_vui_parameters(v: &VUIParameters) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = vec![match v.aspect_ratio_info_present_flag {
        true => 1u8,
        false => 0u8,
    }];

    if v.aspect_ratio_info_present_flag {
        bitstream_array.append(&mut generate_unsigned_binary(v.aspect_ratio_idc as u32, 8));

        if v.aspect_ratio_idc == 255 {
            bitstream_array.append(&mut generate_unsigned_binary(v.sar_width as u32, 16));
            bitstream_array.append(&mut generate_unsigned_binary(v.sar_height as u32, 16));
        }
    }
    bitstream_array.push(match v.overscan_info_present_flag {
        true => 1u8,
        false => 0u8,
    });

    if v.overscan_info_present_flag {
        bitstream_array.push(match v.overscan_appropriate_flag {
            true => 1u8,
            false => 0u8,
        });
    }

    bitstream_array.push(match v.video_signal_type_present_flag {
        true => 1u8,
        false => 0u8,
    });
    if v.video_signal_type_present_flag {
        bitstream_array.append(&mut generate_unsigned_binary(v.video_format as u32, 3));

        bitstream_array.push(match v.video_full_range_flag {
            true => 1u8,
            false => 0u8,
        });

        bitstream_array.push(match v.colour_description_present_flag {
            true => 1u8,
            false => 0u8,
        });
        if v.colour_description_present_flag {
            bitstream_array.append(&mut generate_unsigned_binary(v.colour_primaries as u32, 8));
            bitstream_array.append(&mut generate_unsigned_binary(
                v.transfer_characteristics as u32,
                8,
            ));
            bitstream_array.append(&mut generate_unsigned_binary(
                v.matrix_coefficients as u32,
                8,
            ));
        }
    }

    bitstream_array.push(match v.chroma_loc_info_present_flag {
        true => 1u8,
        false => 0u8,
    });
    if v.chroma_loc_info_present_flag {
        bitstream_array.append(&mut exp_golomb_encode_one(
            v.chroma_sample_loc_type_top_field as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            v.chroma_sample_loc_type_bottom_field as i32,
            false,
            0,
            false,
        ));
    }

    bitstream_array.push(match v.timing_info_present_flag {
        true => 1u8,
        false => 0u8,
    });
    if v.timing_info_present_flag {
        bitstream_array.append(&mut generate_unsigned_binary(v.num_units_in_tick, 32));
        bitstream_array.append(&mut generate_unsigned_binary(v.time_scale, 32));

        bitstream_array.push(match v.fixed_frame_rate_flag {
            true => 1u8,
            false => 0u8,
        });
    }

    bitstream_array.push(match v.nal_hrd_parameters_present_flag {
        true => 1u8,
        false => 0u8,
    });
    if v.nal_hrd_parameters_present_flag {
        bitstream_array.append(&mut encode_hrd(&v.nal_hrd_parameters));
    }

    bitstream_array.push(match v.vcl_hrd_parameters_present_flag {
        true => 1u8,
        false => 0u8,
    });
    if v.vcl_hrd_parameters_present_flag {
        bitstream_array.append(&mut encode_hrd(&v.vcl_hrd_parameters));
    }

    if v.nal_hrd_parameters_present_flag || v.vcl_hrd_parameters_present_flag {
        bitstream_array.push(match v.low_delay_hrd_flag {
            true => 1u8,
            false => 0u8,
        });
    }

    bitstream_array.push(match v.pic_struct_present_flag {
        true => 1u8,
        false => 0u8,
    });

    bitstream_array.push(match v.bitstream_restriction_flag {
        true => 1u8,
        false => 0u8,
    });
    if v.bitstream_restriction_flag {
        bitstream_array.push(match v.motion_vectors_over_pic_boundaries_flag {
            true => 1u8,
            false => 0u8,
        });

        bitstream_array.append(&mut exp_golomb_encode_one(
            v.max_bytes_per_pic_denom as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            v.max_bits_per_mb_denom as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            v.log2_max_mv_length_horizontal as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            v.log2_max_mv_length_vertical as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            v.max_num_reorder_frames as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            v.max_dec_frame_buffering as i32,
            false,
            0,
            false,
        ));
    }

    bitstream_array
}

fn encode_hrd(hrd: &HRDParameters) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    bitstream_array.append(&mut exp_golomb_encode_one(
        hrd.cpb_cnt_minus1 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.append(&mut generate_unsigned_binary(hrd.bit_rate_scale as u32, 4));
    bitstream_array.append(&mut generate_unsigned_binary(hrd.cpb_size_scale as u32, 4));
    for sched_sel_idx in 0..=hrd.cpb_cnt_minus1 {
        bitstream_array.append(&mut exp_golomb_encode_one(
            hrd.bit_rate_value_minus1[sched_sel_idx as usize] as i32,
            false,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            hrd.cpb_size_values_minus1[sched_sel_idx as usize] as i32,
            false,
            0,
            false,
        ));
        bitstream_array.push(match hrd.cbr_flag[sched_sel_idx as usize] {
            false => 0,
            true => 1,
        });
    }
    bitstream_array.append(&mut generate_unsigned_binary(
        hrd.initial_cpb_removal_delay_length_minus1 as u32,
        5,
    ));
    bitstream_array.append(&mut generate_unsigned_binary(
        hrd.cpb_removal_delay_length_minus1 as u32,
        5,
    ));
    bitstream_array.append(&mut generate_unsigned_binary(
        hrd.dpb_output_delay_length_minus1 as u32,
        5,
    ));
    bitstream_array.append(&mut generate_unsigned_binary(
        hrd.time_offset_length as u32,
        5,
    ));

    bitstream_array
}

/// Described in 7.3.2.3 -- Picture Parameter Set
pub fn encode_pps(p: &PicParameterSet, s: &SeqParameterSet) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    bitstream_array.append(&mut exp_golomb_encode_one(
        p.pic_parameter_set_id as i32,
        false,
        0,
        false,
    ));
    bitstream_array.append(&mut exp_golomb_encode_one(
        p.seq_parameter_set_id as i32,
        false,
        0,
        false,
    ));

    bitstream_array.push(match p.entropy_coding_mode_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match p.bottom_field_pic_order_in_frame_present_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.append(&mut exp_golomb_encode_one(
        p.num_slice_groups_minus1 as i32,
        false,
        0,
        false,
    ));

    if p.num_slice_groups_minus1 > 0 {
        bitstream_array.append(&mut exp_golomb_encode_one(
            p.slice_group_map_type as i32,
            false,
            0,
            false,
        ));
        if p.slice_group_map_type == 0 {
            for i in 0..=p.num_slice_groups_minus1 {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    p.run_length_minus1[i as usize] as i32,
                    false,
                    0,
                    false,
                ));
            }
        } else if p.slice_group_map_type == 2 {
            for i in 0..p.num_slice_groups_minus1 {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    p.top_left[i as usize] as i32,
                    false,
                    0,
                    false,
                ));
                bitstream_array.append(&mut exp_golomb_encode_one(
                    p.bottom_right[i as usize] as i32,
                    false,
                    0,
                    false,
                ));
            }
        } else if p.slice_group_map_type == 3
            || p.slice_group_map_type == 4
            || p.slice_group_map_type == 5
        {
            bitstream_array.push(match p.slice_group_change_direction_flag {
                true => 1u8,
                false => 0u8,
            });
            bitstream_array.append(&mut exp_golomb_encode_one(
                p.slice_group_change_rate_minus1 as i32,
                false,
                0,
                false,
            ));
        } else if p.slice_group_map_type == 6 {
            bitstream_array.append(&mut exp_golomb_encode_one(
                p.pic_size_in_map_units_minus1 as i32,
                false,
                0,
                false,
            ));

            let bits_to_write = ((p.num_slice_groups_minus1 + 1) as f64).log2().ceil() as u8;
            for i in 0..=p.pic_size_in_map_units_minus1 {
                bitstream_array.append(&mut generate_unsigned_binary(
                    p.slice_group_id[i as usize],
                    bits_to_write as usize,
                ));
            }
        }
    }

    bitstream_array.append(&mut exp_golomb_encode_one(
        p.num_ref_idx_l0_default_active_minus1 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.append(&mut exp_golomb_encode_one(
        p.num_ref_idx_l1_default_active_minus1 as i32,
        false,
        0,
        false,
    ));

    bitstream_array.push(match p.weighted_pred_flag {
        true => 1u8,
        false => 0u8,
    });

    bitstream_array.push((p.weighted_bipred_idc & 2) >> 1);
    bitstream_array.push(p.weighted_bipred_idc & 1);

    bitstream_array.append(&mut exp_golomb_encode_one(
        p.pic_init_qp_minus26,
        true,
        0,
        false,
    ));
    bitstream_array.append(&mut exp_golomb_encode_one(
        p.pic_init_qs_minus26,
        true,
        0,
        false,
    ));
    bitstream_array.append(&mut exp_golomb_encode_one(
        p.chroma_qp_index_offset,
        true,
        0,
        false,
    ));

    bitstream_array.push(match p.deblocking_filter_control_present_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match p.constrained_intra_pred_flag {
        true => 1u8,
        false => 0u8,
    });
    bitstream_array.push(match p.redundant_pic_cnt_present_flag {
        true => 1u8,
        false => 0u8,
    });

    // check if there was more rbsp_data in the original (section 7.2 and Annex B)
    if p.more_data_flag {
        bitstream_array.push(match p.transform_8x8_mode_flag {
            true => 1u8,
            false => 0u8,
        });
        bitstream_array.push(match p.pic_scaling_matrix_present_flag {
            true => 1u8,
            false => 0u8,
        });

        if p.pic_scaling_matrix_present_flag {
            let max_val = 6 + match s.chroma_format_idc != 3 {
                true => 2,
                false => 6,
            } * match p.transform_8x8_mode_flag {
                true => 1,
                false => 0,
            };
            for i in 0..max_val {
                bitstream_array.push(match p.pic_scaling_list_present_flag[i] {
                    true => 1u8,
                    false => 0u8,
                });

                if p.pic_scaling_list_present_flag[i] {
                    if i < 6 {
                        bitstream_array
                            .append(&mut encode_scaling_list(p.delta_scale_4x4[i].clone(), 16));
                    } else {
                        bitstream_array
                            .append(&mut encode_scaling_list(p.delta_scale_8x8[i].clone(), 64));
                    }
                }
            }
        }

        bitstream_array.append(&mut exp_golomb_encode_one(
            p.second_chroma_qp_index_offset,
            true,
            0,
            false,
        ));
    }
    // rbsp_stop_one_bit
    bitstream_array.push(1);

    p.encoder_pretty_print();

    bitstream_to_bytestream(bitstream_array, 0)
}

/// Described in G.7.3.2.1.4 -- Sequence Parameter Set SVC Extension
fn encode_sps_svc_extension(chroma_array_type: u8, ext: &SVCSPSExtension) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    bitstream_array.push(
        match ext.inter_layer_deblocking_filter_control_present_flag {
            true => 1,
            false => 0,
        },
    );
    bitstream_array.append(&mut generate_unsigned_binary(
        ext.extended_spatial_scalability_idc as u32,
        2,
    ));

    if chroma_array_type == 1 || chroma_array_type == 2 {
        bitstream_array.push(match ext.chroma_phase_x_plus1_flag {
            true => 1,
            false => 0,
        });
    }

    if chroma_array_type == 1 {
        bitstream_array.append(&mut generate_unsigned_binary(
            ext.chroma_phase_y_plus1 as u32,
            2,
        ));
    }

    if ext.extended_spatial_scalability_idc == 1 {
        if chroma_array_type > 0 {
            bitstream_array.push(match ext.seq_ref_layer_chroma_phase_x_plus1_flag {
                true => 1,
                false => 0,
            });
            bitstream_array.append(&mut generate_unsigned_binary(
                ext.seq_ref_layer_chroma_phase_y_plus1 as u32,
                2,
            ));
        }
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.seq_scaled_ref_layer_left_offset,
            true,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.seq_scaled_ref_layer_top_offset,
            true,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.seq_scaled_ref_layer_right_offset,
            true,
            0,
            false,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.seq_scaled_ref_layer_bottom_offset,
            true,
            0,
            false,
        ));
    }
    bitstream_array.push(match ext.seq_tcoeff_level_prediction_flag {
        true => 1,
        false => 0,
    });

    if ext.seq_tcoeff_level_prediction_flag {
        bitstream_array.push(match ext.adaptive_tcoeff_level_prediction_flag {
            true => 1,
            false => 0,
        });
    }
    bitstream_array.push(match ext.slice_header_restriction_flag {
        true => 1,
        false => 0,
    });

    ext.encoder_pretty_print();
    bitstream_array
}

/// Described in H.7.3.2.1.4 -- Sequence parameter set MVC extension syntax
fn encode_sps_mvc_extension(
    profile_idc: u8,
    frame_mbs_only_flag: bool,
    ext: &MVCSPSExtension,
) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    bitstream_array.append(&mut exp_golomb_encode_one(
        ext.num_views_minus1 as i32,
        false,
        0,
        false,
    ));

    for i in 0..=ext.num_views_minus1 {
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.view_id[i] as i32,
            false,
            0,
            false,
        ));
    }
    // there are 1 to ext.num_views_minus1 values
    for i in 1..=ext.num_views_minus1 {
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.num_anchor_refs_l0[i] as i32,
            false,
            0,
            false,
        ));
        for j in 0..ext.num_anchor_refs_l0[i] {
            bitstream_array.append(&mut exp_golomb_encode_one(
                ext.anchor_refs_l0[i][j as usize] as i32,
                false,
                0,
                false,
            ));
        }
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.num_anchor_refs_l1[i] as i32,
            false,
            0,
            false,
        ));
        for j in 0..ext.num_anchor_refs_l1[i] {
            bitstream_array.append(&mut exp_golomb_encode_one(
                ext.anchor_refs_l1[i][j as usize] as i32,
                false,
                0,
                false,
            ));
        }
    }

    for i in 1..=ext.num_views_minus1 {
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.num_non_anchor_refs_l0[i] as i32,
            false,
            0,
            false,
        ));
        for j in 0..ext.num_non_anchor_refs_l0[i] {
            bitstream_array.append(&mut exp_golomb_encode_one(
                ext.non_anchor_refs_l0[i][j as usize] as i32,
                false,
                0,
                false,
            ));
        }
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.num_non_anchor_refs_l1[i] as i32,
            false,
            0,
            false,
        ));
        for j in 0..ext.num_non_anchor_refs_l1[i] {
            bitstream_array.append(&mut exp_golomb_encode_one(
                ext.non_anchor_refs_l1[i][j as usize] as i32,
                false,
                0,
                false,
            ));
        }
    }

    bitstream_array.append(&mut exp_golomb_encode_one(
        ext.num_level_values_signalled_minus1 as i32,
        false,
        0,
        false,
    ));
    for i in 0..=ext.num_level_values_signalled_minus1 {
        bitstream_array.append(&mut generate_unsigned_binary(ext.level_idc[i] as u32, 8));
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.num_applicable_ops_minus1[i] as i32,
            false,
            0,
            false,
        ));

        for j in 0..=ext.num_applicable_ops_minus1[i] {
            bitstream_array.append(&mut generate_unsigned_binary(
                ext.applicable_op_temporal_id[i][j as usize] as u32,
                3,
            ));
            bitstream_array.append(&mut exp_golomb_encode_one(
                ext.applicable_op_num_target_views_minus1[i][j as usize] as i32,
                false,
                0,
                false,
            ));

            //insert new
            for k in 0..=ext.applicable_op_num_target_views_minus1[i][j] {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    ext.applicable_op_target_view_id[i][j as usize][k as usize] as i32,
                    false,
                    0,
                    false,
                ));
            }
            bitstream_array.append(&mut exp_golomb_encode_one(
                ext.applicable_op_num_views_minus1[i][j as usize] as i32,
                false,
                0,
                false,
            ));
        }
    }

    if profile_idc == 134 {
        bitstream_array.append(&mut generate_unsigned_binary(ext.mfc_format_idc as u32, 6));
        if ext.mfc_format_idc == 0 || ext.mfc_format_idc == 1 {
            bitstream_array.push(match ext.default_grid_position_flag {
                false => 0,
                true => 1,
            });
            if !ext.default_grid_position_flag {
                bitstream_array.append(&mut generate_unsigned_binary(
                    ext.view0_grid_position_x as u32,
                    4,
                ));
                bitstream_array.append(&mut generate_unsigned_binary(
                    ext.view0_grid_position_y as u32,
                    4,
                ));
                bitstream_array.append(&mut generate_unsigned_binary(
                    ext.view1_grid_position_x as u32,
                    4,
                ));
                bitstream_array.append(&mut generate_unsigned_binary(
                    ext.view1_grid_position_y as u32,
                    4,
                ));
            }
        }
        bitstream_array.push(match ext.rpu_filter_enabled_flag {
            false => 0,
            true => 1,
        });
        if !frame_mbs_only_flag {
            bitstream_array.push(match ext.rpu_field_processing_flag {
                false => 0,
                true => 1,
            });
        }
    }

    bitstream_array
}

fn encode_sps_mvcd_extension(_ext: &MVCDSPSExtension) -> Vec<u8> {
    let bitstream_array = Vec::new();
    println!("encode_sps_mvcd_extension - not yet supported");
    bitstream_array
}

fn encode_sps_3davc_extension(_ext: &AVC3DSPSExtension) -> Vec<u8> {
    let bitstream_array = Vec::new();
    println!("encode_sps_3davc_extension - not yet supported");
    bitstream_array
}

/// Described in 7.3.2.1.2 -- Sequence Parameter Set Extension
pub fn encode_sps_extension(ext: &SPSExtension) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    bitstream_array.append(&mut exp_golomb_encode_one(
        ext.seq_parameter_set_id as i32,
        false,
        0,
        false,
    ));
    bitstream_array.append(&mut exp_golomb_encode_one(
        ext.aux_format_idc as i32,
        false,
        0,
        false,
    ));

    if ext.aux_format_idc != 0 {
        bitstream_array.append(&mut exp_golomb_encode_one(
            ext.bit_depth_aux_minus8 as i32,
            false,
            0,
            false,
        ));
        bitstream_array.push(match ext.alpha_incr_flag {
            false => 0,
            true => 1,
        });

        let bits_to_write = (ext.bit_depth_aux_minus8 + 9) as usize;
        bitstream_array.append(&mut generate_unsigned_binary(
            ext.alpha_opaque_value,
            bits_to_write,
        ));
        bitstream_array.append(&mut generate_unsigned_binary(
            ext.alpha_transparent_value,
            bits_to_write,
        ));
    }
    bitstream_array.push(match ext.additional_extension_flag {
        false => 0,
        true => 1,
    });

    // rbsp_trailing_bits()
    bitstream_array.push(1);

    bitstream_to_bytestream(bitstream_array, 0)
}

/// Described in 7.3.2.1.3 -- Subset Sequence Parameter Set
pub fn encode_subset_sps(s: &SubsetSPS) -> Vec<u8> {
    let mut bitstream_array = encode_sps(&s.sps, true);

    if s.sps.profile_idc == 83 || s.sps.profile_idc == 86 {
        let chroma_array_type = match s.sps.separate_colour_plane_flag {
            true => s.sps.chroma_format_idc,
            false => 0,
        };
        bitstream_array.append(&mut encode_sps_svc_extension(chroma_array_type, &s.sps_svc)); // specified in Annex G
        bitstream_array.push(match s.svc_vui_parameters_present_flag {
            false => 0,
            true => 1,
        });

        if s.svc_vui_parameters_present_flag {
            bitstream_array.append(&mut encode_vui_svc_parameters(&s.svc_vui)); // specified in Annex G
        }
    } else if s.sps.profile_idc == 118 || s.sps.profile_idc == 128 || s.sps.profile_idc == 134 {
        bitstream_array.push(s.bit_equal_to_one);
        bitstream_array.append(&mut encode_sps_mvc_extension(
            s.sps.profile_idc,
            s.sps.frame_mbs_only_flag,
            &s.sps_mvc,
        )); // specified in Annex H
        bitstream_array.push(match s.mvc_vui_parameters_present_flag {
            false => 0,
            true => 1,
        });

        if s.mvc_vui_parameters_present_flag {
            bitstream_array.append(&mut encode_vui_mvc_parameters(&s.mvc_vui)); // specified in Annex H
        }
    } else if s.sps.profile_idc == 138 || s.sps.profile_idc == 135 {
        bitstream_array.push(s.bit_equal_to_one);
        bitstream_array.append(&mut encode_sps_mvcd_extension(&s.sps_mvcd)); // specified in Annex I
    } else if s.sps.profile_idc == 139 {
        bitstream_array.push(s.bit_equal_to_one);
        bitstream_array.append(&mut encode_sps_mvcd_extension(&s.sps_mvcd)); // specified in Annex I
        bitstream_array.append(&mut encode_sps_3davc_extension(&s.sps_3davc)); // specified in Annex J
    }

    for i in 0..s.additional_extension2_flag.len() {
        bitstream_array.push(match s.additional_extension2_flag[i] {
            false => 0,
            true => 1,
        })
    }

    s.encoder_pretty_print();

    // rbsp_trailing_bits()
    bitstream_array.push(1);

    bitstream_to_bytestream(bitstream_array, 0)
}

fn encode_vui_svc_parameters(vui: &SVCVUIParameters) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    // NOTE: may have issues converting large u32 to i32
    bitstream_array.append(&mut exp_golomb_encode_one(
        vui.vui_ext_num_entries_minus1 as i32,
        false,
        0,
        false,
    ));

    for i in 0..=(vui.vui_ext_num_entries_minus1 as usize) {
        bitstream_array.append(&mut generate_unsigned_binary(
            vui.vui_ext_dependency_id[i] as u32,
            3,
        ));
        bitstream_array.append(&mut generate_unsigned_binary(
            vui.vui_ext_quality_id[i] as u32,
            4,
        ));
        bitstream_array.append(&mut generate_unsigned_binary(
            vui.vui_ext_temporal_id[i] as u32,
            3,
        ));
        bitstream_array.push(match vui.vui_ext_timing_info_present_flag[i] {
            true => 1,
            false => 0,
        });
        if vui.vui_ext_timing_info_present_flag[i] {
            bitstream_array.append(&mut generate_unsigned_binary(
                vui.vui_ext_num_units_in_tick[i] as u32,
                32,
            ));
            bitstream_array.append(&mut generate_unsigned_binary(
                vui.vui_ext_time_scale[i] as u32,
                32,
            ));
            bitstream_array.push(match vui.vui_ext_fixed_frame_rate_flag[i] {
                true => 1,
                false => 0,
            });
        }
        bitstream_array.push(match vui.vui_ext_nal_hrd_parameters_present_flag[i] {
            true => 1,
            false => 0,
        });
        if vui.vui_ext_nal_hrd_parameters_present_flag[i] {
            bitstream_array.append(&mut encode_hrd(&vui.vui_ext_nal_hrd_parameters[i]));
        }

        bitstream_array.push(match vui.vui_ext_vcl_hrd_parameters_present_flag[i] {
            true => 1,
            false => 0,
        });
        if vui.vui_ext_vcl_hrd_parameters_present_flag[i] {
            bitstream_array.append(&mut encode_hrd(&vui.vui_ext_vcl_hrd_parameters[i]));
        }

        if vui.vui_ext_nal_hrd_parameters_present_flag[i]
            || vui.vui_ext_vcl_hrd_parameters_present_flag[i]
        {
            bitstream_array.push(match vui.vui_ext_low_delay_hrd_flag[i] {
                true => 1,
                false => 0,
            });
        }

        bitstream_array.push(match vui.vui_ext_pic_struct_present_flag[i] {
            true => 1,
            false => 0,
        });
    }
    vui.encoder_pretty_print();

    bitstream_array
}

fn encode_vui_mvc_parameters(vui: &MVCVUIParameters) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    // NOTE: may have issues converting large u32 to i32
    bitstream_array.append(&mut exp_golomb_encode_one(
        vui.vui_mvc_num_ops_minus1 as i32,
        false,
        0,
        false,
    ));

    for i in 0..=(vui.vui_mvc_num_ops_minus1 as usize) {
        bitstream_array.append(&mut generate_unsigned_binary(
            vui.vui_mvc_temporal_id[i] as u32,
            3,
        ));
        bitstream_array.append(&mut exp_golomb_encode_one(
            vui.vui_mvc_num_target_output_views_minus1[i] as i32,
            false,
            0,
            false,
        ));

        for j in 0..=(vui.vui_mvc_num_target_output_views_minus1[i] as usize) {
            bitstream_array.append(&mut exp_golomb_encode_one(
                vui.vui_mvc_view_id[i][j] as i32,
                false,
                0,
                false,
            ));
        }

        bitstream_array.push(match vui.vui_mvc_timing_info_present_flag[i] {
            true => 1,
            false => 0,
        });
        if vui.vui_mvc_timing_info_present_flag[i] {
            bitstream_array.append(&mut generate_unsigned_binary(
                vui.vui_mvc_num_units_in_tick[i] as u32,
                32,
            ));
            bitstream_array.append(&mut generate_unsigned_binary(
                vui.vui_mvc_time_scale[i] as u32,
                32,
            ));
            bitstream_array.push(match vui.vui_mvc_fixed_frame_rate_flag[i] {
                true => 1,
                false => 0,
            });
        }
        bitstream_array.push(match vui.vui_mvc_nal_hrd_parameters_present_flag[i] {
            true => 1,
            false => 0,
        });
        if vui.vui_mvc_nal_hrd_parameters_present_flag[i] {
            bitstream_array.append(&mut encode_hrd(&vui.vui_mvc_nal_hrd_parameters[i]));
        }

        bitstream_array.push(match vui.vui_mvc_vcl_hrd_parameters_present_flag[i] {
            true => 1,
            false => 0,
        });
        if vui.vui_mvc_vcl_hrd_parameters_present_flag[i] {
            bitstream_array.append(&mut encode_hrd(&vui.vui_mvc_vcl_hrd_parameters[i]));
        }

        if vui.vui_mvc_nal_hrd_parameters_present_flag[i]
            || vui.vui_mvc_vcl_hrd_parameters_present_flag[i]
        {
            bitstream_array.push(match vui.vui_mvc_low_delay_hrd_flag[i] {
                true => 1,
                false => 0,
            });
        }

        bitstream_array.push(match vui.vui_mvc_pic_struct_present_flag[i] {
            true => 1,
            false => 0,
        });
    }
    vui.encoder_pretty_print();

    bitstream_array
}
