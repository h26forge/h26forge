##
# Out of bounds mb_skip_run
def oob_mb_skip_run(ds):
    # Adjust the PPS to avoid some paths
    ds["ppses"][0]["entropy_coding_mode_flag"] = False

    # TODO: include some dependent syntax elements because of CABAC->CAVLC transformation
    for i in range(len(ds["slices"])):
        ds["slices"][i]["sd"]["mb_skip_run"] = [1024]*len(ds["slices"][i]["sd"]["macroblock_vec"])

    return ds

def modify_video(ds):
    return oob_mb_skip_run(ds)