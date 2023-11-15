##
# luma_chroma_thief_8x8
#
# Make the first slice into an luma chroma thief video using 8x8 techniques
#
##
def luma_chroma_thief_8x8(ds):
    from slice_n_remove_residue import slice_n_remove_residue

    print("8x8 vertical luma chroma thief!")

    ds = slice_n_remove_residue(0, ds)

    # disable deblocking filter to remove stolen value post-processing
    ds["ppses"][0]["deblocking_filter_control_present_flag"] = True
    ds["slices"][0]["sh"]["disable_deblocking_filter_idc"] = 1

    for i in range(len(ds["slices"][0]["sd"]["macroblock_vec"])):
        ds["slices"][0]["sd"]["macroblock_vec"][i]["mb_type"] = "INxN"
        ds["slices"][0]["sd"]["macroblock_vec"][i]["transform_size_8x8_flag"] = True

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
        # |           |           |
        # |     0     |     1     |
        # |           |           |
        # |___________|___________|
        # |           |           |
        # |     2     |     3     |
        # |           |           |
        # _________________________
        #
        # Predictions are based on the neighbors above and to the left. If one of these is not
        # not available then the prediction will default to 2. In these cases we will signal
        # 0. Whenever the predicted value is supposed to be 0, then we will take the predicted value

        # this mode should be vertical
        if i == 0:
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][0] = False
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][1] = False
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][2] = False
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][3] = True
        elif i < (ds["spses"][0]["pic_width_in_mbs_minus1"] + 1): # indices 1 to 8
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][0] = False
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][1] = True
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][2] = False
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][3] = True
        elif i % (ds["spses"][0]["pic_width_in_mbs_minus1"] + 1) == 0: # left most edge
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][0] = False
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][1] = True
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][2] = False
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"][3] = True
        else: # internal macroblock
            ds["slices"][0]["sd"]["macroblock_vec"][i]["prev_intra8x8_pred_mode_flag"] = 4 * [True]
        # should be horizontal - taken from Table 8-3
        ds["slices"][0]["sd"]["macroblock_vec"][i]["rem_intra8x8_pred_mode"] = 4 * [1]
        # horizontal intra chroma prediction
        ds["slices"][0]["sd"]["macroblock_vec"][i]["intra_chroma_pred_mode"] = 1
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern"] = 0
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern_chroma"] = 0
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern_luma"] = 0


    return ds


def modify_video(ds):
    return luma_chroma_thief_8x8(ds)