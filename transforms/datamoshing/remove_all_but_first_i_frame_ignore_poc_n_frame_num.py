##
# remove_i_frames
#
# Remove every other I frame to get the different data mosh effects. Also getting mosh effects from the POC/frame number
#
##
def remove_i_frames(ds):
    from helpers import is_slice_type
    import copy

    print("\t Removing all but first I frame")

    slice_idx = 0
    start_length = len(ds["nalu_headers"])
    seen_first_i_frame = False
    # we remove every other I slice
    i = 0
    while i < len(ds["nalu_headers"]):
        # check if it's a video coded layer
        nal_unit_type = ds["nalu_headers"][i]['nal_unit_type']
        if nal_unit_type == 1 or nal_unit_type == 5:
            if is_slice_type(ds["slices"][slice_idx]["sh"]["slice_type"], 'I') and seen_first_i_frame:
                # copy over header elements
                ds["nalu_headers"].pop(i)
                ds["nalu_elements"].pop(i)

                # copy over slices
                ds["slices"].pop(slice_idx)
            else:
                if is_slice_type(ds["slices"][slice_idx]["sh"]["slice_type"], 'I'):
                    seen_first_i_frame = True
               
                slice_idx += 1
        i += 1

    print("\t Removed " + str(start_length - i) + " I slices")
    return ds

def modify_video(ds):
    return remove_i_frames(ds)