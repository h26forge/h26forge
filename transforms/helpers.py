import copy

###############################
# Basic struct definitions
###############################

DEFAULT_TRANSFORM_BLOCK = {
        'available': False,
        'significant_coeff_flag': [],
        'last_significant_coeff_flag': [],
        'coeff_sign_flag': [],
        'coeff_abs_level_minus1': [],
        'coded_block_flag': True,
        'coeff_token' : {
            'total_coeff' : 0,
            'trailing_ones' : 0,
            'n_c' : 0,
        },
        'trailing_ones_sign_flag' : [],
        'level_prefix' : [],
        'level_suffix' : [],
        'total_zeros' : 0,
        'run_before' : []
    }

DEFAULT_HRD_PARAMETERS = {
    "cpb_cnt_minus1": 0,
    "bit_rate_scale": 0,
    "cpb_size_scale": 0,
    "bit_rate_value_minus1": [0],
    "cpb_size_values_minus1": [0],
    "cbr_flag": [False],
    "initial_cpb_removal_delay_length_minus1": 0,
    "cpb_removal_delay_length_minus1": 0,
    "dpb_output_delay_length_minus1": 0,
    "time_offset_length": 0
}

DEFAULT_VUI_PARAMETERS = {
    "aspect_ratio_info_present_flag": False,
    "aspect_ratio_idc": 0,
    "sar_width": 0,
    "sar_height": 0,
    "overscan_info_present_flag": False,
    "overscan_appropriate_flag": False,
    "video_signal_type_present_flag": False,
    "video_format": 0,
    "video_full_range_flag": False,
    "colour_description_present_flag": False,
    "colour_primaries": 0,
    "transfer_characteristics": 0,
    "matrix_coefficients": 0,
    "chroma_loc_info_present_flag": False,
    "chroma_sample_loc_type_top_field": 0,
    "chroma_sample_loc_type_bottom_field": 0,
    "timing_info_present_flag": False,
    "num_units_in_tick": 0,
    "time_scale": 0,
    "fixed_frame_rate_flag": False,
    "nal_hrd_parameters_present_flag": False,
    "nal_hrd_parameters": copy.deepcopy(DEFAULT_HRD_PARAMETERS),
    "vcl_hrd_parameters_present_flag": False,
    "vcl_hrd_parameters": copy.deepcopy(DEFAULT_HRD_PARAMETERS),
    "low_delay_hrd_flag": False,
    "pic_struct_present_flag": False,
    "bitstream_restriction_flag": False,
    "motion_vectors_over_pic_boundaries_flag": False,
    "max_bytes_per_pic_denom": 0,
    "max_bits_per_mb_denom": 0,
    "log2_max_mv_length_horizontal": 0,
    "log2_max_mv_length_vertical": 0,
    "max_num_reorder_frames": 0,
    "max_dec_frame_buffering": 0
}

DEFAULT_SPS = {
    "available": True,
    "profile_idc": 100,
    "constraint_set0_flag": False,
    "constraint_set1_flag": False,
    "constraint_set2_flag": False,
    "constraint_set3_flag": False,
    "constraint_set4_flag": False,
    "constraint_set5_flag": False,
    "reserved_zero_2bits": 0,
    "level_idc": 31,
    "seq_parameter_set_id": 0,
    "chroma_format_idc": 1,
    "separate_colour_plane_flag": False,
    "bit_depth_luma_minus8": 0,
    "bit_depth_chroma_minus8": 0,
    "qpprime_y_zero_transform_bypass_flag": False,
    "seq_scaling_matrix_present_flag": False,
    "seq_scaling_list_present_flag": [],
    "use_default_scaling_matrix_4x4": [],
    "use_default_scaling_matrix_8x8": [],
    "delta_scale_4x4": [],
    "scaling_list_4x4": [],
    "scaling_list_8x8": [],
    "delta_scale_8x8": [],
    "log2_max_frame_num_minus4": 0,
    "pic_order_cnt_type": 0,
    "log2_max_pic_order_cnt_lsb_minus4": 2,
    "delta_pic_order_always_zero_flag": True,
    "offset_for_non_ref_pic": 0,
    "offset_for_top_to_bottom_field": 0,
    "num_ref_frames_in_pic_order_cnt_cycle": 0,
    "offset_for_ref_frame": [],
    "max_num_ref_frames": 0,
    "gaps_in_frame_num_value_allowed_flag": True,
    "pic_width_in_mbs_minus1": 9,
    "pic_height_in_map_units_minus1": 6,
    "frame_mbs_only_flag": True,
    "mb_adaptive_frame_field_flag": False,
    "direct_8x8_inference_flag": True,
    "frame_cropping_flag": False,
    "frame_crop_left_offset": 0,
    "frame_crop_right_offset": 0,
    "frame_crop_top_offset": 0,
    "frame_crop_bottom_offset": 0,
    "vui_parameters_present_flag": False,
    "vui_parameters": copy.deepcopy(DEFAULT_VUI_PARAMETERS),
}

DEFAULT_MVC_SPS_EXTENSION = {
    "num_views_minus1" : 0,
    "view_id" : [],
    "num_anchor_refs_l0" : [],
    "anchor_refs_l0" : [],
    "num_anchor_refs_l1" : [],
    "anchor_refs_l1" : [],
    "num_non_anchor_refs_l0" : [],
    "non_anchor_refs_l0" : [],
    "num_non_anchor_refs_l1" : [],
    "non_anchor_refs_l1" : [],
    "num_level_values_signalled_minus1" : 0,
    "level_idc" : [],
    "num_applicable_ops_minus1" : [],
    "applicable_op_temporal_id" : [],
    "applicable_op_num_target_views_minus1" : [],
    "applicable_op_target_view_id" : [],
    "applicable_op_num_views_minus1" : [],
    "mfc_format_idc" : 0,
    "default_grid_position_flag" : False,
    "view0_grid_position_x" : 0,
    "view0_grid_position_y" : 0,
    "view1_grid_position_x" : 0,
    "view1_grid_position_y" : 0,
    "rpu_filter_enabled_flag" : False,
    "rpu_field_processing_flag" : False,
}

DEFAULT_SUBSET_SPS = {
    "sps" : copy.deepcopy(DEFAULT_SPS),
    "sps_svc" : {},     # TODO
    "svc_vui_parameters_present_flag" : False,
    "svc_vui" : {},     # TODO
    "bit_equal_to_one" : 1,
    "sps_mvc" : copy.deepcopy(DEFAULT_MVC_SPS_EXTENSION),
    "mvc_vui_parameters_present_flag" : False,
    "mvc_vui" : {},     # TODO
    "sps_mvcd" : {},    # TODO
    "sps_3davc" : {},   # TODO
    "additional_extension2_flag" : [],
}

DEFAULT_NALU_ELEMENT = {
        "longstartcode": True,
        "content": []
    }

DEFAULT_NALU_HEADER = {
        "forbidden_zero_bit": 0,
        "nal_ref_idc": 2,
        "nal_unit_type": 1,
        "svc_extension_flag": False,
        "svc_extension": {
            "idr_flag": False,
            "priority_id": 0,
            "no_inter_layer_pred_flag": False,
            "dependency_id": 0,
            "quality_id": 0,
            "temporal_id": 0,
            "use_ref_base_pic_flag": False,
            "discardable_flag": False,
            "output_flag": False,
            "reserved_three_2bits": 0
        },
        "avc_3d_extension_flag": False,
        "avc_3d_extension": {
            "view_idx": 0,
            "depth_flag": False,
            "non_idr_flag": False,
            "temporal_id": 0,
            "anchor_pic_flag": False,
            "inter_view_flag": False
        },
        "mvc_extension": {
            "non_idr_flag": False,
            "priority_id": 0,
            "view_id": 0,
            "temporal_id": 0,
            "anchor_pic_flag": False,
            "inter_view_flag": False,
            "reserved_one_bit": False
        }
    }

DEFAULT_SEI_NALU = {
    "payload_type": [0],
    "payload_size": [0],
    "payload": [{
        "buffering_period": {
            "seq_parameter_set_id": 0,
            "nal_initial_cpb_removal_delay": [],
            "nal_initial_cpb_removal_delay_offset": [],
            "vcl_initial_cpb_removal_delay": [],
            "vcl_initial_cpb_removal_delay_offset": []
        },
        "pic_timing": {
            "cpb_removal_delay": 0,
            "dpb_output_delay": 0,
            "pic_struct": 0,
            "clock_timestamp_flag": [],
            "ct_type": [],
            "nuit_field_based_flag": [],
            "counting_type": [],
            "full_timestamp_flag": [],
            "discontinuity_flag": [],
            "cnt_dropped_flag": [],
            "n_frames": [],
            "seconds_value": [],
            "minutes_value": [],
            "hours_value": [],
            "seconds_flag": [],
            "minutes_flag": [],
            "hours_flag": [],
            "time_offset": []
        },
        "unregistered_user_data": {
            "uuid_iso_iec_11578": [],
            "user_data_apple1": {
                "mystery_param1": 0
            },
            "user_data_apple2": {
                "mystery_param1": 0,
                "mystery_param2": 0,
                "mystery_param3": 0,
                "mystery_param4": 0,
                "mystery_param5": 0,
                "mystery_param6": 0,
                "mystery_param7": 0,
                "mystery_param8": 0
            },
            "user_data_payload_byte": []
        },
        "recovery_point": {
            "recovery_frame_cnt": 0,
            "exact_match_flag": False,
            "broken_link_flag": False,
            "changing_slice_group_idc": 0
        },
        "film_grain_characteristics": {
            "film_grain_characteristics_cancel_flag": False,
            "film_grain_model_id": 0,
            "separate_colour_description_present_flag": False,
            "film_grain_bit_depth_luma_minus8": 0,
            "film_grain_bit_depth_chroma_minus8": 0,
            "film_grain_full_range_flag": False,
            "film_grain_colour_primaries": 0,
            "film_grain_transfer_characteristics": 0,
            "film_grain_matrix_coefficients": 0,
            "blending_mode_id": 0,
            "log2_scale_factor": 0,
            "comp_model_present_flag": [],
            "num_intensity_intervals_minus1": [],
            "num_model_values_minus1": [],
            "intensity_interval_lower_bound": [],
            "intensity_interval_upper_bound": [],
            "comp_model_value": [],
            "film_grain_characteristics_repetition_period": 0
        },
        "frame_packing": {}
    }]
}

#######################################
# Update Dependent Syntax Elements
#######################################

def set_cbp_chroma_and_luma(slice_idx, mb_idx, ds):
    '''Set the Coded Block Pattern default values for a MB Type
    '''
    mbtype = ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['mb_type']
    if mbtype == 'I16x16_0_0_0' or mbtype == 'I16x16_1_0_0' or mbtype == 'I16x16_2_0_0' or mbtype == 'I16x16_3_0_0':
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_luma'] = 0
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_chroma'] = 0
    if mbtype == 'I16x16_0_1_0' or mbtype == 'I16x16_1_1_0' or mbtype == 'I16x16_2_1_0' or mbtype == 'I16x16_3_1_0':
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_luma'] = 0
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_chroma'] = 1
    if mbtype == 'I16x16_0_2_0' or mbtype == 'I16x16_1_2_0' or mbtype == 'I16x16_2_2_0' or mbtype == 'I16x16_3_2_0':
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_luma'] = 0
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_chroma'] = 2
    if mbtype == 'I16x16_0_0_1' or mbtype == 'I16x16_1_0_1' or mbtype == 'I16x16_2_0_1' or mbtype == 'I16x16_3_0_1':
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_luma'] = 15
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_chroma'] = 0
    if mbtype == 'I16x16_0_1_1' or mbtype == 'I16x16_1_1_1' or mbtype == 'I16x16_2_1_1' or mbtype == 'I16x16_3_1_1':
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_luma'] = 15
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_chroma'] = 1
    if mbtype == 'I16x16_0_2_1' or mbtype == 'I16x16_1_2_1' or mbtype == 'I16x16_2_2_1' or mbtype == 'I16x16_3_2_1':
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_luma'] = 15
        ds['slices'][slice_idx]['sd']['macroblock_vec'][mb_idx]['coded_block_pattern_chroma'] = 2
    return ds

#######################################
# Get Video Properties
#######################################

def is_slice_type(slice_num, slice_letter):
    '''Check if a Slice number is a target Slice letter type
    '''
    sl = slice_letter.upper()
    if sl == 'P':
        return slice_num % 5 == 0 # 0 or 5
    elif sl == 'B':
        return slice_num % 5 == 1 # 1 or 6
    elif sl == 'I':
        return slice_num % 5 == 2 # 2 or 7
    elif sl == 'SP':
        return slice_num % 5 == 3 # 3 or 8
    elif sl == 'SI':
        return slice_num % 5 == 4 # 4 or 9
    return False

def get_chroma_width_and_height(sps_idx, ds):
    '''Calculate and return the Chroma Width and Height for a Slice
    '''
    sub_width_c = 0
    sub_height_c = 0

    cfidc = ds["spses"][sps_idx]["chroma_format_idc"]
    scpf = ds["spses"][sps_idx]["separate_colour_plane_flag"]

    if cfidc == 1:
        if not scpf:
            sub_width_c = 2
            sub_height_c = 2
    elif cfidc == 2:
        if not scpf:
            sub_width_c = 2
            sub_height_c = 1
    elif cfidc == 3:
        if not scpf:
            sub_width_c = 1
            sub_height_c = 1
    mb_width_c = 0
    mb_height_c = 0
    if sub_height_c > 0 and sub_width_c > 0:
        mb_height_c = 16 / sub_height_c
        mb_width_c = 16 / sub_width_c
    return (int(mb_width_c), int(mb_height_c))

#######################################
# Return Blank Canvas
#######################################

def new_transform_block():
    '''Return a TransformBlock for luma/chroma residue
    '''
    return copy.deepcopy(DEFAULT_TRANSFORM_BLOCK)

def new_sps():
    '''Return a new Sequence Parameter Set
    '''
    return copy.deepcopy(DEFAULT_SPS)

def new_vui_parameter():
    '''Return a new VUI Parameter Set
    '''
    return copy.deepcopy(DEFAULT_VUI_PARAMETERS)

def new_hrd_parameter():
    '''Return a new HRD Parameter Set
    '''
    return copy.deepcopy(DEFAULT_HRD_PARAMETERS)

def new_subset_sps():
    '''Return a new Subset Sequence Parameter Set
    '''
    return copy.deepcopy(DEFAULT_SUBSET_SPS)

#######################################
# Insert New Syntax Elements
#######################################

def add_sei_nalu(ds):
    '''Add an SEI NALU to the Decoded Syntax Elements
    '''
    # Add NALU Element
    ds["nalu_elements"].append(copy.deepcopy(DEFAULT_NALU_ELEMENT))
    # Add NALU Header
    nalu_header = copy.deepcopy(DEFAULT_NALU_HEADER)
    nalu_header['nal_unit_type'] = 6 # SEI NALU
    ds["nalu_headers"].append(nalu_header)
    # Add SEI struct
    ds["seis"].append(copy.deepcopy(DEFAULT_SEI_NALU))
    return ds

def add_sei_nalu_at_position(ds, idx):
    '''Add an SEI NALU to the Decoded Syntax Elements at Position idx
       'idx' is only for NALU Elements and NALU Headers; SEI is still appended
    '''
    # Add NALU Element
    ds["nalu_elements"].insert(idx, copy.deepcopy(DEFAULT_NALU_ELEMENT))
    # Add NALU Header
    nalu_header = copy.deepcopy(DEFAULT_NALU_HEADER)
    nalu_header['nal_unit_type'] = 6 # SEI NALU
    nalu_header['nal_ref_idc'] = 1 # The standard for SEIs
    ds["nalu_headers"].insert(idx, nalu_header)
    # Add SEI struct
    ds["seis"].append(copy.deepcopy(DEFAULT_SEI_NALU))
    return ds

def clone_and_append_existing_slice(ds, nalu_idx, slice_idx):
    '''Clone a slice at slice_idx and append it to the end of the video
    '''
    # Copy NALU element
    ds["nalu_elements"].append(copy.deepcopy(ds["nalu_elements"][nalu_idx]))
    # Copy Nalu Header
    ds["nalu_headers"].append(copy.deepcopy(ds["nalu_headers"][nalu_idx]))
    # Copy Slice
    ds["slices"].append(copy.deepcopy(ds["slices"][slice_idx]))
    return ds

