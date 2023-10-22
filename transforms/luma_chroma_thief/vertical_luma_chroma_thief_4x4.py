##
# luma_chroma_thief_4x4
#
# Make the first slice into an luma chroma thief video using 4x4 techniques
#
##
def luma_chroma_thief_4x4(ds):
    from slice_n_remove_residue import slice_n_remove_residue

    print("4x4 vertical luma chroma thief!")

    ds = slice_n_remove_residue(0, ds)

    # disable deblocking filter to remove stolen value post-processing
    ds["ppses"][0]["deblocking_filter_control_present_flag"] = True
    ds["slices"][0]["sh"]["disable_deblocking_filter_idc"] = 1

    for i in range(len(ds["slices"][0]["sd"]["macroblock_vec"])):
        ds["slices"][0]["sd"]["macroblock_vec"][i]["mb_type"] = "INxN"
        ds["slices"][0]["sd"]["macroblock_vec"][i]["transform_size_8x8_flag"] = False

        # It will try to predict what mode to use. If the signaled mode is less than the
        # predicted mode then the decoder will use the signaled mode. Otherwise, it will take
        # the signaled value + 1. 
        # 
        # We need to decide when to take the predicted mode to ensure all the values are vertical (0)
        # and not sometimes horizontal (1)
        #
        # The indices correspond to the following figure:
        #
        # _________________________
        # |  0  |  1  |  4  |  5  |
        # |-----------|-----------|
        # |  2  |  3  |  6  |  7  |
        # |___________|___________|
        # |  8  |  9  |  12 |  13 |
        # |-----------|-----------|
        # |  10 |  11 |  14 |  15 |
        # _________________________
        #
        # Predictions are based on the neighbors above and to the left. If one of these is not
        # not available then the prediction will default to 2. In these cases we will signal
        # 0. Whenever the predicted value is supposed to be 0, then we will take the predicted value
        
        if i == 0:
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][0] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][1] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][2] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][3] = True # take the neighbor values 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][4] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][5] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][6] = True
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][7] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][8] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][9] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][10] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][11] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][12] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][13] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][14] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][15] = True 
        elif i < (ds["spses"][0]["pic_width_in_mbs_minus1"] + 1): # indices 1 to 8
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][0] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][1] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][2] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][3] = True # take the neighbor values 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][4] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][5] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][6] = True
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][7] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][8] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][9] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][10] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][11] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][12] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][13] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][14] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][15] = True 
        elif i % (ds["spses"][0]["pic_width_in_mbs_minus1"] + 1) == 0: # left most edge
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][0] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][1] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][2] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][3] = True # take the neighbor values 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][4] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][5] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][6] = True
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][7] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][8] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][9] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][10] = False 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][11] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][12] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][13] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][14] = True 
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"][15] = True 
        else: # internal video
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra4x4_pred_mode_flag"] = 16 * [True]


        # this mode should be vertical
        ds["slices"][0]["sd"]["macroblock_vec"][i]["rem_intra4x4_pred_mode"] = 16 * [0]
        # vertical intra chroma prediction
        ds["slices"][0]["sd"]["macroblock_vec"][i]["intra_chroma_pred_mode"] = 2
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern"] = 0
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern_chroma"] = 0
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern_luma"] = 0


    return ds


def modify_video(ds):
    return luma_chroma_thief_4x4(ds)