##
# remove_all_frame_residue
#
# Takes in a decoded H.264 bitstream
#
# Get rid of all residue by setting the coded_block_flag to false
# and total_coeff and trailing_ones of a CoeffToken to 0
##
def slice_all_remove_residue(ds):
    from helpers import set_cbp_chroma_and_luma
    print("\t Setting the residue values of the all frames to 0")
 
    for i in range(len(ds["slices"])):
        for j in range(len(ds["slices"][i]["sd"]["macroblock_vec"])):
            # zero out the coded_block_pattern
            ds["slices"][i]["sd"]["macroblock_vec"][j]["coded_block_pattern"] = 0
            ds["slices"][i]["sd"]["macroblock_vec"][j]["coded_block_pattern_chroma"] = 0
            ds["slices"][i]["sd"]["macroblock_vec"][j]["coded_block_pattern_luma"] = 0

            # in case the above are not consistent with the MbType, we run this function
            ds = set_cbp_chroma_and_luma(i, j, ds)
           
            # get rid of all coded_block_flags for luma components
            ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_16x16_dc_level_transform_blocks"]["coded_block_flag"] = False
            ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_16x16_dc_level_transform_blocks"]["coeff_token"]["total_coeff"] = 0
            ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_16x16_dc_level_transform_blocks"]["coeff_token"]["trailing_ones"] = 0

            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_16x16_ac_level_transform_blocks"])):
                # for CABAC
                ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_16x16_ac_level_transform_blocks"][k]["coded_block_flag"] = False
                # for CAVLC
                ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_16x16_ac_level_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_16x16_ac_level_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
           
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_4x4_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_4x4_transform_blocks"][k]["coded_block_flag"] = False
                # for CAVLC
                ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_4x4_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_4x4_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
           
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_8x8_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_8x8_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_8x8_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["luma_level_8x8_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
           
            # get rid of all coded_block_flags for Cb components
            ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_intra_16x16_dc_level_transform_blocks"]["coded_block_flag"] = False
            ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_intra_16x16_dc_level_transform_blocks"]["coeff_token"]["total_coeff"] = 0
            ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_intra_16x16_dc_level_transform_blocks"]["coeff_token"]["trailing_ones"] = 0

            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_intra_16x16_ac_level_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_intra_16x16_ac_level_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_intra_16x16_ac_level_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_intra_16x16_ac_level_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
               
           
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_4x4_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_4x4_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_4x4_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_4x4_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
               
           
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_8x8_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_8x8_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_8x8_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cb_level_8x8_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
           
            # get rid of all coded_block_flags for Cr components
            ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_intra_16x16_dc_level_transform_blocks"]["coded_block_flag"] = False
            ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_intra_16x16_dc_level_transform_blocks"]["coeff_token"]["total_coeff"] = 0
            ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_intra_16x16_dc_level_transform_blocks"]["coeff_token"]["trailing_ones"] = 0

            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_intra_16x16_ac_level_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_intra_16x16_ac_level_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_intra_16x16_ac_level_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_intra_16x16_ac_level_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
           
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_4x4_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_4x4_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_4x4_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_4x4_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
           
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_8x8_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_8x8_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_8x8_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["cr_level_8x8_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0
           
            # get rid of chroma DC components
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_dc_level_transform_blocks"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_dc_level_transform_blocks"][k]["coded_block_flag"] = False
                ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_dc_level_transform_blocks"][k]["coeff_token"]["total_coeff"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_dc_level_transform_blocks"][k]["coeff_token"]["trailing_ones"] = 0

            # get rid of chroma AC components
            for k in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_ac_level_transform_blocks"])):
                for l in range(len(ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_ac_level_transform_blocks"][k])):
                    ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_ac_level_transform_blocks"][k][l]["coded_block_flag"] = False
                    ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_ac_level_transform_blocks"][k][l]["coeff_token"]["total_coeff"] = 0
                    ds["slices"][i]["sd"]["macroblock_vec"][j]["chroma_ac_level_transform_blocks"][k][l]["coeff_token"]["trailing_ones"] = 0

    return ds

def modify_video(ds):
    return slice_all_remove_residue(ds)