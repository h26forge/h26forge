##
# luma_chroma_thief_16x16
#
# Make the first slice into an luma chroma thief video using 16x16 techniques
#
##
def luma_chroma_thief_16x16(ds):
    from slice_n_remove_residue import slice_n_remove_residue
    from helpers import set_cbp_chroma_and_luma

    print("16x16 horizontal luma chroma thief!")

    ds = slice_n_remove_residue(0, ds)

    # disable deblocking filter to remove stolen value post-processing
    ds["ppses"][0]["deblocking_filter_control_present_flag"] = True
    ds["slices"][0]["sh"]["disable_deblocking_filter_idc"] = 1

    for i in range(len(ds["slices"][0]["sd"]["macroblock_vec"])):
        # 16x16 horizontal luma prediction - taken from Table 8-4 from the spec
        ds["slices"][0]["sd"]["macroblock_vec"][i]["mb_type"] = "I16x16_1_0_0"
        # make sure these values are correct for re-encoding
        ds["slices"][0]["sd"]["macroblock_vec"][i]["coded_block_pattern"] = 0
        ds = set_cbp_chroma_and_luma(0, i, ds)

        # horizontal intra chroma prediction - taken from table 7-16
        ds["slices"][0]["sd"]["macroblock_vec"][i]["intra_chroma_pred_mode"] = 1


    return ds


def modify_video(ds):
    return luma_chroma_thief_16x16(ds)