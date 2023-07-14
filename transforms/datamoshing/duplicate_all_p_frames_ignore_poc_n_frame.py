##
# duplicate_p_frames
#
# Aims to get a bloom effect described here: http://datamoshing.com/
#
# Also getting mosh effects from the POC/frame number
##
def duplicate_p_frames(ds):
    from helpers import is_slice_type
    import copy

    dup_amount = 4
    print("\t Duplicating all P frames " + str(dup_amount) + " times")

    #frame_num = 0
    #pic_order_cnt_lsb = 0

    slice_idx = 0
    start_length = len(ds["nalu_headers"])
    # we insert new slices then skip over inserted oens
    i = 0
    while True:
        # check if it's a video coded layer
        nal_unit_type = ds["nalu_headers"][i]['nal_unit_type']
        if nal_unit_type == 1 or nal_unit_type == 5:
            # update the counts of the current slices
            #ds["slices"][slice_idx]["sh"]["frame_num"] = frame_num
            #ds["slices"][slice_idx]["sh"]["pic_order_cnt_lsb"] = pic_order_cnt_lsb

            #frame_num += 1
            #pic_order_cnt_lsb += 2
            #print("slice_idx: " + str(slice_idx))

            og_slice_idx = slice_idx
            # if we have a P slice then duplicate and put it into new_ds
            if is_slice_type(ds["slices"][slice_idx]["sh"]["slice_type"], 'P'):
                for _ in range(dup_amount):
                    slice_idx += 1

                    # copy over header elements
                    ds["nalu_headers"].insert(i, ds["nalu_headers"][i])
                    ds["nalu_elements"].insert(i, ds["nalu_elements"][i])
                    ds["video_params"].insert(i, ds["video_params"][i])

                    # copy over slices
                    ds["slices"].insert(slice_idx, copy.deepcopy(ds["slices"][og_slice_idx]))

                    #ds["slices"][slice_idx]["sh"]["frame_num"] = frame_num
                    #ds["slices"][slice_idx]["sh"]["pic_order_cnt_lsb"] = pic_order_cnt_lsb
                    #frame_num += 1
                    #pic_order_cnt_lsb += 2
                i += dup_amount

            slice_idx += 1
        i += 1
        if i == len(ds["nalu_headers"]):
            break
    print("\t Added " + str(i - start_length) + " slices")
    return ds

def modify_video(ds):
    return duplicate_p_frames(ds)