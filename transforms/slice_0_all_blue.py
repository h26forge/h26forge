##
# make_first_frame_all_blue
#
# Makes the first frame to have an all blue chroma residue, nabbed from solid_blue.png
##
def slice_0_all_blue(ds):
    from helpers import set_cbp_chroma_and_luma, new_transform_block

    print("\t Setting the first frame to be all blue")
    ds["slices"][0]["sd"]["macroblock_vec"][0]["mb_type"] = "I16x16_2_1_0"
    ds = set_cbp_chroma_and_luma(0, 0, ds)

    ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_chroma_pred_mode"] = 0
    ds["slices"][0]["sd"]["macroblock_vec"][0]["mb_qp_delta"] = 0

    # set the luma values
    # if available then we just set the index
    if ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["available"]:
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["coded_block_flag"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["significant_coeff_flag"][0] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["last_significant_coeff_flag"][0] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["coeff_abs_level_minus1"][0] = 1112
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["coeff_sign_flag"][0] = True
    else:
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["available"] = True

        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["coded_block_flag"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["significant_coeff_flag"].append(True)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["last_significant_coeff_flag"].append(True)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["coeff_abs_level_minus1"].append(1112)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["intra_16x16_dc_level_transform_blocks"]["coeff_sign_flag"].append(True)

    if len(ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"]) < 2:
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"].append(new_transform_block())
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"].append(new_transform_block())

    if ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["available"]:
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["coded_block_flag"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["significant_coeff_flag"][0] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["last_significant_coeff_flag"][0] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["coeff_abs_level_minus1"][0] = 891
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["coeff_sign_flag"][0] = False
    else:
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["available"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["coded_block_flag"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["significant_coeff_flag"].append(True)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["last_significant_coeff_flag"].append(True)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["coeff_abs_level_minus1"].append(891)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][0]["coeff_sign_flag"].append(False)

    if ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["available"]:

        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["coded_block_flag"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["significant_coeff_flag"][0] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["last_significant_coeff_flag"][0] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["coeff_abs_level_minus1"][0] = 140
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["coeff_sign_flag"][0] = True
    else:
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["available"] = True

        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["coded_block_flag"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["significant_coeff_flag"].append(True)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["last_significant_coeff_flag"].append(True)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["coeff_abs_level_minus1"].append(140)
        ds["slices"][0]["sd"]["macroblock_vec"][0]["chroma_dc_level_transform_blocks"][1]["coeff_sign_flag"].append(True)

    # set all the other macroblocks
    for i in range(1, len(ds["slices"][0]["sd"]["macroblock_vec"])):
        ds["slices"][0]["sd"]["macroblock_vec"][i]["mb_type"] = "I16x16_2_0_0"
        ds = set_cbp_chroma_and_luma(0, i, ds)

        ds["slices"][0]["sd"]["macroblock_vec"][i]["intra_chroma_pred_mode"] = 0
        ds["slices"][0]["sd"]["macroblock_vec"][i]["mb_qp_delta"] = 0
        ds["slices"][0]["sd"]["macroblock_vec"][i]["intra_16x16_dc_level_transform_blocks"]["available"] = True
        ds["slices"][0]["sd"]["macroblock_vec"][i]["intra_16x16_dc_level_transform_blocks"]["coded_block_flag"] = False

    return ds

def modify_video(ds):
    return slice_0_all_blue(ds)