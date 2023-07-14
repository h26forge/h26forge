def clear_out_frames(ds):
    from helpers import is_slice_type, new_vui_parameter
    from slice_all_remove_residue import remove_all_frame_residue

    # Remove VUI information
    for i in range(len(ds["spses"])):
        ds["spses"][i]["vui_parameters_present_flag"] = False
        # this is to prevent dependent variables from using stale data in the encoder
        ds["spses"][i]["vui_parameters"] = new_vui_parameter()

    # Ignore Inter by setting mb_skip_flag to True
    for i in range(len(ds["slices"])):
        slice_type = ds["slices"][i]["sh"]["slice_type"]

        if is_slice_type(slice_type, "I"):
            for j in range(len(ds["slices"][i]["sd"]["macroblock_vec"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["mb_type"] = "INxN"
                ds["slices"][i]["sd"]["macroblock_vec"][j]["transform_size_8x8_flag"] = True
                ds["slices"][i]["sd"]["macroblock_vec"][j]["intra_chroma_pred_mode"] = 0
                ds["slices"][i]["sd"]["macroblock_vec"][j]["prev_intra8x8_pred_mode_flag"] = [True]*4

        if is_slice_type(slice_type, "P") or is_slice_type(slice_type, "B"):
            for j in range(len(ds["slices"][i]["sd"]["macroblock_vec"])):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["mb_skip_flag"] = True

    # Now that we've updated the MB Type, 
    # remove all the residue information from each frame
    ds = remove_all_frame_residue(ds)

    return ds


def modify_video(ds):
    return clear_out_frames(ds)