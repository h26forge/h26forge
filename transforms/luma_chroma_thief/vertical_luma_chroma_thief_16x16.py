##
# luma_chroma_thief_16x16
#
# Make the first slice into an luma chroma thief video using 16x16 techniques
#
##
def luma_chroma_thief_16x16(ds):
    from slice_n_remove_residue import slice_n_remove_residue
    from helpers import set_cbp_chroma_and_luma

    print("16x16 vertical luma chroma thief!")

    ds = slice_n_remove_residue(0, ds)

    # clean up the SPS
    ds["spses"][0]["profile_idc"] = 100
    ds["spses"][0]["constraint_set0_flag"] = False
    ds["spses"][0]["constraint_set1_flag"] = False
    ds["spses"][0]["constraint_set2_flag"] = False
    ds["spses"][0]["constraint_set3_flag"] = False
    ds["spses"][0]["constraint_set4_flag"] = False
    ds["spses"][0]["constraint_set5_flag"] = False
    ds["spses"][0]["reserved_zero_2bits"] = 0
    ds["spses"][0]["level_idc"] = 51
    ds["spses"][0]["seq_parameter_set_id"] = 0
    ds["spses"][0]["chroma_format_idc"] = 1
    ds["spses"][0]["bit_depth_luma_minus8"] = 0
    ds["spses"][0]["bit_depth_chroma_minus8"] = 0
    ds["spses"][0]["qpprime_y_zero_transform_bypass_flag"] = False
    ds["spses"][0]["seq_scaling_matrix_present_flag"] = False
    ds["spses"][0]["log2_max_frame_num_minus4"] = 0
    ds["spses"][0]["pic_order_cnt_type"] = 0
    ds["spses"][0]["log2_max_pic_order_cnt_lsb_minus4"] = 0
    ds["spses"][0]["max_num_ref_frames"] = 0
    ds["spses"][0]["gaps_in_frame_num_value_allowed_flag"] = False
    #ds["spses"][0]["pic_width_in_mbs_minus1"] = 127 # max works: 127
    #ds["spses"][0]["pic_height_in_map_units_minus1"] = False
    ds["spses"][0]["frame_mbs_only_flag"] = True
    ds["spses"][0]["direct_8x8_inference_flag"] = True
    ds["spses"][0]["frame_cropping_flag"] = False
    #ds["spses"][0]["frame_crop_left_offset"] = False
    #ds["spses"][0]["frame_crop_right_offset"] = False
    #ds["spses"][0]["frame_crop_top_offset"] = False
    #ds["spses"][0]["frame_crop_bottom_offset"] = False
    ds["spses"][0]["vui_parameters_present_flag"] = False


    ds["ppses"][0]["pic_parameter_set_id"] = 0
    ds["ppses"][0]["seq_parameter_set_id"] = 0
    ds["ppses"][0]["entropy_coding_mode_flag"] = True
    ds["ppses"][0]["bottom_field_pic_order_in_frame_present_flag"] = False
    ds["ppses"][0]["num_slice_groups_minus1"] = 0
    ds["ppses"][0]["num_ref_idx_l0_default_active_minus1"] = 0
    ds["ppses"][0]["num_ref_idx_l1_default_active_minus1"] = 0
    ds["ppses"][0]["weighted_pred_flag"] = False
    ds["ppses"][0]["weighted_bipred_idc"] = 0
    ds["ppses"][0]["pic_init_qp_minus26"] = 0
    ds["ppses"][0]["pic_init_qs_minus26"] = 0
    ds["ppses"][0]["chroma_qp_index_offset"] = 0
    ds["ppses"][0]["deblocking_filter_control_present_flag"] = True
    ds["ppses"][0]["constrained_intra_pred_flag"] = False
    ds["ppses"][0]["redundant_pic_cnt_present_flag"] = False
    ds["ppses"][0]["transform_8x8_mode_flag"] = False
    ds["ppses"][0]["pic_scaling_matrix_present_flag"] = False
    ds["ppses"][0]["second_chroma_qp_index_offset"] = 0


    ds["slices"][0]["sh"]["first_mb_in_slice"] = 0
    ds["slices"][0]["sh"]["slice_type"] = 2
    ds["slices"][0]["sh"]["pic_parameter_set_id"] = 0
    ds["slices"][0]["sh"]["frame_num"] = 0
    ds["slices"][0]["sh"]["idr_pic_id"] = 0
    ds["slices"][0]["sh"]["pic_order_cnt_lsb"] = 0
    ds["slices"][0]["sh"]["no_output_of_prior_pics_flag"] = False
    ds["slices"][0]["sh"]["long_term_reference_flag"] = False
    ds["slices"][0]["sh"]["slice_qp_delta"] = 0
    ds["slices"][0]["sh"]["disable_deblocking_filter_idc"] = 1

    # disable deblocking filter to remove stolen value post-processing
    ds["ppses"][0]["deblocking_filter_control_present_flag"] = True
    ds["slices"][0]["sh"]["disable_deblocking_filter_idc"] = 1

    for i in range(len(ds["slices"][0]["sd"]["macroblock_vec"])):
        # 16x16 vertical luma prediction
        ds["slices"][0]["sd"]["macroblock_vec"][i]["mb_type"] = "I16x16_0_0_0"
        # make sure these values are correct for re-encoding
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern"] = 0
        ds = set_cbp_chroma_and_luma(0, i, ds)

        # vertical intra chroma prediction
        ds["slices"][0]["sd"]["macroblock_vec"][i]["intra_chroma_pred_mode"] = 2


    return ds


def modify_video(ds):
    return luma_chroma_thief_16x16(ds)