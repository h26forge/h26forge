##
# remove_i_frames
#
# Remove every other I frame to get the different data mosh effects. Also getting mosh effects from the POC/frame number
#
##
def remove_i_frames(ds):
    from helpers import is_slice_type

    print("\t Removing every other I frame")

    frame_num = 0
    pic_order_cnt_lsb = 0

    slice_idx = 0
    start_length = len(ds["nalu_headers"])
    slice_i_count = 0
    # we remove every other I slice
    i = 0
    while i < len(ds["nalu_headers"]):
        # check if it's a video coded layer
        nal_unit_type = ds["nalu_headers"][i]['nal_unit_type']
        if nal_unit_type == 1 or nal_unit_type == 5:
            if is_slice_type(ds["slices"][slice_idx]["sh"]["slice_type"], 'I') and slice_i_count % 2 == 1:
                # copy over header elements
                ds["nalu_headers"].pop(i)
                ds["nalu_elements"].pop(i)
                ds["video_params"].pop(i)

                # copy over slices
                ds["slices"].pop(slice_idx)
                slice_i_count += 1
            else:
                ds["slices"][slice_idx]["sh"]["frame_num"] = frame_num
                ds["slices"][slice_idx]["sh"]["pic_order_cnt_lsb"] = pic_order_cnt_lsb
                frame_num += 1
                pic_order_cnt_lsb += 2
                if is_slice_type(ds["slices"][slice_idx]["sh"]["slice_type"], 'I'):
                    slice_i_count += 1
               
                slice_idx += 1
        i += 1

    print("\t Removed " + str(start_length - i) + " I slices")
    return ds

def modify_video(ds):
    return remove_i_frames(ds)